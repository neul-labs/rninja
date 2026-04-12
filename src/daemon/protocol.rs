//! Daemon IPC protocol
//!
//! Defines message types for communication between CLI clients and the daemon.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Protocol version for compatibility checking
pub const DAEMON_PROTOCOL_VERSION: u32 = 1;

/// Default socket path pattern
pub const DEFAULT_SOCKET_NAME: &str = "daemon.sock";

/// Request from client to daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DaemonRequest {
    /// Build targets
    Build(BuildRequest),

    /// Query build information
    Query(QueryRequest),

    /// Ping daemon (health check)
    Ping,

    /// Request daemon shutdown
    Shutdown,

    /// Invalidate cached manifest for a directory
    InvalidateCache { build_dir: PathBuf },

    /// Get daemon status
    Status,

    /// Cancel a running build
    CancelBuild { session_id: String },
}

/// Build request with all parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildRequest {
    /// Unique session ID for this build
    pub session_id: String,

    /// Working directory for the build
    pub build_dir: PathBuf,

    /// Path to build.ninja file (relative to build_dir)
    pub build_file: String,

    /// Targets to build (empty = default target)
    pub targets: Vec<String>,

    /// Number of parallel jobs
    pub parallelism: usize,

    /// Dry run mode
    pub dry_run: bool,

    /// Verbose output
    pub verbose: bool,

    /// Explain rebuilds
    pub explain: bool,

    /// Keep going on failures (0 = infinite)
    pub keep_going: usize,

    /// Output in JSON format
    pub json_output: bool,
}

impl BuildRequest {
    pub fn new(session_id: String, build_dir: PathBuf) -> Self {
        Self {
            session_id,
            build_dir,
            build_file: "build.ninja".to_string(),
            targets: Vec::new(),
            parallelism: num_cpus::get(),
            dry_run: false,
            verbose: false,
            explain: false,
            keep_going: 1,
            json_output: false,
        }
    }
}

/// Query request types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryRequest {
    /// List all targets
    Targets { build_dir: PathBuf },

    /// List all rules
    Rules { build_dir: PathBuf },

    /// Show commands for targets
    Commands { build_dir: PathBuf, targets: Vec<String> },

    /// Show dependencies
    Deps { build_dir: PathBuf, target: String },

    /// Show inputs for a target
    Inputs { build_dir: PathBuf, target: String },

    /// Compilation database
    CompDb { build_dir: PathBuf },
}

/// Response from daemon to client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DaemonResponse {
    /// Build started
    BuildStarted {
        session_id: String,
        total_targets: usize,
    },

    /// A target build started
    TargetStarted {
        target: String,
        index: usize,
        total: usize,
        command: Option<String>,
    },

    /// Output from a target
    TargetOutput {
        target: String,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    },

    /// A target build finished
    TargetFinished {
        target: String,
        index: usize,
        total: usize,
        success: bool,
        cached: bool,
        duration_ms: u64,
    },

    /// Build completed
    BuildFinished {
        session_id: String,
        success: bool,
        stats: BuildStats,
    },

    /// Query result
    QueryResult { data: String },

    /// Ping response
    Pong {
        version: String,
        protocol_version: u32,
        uptime_secs: u64,
    },

    /// Daemon status
    DaemonStatus {
        active_builds: usize,
        cached_manifests: usize,
        cache_stats: CacheStatsInfo,
        uptime_secs: u64,
    },

    /// Acknowledgment (for shutdown, cancel, etc.)
    Ack,

    /// No work to do
    NoWorkToDo,

    /// Error response
    Error { code: ErrorCode, message: String },
}

/// Build statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuildStats {
    pub targets_built: usize,
    pub targets_skipped: usize,
    pub targets_failed: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub duration_ms: u64,
}

/// Cache statistics for status reporting
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStatsInfo {
    pub local_hits: usize,
    pub local_misses: usize,
    pub remote_hits: usize,
    pub remote_misses: usize,
}

/// Error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    /// Build failed
    BuildFailed,

    /// Target not found
    TargetNotFound,

    /// Parse error in build.ninja
    ParseError,

    /// File not found
    FileNotFound,

    /// Session not found
    SessionNotFound,

    /// Daemon is shutting down
    ShuttingDown,

    /// Protocol version mismatch
    VersionMismatch,

    /// Internal error
    InternalError,

    /// Circular dependency
    CircularDependency,
}

/// Request envelope with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestEnvelope {
    /// Protocol version
    pub version: u32,

    /// The actual request
    pub request: DaemonRequest,
}

impl RequestEnvelope {
    pub fn new(request: DaemonRequest) -> Self {
        Self {
            version: DAEMON_PROTOCOL_VERSION,
            request,
        }
    }
}

/// Serialize a request to bytes
pub fn serialize_request(envelope: &RequestEnvelope) -> Result<Vec<u8>, rmp_serde::encode::Error> {
    rmp_serde::to_vec(envelope)
}

/// Deserialize a request from bytes
pub fn deserialize_request(data: &[u8]) -> Result<RequestEnvelope, rmp_serde::decode::Error> {
    rmp_serde::from_slice(data)
}

/// Serialize a response to bytes
pub fn serialize_response(response: &DaemonResponse) -> Result<Vec<u8>, rmp_serde::encode::Error> {
    rmp_serde::to_vec(response)
}

/// Deserialize a response from bytes
pub fn deserialize_response(data: &[u8]) -> Result<DaemonResponse, rmp_serde::decode::Error> {
    rmp_serde::from_slice(data)
}

/// Get the default socket path for this user
pub fn get_default_socket_path() -> PathBuf {
    // Try XDG_RUNTIME_DIR first
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        let path = PathBuf::from(runtime_dir).join("rninja");
        return path.join(DEFAULT_SOCKET_NAME);
    }

    // Fall back to /tmp/rninja-{uid}
    let uid = get_effective_uid();
    PathBuf::from(format!("/tmp/rninja-{}", uid)).join(DEFAULT_SOCKET_NAME)
}

/// Get the effective user ID (platform-specific)
#[cfg(unix)]
fn get_effective_uid() -> u32 {
    // SAFETY: getuid() is a standard POSIX function that reads process state
    // and does not modify any memory or interact with the system in unsafe ways.
    unsafe { libc::getuid() as u32 }
}

#[cfg(not(unix))]
fn get_effective_uid() -> u32 {
    // On non-Unix platforms, use a hash of the temp dir as a surrogate
    static RES: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
    *RES.get_or_init(|| {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let temp_dir = std::env::temp_dir();
        let mut hasher = DefaultHasher::new();
        temp_dir.hash(&mut hasher);
        hasher.finish() as u32
    })
}

/// Generate a unique session ID
pub fn generate_session_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_roundtrip() {
        let req = RequestEnvelope::new(DaemonRequest::Ping);
        let data = serialize_request(&req).unwrap();
        let decoded = deserialize_request(&data).unwrap();

        assert_eq!(decoded.version, DAEMON_PROTOCOL_VERSION);
        assert!(matches!(decoded.request, DaemonRequest::Ping));
    }

    #[test]
    fn test_response_roundtrip() {
        let resp = DaemonResponse::Pong {
            version: "0.1.0".to_string(),
            protocol_version: DAEMON_PROTOCOL_VERSION,
            uptime_secs: 100,
        };
        let data = serialize_response(&resp).unwrap();
        let decoded = deserialize_response(&data).unwrap();

        if let DaemonResponse::Pong { version, .. } = decoded {
            assert_eq!(version, "0.1.0");
        } else {
            panic!("Expected Pong response");
        }
    }

    #[test]
    fn test_build_request() {
        let req = BuildRequest::new(
            generate_session_id(),
            PathBuf::from("/home/user/project"),
        );
        assert_eq!(req.build_file, "build.ninja");
        assert!(!req.dry_run);
    }
}
