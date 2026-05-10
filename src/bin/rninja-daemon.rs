//! rninja-daemon - Long-running build daemon
//!
//! Caches parsed manifests and dependency graphs for faster incremental builds.

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

// Import from the main crate
use rninja::daemon::protocol::get_default_socket_path;
use rninja::daemon::server::DaemonServer;
use rninja::daemon::state::DaemonConfig;

/// rninja-daemon - Build daemon for rninja
#[derive(Parser, Debug)]
#[command(name = "rninja-daemon", version, about)]
struct Args {
    /// Socket path to listen on
    #[arg(short, long)]
    socket: Option<PathBuf>,

    /// Maximum number of concurrent builds
    #[arg(short = 'j', long, default_value = "4")]
    max_builds: usize,

    /// Maximum cached manifests
    #[arg(long, default_value = "100")]
    max_cached: usize,

    /// Run in foreground (don't daemonize)
    #[arg(short, long)]
    foreground: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Set up logging
    let level = if args.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };
    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_target(false)
        .with_file(false)
        .with_line_number(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Determine socket path
    let socket_path = args.socket.unwrap_or_else(get_default_socket_path);

    info!("rninja-daemon v{}", env!("CARGO_PKG_VERSION"));
    info!("Socket: {}", socket_path.display());

    // Create daemon config
    let config = DaemonConfig {
        max_cached_manifests: args.max_cached,
        max_concurrent_builds: args.max_builds,
        ..Default::default()
    };

    // Create and start server
    let mut server = DaemonServer::new(socket_path, config)?;

    // Set up signal handlers
    let shutdown = server.shutdown_handle();
    ctrlc::set_handler(move || {
        info!("Received shutdown signal");
        shutdown.store(true, std::sync::atomic::Ordering::SeqCst);
    })?;

    // Start listening
    server.start()?;

    // Run main loop
    server.run()?;

    info!("Daemon stopped");
    Ok(())
}
