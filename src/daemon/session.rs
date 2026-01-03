//! Build session management
//!
//! Each client build request creates a session that tracks build progress.

use crate::daemon::protocol::{BuildRequest, BuildStats, DaemonResponse};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info};

/// State of a build session
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    /// Build is running
    Running,
    /// Build completed successfully
    Completed,
    /// Build failed
    Failed,
    /// Build was cancelled
    Cancelled,
}

/// A build session representing an active or completed build
pub struct BuildSession {
    /// Unique session ID
    pub id: String,

    /// The original build request
    pub request: BuildRequest,

    /// Current state
    pub state: RwLock<SessionState>,

    /// When the session started
    pub started: Instant,

    /// Build statistics
    pub stats: RwLock<BuildStats>,

    /// Pending responses to send to client
    responses: RwLock<Vec<DaemonResponse>>,
}

impl BuildSession {
    /// Create a new build session
    pub fn new(request: BuildRequest) -> Self {
        Self {
            id: request.session_id.clone(),
            request,
            state: RwLock::new(SessionState::Running),
            started: Instant::now(),
            stats: RwLock::new(BuildStats::default()),
            responses: RwLock::new(Vec::new()),
        }
    }

    /// Get the session ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Check if the session is still running
    pub fn is_running(&self) -> bool {
        *self.state.read() == SessionState::Running
    }

    /// Mark the session as completed
    pub fn complete(&self, success: bool) {
        let mut state = self.state.write();
        *state = if success {
            SessionState::Completed
        } else {
            SessionState::Failed
        };

        let mut stats = self.stats.write();
        stats.duration_ms = self.started.elapsed().as_millis() as u64;
    }

    /// Mark the session as cancelled
    pub fn cancel(&self) {
        let mut state = self.state.write();
        *state = SessionState::Cancelled;
    }

    /// Queue a response for the client
    pub fn queue_response(&self, response: DaemonResponse) {
        self.responses.write().push(response);
    }

    /// Take all pending responses
    pub fn take_responses(&self) -> Vec<DaemonResponse> {
        std::mem::take(&mut *self.responses.write())
    }

    /// Record that a target build started
    pub fn target_started(&self, target: &str, index: usize, total: usize, command: Option<String>) {
        debug!("Session {}: target started {}/{} - {}", self.id, index, total, target);
        self.queue_response(DaemonResponse::TargetStarted {
            target: target.to_string(),
            index,
            total,
            command,
        });
    }

    /// Record target output
    pub fn target_output(&self, target: &str, stdout: Vec<u8>, stderr: Vec<u8>) {
        if !stdout.is_empty() || !stderr.is_empty() {
            self.queue_response(DaemonResponse::TargetOutput {
                target: target.to_string(),
                stdout,
                stderr,
            });
        }
    }

    /// Record that a target build finished
    pub fn target_finished(
        &self,
        target: &str,
        index: usize,
        total: usize,
        success: bool,
        cached: bool,
        duration_ms: u64,
    ) {
        debug!(
            "Session {}: target finished {}/{} - {} (success={}, cached={})",
            self.id, index, total, target, success, cached
        );

        // Update stats
        {
            let mut stats = self.stats.write();
            if success {
                stats.targets_built += 1;
                if cached {
                    stats.cache_hits += 1;
                } else {
                    stats.cache_misses += 1;
                }
            } else {
                stats.targets_failed += 1;
            }
        }

        self.queue_response(DaemonResponse::TargetFinished {
            target: target.to_string(),
            index,
            total,
            success,
            cached,
            duration_ms,
        });
    }

    /// Record that the build finished
    pub fn build_finished(&self, success: bool) {
        self.complete(success);

        let stats = self.stats.read().clone();
        info!(
            "Session {}: build finished (success={}, built={}, failed={}, cached={})",
            self.id, success, stats.targets_built, stats.targets_failed, stats.cache_hits
        );

        self.queue_response(DaemonResponse::BuildFinished {
            session_id: self.id.clone(),
            success,
            stats,
        });
    }

    /// Get elapsed time
    pub fn elapsed_ms(&self) -> u64 {
        self.started.elapsed().as_millis() as u64
    }
}

/// Manager for all active build sessions
pub struct SessionManager {
    sessions: RwLock<HashMap<String, Arc<BuildSession>>>,
    max_concurrent: usize,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            max_concurrent,
        }
    }

    /// Create a new session
    pub fn create_session(&self, request: BuildRequest) -> Result<Arc<BuildSession>, String> {
        let sessions = self.sessions.read();

        // Check concurrent build limit
        let active_count = sessions.values().filter(|s| s.is_running()).count();
        if active_count >= self.max_concurrent {
            return Err(format!(
                "Maximum concurrent builds ({}) reached",
                self.max_concurrent
            ));
        }

        drop(sessions);

        let session = Arc::new(BuildSession::new(request));
        self.sessions.write().insert(session.id.clone(), session.clone());

        Ok(session)
    }

    /// Get a session by ID
    pub fn get_session(&self, id: &str) -> Option<Arc<BuildSession>> {
        self.sessions.read().get(id).cloned()
    }

    /// Remove a session
    pub fn remove_session(&self, id: &str) -> Option<Arc<BuildSession>> {
        self.sessions.write().remove(id)
    }

    /// Cancel a session
    pub fn cancel_session(&self, id: &str) -> bool {
        if let Some(session) = self.get_session(id) {
            session.cancel();
            true
        } else {
            false
        }
    }

    /// Get the number of active builds
    pub fn active_build_count(&self) -> usize {
        self.sessions
            .read()
            .values()
            .filter(|s| s.is_running())
            .count()
    }

    /// Clean up completed sessions older than the given duration
    pub fn cleanup_old_sessions(&self, max_age_secs: u64) {
        let mut sessions = self.sessions.write();
        let cutoff = std::time::Duration::from_secs(max_age_secs);

        sessions.retain(|_, session| {
            session.is_running() || session.started.elapsed() < cutoff
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_request() -> BuildRequest {
        BuildRequest::new("test-session".to_string(), PathBuf::from("/tmp"))
    }

    #[test]
    fn test_session_creation() {
        let session = BuildSession::new(test_request());
        assert!(session.is_running());
        assert_eq!(session.id(), "test-session");
    }

    #[test]
    fn test_session_completion() {
        let session = BuildSession::new(test_request());
        session.complete(true);
        assert!(!session.is_running());
        assert_eq!(*session.state.read(), SessionState::Completed);
    }

    #[test]
    fn test_session_manager() {
        let manager = SessionManager::new(2);

        let s1 = manager.create_session(BuildRequest::new("s1".to_string(), PathBuf::from("/tmp")));
        assert!(s1.is_ok());

        let s2 = manager.create_session(BuildRequest::new("s2".to_string(), PathBuf::from("/tmp")));
        assert!(s2.is_ok());

        // Third should fail
        let s3 = manager.create_session(BuildRequest::new("s3".to_string(), PathBuf::from("/tmp")));
        assert!(s3.is_err());

        // Complete one session
        s1.unwrap().complete(true);

        // Now we should be able to create another
        let s4 = manager.create_session(BuildRequest::new("s4".to_string(), PathBuf::from("/tmp")));
        assert!(s4.is_ok());
    }
}
