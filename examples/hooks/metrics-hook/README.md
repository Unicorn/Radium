# Metrics Hook Example

This is an example hook implementation that demonstrates how to create hooks for aggregating telemetry metrics in Radium.

## Overview

The metrics hook aggregates telemetry data including:
- Total token usage (input, output, and combined)
- Total cost accumulation
- Call counts
- Per-model statistics

## Features

- Real-time metrics aggregation
- Per-model tracking
- Periodic summary logging
- Thread-safe metrics storage

## Usage

### Building

```bash
cd examples/hooks/metrics-hook
cargo build --release
```

### Integration

To use this hook in your Radium workspace:

1. Copy the compiled library to your extension's hooks directory
2. Register the hook in your extension manifest or workspace config
3. The hook will automatically aggregate telemetry when registered

### Example Configuration

```toml
[[hooks]]
name = "metrics-hook"
type = "telemetry_collection"
priority = 100
enabled = true
```

## Implementation Details

The hook implements the `Hook` trait directly for the `TelemetryCollection` hook type. It maintains internal state using `Arc<RwLock<>>` for thread-safe metrics aggregation.

## Metrics Tracked

- **Total Input Tokens**: Sum of all input tokens across all calls
- **Total Output Tokens**: Sum of all output tokens across all calls
- **Total Tokens**: Combined token count
- **Total Cost**: Accumulated cost from all model calls
- **Call Count**: Number of telemetry events processed
- **Per-Model Metrics**: Separate tracking for each model/provider combination

## Customization

You can customize the hook by:
- Adjusting the priority (higher = executes first)
- Changing the summary logging frequency
- Adding additional metrics to track
- Implementing custom aggregation logic

