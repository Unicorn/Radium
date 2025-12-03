//! Advanced prompt processing features.
//!
//! Provides file content injection and other advanced prompt processing capabilities.

use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Prompt processing errors.
#[derive(Debug, Error)]
pub enum ProcessingError {
    /// File not found.
    #[error("file not found: {0}")]
    FileNotFound(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid injection syntax.
    #[error("invalid injection syntax: {0}")]
    InvalidSyntax(String),

    /// Missing base path for relative file resolution.
    #[error("missing base path for relative file resolution")]
    MissingBasePath,
}

/// Result type for processing operations.
pub type Result<T> = std::result::Result<T, ProcessingError>;

/// File injection syntax parser and processor.
///
/// Supports syntax like:
/// - `agent[input:file1.md,file2.md]` - Inject file contents
/// - `agent[tail:50]` - Inject last N lines from memory
pub struct FileInjector {
    /// Base path for resolving relative file paths.
    base_path: Option<PathBuf>,
}

impl FileInjector {
    /// Create a new file injector.
    pub fn new() -> Self {
        Self { base_path: None }
    }

    /// Create a new file injector with a base path for relative resolution.
    pub fn with_base_path(base_path: impl AsRef<Path>) -> Self {
        Self { base_path: Some(base_path.as_ref().to_path_buf()) }
    }

    /// Process file injection syntax in a prompt template.
    ///
    /// Replaces patterns like `agent[input:file1.md,file2.md]` with file contents.
    ///
    /// # Syntax
    ///
    /// - `agent[input:file1.md,file2.md]` - Inject contents of files
    /// - `agent[tail:N]` - Inject last N lines (for memory/context)
    ///
    /// # Errors
    ///
    /// Returns error if files cannot be read or syntax is invalid.
    pub fn process(&self, content: &str) -> Result<String> {
        let mut result = content.to_string();

        // Find all injection patterns: agent[input:...] or agent[tail:N]
        let patterns = Self::find_injection_patterns(&result);

        for pattern in patterns {
            let replacement = self.process_injection(&pattern)?;
            result = result.replace(&pattern.original, &replacement);
        }

        Ok(result)
    }

    /// Find all injection patterns in content.
    fn find_injection_patterns(content: &str) -> Vec<InjectionPattern> {
        let mut patterns = Vec::new();
        let mut chars = content.chars().peekable();
        let mut i = 0;

        while let Some(c) = chars.next() {
            if c == '[' {
                let start = i;
                let mut bracket_content = String::new();
                let mut depth = 1;

                // Read until matching closing bracket
                while let Some(c) = chars.next() {
                    i += 1;
                    match c {
                        '[' => depth += 1,
                        ']' => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        _ => {}
                    }
                    bracket_content.push(c);
                }

                if depth == 0 {
                    // Check if this looks like an injection pattern
                    if let Some(pattern) = Self::parse_injection_pattern(&bracket_content, start) {
                        patterns.push(pattern);
                    }
                }
            }
            i += 1;
        }

        patterns
    }

    /// Parse an injection pattern from bracket content.
    fn parse_injection_pattern(content: &str, start_pos: usize) -> Option<InjectionPattern> {
        // Look for patterns like: input:file1.md,file2.md or tail:50
        if content.starts_with("input:") {
            let files_str = &content[6..]; // Skip "input:"
            let files: Vec<String> = files_str.split(',').map(|s| s.trim().to_string()).collect();
            if !files.is_empty() {
                return Some(InjectionPattern {
                    original: format!("[{}]", content),
                    injection_type: InjectionType::Input(files),
                });
            }
        } else if content.starts_with("tail:") {
            let tail_str = &content[5..]; // Skip "tail:"
            if let Ok(lines) = tail_str.trim().parse::<usize>() {
                return Some(InjectionPattern {
                    original: format!("[{}]", content),
                    injection_type: InjectionType::Tail(lines),
                });
            }
        }

        None
    }

    /// Process a single injection pattern.
    fn process_injection(&self, pattern: &InjectionPattern) -> Result<String> {
        match &pattern.injection_type {
            InjectionType::Input(files) => self.inject_files(files),
            InjectionType::Tail(lines) => self.inject_tail(*lines),
        }
    }

    /// Inject file contents.
    fn inject_files(&self, files: &[String]) -> Result<String> {
        let mut contents = Vec::new();

        for file in files {
            let path = self.resolve_path(file)?;
            if !path.exists() {
                return Err(ProcessingError::FileNotFound(path.display().to_string()));
            }

            let content = fs::read_to_string(&path)?;
            contents.push(format!("--- File: {} ---\n{}\n", file, content));
        }

        Ok(contents.join("\n"))
    }

    /// Inject tail content (placeholder for memory system integration).
    fn inject_tail(&self, _lines: usize) -> Result<String> {
        // TODO: Integrate with memory system when Step 5 is implemented
        // For now, return a placeholder
        Ok(format!("[Tail context: last {} lines - memory system integration pending]", _lines))
    }

    /// Resolve a file path (relative or absolute).
    fn resolve_path(&self, file: &str) -> Result<PathBuf> {
        let path = PathBuf::from(file);

        if path.is_absolute() {
            Ok(path)
        } else if let Some(ref base) = self.base_path {
            Ok(base.join(&path))
        } else {
            // Try current directory
            Ok(std::env::current_dir()
                .map_err(|_| ProcessingError::MissingBasePath)?
                .join(&path))
        }
    }
}

impl Default for FileInjector {
    fn default() -> Self {
        Self::new()
    }
}

/// Injection pattern found in content.
#[derive(Debug, Clone)]
struct InjectionPattern {
    /// Original pattern text (e.g., "[input:file1.md]")
    original: String,
    /// Parsed injection type
    injection_type: InjectionType,
}

/// Type of injection.
#[derive(Debug, Clone)]
enum InjectionType {
    /// Inject file contents: `input:file1.md,file2.md`
    Input(Vec<String>),
    /// Inject tail content: `tail:N`
    Tail(usize),
}

/// Process a prompt template with file injection.
///
/// This is a convenience function that processes file injection syntax
/// in a prompt template.
///
/// # Example
///
/// ```rust,no_run
/// use radium_core::prompts::{PromptTemplate, processing::process_file_injection};
/// use std::path::Path;
///
/// # fn main() -> anyhow::Result<()> {
/// let template = PromptTemplate::load(Path::new("prompts/test.md"))?;
/// let processed = process_file_injection(template.content(), Some(Path::new(".")))?;
/// # Ok(())
/// # }
/// ```
pub fn process_file_injection(
    content: &str,
    base_path: Option<&Path>,
) -> std::result::Result<String, ProcessingError> {
    let injector = if let Some(base) = base_path {
        FileInjector::with_base_path(base)
    } else {
        FileInjector::new()
    };

    injector.process(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn test_find_input_injection() {
        let content = "agent[input:file1.md,file2.md]";
        let patterns = FileInjector::find_injection_patterns(content);
        assert_eq!(patterns.len(), 1);
        assert!(matches!(patterns[0].injection_type, InjectionType::Input(_)));
    }

    #[test]
    fn test_find_tail_injection() {
        let content = "agent[tail:50]";
        let patterns = FileInjector::find_injection_patterns(content);
        assert_eq!(patterns.len(), 1);
        assert!(matches!(patterns[0].injection_type, InjectionType::Tail(50)));
    }

    #[test]
    fn test_inject_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("test.md");
        fs::write(&file, "Test content").unwrap();

        let injector = FileInjector::with_base_path(temp_dir.path());
        let content = "agent[input:test.md]";
        let result = injector.process(content).unwrap();

        assert!(result.contains("Test content"));
        assert!(result.contains("File: test.md"));
    }

    #[test]
    fn test_inject_multiple_files() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("file1.md");
        let file2 = temp_dir.path().join("file2.md");
        fs::write(&file1, "Content 1").unwrap();
        fs::write(&file2, "Content 2").unwrap();

        let injector = FileInjector::with_base_path(temp_dir.path());
        let content = "agent[input:file1.md,file2.md]";
        let result = injector.process(content).unwrap();

        assert!(result.contains("Content 1"));
        assert!(result.contains("Content 2"));
    }

    #[test]
    fn test_inject_tail() {
        let injector = FileInjector::new();
        let content = "agent[tail:50]";
        let result = injector.process(content).unwrap();

        assert!(result.contains("Tail context"));
        assert!(result.contains("50"));
    }

    #[test]
    fn test_process_file_injection_function() {
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("test.md");
        fs::write(&file, "Test").unwrap();

        let content = "agent[input:test.md]";
        let result = process_file_injection(content, Some(temp_dir.path())).unwrap();

        assert!(result.contains("Test"));
    }

    #[test]
    fn test_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let injector = FileInjector::with_base_path(temp_dir.path());
        let content = "agent[input:nonexistent.md]";

        let result = injector.process(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_injections() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("file1.md");
        let file2 = temp_dir.path().join("file2.md");
        fs::write(&file1, "Content 1").unwrap();
        fs::write(&file2, "Content 2").unwrap();

        let injector = FileInjector::with_base_path(temp_dir.path());
        let content = "First: agent[input:file1.md]\nSecond: agent[input:file2.md]";
        let result = injector.process(content).unwrap();

        assert!(result.contains("Content 1"));
        assert!(result.contains("Content 2"));
    }
}
