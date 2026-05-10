//! rninja-cached - Remote cache server for rninja
//!
//! This binary runs the cache server that clients connect to for remote caching.

use clap::Parser;
use std::path::PathBuf;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

// Import server module from the main crate
use rninja::server::{run_server, ServerConfig};

#[derive(Parser, Debug)]
#[command(name = "rninja-cached")]
#[command(author, version, about = "Remote cache server for rninja")]
struct Cli {
    /// Configuration file path
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Listen address (e.g., tcp://0.0.0.0:9999)
    #[arg(short, long, value_name = "ADDR")]
    listen: Option<String>,

    /// Storage directory
    #[arg(short, long, value_name = "DIR")]
    storage: Option<PathBuf>,

    /// Authentication tokens (comma-separated)
    #[arg(short = 't', long, value_name = "TOKENS")]
    tokens: Option<String>,

    /// Maximum storage size (e.g., 10G, 500M)
    #[arg(short = 'm', long, value_name = "SIZE")]
    max_size: Option<String>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Initialize logging
    let level = if cli.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };
    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // Load configuration
    let mut config = if let Some(ref config_path) = cli.config {
        match ServerConfig::from_file(config_path) {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to load config from {:?}: {}", config_path, e);
                std::process::exit(1);
            }
        }
    } else {
        ServerConfig::from_env()
    };

    // Override with CLI arguments
    if let Some(listen) = cli.listen {
        config.listen_addr = listen;
    }
    if let Some(storage) = cli.storage {
        config.storage_dir = storage;
    }
    if let Some(tokens) = cli.tokens {
        config.auth.tokens = tokens.split(',').map(|s| s.trim().to_string()).collect();
    }
    if let Some(max_size) = cli.max_size {
        if let Some(bytes) = parse_size(&max_size) {
            config.max_storage_size = Some(bytes);
        }
    }

    // Validate configuration
    if config.auth.require_auth && config.auth.tokens.is_empty() {
        error!("Authentication is required but no tokens are configured");
        error!("Either set RNINJA_SERVER_TOKENS or use --tokens, or disable auth in config");
        std::process::exit(1);
    }

    info!("rninja-cached starting...");
    info!("Listen address: {}", config.listen_addr);
    info!("Storage directory: {}", config.storage_dir.display());
    if let Some(max_size) = config.max_storage_size {
        info!("Max storage size: {} bytes", max_size);
    }
    info!(
        "Authentication: {} tokens configured",
        config.auth.tokens.len()
    );

    // Run server
    if let Err(e) = run_server(config).await {
        error!("Server error: {}", e);
        std::process::exit(1);
    }
}

fn parse_size(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
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

    num.parse::<u64>().ok().map(|n| n * mult)
}
