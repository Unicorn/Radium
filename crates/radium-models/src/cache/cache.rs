//! ModelCache implementation with lazy loading and LRU eviction.

use crate::factory::{ModelConfig, ModelFactory};
use radium_abstraction::{Model, ModelError};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{debug, info};

use super::config::{CacheConfig, CacheConfigError};
use super::types::{CacheKey, CacheStats, CachedModel};

/// Model cache for optimizing model lifecycle.
///
/// Provides lazy loading, LRU eviction, and automatic cleanup of inactive models.
#[derive(Debug)]
pub struct ModelCache {
    /// The cache storage (key -> cached model).
    cache: Arc<RwLock<HashMap<CacheKey, CachedModel>>>,
    /// Cache configuration.
    config: CacheConfig,
    /// Cache statistics.
    stats: Arc<RwLock<CacheStats>>,
}

impl ModelCache {
    /// Create a new model cache with the given configuration.
    ///
    /// # Arguments
    /// * `config` - Cache configuration
    ///
    /// # Errors
    /// Returns `CacheConfigError` if the configuration is invalid.
    pub fn new(config: CacheConfig) -> Result<Self, CacheConfigError> {
        config.validate()?;

        Ok(Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            config,
            stats: Arc::new(RwLock::new(CacheStats::default())),
        })
    }

    /// Get a model from cache or create it if not present.
    ///
    /// This implements lazy loading: models are only created when first requested.
    /// If the cache is full, the least-recently-used model is evicted.
    ///
    /// # Arguments
    /// * `config` - Model configuration
    ///
    /// # Returns
    /// The cached or newly created model instance.
    ///
    /// # Errors
    /// Returns `ModelError` if model creation fails.
    pub fn get_or_create(
        &self,
        config: ModelConfig,
    ) -> Result<Arc<dyn Model + Send + Sync>, ModelError> {
        // Create cache key from model config
        let cache_key = CacheKey::new(
            config.model_type.clone(),
            config.model_id.clone(),
            config.api_key.as_deref(),
        );

        // Try to get from cache (read lock)
        {
            let cache = self.cache.read().expect("Cache lock poisoned");
            if cache.get(&cache_key).is_some() {
                // Cache hit - update stats and return
                drop(cache);
                let mut stats = self.stats.write().expect("Stats lock poisoned");
                stats.total_hits += 1;
                drop(stats);

                // Update last_accessed (need write lock for this)
                let mut cache = self.cache.write().expect("Cache lock poisoned");
                if let Some(cached) = cache.get_mut(&cache_key) {
                    cached.touch();
                }
                let cached = cache.get(&cache_key).unwrap().clone();
                debug!(
                    provider = ?cache_key.provider,
                    model = %cache_key.model_name,
                    "Cache hit"
                );
                return Ok(cached.model);
            }
        }

        // Cache miss - create new model
        {
            let mut stats = self.stats.write().expect("Stats lock poisoned");
            stats.total_misses += 1;
            drop(stats);
        }

        debug!(
            provider = ?cache_key.provider,
            model = %cache_key.model_name,
            "Cache miss, creating model"
        );

        // Check if cache is full and evict LRU if needed
        {
            let mut cache = self.cache.write().expect("Cache lock poisoned");
            let current_size = cache.len();

            if current_size >= self.config.max_cache_size {
                // Find and evict least-recently-used model
                if let Some(lru_key) = Self::find_lru_key(&cache) {
                    cache.remove(&lru_key);
                    let mut stats = self.stats.write().expect("Stats lock poisoned");
                    stats.total_evictions += 1;
                    stats.cache_size = cache.len();
                    drop(stats);
                    info!(
                        provider = ?lru_key.provider,
                        model = %lru_key.model_name,
                        "Evicted LRU model from cache"
                    );
                }
            }
        }

        // Create the model
        let model = ModelFactory::create(config)?;

        // Insert into cache
        {
            let mut cache = self.cache.write().expect("Cache lock poisoned");
            let cached = CachedModel::new(Arc::clone(&model));
            cache.insert(cache_key.clone(), cached);
            let mut stats = self.stats.write().expect("Stats lock poisoned");
            stats.cache_size = cache.len();
            drop(stats);
        }

        info!(
            provider = ?cache_key.provider,
            model = %cache_key.model_name,
            "Model cached"
        );

        Ok(model)
    }

    /// Find the least-recently-used key in the cache.
    ///
    /// # Arguments
    /// * `cache` - The cache to search
    ///
    /// # Returns
    /// The key of the least-recently-used entry, or None if cache is empty.
    fn find_lru_key(cache: &HashMap<CacheKey, CachedModel>) -> Option<CacheKey> {
        cache
            .iter()
            .min_by_key(|(_, cached)| cached.last_accessed)
            .map(|(key, _)| key.clone())
    }

    /// Get current cache statistics.
    ///
    /// # Returns
    /// A snapshot of current cache statistics.
    #[must_use]
    pub fn get_stats(&self) -> CacheStats {
        let stats = self.stats.read().expect("Stats lock poisoned");
        let mut result = stats.clone();
        let cache = self.cache.read().expect("Cache lock poisoned");
        result.cache_size = cache.len();
        result
    }

    /// Clear all models from the cache.
    pub fn clear(&self) {
        let mut cache = self.cache.write().expect("Cache lock poisoned");
        let cleared_count = cache.len();
        cache.clear();
        let mut stats = self.stats.write().expect("Stats lock poisoned");
        stats.cache_size = 0;
        drop(stats);
        info!(cleared_count, "Cleared all models from cache");
    }

    /// Remove a specific model from the cache.
    ///
    /// # Arguments
    /// * `key` - The cache key to remove
    ///
    /// # Returns
    /// `true` if the model was removed, `false` if it was not found.
    pub fn remove(&self, key: &CacheKey) -> bool {
        let mut cache = self.cache.write().expect("Cache lock poisoned");
        let removed = cache.remove(key).is_some();
        if removed {
            let mut stats = self.stats.write().expect("Stats lock poisoned");
            stats.cache_size = cache.len();
            drop(stats);
            info!(
                provider = ?key.provider,
                model = %key.model_name,
                "Removed model from cache"
            );
        }
        removed
    }

    /// Get the cache configuration.
    #[must_use]
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::ModelType;

    #[test]
    fn test_cache_hit_returns_same_instance() {
        let cache = ModelCache::new(CacheConfig::default()).unwrap();
        let config1 = ModelConfig::new(ModelType::Mock, "test-model".to_string());
        let config2 = ModelConfig::new(ModelType::Mock, "test-model".to_string());

        // First call - cache miss
        let model1 = cache.get_or_create(config1).unwrap();

        // Second call - cache hit
        let model2 = cache.get_or_create(config2).unwrap();

        // Should be the same Arc (pointer equality)
        assert!(Arc::ptr_eq(&model1, &model2));

        let stats = cache.get_stats();
        assert_eq!(stats.total_hits, 1);
        assert_eq!(stats.total_misses, 1);
    }

    #[test]
    fn test_lru_eviction_when_cache_full() {
        let mut config = CacheConfig::default();
        config.max_cache_size = 2;
        let cache = ModelCache::new(config).unwrap();

        // Create 3 different models
        let _model1 = cache
            .get_or_create(ModelConfig::new(ModelType::Mock, "model-1".to_string()))
            .unwrap();
        let model2 = cache
            .get_or_create(ModelConfig::new(ModelType::Mock, "model-2".to_string()))
            .unwrap();

        // Access model1 to update its last_accessed
        let _ = cache
            .get_or_create(ModelConfig::new(ModelType::Mock, "model-1".to_string()))
            .unwrap();

        // Create model3 - should evict model2 (least recently used)
        let _model3 = cache
            .get_or_create(ModelConfig::new(ModelType::Mock, "model-3".to_string()))
            .unwrap();

        // Verify model2 is evicted (cache miss on next access)
        let stats = cache.get_stats();
        assert_eq!(stats.total_evictions, 1);
        assert_eq!(stats.cache_size, 2);

        // model2 should not be in cache anymore
        let config2 = ModelConfig::new(ModelType::Mock, "model-2".to_string());
        let model2_new = cache.get_or_create(config2).unwrap();
        // Should be a different instance (newly created)
        assert!(!Arc::ptr_eq(&model2, &model2_new));
    }

    #[test]
    fn test_cache_stats_tracking() {
        let cache = ModelCache::new(CacheConfig::default()).unwrap();

        // Create a model (miss)
        let _ = cache
            .get_or_create(ModelConfig::new(ModelType::Mock, "test".to_string()))
            .unwrap();

        let stats = cache.get_stats();
        assert_eq!(stats.total_misses, 1);
        assert_eq!(stats.total_hits, 0);
        assert_eq!(stats.cache_size, 1);

        // Access again (hit)
        let _ = cache
            .get_or_create(ModelConfig::new(ModelType::Mock, "test".to_string()))
            .unwrap();

        let stats = cache.get_stats();
        assert_eq!(stats.total_misses, 1);
        assert_eq!(stats.total_hits, 1);
        assert_eq!(stats.cache_size, 1);
    }

    #[test]
    fn test_cache_clear() {
        let cache = ModelCache::new(CacheConfig::default()).unwrap();

        // Add some models
        let _ = cache
            .get_or_create(ModelConfig::new(ModelType::Mock, "model-1".to_string()))
            .unwrap();
        let _ = cache
            .get_or_create(ModelConfig::new(ModelType::Mock, "model-2".to_string()))
            .unwrap();

        assert_eq!(cache.get_stats().cache_size, 2);

        cache.clear();

        assert_eq!(cache.get_stats().cache_size, 0);
    }

    #[test]
    fn test_cache_remove() {
        let cache = ModelCache::new(CacheConfig::default()).unwrap();

        let config = ModelConfig::new(ModelType::Mock, "test-model".to_string());
        let key = CacheKey::new(
            config.model_type.clone(),
            config.model_id.clone(),
            config.api_key.as_deref(),
        );

        // Add model
        let _ = cache.get_or_create(config).unwrap();
        assert_eq!(cache.get_stats().cache_size, 1);

        // Remove it
        assert!(cache.remove(&key));
        assert_eq!(cache.get_stats().cache_size, 0);

        // Try to remove again (should return false)
        assert!(!cache.remove(&key));
    }
}
