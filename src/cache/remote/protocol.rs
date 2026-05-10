//! Remote cache protocol types for async-nng communication
//!
//! Uses MessagePack for compact binary serialization.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

/// Protocol version for compatibility checking
pub const PROTOCOL_VERSION: u32 = 1;

/// Default chunk size for blob transfers (1MB)
pub const DEFAULT_CHUNK_SIZE: usize = 1024 * 1024;

/// Authentication header included with every request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthHeader {
    /// Bearer token for authentication
    pub token: String,
    /// Optional client identifier for tracking
    pub client_id: Option<String>,
}

/// Request envelope containing auth and payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestEnvelope {
    /// Protocol version
    pub version: u32,
    /// Authentication header
    pub auth: AuthHeader,
    /// The actual request
    pub request: Request,
}

impl RequestEnvelope {
    pub fn new(token: String, client_id: Option<String>, request: Request) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            auth: AuthHeader { token, client_id },
            request,
        }
    }
}

/// Cache entry for wire transfer (serde-compatible version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireCacheEntry {
    /// The command that was executed
    pub command: String,
    /// Output files and their content hashes
    pub outputs: Vec<(String, String)>, // (path, hash)
    /// Creation timestamp as Unix epoch seconds
    pub created_secs: u64,
    /// Creation timestamp nanoseconds component
    pub created_nanos: u32,
}

impl WireCacheEntry {
    pub fn from_entry(command: &str, outputs: &[(PathBuf, String)], created: SystemTime) -> Self {
        let duration = created
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();

        Self {
            command: command.to_string(),
            outputs: outputs
                .iter()
                .map(|(p, h)| (p.to_string_lossy().to_string(), h.clone()))
                .collect(),
            created_secs: duration.as_secs(),
            created_nanos: duration.subsec_nanos(),
        }
    }

    pub fn to_outputs(&self) -> Vec<(PathBuf, String)> {
        self.outputs
            .iter()
            .map(|(p, h)| (PathBuf::from(p), h.clone()))
            .collect()
    }

    pub fn created_time(&self) -> SystemTime {
        SystemTime::UNIX_EPOCH + std::time::Duration::new(self.created_secs, self.created_nanos)
    }
}

/// Request types sent from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    /// Check if cache entries exist (batch operation)
    Exists { keys: Vec<String> },

    /// Lookup cache entry metadata (no blobs transferred)
    Lookup { key: String },

    /// Pull blob data for given hashes
    PullBlobs { hashes: Vec<String> },

    /// Push cache entry metadata
    PushEntry { key: String, entry: WireCacheEntry },

    /// Push blob data (chunked transfer)
    PushBlob {
        hash: String,
        data: Vec<u8>,
        offset: u64,
        total: u64,
    },

    /// Complete a chunked blob push
    PushBlobComplete { hash: String, checksum: String },

    /// Health check / ping
    Ping,

    /// Get server statistics
    Stats,
}

/// Error codes returned by the server
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    /// Entry or blob not found
    NotFound,
    /// Authentication required
    AuthRequired,
    /// Authentication failed (invalid token)
    AuthFailed,
    /// Invalid request format
    InvalidRequest,
    /// Server storage is full
    StorageFull,
    /// Internal server error
    ServerError,
    /// Rate limited - too many requests
    RateLimited,
    /// Protocol version mismatch
    VersionMismatch,
}

/// Response types sent from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    /// Result of existence check
    Exists { results: HashMap<String, bool> },

    /// Cache entry found
    Entry { entry: WireCacheEntry },

    /// Entry or blob not found
    NotFound,

    /// Blob data chunk
    BlobChunk {
        hash: String,
        data: Vec<u8>,
        offset: u64,
        total: u64,
    },

    /// All chunks for a blob have been sent
    BlobComplete { hash: String },

    /// Operation succeeded
    Ok,

    /// Error response
    Error { code: ErrorCode, message: String },

    /// Pong response to ping
    Pong { version: String, server_time: u64 },

    /// Server statistics
    Statistics {
        entries: u64,
        blobs: u64,
        total_size_bytes: u64,
        uptime_secs: u64,
    },
}

impl Response {
    pub fn error(code: ErrorCode, message: impl Into<String>) -> Self {
        Response::Error {
            code,
            message: message.into(),
        }
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Response::Error { .. })
    }
}

/// Serialize a request envelope to MessagePack bytes
pub fn serialize_request(envelope: &RequestEnvelope) -> Result<Vec<u8>, rmp_serde::encode::Error> {
    rmp_serde::to_vec(envelope)
}

/// Deserialize a request envelope from MessagePack bytes
pub fn deserialize_request(data: &[u8]) -> Result<RequestEnvelope, rmp_serde::decode::Error> {
    rmp_serde::from_slice(data)
}

/// Serialize a response to MessagePack bytes
pub fn serialize_response(response: &Response) -> Result<Vec<u8>, rmp_serde::encode::Error> {
    rmp_serde::to_vec(response)
}

/// Deserialize a response from MessagePack bytes
pub fn deserialize_response(data: &[u8]) -> Result<Response, rmp_serde::decode::Error> {
    rmp_serde::from_slice(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_roundtrip() {
        let envelope = RequestEnvelope::new(
            "test-token".to_string(),
            Some("test-client".to_string()),
            Request::Lookup {
                key: "abc123".to_string(),
            },
        );

        let data = serialize_request(&envelope).unwrap();
        let restored = deserialize_request(&data).unwrap();

        assert_eq!(restored.version, PROTOCOL_VERSION);
        assert_eq!(restored.auth.token, "test-token");
        assert_eq!(restored.auth.client_id, Some("test-client".to_string()));
    }

    #[test]
    fn test_response_roundtrip() {
        let response = Response::Entry {
            entry: WireCacheEntry {
                command: "gcc -c foo.c".to_string(),
                outputs: vec![("foo.o".to_string(), "hash123".to_string())],
                created_secs: 1234567890,
                created_nanos: 123456789,
            },
        };

        let data = serialize_response(&response).unwrap();
        let restored = deserialize_response(&data).unwrap();

        if let Response::Entry { entry } = restored {
            assert_eq!(entry.command, "gcc -c foo.c");
            assert_eq!(entry.outputs.len(), 1);
        } else {
            panic!("Expected Entry response");
        }
    }

    #[test]
    fn test_error_response() {
        let response = Response::error(ErrorCode::AuthFailed, "Invalid token");
        let data = serialize_response(&response).unwrap();
        let restored = deserialize_response(&data).unwrap();

        if let Response::Error { code, message } = restored {
            assert_eq!(code, ErrorCode::AuthFailed);
            assert_eq!(message, "Invalid token");
        } else {
            panic!("Expected Error response");
        }
    }
}
