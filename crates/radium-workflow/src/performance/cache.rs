//! Compilation Cache
//!
//! LRU-based caching for compiled workflow templates to avoid
//! redundant compilation of unchanged workflows.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

/// Hash identifying a unique workflow compilation
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct WorkflowHash {
    /// Hash of the workflow definition content
    pub definition_hash: u64,
    /// Compiler version used
    pub compiler_version: String,
}

impl WorkflowHash {
    /// Create a new workflow hash
    pub fn new(definition_hash: u64, compiler_version: impl Into<String>) -> Self {
        Self {
            definition_hash,
            compiler_version: compiler_version.into(),
        }
    }

    /// Create hash from workflow JSON
    pub fn from_json(json: &str) -> Self {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        json.hash(&mut hasher);
        Self {
            definition_hash: hasher.finish(),
            compiler_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Cached compilation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedCompilation {
    /// Generated TypeScript code
    pub typescript_code: String,
    /// When the compilation was generated
    pub generated_at: chrono::DateTime<chrono::Utc>,
    /// How long compilation took in milliseconds
    pub compilation_time_ms: u64,
    /// Size of generated code in bytes
    pub code_size_bytes: usize,
}

impl CachedCompilation {
    /// Create a new cached compilation
    pub fn new(typescript_code: String, compilation_time_ms: u64) -> Self {
        let code_size_bytes = typescript_code.len();
        Self {
            typescript_code,
            generated_at: chrono::Utc::now(),
            compilation_time_ms,
            code_size_bytes,
        }
    }
}

/// Statistics for cache performance monitoring
#[derive(Debug, Default)]
pub struct CacheStats {
    /// Total cache hits
    pub hits: AtomicU64,
    /// Total cache misses
    pub misses: AtomicU64,
    /// Total evictions
    pub evictions: AtomicU64,
    /// Total bytes saved by cache hits
    pub bytes_saved: AtomicU64,
    /// Total time saved by cache hits (ms)
    pub time_saved_ms: AtomicU64,
}

/// Snapshot of cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStatsSnapshot {
    /// Total cache hits
    pub hits: u64,
    /// Total cache misses
    pub misses: u64,
    /// Total evictions
    pub evictions: u64,
    /// Current cache size (entries)
    pub size: usize,
    /// Maximum cache capacity
    pub capacity: usize,
    /// Total bytes saved
    pub bytes_saved: u64,
    /// Total time saved in milliseconds
    pub time_saved_ms: u64,
}

impl CacheStatsSnapshot {
    /// Calculate cache hit rate as percentage
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }

    /// Calculate average time saved per hit
    pub fn avg_time_saved_ms(&self) -> f64 {
        if self.hits == 0 {
            0.0
        } else {
            self.time_saved_ms as f64 / self.hits as f64
        }
    }
}

/// LRU Cache entry
struct CacheEntry {
    value: CachedCompilation,
    /// For LRU tracking - lower is older
    access_order: u64,
}

/// LRU-based compilation cache
///
/// Thread-safe cache using RwLock for concurrent access.
/// Automatically evicts least recently used entries when capacity is reached.
pub struct CompilationCache {
    /// The cache storage
    cache: RwLock<HashMap<WorkflowHash, CacheEntry>>,
    /// Maximum number of entries
    capacity: usize,
    /// Access counter for LRU ordering
    access_counter: AtomicU64,
    /// Cache statistics
    stats: CacheStats,
}

impl CompilationCache {
    /// Create a new compilation cache with specified capacity
    ///
    /// # Panics
    /// Panics if capacity is 0
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Cache capacity must be greater than 0");
        Self {
            cache: RwLock::new(HashMap::with_capacity(capacity)),
            capacity,
            access_counter: AtomicU64::new(0),
            stats: CacheStats::default(),
        }
    }

    /// Get a cached compilation if it exists
    ///
    /// Updates access time for LRU tracking
    pub fn get(&self, hash: &WorkflowHash) -> Option<CachedCompilation> {
        let mut cache = self.cache.write().unwrap();

        if let Some(entry) = cache.get_mut(hash) {
            // Update access order for LRU
            entry.access_order = self.access_counter.fetch_add(1, Ordering::Relaxed);

            // Update stats
            self.stats.hits.fetch_add(1, Ordering::Relaxed);
            self.stats
                .time_saved_ms
                .fetch_add(entry.value.compilation_time_ms, Ordering::Relaxed);
            self.stats
                .bytes_saved
                .fetch_add(entry.value.code_size_bytes as u64, Ordering::Relaxed);

            Some(entry.value.clone())
        } else {
            self.stats.misses.fetch_add(1, Ordering::Relaxed);
            None
        }
    }

    /// Store a compilation in the cache
    ///
    /// Evicts least recently used entry if at capacity
    pub fn put(&self, hash: WorkflowHash, compilation: CachedCompilation) {
        let mut cache = self.cache.write().unwrap();

        // Evict if at capacity and not updating existing entry
        if cache.len() >= self.capacity && !cache.contains_key(&hash) {
            self.evict_lru(&mut cache);
        }

        let access_order = self.access_counter.fetch_add(1, Ordering::Relaxed);
        cache.insert(
            hash,
            CacheEntry {
                value: compilation,
                access_order,
            },
        );
    }

    /// Evict the least recently used entry
    fn evict_lru(&self, cache: &mut HashMap<WorkflowHash, CacheEntry>) {
        if let Some((lru_key, _)) = cache
            .iter()
            .min_by_key(|(_, entry)| entry.access_order)
            .map(|(k, v)| (k.clone(), v.access_order))
        {
            cache.remove(&lru_key);
            self.stats.evictions.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Get current cache statistics
    pub fn stats(&self) -> CacheStatsSnapshot {
        let cache = self.cache.read().unwrap();
        CacheStatsSnapshot {
            hits: self.stats.hits.load(Ordering::Relaxed),
            misses: self.stats.misses.load(Ordering::Relaxed),
            evictions: self.stats.evictions.load(Ordering::Relaxed),
            size: cache.len(),
            capacity: self.capacity,
            bytes_saved: self.stats.bytes_saved.load(Ordering::Relaxed),
            time_saved_ms: self.stats.time_saved_ms.load(Ordering::Relaxed),
        }
    }

    /// Clear the entire cache
    pub fn clear(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }

    /// Remove a specific entry from the cache
    pub fn invalidate(&self, hash: &WorkflowHash) -> bool {
        let mut cache = self.cache.write().unwrap();
        cache.remove(hash).is_some()
    }

    /// Check if a hash exists in the cache
    pub fn contains(&self, hash: &WorkflowHash) -> bool {
        let cache = self.cache.read().unwrap();
        cache.contains_key(hash)
    }

    /// Get current number of entries
    pub fn len(&self) -> usize {
        let cache = self.cache.read().unwrap();
        cache.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for CompilationCache {
    fn default() -> Self {
        Self::new(1000) // Default to 1000 entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_basic_operations() {
        let cache = CompilationCache::new(10);

        let hash = WorkflowHash::new(12345, "1.0.0");
        let compilation = CachedCompilation::new("const x = 1;".to_string(), 50);

        // Should miss on first access
        assert!(cache.get(&hash).is_none());

        // Put and get
        cache.put(hash.clone(), compilation.clone());
        let result = cache.get(&hash);
        assert!(result.is_some());
        assert_eq!(result.unwrap().typescript_code, "const x = 1;");
    }

    #[test]
    fn test_cache_lru_eviction() {
        let cache = CompilationCache::new(3);

        // Fill cache
        for i in 0..3 {
            let hash = WorkflowHash::new(i, "1.0.0");
            let compilation = CachedCompilation::new(format!("code_{}", i), 10);
            cache.put(hash, compilation);
        }

        assert_eq!(cache.len(), 3);

        // Access first entry to make it recently used
        let _ = cache.get(&WorkflowHash::new(0, "1.0.0"));

        // Add new entry - should evict entry 1 (oldest non-accessed)
        let new_hash = WorkflowHash::new(100, "1.0.0");
        cache.put(new_hash, CachedCompilation::new("new".to_string(), 10));

        assert_eq!(cache.len(), 3);

        // Entry 0 should still exist (was accessed)
        assert!(cache.contains(&WorkflowHash::new(0, "1.0.0")));

        // Entry 1 should be evicted (oldest)
        assert!(!cache.contains(&WorkflowHash::new(1, "1.0.0")));
    }

    #[test]
    fn test_cache_stats() {
        let cache = CompilationCache::new(10);

        let hash = WorkflowHash::new(1, "1.0.0");
        let compilation = CachedCompilation::new("code".to_string(), 100);

        // Miss
        cache.get(&hash);

        // Put and hit
        cache.put(hash.clone(), compilation);
        cache.get(&hash);

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate(), 50.0);
    }

    #[test]
    fn test_cache_clear() {
        let cache = CompilationCache::new(10);

        for i in 0..5 {
            let hash = WorkflowHash::new(i, "1.0.0");
            cache.put(hash, CachedCompilation::new("code".to_string(), 10));
        }

        assert_eq!(cache.len(), 5);

        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_workflow_hash_from_json() {
        let json1 = r#"{"name": "test"}"#;
        let json2 = r#"{"name": "test"}"#;
        let json3 = r#"{"name": "different"}"#;

        let hash1 = WorkflowHash::from_json(json1);
        let hash2 = WorkflowHash::from_json(json2);
        let hash3 = WorkflowHash::from_json(json3);

        // Same JSON should produce same hash
        assert_eq!(hash1.definition_hash, hash2.definition_hash);

        // Different JSON should produce different hash
        assert_ne!(hash1.definition_hash, hash3.definition_hash);
    }

    #[test]
    fn test_cache_invalidate() {
        let cache = CompilationCache::new(10);

        let hash = WorkflowHash::new(1, "1.0.0");
        cache.put(hash.clone(), CachedCompilation::new("code".to_string(), 10));

        assert!(cache.contains(&hash));
        assert!(cache.invalidate(&hash));
        assert!(!cache.contains(&hash));

        // Invalidating non-existent should return false
        assert!(!cache.invalidate(&hash));
    }
}
