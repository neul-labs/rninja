//! Remote cache server module
//!
//! Provides the rninja-cached server that handles remote cache requests.

pub mod auth;
pub mod config;
pub mod handler;

pub use config::ServerConfig;
pub use handler::CacheServer;

use crate::cache::remote::protocol::PROTOCOL_VERSION;
use crate::error::ExecError;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::info;

/// Server statistics
#[derive(Debug, Default)]
pub struct ServerStats {
    pub requests_total: AtomicU64,
    pub requests_success: AtomicU64,
    pub requests_failed: AtomicU64,
    pub bytes_received: AtomicU64,
    pub bytes_sent: AtomicU64,
    pub cache_entries: AtomicU64,
    pub cache_blobs: AtomicU64,
    pub cache_size_bytes: AtomicU64,
}

impl ServerStats {
    pub fn record_request(&self, success: bool, bytes_in: usize, bytes_out: usize) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
        if success {
            self.requests_success.fetch_add(1, Ordering::Relaxed);
        } else {
            self.requests_failed.fetch_add(1, Ordering::Relaxed);
        }
        self.bytes_received
            .fetch_add(bytes_in as u64, Ordering::Relaxed);
        self.bytes_sent
            .fetch_add(bytes_out as u64, Ordering::Relaxed);
    }
}

/// Run the cache server
pub async fn run_server(config: ServerConfig) -> Result<(), ExecError> {
    info!(
        "Starting rninja-cached server (protocol v{})",
        PROTOCOL_VERSION
    );
    info!("Listening on {}", config.listen_addr);
    info!("Storage directory: {}", config.storage_dir.display());

    let server = CacheServer::new(config)?;
    server.run().await
}
