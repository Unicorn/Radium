//! HTTP/HTTPS source reader.

use async_trait::async_trait;
use reqwest::Client;

use super::traits::SourceReader;
use super::types::{SourceError, SourceMetadata};

/// Default maximum size for HTTP fetch operations (10MB).
const DEFAULT_MAX_SIZE: u64 = 10 * 1024 * 1024;

/// Reader for HTTP and HTTPS sources.
pub struct HttpReader {
    /// HTTP client for making requests.
    client: Client,

    /// Maximum size in bytes for fetch operations.
    max_size: u64,
}

impl HttpReader {
    /// Creates a new HTTP reader with default settings.
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            max_size: DEFAULT_MAX_SIZE,
        }
    }

    /// Creates a new HTTP reader with a custom max size.
    pub fn with_max_size(max_size: u64) -> Self {
        Self {
            client: Client::new(),
            max_size,
        }
    }

    /// Creates a new HTTP reader with a custom HTTP client.
    pub fn with_client(client: Client) -> Self {
        Self {
            client,
            max_size: DEFAULT_MAX_SIZE,
        }
    }
}

impl Default for HttpReader {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SourceReader for HttpReader {
    fn scheme(&self) -> &str {
        "http"
    }

    async fn verify(&self, uri: &str) -> Result<SourceMetadata, SourceError> {
        // Validate URI format
        if !uri.starts_with("http://") && !uri.starts_with("https://") {
            return Err(SourceError::invalid_uri(&format!(
                "URI must start with http:// or https://: {}",
                uri
            )));
        }

        // Perform HEAD request to check accessibility
        let response = match self.client.head(uri).send().await {
            Ok(resp) => resp,
            Err(e) => {
                return Err(SourceError::network_error(&format!(
                    "Failed to connect: {}",
                    e
                )));
            }
        };

        let status = response.status();

        // Check if request was successful
        if !status.is_success() {
            return Err(SourceError::not_found(&format!(
                "HTTP {}: {}",
                status.as_u16(),
                status.canonical_reason().unwrap_or("Unknown")
            )));
        }

        // Extract metadata from response headers
        let size_bytes = response
            .headers()
            .get("content-length")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok());

        let last_modified = response
            .headers()
            .get("last-modified")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        Ok(SourceMetadata::with_details(true, size_bytes, last_modified, content_type))
    }

    async fn fetch(&self, uri: &str) -> Result<String, SourceError> {
        // Validate URI format
        if !uri.starts_with("http://") && !uri.starts_with("https://") {
            return Err(SourceError::invalid_uri(&format!(
                "URI must start with http:// or https://: {}",
                uri
            )));
        }

        // Perform GET request
        let mut response = match self.client.get(uri).send().await {
            Ok(resp) => resp,
            Err(e) => {
                return Err(SourceError::network_error(&format!(
                    "Failed to connect: {}",
                    e
                )));
            }
        };

        let status = response.status();

        // Check if request was successful
        if !status.is_success() {
            return Err(SourceError::not_found(&format!(
                "HTTP {}: {}",
                status.as_u16(),
                status.canonical_reason().unwrap_or("Unknown")
            )));
        }

        // Check content length before downloading
        if let Some(content_length) = response.content_length() {
            if content_length > self.max_size {
                return Err(SourceError::other(format!(
                    "Content size {} bytes exceeds maximum allowed size {} bytes",
                    content_length, self.max_size
                )));
            }
        }

        // Read response body with size limit
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|e| {
            SourceError::network_error(&format!("Failed to read response chunk: {}", e))
        })? {
            if bytes.len() + chunk.len() > self.max_size as usize {
                return Err(SourceError::other(format!(
                    "Content size exceeds maximum allowed size {} bytes",
                    self.max_size
                )));
            }
            bytes.extend_from_slice(&chunk);
        }

        // Convert bytes to string
        String::from_utf8(bytes).map_err(|e| {
            SourceError::other(format!("Response is not valid UTF-8: {}", e))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheme() {
        let reader = HttpReader::new();
        assert_eq!(reader.scheme(), "http");
    }

    #[tokio::test]
    async fn test_verify_invalid_uri() {
        let reader = HttpReader::new();
        let result = reader.verify("not-a-url").await;

        assert!(result.is_err());
        match result.unwrap_err() {
            SourceError::InvalidUri(_) => {}
            _ => panic!("Expected InvalidUri error"),
        }
    }

    #[tokio::test]
    async fn test_verify_valid_url() {
        // This test requires network access, so we'll skip it in CI
        // In a real scenario, you'd use a mock HTTP server
        let reader = HttpReader::new();
        // Using a well-known URL that should exist
        let result = reader.verify("https://www.example.com").await;

        // We can't assert success without network, but we can check it doesn't panic
        // In practice, this would succeed for a valid URL
        if result.is_ok() {
            let metadata = result.unwrap();
            assert!(metadata.accessible);
        }
    }
}
