# Creating Custom Hooks

This guide walks you through creating custom hooks for the Radium hooks system.

## Overview

Hooks in Radium allow you to intercept and customize behavior at various execution points. You can create hooks for:
- Model calls (before/after)
- Tool execution (before/after/selection)
- Error handling (interception, transformation, recovery, logging)
- Telemetry collection

## Quick Start

### Step 1: Create a Hook Project

Create a new Rust library project:

```bash
cargo new --lib my-custom-hook
cd my-custom-hook
```

### Step 2: Add Dependencies

Update `Cargo.toml`:

```toml
[package]
name = "my-custom-hook"
version = "0.1.0"
edition = "2021"

[dependencies]
radium-core = { path = "../../crates/radium-core" }
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
tokio = { version = "1.0", features = ["rt-multi-thread"] }
```

### Step 3: Implement Your Hook

Choose the appropriate trait based on your use case:

- **ModelHook**: For model call hooks
- **ToolHook**: For tool execution hooks
- **ErrorHook**: For error handling hooks
- **Hook**: For telemetry or custom hooks

## Model Hook Example

Create a hook that logs and validates model calls:

```rust
use async_trait::async_trait;
use radium_core::hooks::model::{ModelHook, ModelHookContext};
use radium_core::hooks::types::{HookPriority, HookResult as HookExecutionResult};
use radium_core::hooks::error::Result;
use serde_json::json;
use tracing::info;

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

    async fn before_model_call(&self, context: &ModelHookContext) -> Result<HookExecutionResult> {
        // Validate input
        if context.input.is_empty() {
            return Ok(HookExecutionResult::stop("Input cannot be empty"));
        }

        // Log the call
        info!(
            hook = %self.name,
            model = %context.model_id,
            input_len = context.input.len(),
            "Before model call"
        );

        // Optionally modify input
        let modified = format!("[HOOK] {}", context.input);
        Ok(HookExecutionResult::with_data(json!({
            "modified_input": modified
        })))
    }

    async fn after_model_call(&self, context: &ModelHookContext) -> Result<HookExecutionResult> {
        // Log the response
        if let Some(response) = &context.response {
            info!(
                hook = %self.name,
                model = %context.model_id,
                response_len = response.len(),
                "After model call"
            );
        }

        Ok(HookExecutionResult::success())
    }
}
```

### Registering Model Hooks

Use adapters to register model hooks:

```rust
use radium_core::hooks::model::ModelHookAdapter;
use radium_core::hooks::registry::HookRegistry;
use std::sync::Arc;

let registry = Arc::new(HookRegistry::new());
let hook = Arc::new(MyModelHook::new("my-model-hook", 100));

// Register before hook
let before_adapter = ModelHookAdapter::before(hook.clone());
registry.register(before_adapter).await?;

// Register after hook
let after_adapter = ModelHookAdapter::after(hook);
registry.register(after_adapter).await?;
```

## Tool Hook Example

Create a hook that validates tool arguments:

```rust
use async_trait::async_trait;
use radium_core::hooks::tool::{ToolHook, ToolHookContext};
use radium_core::hooks::types::{HookPriority, HookResult as HookExecutionResult};
use radium_core::hooks::error::Result;
use tracing::warn;

pub struct ValidationToolHook {
    name: String,
    priority: HookPriority,
}

impl ValidationToolHook {
    pub fn new(name: impl Into<String>, priority: u32) -> Self {
        Self {
            name: name.into(),
            priority: HookPriority::new(priority),
        }
    }
}

#[async_trait]
impl ToolHook for ValidationToolHook {
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> HookPriority {
        self.priority
    }

    async fn before_tool_execution(&self, context: &ToolHookContext) -> Result<HookExecutionResult> {
        // Validate file operations
        if context.tool_name == "read_file" || context.tool_name == "write_file" {
            if let Some(path) = context.arguments.get("path").and_then(|v| v.as_str()) {
                // Check for path traversal
                if path.contains("..") {
                    warn!(hook = %self.name, path = %path, "Invalid path detected");
                    return Ok(HookExecutionResult::stop("Invalid path: path traversal detected"));
                }
            }
        }

        Ok(HookExecutionResult::success())
    }

    async fn after_tool_execution(&self, context: &ToolHookContext) -> Result<HookExecutionResult> {
        // Log tool execution
        tracing::info!(
            hook = %self.name,
            tool = %context.tool_name,
            "Tool execution completed"
        );
        Ok(HookExecutionResult::success())
    }

    async fn tool_selection(&self, context: &ToolHookContext) -> Result<HookExecutionResult> {
        // Allow all tools
        Ok(HookExecutionResult::success())
    }
}
```

## Error Hook Example

Create a hook that transforms error messages:

```rust
use async_trait::async_trait;
use radium_core::hooks::error_hooks::{ErrorHook, ErrorHookContext, ErrorHookType};
use radium_core::hooks::types::{HookPriority, HookResult as HookExecutionResult};
use radium_core::hooks::error::Result;
use serde_json::json;

pub struct ErrorTransformationHook {
    name: String,
    priority: HookPriority,
}

impl ErrorTransformationHook {
    pub fn new(name: impl Into<String>, priority: u32) -> Self {
        Self {
            name: name.into(),
            priority: HookPriority::new(priority),
        }
    }
}

#[async_trait]
impl ErrorHook for ErrorTransformationHook {
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> HookPriority {
        self.priority
    }

    async fn error_transformation(
        &self,
        context: &ErrorHookContext,
    ) -> Result<HookExecutionResult> {
        // Transform technical errors to user-friendly messages
        let transformed = match context.error_type.as_str() {
            "NetworkError" => "Connection failed. Please check your internet connection.",
            "TimeoutError" => "Request timed out. Please try again.",
            "ValidationError" => "Invalid input. Please check your request.",
            _ => &context.error_message,
        };

        Ok(HookExecutionResult::with_data(json!({
            "transformed_error": transformed
        })))
    }

    // Implement other error hook methods as needed
    async fn error_interception(&self, _context: &ErrorHookContext) -> Result<HookExecutionResult> {
        Ok(HookExecutionResult::success())
    }

    async fn error_recovery(&self, _context: &ErrorHookContext) -> Result<HookExecutionResult> {
        Ok(HookExecutionResult::success())
    }

    async fn error_logging(&self, context: &ErrorHookContext) -> Result<HookExecutionResult> {
        tracing::error!(
            hook = %self.name,
            error_type = %context.error_type,
            error_message = %context.error_message,
            "Error logged"
        );
        Ok(HookExecutionResult::success())
    }
}
```

## Telemetry Hook Example

Create a hook that collects telemetry:

```rust
use async_trait::async_trait;
use radium_core::hooks::registry::{Hook, HookType};
use radium_core::hooks::types::{HookContext, HookPriority, HookResult as HookExecutionResult};
use radium_core::hooks::error::Result;
use std::sync::Arc;
use std::collections::HashMap;

pub struct TelemetryCollectionHook {
    name: String,
    priority: HookPriority,
    metrics: Arc<tokio::sync::RwLock<HashMap<String, u64>>>,
}

impl TelemetryCollectionHook {
    pub fn new(name: impl Into<String>, priority: u32) -> Self {
        Self {
            name: name.into(),
            priority: HookPriority::new(priority),
            metrics: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl Hook for TelemetryCollectionHook {
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> HookPriority {
        self.priority
    }

    fn hook_type(&self) -> HookType {
        HookType::TelemetryCollection
    }

    async fn execute(&self, context: &HookContext) -> Result<HookExecutionResult> {
        // Extract telemetry data
        if let Some(tokens) = context.data.get("total_tokens").and_then(|v| v.as_u64()) {
            let mut metrics = self.metrics.write().await;
            *metrics.entry("total_tokens".to_string()).or_insert(0) += tokens;
        }

        if let Some(cost) = context.data.get("estimated_cost").and_then(|v| v.as_f64()) {
            let mut metrics = self.metrics.write().await;
            let cost_key = "total_cost".to_string();
            let current = metrics.get(&cost_key).copied().unwrap_or(0) as f64;
            metrics.insert(cost_key, (current + cost) as u64);
        }

        Ok(HookExecutionResult::success())
    }
}
```

## Configuration

Create a configuration file for your hook:

`.radium/hooks.toml`:

```toml
[[hooks]]
name = "my-model-hook"
type = "before_model"
priority = 100
enabled = true

[hooks.config]
log_level = "info"
validate_input = true
```

Load configuration in your code:

```rust
use radium_core::hooks::loader::HookLoader;
use radium_core::hooks::config::HookConfig;

// Load configuration
let config_path = workspace_root.join(".radium").join("hooks.toml");
if config_path.exists() {
    let config = HookConfig::from_file(&config_path)?;
    // Apply configuration to your hooks
}
```

## Testing

Write tests for your hooks:

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
    async fn test_model_hook_empty_input() {
        let hook = MyModelHook::new("test-hook", 100);
        let context = ModelHookContext::before(
            "".to_string(),
            "test-model".to_string(),
        );

        let result = hook.before_model_call(&context).await.unwrap();
        assert!(!result.should_continue);
    }
}
```

## Best Practices

1. **Error Handling**: Always handle errors gracefully. Don't let hook failures crash the system.

2. **Performance**: Keep hook execution fast. Long-running operations should be async.

3. **Thread Safety**: Hooks must be `Send + Sync`. Use `Arc` and `RwLock` for shared state.

4. **Logging**: Use the `tracing` crate for logging. Don't use `println!`.

5. **Priority**: Choose appropriate priorities. Critical hooks should have high priority (200+).

6. **Idempotency**: Hooks should be idempotent when possible.

7. **Configuration**: Use `HookConfig` for configurable behavior.

8. **Testing**: Write comprehensive tests for your hooks.

## Next Steps

- See [Hook Types Reference](hook-types.md) for detailed information about each hook type
- Check out [Examples](examples.md) for more patterns
- Review [API Reference](api-reference.md) for complete API documentation
- Read [Architecture](architecture.md) to understand the system design

