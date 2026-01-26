use thiserror::Error;

// ============================================================================
// Error Handling Strategy
// ============================================================================
// This module defines the error types used throughout rninja.
//
// Design principles:
// 1. **ParseError**, **GraphError**: Specific errors for parsing and graph building
// 2. **ExecError**: Errors from build execution (command failures, spawn errors)
// 3. **CacheError**: Errors from cache operations (I/O, database, serialization)
//
// Error conversions:
// - CacheError can be converted to ExecError via From (for build context)
// - anyhow::Error can be converted to CacheError for external error handling
// - This allows flexibility while maintaining type safety for core operations
// ============================================================================

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("line {line}: {message}")]
    Syntax { line: usize, message: String },

    #[error("unknown rule '{name}'")]
    UnknownRule { name: String },

    #[error("duplicate rule '{name}'")]
    DuplicateRule { name: String },

    #[error("missing required variable '{var}' in rule '{rule}'")]
    MissingVariable { var: String, rule: String },

    #[error("circular include detected: {path}")]
    CircularInclude { path: String },
}

#[derive(Error, Debug)]
pub enum GraphError {
    #[error("unknown target '{target}'")]
    UnknownTarget { target: String },

    #[error("dependency cycle detected involving '{target}'")]
    Cycle { target: String },

    #[error("multiple rules generate '{output}'")]
    DuplicateOutput { output: String },
}

#[derive(Error, Debug)]
pub enum ExecError {
    #[error("command failed with exit code {code}: {command}")]
    CommandFailed { command: String, code: i32 },

    #[error("subcommand failed")]
    SubcommandFailed,

    #[error("failed to spawn command: {0}")]
    SpawnError(#[from] std::io::Error),

    #[error("build stopped: {0} targets failed")]
    BuildFailed(usize),
}

/// Errors that can occur during cache operations.
///
/// This error type wraps the various failure modes of the content-addressed
/// cache including database errors, blob I/O errors, and serialization errors.
#[derive(Error, Debug)]
pub enum CacheError {
    /// Database operation failed (sled)
    #[error("database error: {0}")]
    Database(#[from] sled::Error),

    /// File I/O error
    #[error("blob I/O error: {0}")]
    BlobIo(#[from] std::io::Error),

    /// Failed to serialize or deserialize cache data
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Failed to compute content hash
    #[error("hash computation failed: {0}")]
    HashError(String),

    /// The cache directory could not be created or accessed
    #[error("cache directory error: {0}")]
    CacheDir(String),
}

impl From<anyhow::Error> for CacheError {
    /// Convert an anyhow error to CacheError.
    ///
    /// This allows external errors (e.g., from optional dependencies) to be
    /// handled within the cache error hierarchy.
    fn from(e: anyhow::Error) -> Self {
        CacheError::Serialization(e.to_string())
    }
}

impl From<CacheError> for ExecError {
    /// Convert a CacheError to ExecError for build context.
    ///
    /// This allows cache operations to fail gracefully within the build
    /// pipeline without stopping the entire build.
    fn from(e: CacheError) -> Self {
        ExecError::SpawnError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("cache error: {}", e),
        ))
    }
}
