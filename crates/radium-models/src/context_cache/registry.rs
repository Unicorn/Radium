//! Cache registry for metadata storage and management.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

use crate::context_cache::types::{CacheHandle, CachedContext};

/// Thread-safe registry for storing cache metadata.
///
/// This registry stores metadata about cached contexts across all providers.
/// It provides thread-safe operations for registering, retrieving, and managing
/// cache metadata.
///
/// # Thread Safety
///
/// All operations are thread-safe using `Arc<RwLock<>>` for concurrent access.
#[derive(Debug, Clone)]
pub struct CacheRegistry {
    /// Thread-safe storage for cache metadata.
    caches: Arc<RwLock<HashMap<String, CachedContext>>>,
}

impl CacheRegistry {
    /// Create a new cache registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            caches: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a cached context in the registry.
    ///
    /// # Arguments
    /// * `handle` - The cache handle
    /// * `context` - The cached context metadata
    ///
    /// # Errors
    /// Returns an error if the lock is poisoned.
    pub fn register(
        &self,
        handle: &CacheHandle,
        context: CachedContext,
    ) -> Result<(), String> {
        let key = Self::handle_to_key(handle);
        let mut caches = self.caches.write().map_err(|_| "Cache registry lock poisoned".to_string())?;
        caches.insert(key, context);
        Ok(())
    }

    /// Get cached context metadata by handle.
    ///
    /// # Arguments
    /// * `handle` - The cache handle to look up
    ///
    /// # Returns
    /// `Some(CachedContext)` if found, `None` if not found.
    ///
    /// # Errors
    /// Returns an error if the lock is poisoned.
    pub fn get(&self, handle: &CacheHandle) -> Result<Option<CachedContext>, String> {
        let key = Self::handle_to_key(handle);
        let caches = self.caches.read().map_err(|_| "Cache registry lock poisoned".to_string())?;
        Ok(caches.get(&key).cloned())
    }

    /// Remove a cached context from the registry.
    ///
    /// # Arguments
    /// * `handle` - The cache handle to remove
    ///
    /// # Returns
    /// `Some(CachedContext)` if removed, `None` if not found.
    ///
    /// # Errors
    /// Returns an error if the lock is poisoned.
    pub fn remove(&self, handle: &CacheHandle) -> Result<Option<CachedContext>, String> {
        let key = Self::handle_to_key(handle);
        let mut caches = self.caches.write().map_err(|_| "Cache registry lock poisoned".to_string())?;
        Ok(caches.remove(&key))
    }

    /// List all cached contexts in the registry.
    ///
    /// # Returns
    /// Vector of all cached contexts.
    ///
    /// # Errors
    /// Returns an error if the lock is poisoned.
    pub fn list_all(&self) -> Result<Vec<CachedContext>, String> {
        let caches = self.caches.read().map_err(|_| "Cache registry lock poisoned".to_string())?;
        Ok(caches.values().cloned().collect())
    }

    /// Clean up expired caches from the registry.
    ///
    /// # Returns
    /// Number of expired caches removed.
    ///
    /// # Errors
    /// Returns an error if the lock is poisoned.
    pub fn cleanup_expired(&self) -> Result<usize, String> {
        let now = SystemTime::now();
        let mut caches = self.caches.write().map_err(|_| "Cache registry lock poisoned".to_string())?;
        let initial_count = caches.len();
        caches.retain(|_, context| {
            context.expires_at.duration_since(now).is_ok()
        });
        Ok(initial_count - caches.len())
    }

    /// Generate a unique cache key from a cache handle.
    ///
    /// # Arguments
    /// * `handle` - The cache handle
    ///
    /// # Returns
    /// A string key uniquely identifying the cache.
    fn handle_to_key(handle: &CacheHandle) -> String {
        match handle {
            CacheHandle::Claude { cache_control } => {
                format!("claude:{}", serde_json::to_string(cache_control).unwrap_or_else(|_| "unknown".to_string()))
            }
            CacheHandle::OpenAI => "openai:auto".to_string(),
            CacheHandle::Gemini { cache_name } => {
                format!("gemini:{}", cache_name)
            }
        }
    }

    /// Get the current number of caches in the registry.
    ///
    /// # Returns
    /// Number of cached contexts.
    ///
    /// # Errors
    /// Returns an error if the lock is poisoned.
    pub fn len(&self) -> Result<usize, String> {
        let caches = self.caches.read().map_err(|_| "Cache registry lock poisoned".to_string())?;
        Ok(caches.len())
    }

    /// Check if the registry is empty.
    ///
    /// # Returns
    /// `true` if empty, `false` otherwise.
    ///
    /// # Errors
    /// Returns an error if the lock is poisoned.
    pub fn is_empty(&self) -> Result<bool, String> {
        let caches = self.caches.read().map_err(|_| "Cache registry lock poisoned".to_string())?;
        Ok(caches.is_empty())
    }
}

impl Default for CacheRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_cache_registry_new() {
        let registry = CacheRegistry::new();
        assert!(registry.is_empty().unwrap());
        assert_eq!(registry.len().unwrap(), 0);
    }

    #[test]
    fn test_register_and_get() {
        let registry = CacheRegistry::new();
        let handle = CacheHandle::OpenAI;
        let context = CachedContext::new(handle.clone(), Duration::from_secs(300), 1000);

        registry.register(&handle, context.clone()).unwrap();
        let retrieved = registry.get(&handle).unwrap().unwrap();

        assert_eq!(retrieved.token_count, 1000);
        assert_eq!(retrieved.handle, handle);
    }

    #[test]
    fn test_remove() {
        let registry = CacheRegistry::new();
        let handle = CacheHandle::OpenAI;
        let context = CachedContext::new(handle.clone(), Duration::from_secs(300), 1000);

        registry.register(&handle, context).unwrap();
        assert_eq!(registry.len().unwrap(), 1);

        let removed = registry.remove(&handle).unwrap();
        assert!(removed.is_some());
        assert_eq!(registry.len().unwrap(), 0);
    }

    #[test]
    fn test_cleanup_expired() {
        let registry = CacheRegistry::new();
        let handle1 = CacheHandle::OpenAI;
        let handle2 = CacheHandle::Gemini {
            cache_name: "cachedContents/test".to_string(),
        };

        // Create one expired and one valid cache
        let expired_context = CachedContext::new(handle1.clone(), Duration::from_secs(0), 1000);
        let valid_context = CachedContext::new(handle2.clone(), Duration::from_secs(300), 2000);

        registry.register(&handle1, expired_context).unwrap();
        registry.register(&handle2, valid_context).unwrap();

        // Wait a tiny bit to ensure expiration
        std::thread::sleep(Duration::from_millis(10));

        let removed = registry.cleanup_expired().unwrap();
        assert_eq!(removed, 1);
        assert_eq!(registry.len().unwrap(), 1);

        // Valid cache should still be there
        assert!(registry.get(&handle2).unwrap().is_some());
        assert!(registry.get(&handle1).unwrap().is_none());
    }

    #[test]
    fn test_list_all() {
        let registry = CacheRegistry::new();
        let handle1 = CacheHandle::OpenAI;
        let handle2 = CacheHandle::Gemini {
            cache_name: "cachedContents/test".to_string(),
        };

        let context1 = CachedContext::new(handle1.clone(), Duration::from_secs(300), 1000);
        let context2 = CachedContext::new(handle2.clone(), Duration::from_secs(300), 2000);

        registry.register(&handle1, context1).unwrap();
        registry.register(&handle2, context2).unwrap();

        let all = registry.list_all().unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_handle_to_key() {
        let claude_handle = CacheHandle::Claude {
            cache_control: serde_json::json!({"type": "ephemeral"}),
        };
        let key = CacheRegistry::handle_to_key(&claude_handle);
        assert!(key.starts_with("claude:"));

        let openai_handle = CacheHandle::OpenAI;
        let key = CacheRegistry::handle_to_key(&openai_handle);
        assert_eq!(key, "openai:auto");

        let gemini_handle = CacheHandle::Gemini {
            cache_name: "cachedContents/abc123".to_string(),
        };
        let key = CacheRegistry::handle_to_key(&gemini_handle);
        assert_eq!(key, "gemini:cachedContents/abc123");
    }
}

