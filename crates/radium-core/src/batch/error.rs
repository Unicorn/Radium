//! Error types for batch processing.

use std::fmt;

/// Errors that can occur during batch processing.
#[derive(Debug, Clone)]
pub enum BatchError {
    /// An error occurred processing a specific item.
    ItemError {
        /// Index of the item that failed.
        index: usize,
        /// Input that caused the error.
        input: String,
        /// Error message.
        error: String,
        /// Type of error.
        error_type: String,
    },
    /// Batch processing was cancelled.
    Cancelled,
    /// Batch processing timed out.
    Timeout,
    /// Invalid configuration.
    InvalidConfig(String),
}

impl fmt::Display for BatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BatchError::ItemError {
                index,
                input,
                error,
                error_type,
            } => write!(
                f,
                "Item {} failed ({}): {} (input: {})",
                index, error_type, error, input
            ),
            BatchError::Cancelled => write!(f, "Batch processing was cancelled"),
            BatchError::Timeout => write!(f, "Batch processing timed out"),
            BatchError::InvalidConfig(msg) => write!(f, "Invalid configuration: {}", msg),
        }
    }
}

impl std::error::Error for BatchError {}

