---
id: "context-caching"
title: "Context Caching"
sidebar_label: "Context Caching"
---

# Context Caching

## Overview

Context caching reduces token costs by 50%+ for repeated prompts by caching processed tokens at the provider level. This feature leverages native caching capabilities from Claude, OpenAI, and Gemini APIs to cache large, reusable prompt contexts.

## Enabling Context Caching

### Basic Configuration

```rust
use radium_models::{ModelConfig, ModelFactory, ModelType};
use std::time::Duration;

let config = ModelConfig::new(ModelType::Claude, "claude-3-sonnet".to_string())
    .with_context_caching(true)
    .with_cache_ttl(Duration::from_secs(300));

let model = ModelFactory::create(config)?;
```

### Provider-Specific Configuration

#### Claude

Claude uses `cache_control` blocks in messages to enable prompt caching.

- Minimum 1024 tokens for cache creation
- 5-minute default TTL
- Configure breakpoints to mark which messages should be cached:

```rust
let config = ModelConfig::new(ModelType::Claude, "claude-3-sonnet".to_string())
    .with_context_caching(true)
    .with_cache_breakpoints(vec![0, 2]); // Cache messages at indices 0 and 2
```

#### OpenAI

OpenAI handles caching automatically for GPT-4 and newer models. No explicit configuration needed beyond enabling caching:

```rust
let config = ModelConfig::new(ModelType::OpenAI, "gpt-4".to_string())
    .with_context_caching(true);
```

#### Gemini

Gemini uses the cachedContent API with explicit cache identifiers:

```rust
let config = ModelConfig::new(ModelType::Gemini, "gemini-1.5-pro".to_string())
    .with_context_caching(true)
    .with_cache_identifier("cachedContents/my-cache-id".to_string());
```

## Monitoring Cache Performance

Cache metrics are available in `ModelResponse`:

```rust
if let Some(usage) = response.usage {
    if let Some(cache_usage) = usage.cache_usage {
        println!("Cache read tokens: {}", cache_usage.cache_read_tokens);
        println!("Regular tokens: {}", cache_usage.regular_tokens);
        println!("Cache creation tokens: {}", cache_usage.cache_creation_tokens);
        
        // Calculate cost savings
        let total_cached = cache_usage.cache_read_tokens + cache_usage.cache_creation_tokens;
        let savings_percentage = if total_cached > 0 {
            (cache_usage.cache_read_tokens as f64 / total_cached as f64) * 100.0
        } else {
            0.0
        };
        println!("Cache hit rate: {:.1}%", savings_percentage);
    }
}
```

## Provider Comparison

| Provider | Caching Method | TTL | Minimum Tokens | Explicit Control |
|----------|---------------|-----|----------------|------------------|
| Claude | cache_control blocks | 5 minutes | 1024 | Yes (breakpoints) |
| OpenAI | Automatic | Variable | None | No |
| Gemini | cachedContent API | Hours | None | Yes (cache identifier) |

## Best Practices

1. **Use caching for large, stable contexts**: System prompts, reference documents, and few-shot examples are ideal candidates.

2. **Monitor cache hit rates**: Track `cache_read_tokens` to ensure caching is effective.

3. **Set appropriate TTLs**: Balance cache freshness with cost savings.

4. **Use breakpoints wisely (Claude)**: Mark stable context boundaries, not dynamic conversation content.

## Troubleshooting

### Cache not being used

- Verify `enable_context_caching` is set to `true`
- Check that your context meets minimum token requirements (Claude: 1024 tokens)
- Ensure you're using a supported model (OpenAI: GPT-4+)

### Low cache hit rate

- Review cache breakpoint placement (Claude)
- Verify cache identifier is being reused (Gemini)
- Check that context is actually being repeated across requests

