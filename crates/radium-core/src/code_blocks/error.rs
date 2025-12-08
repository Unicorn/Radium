//! Error types for code block operations.

use std::fmt;

/// Errors that can occur during code block operations.
#[derive(Debug)]
pub enum CodeBlockError {
    /// I/O error during file operations.
    Io(std::io::Error),
    /// JSON serialization/deserialization error.
    Serialization(serde_json::Error),
    /// Block not found at specified index.
    NotFound(usize),
    /// Invalid block index or range.
    InvalidIndex(String),
    /// Storage directory creation failed.
    StorageCreation(String),
}

impl fmt::Display for CodeBlockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodeBlockError::Io(e) => write!(f, "I/O error: {}", e),
            CodeBlockError::Serialization(e) => write!(f, "Serialization error: {}", e),
            CodeBlockError::NotFound(index) => write!(f, "Code block at index {} not found", index),
            CodeBlockError::InvalidIndex(msg) => write!(f, "Invalid index: {}", msg),
            CodeBlockError::StorageCreation(msg) => write!(f, "Failed to create storage: {}", msg),
        }
    }
}

impl std::error::Error for CodeBlockError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CodeBlockError::Io(e) => Some(e),
            CodeBlockError::Serialization(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for CodeBlockError {
    fn from(e: std::io::Error) -> Self {
        CodeBlockError::Io(e)
    }
}

impl From<serde_json::Error> for CodeBlockError {
    fn from(e: serde_json::Error) -> Self {
        CodeBlockError::Serialization(e)
    }
}

/// Result type for code block operations.
pub type Result<T> = std::result::Result<T, CodeBlockError>;

