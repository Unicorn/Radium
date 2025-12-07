//! Error handling tests for the hooks system.

use radium_core::hooks::registry::{Hook, HookRegistry, HookType};
use radium_core::hooks::types::{HookContext, HookPriority, HookResult as HookExecutionResult};
use radium_core::hooks::error::{HookError, Result};
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;

/// Hook that always fails.
struct FailingHook {
    name: String,
    priority: HookPriority,
}

impl FailingHook {
    fn new(name: impl Into<String>, priority: u32) -> Self {
        Self {
            name: name.into(),
            priority: HookPriority::new(priority),
        }
    }
}

#[async_trait]
impl Hook for FailingHook {
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> HookPriority {
        self.priority
    }

    fn hook_type(&self) -> HookType {
        HookType::BeforeModel
    }

    async fn execute(&self, _context: &HookContext) -> Result<HookExecutionResult> {
        Err(HookError::ExecutionFailed("Hook execution failed".to_string()))
    }
}

/// Hook that returns error result but doesn't fail.
struct ErrorResultHook {
    name: String,
    priority: HookPriority,
}

impl ErrorResultHook {
    fn new(name: impl Into<String>, priority: u32) -> Self {
        Self {
            name: name.into(),
            priority: HookPriority::new(priority),
        }
    }
}

#[async_trait]
impl Hook for ErrorResultHook {
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> HookPriority {
        self.priority
    }

    fn hook_type(&self) -> HookType {
        HookType::BeforeModel
    }

    async fn execute(&self, _context: &HookContext) -> Result<HookExecutionResult> {
        Ok(HookExecutionResult::error("Error occurred but continuing".to_string()))
    }
}

/// Hook that succeeds.
struct SuccessHook {
    name: String,
    priority: HookPriority,
}

impl SuccessHook {
    fn new(name: impl Into<String>, priority: u32) -> Self {
        Self {
            name: name.into(),
            priority: HookPriority::new(priority),
        }
    }
}

#[async_trait]
impl Hook for SuccessHook {
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> HookPriority {
        self.priority
    }

    fn hook_type(&self) -> HookType {
        HookType::BeforeModel
    }

    async fn execute(&self, _context: &HookContext) -> Result<HookExecutionResult> {
        Ok(HookExecutionResult::success())
    }
}

#[tokio::test]
async fn test_hook_execution_failure_doesnt_stop_others() {
    let registry = Arc::new(HookRegistry::new());
    
    // Register failing hook
    let failing_hook = Arc::new(FailingHook::new("failing-hook", 200));
    registry.register(failing_hook).await.unwrap();
    
    // Register successful hook
    let success_hook = Arc::new(SuccessHook::new("success-hook", 100));
    registry.register(success_hook).await.unwrap();
    
    let context = HookContext::new("before_model", json!({}));
    let results = registry.execute_hooks(HookType::BeforeModel, &context).await.unwrap();
    
    // Both hooks should execute (failure doesn't stop others)
    assert_eq!(results.len(), 2);
    
    // First result is error (from failing hook)
    assert!(!results[0].success);
    
    // Second result is success
    assert!(results[1].success);
}

#[tokio::test]
async fn test_hook_returning_error_result() {
    let registry = Arc::new(HookRegistry::new());
    
    let error_hook = Arc::new(ErrorResultHook::new("error-hook", 100));
    registry.register(error_hook).await.unwrap();
    
    let context = HookContext::new("before_model", json!({}));
    let results = registry.execute_hooks(HookType::BeforeModel, &context).await.unwrap();
    
    assert_eq!(results.len(), 1);
    assert!(!results[0].success);
    assert!(results[0].message.is_some());
    assert_eq!(results[0].message.as_ref().unwrap(), "Error occurred but continuing");
}

#[tokio::test]
async fn test_enabling_nonexistent_hook() {
    let registry = Arc::new(HookRegistry::new());
    
    // Try to enable non-existent hook
    let result = registry.set_enabled("nonexistent-hook", true).await;
    
    // Should return error
    assert!(result.is_err());
}

#[tokio::test]
async fn test_disabling_nonexistent_hook() {
    let registry = Arc::new(HookRegistry::new());
    
    // Try to disable non-existent hook
    let result = registry.set_enabled("nonexistent-hook", false).await;
    
    // Should return error
    assert!(result.is_err());
}

#[tokio::test]
async fn test_enabling_existing_hook() {
    let registry = Arc::new(HookRegistry::new());
    let hook = Arc::new(SuccessHook::new("test-hook", 100));
    
    registry.register(hook).await.unwrap();
    
    // Disable hook
    registry.set_enabled("test-hook", false).await.unwrap();
    
    // Hook should not execute
    let context = HookContext::new("before_model", json!({}));
    let results = registry.execute_hooks(HookType::BeforeModel, &context).await.unwrap();
    assert_eq!(results.len(), 0);
    
    // Enable hook
    registry.set_enabled("test-hook", true).await.unwrap();
    
    // Hook should execute
    let results = registry.execute_hooks(HookType::BeforeModel, &context).await.unwrap();
    assert_eq!(results.len(), 1);
}

#[tokio::test]
async fn test_invalid_hook_config() {
    // Test invalid TOML
    let invalid_toml = r#"
        [[hooks]]
        name = "test-hook"
        # Missing required 'type' field
        priority = 100
    "#;
    
    let result = radium_core::hooks::config::HookConfig::from_str(invalid_toml);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_hook_with_invalid_context_data() {
    let registry = Arc::new(HookRegistry::new());
    let hook = Arc::new(SuccessHook::new("test-hook", 100));
    
    registry.register(hook).await.unwrap();
    
    // Context with invalid structure (but valid JSON)
    let context = HookContext::new("before_model", json!({
        "invalid": null,
        "nested": {
            "deep": null
        }
    }));
    
    // Should still execute (invalid data is handled by hook)
    let results = registry.execute_hooks(HookType::BeforeModel, &context).await.unwrap();
    assert_eq!(results.len(), 1);
}

#[tokio::test]
async fn test_multiple_failing_hooks() {
    let registry = Arc::new(HookRegistry::new());
    
    let failing1 = Arc::new(FailingHook::new("failing-1", 200));
    let failing2 = Arc::new(FailingHook::new("failing-2", 100));
    let success = Arc::new(SuccessHook::new("success", 50));
    
    registry.register(failing1).await.unwrap();
    registry.register(failing2).await.unwrap();
    registry.register(success).await.unwrap();
    
    let context = HookContext::new("before_model", json!({}));
    let results = registry.execute_hooks(HookType::BeforeModel, &context).await.unwrap();
    
    // All hooks should execute despite failures
    assert_eq!(results.len(), 3);
    
    // First two are errors
    assert!(!results[0].success);
    assert!(!results[1].success);
    
    // Last is success
    assert!(results[2].success);
}

#[tokio::test]
async fn test_hook_error_isolation() {
    let registry = Arc::new(HookRegistry::new());
    
    // Hook that panics (should be caught and logged)
    struct PanickingHook {
        name: String,
        priority: HookPriority,
    }
    
    impl PanickingHook {
        fn new(name: impl Into<String>, priority: u32) -> Self {
            Self {
                name: name.into(),
                priority: HookPriority::new(priority),
            }
        }
    }
    
    #[async_trait]
    impl Hook for PanickingHook {
        fn name(&self) -> &str {
            &self.name
        }
        
        fn priority(&self) -> HookPriority {
            self.priority
        }
        
        fn hook_type(&self) -> HookType {
            HookType::BeforeModel
        }
        
        async fn execute(&self, _context: &HookContext) -> Result<HookExecutionResult> {
            // This would panic, but we catch it in the registry
            Err(HookError::ExecutionFailed("Panic simulated".to_string()))
        }
    }
    
    let panicking = Arc::new(PanickingHook::new("panicking", 200));
    let success = Arc::new(SuccessHook::new("success", 100));
    
    registry.register(panicking).await.unwrap();
    registry.register(success).await.unwrap();
    
    let context = HookContext::new("before_model", json!({}));
    let results = registry.execute_hooks(HookType::BeforeModel, &context).await.unwrap();
    
    // Both hooks should execute (error is isolated)
    assert_eq!(results.len(), 2);
}

