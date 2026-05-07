//! Daemon server implementation
//!
//! NNG-based server that handles client requests.

use nng::options::Options;

use crate::daemon::protocol::{
    deserialize_request, serialize_response, BuildRequest, CacheStatsInfo, DaemonRequest,
    DaemonResponse, ErrorCode, QueryRequest, DAEMON_PROTOCOL_VERSION,
};
use crate::daemon::session::SessionManager;
use crate::daemon::state::{DaemonConfig, DaemonState};
use crate::daemon::watcher::{FileWatcher, WatcherProcessor};
use anyhow::{Context, Result};
use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// The rninja daemon server
pub struct DaemonServer {
    /// Socket path
    socket_path: PathBuf,

    /// NNG socket
    socket: Option<nng::Socket>,

    /// Shared state
    state: Arc<DaemonState>,

    /// Session manager
    sessions: Arc<SessionManager>,

    /// File watcher
    watcher: RwLock<FileWatcher>,

    /// Shutdown flag
    shutdown: Arc<AtomicBool>,
}

impl DaemonServer {
    /// Create a new daemon server
    pub fn new(socket_path: PathBuf, config: DaemonConfig) -> Result<Self> {
        let state = Arc::new(DaemonState::new(config.clone()));
        let sessions = Arc::new(SessionManager::new(config.max_concurrent_builds));
        let watcher = FileWatcher::new().context("Failed to create file watcher")?;

        Ok(Self {
            socket_path,
            socket: None,
            state,
            sessions,
            watcher: RwLock::new(watcher),
            shutdown: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Get the socket path
    pub fn socket_path(&self) -> &PathBuf {
        &self.socket_path
    }

    /// Start the server
    pub fn start(&mut self) -> Result<()> {
        // Ensure socket directory exists
        if let Some(parent) = self.socket_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Remove stale socket file if it exists.
        // Use remove_file directly - it handles race conditions better than
        // checking exists() first. If file doesn't exist, that's fine.
        // If another process created it, we'll detect and report the error.
        if let Err(e) = std::fs::remove_file(&self.socket_path) {
            if e.kind() != std::io::ErrorKind::NotFound {
                return Err(e.into());
            }
            // File didn't exist, which is fine
        }

        // Create and bind socket
        let socket = nng::Socket::new(nng::Protocol::Rep0)
            .context("Failed to create NNG socket")?;

        let url = format!("ipc://{}", self.socket_path.display());
        socket.listen(&url).context("Failed to bind to socket")?;

        info!("Daemon listening on {}", self.socket_path.display());
        self.socket = Some(socket);

        Ok(())
    }

    /// Run the server main loop
    pub fn run(&self) -> Result<()> {
        let socket = self.socket.as_ref().context("Server not started")?;

        // Set up watcher processor
        let watcher_processor = {
            let mut watcher = self.watcher.write();
            WatcherProcessor::new(&mut watcher)
        };

        // Main loop
        while !self.shutdown.load(Ordering::SeqCst) {
            // Process file watcher events
            if let Some(ref processor) = watcher_processor {
                for path in processor.process_events() {
                    if let Some(build_dir) = self.state.find_build_dir_for_file(&path) {
                        self.state.invalidate(&build_dir);
                    }
                }
            }

            // Try to receive a request with timeout
            if let Err(e) = socket.set_opt::<nng::options::RecvTimeout>(Some(Duration::from_millis(100))) {
                tracing::warn!("Failed to set receive timeout: {}", e);
            }

            match socket.recv() {
                Ok(msg) => {
                    let response = self.handle_message(msg.as_slice());
                    if let Ok(data) = serialize_response(&response) {
                        let reply = nng::Message::from(&data[..]);
                        if let Err(e) = socket.send(reply) {
                            error!("Failed to send response: {:?}", e);
                        }
                    }
                }
                Err(nng::Error::TimedOut) => {
                    // Normal timeout, continue loop
                    continue;
                }
                Err(e) => {
                    error!("Error receiving message: {}", e);
                }
            }

            // Periodic cleanup
            self.sessions.cleanup_old_sessions(3600); // 1 hour
        }

        info!("Daemon shutting down");
        Ok(())
    }

    /// Handle an incoming message
    fn handle_message(&self, data: &[u8]) -> DaemonResponse {
        match deserialize_request(data) {
            Ok(envelope) => {
                // Check protocol version
                if envelope.version != DAEMON_PROTOCOL_VERSION {
                    return DaemonResponse::Error {
                        code: ErrorCode::VersionMismatch,
                        message: format!(
                            "Protocol version mismatch: client v{}, server v{}",
                            envelope.version, DAEMON_PROTOCOL_VERSION
                        ),
                    };
                }

                self.handle_request(envelope.request)
            }
            Err(e) => DaemonResponse::Error {
                code: ErrorCode::InternalError,
                message: format!("Failed to parse request: {}", e),
            },
        }
    }

    /// Handle a daemon request
    fn handle_request(&self, request: DaemonRequest) -> DaemonResponse {
        match request {
            DaemonRequest::Ping => self.handle_ping(),
            DaemonRequest::Status => self.handle_status(),
            DaemonRequest::Shutdown => self.handle_shutdown(),
            DaemonRequest::InvalidateCache { build_dir } => {
                self.handle_invalidate_cache(build_dir)
            }
            DaemonRequest::Build(req) => self.handle_build(req),
            DaemonRequest::Query(req) => self.handle_query(req),
            DaemonRequest::CancelBuild { session_id } => self.handle_cancel_build(session_id),
        }
    }

    /// Handle ping request
    fn handle_ping(&self) -> DaemonResponse {
        DaemonResponse::Pong {
            version: env!("CARGO_PKG_VERSION").to_string(),
            protocol_version: DAEMON_PROTOCOL_VERSION,
            uptime_secs: self.state.uptime_secs(),
        }
    }

    /// Handle status request
    fn handle_status(&self) -> DaemonResponse {
        DaemonResponse::DaemonStatus {
            active_builds: self.sessions.active_build_count(),
            cached_manifests: self.state.cached_manifest_count(),
            cache_stats: CacheStatsInfo::default(), // TODO: Get real stats from cache
            uptime_secs: self.state.uptime_secs(),
        }
    }

    /// Handle shutdown request
    fn handle_shutdown(&self) -> DaemonResponse {
        info!("Shutdown requested");
        self.shutdown.store(true, Ordering::SeqCst);
        DaemonResponse::Ack
    }

    /// Handle cache invalidation request
    fn handle_invalidate_cache(&self, build_dir: PathBuf) -> DaemonResponse {
        self.state.invalidate(&build_dir);
        DaemonResponse::Ack
    }

    /// Handle build request
    fn handle_build(&self, request: BuildRequest) -> DaemonResponse {
        debug!("Build request: {:?}", request);

        // Create session
        let session = match self.sessions.create_session(request.clone()) {
            Ok(s) => s,
            Err(e) => {
                return DaemonResponse::Error {
                    code: ErrorCode::InternalError,
                    message: e,
                };
            }
        };

        // Get or parse manifest
        let cached = match self.state.get_or_parse_manifest(
            &request.build_dir,
            &request.build_file,
        ) {
            Ok(c) => c,
            Err(e) => {
                session.build_finished(false);
                return DaemonResponse::Error {
                    code: ErrorCode::ParseError,
                    message: format!("Failed to parse manifest: {}", e),
                };
            }
        };

        // Watch manifest files
        {
            let mut watcher = self.watcher.write();
            if let Err(e) = watcher.watch_paths(&cached.included_files) {
                warn!("Failed to set up file watching: {}", e);
            }
        }

        // Determine targets
        let targets: Vec<&str> = if request.targets.is_empty() {
            cached.manifest.defaults.iter().map(|s| s.as_str()).collect()
        } else {
            request.targets.iter().map(|s| s.as_str()).collect()
        };

        if targets.is_empty() {
            session.build_finished(false);
            return DaemonResponse::Error {
                code: ErrorCode::TargetNotFound,
                message: "No targets specified and no default target".to_string(),
            };
        }

        // Get build order
        let build_order = match cached.graph.topological_order(&targets) {
            Ok(order) => order,
            Err(e) => {
                session.build_finished(false);
                return DaemonResponse::Error {
                    code: ErrorCode::CircularDependency,
                    message: format!("Dependency error: {}", e),
                };
            }
        };

        let total_targets = build_order.len();

        // For now, return BuildStarted - actual execution would be async
        // In a full implementation, we'd spawn a task to run the build
        // and stream responses back to the client
        DaemonResponse::BuildStarted {
            session_id: session.id().to_string(),
            total_targets,
        }
    }

    /// Helper to execute a query that needs a parsed manifest.
    fn with_manifest<F>(&self, build_dir: &PathBuf, f: F) -> DaemonResponse
    where
        F: FnOnce(&crate::daemon::state::CachedManifest) -> DaemonResponse,
    {
        match self.state.get_or_parse_manifest(build_dir, "build.ninja") {
            Ok(cached) => f(&cached),
            Err(e) => DaemonResponse::Error {
                code: ErrorCode::ParseError,
                message: e.to_string(),
            },
        }
    }

    /// Handle query request
    fn handle_query(&self, request: QueryRequest) -> DaemonResponse {
        match request {
            QueryRequest::Targets { build_dir } => {
                self.with_manifest(&build_dir, |cached| {
                    let targets: Vec<String> = cached
                        .graph
                        .outputs()
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect();
                    DaemonResponse::QueryResult {
                        data: targets.join("\n"),
                    }
                })
            }
            QueryRequest::Rules { build_dir } => {
                self.with_manifest(&build_dir, |cached| {
                    let rules: Vec<String> =
                        cached.manifest.rules.keys().cloned().collect();
                    DaemonResponse::QueryResult {
                        data: rules.join("\n"),
                    }
                })
            }
            QueryRequest::Commands { build_dir, targets } => {
                self.with_manifest(&build_dir, |cached| {
                    let mut commands = Vec::new();
                    for target in &targets {
                        if let Some(node) = cached.graph.get_node(target) {
                            if let Some(cmd) = &node.command {
                                commands.push(cmd.clone());
                            }
                        }
                    }
                    DaemonResponse::QueryResult {
                        data: commands.join("\n"),
                    }
                })
            }
            QueryRequest::Deps { build_dir, target } => {
                self.with_manifest(&build_dir, |cached| {
                    if let Some(inputs) = cached.graph.inputs_for(&target) {
                        DaemonResponse::QueryResult {
                            data: inputs.join("\n"),
                        }
                    } else {
                        DaemonResponse::Error {
                            code: ErrorCode::TargetNotFound,
                            message: format!("Target not found: {}", target),
                        }
                    }
                })
            }
            QueryRequest::Inputs { build_dir, target } => {
                self.with_manifest(&build_dir, |cached| {
                    match cached.graph.topological_order(&[&target]) {
                        Ok(order) => {
                            let mut inputs = Vec::new();
                            for t in order {
                                if let Some(node) = cached.graph.get_node(&t) {
                                    if node.is_source {
                                        inputs.push(t);
                                    }
                                }
                            }
                            DaemonResponse::QueryResult {
                                data: inputs.join("\n"),
                            }
                        }
                        Err(e) => DaemonResponse::Error {
                            code: ErrorCode::InternalError,
                            message: e.to_string(),
                        },
                    }
                })
            }
            QueryRequest::CompDb { build_dir } => {
                self.with_manifest(&build_dir, |cached| {
                    let mut entries = Vec::new();
                    for build in &cached.manifest.builds {
                        if let Some(rule) = cached.manifest.rules.get(&build.rule) {
                            if let Some(cmd) = &rule.command {
                                for input in &build.inputs {
                                    if input.ends_with(".c")
                                        || input.ends_with(".cc")
                                        || input.ends_with(".cpp")
                                    {
                                        entries.push(serde_json::json!({
                                            "directory": build_dir.to_string_lossy(),
                                            "command": cmd,
                                            "file": input,
                                        }));
                                    }
                                }
                            }
                        }
                    }
                    match serde_json::to_string_pretty(&entries) {
                        Ok(data) => DaemonResponse::QueryResult { data },
                        Err(e) => DaemonResponse::Error {
                            code: ErrorCode::ParseError,
                            message: format!(
                                "Failed to serialize compilation database: {}",
                                e
                            ),
                        },
                    }
                })
            }
        }
    }

    /// Handle cancel build request
    fn handle_cancel_build(&self, session_id: String) -> DaemonResponse {
        if self.sessions.cancel_session(&session_id) {
            info!("Cancelled build session: {}", session_id);
            DaemonResponse::Ack
        } else {
            DaemonResponse::Error {
                code: ErrorCode::SessionNotFound,
                message: format!("Session not found: {}", session_id),
            }
        }
    }

    /// Get a shutdown handle that can be used to stop the server from another thread
    pub fn shutdown_handle(&self) -> Arc<AtomicBool> {
        self.shutdown.clone()
    }
}

impl Drop for DaemonServer {
    fn drop(&mut self) {
        // Clean up socket file
        if self.socket_path.exists() {
            let _ = std::fs::remove_file(&self.socket_path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_server_creation() {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("test.sock");
        let server = DaemonServer::new(socket_path, DaemonConfig::default());
        assert!(server.is_ok());
    }
}
