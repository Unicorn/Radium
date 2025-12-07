# Hooks System - Getting Started

The Radium hooks system enables you to intercept and customize behavior at various points in the execution flow. This guide will help you get started with using hooks.

## Overview

Hooks allow you to:
- Add custom logging and monitoring
- Modify execution flow dynamically
- Inject telemetry or metrics collection
- Handle errors with custom logic
- Intercept tool execution

## Installation

Hooks are installed via extensions. To use a hook:

1. Install an extension that provides hooks
2. Configure hooks in your workspace (`.radium/hooks.toml`) or extension manifest
3. Hooks are automatically loaded and registered at startup

## Basic Usage

### Using Example Hooks

Radium includes example hook implementations:

- **Logging Hook**: Logs model calls with timestamps and metadata
- **Metrics Hook**: Aggregates telemetry data including token usage and costs

### Configuration

Hooks are configured via TOML files. Create a `.radium/hooks.toml` file in your workspace:

```toml
[[hooks]]
name = "logging-hook-before"
type = "before_model"
priority = 100
enabled = true

[hooks.config]
log_level = "info"
log_format = "json"
```

### CLI Commands

List all registered hooks:

```bash
rad hooks list
```

Show details for a specific hook:

```bash
rad hooks info logging-hook-before
```

Enable or disable a hook:

```bash
rad hooks enable logging-hook-before
rad hooks disable logging-hook-before
```

## Hook Types

Radium supports the following hook types:

### Model Hooks
- `before_model` - Executed before model calls
- `after_model` - Executed after model calls

### Tool Hooks
- `before_tool` - Executed before tool execution
- `after_tool` - Executed after tool execution
- `tool_selection` - Executed during tool selection

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

## Common Use Cases

### Logging Model Calls

Use a logging hook to track all model calls:

```toml
[[hooks]]
name = "model-logger"
type = "before_model"
priority = 100
enabled = true
```

### Tracking Costs

Use a metrics hook to track token usage and costs:

```toml
[[hooks]]
name = "cost-tracker"
type = "telemetry_collection"
priority = 100
enabled = true
```

### Error Handling

Use error hooks to implement custom error handling:

```toml
[[hooks]]
name = "error-handler"
type = "error_interception"
priority = 200
enabled = true

[hooks.config]
notify_on_error = true
retry_count = 3
```

## Hook Priority

Hooks execute in priority order (higher priority = executes first). Default priority is 100.

- **High priority (200+)**: Critical hooks that must run first
- **Medium priority (100-199)**: Standard hooks
- **Low priority (<100)**: Optional hooks

## Next Steps

- See [Hook Development Guide](hook-development.md) to create your own hooks
- See [API Reference](api-reference.md) for complete API documentation
- Check out example implementations in `examples/hooks/`

