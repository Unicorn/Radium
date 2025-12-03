//! Prompt template loading and processing.
//!
//! Implements markdown-based prompt templates with placeholder replacement.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use thiserror::Error;

/// Prompt template errors.
#[derive(Debug, Error)]
pub enum PromptError {
    /// Template file not found.
    #[error("template not found: {0}")]
    NotFound(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Missing placeholder value.
    #[error("missing placeholder value: {0}")]
    MissingPlaceholder(String),

    /// Invalid template syntax.
    #[error("invalid template syntax: {0}")]
    InvalidSyntax(String),
}

/// Result type for prompt operations.
pub type Result<T> = std::result::Result<T, PromptError>;

/// Prompt template context for variable replacement.
///
/// Stores key-value pairs that will be used to replace placeholders in templates.
#[derive(Debug, Clone, Default)]
pub struct PromptContext {
    values: HashMap<String, String>,
}

impl PromptContext {
    /// Create a new empty context.
    pub fn new() -> Self {
        Self { values: HashMap::new() }
    }

    /// Set a context value.
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.values.insert(key.into(), value.into());
    }

    /// Get a context value.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }

    /// Check if context contains a key.
    pub fn contains(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    /// Remove a context value.
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.values.remove(key)
    }

    /// Clear all context values.
    pub fn clear(&mut self) {
        self.values.clear();
    }
}

/// Prompt template.
///
/// Represents a loaded prompt template with support for placeholder replacement.
#[derive(Debug, Clone)]
pub struct PromptTemplate {
    /// Template content.
    content: String,

    /// Template file path (if loaded from file).
    file_path: Option<PathBuf>,
}

impl PromptTemplate {
    /// Load a prompt template from a file.
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be read.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(PromptError::NotFound(path.display().to_string()));
        }

        let content = fs::read_to_string(path)?;
        Ok(Self { content, file_path: Some(path.to_path_buf()) })
    }

    /// Create a template from a string.
    pub fn from_string(content: impl Into<String>) -> Self {
        Self { content: content.into(), file_path: None }
    }

    /// Get the template content.
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Get the template file path.
    pub fn file_path(&self) -> Option<&Path> {
        self.file_path.as_deref()
    }

    /// Render the template with the given context.
    ///
    /// Replaces all placeholders in the format `{{KEY}}` with values from the context.
    ///
    /// # Errors
    ///
    /// Returns error if a required placeholder is missing from the context.
    pub fn render(&self, context: &PromptContext) -> Result<String> {
        self.render_with_options(context, &RenderOptions::default())
    }

    /// Render the template with custom options.
    ///
    /// # Errors
    ///
    /// Returns error if a required placeholder is missing from the context.
    pub fn render_with_options(
        &self,
        context: &PromptContext,
        options: &RenderOptions,
    ) -> Result<String> {
        let mut result = self.content.clone();

        // Find all placeholders in the format {{KEY}}
        let placeholders = Self::find_placeholders(&result);

        for placeholder in placeholders {
            let value = context.get(&placeholder);

            let replacement = if let Some(value) = value {
                value.to_string()
            } else if options.strict {
                return Err(PromptError::MissingPlaceholder(placeholder));
            } else if let Some(default) = &options.default_value {
                default.clone()
            } else {
                String::new()
            };

            let pattern = format!("{{{{{}}}}}", placeholder);
            result = result.replace(&pattern, &replacement);
        }

        Ok(result)
    }

    /// Find all placeholders in the template.
    fn find_placeholders(content: &str) -> Vec<String> {
        let mut placeholders = Vec::new();
        let mut chars = content.chars().peekable();

        while let Some(c) = chars.next() {
            #[allow(clippy::collapsible_if)]
            if c == '{' {
                if chars.peek() == Some(&'{') {
                    chars.next(); // consume second {

                    // Read until we find }}
                    let mut placeholder = String::new();
                    let mut found_end = false;

                    #[allow(clippy::collapsible_if)]
                    while let Some(c) = chars.next() {
                        if c == '}' {
                            if chars.peek() == Some(&'}') {
                                chars.next(); // consume second }
                                found_end = true;
                                break;
                            }
                        }
                        placeholder.push(c);
                    }

                    if found_end && !placeholder.is_empty() {
                        let placeholder = placeholder.trim().to_string();
                        if !placeholders.contains(&placeholder) {
                            placeholders.push(placeholder);
                        }
                    }
                }
            }
        }

        placeholders
    }

    /// List all placeholders in the template.
    pub fn list_placeholders(&self) -> Vec<String> {
        Self::find_placeholders(&self.content)
    }
}

/// Options for template rendering.
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct RenderOptions {
    /// Strict mode: error if placeholder is missing.
    pub strict: bool,

    /// Default value for missing placeholders (only used if not strict).
    pub default_value: Option<String>,
}

/// Cached template entry.
#[derive(Debug, Clone)]
struct CachedTemplate {
    /// The cached template.
    template: PromptTemplate,
    /// When this template was loaded.
    loaded_at: SystemTime,
    /// File modification time when cached.
    file_mtime: Option<SystemTime>,
}

/// Prompt template cache.
///
/// Caches loaded templates to avoid repeated file I/O.
/// Templates are automatically invalidated when source files change.
#[derive(Debug, Clone)]
pub struct PromptCache {
    inner: Arc<RwLock<HashMap<PathBuf, CachedTemplate>>>,
    /// Maximum cache age (None = no expiration).
    max_age: Option<Duration>,
}

impl PromptCache {
    /// Create a new cache with no expiration.
    pub fn new() -> Self {
        Self { inner: Arc::new(RwLock::new(HashMap::new())), max_age: None }
    }

    /// Create a new cache with a maximum age.
    pub fn with_max_age(max_age: Duration) -> Self {
        Self { inner: Arc::new(RwLock::new(HashMap::new())), max_age: Some(max_age) }
    }

    /// Load a template, using cache if available and valid.
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be read or cache check fails.
    pub fn load(&self, path: impl AsRef<Path>) -> Result<PromptTemplate> {
        let path = path.as_ref();
        let path_buf = path.to_path_buf();

        // Check cache
        {
            let cache = self.inner.read().map_err(|e| {
                PromptError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("cache lock error: {}", e),
                ))
            })?;

            if let Some(cached) = cache.get(&path_buf) {
                // Check if cache entry is still valid
                if self.is_cache_valid(cached, path)? {
                    return Ok(cached.template.clone());
                }
            }
        }

        // Load from file
        let template = PromptTemplate::load(path)?;

        // Update cache
        {
            let file_mtime = Self::get_file_mtime(path).ok();
            let cached = CachedTemplate {
                template: template.clone(),
                loaded_at: SystemTime::now(),
                file_mtime,
            };

            let mut cache = self.inner.write().map_err(|e| {
                PromptError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("cache lock error: {}", e),
                ))
            })?;

            cache.insert(path_buf, cached);
        }

        Ok(template)
    }

    /// Check if a cached entry is still valid.
    fn is_cache_valid(&self, cached: &CachedTemplate, path: &Path) -> Result<bool> {
        // Check max age
        if let Some(max_age) = self.max_age {
            if let Ok(elapsed) = cached.loaded_at.elapsed() {
                if elapsed > max_age {
                    return Ok(false);
                }
            }
        }

        // Check file modification time
        if let Some(cached_mtime) = cached.file_mtime {
            if let Ok(current_mtime) = Self::get_file_mtime(path) {
                if current_mtime != cached_mtime {
                    return Ok(false);
                }
            } else {
                // File might have been deleted
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Get file modification time.
    fn get_file_mtime(path: &Path) -> std::io::Result<SystemTime> {
        let metadata = fs::metadata(path)?;
        metadata.modified()
    }

    /// Clear the cache.
    pub fn clear(&self) -> Result<()> {
        let mut cache = self.inner.write().map_err(|e| {
            PromptError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("cache lock error: {}", e),
            ))
        })?;

        cache.clear();
        Ok(())
    }

    /// Remove a specific entry from the cache.
    pub fn remove(&self, path: impl AsRef<Path>) -> Result<()> {
        let mut cache = self.inner.write().map_err(|e| {
            PromptError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("cache lock error: {}", e),
            ))
        })?;

        cache.remove(&path.as_ref().to_path_buf());
        Ok(())
    }

    /// Get cache size.
    pub fn len(&self) -> usize {
        self.inner.read().map(|c| c.len()).unwrap_or(0)
    }

    /// Check if cache is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.read().map(|c| c.is_empty()).unwrap_or(true)
    }
}

impl Default for PromptCache {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_prompt_context() {
        let mut context = PromptContext::new();
        context.set("name", "World");
        context.set("greeting", "Hello");

        assert_eq!(context.get("name"), Some("World"));
        assert_eq!(context.get("greeting"), Some("Hello"));
        assert!(context.contains("name"));
        assert!(!context.contains("missing"));
    }

    #[test]
    fn test_template_from_str() {
        let template = PromptTemplate::from_string("Hello {{name}}!");
        assert_eq!(template.content(), "Hello {{name}}!");
        assert!(template.file_path().is_none());
    }

    #[test]
    fn test_template_load() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"Hello {{name}}!").unwrap();
        file.flush().unwrap();

        let template = PromptTemplate::load(file.path()).unwrap();
        assert_eq!(template.content(), "Hello {{name}}!");
        assert!(template.file_path().is_some());
    }

    #[test]
    fn test_template_render() {
        let template = PromptTemplate::from_string("Hello {{name}}!");
        let mut context = PromptContext::new();
        context.set("name", "World");

        let result = template.render(&context).unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_template_render_multiple() {
        let template = PromptTemplate::from_string("{{greeting}} {{name}}! Welcome to {{place}}.");
        let mut context = PromptContext::new();
        context.set("greeting", "Hello");
        context.set("name", "Alice");
        context.set("place", "Wonderland");

        let result = template.render(&context).unwrap();
        assert_eq!(result, "Hello Alice! Welcome to Wonderland.");
    }

    #[test]
    fn test_template_missing_placeholder() {
        let template = PromptTemplate::from_string("Hello {{name}}!");
        let context = PromptContext::new();

        // Non-strict mode: should succeed with empty replacement
        let result = template.render(&context).unwrap();
        assert_eq!(result, "Hello !");

        // Strict mode: should error
        let options = RenderOptions { strict: true, default_value: None };
        let result = template.render_with_options(&context, &options);
        assert!(result.is_err());
    }

    #[test]
    fn test_template_default_value() {
        let template = PromptTemplate::from_string("Hello {{name}}!");
        let context = PromptContext::new();

        let options = RenderOptions { strict: false, default_value: Some("stranger".to_string()) };

        let result = template.render_with_options(&context, &options).unwrap();
        assert_eq!(result, "Hello stranger!");
    }

    #[test]
    fn test_list_placeholders() {
        let template = PromptTemplate::from_string("{{greeting}} {{name}}! {{greeting}} again.");
        let placeholders = template.list_placeholders();

        assert_eq!(placeholders.len(), 2);
        assert!(placeholders.contains(&"greeting".to_string()));
        assert!(placeholders.contains(&"name".to_string()));
    }

    #[test]
    fn test_find_placeholders_edge_cases() {
        // Single braces should be ignored
        let template = PromptTemplate::from_string("Hello {name}!");
        assert_eq!(template.list_placeholders().len(), 0);

        // Nested braces
        let template = PromptTemplate::from_string("{{outer}}");
        let placeholders = template.list_placeholders();
        assert_eq!(placeholders, vec!["outer"]);

        // Empty placeholder
        let template = PromptTemplate::from_string("{{}}");
        assert_eq!(template.list_placeholders().len(), 0);

        // Whitespace in placeholder
        let template = PromptTemplate::from_string("{{ name }}");
        assert_eq!(template.list_placeholders(), vec!["name"]);
    }

    #[test]
    fn test_template_multiline() {
        let content = r#"
# Agent Prompt

Hello {{name}}!

Your task is to {{task}}.

Please complete this by {{deadline}}.
"#;
        let template = PromptTemplate::from_string(content);
        let mut context = PromptContext::new();
        context.set("name", "Agent");
        context.set("task", "analyze the code");
        context.set("deadline", "tomorrow");

        let result = template.render(&context).unwrap();
        assert!(result.contains("Hello Agent!"));
        assert!(result.contains("analyze the code"));
        assert!(result.contains("tomorrow"));
    }

    #[test]
    fn test_prompt_cache_load() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"Hello {{name}}!").unwrap();
        file.flush().unwrap();

        let cache = PromptCache::new();
        let template1 = cache.load(file.path()).unwrap();
        let template2 = cache.load(file.path()).unwrap();

        assert_eq!(template1.content(), template2.content());
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_prompt_cache_invalidation() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"Version 1").unwrap();
        file.flush().unwrap();

        let cache = PromptCache::new();
        let template1 = cache.load(file.path()).unwrap();
        assert_eq!(template1.content(), "Version 1");

        // Modify file
        file.write_all(b"Version 2").unwrap();
        file.flush().unwrap();

        // Should reload from file
        let template2 = cache.load(file.path()).unwrap();
        assert_eq!(template2.content(), "Version 2");
    }

    #[test]
    fn test_prompt_cache_clear() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"Test").unwrap();
        file.flush().unwrap();

        let cache = PromptCache::new();
        cache.load(file.path()).unwrap();
        assert_eq!(cache.len(), 1);

        cache.clear().unwrap();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_prompt_cache_remove() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        file1.write_all(b"File 1").unwrap();
        file2.write_all(b"File 2").unwrap();
        file1.flush().unwrap();
        file2.flush().unwrap();

        let cache = PromptCache::new();
        cache.load(file1.path()).unwrap();
        cache.load(file2.path()).unwrap();
        assert_eq!(cache.len(), 2);

        cache.remove(file1.path()).unwrap();
        assert_eq!(cache.len(), 1);
    }
}
