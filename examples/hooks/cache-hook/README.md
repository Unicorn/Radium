# Cache Hook Example

This is an example hook implementation that demonstrates response caching for model calls in Radium.

## Overview

The cache hook caches model responses to reduce costs and improve performance. It demonstrates:

- Response caching
- Cache key generation
- TTL (time-to-live) management
- Cache hit/miss handling

## Features

- Caches model responses in memory
- Configurable TTL (time-to-live)
- Automatic cache cleanup
- Cache hit/miss logging
- Low priority (50) to run after other hooks

## Usage

### Building

```bash
cd examples/hooks/cache-hook
cargo build --release
```

### Integration

To use this hook in your Radium workspace:

1. Register the hook programmatically
2. Configure it in `.radium/hooks.toml`
3. The hook will automatically cache responses

### Example Configuration

```toml
[[hooks]]
name = "cache-hook"
type = "before_model"
priority = 50
enabled = true

[hooks.config]
ttl_seconds = 3600
```

## Implementation Details

The hook implements the `ModelHook` trait and handles both `before_model_call` (cache lookup) and `after_model_call` (cache storage). It uses a simple in-memory HashMap for caching.

## Cache Behavior

- Cache key: `{model_id}:{input}`
- TTL: Configurable (default: 3600 seconds / 1 hour)
- Cache hits return cached response immediately
- Cache misses allow normal execution
- Expired entries are automatically cleaned up

## Default Configuration

- TTL: 3600 seconds (1 hour)
- Priority: 50 (low priority to run after other hooks)

## Limitations

This is an example implementation with limitations:

- In-memory cache (lost on restart)
- No cache size limits
- No cache eviction policy
- Single process only

For production use, consider:
- Persistent cache (Redis, etc.)
- Cache size limits
- LRU eviction policy
- Distributed caching

## Customization

You can customize the hook by:

- Adjusting TTL duration
- Modifying cache key generation
- Adding cache size limits
- Integrating with external cache systems
- Adding cache statistics

