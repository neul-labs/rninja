use std::path::PathBuf;
use std::time::Duration;

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Whether caching is enabled
    pub enabled: bool,
    /// Cache directory path
    pub cache_dir: PathBuf,
    /// Maximum age for cache entries (None = no expiry)
    pub max_age: Option<Duration>,
    /// Maximum cache size in bytes (None = no limit)
    pub max_size: Option<u64>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cache_dir: default_cache_dir(),
            max_age: None,
            max_size: None,
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

        config
    }

    /// Disable caching
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Self::default()
        }
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
