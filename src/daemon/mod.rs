//! Daemon module for rninja
//!
//! Provides a long-running daemon process that caches parsed manifests
//! and dependency graphs for faster incremental builds.

pub mod protocol;
pub mod server;
pub mod session;
pub mod state;
pub mod watcher;

pub use protocol::*;
pub use server::DaemonServer;
pub use session::BuildSession;
pub use state::DaemonState;
