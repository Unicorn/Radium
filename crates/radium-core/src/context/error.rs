//! Context system error types.

use std::io;

/// Context system errors.
#[derive(Debug, thiserror::Error)]
pub enum ContextError {
    /// I/O error during context operations.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// File not found.
    #[error("file not found: {0}")]
    FileNotFound(String),

    /// Invalid injection syntax.
    #[error("invalid injection syntax: {0}")]
    InvalidSyntax(String),

    /// Memory error.
    #[error("memory error: {0}")]
    Memory(#[from] crate::memory::MemoryError),

    /// Workspace error.
    #[error("workspace error: {0}")]
    Workspace(#[from] crate::workspace::WorkspaceError),

    /// Invalid tail context size.
    #[error("invalid tail context size: {0}")]
    InvalidTailSize(String),
}

/// Result type for context operations.
pub type Result<T> = std::result::Result<T, ContextError>;
