//! Daemon connection handling with auto-spawn logic

use nng::options::Options;

use crate::daemon::protocol::{
    deserialize_response, get_default_socket_path, serialize_request, DaemonRequest,
    DaemonResponse, RequestEnvelope, DAEMON_PROTOCOL_VERSION,
};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;
use tracing::{debug, info, warn};

/// Connection timeout for daemon operations
const CONNECT_TIMEOUT_MS: u64 = 5000;

/// Time to wait for daemon to start up
const DAEMON_STARTUP_TIMEOUT_MS: u64 = 5000;

/// Interval between connection attempts when waiting for daemon
const CONNECT_RETRY_INTERVAL_MS: u64 = 100;

/// Client for communicating with the rninja daemon
pub struct DaemonClient {
    socket_path: PathBuf,
    #[allow(dead_code)]
    socket: Option<nng::Socket>,
}

/// Represents an active connection to the daemon
pub struct DaemonConnection {
    socket: nng::Socket,
}

impl DaemonClient {
    /// Create a new daemon client with the default socket path
    pub fn new() -> Self {
        Self {
            socket_path: get_default_socket_path(),
            socket: None,
        }
    }

    /// Create a new daemon client with a custom socket path
    pub fn with_socket_path(socket_path: PathBuf) -> Self {
        Self {
            socket_path,
            socket: None,
        }
    }

    /// Get the socket path being used
    pub fn socket_path(&self) -> &PathBuf {
        &self.socket_path
    }

    /// Check if the daemon is running
    pub fn is_daemon_running(&self) -> bool {
        self.try_connect().is_ok()
    }

    /// Try to connect to the daemon without spawning
    fn try_connect(&self) -> Result<nng::Socket> {
        let socket =
            nng::Socket::new(nng::Protocol::Req0).context("Failed to create NNG socket")?;

        // Set timeouts
        if let Err(e) = socket
            .set_opt::<nng::options::RecvTimeout>(Some(Duration::from_millis(CONNECT_TIMEOUT_MS)))
        {
            tracing::warn!("Failed to set receive timeout: {}", e);
        }
        if let Err(e) = socket
            .set_opt::<nng::options::SendTimeout>(Some(Duration::from_millis(CONNECT_TIMEOUT_MS)))
        {
            tracing::warn!("Failed to set send timeout: {}", e);
        }

        let url = format!("ipc://{}", self.socket_path.display());
        socket.dial(&url).context("Failed to connect to daemon")?;

        Ok(socket)
    }

    /// Connect to the daemon, spawning it if necessary
    pub fn connect(&mut self) -> Result<DaemonConnection> {
        // Try connecting to existing daemon first
        match self.try_connect() {
            Ok(socket) => {
                debug!(
                    "Connected to existing daemon at {}",
                    self.socket_path.display()
                );

                // Verify protocol version
                let conn = DaemonConnection { socket };
                if let Err(e) = conn.verify_version() {
                    warn!("Protocol version mismatch: {}", e);
                    // Could try to restart daemon here, but for now just fail
                    return Err(e);
                }

                return Ok(conn);
            }
            Err(e) => {
                debug!("No existing daemon found: {}", e);
            }
        }

        // Spawn the daemon
        info!("Spawning rninja-daemon...");
        self.spawn_daemon()?;

        // Wait for daemon to be ready
        let start = std::time::Instant::now();
        let timeout = Duration::from_millis(DAEMON_STARTUP_TIMEOUT_MS);

        while start.elapsed() < timeout {
            match self.try_connect() {
                Ok(socket) => {
                    info!("Daemon started in {:?}", start.elapsed());
                    let conn = DaemonConnection { socket };
                    conn.verify_version()?;
                    return Ok(conn);
                }
                Err(e) => {
                    let attempt = (start.elapsed().as_millis()
                        / u128::from(CONNECT_RETRY_INTERVAL_MS))
                        as u64;
                    debug!("Waiting for daemon (attempt {}): {}", attempt, e);
                    std::thread::sleep(Duration::from_millis(CONNECT_RETRY_INTERVAL_MS));
                }
            }
        }

        anyhow::bail!("Daemon failed to start within {:?}", timeout)
    }

    /// Spawn the daemon process
    fn spawn_daemon(&self) -> Result<()> {
        // Ensure socket directory exists
        if let Some(parent) = self.socket_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create socket directory")?;
        }

        // Find rninja-daemon binary
        let daemon_path = find_daemon_binary()?;

        debug!(
            "Starting daemon: {} --socket {}",
            daemon_path.display(),
            self.socket_path.display()
        );

        // Spawn daemon as a detached process
        Command::new(&daemon_path)
            .arg("--socket")
            .arg(&self.socket_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .with_context(|| format!("Failed to spawn daemon: {}", daemon_path.display()))?;

        Ok(())
    }

    /// Connect without auto-spawning (for --no-daemon mode validation)
    pub fn connect_no_spawn(&mut self) -> Result<DaemonConnection> {
        let socket = self.try_connect()?;
        let conn = DaemonConnection { socket };
        conn.verify_version()?;
        Ok(conn)
    }
}

impl Default for DaemonClient {
    fn default() -> Self {
        Self::new()
    }
}

impl DaemonConnection {
    /// Send a request and receive a response
    pub fn request(&self, request: DaemonRequest) -> Result<DaemonResponse> {
        let envelope = RequestEnvelope::new(request);
        let data = serialize_request(&envelope).context("Failed to serialize request")?;

        let msg = nng::Message::from(&data[..]);
        self.socket
            .send(msg)
            .map_err(|(_, e)| anyhow::anyhow!("Send failed: {}", e))?;

        let response_msg = self.socket.recv().context("Failed to receive response")?;

        deserialize_response(response_msg.as_slice()).context("Failed to deserialize response")
    }

    /// Verify protocol version compatibility
    fn verify_version(&self) -> Result<()> {
        match self.request(DaemonRequest::Ping)? {
            DaemonResponse::Pong {
                version,
                protocol_version,
                uptime_secs,
            } => {
                if protocol_version != DAEMON_PROTOCOL_VERSION {
                    anyhow::bail!(
                        "Protocol version mismatch: daemon has v{}, client expects v{}",
                        protocol_version,
                        DAEMON_PROTOCOL_VERSION
                    );
                }
                debug!(
                    "Connected to daemon v{} (protocol v{}), uptime {}s",
                    version, protocol_version, uptime_secs
                );
                Ok(())
            }
            DaemonResponse::Error { code, message } => {
                anyhow::bail!("Daemon error ({:?}): {}", code, message)
            }
            other => {
                anyhow::bail!("Unexpected response to Ping: {:?}", other)
            }
        }
    }

    /// Send a ping request
    pub fn ping(&self) -> Result<(String, u64)> {
        match self.request(DaemonRequest::Ping)? {
            DaemonResponse::Pong {
                version,
                uptime_secs,
                ..
            } => Ok((version, uptime_secs)),
            other => anyhow::bail!("Unexpected response: {:?}", other),
        }
    }

    /// Request daemon status
    pub fn status(&self) -> Result<DaemonResponse> {
        self.request(DaemonRequest::Status)
    }

    /// Request daemon shutdown
    pub fn shutdown(&self) -> Result<()> {
        match self.request(DaemonRequest::Shutdown)? {
            DaemonResponse::Ack => Ok(()),
            DaemonResponse::Error { code, message } => {
                anyhow::bail!("Shutdown failed ({:?}): {}", code, message)
            }
            other => anyhow::bail!("Unexpected response: {:?}", other),
        }
    }
}

/// Find the rninja-daemon binary
fn find_daemon_binary() -> Result<PathBuf> {
    // First, try next to the current executable
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let daemon_path = dir.join("rninja-daemon");
            if daemon_path.exists() {
                return Ok(daemon_path);
            }
        }
    }

    // Try in PATH
    if let Ok(path) = which::which("rninja-daemon") {
        return Ok(path);
    }

    // Fall back to just the name (let the OS find it)
    Ok(PathBuf::from("rninja-daemon"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = DaemonClient::new();
        assert!(client
            .socket_path()
            .to_string_lossy()
            .contains("daemon.sock"));
    }

    #[test]
    fn test_custom_socket_path() {
        let path = PathBuf::from("/tmp/test-rninja.sock");
        let client = DaemonClient::with_socket_path(path.clone());
        assert_eq!(client.socket_path(), &path);
    }
}
