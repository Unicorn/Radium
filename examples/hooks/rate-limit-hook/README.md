# Rate Limit Hook Example

This is an example hook implementation that demonstrates rate limiting enforcement for model calls in Radium.

## Overview

The rate limit hook enforces rate limits on model calls to prevent excessive usage. It demonstrates:

- Per-model rate limiting
- Time window tracking
- Automatic window reset
- Configurable limits

## Features

- Enforces rate limits per model
- Sliding window rate limiting
- Automatic cleanup of expired entries
- Configurable max calls and window duration
- High priority (200) to run before other hooks

## Usage

### Building

```bash
cd examples/hooks/rate-limit-hook
cargo build --release
```

### Integration

To use this hook in your Radium workspace:

1. Register the hook programmatically
2. Configure it in `.radium/hooks.toml`
3. The hook will automatically enforce rate limits

### Example Configuration

```toml
[[hooks]]
name = "rate-limit-hook"
type = "before_model"
priority = 200
enabled = true

[hooks.config]
max_calls = 10
window_seconds = 60
```

## Implementation Details

The hook implements the `ModelHook` trait and enforces rate limits in the `before_model_call` method. It uses a sliding window approach where calls are tracked per model within a time window.

## Default Configuration

- Max calls: 10
- Window duration: 60 seconds
- Priority: 200 (high priority to run before other hooks)

## Rate Limiting Behavior

- Each model has its own rate limit counter
- Counters reset when the time window expires
- Calls exceeding the limit are blocked with an error message
- Expired entries are automatically cleaned up

## Customization

You can customize the hook by:

- Adjusting max calls per window
- Modifying window duration
- Adding per-user rate limiting
- Integrating with external rate limit services
- Adding different limits for different models

