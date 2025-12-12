---
id: "best-practices"
title: "Hook Development Best Practices"
sidebar_label: "Hook Development Best Practices"
---

# Hook Development Best Practices

This document outlines best practices and anti-patterns for developing hooks in Radium.

## Do's ✅

### Keep Hooks Focused on Single Responsibility

**Good**:
```rust
pub struct LoggingHook {
    // Only handles logging
}

impl ModelHook for LoggingHook {
    async fn before_model_call(&self, context: &ModelHookContext) -> Result<HookExecutionResult> {
        tracing::info!("Model call: {}", context.model_id);
        Ok(HookExecutionResult::success())
    }
}
```

**Bad**:
```rust
pub struct LoggingAndValidationHook {
    // Does too much: logging AND validation
}

impl ModelHook for LoggingAndValidationHook {
    async fn before_model_call(&self, context: &ModelHookContext) -> Result<HookExecutionResult> {
        // Logging
        tracing::info!("Model call: {}", context.model_id);
        
        // Validation (should be separate hook)
        if context.input.is_empty() {
            return Ok(HookExecutionResult::stop("Empty input"));
        }
        
        // Metrics (should be separate hook)
        self.metrics.increment();
        
        Ok(HookExecutionResult::success())
    }
}
```

### Use Appropriate Priority Levels

**Guidelines**:
- **High Priority (200+)**: Security checks, critical validation, access control
- **Medium Priority (100-199)**: Standard operations, logging, transformation
- **Low Priority (<100)**: Optional monitoring, non-critical telemetry

**Good**:
```rust
// Security hook - high priority
pub struct SecurityHook {
    priority: HookPriority::new(250),
}

// Logging hook - medium priority
pub struct LoggingHook {
    priority: HookPriority::new(100),
}

// Metrics hook - low priority
pub struct MetricsHook {
    priority: HookPriority::new(50),
}
```

**Bad**:
```rust
// Security hook with low priority - runs after other hooks!
pub struct SecurityHook {
    priority: HookPriority::new(50), // Too low!
}
```

### Handle Errors Gracefully

**Good**:
```rust
async fn execute(&self, context: &HookContext) -> Result<HookExecutionResult> {
    match self.process(context).await {
        Ok(_) => Ok(HookExecutionResult::success()),
        Err(e) => {
            // Log error but don't crash
            tracing::warn!(error = %e, "Hook processing failed");
            // Return success to allow other hooks to run
            Ok(HookExecutionResult::error(format!("Processing failed: {}", e)))
        }
    }
}
```

**Bad**:
```rust
async fn execute(&self, context: &HookContext) -> Result<HookExecutionResult> {
    // Panics on error - crashes entire system!
    self.process(context).await.unwrap();
    Ok(HookExecutionResult::success())
}
```

### Test Hooks in Isolation

**Good**:
```rust
#[tokio::test]
async fn test_hook_in_isolation() {
    let hook = MyHook::new("test", 100);
    let context = create_test_context();
    
    let result = hook.execute(&context).await.unwrap();
    assert!(result.success);
}
```

**Bad**:
```rust
// Testing with real registry and other hooks - not isolated
#[tokio::test]
async fn test_hook_with_registry() {
    let registry = HookRegistry::new();
    registry.register(hook1).await?;
    registry.register(hook2).await?;
    registry.register(my_hook).await?; // Hard to test in isolation
    // ...
}
```

### Use Thread-Safe Patterns

**Good**:
```rust
pub struct SharedStateHook {
    state: Arc<RwLock<HashMap<String, u64>>>,
}

async fn execute(&self, context: &HookContext) -> Result<HookExecutionResult> {
    let mut state = self.state.write().await; // Proper locking
    state.insert(key, value);
    Ok(HookExecutionResult::success())
}
```

**Bad**:
```rust
pub struct UnsafeStateHook {
    state: HashMap<String, u64>, // Not thread-safe!
}

async fn execute(&self, context: &HookContext) -> Result<HookExecutionResult> {
    self.state.insert(key, value); // Race condition!
    Ok(HookExecutionResult::success())
}
```

## Don'ts ❌

### Don't Perform Blocking Operations

**Bad**:
```rust
async fn execute(&self, context: &HookContext) -> Result<HookExecutionResult> {
    // Blocking I/O in async function!
    std::thread::sleep(Duration::from_secs(1));
    std::fs::write("file.txt", "data").unwrap();
    Ok(HookExecutionResult::success())
}
```

**Good**:
```rust
async fn execute(&self, context: &HookContext) -> Result<HookExecutionResult> {
    // Async I/O
    tokio::time::sleep(Duration::from_secs(1)).await;
    tokio::fs::write("file.txt", "data").await?;
    Ok(HookExecutionResult::success())
}
```

### Don't Modify Shared State Without Synchronization

**Bad**:
```rust
pub struct UnsafeCounterHook {
    counter: u64, // Not synchronized!
}

async fn execute(&self, context: &HookContext) -> Result<HookExecutionResult> {
    self.counter += 1; // Race condition!
    Ok(HookExecutionResult::success())
}
```

**Good**:
```rust
pub struct SafeCounterHook {
    counter: Arc<RwLock<u64>>, // Synchronized
}

async fn execute(&self, context: &HookContext) -> Result<HookExecutionResult> {
    let mut counter = self.counter.write().await;
    *counter += 1;
    Ok(HookExecutionResult::success())
}
```

### Don't Ignore Hook Execution Failures

**Bad**:
```rust
async fn execute(&self, context: &HookContext) -> Result<HookExecutionResult> {
    // Ignores errors silently
    let _ = self.process(context).await;
    Ok(HookExecutionResult::success())
}
```

**Good**:
```rust
async fn execute(&self, context: &HookContext) -> Result<HookExecutionResult> {
    match self.process(context).await {
        Ok(_) => Ok(HookExecutionResult::success()),
        Err(e) => {
            tracing::warn!(error = %e, "Hook processing failed");
            Ok(HookExecutionResult::error(e.to_string()))
        }
    }
}
```

### Don't Use High Priority for Non-Critical Operations

**Bad**:
```rust
// Logging doesn't need high priority
pub struct LoggingHook {
    priority: HookPriority::new(250), // Too high!
}
```

**Good**:
```rust
// Logging with appropriate priority
pub struct LoggingHook {
    priority: HookPriority::new(100), // Appropriate
}
```

### Don't Store Large Data in Context

**Bad**:
```rust
async fn execute(&self, context: &HookContext) -> Result<HookExecutionResult> {
    // Storing large data in context
    let large_data = vec![0u8; 10_000_000];
    Ok(HookExecutionResult::with_data(json!({
        "large_data": large_data // Too large!
    })))
}
```

**Good**:
```rust
async fn execute(&self, context: &HookContext) -> Result<HookExecutionResult> {
    // Store reference or summary instead
    let summary = calculate_summary(&large_data);
    Ok(HookExecutionResult::with_data(json!({
        "summary": summary
    })))
}
```

## Hook Type Selection

### When to Use Model Hooks

- Input/output validation
- Request/response transformation
- Logging model calls
- Cost tracking
- Rate limiting

### When to Use Tool Hooks

- Tool argument validation
- Security checks
- Tool execution logging
- Tool result transformation
- Access control

### When to Use Error Hooks

- Error transformation
- Error recovery
- Error logging
- Error notification
- Error aggregation

### When to Use Telemetry Hooks

- Metrics collection
- Performance monitoring
- Cost tracking
- Usage analytics
- Custom logging

## Priority Selection Guidelines

### High Priority (200+)

Use for:
- Security checks
- Critical validation
- Access control
- Safety checks

Example:
```rust
pub struct SecurityHook {
    priority: HookPriority::new(250),
}
```

### Medium Priority (100-199)

Use for:
- Standard operations
- Logging
- Transformation
- Standard validation

Example:
```rust
pub struct LoggingHook {
    priority: HookPriority::new(100),
}
```

### Low Priority (<100)

Use for:
- Optional monitoring
- Non-critical telemetry
- Background tasks
- Caching

Example:
```rust
pub struct MetricsHook {
    priority: HookPriority::new(50),
}
```

## Performance Considerations

### Keep Hook Execution Fast

**Good**:
```rust
async fn execute(&self, context: &HookContext) -> Result<HookExecutionResult> {
    // Fast operation
    let result = self.cache.get(&key).await;
    Ok(HookExecutionResult::success())
}
```

**Bad**:
```rust
async fn execute(&self, context: &HookContext) -> Result<HookExecutionResult> {
    // Slow operation blocks execution
    tokio::time::sleep(Duration::from_secs(10)).await;
    Ok(HookExecutionResult::success())
}
```

### Use Async Operations

**Good**:
```rust
async fn execute(&self, context: &HookContext) -> Result<HookExecutionResult> {
    tokio::fs::read("file.txt").await?;
    Ok(HookExecutionResult::success())
}
```

**Bad**:
```rust
async fn execute(&self, context: &HookContext) -> Result<HookExecutionResult> {
    std::fs::read("file.txt")?; // Blocking!
    Ok(HookExecutionResult::success())
}
```

### Cache Expensive Operations

**Good**:
```rust
pub struct CachedHook {
    cache: Arc<RwLock<HashMap<String, String>>>,
}

async fn execute(&self, context: &HookContext) -> Result<HookExecutionResult> {
    let key = context.data.get("key").and_then(|v| v.as_str()).unwrap();
    
    // Check cache first
    if let Some(cached) = self.cache.read().await.get(key) {
        return Ok(HookExecutionResult::with_data(json!({
            "result": cached
        })));
    }
    
    // Expensive operation only if not cached
    let result = expensive_operation().await?;
    self.cache.write().await.insert(key.to_string(), result.clone());
    
    Ok(HookExecutionResult::with_data(json!({
        "result": result
    })))
}
```

## Summary

- ✅ Keep hooks focused on single responsibility
- ✅ Use appropriate priority levels
- ✅ Handle errors gracefully
- ✅ Test hooks in isolation
- ✅ Use thread-safe patterns
- ❌ Don't perform blocking operations
- ❌ Don't modify shared state without synchronization
- ❌ Don't ignore hook execution failures
- ❌ Don't use high priority for non-critical operations
- ❌ Don't store large data in context

