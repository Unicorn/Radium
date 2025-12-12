# Testing Hooks

This guide covers testing strategies for hooks in the Radium hooks system.

## Overview

Testing hooks is crucial for ensuring reliability and correctness. This guide covers:
- Unit testing individual hooks
- Integration testing with the registry
- Testing hook execution order
- Testing context modifications
- Mocking and test doubles

## Unit Testing

### Testing Model Hooks

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_model_hook_before() {
        let hook = MyModelHook::new("test-hook", 100);
        let context = ModelHookContext::before(
            "test input".to_string(),
            "test-model".to_string(),
        );

        let result = hook.before_model_call(&context).await.unwrap();
        assert!(result.success);
        assert!(result.should_continue);
    }

    #[tokio::test]
    async fn test_model_hook_after() {
        let hook = MyModelHook::new("test-hook", 100);
        let context = ModelHookContext::after(
            "test input".to_string(),
            "test-model".to_string(),
            "test response".to_string(),
        );

        let result = hook.after_model_call(&context).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_model_hook_input_modification() {
        let hook = MyModelHook::new("test-hook", 100);
        let context = ModelHookContext::before(
            "original input".to_string(),
            "test-model".to_string(),
        );

        let result = hook.before_model_call(&context).await.unwrap();
        assert!(result.modified_data.is_some());
        
        if let Some(data) = result.modified_data {
            let modified_input = data.get("modified_input").and_then(|v| v.as_str()).unwrap();
            assert_eq!(modified_input, "PREFIX: original input");
        }
    }
}
```

### Testing Tool Hooks

```rust
#[tokio::test]
async fn test_tool_hook_validation() {
    let hook = ValidationToolHook::new("test-validation", 200);
    let context = ToolHookContext::before(
        "read_file".to_string(),
        json!({"path": "../../etc/passwd"}),
    );

    let result = hook.before_tool_execution(&context).await.unwrap();
    assert!(!result.should_continue); // Should block path traversal
}

#[tokio::test]
async fn test_tool_hook_valid_path() {
    let hook = ValidationToolHook::new("test-validation", 200);
    let context = ToolHookContext::before(
        "read_file".to_string(),
        json!({"path": "src/main.rs"}),
    );

    let result = hook.before_tool_execution(&context).await.unwrap();
    assert!(result.should_continue); // Should allow valid path
}
```

### Testing Error Hooks

```rust
#[tokio::test]
async fn test_error_hook_recovery() {
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
async fn test_error_hook_transformation() {
    let hook = ErrorTransformationHook::new("test-transform", 100);
    let context = ErrorHookContext::transformation(
        "Connection timeout".to_string(),
        "TimeoutError".to_string(),
        Some("model_call".to_string()),
    );

    let result = hook.error_transformation(&context).await.unwrap();
    assert!(result.modified_data.is_some());
    
    if let Some(data) = result.modified_data {
        let transformed = data.get("transformed_error").and_then(|v| v.as_str()).unwrap();
        assert!(transformed.contains("timed out"));
    }
}
```

## Integration Testing

### Testing with Hook Registry

```rust
#[tokio::test]
async fn test_hook_registration() {
    let registry = Arc::new(HookRegistry::new());
    let hook = Arc::new(MyHook::new("test-hook", 100));
    
    registry.register(hook.clone()).await.unwrap();
    
    let hooks = registry.get_hooks(HookType::BeforeModel).await;
    assert_eq!(hooks.len(), 1);
    assert_eq!(hooks[0].name(), "test-hook");
}

#[tokio::test]
async fn test_hook_execution() {
    let registry = Arc::new(HookRegistry::new());
    let hook = Arc::new(MyHook::new("test-hook", 100));
    
    registry.register(hook).await.unwrap();
    
    let context = HookContext::new(
        "before_model",
        json!({"input": "test"}),
    );
    
    let results = registry.execute_hooks(HookType::BeforeModel, &context).await.unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].success);
}
```

### Testing Hook Execution Order

```rust
#[tokio::test]
async fn test_hook_priority_order() {
    let registry = Arc::new(HookRegistry::new());
    
    // Register hooks with different priorities
    let hook1 = Arc::new(MyHook::new("hook-1", 100));
    let hook2 = Arc::new(MyHook::new("hook-2", 200)); // Higher priority
    let hook3 = Arc::new(MyHook::new("hook-3", 50));  // Lower priority
    
    registry.register(hook1).await.unwrap();
    registry.register(hook2).await.unwrap();
    registry.register(hook3).await.unwrap();
    
    let hooks = registry.get_hooks(HookType::BeforeModel).await;
    
    // Hooks should be sorted by priority (descending)
    assert_eq!(hooks[0].name(), "hook-2"); // Highest priority
    assert_eq!(hooks[1].name(), "hook-1");
    assert_eq!(hooks[2].name(), "hook-3"); // Lowest priority
}
```

### Testing Enable/Disable

```rust
#[tokio::test]
async fn test_hook_enable_disable() {
    let registry = Arc::new(HookRegistry::new());
    let hook = Arc::new(MyHook::new("test-hook", 100));
    
    registry.register(hook).await.unwrap();
    
    // Hook should be enabled by default
    let context = HookContext::new("before_model", json!({}));
    let results = registry.execute_hooks(HookType::BeforeModel, &context).await.unwrap();
    assert_eq!(results.len(), 1);
    
    // Disable hook
    registry.set_enabled("test-hook", false).await.unwrap();
    let results = registry.execute_hooks(HookType::BeforeModel, &context).await.unwrap();
    assert_eq!(results.len(), 0); // Should not execute
    
    // Re-enable hook
    registry.set_enabled("test-hook", true).await.unwrap();
    let results = registry.execute_hooks(HookType::BeforeModel, &context).await.unwrap();
    assert_eq!(results.len(), 1); // Should execute again
}
```

## Testing Context Modifications

### Testing Input Modification

```rust
#[tokio::test]
async fn test_input_modification() {
    let hook = InputModificationHook::new("test-modify", 100);
    let context = ModelHookContext::before(
        "original".to_string(),
        "test-model".to_string(),
    );

    let result = hook.before_model_call(&context).await.unwrap();
    
    assert!(result.modified_data.is_some());
    if let Some(data) = result.modified_data {
        let modified = data.get("modified_input").and_then(|v| v.as_str()).unwrap();
        assert_eq!(modified, "PREFIX: original");
    }
}
```

### Testing Response Modification

```rust
#[tokio::test]
async fn test_response_modification() {
    let hook = ResponseModificationHook::new("test-modify", 100);
    let context = ModelHookContext::after(
        "input".to_string(),
        "test-model".to_string(),
        "original response".to_string(),
    );

    let result = hook.after_model_call(&context).await.unwrap();
    
    assert!(result.modified_data.is_some());
    if let Some(data) = result.modified_data {
        let modified = data.get("response").and_then(|v| v.as_str()).unwrap();
        assert!(modified.contains("MODIFIED"));
    }
}
```

## Mocking and Test Doubles

### Creating Test Doubles

```rust
// Test double for ModelHook
pub struct MockModelHook {
    name: String,
    priority: HookPriority,
    should_fail: bool,
}

impl MockModelHook {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            priority: HookPriority::new(100),
            should_fail: false,
        }
    }

    pub fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }
}

#[async_trait]
impl ModelHook for MockModelHook {
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> HookPriority {
        self.priority
    }

    async fn before_model_call(&self, _context: &ModelHookContext) -> Result<HookExecutionResult> {
        if self.should_fail {
            Err(HookError::ExecutionFailed("Mock failure".to_string()))
        } else {
            Ok(HookExecutionResult::success())
        }
    }

    async fn after_model_call(&self, _context: &ModelHookContext) -> Result<HookExecutionResult> {
        Ok(HookExecutionResult::success())
    }
}
```

### Using Test Doubles

```rust
#[tokio::test]
async fn test_with_mock_hook() {
    let registry = Arc::new(HookRegistry::new());
    let mock_hook = Arc::new(MockModelHook::new("mock-hook"));
    let adapter = ModelHookAdapter::before(mock_hook);
    
    registry.register(adapter).await.unwrap();
    
    let context = HookContext::new("before_model", json!({}));
    let results = registry.execute_hooks(HookType::BeforeModel, &context).await.unwrap();
    
    assert_eq!(results.len(), 1);
    assert!(results[0].success);
}
```

## Testing Error Scenarios

### Testing Hook Failures

```rust
#[tokio::test]
async fn test_hook_failure_doesnt_stop_others() {
    let registry = Arc::new(HookRegistry::new());
    
    // Register failing hook
    let failing_hook = Arc::new(MockModelHook::new("failing-hook").with_failure());
    registry.register(ModelHookAdapter::before(failing_hook)).await.unwrap();
    
    // Register successful hook
    let success_hook = Arc::new(MockModelHook::new("success-hook"));
    registry.register(ModelHookAdapter::before(success_hook)).await.unwrap();
    
    let context = HookContext::new("before_model", json!({}));
    let results = registry.execute_hooks(HookType::BeforeModel, &context).await.unwrap();
    
    // Both hooks should execute (failure doesn't stop others)
    assert_eq!(results.len(), 2);
    // First result is error, second is success
    assert!(!results[0].success);
    assert!(results[1].success);
}
```

### Testing Early Termination

```rust
#[tokio::test]
async fn test_early_termination() {
    let registry = Arc::new(HookRegistry::new());
    
    // Hook that stops execution
    let stop_hook = Arc::new(StopHook::new("stop-hook", 200));
    registry.register(ModelHookAdapter::before(stop_hook)).await.unwrap();
    
    // Hook that should not execute
    let later_hook = Arc::new(MockModelHook::new("later-hook"));
    registry.register(ModelHookAdapter::before(later_hook)).await.unwrap();
    
    let context = HookContext::new("before_model", json!({}));
    let results = registry.execute_hooks(HookType::BeforeModel, &context).await.unwrap();
    
    // Only first hook should execute
    assert_eq!(results.len(), 1);
    assert!(!results[0].should_continue);
}
```

## Performance Testing

### Benchmarking Hooks

```rust
#[tokio::test]
async fn test_hook_performance() {
    use std::time::Instant;
    
    let hook = MyHook::new("test-hook", 100);
    let context = HookContext::new("before_model", json!({}));
    
    let start = Instant::now();
    for _ in 0..1000 {
        hook.execute(&context).await.unwrap();
    }
    let duration = start.elapsed();
    
    // Should complete 1000 executions in reasonable time
    assert!(duration.as_millis() < 1000);
}
```

## Best Practices

1. **Test in Isolation**: Test each hook independently
2. **Test Edge Cases**: Test with empty inputs, large inputs, invalid data
3. **Test Error Handling**: Verify hooks handle errors gracefully
4. **Test Modifications**: Verify context modifications work correctly
5. **Test Priority Order**: Ensure hooks execute in correct order
6. **Test Enable/Disable**: Verify enable/disable functionality
7. **Use Mocks**: Use test doubles for complex dependencies
8. **Test Performance**: Ensure hooks don't introduce significant overhead

## Summary

- Unit test individual hooks
- Integration test with registry
- Test hook execution order
- Test context modifications
- Test error scenarios
- Use mocks for complex dependencies
- Test performance characteristics

