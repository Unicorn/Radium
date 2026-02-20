//! ModelCache implementation with lazy loading and LRU eviction.

use crate::factory::{ModelConfig, ModelFactory};
use radium_abstraction::{Model, ModelError};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::time::interval;
use tracing::{debug, info};

use super::config::{CacheConfig, CacheConfigError};
use super::types::{CacheKey, CacheStats, CachedModel};

/// Model cache for optimizing model lifecycle.
///
/// Provides lazy loading, LRU eviction, and automatic cleanup of inactive models.
pub struct ModelCache {
    /// The cache storage (key -> cached model).
    cache: Arc<RwLock<HashMap<CacheKey, CachedModel>>>,
    /// Cache configuration.
    config: CacheConfig,
    /// Cache statistics.
    stats: Arc<RwLock<CacheStats>>,
    /// Cleanup task handle.
    #[allow(dead_code)] // Used in Drop
    cleanup_handle: Option<tokio::task::JoinHandle<()>>,
    /// Shutdown signal sender.
    #[allow(dead_code)] // Used in Drop
    shutdown_tx: Option<oneshot::Sender<()>>,
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

        let cache = Arc::new(RwLock::new(HashMap::new()));
        let stats = Arc::new(RwLock::new(CacheStats::default()));

        // Spawn background cleanup task if enabled and in a Tokio runtime
        let (cleanup_handle, shutdown_tx) = if config.enabled {
            // Only spawn if we're in a Tokio runtime
            if tokio::runtime::Handle::try_current().is_ok() {
                let (tx, rx) = oneshot::channel();
                let cache_clone = Arc::clone(&cache);
                let stats_clone = Arc::clone(&stats);
                let config_clone = config.clone();
                let cleanup_interval = config.cleanup_interval();

                let handle = tokio::spawn(async move {
                    Self::cleanup_task(cache_clone, stats_clone, config_clone, cleanup_interval, rx).await;
                });

                (Some(handle), Some(tx))
            } else {
                // Not in a Tokio runtime - cleanup task will not run
                debug!("ModelCache created outside Tokio runtime, cleanup task disabled");
                (None, None)
            }
        } else {
            (None, None)
        };

        Ok(Self {
            cache,
            config,
            stats,
            cleanup_handle,
            shutdown_tx,
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

    /// List all cached models.
    ///
    /// # Returns
    /// A vector of (cache key, cached model) pairs.
    pub fn list_models(&self) -> Vec<(CacheKey, CachedModel)> {
        let cache = self.cache.read().expect("Cache lock poisoned");
        cache.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    /// Background cleanup task that periodically evicts stale models.
    async fn cleanup_task(
        cache: Arc<RwLock<HashMap<CacheKey, CachedModel>>>,
        stats: Arc<RwLock<CacheStats>>,
        config: CacheConfig,
        cleanup_interval: Duration,
        mut shutdown_rx: oneshot::Receiver<()>,
    ) {
        let mut interval = interval(cleanup_interval);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    Self::cleanup_stale_models(&cache, &stats, &config);
                }
                _ = &mut shutdown_rx => {
                    debug!("Cleanup task received shutdown signal");
                    break;
                }
            }
        }
    }

    /// Clean up stale models that exceed the inactivity timeout.
    fn cleanup_stale_models(
        cache: &Arc<RwLock<HashMap<CacheKey, CachedModel>>>,
        stats: &Arc<RwLock<CacheStats>>,
        config: &CacheConfig,
    ) {
        let now = std::time::Instant::now();
        let timeout = config.inactivity_timeout();

        // Find stale keys (read lock)
        let stale_keys: Vec<CacheKey> = {
            let cache_guard = cache.read().expect("Cache lock poisoned");
            cache_guard
                .iter()
                .filter(|(_, cached)| now.duration_since(cached.last_accessed) >= timeout)
                .map(|(key, _)| key.clone())
                .collect()
        };

        if stale_keys.is_empty() {
            return;
        }

        // Remove stale entries (write lock)
        let mut cache_guard = cache.write().expect("Cache lock poisoned");
        let mut stats_guard = stats.write().expect("Stats lock poisoned");

        for key in stale_keys {
            if let Some(cached) = cache_guard.remove(&key) {
                stats_guard.total_evictions += 1;
                let age = now.duration_since(cached.created_at);
                info!(
                    provider = ?key.provider,
                    model = %key.model_name,
                    age_secs = age.as_secs(),
                    "Evicted stale model due to inactivity timeout"
                );
            }
        }

        stats_guard.cache_size = cache_guard.len();
    }
}

impl Drop for ModelCache {
    fn drop(&mut self) {
        // Send shutdown signal to cleanup task
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }

        // Wait for cleanup task to finish (with timeout to prevent hanging)
        if let Some(handle) = self.cleanup_handle.take() {
            // Try to wait for the task, but don't block if we're in an async context
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(rt) = rt {
                // We're in a Tokio runtime - spawn a blocking task to wait
                if rt.runtime_flavor() == tokio::runtime::RuntimeFlavor::CurrentThread {
                    // Current thread runtime - can't block, just abort
                    handle.abort();
                } else {
                    // Multi-thread runtime - can use spawn_blocking
                    let _ = rt.spawn_blocking(move || {
                        // Use a simple timeout approach
                        std::thread::sleep(Duration::from_millis(100));
                        handle.abort();
                    });
                }
            } else {
                // Not in a Tokio runtime - just abort the task
                handle.abort();
            }
        }
    }
}

impl std::fmt::Debug for ModelCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelCache")
            .field("cache_size", &self.cache.read().unwrap().len())
            .field("config", &self.config)
            .field("stats", &self.get_stats())
            .finish()
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

    #[tokio::test]
    async fn test_stale_models_evicted() {
        let mut config = CacheConfig::default();
        config.inactivity_timeout_secs = 1; // 1 second timeout
        config.cleanup_interval_secs = 1; // Check every second
        let cache = ModelCache::new(config).unwrap();

        // Create a model
        let _ = cache
            .get_or_create(ModelConfig::new(ModelType::Mock, "test-model".to_string()))
            .unwrap();

        assert_eq!(cache.get_stats().cache_size, 1);

        // Wait for inactivity timeout + cleanup interval
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;

        // Model should be evicted
        let stats = cache.get_stats();
        assert_eq!(stats.cache_size, 0);
        assert!(stats.total_evictions > 0);
    }

    #[tokio::test]
    async fn test_active_models_not_evicted() {
        let mut config = CacheConfig::default();
        config.inactivity_timeout_secs = 2; // 2 second timeout
        config.cleanup_interval_secs = 1; // Check every second
        let cache = ModelCache::new(config).unwrap();

        // Create a model
        let _ = cache
            .get_or_create(ModelConfig::new(ModelType::Mock, "test-model".to_string()))
            .unwrap();

        // Access it every second (before timeout)
        for _ in 0..3 {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            let _ = cache
                .get_or_create(ModelConfig::new(ModelType::Mock, "test-model".to_string()))
                .unwrap();
        }

        // Model should still be in cache
        assert_eq!(cache.get_stats().cache_size, 1);
    }

    #[tokio::test]
    async fn test_cleanup_task_shuts_down_gracefully() {
        let config = CacheConfig::default();
        let cache = ModelCache::new(config).unwrap();

        // Drop the cache - cleanup task should shut down
        drop(cache);

        // Give it a moment to shut down
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // If we get here without hanging, shutdown worked
    }
}
