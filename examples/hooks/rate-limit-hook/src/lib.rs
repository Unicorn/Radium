//! Example rate limit hook implementation.
//!
//! This hook enforces rate limits on model calls to prevent excessive usage.

use async_trait::async_trait;
use radium_core::hooks::model::{ModelHook, ModelHookContext};
use radium_core::hooks::types::{HookPriority, HookResult as HookExecutionResult};
use radium_core::hooks::error::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn};
use std::collections::HashMap;

/// Rate limit entry tracking calls in a time window.
struct RateLimitEntry {
    count: u32,
    window_start: Instant,
}

/// Rate limit hook that enforces rate limits on model calls.
pub struct RateLimitHook {
    name: String,
    priority: HookPriority,
    max_calls: u32,
    window_duration: Duration,
    limits: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
}

impl RateLimitHook {
    /// Create a new rate limit hook.
    pub fn new(name: impl Into<String>, priority: u32, max_calls: u32, window_seconds: u64) -> Self {
        Self {
            name: name.into(),
            priority: HookPriority::new(priority),
            max_calls,
            window_duration: Duration::from_secs(window_seconds),
            limits: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ModelHook for RateLimitHook {
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> HookPriority {
        self.priority
    }

    async fn before_model_call(&self, context: &ModelHookContext) -> Result<HookExecutionResult> {
        let now = Instant::now();
        let key = context.model_id.clone();

        let mut limits = self.limits.write().await;

        // Clean up expired entries
        limits.retain(|_, entry| now.duration_since(entry.window_start) < self.window_duration);

        // Get or create entry for this model
        let entry = limits.entry(key.clone()).or_insert_with(|| RateLimitEntry {
            count: 0,
            window_start: now,
        });

        // Check if window has expired
        if now.duration_since(entry.window_start) >= self.window_duration {
            // Reset window
            entry.count = 1;
            entry.window_start = now;
            info!(
                hook = %self.name,
                model = %context.model_id,
                count = 1,
                "Rate limit window reset"
            );
            return Ok(HookExecutionResult::success());
        }

        // Check rate limit
        if entry.count >= self.max_calls {
            warn!(
                hook = %self.name,
                model = %context.model_id,
                count = entry.count,
                max_calls = self.max_calls,
                "Rate limit exceeded"
            );
            return Ok(HookExecutionResult::stop(format!(
                "Rate limit exceeded: {} calls in {} seconds (max: {})",
                entry.count,
                self.window_duration.as_secs(),
                self.max_calls
            )));
        }

        // Increment count
        entry.count += 1;
        info!(
            hook = %self.name,
            model = %context.model_id,
            count = entry.count,
            max_calls = self.max_calls,
            "Rate limit check passed"
        );

        Ok(HookExecutionResult::success())
    }

    async fn after_model_call(&self, _context: &ModelHookContext) -> Result<HookExecutionResult> {
        // No action needed after model call
        Ok(HookExecutionResult::success())
    }
}

/// Create a rate limit hook.
pub fn create_rate_limit_hook() -> std::sync::Arc<dyn radium_core::hooks::registry::Hook> {
    let hook = std::sync::Arc::new(RateLimitHook::new("rate-limit-hook", 200, 10, 60));
    radium_core::hooks::model::ModelHookAdapter::before(hook)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limit_hook_within_limit() {
        let hook = RateLimitHook::new("test-rate-limit", 200, 5, 60);
        let context = ModelHookContext::before(
            "test input".to_string(),
            "test-model".to_string(),
        );

        // First call should pass
        let result = hook.before_model_call(&context).await.unwrap();
        assert!(result.should_continue);
    }

    #[tokio::test]
    async fn test_rate_limit_hook_exceeded() {
        let hook = RateLimitHook::new("test-rate-limit", 200, 2, 60);
        let context = ModelHookContext::before(
            "test input".to_string(),
            "test-model".to_string(),
        );

        // First two calls should pass
        hook.before_model_call(&context).await.unwrap();
        hook.before_model_call(&context).await.unwrap();

        // Third call should be blocked
        let result = hook.before_model_call(&context).await.unwrap();
        assert!(!result.should_continue);
    }
}

