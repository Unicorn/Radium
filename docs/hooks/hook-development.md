# Hook Development Guide

This guide will help you create custom hooks for Radium. Hooks allow you to intercept and customize behavior at various points in the execution flow.

## Overview

Hooks in Radium implement the `Hook` trait and are registered with the `HookRegistry`. They can be implemented in Rust and loaded dynamically, or configured via TOML for simpler use cases.

## Creating a Hook

### Step 1: Create a New Cargo Project

Create a new library crate for your hook:

```bash
cargo new --lib my-hook
cd my-hook
```

### Step 2: Add Dependencies

Update your `Cargo.toml`:

```toml
[package]
name = "my-hook"
version = "0.1.0"
edition = "2021"

[lib]
name = "my_hook"
crate-type = ["cdylib", "rlib"]

[dependencies]
radium-core = { path = "../../../crates/radium-core" }
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
```

### Step 3: Implement the Hook Trait

For model hooks, implement the `ModelHook` trait:

```rust
use async_trait::async_trait;
use radium_core::hooks::model::{ModelHook, ModelHookContext};
use radium_core::hooks::types::{HookPriority, HookResult};
use radium_core::hooks::error::Result;

pub struct MyModelHook {
    name: String,
    priority: HookPriority,
}

impl MyModelHook {
    pub fn new(name: impl Into<String>, priority: u32) -> Self {
        Self {
            name: name.into(),
            priority: HookPriority::new(priority),
        }
    }
}

#[async_trait]
impl ModelHook for MyModelHook {
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> HookPriority {
        self.priority
    }

    async fn before_model_call(&self, context: &ModelHookContext) -> Result<HookResult> {
        // Your logic here
        tracing::info!("Before model call: {}", context.model_id);
        Ok(HookResult::success())
    }

    async fn after_model_call(&self, context: &ModelHookContext) -> Result<HookResult> {
        // Your logic here
        tracing::info!("After model call: {}", context.model_id);
        Ok(HookResult::success())
    }
}
```

For telemetry hooks, implement the `Hook` trait directly:

```rust
use async_trait::async_trait;
use radium_core::hooks::registry::{Hook, HookType};
use radium_core::hooks::types::{HookContext, HookPriority, HookResult};
use radium_core::hooks::error::Result;

pub struct MyTelemetryHook {
    name: String,
    priority: HookPriority,
}

#[async_trait]
impl Hook for MyTelemetryHook {
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> HookPriority {
        self.priority
    }

    fn hook_type(&self) -> HookType {
        HookType::TelemetryCollection
    }

    async fn execute(&self, context: &HookContext) -> Result<HookResult> {
        // Extract telemetry data from context.data
        let data = &context.data;
        // Your logic here
        Ok(HookResult::success())
    }
}
```

### Step 4: Create Hook Adapters

For model hooks, use `ModelHookAdapter`:

```rust
use radium_core::hooks::model::ModelHookAdapter;
use std::sync::Arc;

pub fn create_before_hook() -> Arc<dyn radium_core::hooks::registry::Hook> {
    let hook = Arc::new(MyModelHook::new("my-hook-before", 100));
    ModelHookAdapter::before(hook)
}

pub fn create_after_hook() -> Arc<dyn radium_core::hooks::registry::Hook> {
    let hook = Arc::new(MyModelHook::new("my-hook-after", 100));
    ModelHookAdapter::after(hook)
}
```

### Step 5: Build and Package

Build your hook:

```bash
cargo build --release
```

The compiled library can be loaded dynamically or linked statically.

## Hook Contexts

Different hook types receive different context data:

### Model Hook Context

```rust
pub struct ModelHookContext {
    pub input: String,
    pub model_id: String,
    pub request_modifications: Option<serde_json::Value>,
    pub response: Option<String>,  // Only for after hooks
    pub modified_input: Option<String>,
}
```

### Tool Hook Context

```rust
pub struct ToolHookContext {
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub result: Option<serde_json::Value>,  // Only for after hooks
    pub modified_arguments: Option<serde_json::Value>,
    pub modified_result: Option<serde_json::Value>,
}
```

### Telemetry Hook Context

Telemetry data is passed as JSON in `HookContext.data`:

```json
{
    "agent_id": "agent-123",
    "input_tokens": 100,
    "output_tokens": 50,
    "total_tokens": 150,
    "estimated_cost": 0.001,
    "model": "gpt-4",
    "provider": "openai"
}
```

## Hook Results

Hooks return `HookResult` which can:

- **Continue execution**: `HookResult::success()`
- **Modify data**: `HookResult::with_data(modified_data)`
- **Stop execution**: `HookResult::stop("reason")`
- **Report error but continue**: `HookResult::error("message")`

### Modifying Execution Flow

To modify input/output:

```rust
// Modify input in before_model hook
let modified_input = format!("Prefix: {}", context.input);
let modified_data = json!({
    "input": modified_input,
});
Ok(HookResult::with_data(modified_data))
```

To stop execution:

```rust
// Stop execution if condition is met
if some_condition {
    Ok(HookResult::stop("Execution stopped by hook"))
} else {
    Ok(HookResult::success())
}
```

## Testing Hooks

Write unit tests for your hooks:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_my_hook() {
        let hook = MyModelHook::new("test-hook", 100);
        let context = ModelHookContext::before(
            "test input".to_string(),
            "test-model".to_string(),
        );
        
        let result = hook.before_model_call(&context).await.unwrap();
        assert!(result.success);
        assert!(result.should_continue);
    }
}
```

## Best Practices

1. **Error Handling**: Always handle errors gracefully. Don't let hook failures crash the system.

2. **Performance**: Keep hook execution fast. Long-running operations should be async.

3. **Thread Safety**: Hooks must be `Send + Sync`. Use `Arc` and `RwLock` for shared state.

4. **Logging**: Use the `tracing` crate for logging. Don't use `println!`.

5. **Priority**: Choose appropriate priorities. Critical hooks should have high priority.

6. **Idempotency**: Hooks should be idempotent when possible.

7. **Configuration**: Use `HookConfig` for configurable behavior.

## Example Implementations

See the example hooks in `examples/hooks/`:
- `logging-hook`: Logs model calls
- `metrics-hook`: Aggregates telemetry data

## Packaging and Distribution

### Extension Integration

To distribute your hook via extensions:

1. Create an extension manifest (`radium-extension.json`)
2. Add hooks to the `components.hooks` field
3. Package the extension

Example manifest:

```json
{
  "name": "my-hook-extension",
  "version": "1.0.0",
  "description": "My custom hook",
  "author": "Your Name",
  "components": {
    "hooks": ["hooks/*.so"]
  }
}
```

### Workspace Configuration

Alternatively, configure hooks directly in workspace:

```toml
# .radium/hooks.toml
[[hooks]]
name = "my-hook"
type = "before_model"
priority = 100
enabled = true
```

## Debugging

Enable debug logging:

```bash
RUST_LOG=radium_core::hooks=debug rad <command>
```

Check hook registration:

```bash
rad hooks list
rad hooks info my-hook
```

## Next Steps

- See [API Reference](api-reference.md) for complete API documentation
- See [Getting Started](getting-started.md) for usage examples
- Check out example implementations in `examples/hooks/`

