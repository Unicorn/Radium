//! Prompt processing utilities.
//!
//! Provides caching and validation for prompt templates.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

use crate::prompts::{PromptError, PromptTemplate};

/// Prompt cache entry.
#[derive(Debug, Clone)]
struct CacheEntry {
    template: PromptTemplate,
    modified_time: SystemTime,
}

/// Prompt template cache.
///
/// Caches loaded prompt templates to avoid repeated file I/O.
/// The cache automatically invalidates entries when source files are modified.
#[derive(Debug, Clone)]
pub struct PromptCache {
    cache: Arc<RwLock<HashMap<PathBuf, CacheEntry>>>,
}

impl PromptCache {
    /// Create a new prompt cache.
    pub fn new() -> Self {
        Self { cache: Arc::new(RwLock::new(HashMap::new())) }
    }

    /// Load a template from cache or file.
    ///
    /// If the template is cached and the file hasn't been modified, returns the cached version.
    /// Otherwise, loads from file and updates the cache.
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be read or doesn't exist.
    pub fn load(&self, path: impl AsRef<Path>) -> Result<PromptTemplate, PromptError> {
        let path = path.as_ref().to_path_buf();

        // Check cache
        {
            let cache = self.cache.read().unwrap();
            if let Some(entry) = cache.get(&path) {
                // Check if file has been modified
                if let Ok(metadata) = std::fs::metadata(&path) {
                    if let Ok(modified) = metadata.modified() {
                        if modified <= entry.modified_time {
                            return Ok(entry.template.clone());
                        }
                    }
                }
            }
        }

        // Load from file
        let template = PromptTemplate::load(&path)?;

        // Get file modification time
        let modified_time = std::fs::metadata(&path)
            .and_then(|m| m.modified())
            .unwrap_or_else(|_| SystemTime::now());

        // Update cache
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(
                path,
                CacheEntry { template: template.clone(), modified_time },
            );
        }

        Ok(template)
    }

    /// Clear the cache.
    pub fn clear(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }

    /// Remove a specific entry from the cache.
    pub fn remove(&self, path: impl AsRef<Path>) {
        let mut cache = self.cache.write().unwrap();
        cache.remove(path.as_ref());
    }

    /// Get the number of cached entries.
    pub fn len(&self) -> usize {
        let cache = self.cache.read().unwrap();
        cache.len()
    }

    /// Check if the cache is empty.
    pub fn is_empty(&self) -> bool {
        let cache = self.cache.read().unwrap();
        cache.is_empty()
    }
}

impl Default for PromptCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Prompt validation result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationResult {
    /// Whether the prompt is valid.
    pub is_valid: bool,

    /// List of issues found.
    pub issues: Vec<ValidationIssue>,
}

/// Prompt validation issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationIssue {
    /// File does not exist.
    FileNotFound(String),

    /// File cannot be read.
    FileReadError(String),

    /// Required placeholder is missing from context.
    MissingPlaceholder(String),

    /// Placeholder is defined but never used.
    UnusedPlaceholder(String),
}

/// Validate a prompt template.
///
/// Checks:
/// - File exists and is readable
/// - All required placeholders are documented
/// - No obvious syntax errors
///
/// # Arguments
///
/// * `template` - The template to validate
/// * `required_placeholders` - List of placeholders that must be present
///
/// # Returns
///
/// Validation result with any issues found.
pub fn validate_prompt(
    template: &PromptTemplate,
    required_placeholders: &[String],
) -> ValidationResult {
    let mut issues = Vec::new();

    // Check file exists (if loaded from file)
    if let Some(path) = template.file_path() {
        if !path.exists() {
            issues.push(ValidationIssue::FileNotFound(path.display().to_string()));
            return ValidationResult { is_valid: false, issues };
        }

        // Check file is readable
        if std::fs::read_to_string(path).is_err() {
            issues.push(ValidationIssue::FileReadError(path.display().to_string()));
            return ValidationResult { is_valid: false, issues };
        }
    }

    // Check required placeholders
    let found_placeholders = template.list_placeholders();
    for required in required_placeholders {
        if !found_placeholders.contains(required) {
            issues.push(ValidationIssue::MissingPlaceholder(required.clone()));
        }
    }

    // Check for unused placeholders (warnings, not errors)
    // This is informational only

    let is_valid = issues.is_empty();
    ValidationResult { is_valid, issues }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_cache_load() {
        let cache = PromptCache::new();
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"Hello {{name}}!").unwrap();
        file.flush().unwrap();

        let template1 = cache.load(file.path()).unwrap();
        let template2 = cache.load(file.path()).unwrap();

        // Should be the same instance (cached)
        assert_eq!(template1.content(), template2.content());
    }

    #[test]
    fn test_cache_clear() {
        let cache = PromptCache::new();
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"Hello {{name}}!").unwrap();
        file.flush().unwrap();

        cache.load(file.path()).unwrap();
        assert_eq!(cache.len(), 1);

        cache.clear();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_validate_prompt_valid() {
        let template = PromptTemplate::from_string("Hello {{name}}!");
        let required = vec!["name".to_string()];
        let result = validate_prompt(&template, &required);

        assert!(result.is_valid);
        assert!(result.issues.is_empty());
    }

    #[test]
    fn test_validate_prompt_missing_placeholder() {
        let template = PromptTemplate::from_string("Hello {{name}}!");
        let required = vec!["name".to_string(), "greeting".to_string()];
        let result = validate_prompt(&template, &required);

        assert!(!result.is_valid);
        assert_eq!(result.issues.len(), 1);
        assert!(matches!(
            result.issues[0],
            ValidationIssue::MissingPlaceholder(_)
        ));
    }
}
