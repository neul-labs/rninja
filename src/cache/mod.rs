mod blob;
mod config;
mod entry;
mod hasher;
pub mod remote;
pub mod schema;

pub use blob::BlobStore;
pub use schema::{check_and_migrate, SchemaInfo, CURRENT_SCHEMA_VERSION};
pub use config::{CacheConfig, CacheMode, PullPolicy, PushPolicy, RemoteCacheConfig};
pub use entry::CacheEntry;
pub use remote::{RemoteCacheClient, RemoteCacheError, RemoteClientConfig, WireCacheEntry};

use crate::error::ExecError;
use parking_lot::RwLock;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tracing::{debug, info, warn};

/// Cache statistics
#[derive(Debug, Default, Clone)]
pub struct CacheStats {
    pub hits: usize,
    pub misses: usize,
    pub stores: usize,
    pub errors: usize,
    // Remote cache stats
    pub remote_hits: usize,
    pub remote_misses: usize,
    pub remote_errors: usize,
    pub remote_timeouts: usize,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    pub fn total_hits(&self) -> usize {
        self.hits + self.remote_hits
    }

    pub fn total_misses(&self) -> usize {
        self.misses + self.remote_misses
    }

    pub fn combined_hit_rate(&self) -> f64 {
        let total = self.total_hits() + self.total_misses();
        if total == 0 {
            0.0
        } else {
            self.total_hits() as f64 / total as f64
        }
    }
}

/// The build cache (local + optional remote)
pub struct Cache {
    /// Configuration
    config: CacheConfig,
    /// Sled database for cache index
    db: sled::Db,
    /// Blob store for artifacts
    blobs: BlobStore,
    /// Statistics
    stats: RwLock<CacheStats>,
    /// Remote cache client (if configured)
    remote: Option<RemoteCacheClient>,
}

impl Cache {
    /// Open or create a cache at the given directory
    pub fn open(config: CacheConfig) -> Result<Self, ExecError> {
        let cache_dir = &config.cache_dir;
        std::fs::create_dir_all(cache_dir).map_err(|e| {
            ExecError::SpawnError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("failed to create cache dir: {}", e),
            ))
        })?;

        let db_path = cache_dir.join("index");
        let db = sled::open(&db_path).map_err(|e| {
            ExecError::SpawnError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("failed to open cache db: {}", e),
            ))
        })?;

        let blobs_path = cache_dir.join("blobs");
        let blobs = BlobStore::open(&blobs_path)?;

        // Create remote client if configured
        let remote = if config.has_remote() {
            let remote_config = RemoteClientConfig {
                server_addr: config.remote.server_addr.clone(),
                token: config.remote.token.clone(),
                client_id: None,
                connect_timeout: config.remote.connect_timeout,
                request_timeout: config.remote.request_timeout,
                max_concurrent: config.remote.max_concurrent,
                chunk_size: remote::protocol::DEFAULT_CHUNK_SIZE,
                retry: remote::RetryConfig {
                    max_retries: config.remote.max_retries,
                    initial_backoff: config.remote.initial_backoff,
                    max_backoff: config.remote.max_backoff,
                },
            };

            match RemoteCacheClient::new(remote_config) {
                Ok(client) => {
                    info!(
                        "Remote cache configured: {}",
                        config.remote.server_addr
                    );
                    Some(client)
                }
                Err(e) => {
                    warn!("Failed to create remote cache client: {}", e);
                    None
                }
            }
        } else {
            None
        };

        info!(
            "Cache opened at {} (mode: {:?})",
            cache_dir.display(),
            config.mode
        );

        Ok(Self {
            config,
            db,
            blobs,
            stats: RwLock::new(CacheStats::default()),
            remote,
        })
    }

    /// Get the cache mode
    pub fn mode(&self) -> CacheMode {
        self.config.mode
    }

    /// Check if remote cache is connected
    pub fn has_remote(&self) -> bool {
        self.remote.as_ref().map(|r| r.is_connected()).unwrap_or(false)
    }

    /// Connect to remote cache (if configured)
    pub async fn connect_remote(&self) -> Result<(), RemoteCacheError> {
        if let Some(ref client) = self.remote {
            client.connect().await
        } else {
            Ok(())
        }
    }

    /// Compute the cache key for a build action
    pub fn action_key(
        &self,
        command: &str,
        inputs: &[&Path],
        env_vars: &[(&str, &str)],
    ) -> Result<String, ExecError> {
        hasher::compute_action_key(command, inputs, env_vars)
    }

    /// Look up a cached result (local only)
    pub fn lookup_local(&self, key: &str) -> Option<CacheEntry> {
        match self.db.get(key.as_bytes()) {
            Ok(Some(data)) => {
                match CacheEntry::deserialize(&data) {
                    Ok(entry) => {
                        // Check if entry has expired
                        if let Some(max_age) = self.config.max_age {
                            if let Ok(elapsed) = entry.created.elapsed() {
                                if elapsed > max_age {
                                    debug!("Cache entry {} expired", key);
                                    self.stats.write().misses += 1;
                                    return None;
                                }
                            }
                        }
                        debug!("Local cache hit for {}", key);
                        self.stats.write().hits += 1;
                        Some(entry)
                    }
                    Err(e) => {
                        warn!("Failed to deserialize cache entry: {}", e);
                        self.stats.write().errors += 1;
                        None
                    }
                }
            }
            Ok(None) => {
                debug!("Local cache miss for {}", key);
                self.stats.write().misses += 1;
                None
            }
            Err(e) => {
                warn!("Cache lookup error: {}", e);
                self.stats.write().errors += 1;
                None
            }
        }
    }

    /// Look up a cached result (sync, local only for compatibility)
    pub fn lookup(&self, key: &str) -> Option<CacheEntry> {
        self.lookup_local(key)
    }

    /// Look up a cached result (async, respects cache mode)
    pub async fn lookup_async(&self, key: &str) -> Option<CacheEntry> {
        match self.config.mode {
            CacheMode::Local => self.lookup_local(key),
            CacheMode::Remote => {
                // Remote only - try remote, fail if not available
                self.lookup_remote(key).await
            }
            CacheMode::Auto => {
                // Try remote first with timeout, fall back to local
                if self.config.remote.pull_policy == PullPolicy::Never {
                    return self.lookup_local(key);
                }

                if let Some(ref client) = self.remote {
                    if client.is_connected() {
                        let timeout = Duration::from_secs(2);
                        match tokio::time::timeout(timeout, client.lookup(key)).await {
                            Ok(Ok(Some(wire_entry))) => {
                                debug!("Remote cache hit for {}", key);
                                self.stats.write().remote_hits += 1;
                                // Convert wire entry to CacheEntry
                                return Some(CacheEntry {
                                    command: wire_entry.command.clone(),
                                    outputs: wire_entry.to_outputs(),
                                    created: wire_entry.created_time(),
                                });
                            }
                            Ok(Ok(None)) => {
                                debug!("Remote cache miss for {}", key);
                                self.stats.write().remote_misses += 1;
                            }
                            Ok(Err(e)) => {
                                warn!("Remote cache error: {}", e);
                                self.stats.write().remote_errors += 1;
                            }
                            Err(_) => {
                                debug!("Remote cache timeout for {}", key);
                                self.stats.write().remote_timeouts += 1;
                            }
                        }
                    }
                }

                // Fall back to local
                self.lookup_local(key)
            }
        }
    }

    /// Look up from remote cache only
    async fn lookup_remote(&self, key: &str) -> Option<CacheEntry> {
        if let Some(ref client) = self.remote {
            match client.lookup(key).await {
                Ok(Some(wire_entry)) => {
                    debug!("Remote cache hit for {}", key);
                    self.stats.write().remote_hits += 1;
                    Some(CacheEntry {
                        command: wire_entry.command.clone(),
                        outputs: wire_entry.to_outputs(),
                        created: wire_entry.created_time(),
                    })
                }
                Ok(None) => {
                    debug!("Remote cache miss for {}", key);
                    self.stats.write().remote_misses += 1;
                    None
                }
                Err(e) => {
                    warn!("Remote cache error: {}", e);
                    self.stats.write().remote_errors += 1;
                    None
                }
            }
        } else {
            None
        }
    }

    /// Store a build result in the local cache
    pub fn store_local(
        &self,
        key: &str,
        outputs: &[&Path],
        command: &str,
    ) -> Result<CacheEntry, ExecError> {
        // Store each output in the blob store
        let mut output_hashes = Vec::new();
        for output in outputs {
            if output.exists() {
                let hash = self.blobs.store(output)?;
                output_hashes.push((output.to_path_buf(), hash));
            }
        }

        // Create cache entry
        let entry = CacheEntry {
            command: command.to_string(),
            outputs: output_hashes,
            created: SystemTime::now(),
        };

        // Serialize and store
        let data = entry.serialize()?;
        self.db.insert(key.as_bytes(), data).map_err(|e| {
            ExecError::SpawnError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("failed to store cache entry: {}", e),
            ))
        })?;

        debug!("Cached result locally for {}", key);
        self.stats.write().stores += 1;

        Ok(entry)
    }

    /// Store a build result in the cache (sync, local only for compatibility)
    pub fn store(
        &self,
        key: &str,
        outputs: &[&Path],
        command: &str,
    ) -> Result<(), ExecError> {
        self.store_local(key, outputs, command)?;
        Ok(())
    }

    /// Store a build result (async, respects push policy)
    pub async fn store_async(
        &self,
        key: &str,
        outputs: &[&Path],
        command: &str,
    ) -> Result<(), ExecError> {
        // Always store locally first
        let entry = self.store_local(key, outputs, command)?;

        // Push to remote if configured
        if self.should_push() {
            if let Some(ref client) = self.remote {
                if client.is_connected() {
                    let wire_entry = WireCacheEntry::from_entry(
                        &entry.command,
                        &entry.outputs,
                        entry.created,
                    );

                    // Push entry metadata
                    if let Err(e) = client.push_entry(key, &wire_entry).await {
                        warn!("Failed to push cache entry to remote: {}", e);
                    } else {
                        debug!("Pushed cache entry to remote: {}", key);

                        // Push blobs
                        for (path, _hash) in &entry.outputs {
                            if let Err(e) = client.push_blob_from_file(path).await {
                                warn!("Failed to push blob to remote: {}", e);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if we should push to remote cache
    fn should_push(&self) -> bool {
        if !self.config.has_remote() {
            return false;
        }

        match self.config.remote.push_policy {
            PushPolicy::Never => false,
            PushPolicy::OnSuccess | PushPolicy::Always => true,
        }
    }

    /// Restore cached outputs to their original locations
    pub fn restore(&self, entry: &CacheEntry) -> Result<bool, ExecError> {
        for (path, hash) in &entry.outputs {
            if !self.blobs.restore(hash, path)? {
                debug!("Blob {} not found, cache entry invalid", hash);
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        self.stats.read().clone()
    }

    /// Run cache garbage collection
    pub fn gc(&self) -> Result<GcStats, ExecError> {
        let mut stats = GcStats::default();
        let now = SystemTime::now();

        // Collect expired entries
        let mut expired_keys = Vec::new();
        for item in self.db.iter() {
            if let Ok((key, value)) = item {
                if let Ok(entry) = CacheEntry::deserialize(&value) {
                    let should_remove = if let Some(max_age) = self.config.max_age {
                        entry.created.elapsed().map(|e| e > max_age).unwrap_or(false)
                    } else {
                        false
                    };

                    if should_remove {
                        expired_keys.push(key.to_vec());
                    }
                }
            }
        }

        // Remove expired entries
        for key in expired_keys {
            if self.db.remove(&key).is_ok() {
                stats.entries_removed += 1;
            }
        }

        // Run blob GC
        let blob_stats = self.blobs.gc()?;
        stats.bytes_freed = blob_stats.bytes_freed;
        stats.blobs_removed = blob_stats.blobs_removed;

        info!(
            "GC complete: {} entries, {} blobs, {} bytes freed",
            stats.entries_removed, stats.blobs_removed, stats.bytes_freed
        );

        Ok(stats)
    }

    /// Get the cache directory
    pub fn cache_dir(&self) -> &Path {
        &self.config.cache_dir
    }

    /// Check if caching is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

/// Garbage collection statistics
#[derive(Debug, Default)]
pub struct GcStats {
    pub entries_removed: usize,
    pub blobs_removed: usize,
    pub bytes_freed: u64,
}
