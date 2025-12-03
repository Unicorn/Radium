//! Advanced prompt processing features.
//!
//! Provides file content injection, caching, and other advanced processing
//! capabilities for prompt templates.

use crate::prompts::{PromptContext, PromptError, PromptTemplate, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

/// Prompt template cache entry.
#[derive(Debug, Clone)]
struct CacheEntry {
    template: PromptTemplate,
    loaded_at: SystemTime,
}

/// Prompt template cache.
///
/// Caches loaded templates to avoid repeated file I/O operations.
#[derive(Debug, Clone)]
pub struct PromptCache {
    cache: Arc<RwLock<HashMap<PathBuf, CacheEntry>>>,
    ttl: Option<Duration>,
}

impl PromptCache {
    /// Create a new cache with no TTL (templates cached indefinitely).
    pub fn new() -> Self {
        Self { cache: Arc::new(RwLock::new(HashMap::new())), ttl: None }
    }

    /// Create a new cache with a time-to-live (TTL).
    ///
    /// Templates will be evicted after the TTL expires.
    pub fn with_ttl(ttl: Duration) -> Self {
        Self { cache: Arc::new(RwLock::new(HashMap::new())), ttl: Some(ttl) }
    }

    /// Load a template, using cache if available.
    ///
    /// # Errors
    ///
    /// Returns error if template cannot be loaded.
    pub fn load(&self, path: impl AsRef<Path>) -> Result<PromptTemplate> {
        let path = path.as_ref().to_path_buf();

        // Check cache first
        {
            let cache = self.cache.read().map_err(|_| {
                PromptError::InvalidSyntax("Cache lock poisoned".to_string())
            })?;

            if let Some(entry) = cache.get(&path) {
                // Check if entry is still valid (if TTL is set)
                if let Some(ttl) = self.ttl {
                    if let Ok(elapsed) = entry.loaded_at.elapsed() {
                        if elapsed < ttl {
                            return Ok(entry.template.clone());
                        }
                    }
                } else {
                    // No TTL, return cached entry
                    return Ok(entry.template.clone());
                }
            }
        }

        // Cache miss or expired, load from file
        let template = PromptTemplate::load(&path)?;

        // Store in cache
        {
            let mut cache = self.cache.write().map_err(|_| {
                PromptError::InvalidSyntax("Cache lock poisoned".to_string())
            })?;

            cache.insert(
                path,
                CacheEntry { template: template.clone(), loaded_at: SystemTime::now() },
            );
        }

        Ok(template)
    }

    /// Clear the cache.
    pub fn clear(&self) -> Result<()> {
        let mut cache = self.cache.write().map_err(|_| {
            PromptError::InvalidSyntax("Cache lock poisoned".to_string())
        })?;
        cache.clear();
        Ok(())
    }

    /// Remove a specific template from the cache.
    pub fn evict(&self, path: impl AsRef<Path>) -> Result<()> {
        let mut cache = self.cache.write().map_err(|_| {
            PromptError::InvalidSyntax("Cache lock poisoned".to_string())
        })?;
        cache.remove(path.as_ref());
        Ok(())
    }

    /// Get cache statistics.
    pub fn stats(&self) -> Result<CacheStats> {
        let cache = self.cache.read().map_err(|_| {
            PromptError::InvalidSyntax("Cache lock poisoned".to_string())
        })?;

        Ok(CacheStats {
            size: cache.len(),
            ttl: self.ttl,
        })
    }
}

impl Default for PromptCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics.
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of cached templates.
    pub size: usize,
    /// Time-to-live for cached entries (if set).
    pub ttl: Option<Duration>,
}

/// File content injection options.
#[derive(Debug, Clone, Default)]
pub struct FileInjectionOptions {
    /// Base path for resolving relative file paths.
    pub base_path: Option<PathBuf>,
    /// Maximum file size to inject (in bytes).
    pub max_file_size: Option<usize>,
    /// Format for file content injection.
    pub format: FileInjectionFormat,
}

/// Format for file content injection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileInjectionFormat {
    /// Inject as plain text.
    Plain,
    /// Inject as code block with language.
    CodeBlock,
    /// Inject with markdown formatting.
    Markdown,
}

impl Default for FileInjectionFormat {
    fn default() -> Self {
        Self::Plain
    }
}

/// Process a prompt template with file content injection.
///
/// This function processes a prompt template and injects file contents
/// based on special syntax in the template or context.
///
/// # File Injection Syntax
///
/// Files can be injected using placeholders in the format:
/// - `{{file:path/to/file.md}}` - Inject file content
/// - `{{file:path/to/file.md:code}}` - Inject as code block
/// - `{{file:path/to/file.md:markdown}}` - Inject with markdown formatting
///
/// # Errors
///
/// Returns error if file cannot be read or template processing fails.
pub fn process_with_file_injection(
    template: &PromptTemplate,
    context: &PromptContext,
    options: &FileInjectionOptions,
) -> Result<String> {
    let mut result = template.content().to_string();

    // Find file injection placeholders
    let file_placeholders = find_file_placeholders(&result);

    for (placeholder, file_path, format) in file_placeholders {
        // Resolve file path
        let resolved_path = if file_path.is_absolute() {
            file_path.clone()
        } else if let Some(base) = &options.base_path {
            base.join(&file_path)
        } else if let Some(template_path) = template.file_path() {
            template_path.parent().map(|p| p.join(&file_path)).unwrap_or(file_path)
        } else {
            file_path.clone()
        };

        // Check file size limit
        if let Some(max_size) = options.max_file_size {
            if let Ok(metadata) = fs::metadata(&resolved_path) {
                if metadata.len() > max_size as u64 {
                    return Err(PromptError::InvalidSyntax(format!(
                        "File {} exceeds maximum size of {} bytes",
                        resolved_path.display(),
                        max_size
                    )));
                }
            }
        }

        // Read file content
        let content = fs::read_to_string(&resolved_path).map_err(|e| {
            PromptError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Failed to read file {}: {}", resolved_path.display(), e),
            ))
        })?;

        // Format content based on injection format
        let formatted_content = match format {
            FileInjectionFormat::Plain => content,
            FileInjectionFormat::CodeBlock => {
                let ext = resolved_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("text");
                format!("```{}\n{}\n```", ext, content)
            }
            FileInjectionFormat::Markdown => format!("\n---\n{}\n---\n", content),
        };

        // Replace placeholder
        result = result.replace(&placeholder, &formatted_content);
    }

    // Now render with context (this will handle regular placeholders)
    let processed_template = PromptTemplate::from_string(result);
    processed_template.render(context)
}

/// Find file injection placeholders in template content.
///
/// Returns a vector of (full_placeholder, file_path, format) tuples.
fn find_file_placeholders(content: &str) -> Vec<(String, PathBuf, FileInjectionFormat)> {
    let mut placeholders = Vec::new();
    let mut chars = content.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '{' && chars.peek() == Some(&'{') {
            chars.next(); // consume second {

            // Read until we find }}
            let mut placeholder = String::new();
            let mut found_end = false;

            while let Some(c) = chars.next() {
                if c == '}' && chars.peek() == Some(&'}') {
                    chars.next(); // consume second }
                    found_end = true;
                    break;
                }
                placeholder.push(c);
            }

            if found_end && placeholder.trim().starts_with("file:") {
                let placeholder = placeholder.trim();
                let parts: Vec<&str> = placeholder[5..].split(':').collect();
                if !parts.is_empty() {
                    let file_path = PathBuf::from(parts[0].trim());
                    let format = if parts.len() > 1 {
                        match parts[1].trim() {
                            "code" => FileInjectionFormat::CodeBlock,
                            "markdown" => FileInjectionFormat::Markdown,
                            _ => FileInjectionFormat::Plain,
                        }
                    } else {
                        FileInjectionFormat::Plain
                    };

                    placeholders.push((format!("{{{{{}}}}}", placeholder), file_path, format));
                }
            }
        }
    }

    placeholders
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn test_prompt_cache_load() {
        let cache = PromptCache::new();
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"Hello {{name}}!").unwrap();
        file.flush().unwrap();

        let template1 = cache.load(file.path()).unwrap();
        let template2 = cache.load(file.path()).unwrap();

        // Should be the same instance (from cache)
        assert_eq!(template1.content(), template2.content());
    }

    #[test]
    fn test_prompt_cache_clear() {
        let cache = PromptCache::new();
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"Hello {{name}}!").unwrap();
        file.flush().unwrap();

        cache.load(file.path()).unwrap();
        assert_eq!(cache.stats().unwrap().size, 1);

        cache.clear().unwrap();
        assert_eq!(cache.stats().unwrap().size, 0);
    }

    #[test]
    fn test_file_injection_plain() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "File content").unwrap();

        let template = PromptTemplate::from_string("Content: {{file:test.txt}}");
        let context = PromptContext::new();
        let options = FileInjectionOptions {
            base_path: Some(temp_dir.path().to_path_buf()),
            ..Default::default()
        };

        let result = process_with_file_injection(&template, &context, &options).unwrap();
        assert!(result.contains("File content"));
    }

    #[test]
    fn test_file_injection_code_block() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        fs::write(&file_path, "fn main() {}").unwrap();

        let template = PromptTemplate::from_string("Code: {{file:test.rs:code}}");
        let context = PromptContext::new();
        let options = FileInjectionOptions {
            base_path: Some(temp_dir.path().to_path_buf()),
            ..Default::default()
        };

        let result = process_with_file_injection(&template, &context, &options).unwrap();
        assert!(result.contains("```rs"));
        assert!(result.contains("fn main() {}"));
    }

    #[test]
    fn test_file_injection_with_placeholder() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "File content").unwrap();

        let template = PromptTemplate::from_string("Hello {{name}}! {{file:test.txt}}");
        let mut context = PromptContext::new();
        context.set("name", "World");
        let options = FileInjectionOptions {
            base_path: Some(temp_dir.path().to_path_buf()),
            ..Default::default()
        };

        let result = process_with_file_injection(&template, &context, &options).unwrap();
        assert!(result.contains("Hello World!"));
        assert!(result.contains("File content"));
    }
}
