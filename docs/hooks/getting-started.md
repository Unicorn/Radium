# Getting Started with Hooks

This guide will walk you through creating your first hook and integrating it into your Radium workflow.

## Prerequisites

- Basic understanding of Rust
- Radium workspace set up
- Familiarity with async/await in Rust

## Step 1: Create a Simple Hook

Let's create a simple logging hook that logs all model calls:

```rust
use radium_core::hooks::model::{ModelHook, ModelHookContext};
use radium_core::hooks::types::{HookPriority, HookResult};
use radium_core::hooks::error::Result;
use async_trait::async_trait;
use std::sync::Arc;

struct ModelCallLogger;

#[async_trait]
impl ModelHook for ModelCallLogger {
    fn name(&self) -> &str {
        "model-call-logger"
    }

    fn priority(&self) -> HookPriority {
        HookPriority::new(100)
    }

    async fn before_model_call(
        &self,
        context: &ModelHookContext,
    ) -> Result<HookResult> {
        println!("[LOG] Model call to {}: {}", context.model_id, context.input);
        Ok(HookResult::success())
    }

    async fn after_model_call(
        &self,
        context: &ModelHookContext,
    ) -> Result<HookResult> {
        println!("[LOG] Model response: {}", context.response);
        Ok(HookResult::success())
    }
}
```

## Step 2: Register the Hook

Register your hook with the hook registry:

```rust
use radium_core::hooks::registry::HookRegistry;

let registry = Arc::new(HookRegistry::new());
registry.register(Arc::new(ModelCallLogger)).await?;
```

## Step 3: Configure via TOML (Optional)

Create `.radium/hooks.toml`:

```toml
[[hooks]]
name = "model-call-logger"
type = "before_model_call"
priority = 100
enabled = true
```

## Step 4: Use in Your Application

Integrate the hook registry into your orchestrator:

```rust
use radium_core::hooks::integration::OrchestratorHooks;

let hooks = OrchestratorHooks::new(registry);

// Before model call
let (modified_input, _) = hooks.before_model_call(input, model_id).await?;

// After model call
let modified_response = hooks.after_model_call(input, model_id, response).await?;
```

## Next Steps

- Explore [Hook Types Reference](./hook-types.md) for all available hook types
- Check out [Examples](./examples/) for more complex use cases
- Read [Configuration Guide](./configuration.md) for advanced configuration

