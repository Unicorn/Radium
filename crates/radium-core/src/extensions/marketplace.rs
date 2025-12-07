//! Extension marketplace discovery API.
//!
//! Provides functionality for discovering and fetching extension metadata
//! from a remote marketplace.

use crate::extensions::manifest::ExtensionManifest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use thiserror::Error;

/// Marketplace client errors.
#[derive(Debug, Error)]
pub enum MarketplaceError {
    /// HTTP request error.
    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON parsing error.
    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    /// Invalid response format.
    #[error("invalid response format: {0}")]
    InvalidResponse(String),

    /// Network timeout.
    #[error("network timeout")]
    Timeout,
}

/// Result type for marketplace operations.
pub type Result<T> = std::result::Result<T, MarketplaceError>;

/// Marketplace extension metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceExtension {
    /// Extension name.
    pub name: String,

    /// Extension version.
    pub version: String,

    /// Extension description.
    pub description: String,

    /// Extension author.
    pub author: String,

    /// Download URL for the extension package.
    pub download_url: String,

    /// Optional download count.
    #[serde(default)]
    pub download_count: Option<u64>,

    /// Optional rating.
    #[serde(default)]
    pub rating: Option<f64>,

    /// Optional tags/categories.
    #[serde(default)]
    pub tags: Vec<String>,

    /// Full manifest (optional, may be included in search results).
    #[serde(default)]
    pub manifest: Option<ExtensionManifest>,
}

/// Cached marketplace data.
#[derive(Debug, Clone)]
struct CachedData {
    data: Vec<MarketplaceExtension>,
    timestamp: Instant,
}

/// Marketplace client for discovering and fetching extensions.
pub struct MarketplaceClient {
    /// Base URL for the marketplace API.
    base_url: String,

    /// HTTP client.
    client: reqwest::blocking::Client,

    /// Cache for marketplace responses.
    cache: HashMap<String, CachedData>,

    /// Cache TTL in seconds.
    cache_ttl: Duration,
}

impl MarketplaceClient {
    /// Creates a new marketplace client with default configuration.
    ///
    /// # Returns
    /// New marketplace client instance
    pub fn new() -> Result<Self> {
        Self::with_url(
            std::env::var("RADIUM_MARKETPLACE_URL")
                .unwrap_or_else(|_| "https://marketplace.radium.dev/api/v1".to_string()),
        )
    }

    /// Creates a new marketplace client with custom base URL.
    ///
    /// # Arguments
    /// * `base_url` - Base URL for the marketplace API
    ///
    /// # Returns
    /// New marketplace client instance
    pub fn with_url(base_url: String) -> Result<Self> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(MarketplaceError::Http)?;

        Ok(Self {
            base_url,
            client,
            cache: HashMap::new(),
            cache_ttl: Duration::from_secs(300), // 5 minutes default TTL
        })
    }

    /// Sets the cache TTL.
    ///
    /// # Arguments
    /// * `ttl` - Cache time-to-live duration
    pub fn set_cache_ttl(&mut self, ttl: Duration) {
        self.cache_ttl = ttl;
    }

    /// Searches for extensions in the marketplace.
    ///
    /// # Arguments
    /// * `query` - Search query string
    ///
    /// # Returns
    /// Vector of matching marketplace extensions
    ///
    /// # Errors
    /// Returns error if the search request fails
    pub fn search_extensions(&mut self, query: &str) -> Result<Vec<MarketplaceExtension>> {
        let cache_key = format!("search:{}", query);

        // Check cache
        if let Some(cached) = self.cache.get(&cache_key) {
            if cached.timestamp.elapsed() < self.cache_ttl {
                return Ok(cached.data.clone());
            }
        }

        // Build search URL
        let url = format!("{}/extensions/search?q={}", self.base_url, urlencoding::encode(query));

        // Make request with retry logic
        let response = self.make_request_with_retry(&url, 3)?;

        // Parse response
        let extensions: Vec<MarketplaceExtension> = response
            .json()
            .map_err(|e| MarketplaceError::InvalidResponse(format!("Failed to parse search results: {}", e)))?;

        // Update cache
        self.cache.insert(
            cache_key,
            CachedData {
                data: extensions.clone(),
                timestamp: Instant::now(),
            },
        );

        Ok(extensions)
    }

    /// Gets detailed information about a specific extension.
    ///
    /// # Arguments
    /// * `name` - Extension name
    ///
    /// # Returns
    /// Marketplace extension metadata if found
    ///
    /// # Errors
    /// Returns error if the request fails
    pub fn get_extension_info(&mut self, name: &str) -> Result<Option<MarketplaceExtension>> {
        let cache_key = format!("info:{}", name);

        // Check cache
        if let Some(cached) = self.cache.get(&cache_key) {
            if cached.timestamp.elapsed() < self.cache_ttl {
                return Ok(cached.data.first().cloned());
            }
        }

        // Build info URL
        let url = format!("{}/extensions/{}", self.base_url, urlencoding::encode(name));

        // Make request with retry logic
        let response = match self.make_request_with_retry(&url, 3) {
            Ok(resp) => resp,
            Err(MarketplaceError::Http(e)) if e.status() == Some(reqwest::StatusCode::NOT_FOUND) => {
                return Ok(None);
            }
            Err(e) => return Err(e),
        };

        // Parse response
        let extension: MarketplaceExtension = response
            .json()
            .map_err(|e| MarketplaceError::InvalidResponse(format!("Failed to parse extension info: {}", e)))?;

        // Update cache
        self.cache.insert(
            cache_key,
            CachedData {
                data: vec![extension.clone()],
                timestamp: Instant::now(),
            },
        );

        Ok(Some(extension))
    }

    /// Publishes an extension to the marketplace.
    ///
    /// # Arguments
    /// * `extension_path` - Path to the extension package
    /// * `api_key` - Marketplace API key for authentication
    ///
    /// # Returns
    /// Published extension metadata
    ///
    /// # Errors
    /// Returns error if publishing fails
    pub fn publish_extension(
        &self,
        extension_path: &std::path::Path,
        api_key: &str,
    ) -> Result<MarketplaceExtension> {
        use std::fs::File;
        use std::io::Read;

        // Read extension package
        let mut file = File::open(extension_path)
            .map_err(|e| MarketplaceError::InvalidResponse(format!("Failed to open extension package: {}", e)))?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|e| MarketplaceError::InvalidResponse(format!("Failed to read extension package: {}", e)))?;

        // Build publish URL
        let url = format!("{}/extensions/publish", self.base_url);

        // Create multipart form
        let form = reqwest::blocking::multipart::Form::new()
            .text("api_key", api_key.to_string())
            .part(
                "package",
                reqwest::blocking::multipart::Part::bytes(buffer)
                    .file_name(
                        extension_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("extension.tar.gz")
                            .to_string(),
                    )
                    .mime_str("application/gzip")
                    .map_err(|e| MarketplaceError::InvalidResponse(format!("Invalid mime type: {}", e)))?,
            );

        // Make request
        let response = self
            .client
            .post(&url)
            .multipart(form)
            .send()
            .map_err(MarketplaceError::Http)?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
            return Err(MarketplaceError::InvalidResponse(format!(
                "Publish failed with status {}: {}",
                status, error_text
            )));
        }

        // Parse response
        let extension: MarketplaceExtension = response
            .json()
            .map_err(|e| MarketplaceError::InvalidResponse(format!("Failed to parse publish response: {}", e)))?;

        Ok(extension)
    }

    /// Makes an HTTP request with retry logic.
    ///
    /// # Arguments
    /// * `url` - URL to request
    /// * `max_retries` - Maximum number of retry attempts
    ///
    /// # Returns
    /// HTTP response
    ///
    /// # Errors
    /// Returns error if all retry attempts fail
    fn make_request_with_retry(&self, url: &str, max_retries: u32) -> Result<reqwest::blocking::Response> {
        let mut last_error = None;

        for attempt in 0..=max_retries {
            match self.client.get(url).send() {
                Ok(response) => {
                    if response.status().is_success() {
                        return Ok(response);
                    } else if response.status().is_server_error() && attempt < max_retries {
                        // Retry on server errors
                        std::thread::sleep(Duration::from_millis(100 * (attempt + 1) as u64));
                        continue;
                    } else {
                        return Err(MarketplaceError::InvalidResponse(format!(
                            "Request failed with status: {}",
                            response.status()
                        )));
                    }
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_retries {
                        // Retry on network errors
                        std::thread::sleep(Duration::from_millis(100 * (attempt + 1) as u64));
                        continue;
                    }
                }
            }
        }

        Err(last_error
            .map(MarketplaceError::Http)
            .unwrap_or_else(|| MarketplaceError::Timeout))
    }
}

impl Default for MarketplaceClient {
    fn default() -> Self {
        Self::new().expect("Failed to create marketplace client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_marketplace_client_creation() {
        let client = MarketplaceClient::with_url("http://localhost:8080".to_string());
        assert!(client.is_ok());
    }

    #[test]
    fn test_cache_ttl() {
        let mut client = MarketplaceClient::with_url("http://localhost:8080".to_string()).unwrap();
        client.set_cache_ttl(Duration::from_secs(60));
        // Cache TTL should be set (no way to verify without making requests)
    }
}

