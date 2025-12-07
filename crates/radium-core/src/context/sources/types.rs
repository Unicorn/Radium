//! Types for source reading and verification.

use thiserror::Error;

/// Metadata about a source, returned after verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceMetadata {
    /// Whether the source is accessible.
    pub accessible: bool,

    /// Size of the source in bytes, if known.
    pub size_bytes: Option<u64>,

    /// Last modification time as an ISO 8601 string, if known.
    pub last_modified: Option<String>,

    /// Content type/MIME type of the source, if known.
    pub content_type: Option<String>,
}

impl SourceMetadata {
    /// Creates new source metadata.
    pub fn new(accessible: bool) -> Self {
        Self {
            accessible,
            size_bytes: None,
            last_modified: None,
            content_type: None,
        }
    }

    /// Creates source metadata with all fields.
    pub fn with_details(
        accessible: bool,
        size_bytes: Option<u64>,
        last_modified: Option<String>,
        content_type: Option<String>,
    ) -> Self {
        Self {
            accessible,
            size_bytes,
            last_modified,
            content_type,
        }
    }
}

/// Errors that can occur during source reading operations.
#[derive(Debug, Error)]
pub enum SourceError {
    /// Source not found.
    #[error("source not found: {0}")]
    NotFound(String),

    /// Authentication required or failed.
    #[error("authentication required: {0}")]
    Unauthorized(String),

    /// Network error occurred.
    #[error("network error: {0}")]
    NetworkError(String),

    /// Invalid URI format.
    #[error("invalid URI: {0}")]
    InvalidUri(String),

    /// I/O error occurred.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Generic error with message.
    #[error("{0}")]
    Other(String),
}

impl SourceError {
    /// Creates a new NotFound error.
    pub fn not_found(uri: &str) -> Self {
        Self::NotFound(uri.to_string())
    }

    /// Creates a new Unauthorized error.
    pub fn unauthorized(message: &str) -> Self {
        Self::Unauthorized(message.to_string())
    }

    /// Creates a new NetworkError.
    pub fn network_error(message: &str) -> Self {
        Self::NetworkError(message.to_string())
    }

    /// Creates a new InvalidUri error.
    pub fn invalid_uri(uri: &str) -> Self {
        Self::InvalidUri(uri.to_string())
    }

    /// Creates a new Other error.
    pub fn other(message: impl Into<String>) -> Self {
        Self::Other(message.into())
    }
}
