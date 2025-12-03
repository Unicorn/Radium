//! Error types for the storage layer.

use thiserror::Error;

/// Errors that can occur in the storage layer.
#[derive(Error, Debug)]
pub enum StorageError {
    /// Database connection error.
    #[error("Database connection error: {0}")]
    Connection(#[from] rusqlite::Error),

    /// Item not found in storage.
    #[error("Not found: {0}")]
    NotFound(String),

    /// Serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Invalid data error.
    #[error("Invalid data: {0}")]
    InvalidData(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type alias for storage operations.
pub type StorageResult<T> = std::result::Result<T, StorageError>;
