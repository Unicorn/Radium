//! Memory system error types.

use std::io;

/// Memory system errors.
#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    /// I/O error during memory operations.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Memory entry not found.
    #[error("memory entry not found: {0}")]
    NotFound(String),

    /// Invalid agent ID.
    #[error("invalid agent ID: {0}")]
    InvalidAgentId(String),

    /// Memory store not initialized.
    #[error("memory store not initialized for plan: {0}")]
    NotInitialized(String),

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Result type for memory operations.
pub type Result<T> = std::result::Result<T, MemoryError>;
