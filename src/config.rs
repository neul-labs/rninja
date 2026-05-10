//! Configuration file support for rninja
//!
//! Loads configuration from:
//! 1. ~/.config/rninja/config.toml (XDG style)
//! 2. ~/.rninjarc (traditional)
//! 3. .rninjarc (project local)

use serde::Deserialize;
use std::path::PathBuf;
use tracing::debug;

/// User configuration loaded from config file
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Build settings
    pub build: BuildConfig,
    /// Cache settings
    pub cache: CacheConfig,
    /// Output settings
    pub output: OutputConfig,
}

/// Build-related configuration
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct BuildConfig {
    /// Default number of parallel jobs (0 = number of CPUs)
    pub jobs: usize,
    /// Default keep_going value (0 = keep going forever)
    pub keep_going: usize,
    /// Whether to explain why targets are rebuilt
    pub explain: bool,
    /// Default targets to build (empty = use manifest defaults)
    pub default_targets: Vec<String>,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            jobs: 0, // 0 means use number of CPUs
            keep_going: 1,
            explain: false,
            default_targets: Vec::new(),
        }
    }
}

/// Cache-related configuration
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct CacheConfig {
    /// Whether cache is enabled
    pub enabled: bool,
    /// Cache mode: "local", "remote", or "auto"
    pub mode: String,
    /// Cache directory (default: ~/.cache/rninja)
    pub directory: Option<String>,
    /// Maximum cache size in bytes (default: 10GB)
    pub max_size: Option<u64>,
    /// Remote cache daemon socket path
    pub daemon_socket: Option<String>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: "auto".to_string(),
            directory: None,
            max_size: None,
            daemon_socket: None,
        }
    }
}

/// Output-related configuration
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct OutputConfig {
    /// Show verbose output by default
    pub verbose: bool,
    /// Show build statistics at the end
    pub stats: bool,
    /// Enable colored output (true, false, or "auto")
    pub color: String,
    /// Trace output file (if set, always generate trace)
    pub trace_file: Option<String>,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            stats: false,
            color: "auto".to_string(),
            trace_file: None,
        }
    }
}

impl Config {
    /// Load configuration from default locations
    pub fn load() -> Self {
        // Try config locations in order of precedence
        let config_paths = Self::config_paths();

        for path in config_paths {
            if path.exists() {
                debug!("Loading config from {:?}", path);
                match Self::load_from_file(&path) {
                    Ok(config) => return config,
                    Err(e) => {
                        debug!("Failed to load config from {:?}: {}", path, e);
                    }
                }
            }
        }

        // Return default config if no config file found
        Self::default()
    }

    /// Load configuration from a specific file
    pub fn load_from_file(path: &PathBuf) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        toml::from_str(&content).map_err(|e| format!("Failed to parse config file: {}", e))
    }

    /// Get list of config file paths to check
    fn config_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Project local config (highest precedence)
        paths.push(PathBuf::from(".rninjarc"));

        // User config locations
        if let Some(home) = dirs_next::home_dir() {
            // Traditional location
            paths.push(home.join(".rninjarc"));
        }

        if let Some(config_dir) = dirs_next::config_dir() {
            // XDG style
            paths.push(config_dir.join("rninja").join("config.toml"));
        }

        paths
    }

    /// Get the default config file path for writing
    pub fn default_config_path() -> Option<PathBuf> {
        dirs_next::config_dir().map(|d| d.join("rninja").join("config.toml"))
    }

    /// Generate a sample config file
    pub fn sample_config() -> &'static str {
        r#"# rninja configuration file

[build]
# Number of parallel jobs (0 = number of CPUs)
jobs = 0

# How many failures to allow before stopping (0 = keep going forever)
keep_going = 1

# Whether to explain why targets are rebuilt
explain = false

# Default targets to build (empty = use manifest defaults)
default_targets = []

[cache]
# Enable the build cache
enabled = true

# Cache mode: "local", "remote", or "auto"
# auto = try remote first, fall back to local
mode = "auto"

# Cache directory (default: ~/.cache/rninja)
# directory = "/path/to/cache"

# Maximum cache size in bytes (default: 10GB)
# max_size = 10737418240

# Remote cache daemon socket path
# daemon_socket = "/tmp/rninja-daemon.sock"

[output]
# Show verbose output by default
verbose = false

# Show build statistics at the end
stats = false

# Enable colored output: "auto", "always", or "never"
color = "auto"

# Trace output file (if set, always generate trace)
# trace_file = "build_trace.json"
"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.build.jobs, 0);
        assert!(config.cache.enabled);
        assert!(!config.output.verbose);
    }

    #[test]
    fn test_parse_sample_config() {
        let sample = Config::sample_config();
        let config: Config = toml::from_str(sample).expect("Failed to parse sample config");
        assert_eq!(config.build.jobs, 0);
        assert!(config.cache.enabled);
    }
}
