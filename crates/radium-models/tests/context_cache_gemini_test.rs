//! Unit tests for Gemini context caching.

use radium_models::context_cache::{GeminiContextCache, ContextCache};
use std::time::Duration;

#[test]
fn test_gemini_context_cache_new() {
    let cache = GeminiContextCache::new("test-key".to_string(), None);
    assert!(std::mem::size_of_val(&cache) > 0);
}

#[tokio::test]
#[ignore] // Requires real API key
async fn test_gemini_context_cache_create() {
    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY not set");
    let cache = GeminiContextCache::new(api_key, None);
    let handle = cache
        .create_cache("test content", Duration::from_secs(300))
        .await
        .unwrap();

    match handle {
        radium_models::context_cache::CacheHandle::Gemini { cache_name } => {
            assert!(cache_name.starts_with("cachedContents/"));
        }
        _ => panic!("Expected Gemini cache handle"),
    }
}

