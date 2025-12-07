//! Example cache hook implementation.
//!
//! This hook caches model responses to reduce costs and improve performance.

use async_trait::async_trait;
use radium_core::hooks::model::{ModelHook, ModelHookContext};
use radium_core::hooks::types::{HookPriority, HookResult as HookExecutionResult};
use radium_core::hooks::error::Result;
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info};
use std::collections::HashMap;

/// Cache entry storing response and timestamp.
struct CacheEntry {
    response: String,
    timestamp: Instant,
}

/// Cache hook that caches model responses.
pub struct CacheHook {
    name: String,
    priority: HookPriority,
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    ttl: Duration,
}

impl CacheHook {
    /// Create a new cache hook.
    pub fn new(name: impl Into<String>, priority: u32, ttl_seconds: u64) -> Self {
        Self {
            name: name.into(),
            priority: HookPriority::new(priority),
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    /// Generate cache key from model and input.
    fn cache_key(&self, model_id: &str, input: &str) -> String {
        format!("{}:{}", model_id, input)
    }

    /// Clean up expired cache entries.
    async fn cleanup_expired(&self) {
        let now = Instant::now();
        let mut cache = self.cache.write().await;
        cache.retain(|_, entry| now.duration_since(entry.timestamp) < self.ttl);
    }
}

#[async_trait]
impl ModelHook for CacheHook {
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> HookPriority {
        self.priority
    }

    async fn before_model_call(&self, context: &ModelHookContext) -> Result<HookExecutionResult> {
        // Clean up expired entries periodically
        self.cleanup_expired().await;

        // Check cache
        let cache_key = self.cache_key(&context.model_id, &context.input);
        let cache = self.cache.read().await;

        if let Some(entry) = cache.get(&cache_key) {
            // Check if entry is still valid
            if Instant::now().duration_since(entry.timestamp) < self.ttl {
                info!(
                    hook = %self.name,
                    model = %context.model_id,
                    "Cache hit"
                );
                // Return cached response
                return Ok(HookExecutionResult::with_data(json!({
                    "cached": true,
                    "response": entry.response.clone()
                })));
            }
        }

        drop(cache);
        debug!(
            hook = %self.name,
            model = %context.model_id,
            "Cache miss"
        );

        Ok(HookExecutionResult::success())
    }

    async fn after_model_call(&self, context: &ModelHookContext) -> Result<HookExecutionResult> {
        // Store response in cache
        if let Some(response) = &context.response {
            let cache_key = self.cache_key(&context.model_id, &context.input);
            let mut cache = self.cache.write().await;

            cache.insert(
                cache_key,
                CacheEntry {
                    response: response.clone(),
                    timestamp: Instant::now(),
                },
            );

            info!(
                hook = %self.name,
                model = %context.model_id,
                "Response cached"
            );
        }

        Ok(HookExecutionResult::success())
    }
}

/// Create a cache hook.
pub fn create_cache_hook() -> std::sync::Arc<dyn radium_core::hooks::registry::Hook> {
    let hook = std::sync::Arc::new(CacheHook::new("cache-hook", 50, 3600));
    radium_core::hooks::model::ModelHookAdapter::before(hook.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_hook_cache_miss() {
        let hook = CacheHook::new("test-cache", 50, 3600);
        let context = ModelHookContext::before(
            "test input".to_string(),
            "test-model".to_string(),
        );

        let result = hook.before_model_call(&context).await.unwrap();
        assert!(result.should_continue);
        assert!(result.modified_data.is_none());
    }

    #[tokio::test]
    async fn test_cache_hook_cache_hit() {
        let hook = CacheHook::new("test-cache", 50, 3600);
        let input = "test input".to_string();
        let model_id = "test-model".to_string();
        let response = "test response".to_string();

        // Store in cache
        let after_context = ModelHookContext::after(
            input.clone(),
            model_id.clone(),
            response.clone(),
        );
        hook.after_model_call(&after_context).await.unwrap();

        // Check cache
        let before_context = ModelHookContext::before(input, model_id);
        let result = hook.before_model_call(&before_context).await.unwrap();
        assert!(result.modified_data.is_some());
        if let Some(data) = result.modified_data {
            assert_eq!(data.get("cached"), Some(&json!(true)));
            assert_eq!(data.get("response"), Some(&json!(response)));
        }
    }
}

