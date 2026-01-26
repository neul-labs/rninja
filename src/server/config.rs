//! Server configuration

use serde::Deserialize;
use std::path::PathBuf;
use std::time::Duration;

/// Cache server configuration
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// Listen address (e.g., "tcp://0.0.0.0:9999")
    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,

    /// Storage directory for cache data
    #[serde(default = "default_storage_dir")]
    pub storage_dir: PathBuf,

    /// Maximum storage size in bytes (None = unlimited)
    #[serde(default)]
    pub max_storage_size: Option<u64>,

    /// Authentication configuration
    #[serde(default)]
    pub auth: AuthConfig,

    /// Number of worker threads (0 = auto)
    #[serde(default)]
    pub workers: usize,

    /// Maximum concurrent connections
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,

    /// Cache entry TTL (None = no expiry)
    #[serde(default)]
    pub entry_ttl_secs: Option<u64>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            listen_addr: default_listen_addr(),
            storage_dir: default_storage_dir(),
            max_storage_size: None,
            auth: AuthConfig::default(),
            workers: 0,
            max_connections: default_max_connections(),
            entry_ttl_secs: None,
        }
    }
}

impl ServerConfig {
    /// Load configuration from a TOML file
    pub fn from_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: ServerConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Create config from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(addr) = std::env::var("RNINJA_SERVER_LISTEN") {
            config.listen_addr = addr;
        }

        if let Ok(dir) = std::env::var("RNINJA_SERVER_STORAGE") {
            config.storage_dir = PathBuf::from(dir);
        }

        if let Ok(size) = std::env::var("RNINJA_SERVER_MAX_SIZE") {
            if let Ok(bytes) = parse_size(&size) {
                config.max_storage_size = Some(bytes);
            }
        }

        if let Ok(tokens) = std::env::var("RNINJA_SERVER_TOKENS") {
            config.auth.tokens = tokens.split(',').map(|s| s.trim().to_string()).collect();
        }

        if let Ok(ttl) = std::env::var("RNINJA_SERVER_ENTRY_TTL") {
            if let Ok(secs) = ttl.parse::<u64>() {
                config.entry_ttl_secs = Some(secs);
            }
        }

        config
    }

    /// Get entry TTL as Duration
    pub fn entry_ttl(&self) -> Option<Duration> {
        self.entry_ttl_secs.map(Duration::from_secs)
    }
}

/// Authentication configuration
#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    /// Valid authentication tokens
    #[serde(default)]
    pub tokens: Vec<String>,

    /// Whether authentication is required
    #[serde(default = "default_require_auth")]
    pub require_auth: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            tokens: Vec::new(),
            require_auth: default_require_auth(),
        }
    }
}

fn default_listen_addr() -> String {
    "tcp://0.0.0.0:9999".to_string()
}

fn default_storage_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_DATA_HOME") {
        return PathBuf::from(dir).join("rninja-cached");
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("rninja-cached");
    }
    PathBuf::from("/var/cache/rninja-cached")
}

fn default_max_connections() -> usize {
    100
}

fn default_require_auth() -> bool {
    true
}

/// Parse a size string like "1G", "500M", "1024K", or just bytes
fn parse_size(s: &str) -> Result<u64, ()> {
    let s = s.trim();
    if s.is_empty() {
        return Err(());
    }

    let (num, mult) = if s.ends_with('G') || s.ends_with('g') {
        (&s[..s.len() - 1], 1024u64 * 1024 * 1024)
    } else if s.ends_with('M') || s.ends_with('m') {
        (&s[..s.len() - 1], 1024u64 * 1024)
    } else if s.ends_with('K') || s.ends_with('k') {
        (&s[..s.len() - 1], 1024u64)
    } else {
        (s, 1u64)
    };

    num.parse::<u64>().map(|n| n * mult).map_err(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ServerConfig::default();
        assert_eq!(config.listen_addr, "tcp://0.0.0.0:9999");
        assert!(config.auth.require_auth);
    }

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("1024"), Ok(1024));
        assert_eq!(parse_size("1K"), Ok(1024));
        assert_eq!(parse_size("1M"), Ok(1024 * 1024));
        assert_eq!(parse_size("1G"), Ok(1024 * 1024 * 1024));
    }
}
