//! Integration tests for context caching across providers.

use radium_models::context_cache::registry::CacheRegistry;
use radium_models::context_cache::types::{CacheHandle, CachedContext};
use std::time::Duration;

#[test]
fn test_cache_registry_operations() {
    let registry = CacheRegistry::new();
    let handle = CacheHandle::Gemini {
        cache_name: "cachedContents/test".to_string(),
    };
    let context = CachedContext::new(handle.clone(), Duration::from_secs(300), 1000);

    registry.register(&handle, context.clone()).unwrap();
    let retrieved = registry.get(&handle).unwrap().unwrap();
    assert_eq!(retrieved.token_count, 1000);

    let removed = registry.remove(&handle).unwrap();
    assert!(removed.is_some());
    assert!(registry.get(&handle).unwrap().is_none());
}

#[test]
fn test_cache_registry_cleanup_expired() {
    let registry = CacheRegistry::new();
    let handle1 = CacheHandle::OpenAI;
    let handle2 = CacheHandle::Gemini {
        cache_name: "cachedContents/test".to_string(),
    };

    let expired_context = CachedContext::new(handle1.clone(), Duration::from_secs(0), 1000);
    let valid_context = CachedContext::new(handle2.clone(), Duration::from_secs(300), 2000);

    registry.register(&handle1, expired_context).unwrap();
    registry.register(&handle2, valid_context).unwrap();

    std::thread::sleep(Duration::from_millis(10));

    let removed = registry.cleanup_expired().unwrap();
    assert_eq!(removed, 1);
    assert_eq!(registry.len().unwrap(), 1);
    assert!(registry.get(&handle2).unwrap().is_some());
}

