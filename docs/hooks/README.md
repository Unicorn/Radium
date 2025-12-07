# Hooks System

The Radium hooks system provides a comprehensive framework for intercepting and customizing execution flow at various points in agent execution. This system enables advanced behavior customization, logging, error handling, and telemetry collection.

## Quick Start

1. **Create a hook configuration** in `.radium/hooks.toml`:
```toml
[[hooks]]
name = "my-logging-hook"
type = "before_model"
priority = 100
enabled = true
```

2. **Register your hook programmatically**:
```rust
use radium_core::hooks::registry::HookRegistry;
use std::sync::Arc;

let registry = Arc::new(HookRegistry::new());
// Register your hook implementation
```

3. **Use CLI commands** to manage hooks:
```bash
rad hooks list
rad hooks info my-logging-hook
rad hooks enable my-logging-hook
```

## Documentation

- **[Getting Started](getting-started.md)** - Quick start guide and basic usage
- **[Architecture](architecture.md)** - System architecture and design patterns
- **[Creating Hooks](creating-hooks.md)** - Guide for creating custom hooks
- **[Hook Types](hook-types.md)** - Detailed reference for all 13 hook types
- **[Examples](examples.md)** - Practical examples and patterns
- **[API Reference](api-reference.md)** - Complete API documentation
- **[Development Guide](hook-development.md)** - Advanced development topics

## Features

- **13 Hook Types**: Model, tool, error, and telemetry hooks
- **Priority-Based Execution**: Hooks execute in priority order
- **Thread-Safe**: Full async/await support with `Arc<RwLock>`
- **Enable/Disable**: Dynamic hook management
- **Configuration-Based**: TOML configuration for hook settings
- **Extension Support**: Distribute hooks via extensions

## Hook Types Overview

### Model Hooks
- `before_model` - Execute before model calls
- `after_model` - Execute after model calls

### Tool Hooks
- `before_tool` - Execute before tool execution
- `after_tool` - Execute after tool execution
- `tool_selection` - Execute during tool selection

### Error Hooks
- `error_interception` - Intercept errors before propagation
- `error_transformation` - Transform error messages
- `error_recovery` - Attempt error recovery
- `error_logging` - Log errors with custom formatting

### Telemetry Hooks
- `telemetry_collection` - Collect and aggregate telemetry
- `custom_logging` - Custom logging hooks
- `metrics_aggregation` - Aggregate metrics
- `performance_monitoring` - Monitor performance

## Example Implementations

See example hooks in `examples/hooks/`:
- `logging-hook/` - Logs model calls with timestamps
- `metrics-hook/` - Aggregates telemetry data

## Next Steps

- Read the [Getting Started Guide](getting-started.md) for basic usage
- Check out [Creating Hooks](creating-hooks.md) to build your first hook
- Review [Hook Types](hook-types.md) to understand all available hook types
- Explore [Examples](examples.md) for practical patterns

