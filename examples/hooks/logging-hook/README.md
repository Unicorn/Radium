# Logging Hook Example

This is an example hook implementation that demonstrates how to create hooks for logging model calls in Radium.

## Overview

The logging hook logs model calls at two points:
- **Before Model Call**: Logs the input and model ID before the model is called
- **After Model Call**: Logs the input, output, and model ID after the model returns

## Features

- Configurable log level (info, debug, warn, error)
- Configurable log format (text or JSON)
- Timestamp tracking
- Input/output length tracking

## Usage

### Building

```bash
cd examples/hooks/logging-hook
cargo build --release
```

### Integration

To use this hook in your Radium workspace:

1. Copy the compiled library to your extension's hooks directory
2. Register the hook in your extension manifest or workspace config
3. The hook will automatically log model calls when registered

### Example Configuration

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

## Implementation Details

The hook implements the `ModelHook` trait and uses `ModelHookAdapter` to convert it to the base `Hook` trait. This allows it to work with both before and after model call hook points.

## Customization

You can customize the hook by:
- Adjusting the priority (higher = executes first)
- Changing the log level
- Switching between text and JSON log formats
- Adding additional metadata to logs

