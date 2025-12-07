//! Example logging hook implementation.
//!
//! This hook logs model calls with timestamps, agent ID, and input/output information.
//! It demonstrates how to implement hooks for BeforeModel and AfterModel hook points.

use async_trait::async_trait;
use radium_core::hooks::model::{ModelHook, ModelHookContext, ModelHookType};
use radium_core::hooks::registry::HookType;
use radium_core::hooks::types::{HookContext, HookPriority, HookResult as HookExecutionResult};
use radium_core::hooks::error::Result;
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Logging hook that logs model calls.
pub struct LoggingHook {
    name: String,
    priority: HookPriority,
    log_level: String,
    log_format: String,
}

impl LoggingHook {
    /// Create a new logging hook.
    pub fn new(name: impl Into<String>, priority: u32) -> Self {
        Self {
            name: name.into(),
            priority: HookPriority::new(priority),
            log_level: "info".to_string(),
            log_format: "text".to_string(),
        }
    }

    /// Create a new logging hook with configuration.
    pub fn with_config(
        name: impl Into<String>,
        priority: u32,
        log_level: impl Into<String>,
        log_format: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            priority: HookPriority::new(priority),
            log_level: log_level.into(),
            log_format: log_format.into(),
        }
    }
}

#[async_trait]
impl ModelHook for LoggingHook {
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> HookPriority {
        self.priority
    }

    async fn before_model_call(&self, context: &ModelHookContext) -> Result<HookExecutionResult> {
        let timestamp = chrono::Utc::now().to_rfc3339();
        let input_len = context.input.len();

        if self.log_format == "json" {
            let log_data = json!({
                "timestamp": timestamp,
                "hook": self.name,
                "event": "before_model_call",
                "model_id": context.model_id,
                "input_length": input_len,
                "log_level": self.log_level,
            });
            info!(hook = %self.name, %timestamp, model_id = %context.model_id, input_len = input_len, "{}", serde_json::to_string(&log_data).unwrap_or_default());
        } else {
            info!(
                hook = %self.name,
                %timestamp,
                model_id = %context.model_id,
                input_len = input_len,
                "Before model call: model={}, input_length={}",
                context.model_id,
                input_len
            );
        }

        Ok(HookExecutionResult::success())
    }

    async fn after_model_call(&self, context: &ModelHookContext) -> Result<HookExecutionResult> {
        let timestamp = chrono::Utc::now().to_rfc3339();
        let input_len = context.input.len();
        let output_len = context.response.as_ref().map(|r| r.len()).unwrap_or(0);

        if self.log_format == "json" {
            let log_data = json!({
                "timestamp": timestamp,
                "hook": self.name,
                "event": "after_model_call",
                "model_id": context.model_id,
                "input_length": input_len,
                "output_length": output_len,
                "log_level": self.log_level,
            });
            info!(hook = %self.name, %timestamp, model_id = %context.model_id, input_len = input_len, output_len = output_len, "{}", serde_json::to_string(&log_data).unwrap_or_default());
        } else {
            info!(
                hook = %self.name,
                %timestamp,
                model_id = %context.model_id,
                input_len = input_len,
                output_len = output_len,
                "After model call: model={}, input_length={}, output_length={}",
                context.model_id,
                input_len,
                output_len
            );
        }

        Ok(HookExecutionResult::success())
    }
}

/// Create before model call hook adapter.
pub fn create_before_hook() -> Arc<dyn radium_core::hooks::registry::Hook> {
    let hook = Arc::new(LoggingHook::new("logging-hook-before", 100));
    radium_core::hooks::model::ModelHookAdapter::before(hook)
}

/// Create after model call hook adapter.
pub fn create_after_hook() -> Arc<dyn radium_core::hooks::registry::Hook> {
    let hook = Arc::new(LoggingHook::new("logging-hook-after", 100));
    radium_core::hooks::model::ModelHookAdapter::after(hook)
}

/// Create hooks with custom configuration.
pub fn create_hooks_with_config(
    log_level: impl Into<String>,
    log_format: impl Into<String>,
) -> (Arc<dyn radium_core::hooks::registry::Hook>, Arc<dyn radium_core::hooks::registry::Hook>) {
    let log_level = log_level.into();
    let log_format = log_format.into();
    
    let before_hook = Arc::new(LoggingHook::with_config(
        "logging-hook-before",
        100,
        log_level.clone(),
        log_format.clone(),
    ));
    
    let after_hook = Arc::new(LoggingHook::with_config(
        "logging-hook-after",
        100,
        log_level,
        log_format,
    ));
    
    (
        radium_core::hooks::model::ModelHookAdapter::before(before_hook),
        radium_core::hooks::model::ModelHookAdapter::after(after_hook),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_logging_hook_before() {
        let hook = LoggingHook::new("test-logging", 100);
        let context = ModelHookContext::before("test input".to_string(), "test-model".to_string());
        
        let result = hook.before_model_call(&context).await.unwrap();
        assert!(result.success);
        assert!(result.should_continue);
    }

    #[tokio::test]
    async fn test_logging_hook_after() {
        let hook = LoggingHook::new("test-logging", 100);
        let context = ModelHookContext::after(
            "test input".to_string(),
            "test-model".to_string(),
            "test output".to_string(),
        );
        
        let result = hook.after_model_call(&context).await.unwrap();
        assert!(result.success);
        assert!(result.should_continue);
    }
}

