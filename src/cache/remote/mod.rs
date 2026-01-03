//! Remote cache module
//!
//! Provides remote cache functionality using NNG transport with MessagePack serialization.

pub mod client;
pub mod error;
pub mod protocol;

pub use client::{ClientStats, RemoteCacheClient, RemoteClientConfig, RetryConfig};
pub use error::RemoteCacheError;
pub use protocol::{
    ErrorCode, Request, RequestEnvelope, Response, WireCacheEntry, PROTOCOL_VERSION,
};
