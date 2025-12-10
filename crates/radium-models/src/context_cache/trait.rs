//! Context cache trait for provider-agnostic context caching.

use async_trait::async_trait;
use std::time::Duration;

use crate::context_cache::types::{CacheError, CacheHandle, CachedContext};

/// Trait for provider-specific context caching implementations.
///
/// This trait provides a unified interface for caching context/prompts
/// across different providers (Claude, OpenAI, Gemini). Each provider
/// has different caching mechanisms, but this trait abstracts them.
///
/// # Provider Differences
///
/// - **Claude**: Uses cache_control blocks in messages (implicit caching)
/// - **OpenAI**: Automatic caching (no explicit cache management)
/// - **Gemini**: Explicit cachedContent API with cache names
///
/// # Example
///
/// ```rust,no_run
/// use radium_models::context_cache::{ContextCache, CacheHandle};
/// use std::time::Duration;
///
/// # async fn example(cache: impl ContextCache) -> Result<(), Box<dyn std::error::Error>> {
/// // Create a cache
/// let handle = cache.create_cache("Large system prompt...", Duration::from_secs(300)).await?;
///
/// // Retrieve cache metadata
/// if let Some(context) = cache.get_cache(&handle).await? {
///     println!("Cache has {} tokens", context.token_count);
/// }
///
/// // Refresh cache TTL
/// cache.refresh_cache(&handle).await?;
///
/// // Delete cache
/// cache.delete_cache(&handle).await?;
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait ContextCache: Send + Sync {
    /// Create a new cached context.
    ///
    /// This method creates a cache for the given content with the specified
    /// time-to-live (TTL). The exact behavior depends on the provider:
    ///
    /// - **Claude**: Prepares cache_control metadata (actual caching happens during request)
    /// - **OpenAI**: No-op (caching is automatic, no explicit creation)
    /// - **Gemini**: Creates a cachedContent resource via API
    ///
    /// # Arguments
    /// * `content` - The content to cache (may be a prompt, system message, etc.)
    /// * `ttl` - Time to live for the cache
    ///
    /// # Returns
    /// A `CacheHandle` that can be used to reference this cache in future operations.
    ///
    /// # Errors
    /// Returns `CacheError` if cache creation fails (e.g., API error, invalid configuration).
    async fn create_cache(
        &self,
        content: &str,
        ttl: Duration,
    ) -> Result<CacheHandle, CacheError>;

    /// Get metadata for a cached context.
    ///
    /// Retrieves metadata about a cached context, including creation time,
    /// expiration time, and token count.
    ///
    /// # Arguments
    /// * `handle` - The cache handle to look up
    ///
    /// # Returns
    /// `Some(CachedContext)` if the cache exists and is valid, `None` if not found.
    ///
    /// # Errors
    /// Returns `CacheError` if there's an error retrieving the cache (e.g., network error).
    ///
    /// # Note
    /// For OpenAI, this will return `ProviderNotSupported` since OpenAI
    /// doesn't expose cache metadata.
    async fn get_cache(
        &self,
        handle: &CacheHandle,
    ) -> Result<Option<CachedContext>, CacheError>;

    /// Refresh the TTL of an existing cache.
    ///
    /// Extends the expiration time of a cached context without re-creating it.
    /// This is useful for keeping frequently-used caches alive.
    ///
    /// # Arguments
    /// * `handle` - The cache handle to refresh
    ///
    /// # Returns
    /// `Ok(())` if the refresh was successful.
    ///
    /// # Errors
    /// Returns `CacheError` if the cache doesn't exist, has expired, or refresh fails.
    ///
    /// # Note
    /// For OpenAI, this is a no-op since OpenAI manages cache lifecycle automatically.
    async fn refresh_cache(&self, handle: &CacheHandle) -> Result<(), CacheError>;

    /// Delete a cached context.
    ///
    /// Explicitly removes a cached context. This is useful for freeing up
    /// resources or invalidating stale caches.
    ///
    /// # Arguments
    /// * `handle` - The cache handle to delete
    ///
    /// # Returns
    /// `Ok(())` if the deletion was successful (or if the cache didn't exist).
    ///
    /// # Errors
    /// Returns `CacheError` if deletion fails (e.g., network error).
    ///
    /// # Note
    /// For OpenAI, this is a no-op since OpenAI manages cache lifecycle automatically.
    async fn delete_cache(&self, handle: &CacheHandle) -> Result<(), CacheError>;
}

