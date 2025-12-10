//! Gemini-specific context caching implementation.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

use crate::context_cache::types::{CacheError, CacheHandle, CachedContext};
use crate::context_cache::ContextCache;

/// Gemini context cache implementation.
///
/// Gemini uses the cachedContent API to create and manage cached content resources.
/// Cached content can be referenced in generation requests to reduce token costs.
///
/// # Cache Behavior
///
/// - Explicit cache creation via POST /v1beta/cachedContents
/// - Cache names follow format: "cachedContents/{cache-id}"
/// - TTL can be specified in seconds (e.g., "300s") or as expire_time (RFC3339)
/// - Cache can be refreshed (PATCH) or deleted (DELETE)
#[derive(Debug, Clone)]
pub struct GeminiContextCache {
    /// API key for authentication.
    api_key: String,
    /// Base URL for Gemini API.
    base_url: String,
    /// HTTP client for making requests.
    client: Client,
}

impl GeminiContextCache {
    /// Create a new Gemini context cache instance.
    ///
    /// # Arguments
    /// * `api_key` - Gemini API key
    /// * `base_url` - Base URL for Gemini API (default: "https://generativelanguage.googleapis.com/v1beta")
    #[must_use]
    pub fn new(api_key: String, base_url: Option<String>) -> Self {
        Self {
            api_key,
            base_url: base_url.unwrap_or_else(|| {
                "https://generativelanguage.googleapis.com/v1beta".to_string()
            }),
            client: Client::new(),
        }
    }
}

/// Request to create cached content.
#[derive(Debug, Serialize)]
struct CreateCachedContentRequest {
    /// Model name (e.g., "models/gemini-1.5-pro").
    model: String,
    /// Contents to cache.
    contents: Vec<serde_json::Value>, // Using Value for flexibility
    /// Time-to-live in seconds (e.g., "300s").
    #[serde(skip_serializing_if = "Option::is_none")]
    ttl: Option<String>,
    /// Expiration time as RFC3339 timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    expire_time: Option<String>,
}

/// Response from cachedContent API.
#[derive(Debug, Deserialize)]
struct CachedContentResponse {
    /// Cache name (e.g., "cachedContents/abc123").
    name: String,
    /// Model used for caching.
    model: String,
    /// Creation time (RFC3339).
    create_time: String,
    /// Last update time (RFC3339).
    update_time: String,
    /// Expiration time (RFC3339).
    expire_time: String,
    /// Usage metadata.
    #[serde(default)]
    usage_metadata: Option<CacheUsageMetadata>,
}

/// Usage metadata for cached content.
#[derive(Debug, Deserialize)]
struct CacheUsageMetadata {
    /// Total token count in cached content.
    #[serde(default, rename = "totalTokenCount")]
    total_token_count: Option<u32>,
}

#[async_trait]
impl ContextCache for GeminiContextCache {
    async fn create_cache(
        &self,
        content: &str,
        ttl: Duration,
    ) -> Result<CacheHandle, CacheError> {
        let url = format!("{}/cachedContents?key={}", self.base_url, self.api_key);

        // Convert content to Gemini format (simplified - just text for now)
        let contents = vec![serde_json::json!({
            "role": "user",
            "parts": [{"text": content}]
        })];

        let request = CreateCachedContentRequest {
            model: "models/gemini-1.5-pro".to_string(), // Default model
            contents,
            ttl: Some(format!("{}s", ttl.as_secs())),
            expire_time: None,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| CacheError::NetworkError {
                message: format!("Failed to create cache: {}", e),
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(CacheError::CacheCreationFailed {
                provider: "Gemini".to_string(),
                reason: error_text,
            });
        }

        let cached_content: CachedContentResponse = response.json().await.map_err(|e| {
            CacheError::ParseError {
                message: format!("Failed to parse response: {}", e),
            }
        })?;

        Ok(CacheHandle::Gemini {
            cache_name: cached_content.name,
        })
    }

    async fn get_cache(
        &self,
        handle: &CacheHandle,
    ) -> Result<Option<CachedContext>, CacheError> {
        let cache_name = match handle {
            CacheHandle::Gemini { cache_name } => cache_name,
            _ => {
                return Err(CacheError::InvalidCacheConfiguration {
                    reason: "Invalid cache handle type for Gemini".to_string(),
                });
            }
        };

        let url = format!("{}/{}?key={}", self.base_url, cache_name, self.api_key);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| CacheError::NetworkError {
                message: format!("Failed to get cache: {}", e),
            })?;

        if response.status() == 404 {
            return Ok(None);
        }

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(CacheError::CacheNotFound {
                handle: handle.clone(),
            });
        }

        let cached_content: CachedContentResponse = response.json().await.map_err(|e| {
            CacheError::ParseError {
                message: format!("Failed to parse response: {}", e),
            }
        })?;

        // Parse expiration time
        let expires_at = DateTime::parse_from_rfc3339(&cached_content.expire_time)
            .map_err(|e| CacheError::ParseError {
                message: format!("Failed to parse expire_time: {}", e),
            })?
            .with_timezone(&Utc)
            .into();

        let token_count = cached_content
            .usage_metadata
            .and_then(|m| m.total_token_count)
            .unwrap_or(0);

        Ok(Some(CachedContext {
            handle: handle.clone(),
            created_at: SystemTime::now(), // Approximate - API doesn't provide this directly
            expires_at,
            token_count,
        }))
    }

    async fn refresh_cache(&self, handle: &CacheHandle) -> Result<(), CacheError> {
        let cache_name = match handle {
            CacheHandle::Gemini { cache_name } => cache_name,
            _ => {
                return Err(CacheError::InvalidCacheConfiguration {
                    reason: "Invalid cache handle type for Gemini".to_string(),
                });
            }
        };

        // Get current cache to check expiration
        let cached_context = self.get_cache(handle).await?;
        if let Some(context) = cached_context {
            if context.is_expired() {
                return Err(CacheError::CacheExpired {
                    handle: handle.clone(),
                });
            }

            // Extend TTL by 1 hour (default refresh)
            let new_expire_time = SystemTime::now() + Duration::from_secs(3600);
            let expire_time_str = DateTime::<Utc>::from(new_expire_time)
                .to_rfc3339();

            let url = format!("{}/{}?key={}", self.base_url, cache_name, self.api_key);
            let update_request = serde_json::json!({
                "ttl": "3600s"
            });

            let response = self
                .client
                .patch(&url)
                .json(&update_request)
                .send()
                .await
                .map_err(|e| CacheError::NetworkError {
                    message: format!("Failed to refresh cache: {}", e),
                })?;

            if !response.status().is_success() {
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                return Err(CacheError::CacheCreationFailed {
                    provider: "Gemini".to_string(),
                    reason: format!("Failed to refresh cache: {}", error_text),
                });
            }
        } else {
            return Err(CacheError::CacheNotFound {
                handle: handle.clone(),
            });
        }

        Ok(())
    }

    async fn delete_cache(&self, handle: &CacheHandle) -> Result<(), CacheError> {
        let cache_name = match handle {
            CacheHandle::Gemini { cache_name } => cache_name,
            _ => {
                return Err(CacheError::InvalidCacheConfiguration {
                    reason: "Invalid cache handle type for Gemini".to_string(),
                });
            }
        };

        let url = format!("{}/{}?key={}", self.base_url, cache_name, self.api_key);

        let response = self
            .client
            .delete(&url)
            .send()
            .await
            .map_err(|e| CacheError::NetworkError {
                message: format!("Failed to delete cache: {}", e),
            })?;

        if response.status() == 404 {
            // Cache already deleted or doesn't exist - treat as success
            return Ok(());
        }

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(CacheError::CacheCreationFailed {
                provider: "Gemini".to_string(),
                reason: format!("Failed to delete cache: {}", error_text),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemini_context_cache_new() {
        let cache = GeminiContextCache::new("test-key".to_string(), None);
        // Just verify it compiles and can be created
        assert!(std::mem::size_of_val(&cache) > 0);
    }

    #[tokio::test]
    #[ignore] // Requires real API key
    async fn test_create_cache() {
        let api_key = std::env::var("GEMINI_API_KEY").unwrap();
        let cache = GeminiContextCache::new(api_key, None);
        let handle = cache
            .create_cache("test content", Duration::from_secs(300))
            .await
            .unwrap();

        match handle {
            CacheHandle::Gemini { cache_name } => {
                assert!(cache_name.starts_with("cachedContents/"));
            }
            _ => panic!("Expected Gemini cache handle"),
        }
    }
}

