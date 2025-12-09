//! Error types for the privacy filtering module.

use thiserror::Error;

/// Errors that can occur during privacy filtering operations.
#[derive(Error, Debug, Clone)]
pub enum PrivacyError {
    /// Failed to compile regex pattern.
    #[error("Failed to compile pattern '{pattern}': {details}")]
    PatternError {
        pattern: String,
        details: String,
    },

    /// Error during redaction operation.
    #[error("Redaction error: {0}")]
    RedactionError(String),

    /// Invalid privacy configuration.
    #[error("Invalid privacy configuration: {0}")]
    ConfigError(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    IoError(String),
}

impl From<std::io::Error> for PrivacyError {
    fn from(err: std::io::Error) -> Self {
        PrivacyError::IoError(err.to_string())
    }
}

/// Result type alias for privacy operations.
pub type Result<T> = std::result::Result<T, PrivacyError>;

