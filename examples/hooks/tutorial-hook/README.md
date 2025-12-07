# Tutorial Hook: Request Logger

This is the tutorial example hook that demonstrates how to create a custom hook in Radium.

## Overview

The Request Logger Hook logs all model calls with timestamps and metadata. It's designed as a learning example to help you understand:

- How to implement the `ModelHook` trait
- How to create hook adapters
- How to register hooks
- How to test hooks

## Building

```bash
cd examples/hooks/tutorial-hook
cargo build
```

## Usage

### Registering the Hook

```rust
use tutorial_hook::register_tutorial_hooks;
use radium_core::hooks::registry::HookRegistry;
use std::sync::Arc;

let registry = Arc::new(HookRegistry::new());
register_tutorial_hooks(&registry).await?;
```

### Configuration

Create `.radium/hooks.toml`:

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

## Testing

Run the tests:

```bash
cargo test
```

## What This Hook Does

- **Before Model Call**: Logs the model ID and input length
- **After Model Call**: Logs the model ID, input length, and response length

## Learning Points

1. **Trait Implementation**: Shows how to implement `ModelHook`
2. **Adapters**: Demonstrates using `ModelHookAdapter` to convert to base `Hook` trait
3. **Registration**: Shows how to register hooks with the registry
4. **Testing**: Includes unit tests for the hook

## Next Steps

After understanding this example:

1. Try modifying the hook to log additional information
2. Add input/output previews (first 100 characters)
3. Experiment with different priorities
4. Create your own hook for a specific use case

See the [Tutorial Guide](../../../docs/hooks/tutorial.md) for step-by-step instructions.

