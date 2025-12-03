//! Prompt template caching system.
//!
//! Provides in-memory caching for prompt templates to avoid repeated file I/O.

use crate::prompts::{PromptError, PromptTemplate};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

/// Cache entry for a prompt template.
#[derive(Debug, Clone)]
struct CacheEntry {
    /// The cached template.
    template: PromptTemplate,

    /// When this entry was cached.
    cached_at: SystemTime,

    /// File modification time when cached.
    file_mtime: Option<SystemTime>,
}

/// In-memory cache for prompt templates.
#[derive(Debug, Clone)]
pub struct PromptCache {
    /// Internal cache storage.
    cache: Arc<RwLock<HashMap<PathBuf, CacheEntry>>>,

    /// Maximum cache age before revalidation.
    max_age: Duration,
}

impl PromptCache {
    /// Create a new prompt cache with default settings.
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_age: Duration::from_secs(300), // 5 minutes default
        }
    }

    /// Create a new prompt cache with custom max age.
    pub fn with_max_age(max_age: Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_age,
        }
    }

    /// Get a template from cache or load it.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the template file
    ///
    /// # Errors
    ///
    /// Returns error if template cannot be loaded.
    pub fn get_or_load(&self, path: impl AsRef<Path>) -> Result<PromptTemplate, PromptError> {
        let path = path.as_ref();
        let path_buf = path.to_path_buf();

        // Check cache first
        {
            let cache = self.cache.read().unwrap();
            if let Some(entry) = cache.get(&path_buf) {
                // Check if entry is still valid
                if self.is_entry_valid(entry, path) {
                    return Ok(entry.template.clone());
                }
            }
        }

        // Load template
        let template = PromptTemplate::load(path)?;

        // Get file modification time
        let file_mtime = self.get_file_mtime(path).ok();

        // Cache the template
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(
                path_buf,
                CacheEntry {
                    template: template.clone(),
                    cached_at: SystemTime::now(),
                    file_mtime,
                },
            );
        }

        Ok(template)
    }

    /// Check if a cache entry is still valid.
    fn is_entry_valid(&self, entry: &CacheEntry, path: &Path) -> bool {
        // Check cache age
        if let Ok(elapsed) = entry.cached_at.elapsed() {
            if elapsed > self.max_age {
                return false;
            }
        }

        // Check file modification time
        if let Some(cached_mtime) = entry.file_mtime {
            if let Ok(current_mtime) = self.get_file_mtime(path) {
                if current_mtime != cached_mtime {
                    return false;
                }
            }
        }

        true
    }

    /// Get file modification time.
    fn get_file_mtime(&self, path: &Path) -> Result<SystemTime, std::io::Error> {
        let metadata = std::fs::metadata(path)?;
        metadata.modified()
    }

    /// Clear the entire cache.
    pub fn clear(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }

    /// Remove a specific entry from the cache.
    pub fn remove(&self, path: impl AsRef<Path>) {
        let mut cache = self.cache.write().unwrap();
        cache.remove(path.as_ref());
    }

    /// Get cache statistics.
    pub fn stats(&self) -> CacheStats {
        let cache = self.cache.read().unwrap();
        CacheStats {
            entries: cache.len(),
            max_age_seconds: self.max_age.as_secs(),
        }
    }
}

impl Default for PromptCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics.
#[derive(Debug, Clone, Copy)]
pub struct CacheStats {
    /// Number of cached entries.
    pub entries: usize,

    /// Maximum cache age in seconds.
    pub max_age_seconds: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_cache_get_or_load() {
        let cache = PromptCache::new();
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"Hello {{name}}!").unwrap();
        file.flush().unwrap();

        // First load should cache
        let template1 = cache.get_or_load(file.path()).unwrap();
        assert_eq!(template1.content(), "Hello {{name}}!");

        // Second load should use cache
        let template2 = cache.get_or_load(file.path()).unwrap();
        assert_eq!(template2.content(), "Hello {{name}}!");

        // Stats should show 1 entry
        let stats = cache.stats();
        assert_eq!(stats.entries, 1);
    }

    #[test]
    fn test_cache_clear() {
        let cache = PromptCache::new();
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"Test").unwrap();
        file.flush().unwrap();

        cache.get_or_load(file.path()).unwrap();
        assert_eq!(cache.stats().entries, 1);

        cache.clear();
        assert_eq!(cache.stats().entries, 0);
    }

    #[test]
    fn test_cache_remove() {
        let cache = PromptCache::new();
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        file1.write_all(b"File 1").unwrap();
        file2.write_all(b"File 2").unwrap();
        file1.flush().unwrap();
        file2.flush().unwrap();

        cache.get_or_load(file1.path()).unwrap();
        cache.get_or_load(file2.path()).unwrap();
        assert_eq!(cache.stats().entries, 2);

        cache.remove(file1.path());
        assert_eq!(cache.stats().entries, 1);
    }
}
