//! Example retry hook implementation.
//!
//! This hook implements automatic error recovery with retry logic and exponential backoff.

use async_trait::async_trait;
use radium_core::hooks::error_hooks::{ErrorHook, ErrorHookContext, ErrorHookType};
use radium_core::hooks::types::{HookPriority, HookResult as HookExecutionResult};
use radium_core::hooks::error::Result;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Retry configuration.
struct RetryConfig {
    max_retries: u32,
    initial_delay_ms: u64,
    max_delay_ms: u64,
    backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
        }
    }
}

/// Retry hook that implements automatic error recovery.
pub struct RetryHook {
    name: String,
    priority: HookPriority,
    config: RetryConfig,
    retry_counts: Arc<RwLock<std::collections::HashMap<String, u32>>>,
}

impl RetryHook {
    /// Create a new retry hook.
    pub fn new(name: impl Into<String>, priority: u32) -> Self {
        Self {
            name: name.into(),
            priority: HookPriority::new(priority),
            config: RetryConfig::default(),
            retry_counts: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Check if an error is retryable.
    fn is_retryable(&self, error_type: &str, error_message: &str) -> bool {
        let retryable_types = vec!["NetworkError", "TimeoutError", "RateLimitError", "TemporaryError"];
        retryable_types.contains(&error_type) || 
        error_message.contains("timeout") ||
        error_message.contains("network") ||
        error_message.contains("temporary")
    }

    /// Calculate backoff delay.
    fn calculate_delay(&self, retry_count: u32) -> Duration {
        let delay_ms = (self.config.initial_delay_ms as f64 * 
            self.config.backoff_multiplier.powi(retry_count as i32))
            .min(self.config.max_delay_ms as f64) as u64;
        Duration::from_millis(delay_ms)
    }

    /// Get retry count for an error.
    async fn get_retry_count(&self, error_key: &str) -> u32 {
        let counts = self.retry_counts.read().await;
        counts.get(error_key).copied().unwrap_or(0)
    }

    /// Increment retry count.
    async fn increment_retry_count(&self, error_key: &str) {
        let mut counts = self.retry_counts.write().await;
        let count = counts.entry(error_key.to_string()).or_insert(0);
        *count += 1;
    }

    /// Clear retry count (on success).
    async fn clear_retry_count(&self, error_key: &str) {
        let mut counts = self.retry_counts.write().await;
        counts.remove(error_key);
    }
}

#[async_trait]
impl ErrorHook for RetryHook {
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> HookPriority {
        self.priority
    }

    async fn error_recovery(&self, context: &ErrorHookContext) -> Result<HookExecutionResult> {
        // Check if error is retryable
        if !self.is_retryable(&context.error_type, &context.error_message) {
            return Ok(HookExecutionResult::success());
        }

        // Create error key for tracking
        let error_key = format!("{}:{}", context.error_type, context.error_source.as_deref().unwrap_or("unknown"));

        // Get current retry count
        let retry_count = self.get_retry_count(&error_key).await;

        // Check if we've exceeded max retries
        if retry_count >= self.config.max_retries {
            warn!(
                hook = %self.name,
                error_type = %context.error_type,
                retry_count = retry_count,
                "Max retries exceeded"
            );
            return Ok(HookExecutionResult::success());
        }

        // Calculate backoff delay
        let delay = self.calculate_delay(retry_count);
        info!(
            hook = %self.name,
            error_type = %context.error_type,
            retry_count = retry_count + 1,
            delay_ms = delay.as_millis(),
            "Retrying after backoff"
        );

        // Wait for backoff period
        tokio::time::sleep(delay).await;

        // Increment retry count
        self.increment_retry_count(&error_key).await;

        // Return recovery signal
        Ok(HookExecutionResult::with_data(json!({
            "recovered_error": format!("Retrying (attempt {})", retry_count + 1),
            "retry_count": retry_count + 1,
            "delay_ms": delay.as_millis()
        })))
    }

    async fn error_interception(&self, _context: &ErrorHookContext) -> Result<HookExecutionResult> {
        Ok(HookExecutionResult::success())
    }

    async fn error_transformation(&self, _context: &ErrorHookContext) -> Result<HookExecutionResult> {
        Ok(HookExecutionResult::success())
    }

    async fn error_logging(&self, context: &ErrorHookContext) -> Result<HookExecutionResult> {
        // Log retryable errors
        if self.is_retryable(&context.error_type, &context.error_message) {
            let error_key = format!("{}:{}", context.error_type, context.error_source.as_deref().unwrap_or("unknown"));
            let retry_count = self.get_retry_count(&error_key).await;
            
            info!(
                hook = %self.name,
                error_type = %context.error_type,
                error_message = %context.error_message,
                retry_count = retry_count,
                "Retryable error logged"
            );
        }
        Ok(HookExecutionResult::success())
    }
}

/// Create a retry hook.
pub fn create_retry_hook() -> std::sync::Arc<dyn radium_core::hooks::registry::Hook> {
    let hook = std::sync::Arc::new(RetryHook::new("retry-hook", 150));
    radium_core::hooks::error_hooks::ErrorHookAdapter::recovery(hook)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retry_hook_retryable_error() {
        let hook = RetryHook::new("test-retry", 150);
        let context = ErrorHookContext::recovery(
            "Network connection failed".to_string(),
            "NetworkError".to_string(),
            Some("model_call".to_string()),
        );

        let result = hook.error_recovery(&context).await.unwrap();
        assert!(result.success);
        assert!(result.modified_data.is_some());
    }

    #[tokio::test]
    async fn test_retry_hook_non_retryable_error() {
        let hook = RetryHook::new("test-retry", 150);
        let context = ErrorHookContext::recovery(
            "Invalid input".to_string(),
            "ValidationError".to_string(),
            Some("model_call".to_string()),
        );

        let result = hook.error_recovery(&context).await.unwrap();
        assert!(result.success);
    }
}

