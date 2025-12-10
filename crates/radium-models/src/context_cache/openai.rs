//! OpenAI-specific context caching implementation.

use async_trait::async_trait;
use std::time::Duration;

use crate::context_cache::types::{CacheError, CacheHandle, CachedContext};
use crate::context_cache::ContextCache;

/// OpenAI context cache implementation.
///
/// OpenAI handles prompt caching automatically for GPT-4 and newer models.
/// There's no explicit cache creation or management - caching happens
/// automatically based on recent prompts.
///
/// # Cache Behavior
///
/// - Caching is automatic (no explicit control needed)
/// - Works with GPT-4 and newer models
/// - Cached tokens appear in `prompt_tokens_details.cached_tokens` in the response
/// - No minimum token requirement (provider-managed)
#[derive(Debug, Clone)]
pub struct OpenAIContextCache;

impl OpenAIContextCache {
    /// Create a new OpenAI context cache instance.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for OpenAIContextCache {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ContextCache for OpenAIContextCache {
    async fn create_cache(
        &self,
        _content: &str,
        _ttl: Duration,
    ) -> Result<CacheHandle, CacheError> {
        // OpenAI caching is automatic, no explicit creation needed
        Ok(CacheHandle::OpenAI)
    }

    async fn get_cache(
        &self,
        _handle: &CacheHandle,
    ) -> Result<Option<CachedContext>, CacheError> {
        // OpenAI doesn't expose cache metadata
        Err(CacheError::ProviderNotSupported {
            provider: "OpenAI".to_string(),
        })
    }

    async fn refresh_cache(&self, _handle: &CacheHandle) -> Result<(), CacheError> {
        // OpenAI manages cache lifecycle automatically
        Ok(())
    }

    async fn delete_cache(&self, _handle: &CacheHandle) -> Result<(), CacheError> {
        // OpenAI manages cache lifecycle automatically
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_context_cache_new() {
        let cache = OpenAIContextCache::new();
        // Just verify it compiles and can be created
        assert!(std::mem::size_of_val(&cache) > 0);
    }

    #[tokio::test]
    async fn test_create_cache() {
        let cache = OpenAIContextCache::new();
        let handle = cache
            .create_cache("test content", Duration::from_secs(300))
            .await
            .unwrap();

        match handle {
            CacheHandle::OpenAI => {
                // Expected
            }
            _ => panic!("Expected OpenAI cache handle"),
        }
    }

    #[tokio::test]
    async fn test_get_cache_not_supported() {
        let cache = OpenAIContextCache::new();
        let handle = CacheHandle::OpenAI;
        let result = cache.get_cache(&handle).await;

        assert!(result.is_err());
        if let Err(CacheError::ProviderNotSupported { provider }) = result {
            assert_eq!(provider, "OpenAI");
        } else {
            panic!("Expected ProviderNotSupported error");
        }
    }
}

