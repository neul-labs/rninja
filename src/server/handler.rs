//! Request handler for the cache server

use super::auth::{AuthResult, TokenValidator};
use super::config::ServerConfig;
use super::ServerStats;
use crate::cache::remote::protocol::{
    deserialize_request, serialize_response, ErrorCode, Request, Response, WireCacheEntry,
    PROTOCOL_VERSION,
};
use crate::cache::{BlobStore, CacheEntry};
use crate::error::ExecError;
use nng::options::Options;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Instant, SystemTime};
use tracing::{debug, error, info, warn};

/// Validates that a hash contains only safe characters for use in file paths
fn is_valid_hash(hash: &str) -> bool {
    !hash.is_empty() && hash.chars().all(|c| c.is_ascii_hexdigit())
}

/// The cache server
pub struct CacheServer {
    config: ServerConfig,
    db: sled::Db,
    blobs: BlobStore,
    auth: TokenValidator,
    stats: Arc<ServerStats>,
    start_time: Instant,
}

impl CacheServer {
    /// Create a new cache server
    pub fn new(config: ServerConfig) -> Result<Self, ExecError> {
        // Create storage directory
        std::fs::create_dir_all(&config.storage_dir).map_err(|e| {
            ExecError::SpawnError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("failed to create storage dir: {}", e),
            ))
        })?;

        // Open sled database
        let db_path = config.storage_dir.join("index");
        let db = sled::open(&db_path).map_err(|e| {
            ExecError::SpawnError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("failed to open database: {}", e),
            ))
        })?;

        // Open blob store
        let blobs_path = config.storage_dir.join("blobs");
        let blobs = BlobStore::open(&blobs_path)?;

        // Create auth validator
        let auth = TokenValidator::new(&config.auth);

        info!(
            "Cache server initialized with {} tokens",
            auth.token_count()
        );

        Ok(Self {
            config,
            db,
            blobs,
            auth,
            stats: Arc::new(ServerStats::default()),
            start_time: Instant::now(),
        })
    }

    /// Run the server main loop
    pub async fn run(&self) -> Result<(), ExecError> {
        let socket = nng::Socket::new(nng::Protocol::Rep0).map_err(|e| {
            ExecError::SpawnError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("failed to create socket: {}", e),
            ))
        })?;

        socket.listen(&self.config.listen_addr).map_err(|e| {
            ExecError::SpawnError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("failed to listen on {}: {}", self.config.listen_addr, e),
            ))
        })?;

        info!("Server listening on {}", self.config.listen_addr);

        loop {
            // Receive request
            let msg = match socket.recv() {
                Ok(msg) => msg,
                Err(e) => {
                    warn!("Failed to receive message: {}", e);
                    continue;
                }
            };

            let request_bytes = msg.as_slice();
            let bytes_in = request_bytes.len();

            // Process request
            let response = self.handle_request(request_bytes).await;

            // Serialize and send response
            let response_bytes = match serialize_response(&response) {
                Ok(bytes) => bytes,
                Err(e) => {
                    error!("Failed to serialize response: {}", e);
                    let error_response = Response::error(ErrorCode::ServerError, "serialization failed");
                    serialize_response(&error_response).unwrap_or_default()
                }
            };

            let bytes_out = response_bytes.len();
            self.stats.record_request(!response.is_error(), bytes_in, bytes_out);

            let reply = nng::Message::from(response_bytes.as_slice());
            if let Err(e) = socket.send(reply) {
                warn!("Failed to send response: {:?}", e);
            }
        }
    }

    /// Handle a single request
    async fn handle_request(&self, data: &[u8]) -> Response {
        // Deserialize request envelope
        let envelope = match deserialize_request(data) {
            Ok(env) => env,
            Err(e) => {
                warn!("Failed to deserialize request: {}", e);
                return Response::error(ErrorCode::InvalidRequest, "invalid request format");
            }
        };

        // Check protocol version
        if envelope.version != PROTOCOL_VERSION {
            return Response::error(
                ErrorCode::VersionMismatch,
                format!(
                    "protocol version mismatch: expected {}, got {}",
                    PROTOCOL_VERSION, envelope.version
                ),
            );
        }

        // Authenticate
        match self.auth.validate(&envelope.auth) {
            AuthResult::Allowed => {}
            AuthResult::NoToken => {
                return Response::error(ErrorCode::AuthRequired, "authentication required");
            }
            AuthResult::InvalidToken => {
                return Response::error(ErrorCode::AuthFailed, "invalid token");
            }
        }

        // Handle request
        match envelope.request {
            Request::Ping => self.handle_ping(),
            Request::Stats => self.handle_stats(),
            Request::Exists { keys } => self.handle_exists(&keys),
            Request::Lookup { key } => self.handle_lookup(&key),
            Request::PushEntry { key, entry } => self.handle_push_entry(&key, entry),
            Request::PushBlob { hash, data, offset, total } => {
                self.handle_push_blob(&hash, &data, offset, total)
            }
            Request::PushBlobComplete { hash, checksum } => {
                self.handle_push_blob_complete(&hash, &checksum)
            }
            Request::PullBlobs { hashes } => self.handle_pull_blobs(&hashes),
        }
    }

    fn handle_ping(&self) -> Response {
        Response::Pong {
            version: env!("CARGO_PKG_VERSION").to_string(),
            server_time: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    fn handle_stats(&self) -> Response {
        let entries = self.db.len() as u64;
        let uptime = self.start_time.elapsed().as_secs();

        // Get blob stats (simplified)
        let blobs = self.stats.cache_blobs.load(Ordering::Relaxed);
        let size = self.stats.cache_size_bytes.load(Ordering::Relaxed);

        Response::Statistics {
            entries,
            blobs,
            total_size_bytes: size,
            uptime_secs: uptime,
        }
    }

    fn handle_exists(&self, keys: &[String]) -> Response {
        let mut results = HashMap::new();
        for key in keys {
            let exists = self.db.contains_key(key.as_bytes()).unwrap_or(false);
            results.insert(key.clone(), exists);
        }
        Response::Exists { results }
    }

    fn handle_lookup(&self, key: &str) -> Response {
        match self.db.get(key.as_bytes()) {
            Ok(Some(data)) => {
                match CacheEntry::deserialize(&data) {
                    Ok(entry) => {
                        // Check TTL if configured
                        if let Some(ttl) = self.config.entry_ttl() {
                            if let Ok(elapsed) = entry.created.elapsed() {
                                if elapsed > ttl {
                                    debug!("Entry {} expired", key);
                                    return Response::NotFound;
                                }
                            }
                        }

                        let wire_entry = WireCacheEntry::from_entry(
                            &entry.command,
                            &entry.outputs,
                            entry.created,
                        );
                        Response::Entry { entry: wire_entry }
                    }
                    Err(e) => {
                        warn!("Failed to deserialize entry {}: {}", key, e);
                        Response::NotFound
                    }
                }
            }
            Ok(None) => Response::NotFound,
            Err(e) => {
                warn!("Database error looking up {}: {}", key, e);
                Response::error(ErrorCode::ServerError, "database error")
            }
        }
    }

    fn handle_push_entry(&self, key: &str, wire_entry: WireCacheEntry) -> Response {
        // Check storage quota
        if let Some(max_size) = self.config.max_storage_size {
            let current = self.stats.cache_size_bytes.load(Ordering::Relaxed);
            if current >= max_size {
                return Response::error(ErrorCode::StorageFull, "storage quota exceeded");
            }
        }

        // Convert wire entry to cache entry
        let entry = CacheEntry {
            command: wire_entry.command.clone(),
            outputs: wire_entry.to_outputs(),
            created: wire_entry.created_time(),
        };

        // Serialize and store
        match entry.serialize() {
            Ok(data) => {
                if let Err(e) = self.db.insert(key.as_bytes(), data) {
                    warn!("Failed to store entry {}: {}", key, e);
                    return Response::error(ErrorCode::ServerError, "failed to store entry");
                }
                self.stats.cache_entries.fetch_add(1, Ordering::Relaxed);
                debug!("Stored entry: {}", key);
                Response::Ok
            }
            Err(e) => {
                warn!("Failed to serialize entry {}: {}", key, e);
                Response::error(ErrorCode::ServerError, "serialization failed")
            }
        }
    }

    fn handle_push_blob(&self, hash: &str, data: &[u8], offset: u64, total: u64) -> Response {
        // Validate hash to prevent path traversal attacks
        if !is_valid_hash(hash) {
            warn!("Invalid hash received: {}", hash);
            return Response::error(ErrorCode::InvalidRequest, "invalid hash format");
        }

        // For simplicity, we store the entire blob when offset is 0 and data.len() == total
        // A full implementation would handle chunked uploads
        if offset == 0 && data.len() as u64 == total {
            // Store directly
            let blob_path = self.blob_path(hash);
            if let Some(parent) = blob_path.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    warn!("Failed to create blob directory: {}", e);
                    return Response::error(ErrorCode::ServerError, "failed to create directory");
                }
            }

            if let Err(e) = std::fs::write(&blob_path, data) {
                warn!("Failed to write blob {}: {}", hash, e);
                return Response::error(ErrorCode::ServerError, "failed to write blob");
            }

            self.stats.cache_blobs.fetch_add(1, Ordering::Relaxed);
            self.stats
                .cache_size_bytes
                .fetch_add(data.len() as u64, Ordering::Relaxed);

            debug!("Stored blob: {} ({} bytes)", hash, data.len());
        }

        Response::Ok
    }

    fn handle_push_blob_complete(&self, hash: &str, _checksum: &str) -> Response {
        // Validate hash to prevent path traversal attacks
        if !is_valid_hash(hash) {
            warn!("Invalid hash received: {}", hash);
            return Response::error(ErrorCode::InvalidRequest, "invalid hash format");
        }

        // Verify blob exists
        let blob_path = self.blob_path(hash);
        if blob_path.exists() {
            Response::BlobComplete {
                hash: hash.to_string(),
            }
        } else {
            Response::error(ErrorCode::NotFound, "blob not found")
        }
    }

    fn handle_pull_blobs(&self, hashes: &[String]) -> Response {
        // Validate all hashes first
        for hash in hashes {
            if !is_valid_hash(hash) {
                warn!("Invalid hash received: {}", hash);
                return Response::error(ErrorCode::InvalidRequest, "invalid hash format");
            }
        }

        // Check all blobs exist
        for hash in hashes {
            let blob_path = self.blob_path(hash);
            if !blob_path.exists() {
                return Response::NotFound;
            }
        }
        Response::Ok
    }

    fn blob_path(&self, hash: &str) -> PathBuf {
        let prefix = if hash.len() >= 2 { &hash[..2] } else { hash };
        self.config
            .storage_dir
            .join("blobs")
            .join(prefix)
            .join(hash)
    }
}
