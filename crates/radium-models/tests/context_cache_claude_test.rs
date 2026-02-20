//! Unit tests for Claude context caching.

use radium_models::context_cache::{ClaudeContextCache, ContextCache};
use std::time::Duration;

#[tokio::test]
async fn test_claude_context_cache_create() {
    let cache = ClaudeContextCache::new();
    let handle = cache
        .create_cache("test content", Duration::from_secs(300))
        .await
        .unwrap();

    match handle {
        radium_models::context_cache::CacheHandle::Claude { cache_control } => {
            assert_eq!(cache_control["type"], "ephemeral");
        }
        _ => panic!("Expected Claude cache handle"),
    }
}

#[tokio::test]
async fn test_claude_context_cache_get_not_supported() {
    let cache = ClaudeContextCache::new();
    let handle = radium_models::context_cache::CacheHandle::Claude {
        cache_control: serde_json::json!({ "type": "ephemeral" }),
    };

    let result = cache.get_cache(&handle).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_claude_context_cache_refresh() {
    let cache = ClaudeContextCache::new();
    let handle = radium_models::context_cache::CacheHandle::Claude {
        cache_control: serde_json::json!({ "type": "ephemeral" }),
    };

    // Refresh should succeed (no-op for Claude)
    let result = cache.refresh_cache(&handle).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_claude_context_cache_delete() {
    let cache = ClaudeContextCache::new();
    let handle = radium_models::context_cache::CacheHandle::Claude {
        cache_control: serde_json::json!({ "type": "ephemeral" }),
    };

    // Delete should succeed (no-op for Claude)
    let result = cache.delete_cache(&handle).await;
    assert!(result.is_ok());
}

