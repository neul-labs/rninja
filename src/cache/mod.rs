mod blob;
mod config;
mod entry;
mod hasher;

pub use config::CacheConfig;
pub use entry::CacheEntry;

use crate::error::ExecError;
use blob::BlobStore;
use parking_lot::RwLock;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tracing::{debug, info, warn};

/// Cache statistics
#[derive(Debug, Default, Clone)]
pub struct CacheStats {
    pub hits: usize,
    pub misses: usize,
    pub stores: usize,
    pub errors: usize,
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
}

/// The local build cache
pub struct Cache {
    /// Configuration
    config: CacheConfig,
    /// Sled database for cache index
    db: sled::Db,
    /// Blob store for artifacts
    blobs: BlobStore,
    /// Statistics
    stats: RwLock<CacheStats>,
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

        info!("Cache opened at {}", cache_dir.display());

        Ok(Self {
            config,
            db,
            blobs,
            stats: RwLock::new(CacheStats::default()),
        })
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

    /// Look up a cached result
    pub fn lookup(&self, key: &str) -> Option<CacheEntry> {
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
                        debug!("Cache hit for {}", key);
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
                debug!("Cache miss for {}", key);
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

    /// Store a build result in the cache
    pub fn store(
        &self,
        key: &str,
        outputs: &[&Path],
        command: &str,
    ) -> Result<(), ExecError> {
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

        debug!("Cached result for {}", key);
        self.stats.write().stores += 1;

        Ok(())
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
