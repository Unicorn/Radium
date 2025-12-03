//! Error types for checkpoint operations.

use std::io;
use thiserror::Error;

/// Result type for checkpoint operations.
pub type Result<T> = std::result::Result<T, CheckpointError>;

/// Errors that can occur during checkpoint operations.
#[derive(Debug, Error)]
pub enum CheckpointError {
    /// Git command execution failed.
    #[error("Git command failed: {0}")]
    GitCommandFailed(String),

    /// Git repository not found.
    #[error("Git repository not found at: {0}")]
    RepositoryNotFound(String),

    /// Checkpoint not found.
    #[error("Checkpoint not found: {0}")]
    CheckpointNotFound(String),

    /// Invalid checkpoint ID.
    #[error("Invalid checkpoint ID: {0}")]
    InvalidCheckpointId(String),

    /// Shadow repository initialization failed.
    #[error("Failed to initialize shadow repository: {0}")]
    ShadowRepoInitFailed(String),

    /// Restore operation failed.
    #[error("Failed to restore checkpoint: {0}")]
    RestoreFailed(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// UTF-8 conversion error.
    #[error("UTF-8 conversion error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}
