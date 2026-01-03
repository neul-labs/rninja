//! Remote cache error types

use super::protocol::ErrorCode;
use thiserror::Error;

/// Errors that can occur during remote cache operations
#[derive(Debug, Error)]
pub enum RemoteCacheError {
    /// Failed to connect to the cache server
    #[error("connection failed: {0}")]
    ConnectionFailed(String),

    /// Connection timed out
    #[error("connection timeout after {0}ms")]
    ConnectionTimeout(u64),

    /// Request timed out
    #[error("request timeout after {0}ms")]
    RequestTimeout(u64),

    /// Authentication failed
    #[error("authentication failed: {0}")]
    AuthFailed(String),

    /// Server returned an error
    #[error("server error ({code:?}): {message}")]
    ServerError { code: ErrorCode, message: String },

    /// Rate limited by server
    #[error("rate limited: {0}")]
    RateLimited(String),

    /// Network I/O error
    #[error("network error: {0}")]
    NetworkError(String),

    /// Protocol/serialization error
    #[error("protocol error: {0}")]
    ProtocolError(String),

    /// Invalid server response
    #[error("invalid response: {0}")]
    InvalidResponse(String),

    /// Client has been shut down
    #[error("client shutdown")]
    Shutdown,

    /// Blob transfer failed
    #[error("blob transfer failed: {0}")]
    BlobTransferFailed(String),

    /// Checksum mismatch during transfer
    #[error("checksum mismatch for blob {hash}: expected {expected}, got {actual}")]
    ChecksumMismatch {
        hash: String,
        expected: String,
        actual: String,
    },

    /// Configuration error
    #[error("configuration error: {0}")]
    ConfigError(String),
}

impl RemoteCacheError {
    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            RemoteCacheError::ConnectionTimeout(_)
                | RemoteCacheError::RequestTimeout(_)
                | RemoteCacheError::NetworkError(_)
                | RemoteCacheError::RateLimited(_)
                | RemoteCacheError::ServerError {
                    code: ErrorCode::ServerError,
                    ..
                }
        )
    }

    /// Check if this is a fatal error that should stop retries
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            RemoteCacheError::AuthFailed(_)
                | RemoteCacheError::ConfigError(_)
                | RemoteCacheError::Shutdown
                | RemoteCacheError::ServerError {
                    code: ErrorCode::AuthFailed | ErrorCode::VersionMismatch,
                    ..
                }
        )
    }
}

impl From<nng::Error> for RemoteCacheError {
    fn from(err: nng::Error) -> Self {
        RemoteCacheError::NetworkError(err.to_string())
    }
}

impl From<rmp_serde::encode::Error> for RemoteCacheError {
    fn from(err: rmp_serde::encode::Error) -> Self {
        RemoteCacheError::ProtocolError(format!("serialization failed: {}", err))
    }
}

impl From<rmp_serde::decode::Error> for RemoteCacheError {
    fn from(err: rmp_serde::decode::Error) -> Self {
        RemoteCacheError::ProtocolError(format!("deserialization failed: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retryable_errors() {
        assert!(RemoteCacheError::ConnectionTimeout(1000).is_retryable());
        assert!(RemoteCacheError::RequestTimeout(5000).is_retryable());
        assert!(RemoteCacheError::NetworkError("conn reset".into()).is_retryable());
        assert!(RemoteCacheError::RateLimited("too many requests".into()).is_retryable());

        assert!(!RemoteCacheError::AuthFailed("invalid token".into()).is_retryable());
        assert!(!RemoteCacheError::Shutdown.is_retryable());
    }

    #[test]
    fn test_fatal_errors() {
        assert!(RemoteCacheError::AuthFailed("bad token".into()).is_fatal());
        assert!(RemoteCacheError::Shutdown.is_fatal());
        assert!(RemoteCacheError::ConfigError("missing server".into()).is_fatal());

        assert!(!RemoteCacheError::ConnectionTimeout(1000).is_fatal());
        assert!(!RemoteCacheError::NetworkError("temporary".into()).is_fatal());
    }
}
