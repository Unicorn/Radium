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

## Quick Start

1. **Create a hook configuration file** (`.radium/hooks.toml`):

```toml
[[hooks]]
name = "request-logger"
type = "before_model_call"
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
let registry = Arc::new(HookRegistry::new());
registry.register(Arc::new(RequestLogger)).await?;
```

## Documentation

- [Getting Started Guide](./getting-started.md) - Step-by-step tutorial
- [Hook Types Reference](./hook-types.md) - Complete reference for all hook types
- [Configuration Guide](./configuration.md) - Configuration file format and options
- [API Reference](./api-reference.md) - Complete API documentation
- [Examples](./examples/) - Practical examples for common use cases
- [Migration Guide](./migration-guide.md) - Migrating from existing systems

## Hook Types

- **Model Hooks**: Intercept model API calls (before/after)
- **Tool Hooks**: Intercept tool execution (before/after/selection)
- **Error Hooks**: Handle errors and implement recovery strategies
- **Telemetry Hooks**: Collect metrics and performance data
- **Workflow Hooks**: Integrate with workflow behaviors

## Examples

See the [examples directory](./examples/) for:
- [Logging Hook](./examples/logging-hook.md)
- [Telemetry Hook](./examples/telemetry-hook.md)
- [Error Recovery Hook](./examples/error-recovery-hook.md)

## Performance

The hooks system is designed for minimal overhead (<5% impact). See [Performance Documentation](../../crates/radium-core/src/hooks/performance.md) for details.

## Support

For questions, issues, or contributions, please see the main project documentation.

