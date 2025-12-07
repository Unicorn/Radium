//! Edge case tests for the hooks system.

use radium_core::hooks::registry::{Hook, HookRegistry, HookType};
use radium_core::hooks::types::{HookContext, HookPriority, HookResult as HookExecutionResult};
use radium_core::hooks::error::Result;
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;

/// Test hook for edge case testing.
struct TestHook {
    name: String,
    priority: HookPriority,
    should_continue: bool,
}

impl TestHook {
    fn new(name: impl Into<String>, priority: u32, should_continue: bool) -> Self {
        Self {
            name: name.into(),
            priority: HookPriority::new(priority),
            should_continue,
        }
    }
}

#[async_trait]
impl Hook for TestHook {
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
        if self.should_continue {
            Ok(HookExecutionResult::success())
        } else {
            Ok(HookExecutionResult::stop("Hook requested stop"))
        }
    }
}

#[tokio::test]
async fn test_empty_hook_registry() {
    let registry = Arc::new(HookRegistry::new());
    let context = HookContext::new("before_model", json!({}));

    let results = registry.execute_hooks(HookType::BeforeModel, &context).await.unwrap();
    assert_eq!(results.len(), 0);

    let hooks = registry.get_hooks(HookType::BeforeModel).await;
    assert_eq!(hooks.len(), 0);

    assert_eq!(registry.count().await, 0);
}

#[tokio::test]
async fn test_hook_with_maximum_priority() {
    let registry = Arc::new(HookRegistry::new());
    let hook = Arc::new(TestHook::new("max-priority-hook", u32::MAX, true));
    
    registry.register(hook.clone()).await.unwrap();
    
    let hooks = registry.get_hooks(HookType::BeforeModel).await;
    assert_eq!(hooks.len(), 1);
    assert_eq!(hooks[0].priority().value(), u32::MAX);
}

#[tokio::test]
async fn test_hook_with_minimum_priority() {
    let registry = Arc::new(HookRegistry::new());
    let hook = Arc::new(TestHook::new("min-priority-hook", 0, true));
    
    registry.register(hook.clone()).await.unwrap();
    
    let hooks = registry.get_hooks(HookType::BeforeModel).await;
    assert_eq!(hooks.len(), 1);
    assert_eq!(hooks[0].priority().value(), 0);
}

#[tokio::test]
async fn test_multiple_hooks_same_priority() {
    let registry = Arc::new(HookRegistry::new());
    
    let hook1 = Arc::new(TestHook::new("hook-1", 100, true));
    let hook2 = Arc::new(TestHook::new("hook-2", 100, true));
    let hook3 = Arc::new(TestHook::new("hook-3", 100, true));
    
    registry.register(hook1).await.unwrap();
    registry.register(hook2).await.unwrap();
    registry.register(hook3).await.unwrap();
    
    let hooks = registry.get_hooks(HookType::BeforeModel).await;
    assert_eq!(hooks.len(), 3);
    
    // All should have same priority
    for hook in &hooks {
        assert_eq!(hook.priority().value(), 100);
    }
}

#[tokio::test]
async fn test_hook_returning_should_continue_false() {
    let registry = Arc::new(HookRegistry::new());
    
    // Hook that stops execution
    let stop_hook = Arc::new(TestHook::new("stop-hook", 200, false));
    registry.register(stop_hook).await.unwrap();
    
    // Hook that should not execute
    let later_hook = Arc::new(TestHook::new("later-hook", 100, true));
    registry.register(later_hook).await.unwrap();
    
    let context = HookContext::new("before_model", json!({}));
    let results = registry.execute_hooks(HookType::BeforeModel, &context).await.unwrap();
    
    // Only first hook should execute (stops execution)
    assert_eq!(results.len(), 1);
    assert!(!results[0].should_continue);
}

#[tokio::test]
async fn test_hook_with_empty_context_data() {
    let registry = Arc::new(HookRegistry::new());
    let hook = Arc::new(TestHook::new("test-hook", 100, true));
    
    registry.register(hook).await.unwrap();
    
    // Empty context data
    let context = HookContext::new("before_model", json!({}));
    let results = registry.execute_hooks(HookType::BeforeModel, &context).await.unwrap();
    
    assert_eq!(results.len(), 1);
    assert!(results[0].success);
}

#[tokio::test]
async fn test_hook_with_large_context_data() {
    let registry = Arc::new(HookRegistry::new());
    let hook = Arc::new(TestHook::new("test-hook", 100, true));
    
    registry.register(hook).await.unwrap();
    
    // Large context data (1MB string)
    let large_data = "x".repeat(1_000_000);
    let context = HookContext::new("before_model", json!({
        "large_data": large_data
    }));
    
    let results = registry.execute_hooks(HookType::BeforeModel, &context).await.unwrap();
    
    assert_eq!(results.len(), 1);
    assert!(results[0].success);
}

#[tokio::test]
async fn test_unregistering_nonexistent_hook() {
    let registry = Arc::new(HookRegistry::new());
    
    // Try to unregister non-existent hook
    let result = registry.unregister("nonexistent-hook").await;
    
    // Should return error
    assert!(result.is_err());
}

#[tokio::test]
async fn test_unregistering_existing_hook() {
    let registry = Arc::new(HookRegistry::new());
    let hook = Arc::new(TestHook::new("test-hook", 100, true));
    
    registry.register(hook).await.unwrap();
    assert_eq!(registry.count().await, 1);
    
    registry.unregister("test-hook").await.unwrap();
    assert_eq!(registry.count().await, 0);
    
    let hooks = registry.get_hooks(HookType::BeforeModel).await;
    assert_eq!(hooks.len(), 0);
}

#[tokio::test]
async fn test_registry_clear() {
    let registry = Arc::new(HookRegistry::new());
    
    let hook1 = Arc::new(TestHook::new("hook-1", 100, true));
    let hook2 = Arc::new(TestHook::new("hook-2", 100, true));
    
    registry.register(hook1).await.unwrap();
    registry.register(hook2).await.unwrap();
    assert_eq!(registry.count().await, 2);
    
    registry.clear().await;
    assert_eq!(registry.count().await, 0);
}

#[tokio::test]
async fn test_hook_with_nested_json_context() {
    let registry = Arc::new(HookRegistry::new());
    let hook = Arc::new(TestHook::new("test-hook", 100, true));
    
    registry.register(hook).await.unwrap();
    
    // Nested JSON context
    let context = HookContext::new("before_model", json!({
        "level1": {
            "level2": {
                "level3": {
                    "value": "deep"
                }
            }
        }
    }));
    
    let results = registry.execute_hooks(HookType::BeforeModel, &context).await.unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].success);
}

#[tokio::test]
async fn test_hook_with_array_context() {
    let registry = Arc::new(HookRegistry::new());
    let hook = Arc::new(TestHook::new("test-hook", 100, true));
    
    registry.register(hook).await.unwrap();
    
    // Array context
    let context = HookContext::new("before_model", json!({
        "items": [1, 2, 3, 4, 5]
    }));
    
    let results = registry.execute_hooks(HookType::BeforeModel, &context).await.unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].success);
}

