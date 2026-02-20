---
id: "tutorial"
title: "Hooks System Tutorial"
sidebar_label: "Hooks System Tutorial"
---

# Hooks System Tutorial

This tutorial will guide you through creating your first custom hook for Radium. By the end, you'll have built a working "Request Logger Hook" that logs all model calls.

## What You'll Build

A simple hook that logs model calls with timestamps and metadata. This demonstrates the core concepts of hook development in Radium.

## Prerequisites

- Basic Rust knowledge
- Radium workspace set up
- Rust toolchain installed
- Understanding of async/await in Rust

## Learning Objectives

By the end of this tutorial, you will:
- Understand how to implement the `ModelHook` trait
- Know how to register hooks with the hook registry
- Be able to configure hooks in workspace settings
- Know how to test hooks
- Understand how to debug common hook issues

## Step 1: Create Hook Project Structure

First, create a new Rust library project for your hook:

```bash
cd examples/hooks
cargo new --lib tutorial-hook
cd tutorial-hook
```

This creates a new library crate that we'll use for our hook implementation.

## Step 2: Add Dependencies

Update `Cargo.toml` with the necessary dependencies:

```toml
[package]
name = "tutorial-hook"
version = "0.1.0"
edition = "2021"

[lib]
name = "tutorial_hook"
crate-type = ["cdylib", "rlib"]

[dependencies]
radium-core = { path = "../../../crates/radium-core" }
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
chrono = { version = "0.4", features = ["serde"] }
```

**Checkpoint 1**: Verify your `Cargo.toml` matches the above. Run `cargo check` to ensure dependencies resolve correctly.

## Step 3: Implement the ModelHook Trait

Now, let's implement our hook. Create `src/lib.rs`:

```rust
//! Tutorial hook: Request Logger
//!
//! This hook logs all model calls with timestamps and metadata.

use async_trait::async_trait;
use radium_core::hooks::model::{ModelHook, ModelHookContext};
use radium_core::hooks::types::{HookPriority, HookResult as HookExecutionResult};
use radium_core::hooks::error::Result;
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
```

**Checkpoint 2**: Your code should compile. Run `cargo build` to verify.

## Step 4: Create Hook Adapters

Hooks need adapters to work with the registry. Add these functions to `src/lib.rs`:

```rust
use std::sync::Arc;
use radium_core::hooks::model::ModelHookAdapter;

/// Create a before model call hook adapter.
pub fn create_before_hook() -> Arc<dyn radium_core::hooks::registry::Hook> {
    let hook = Arc::new(RequestLoggerHook::new("tutorial-logger-before", 100));
    ModelHookAdapter::before(hook)
}

/// Create an after model call hook adapter.
pub fn create_after_hook() -> Arc<dyn radium_core::hooks::registry::Hook> {
    let hook = Arc::new(RequestLoggerHook::new("tutorial-logger-after", 100));
    ModelHookAdapter::after(hook)
}
```

**Checkpoint 3**: Build again with `cargo build`. Everything should compile successfully.

## Step 5: Add Tests

Let's add a simple test to verify our hook works:

```rust
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
}
```

Run tests with `cargo test`. Both tests should pass.

**Checkpoint 4**: All tests pass. Your hook implementation is complete!

## Step 6: Register the Hook

To use your hook, you need to register it. Create a simple registration example:

```rust
use radium_core::hooks::registry::HookRegistry;
use std::sync::Arc;

pub async fn register_tutorial_hooks(registry: &Arc<HookRegistry>) -> Result<()> {
    registry.register(create_before_hook()).await?;
    registry.register(create_after_hook()).await?;
    Ok(())
}
```

## Step 7: Configure in Workspace

Create a configuration file `.radium/hooks.toml` in your workspace:

```toml
[[hooks]]
name = "tutorial-logger-before"
type = "before_model"
priority = 100
enabled = true

[[hooks]]
name = "tutorial-logger-after"
type = "after_model"
priority = 100
enabled = true
```

## Step 8: Test with Real Execution

To test your hook with real agent execution:

1. Register your hooks in your application code
2. Run a model call
3. Check the logs for your hook's output

You should see log messages like:
```
📝 Before model call: model=test-model, input_length=10
✅ After model call: model=test-model, input_length=10, response_length=20
```

**Checkpoint 5**: Your hook logs appear in the output when model calls are made.

## Troubleshooting

### Hook Not Executing

**Problem**: Hook is registered but doesn't execute.

**Solutions**:
1. Check if hook is enabled: `rad hooks info tutorial-logger-before`
2. Verify hook type matches execution point
3. Check hook priority (might be too low)
4. Ensure hook is registered before execution

### No Logs Appearing

**Problem**: Hook executes but no logs appear.

**Solutions**:
1. Set log level: `RUST_LOG=radium_core::hooks=info`
2. Check that `tracing` is configured in your application
3. Verify hook is actually being called (add a breakpoint)

### Compilation Errors

**Problem**: Code doesn't compile.

**Solutions**:
1. Check all dependencies are in `Cargo.toml`
2. Verify Rust edition is "2021"
3. Ensure `async-trait` is included
4. Check trait implementations match exactly

### Hook Registration Fails

**Problem**: `registry.register()` returns an error.

**Solutions**:
1. Verify hook implements all required traits
2. Check hook name is unique
3. Ensure adapter is created correctly
4. Verify registry is properly initialized

## What You Learned

Congratulations! You've successfully:

✅ Created a custom hook from scratch  
✅ Implemented the `ModelHook` trait  
✅ Registered hooks with the registry  
✅ Configured hooks in workspace settings  
✅ Tested hook execution  
✅ Debugged common issues  

## Next Steps

Now that you've created your first hook, you can:

- Explore [Creating Hooks](creating-hooks.md) for more advanced patterns
- Check out [Hook Types](hook-types.md) to learn about other hook types
- Review [Examples](examples.md) for practical patterns
- Read [Best Practices](best-practices.md) for development guidelines
- Study [API Reference](api-reference.md) for complete API documentation

## Advanced Topics

Once you're comfortable with basic hooks:

- **Tool Hooks**: Intercept tool execution
- **Error Hooks**: Handle errors with custom logic
- **Telemetry Hooks**: Collect metrics and telemetry
- **Priority Management**: Control execution order
- **Context Modification**: Modify inputs/outputs
- **Configuration**: Make hooks configurable

## Summary

This tutorial walked you through:
1. Creating a hook project
2. Implementing the `ModelHook` trait
3. Creating hook adapters
4. Testing your hook
5. Registering and configuring hooks
6. Troubleshooting common issues

You now have the foundation to create more complex hooks for your specific use cases!

