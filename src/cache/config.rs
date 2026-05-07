use std::path::PathBuf;
use std::time::Duration;

/// Cache operation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CacheMode {
    /// Local cache only
    #[default]
    Local,
    /// Remote cache only (fail if unavailable)
    Remote,
    /// Try remote first, fall back to local
    Auto,
}

impl CacheMode {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "local" => Some(CacheMode::Local),
            "remote" => Some(CacheMode::Remote),
            "auto" => Some(CacheMode::Auto),
            _ => None,
        }
    }
}

/// Push policy for remote cache
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum PushPolicy {
    /// Never push to remote
    Never,
    /// Push only successful builds (default)
    #[default]
    OnSuccess,
    /// Push all builds
    Always,
}

impl PushPolicy {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "never" => Some(PushPolicy::Never),
            "on_success" | "onsuccess" => Some(PushPolicy::OnSuccess),
            "always" => Some(PushPolicy::Always),
            _ => None,
        }
    }
}

/// Pull policy for remote cache
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum PullPolicy {
    /// Always try remote first (default)
    #[default]
    Always,
    /// Only pull if local miss
    OnMiss,
    /// Never pull (push-only mode)
    Never,
}

impl PullPolicy {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "always" => Some(PullPolicy::Always),
            "on_miss" | "onmiss" => Some(PullPolicy::OnMiss),
            "never" => Some(PullPolicy::Never),
            _ => None,
        }
    }
}

/// Remote cache configuration
#[derive(Debug, Clone)]
pub struct RemoteCacheConfig {
    /// Server address (e.g., "tcp://cache.example.com:9999")
    pub server_addr: String,
    /// Authentication token
    pub token: String,
    /// Connection timeout
    pub connect_timeout: Duration,
    /// Request timeout
    pub request_timeout: Duration,
    /// Push policy
    pub push_policy: PushPolicy,
    /// Pull policy
    pub pull_policy: PullPolicy,
    /// Maximum concurrent remote operations
    pub max_concurrent: usize,
    /// Maximum retries for transient failures
    pub max_retries: u32,
    /// Initial backoff for retries
    pub initial_backoff: Duration,
    /// Maximum backoff for retries
    pub max_backoff: Duration,
}

impl Default for RemoteCacheConfig {
    fn default() -> Self {
        Self {
            server_addr: String::new(),
            token: String::new(),
            connect_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(30),
            push_policy: PushPolicy::default(),
            pull_policy: PullPolicy::default(),
            max_concurrent: 4,
            max_retries: 3,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(5),
        }
    }
}

impl RemoteCacheConfig {
    /// Check if remote cache is configured
    pub fn is_configured(&self) -> bool {
        !self.server_addr.is_empty() && !self.token.is_empty()
    }

    /// Create from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(server) = std::env::var("RNINJA_CACHE_REMOTE_SERVER") {
            config.server_addr = server;
        }

        if let Ok(token) = std::env::var("RNINJA_CACHE_TOKEN") {
            config.token = token;
        }

        if let Ok(val) = std::env::var("RNINJA_CACHE_PUSH_POLICY") {
            if let Some(policy) = PushPolicy::from_str(&val) {
                config.push_policy = policy;
            }
        }

        if let Ok(val) = std::env::var("RNINJA_CACHE_PULL_POLICY") {
            if let Some(policy) = PullPolicy::from_str(&val) {
                config.pull_policy = policy;
            }
        }

        if let Ok(val) = std::env::var("RNINJA_CACHE_CONNECT_TIMEOUT") {
            if let Ok(secs) = val.parse::<u64>() {
                config.connect_timeout = Duration::from_secs(secs);
            }
        }

        if let Ok(val) = std::env::var("RNINJA_CACHE_REQUEST_TIMEOUT") {
            if let Ok(secs) = val.parse::<u64>() {
                config.request_timeout = Duration::from_secs(secs);
            }
        }

        if let Ok(val) = std::env::var("RNINJA_CACHE_MAX_CONCURRENT") {
            if let Ok(n) = val.parse::<usize>() {
                config.max_concurrent = n;
            }
        }

        config
    }
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Whether caching is enabled
    pub enabled: bool,
    /// Cache mode (local, remote, auto)
    pub mode: CacheMode,
    /// Cache directory path
    pub cache_dir: PathBuf,
    /// Maximum age for cache entries (None = no expiry)
    pub max_age: Option<Duration>,
    /// Maximum cache size in bytes (None = no limit)
    pub max_size: Option<u64>,
    /// Remote cache configuration
    pub remote: RemoteCacheConfig,
    /// Auto-migrate cache schema if needed
    pub auto_migrate: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: CacheMode::default(),
            cache_dir: default_cache_dir(),
            max_age: None,
            max_size: None,
            remote: RemoteCacheConfig::default(),
            auto_migrate: true,
        }
    }
}

impl CacheConfig {
    /// Create config from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // RNINJA_CACHE_DIR - cache directory
        if let Ok(dir) = std::env::var("RNINJA_CACHE_DIR") {
            config.cache_dir = PathBuf::from(dir);
        }

        // RNINJA_CACHE_ENABLED - enable/disable caching
        if let Ok(val) = std::env::var("RNINJA_CACHE_ENABLED") {
            config.enabled = val != "0" && val.to_lowercase() != "false";
        }

        // RNINJA_CACHE_MAX_AGE - max age in seconds
        if let Ok(val) = std::env::var("RNINJA_CACHE_MAX_AGE") {
            if let Ok(secs) = val.parse::<u64>() {
                config.max_age = Some(Duration::from_secs(secs));
            }
        }

        // RNINJA_CACHE_MAX_SIZE - max size in bytes
        if let Ok(val) = std::env::var("RNINJA_CACHE_MAX_SIZE") {
            if let Ok(bytes) = parse_size(&val) {
                config.max_size = Some(bytes);
            }
        }

        // RNINJA_CACHE_MODE - cache mode (local, remote, auto)
        if let Ok(val) = std::env::var("RNINJA_CACHE_MODE") {
            if let Some(mode) = CacheMode::from_str(&val) {
                config.mode = mode;
            }
        }

        // Load remote config from environment
        config.remote = RemoteCacheConfig::from_env();

        // Auto-detect mode based on remote config
        if config.remote.is_configured() && config.mode == CacheMode::Local {
            // If remote is configured but mode is still local, upgrade to auto
            config.mode = CacheMode::Auto;
        }

        config
    }

    /// Disable caching
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Self::default()
        }
    }

    /// Check if remote cache is available
    pub fn has_remote(&self) -> bool {
        self.remote.is_configured() && self.mode != CacheMode::Local
    }
}

/// Get the default cache directory
fn default_cache_dir() -> PathBuf {
    // Try XDG_CACHE_HOME first
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        return PathBuf::from(xdg).join("rninja");
    }

    // Fall back to ~/.cache/rninja
    if let Some(home) = dirs_next_home() {
        return home.join(".cache").join("rninja");
    }

    // Last resort: .rninja-cache in current directory
    PathBuf::from(".rninja-cache")
}

/// Simple home directory detection without external crate
fn dirs_next_home() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}

/// Parse a size string like "1G", "500M", "1024K", or just bytes
fn parse_size(s: &str) -> Result<u64, ()> {
    let s = s.trim();
    if s.is_empty() {
        return Err(());
    }

    let (num, mult) = if s.ends_with('G') || s.ends_with('g') {
        (&s[..s.len() - 1], 1024 * 1024 * 1024)
    } else if s.ends_with('M') || s.ends_with('m') {
        (&s[..s.len() - 1], 1024 * 1024)
    } else if s.ends_with('K') || s.ends_with('k') {
        (&s[..s.len() - 1], 1024)
    } else {
        (s, 1)
    };

    num.parse::<u64>().map(|n| n * mult).map_err(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("1024"), Ok(1024));
        assert_eq!(parse_size("1K"), Ok(1024));
        assert_eq!(parse_size("1k"), Ok(1024));
        assert_eq!(parse_size("1M"), Ok(1024 * 1024));
        assert_eq!(parse_size("1G"), Ok(1024 * 1024 * 1024));
        assert_eq!(parse_size("500M"), Ok(500 * 1024 * 1024));
    }
}
