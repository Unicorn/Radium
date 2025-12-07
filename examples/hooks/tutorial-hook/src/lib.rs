//! Tutorial hook: Request Logger
//!
//! This hook logs all model calls with timestamps and metadata.
//! It's designed as a learning example for creating custom hooks in Radium.

use async_trait::async_trait;
use radium_core::hooks::model::{ModelHook, ModelHookContext, ModelHookAdapter};
use radium_core::hooks::registry::{Hook, HookRegistry};
use radium_core::hooks::types::{HookPriority, HookResult as HookExecutionResult};
use radium_core::hooks::error::Result;
use std::sync::Arc;
use tracing::info;

/// Request Logger Hook that logs all model calls.
pub struct RequestLoggerHook {
    name: String,
    priority: HookPriority,
}

impl RequestLoggerHook {
    /// Create a new request logger hook.
    pub fn new(name: impl Into<String>, priority: u32) -> Self {
        Self {
            name: name.into(),
            priority: HookPriority::new(priority),
        }
    }
}

#[async_trait]
impl ModelHook for RequestLoggerHook {
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> HookPriority {
        self.priority
    }

    async fn before_model_call(&self, context: &ModelHookContext) -> Result<HookExecutionResult> {
        let timestamp = chrono::Utc::now().to_rfc3339();
        
        info!(
            hook = %self.name,
            %timestamp,
            model = %context.model_id,
            input_length = context.input.len(),
            "📝 Before model call: model={}, input_length={}",
            context.model_id,
            context.input.len()
        );

        Ok(HookExecutionResult::success())
    }

    async fn after_model_call(&self, context: &ModelHookContext) -> Result<HookExecutionResult> {
        let timestamp = chrono::Utc::now().to_rfc3339();
        let response_length = context.response.as_ref().map(|r| r.len()).unwrap_or(0);
        
        info!(
            hook = %self.name,
            %timestamp,
            model = %context.model_id,
            input_length = context.input.len(),
            response_length = response_length,
            "✅ After model call: model={}, input_length={}, response_length={}",
            context.model_id,
            context.input.len(),
            response_length
        );

        Ok(HookExecutionResult::success())
    }
}

/// Create a before model call hook adapter.
pub fn create_before_hook() -> Arc<dyn Hook> {
    let hook = Arc::new(RequestLoggerHook::new("tutorial-logger-before", 100));
    ModelHookAdapter::before(hook)
}

/// Create an after model call hook adapter.
pub fn create_after_hook() -> Arc<dyn Hook> {
    let hook = Arc::new(RequestLoggerHook::new("tutorial-logger-after", 100));
    ModelHookAdapter::after(hook)
}

/// Register tutorial hooks with the registry.
pub async fn register_tutorial_hooks(registry: &Arc<HookRegistry>) -> Result<()> {
    registry.register(create_before_hook()).await?;
    registry.register(create_after_hook()).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_request_logger_before() {
        let hook = RequestLoggerHook::new("test-logger", 100);
        let context = ModelHookContext::before(
            "test input".to_string(),
            "test-model".to_string(),
        );

        let result = hook.before_model_call(&context).await.unwrap();
        assert!(result.success);
        assert!(result.should_continue);
    }

    #[tokio::test]
    async fn test_request_logger_after() {
        let hook = RequestLoggerHook::new("test-logger", 100);
        let context = ModelHookContext::after(
            "test input".to_string(),
            "test-model".to_string(),
            "test response".to_string(),
        );

        let result = hook.after_model_call(&context).await.unwrap();
        assert!(result.success);
        assert!(result.should_continue);
    }

    #[tokio::test]
    async fn test_hook_registration() {
        let registry = Arc::new(HookRegistry::new());
        register_tutorial_hooks(&registry).await.unwrap();
        
        // Verify hooks are registered
        let hooks = registry.get_hooks(radium_core::hooks::registry::HookType::BeforeModel).await;
        assert!(hooks.iter().any(|h| h.name() == "tutorial-logger-before"));
        
        let hooks = registry.get_hooks(radium_core::hooks::registry::HookType::AfterModel).await;
        assert!(hooks.iter().any(|h| h.name() == "tutorial-logger-after"));
    }
}

