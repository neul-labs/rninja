//! rninja - A drop-in replacement for Ninja with caching and improved scheduling
//!
//! This library provides the core functionality for the rninja build system,
//! including local and remote caching, parallel execution, and Ninja compatibility.

pub mod admin;
pub mod buildlog;
pub mod cache;
pub mod cli;
pub mod client;
pub mod config;
pub mod daemon;
pub mod error;
pub mod executor;
pub mod graph;
pub mod metrics;
pub mod output;
pub mod parser;
pub mod server;
pub mod trace;
