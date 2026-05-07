//! Remote cache client using NNG transport
//!
//! Provides an async client for communicating with the remote cache server.

use super::error::RemoteCacheError;
use super::protocol::{
    deserialize_response, serialize_request, Request, RequestEnvelope, Response, WireCacheEntry,
    DEFAULT_CHUNK_SIZE,
};
use crate::cache::blob::BlobStore;
use nng::options::Options;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tracing::{debug, info, warn};

/// Configuration for the remote cache client
#[derive(Debug, Clone)]
pub struct RemoteClientConfig {
    /// Server address (e.g., "tcp://cache.example.com:9999")
    pub server_addr: String,
    /// Authentication token
    pub token: String,
    /// Optional client identifier
    pub client_id: Option<String>,
    /// Connection timeout
    pub connect_timeout: Duration,
    /// Request timeout
    pub request_timeout: Duration,
    /// Maximum concurrent operations
    pub max_concurrent: usize,
    /// Chunk size for blob transfers
    pub chunk_size: usize,
    /// Retry configuration
    pub retry: RetryConfig,
}

impl Default for RemoteClientConfig {
    fn default() -> Self {
        Self {
            server_addr: String::new(),
            token: String::new(),
            client_id: None,
            connect_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(30),
            max_concurrent: 4,
            chunk_size: DEFAULT_CHUNK_SIZE,
            retry: RetryConfig::default(),
        }
    }
}

/// Retry configuration for transient failures
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial backoff duration
    pub initial_backoff: Duration,
    /// Maximum backoff duration
    pub max_backoff: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(5),
        }
    }
}

/// Statistics for the remote cache client
#[derive(Debug, Default)]
pub struct ClientStats {
    pub requests_sent: AtomicU64,
    pub requests_failed: AtomicU64,
    pub bytes_sent: AtomicU64,
    pub bytes_received: AtomicU64,
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,
    pub retries: AtomicU64,
    pub avg_latency_us: AtomicU64,
    latency_samples: AtomicUsize,
    latency_sum_us: AtomicU64,
}

impl ClientStats {
    pub fn record_request(&self, bytes_sent: usize, bytes_received: usize) {
        self.requests_sent.fetch_add(1, Ordering::Relaxed);
        self.bytes_sent.fetch_add(bytes_sent as u64, Ordering::Relaxed);
        self.bytes_received
            .fetch_add(bytes_received as u64, Ordering::Relaxed);
    }

    pub fn record_latency(&self, latency: Duration) {
        let us = latency.as_micros() as u64;
        self.latency_sum_us.fetch_add(us, Ordering::Relaxed);
        let samples = self.latency_samples.fetch_add(1, Ordering::Relaxed) + 1;
        let sum = self.latency_sum_us.load(Ordering::Relaxed);
        self.avg_latency_us
            .store(sum / samples as u64, Ordering::Relaxed);
    }

    pub fn record_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_failure(&self) {
        self.requests_failed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_retry(&self) {
        self.retries.fetch_add(1, Ordering::Relaxed);
    }
}

/// Validates that a hash contains only safe characters for use in cache operations.
///
/// Cache hashes are hex-encoded blake3 output, so they should only contain
/// ASCII hexadecimal characters.
fn is_valid_cache_hash(hash: &str) -> bool {
    !hash.is_empty() && hash.len() >= 32 && hash.chars().all(|c| c.is_ascii_hexdigit())
}

/// Validates all hashes in a slice, returning the first invalid hash if any.
fn validate_cache_hashes(hashes: &[impl AsRef<str>]) -> Option<String> {
    for hash in hashes {
        let h = hash.as_ref();
        if h.len() < 32 || !h.chars().all(|c| c.is_ascii_hexdigit()) {
            return Some(h.to_string());
        }
    }
    None
}

/// State of the remote cache connection
#[derive(Debug)]
enum ConnectionState {
    Disconnected,
    Connecting,
    Connected { socket: nng::Socket },
}

/// Error returned when an invalid connection state transition is attempted
#[derive(Debug)]
struct InvalidConnectionTransition {
    from: &'static str,
    to: &'static str,
}

impl std::fmt::Display for InvalidConnectionTransition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid connection state transition: {} -> {}",
            self.from, self.to
        )
    }
}

impl ConnectionState {
    fn transition(
        &self,
        to: ConnectionState,
    ) -> Result<ConnectionState, InvalidConnectionTransition> {
        let valid = match (self, &to) {
            (ConnectionState::Disconnected, ConnectionState::Connecting) => true,
            (ConnectionState::Connecting, ConnectionState::Connected { .. }) => true,
            (ConnectionState::Connecting, ConnectionState::Disconnected) => true,
            (ConnectionState::Connected { .. }, ConnectionState::Disconnected) => true,
            (from, to) if std::mem::discriminant(from) == std::mem::discriminant(to) => {
                true
            }
            _ => false,
        };
        if valid {
            Ok(to)
        } else {
            Err(InvalidConnectionTransition {
                from: self.name(),
                to: to.name(),
            })
        }
    }

    fn name(&self) -> &'static str {
        match self {
            ConnectionState::Disconnected => "Disconnected",
            ConnectionState::Connecting => "Connecting",
            ConnectionState::Connected { .. } => "Connected",
        }
    }

    fn is_connected(&self) -> bool {
        matches!(self, ConnectionState::Connected { .. })
    }
}

/// Remote cache client
pub struct RemoteCacheClient {
    config: RemoteClientConfig,
    connection: RwLock<ConnectionState>,
    semaphore: Arc<Semaphore>,
    stats: Arc<ClientStats>,
}

impl RemoteCacheClient {
    /// Create a new remote cache client
    pub fn new(config: RemoteClientConfig) -> Result<Self, RemoteCacheError> {
        if config.server_addr.is_empty() {
            return Err(RemoteCacheError::ConfigError(
                "server address is required".into(),
            ));
        }
        if config.token.is_empty() {
            return Err(RemoteCacheError::ConfigError(
                "authentication token is required".into(),
            ));
        }

        let semaphore = Arc::new(Semaphore::new(config.max_concurrent));
        let stats = Arc::new(ClientStats::default());

        Ok(Self {
            config,
            connection: RwLock::new(ConnectionState::Disconnected),
            semaphore,
            stats,
        })
    }

    /// Connect to the remote cache server
    pub async fn connect(&self) -> Result<(), RemoteCacheError> {
        // Validate and transition to Connecting
        {
            let mut guard = self.connection.write();
            match (*guard).transition(ConnectionState::Connecting) {
                Ok(new) => *guard = new,
                Err(e) => {
                    return Err(RemoteCacheError::ConfigError(format!(
                        "connection state error: {}",
                        e
                    )));
                }
            }
        }

        let result = self.connect_inner().await;

        // Transition out of Connecting on completion
        {
            let mut guard = self.connection.write();
            match &result {
                Ok(()) => {
                    // The inner connect has already set Connected state
                }
                Err(_) => {
                    if let Ok(new) = (*guard).transition(ConnectionState::Disconnected) {
                        *guard = new;
                    }
                }
            }
        }

        result
    }

    async fn connect_inner(&self) -> Result<(), RemoteCacheError> {
        let socket = nng::Socket::new(nng::Protocol::Req0)?;

        // Set timeouts
        socket
            .set_opt::<nng::options::SendTimeout>(Some(self.config.request_timeout))
            .map_err(|e| RemoteCacheError::ConfigError(format!("failed to set send timeout: {}", e)))?;
        socket
            .set_opt::<nng::options::RecvTimeout>(Some(self.config.request_timeout))
            .map_err(|e| RemoteCacheError::ConfigError(format!("failed to set recv timeout: {}", e)))?;

        // Connect with timeout (blocking call offloaded to spawn_blocking)
        let addr = self.config.server_addr.clone();
        debug!("Connecting to remote cache at {}", addr);

        let socket = tokio::time::timeout(self.config.connect_timeout, async {
            tokio::task::spawn_blocking(move || {
                socket.dial(&addr).map_err(|e| {
                    RemoteCacheError::ConnectionFailed(e.to_string())
                })?;
                Ok::<_, RemoteCacheError>(socket)
            })
            .await
            .map_err(|e| RemoteCacheError::ConnectionFailed(format!("blocking task panicked: {}", e)))?
        })
        .await
        .map_err(|_| RemoteCacheError::ConnectionTimeout(self.config.connect_timeout.as_millis() as u64))??;

        // Verify connection with ping before declaring Connected
        {
            let mut guard = self.connection.write();
            match (*guard).transition(ConnectionState::Connected {
                socket: socket.clone(),
            }) {
                Ok(new) => *guard = new,
                Err(e) => {
                    return Err(RemoteCacheError::ConfigError(format!(
                        "connection state error: {}",
                        e
                    )));
                }
            }
        }

        match self.ping().await {
            Ok(version) => {
                info!("Connected to remote cache server (version: {})", version);
                Ok(())
            }
            Err(e) => {
                self.disconnect();
                Err(e)
            }
        }
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connection.read().is_connected()
    }

    /// Get client statistics
    pub fn stats(&self) -> &ClientStats {
        &self.stats
    }

    /// Generate a client ID based on process info
    fn generate_client_id() -> String {
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        let pid = std::process::id();
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        format!("rninja-{}-{}-{}", hostname, pid, ts)
    }

    /// Send a request and receive a response
    async fn send_request(&self, request: Request) -> Result<Response, RemoteCacheError> {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| RemoteCacheError::Shutdown)?;

        let client_id = self
            .config
            .client_id
            .clone()
            .unwrap_or_else(Self::generate_client_id);

        let envelope = RequestEnvelope::new(self.config.token.clone(), Some(client_id), request);

        let request_bytes = serialize_request(&envelope)?;
        let start = Instant::now();

        // Extract socket while holding the lock, then perform blocking I/O off-thread
        let socket = {
            let guard = self.connection.read();
            match &*guard {
                ConnectionState::Connected { socket } => socket.clone(),
                _ => {
                    return Err(RemoteCacheError::ConnectionFailed(
                        "not connected".into(),
                    ));
                }
            }
        };

        let request_bytes_clone = request_bytes.clone();
        let response_bytes = tokio::task::spawn_blocking(move || {
            // Send request
            let msg = nng::Message::from(request_bytes_clone.as_slice());
            socket
                .send(msg)
                .map_err(|e| RemoteCacheError::NetworkError(format!("send failed: {:?}", e)))?;

            // Receive response
            let response_msg = socket
                .recv()
                .map_err(|e| RemoteCacheError::NetworkError(format!("recv failed: {:?}", e)))?;

            Ok::<_, RemoteCacheError>(response_msg.as_slice().to_vec())
        })
        .await
        .map_err(|e| RemoteCacheError::NetworkError(format!("blocking task panicked: {}", e)))??;

        let latency = start.elapsed();
        self.stats
            .record_request(request_bytes.len(), response_bytes.len());
        self.stats.record_latency(latency);

        let response = deserialize_response(&response_bytes)?;

        // Check for protocol-level errors
        if let Response::Error { code, message } = &response {
            return Err(RemoteCacheError::ServerError {
                code: *code,
                message: message.clone(),
            });
        }

        Ok(response)
    }

    /// Send a request with retry logic
    async fn send_with_retry(&self, request: Request) -> Result<Response, RemoteCacheError> {
        let mut backoff = self.config.retry.initial_backoff;
        let mut last_error = None;

        for attempt in 0..=self.config.retry.max_retries {
            if attempt > 0 {
                self.stats.record_retry();
                debug!(
                    "Retry attempt {} after {:?}",
                    attempt, backoff
                );
                tokio::time::sleep(backoff).await;
                backoff = (backoff * 2).min(self.config.retry.max_backoff);
            }

            match self.send_request(request.clone()).await {
                Ok(response) => return Ok(response),
                Err(e) if e.is_retryable() && attempt < self.config.retry.max_retries => {
                    warn!("Retryable error: {}", e);
                    last_error = Some(e);
                }
                Err(e) => {
                    self.stats.record_failure();
                    return Err(e);
                }
            }
        }

        self.stats.record_failure();
        Err(last_error.unwrap_or_else(|| RemoteCacheError::NetworkError("max retries exceeded".into())))
    }

    /// Ping the server
    pub async fn ping(&self) -> Result<String, RemoteCacheError> {
        match self.send_request(Request::Ping).await? {
            Response::Pong { version, .. } => Ok(version),
            other => Err(RemoteCacheError::InvalidResponse(format!(
                "expected Pong, got {:?}",
                other
            ))),
        }
    }

    /// Check if cache entries exist
    pub async fn exists(&self, keys: &[&str]) -> Result<HashMap<String, bool>, RemoteCacheError> {
        let request = Request::Exists {
            keys: keys.iter().map(|s| s.to_string()).collect(),
        };

        match self.send_with_retry(request).await? {
            Response::Exists { results } => Ok(results),
            other => Err(RemoteCacheError::InvalidResponse(format!(
                "expected Exists, got {:?}",
                other
            ))),
        }
    }

    /// Lookup a cache entry
    pub async fn lookup(&self, key: &str) -> Result<Option<WireCacheEntry>, RemoteCacheError> {
        // Validate hash to prevent potential injection attacks
        if !is_valid_cache_hash(key) {
            warn!("Invalid cache key received: {}", key);
            return Err(RemoteCacheError::InvalidResponse(
                "invalid cache key format".into(),
            ));
        }

        let request = Request::Lookup {
            key: key.to_string(),
        };

        match self.send_with_retry(request).await? {
            Response::Entry { entry } => {
                self.stats.record_hit();
                Ok(Some(entry))
            }
            Response::NotFound => {
                self.stats.record_miss();
                Ok(None)
            }
            other => Err(RemoteCacheError::InvalidResponse(format!(
                "expected Entry or NotFound, got {:?}",
                other
            ))),
        }
    }

    /// Pull blobs from the server
    pub async fn pull_blobs(
        &self,
        hashes: &[String],
        _blob_store: &BlobStore,
    ) -> Result<(), RemoteCacheError> {
        if hashes.is_empty() {
            return Ok(());
        }

        // Validate all hashes before sending request
        if let Some(invalid) = validate_cache_hashes(hashes) {
            warn!("Invalid hash in pull request: {}", invalid);
            return Err(RemoteCacheError::InvalidResponse(
                "invalid hash format".into(),
            ));
        }

        let request = Request::PullBlobs {
            hashes: hashes.to_vec(),
        };

        // For simplicity, we'll handle this as a series of chunk responses
        // In a real implementation, this would be a streaming protocol
        let response = self.send_with_retry(request).await?;

        match response {
            Response::Ok => Ok(()),
            Response::NotFound => Err(RemoteCacheError::ServerError {
                code: super::protocol::ErrorCode::NotFound,
                message: "one or more blobs not found".into(),
            }),
            other => Err(RemoteCacheError::InvalidResponse(format!(
                "expected Ok, got {:?}",
                other
            ))),
        }
    }

    /// Push a cache entry to the server
    pub async fn push_entry(
        &self,
        key: &str,
        entry: &WireCacheEntry,
    ) -> Result<(), RemoteCacheError> {
        let request = Request::PushEntry {
            key: key.to_string(),
            entry: entry.clone(),
        };

        match self.send_with_retry(request).await? {
            Response::Ok => Ok(()),
            other => Err(RemoteCacheError::InvalidResponse(format!(
                "expected Ok, got {:?}",
                other
            ))),
        }
    }

    /// Push a blob to the server
    pub async fn push_blob(&self, hash: &str, data: &[u8]) -> Result<(), RemoteCacheError> {
        // Validate hash before sending data
        if !is_valid_cache_hash(hash) {
            warn!("Invalid hash in push request: {}", hash);
            return Err(RemoteCacheError::InvalidResponse(
                "invalid hash format".into(),
            ));
        }

        let total = data.len() as u64;
        let mut offset = 0u64;

        while offset < total {
            let end = ((offset as usize) + self.config.chunk_size).min(data.len());
            let chunk = &data[offset as usize..end];

            let request = Request::PushBlob {
                hash: hash.to_string(),
                data: chunk.to_vec(),
                offset,
                total,
            };

            self.send_with_retry(request).await?;
            offset = end as u64;
        }

        // Complete the blob push
        let checksum = blake3::hash(data).to_hex().to_string();
        let request = Request::PushBlobComplete {
            hash: hash.to_string(),
            checksum,
        };

        match self.send_with_retry(request).await? {
            Response::Ok | Response::BlobComplete { .. } => Ok(()),
            other => Err(RemoteCacheError::InvalidResponse(format!(
                "expected Ok or BlobComplete, got {:?}",
                other
            ))),
        }
    }

    /// Push a blob from a file
    pub async fn push_blob_from_file(&self, path: &Path) -> Result<String, RemoteCacheError> {
        let data = std::fs::read(path).map_err(|e| {
            RemoteCacheError::BlobTransferFailed(format!("failed to read file: {}", e))
        })?;

        let hash = blake3::hash(&data).to_hex().to_string();
        self.push_blob(&hash, &data).await?;

        Ok(hash)
    }

    /// Get server statistics
    pub async fn server_stats(&self) -> Result<(u64, u64, u64, u64), RemoteCacheError> {
        match self.send_request(Request::Stats).await? {
            Response::Statistics {
                entries,
                blobs,
                total_size_bytes,
                uptime_secs,
            } => Ok((entries, blobs, total_size_bytes, uptime_secs)),
            other => Err(RemoteCacheError::InvalidResponse(format!(
                "expected Statistics, got {:?}",
                other
            ))),
        }
    }

    /// Disconnect from the server
    pub fn disconnect(&self) {
        let mut guard = self.connection.write();
        if let Ok(new) = (*guard).transition(ConnectionState::Disconnected) {
            *guard = new;
        }
    }
}

impl Drop for RemoteCacheClient {
    fn drop(&mut self) {
        self.disconnect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_id_generation() {
        let id1 = RemoteCacheClient::generate_client_id();
        let id2 = RemoteCacheClient::generate_client_id();

        assert!(id1.starts_with("rninja-"));
        assert!(id2.starts_with("rninja-"));
        // IDs should differ due to timestamp
        std::thread::sleep(Duration::from_millis(1));
        let id3 = RemoteCacheClient::generate_client_id();
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_config_validation() {
        let config = RemoteClientConfig {
            server_addr: String::new(),
            token: "test".into(),
            ..Default::default()
        };
        assert!(RemoteCacheClient::new(config).is_err());

        let config = RemoteClientConfig {
            server_addr: "tcp://localhost:9999".into(),
            token: String::new(),
            ..Default::default()
        };
        assert!(RemoteCacheClient::new(config).is_err());

        let config = RemoteClientConfig {
            server_addr: "tcp://localhost:9999".into(),
            token: "test".into(),
            ..Default::default()
        };
        assert!(RemoteCacheClient::new(config).is_ok());
    }

    #[test]
    fn test_stats_recording() {
        let stats = ClientStats::default();

        stats.record_request(100, 200);
        assert_eq!(stats.requests_sent.load(Ordering::Relaxed), 1);
        assert_eq!(stats.bytes_sent.load(Ordering::Relaxed), 100);
        assert_eq!(stats.bytes_received.load(Ordering::Relaxed), 200);

        stats.record_hit();
        stats.record_hit();
        stats.record_miss();
        assert_eq!(stats.cache_hits.load(Ordering::Relaxed), 2);
        assert_eq!(stats.cache_misses.load(Ordering::Relaxed), 1);

        stats.record_latency(Duration::from_micros(1000));
        stats.record_latency(Duration::from_micros(2000));
        assert_eq!(stats.avg_latency_us.load(Ordering::Relaxed), 1500);
    }
}
