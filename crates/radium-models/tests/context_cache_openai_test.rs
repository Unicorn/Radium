//! Unit tests for OpenAI context caching.

use radium_models::context_cache::{OpenAIContextCache, ContextCache};
use std::time::Duration;

#[tokio::test]
async fn test_openai_context_cache_create() {
    let cache = OpenAIContextCache::new();
    let handle = cache
        .create_cache("test content", Duration::from_secs(300))
        .await
        .unwrap();

    match handle {
        radium_models::context_cache::CacheHandle::OpenAI => {
            // Expected
        }
        _ => panic!("Expected OpenAI cache handle"),
    }
}

#[tokio::test]
async fn test_openai_context_cache_get_not_supported() {
    let cache = OpenAIContextCache::new();
    let handle = radium_models::context_cache::CacheHandle::OpenAI;
    let result = cache.get_cache(&handle).await;

    assert!(result.is_err());
    if let Err(radium_models::context_cache::CacheError::ProviderNotSupported { provider }) = result {
        assert_eq!(provider, "OpenAI");
    } else {
        panic!("Expected ProviderNotSupported error");
    }
}

