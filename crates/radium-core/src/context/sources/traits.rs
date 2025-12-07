//! Traits for source reading.

use async_trait::async_trait;

use super::types::{SourceError, SourceMetadata};

/// A trait for reading and verifying sources from different protocols.
///
/// This trait provides a unified interface for accessing sources from various
/// protocols (file, HTTP, Jira, Braingrid, etc.). Each implementation handles
/// a specific scheme and provides both lightweight verification and full content
/// retrieval capabilities.
///
/// # Verification vs Fetch
///
/// - **`verify()`**: Performs a lightweight check to determine if the source
///   exists and is accessible. This is ideal for pre-flight validation before
///   agent execution. It should avoid downloading full content when possible.
///
/// - **`fetch()`**: Retrieves the full content of the source. This is used when
///   the actual content is needed for processing.
#[async_trait]
pub trait SourceReader: Send + Sync {
    /// Returns the URI scheme this reader handles (e.g., "file", "http", "jira").
    ///
    /// This is used by the registry to route URIs to the appropriate reader.
    fn scheme(&self) -> &str;

    /// Verifies that a source exists and is accessible without downloading full content.
    ///
    /// This method performs a lightweight check (e.g., HEAD request for HTTP,
    /// file metadata check for local files) to determine accessibility.
    ///
    /// # Arguments
    ///
    /// * `uri` - The URI of the source to verify
    ///
    /// # Returns
    ///
    /// Returns `SourceMetadata` with `accessible` set to `true` if the source
    /// exists and can be accessed, or an error if verification fails.
    ///
    /// # Errors
    ///
    /// Returns `SourceError` if the source cannot be verified, including cases
    /// where the source doesn't exist, authentication is required, or network
    /// errors occur.
    async fn verify(&self, uri: &str) -> Result<SourceMetadata, SourceError>;

    /// Fetches the full content of a source.
    ///
    /// This method retrieves the complete content of the source. For large sources,
    /// implementations should enforce size limits to prevent memory issues.
    ///
    /// # Arguments
    ///
    /// * `uri` - The URI of the source to fetch
    ///
    /// # Returns
    ///
    /// Returns the full content as a `String`, or an error if fetching fails.
    ///
    /// # Errors
    ///
    /// Returns `SourceError` if the source cannot be fetched, including cases
    /// where the source doesn't exist, authentication is required, network errors,
    /// or content exceeds size limits.
    async fn fetch(&self, uri: &str) -> Result<String, SourceError>;
}
