//! Client module for connecting to the rninja daemon
//!
//! Provides connection management and auto-spawn logic for the daemon.

mod connection;

pub use connection::{DaemonClient, DaemonConnection};
