---
id: "troubleshooting"
title: "Troubleshooting Hooks"
sidebar_label: "Troubleshooting Hooks"
---

# Troubleshooting Hooks

This guide helps you diagnose and fix common issues when working with hooks in Radium.

## Hook Not Executing

### Symptoms
- Hook is registered but never executes
- No logs or output from hook
- Hook appears in `rad hooks list` but doesn't run

### Possible Causes and Solutions

#### Hook is Disabled

**Check**:
```bash
rad hooks info my-hook
```

**Solution**:
```bash
rad hooks enable my-hook
```

Or in code:
```rust
registry.set_enabled("my-hook", true).await?;
```

#### Wrong Hook Type

**Check**: Verify the hook type matches the execution point.

**Solution**: Ensure hook type matches:
- `BeforeModel` hooks only execute before model calls
- `BeforeTool` hooks only execute before tool execution
- etc.

#### Hook Not Registered

**Check**: Verify hook is registered:
```rust
let hooks = registry.get_hooks(HookType::BeforeModel).await;
assert!(hooks.iter().any(|h| h.name() == "my-hook"));
```

**Solution**: Register the hook:
```rust
registry.register(hook).await?;
```

#### Priority Too Low

**Check**: If other hooks stop execution (`should_continue = false`), lower priority hooks won't run.

**Solution**: Increase hook priority:
```rust
let hook = MyHook::new("my-hook", 250); // Higher priority
```

## Priority Issues

### Symptoms
- Hooks execute in wrong order
- Important hooks run after less important ones

### Solution

Hooks execute in priority order (higher priority first). Check priorities:

```rust
let hooks = registry.get_hooks(HookType::BeforeModel).await;
for hook in hooks {
    println!("{}: priority {}", hook.name(), hook.priority().value());
}
```

**Guidelines**:
- Security/validation: 200+
- Standard operations: 100-199
- Optional/monitoring: &lt;100

## Context Serialization Errors

### Symptoms
- `Serialization` errors when executing hooks
- `InvalidConfig` errors
- Data not accessible in context

### Possible Causes and Solutions

#### Invalid JSON in Context

**Error**:
```
Serialization error: invalid JSON
```

**Solution**: Ensure context data is valid JSON:
```rust
let context = HookContext::new(
    "before_model",
    json!({
        "input": "valid string",
        "count": 42,
    }),
);
```

#### Type Mismatch

**Error**:
```
Failed to get field: type mismatch
```

**Solution**: Check types when accessing context:
```rust
// Wrong
let count = context.data.get("count").unwrap().as_u64().unwrap();

// Right - handle Option
if let Some(count) = context.data.get("count").and_then(|v| v.as_u64()) {
    // Use count
}
```

#### Missing Fields

**Error**:
```
Field 'input' not found
```

**Solution**: Check for field existence:
```rust
if let Some(input) = context.data.get("input").and_then(|v| v.as_str()) {
    // Use input
} else {
    // Handle missing field
}
```

## Performance Problems

### Symptoms
- Slow execution
- High latency
- Timeouts

### Possible Causes and Solutions

#### Blocking Operations

**Problem**: Using blocking I/O in async hooks.

**Solution**: Use async operations:
```rust
// Bad
std::fs::read("file.txt")?;

// Good
tokio::fs::read("file.txt").await?;
```

#### Expensive Operations

**Problem**: Performing expensive computations in hooks.

**Solution**: Cache results or move to background:
```rust
// Cache expensive operations
if let Some(cached) = self.cache.get(&key).await {
    return Ok(HookExecutionResult::with_data(cached));
}

let result = expensive_operation().await?;
self.cache.set(&key, result.clone()).await;
```

#### Too Many Hooks

**Problem**: Registering too many hooks slows execution.

**Solution**: 
- Disable unused hooks
- Combine related hooks
- Use lower priority for non-critical hooks

## Thread Safety Issues

### Symptoms
- Race conditions
- Data corruption
- Panics in concurrent execution

### Possible Causes and Solutions

#### Unsynchronized Shared State

**Problem**: Modifying shared state without locks.

**Solution**: Use `Arc<RwLock<>>`:
```rust
// Bad
pub struct UnsafeHook {
    counter: u64, // Not thread-safe!
}

// Good
pub struct SafeHook {
    counter: Arc<RwLock<u64>>, // Thread-safe
}

async fn execute(&self, _context: &HookContext) -> Result<HookExecutionResult> {
    let mut counter = self.counter.write().await;
    *counter += 1;
    Ok(HookExecutionResult::success())
}
```

#### Deadlocks

**Problem**: Holding locks too long or acquiring multiple locks.

**Solution**: 
- Keep lock scope minimal
- Acquire locks in consistent order
- Use `try_lock` when appropriate

## Error Handling Issues

### Symptoms
- Hooks crash on errors
- Errors not logged
- Execution stops unexpectedly

### Possible Causes and Solutions

#### Unhandled Errors

**Problem**: Not handling errors in hooks.

**Solution**: Always handle errors:
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

#### Panicking on Error

**Problem**: Using `unwrap()` or panicking.

**Solution**: Use proper error handling:
```rust
// Bad
let value = context.data.get("key").unwrap().as_str().unwrap();

// Good
if let Some(value) = context.data.get("key").and_then(|v| v.as_str()) {
    // Use value
} else {
    return Ok(HookExecutionResult::error("Missing key"));
}
```

## Configuration Issues

### Symptoms
- Configuration not loading
- Hooks not enabled from config
- Invalid configuration errors

### Possible Causes and Solutions

#### Invalid TOML

**Error**:
```
ConfigParse error: invalid TOML
```

**Solution**: Validate TOML syntax:
```toml
[[hooks]]
name = "my-hook"
type = "before_model"
priority = 100
enabled = true
```

#### Missing Required Fields

**Error**:
```
InvalidConfig: missing required field 'name'
```

**Solution**: Ensure all required fields are present:
- `name`: Hook name
- `type`: Hook type
- `priority`: Priority (optional, defaults to 100)
- `enabled`: Enable state (optional, defaults to true)

#### Configuration Not Loaded

**Problem**: Configuration file exists but hooks not configured.

**Solution**: Load configuration:
```rust
HookLoader::load_from_workspace(workspace_root, &registry).await?;
```

## Integration Issues

### Symptoms
- Hooks not working with orchestrator
- Integration errors
- Missing dependencies

### Possible Causes and Solutions

#### Missing Feature Flag

**Error**:
```
error: cannot find type `OrchestratorHooks` in module `hooks`
```

**Solution**: Enable `orchestrator-integration` feature:
```toml
[dependencies]
radium-core = { path = "../../crates/radium-core", features = ["orchestrator-integration"] }
```

#### Circular Dependency

**Error**:
```
circular dependency detected
```

**Solution**: Use feature flags to break circular dependencies (already handled in core).

## Debugging Tips

### Enable Debug Logging

```bash
RUST_LOG=radium_core::hooks=debug rad <command>
```

### Check Hook Registration

```rust
let hooks = registry.get_hooks(HookType::BeforeModel).await;
for hook in hooks {
    println!("Registered: {} (priority: {})", hook.name(), hook.priority().value());
}
```

### Verify Hook Execution

Add logging to hooks:
```rust
async fn execute(&self, context: &HookContext) -> Result<HookExecutionResult> {
    tracing::debug!(hook = %self.name, "Hook executing");
    // ... hook logic
    tracing::debug!(hook = %self.name, "Hook completed");
    Ok(HookExecutionResult::success())
}
```

### Test Hook in Isolation

```rust
#[tokio::test]
async fn test_hook() {
    let hook = MyHook::new("test", 100);
    let context = create_test_context();
    let result = hook.execute(&context).await.unwrap();
    assert!(result.success);
}
```

## Common Error Messages

### "Hook not found"

**Cause**: Hook not registered or wrong name.

**Solution**: Register hook or check name:
```rust
registry.register(hook).await?;
```

### "Hook execution failed"

**Cause**: Hook returned an error.

**Solution**: Check hook implementation and error handling.

### "Invalid hook configuration"

**Cause**: Configuration file has invalid syntax or missing fields.

**Solution**: Validate configuration file syntax and required fields.

### "Rate limit exceeded"

**Cause**: Too many calls in time window (if using rate limit hook).

**Solution**: Adjust rate limit configuration or wait for window to reset.

## Getting Help

If you're still experiencing issues:

1. Check the [API Reference](api-reference.md) for correct usage
2. Review [Best Practices](best-practices.md) for common patterns
3. Look at [Examples](examples.md) for working code
4. Enable debug logging to see detailed execution flow
5. Test hooks in isolation to identify the issue

## Summary

- **Hook not executing**: Check enable status, registration, hook type, priority
- **Priority issues**: Verify priorities and execution order
- **Serialization errors**: Check JSON validity and type handling
- **Performance problems**: Use async operations, cache results, reduce hook count
- **Thread safety**: Use `Arc<RwLock<>>` for shared state
- **Error handling**: Always handle errors gracefully
- **Configuration**: Validate TOML syntax and required fields
- **Integration**: Enable required feature flags

