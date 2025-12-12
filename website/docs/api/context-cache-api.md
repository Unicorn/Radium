---
id: "context-cache-api"
title: "Context Cache API Reference"
sidebar_label: "Context Cache API Ref"
---

# Context Cache API Reference

## ContextCache Trait

The `ContextCache` trait provides a provider-agnostic interface for context caching.

### Methods

#### create_cache

```rust
async fn create_cache(&self, content: &str, ttl: Duration) -> Result<CacheHandle, CacheError>
```

Creates a new cached content resource.

**Parameters:**
- `content`: The content to cache (prompt, system message, etc.)
- `ttl`: Time-to-live duration for the cache

**Returns:**
- `Ok(CacheHandle)`: Cache handle for future operations
- `Err(CacheError)`: Error if cache creation fails

**Note:** For Claude and OpenAI, this is a no-op or minimal operation since caching is handled automatically.

#### get_cache

```rust
async fn get_cache(&self, handle: &CacheHandle) -> Result<Option<CachedContext>, CacheError>
```

Retrieves cache metadata by handle.

**Parameters:**
- `handle`: The cache handle to look up

**Returns:**
- `Ok(Some(CachedContext))`: Cache metadata if found
- `Ok(None)`: Cache not found
- `Err(CacheError)`: Error retrieving cache

**Note:** For Claude and OpenAI, this returns `ProviderNotSupported` since they don't expose cache metadata.

#### refresh_cache

```rust
async fn refresh_cache(&self, handle: &CacheHandle) -> Result<(), CacheError>
```

Extends cache TTL before expiration.

**Parameters:**
- `handle`: The cache handle to refresh

**Returns:**
- `Ok(())`: Refresh successful
- `Err(CacheError)`: Error refreshing cache

#### delete_cache

```rust
async fn delete_cache(&self, handle: &CacheHandle) -> Result<(), CacheError>
```

Explicitly removes cached content.

**Parameters:**
- `handle`: The cache handle to delete

**Returns:**
- `Ok(())`: Deletion successful
- `Err(CacheError)`: Error deleting cache

## Types

### CacheHandle

Provider-specific cache identifier:

```rust
pub enum CacheHandle {
    Claude { cache_control: serde_json::Value },
    OpenAI,
    Gemini { cache_name: String },
}
```

### CachedContext

Cache metadata including creation time, expiration, and token count:

```rust
pub struct CachedContext {
    pub handle: CacheHandle,
    pub created_at: SystemTime,
    pub expires_at: SystemTime,
    pub token_count: u32,
}
```

### CacheError

Error types for cache operations:

```rust
pub enum CacheError {
    CacheCreationFailed { provider: String, reason: String },
    CacheExpired { handle: CacheHandle },
    CacheNotFound { handle: CacheHandle },
    InvalidCacheConfiguration { reason: String },
    ProviderNotSupported { provider: String },
    NetworkError { message: String },
    ParseError { message: String },
}
```

## Provider Implementations

### ClaudeContextCache

Minimal implementation - caching is handled via `cache_control` blocks in requests.

### OpenAIContextCache

Minimal implementation - caching is automatic, no explicit control.

### GeminiContextCache

Full implementation with cachedContent API support:

```rust
let cache = GeminiContextCache::new(api_key, Some(base_url));
let handle = cache.create_cache("Large context...", Duration::from_secs(3600)).await?;
```

