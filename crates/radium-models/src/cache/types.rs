//! Core data types for model caching.

use radium_abstraction::Model;
use serde::Serialize;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Instant;

use crate::factory::ModelType;

/// Cache key for identifying cached models.
///
/// Models are cached by a composite key of provider, model name, and API key hash.
/// This ensures that models with different API keys are cached separately.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheKey {
    /// The model provider type.
    pub provider: ModelType,
    /// The model name/ID.
    pub model_name: String,
    /// SHA-256 hash of the API key (for security).
    pub api_key_hash: String,
}

impl CacheKey {
    /// Create a new cache key from model configuration.
    ///
    /// # Arguments
    /// * `provider` - The model provider type
    /// * `model_name` - The model name/ID
    /// * `api_key` - Optional API key (will be hashed)
    ///
    /// # Returns
    /// A new CacheKey with the API key hashed using SHA-256.
    pub fn new(provider: ModelType, model_name: String, api_key: Option<&str>) -> Self {
        use sha2::{Digest, Sha256};

        let api_key_hash = if let Some(key) = api_key {
            let mut hasher = Sha256::new();
            hasher.update(key.as_bytes());
            format!("{:x}", hasher.finalize())
        } else {
            "no-key".to_string()
        };

        Self {
            provider,
            model_name,
            api_key_hash,
        }
    }
}

impl Hash for CacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash all fields that make up the key
        self.provider.hash(state);
        self.model_name.hash(state);
        self.api_key_hash.hash(state);
    }
}

/// A cached model entry with metadata.
#[derive(Clone)]
pub struct CachedModel {
    /// The cached model instance.
    pub model: Arc<dyn Model + Send + Sync>,
    /// Timestamp of last access.
    pub last_accessed: Instant,
    /// Number of times this model has been accessed.
    pub access_count: u64,
    /// Timestamp when the model was first cached.
    pub created_at: Instant,
}

impl CachedModel {
    /// Create a new cached model entry.
    ///
    /// # Arguments
    /// * `model` - The model instance to cache
    ///
    /// # Returns
    /// A new CachedModel with current timestamp and access count of 1.
    pub fn new(model: Arc<dyn Model + Send + Sync>) -> Self {
        let now = Instant::now();
        Self {
            model,
            last_accessed: now,
            access_count: 1,
            created_at: now,
        }
    }

    /// Update the last accessed timestamp and increment access count.
    ///
    /// This should be called whenever the cached model is used.
    pub fn touch(&mut self) {
        self.last_accessed = Instant::now();
        self.access_count += 1;
    }
}

impl std::fmt::Debug for CachedModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CachedModel")
            .field("model_id", &self.model.model_id())
            .field("last_accessed", &self.last_accessed)
            .field("access_count", &self.access_count)
            .field("created_at", &self.created_at)
            .finish()
    }
}

/// Cache statistics for observability.
#[derive(Debug, Clone, Serialize)]
pub struct CacheStats {
    /// Total number of cache hits.
    pub total_hits: u64,
    /// Total number of cache misses.
    pub total_misses: u64,
    /// Total number of evictions (manual or automatic).
    pub total_evictions: u64,
    /// Current number of models in cache.
    pub cache_size: usize,
}

impl Default for CacheStats {
    fn default() -> Self {
        Self {
            total_hits: 0,
            total_misses: 0,
            total_evictions: 0,
            cache_size: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_hashing() {
        let key1 = CacheKey::new(ModelType::Gemini, "gemini-pro".to_string(), Some("key1"));
        let key2 = CacheKey::new(ModelType::Gemini, "gemini-pro".to_string(), Some("key1"));
        let key3 = CacheKey::new(ModelType::Gemini, "gemini-pro".to_string(), Some("key2"));

        // Same inputs should produce same hash
        assert_eq!(key1.api_key_hash, key2.api_key_hash);
        // Different keys should produce different hashes
        assert_ne!(key1.api_key_hash, key3.api_key_hash);
    }

    #[test]
    fn test_cache_key_equality() {
        let key1 = CacheKey::new(ModelType::Gemini, "gemini-pro".to_string(), Some("key1"));
        let key2 = CacheKey::new(ModelType::Gemini, "gemini-pro".to_string(), Some("key1"));
        let key3 = CacheKey::new(ModelType::OpenAI, "gpt-4".to_string(), Some("key1"));

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_cached_model_touch() {
        use crate::MockModel;

        let model = Arc::new(MockModel::new("test".to_string()));
        let mut cached = CachedModel::new(model);

        let initial_count = cached.access_count;
        let initial_time = cached.last_accessed;

        // Wait a tiny bit to ensure time difference
        std::thread::sleep(std::time::Duration::from_millis(10));

        cached.touch();

        assert_eq!(cached.access_count, initial_count + 1);
        assert!(cached.last_accessed > initial_time);
    }

    #[test]
    fn test_cache_stats_default() {
        let stats = CacheStats::default();
        assert_eq!(stats.total_hits, 0);
        assert_eq!(stats.total_misses, 0);
        assert_eq!(stats.total_evictions, 0);
        assert_eq!(stats.cache_size, 0);
    }
}

