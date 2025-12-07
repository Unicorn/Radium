# Hooks System

The Radium Hooks System provides a unified, extensible framework for intercepting and customizing behavior at various points in the execution flow. This enables powerful customization capabilities including logging, telemetry, error handling, and workflow control.

## Overview

The hooks system allows you to:

- **Intercept model calls**: Modify inputs/outputs, add logging, implement rate limiting
- **Monitor tool execution**: Track tool usage, validate arguments, transform results
- **Handle errors**: Implement custom error recovery, logging, and notification strategies
- **Collect telemetry**: Gather metrics, performance data, and usage statistics
- **Control workflows**: Integrate with workflow behaviors for dynamic execution control

## Key Features

- **Priority-based execution**: Hooks execute in priority order (higher priority first)
- **Type-safe context**: Type-safe data passing between hooks and execution context
- **Async support**: Full async/await support for I/O operations
- **Thread-safe**: Safe for concurrent execution across multiple threads
- **Configuration-driven**: Configure hooks via TOML files
- **Extensible**: Easy to create custom hooks for specific use cases
- **Extension support**: Hooks can be packaged in extensions for distribution

## Quick Start

1. **Create a hook configuration file** (`.radium/hooks.toml`):

```toml
[[hooks]]
name = "request-logger"
type = "before_model"
priority = 100
enabled = true
```

2. **Implement a hook**:

```rust
use radium_core::hooks::model::{ModelHook, ModelHookContext};
use radium_core::hooks::types::{HookPriority, HookResult};
use async_trait::async_trait;

struct RequestLogger;

#[async_trait]
impl ModelHook for RequestLogger {
    fn name(&self) -> &str { "request-logger" }
    fn priority(&self) -> HookPriority { HookPriority::new(100) }
    
    async fn before_model_call(&self, context: &ModelHookContext) -> Result<HookResult> {
        println!("Model call: {}", context.input);
        Ok(HookResult::success())
    }
}
```

3. **Register the hook**:

```rust
use radium_core::hooks::registry::HookRegistry;
use std::sync::Arc;

let registry = Arc::new(HookRegistry::new());
registry.register(Arc::new(RequestLogger)).await?;
```

## Documentation

- **[Getting Started](getting-started.md)** - Step-by-step guide to creating your first hook
- **[API Reference](api-reference.md)** - Complete API documentation
- **[Hook Types](hook-types.md)** - Reference for all hook types
- **[Configuration](configuration.md)** - Configuration file format and options
- **[Examples](examples/)** - Example implementations
  - [Logging Hook](examples/logging-hook.md)
  - [Telemetry Hook](examples/telemetry-hook.md)
  - [Error Recovery Hook](examples/error-recovery-hook.md)
  - [Extension Hook Example](examples/extension-hook-example.md)
- **[Migration Guide](migration-guide.md)** - Migrating from existing systems

## CLI Commands

Manage hooks via the CLI:

```bash
# List all hooks
rad hooks list

# List hooks by type
rad hooks list --type before_model

# Get hook information
rad hooks info my-hook

# Enable/disable hooks
rad hooks enable my-hook
rad hooks disable my-hook
```

## Extension Integration

Hooks can be packaged in extensions for easy distribution:

1. Create a `hooks/` directory in your extension
2. Add hook configuration files (`.toml`)
3. Declare hooks in `radium-extension.json`:

```json
{
  "components": {
    "hooks": ["hooks/*.toml"]
  }
}
```

See [Extension Hook Example](examples/extension-hook-example.md) for details.

## Architecture

The hooks system consists of:

- **HookRegistry**: Central registry for managing and executing hooks
- **Hook Trait**: Base trait that all hooks implement
- **Hook Types**: Enumeration of all supported hook types
- **Hook Context**: Type-safe data passed to hooks during execution
- **Hook Result**: Result type returned by hooks
- **Hook Loader**: Discovers and loads hooks from extensions and workspace

## Integration Points

Hooks are integrated into:

- **AgentExecutor**: Model call hooks (before/after)
- **PolicyEngine**: Tool execution hooks (before/after)
- **MonitoringService**: Telemetry collection hooks
- **Error Handling**: Error interception, transformation, and recovery hooks

## Best Practices

1. **Use descriptive names**: Make hook names clear and unique
2. **Set appropriate priorities**: Higher priority hooks execute first
3. **Handle errors gracefully**: Don't let hook failures break execution
4. **Document your hooks**: Include clear documentation for custom hooks
5. **Test thoroughly**: Verify hooks work correctly in all scenarios
6. **Use configuration**: Prefer TOML configuration over hardcoded values

## See Also

- [Extension System Guide](../guides/extension-system.md)
- [Workflow System](../features/)
