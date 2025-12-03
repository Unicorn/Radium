//! Advanced prompt processing features.
//!
//! Provides file content injection, prompt validation, and other advanced
//! processing capabilities for prompt templates.

use crate::prompts::{PromptContext, PromptError, PromptTemplate};
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

    /// Prompt validation error.
    #[error("prompt validation error: {0}")]
    Validation(String),
}

/// Result type for processing operations.
pub type Result<T> = std::result::Result<T, ProcessingError>;

/// Process a prompt template with file content injection.
///
/// Supports syntax like `agent[input:file1.md,file2.md]` to inject file contents.
///
/// # Arguments
///
/// * `template` - The prompt template to process
/// * `context` - The prompt context with variables
/// * `base_path` - Base path for resolving relative file paths
///
/// # Errors
///
/// Returns error if file injection fails or files cannot be read.
pub fn process_with_file_injection(
    template: &PromptTemplate,
    context: &PromptContext,
    base_path: Option<&Path>,
) -> Result<String> {
    let mut content = template.content().to_string();

    // Process file injection syntax: agent[input:file1.md,file2.md]
    content = inject_file_contents(&content, base_path)?;

    // Create a temporary template with processed content
    let processed_template = PromptTemplate::from_string(content);

    // Render with context
    processed_template
        .render(context)
        .map_err(|e| ProcessingError::Validation(e.to_string()))
}

/// Inject file contents into prompt template.
///
/// Finds patterns like `agent[input:file1.md,file2.md]` and replaces them
/// with the contents of the specified files.
fn inject_file_contents(content: &str, base_path: Option<&Path>) -> Result<String> {
    // Pattern: agent[input:file1.md,file2.md]
    // Using a simple string-based approach instead of regex
    let mut result = content.to_string();
    let mut start = 0;

    while let Some(open_pos) = result[start..].find("agent[input:") {
        let actual_pos = start + open_pos;
        let after_open = actual_pos + "agent[input:".len();

        // Find the closing bracket
        if let Some(close_pos) = result[after_open..].find(']') {
            let full_match_start = actual_pos;
            let full_match_end = after_open + close_pos + 1;
            let files_str = &result[after_open..after_open + close_pos];

            // Parse comma-separated file list
            let files: Vec<&str> = files_str.split(',').map(|s| s.trim()).collect();

            let mut injected_content = String::new();
            for file in files {
                if file.is_empty() {
                    continue;
                }

                let file_path = resolve_file_path(file, base_path)?;
                let file_content = fs::read_to_string(&file_path)
                    .map_err(|_| ProcessingError::FileNotFound(file_path.display().to_string()))?;

                injected_content.push_str(&format!("\n\n--- Content from {} ---\n\n", file));
                injected_content.push_str(&file_content);
                injected_content.push_str("\n\n--- End of content ---\n\n");
            }

            // Replace the match
            result.replace_range(full_match_start..full_match_end, &injected_content);
            start = full_match_start + injected_content.len();
        } else {
            break;
        }
    }

    Ok(result)
}

/// Resolve a file path relative to base_path or current directory.
fn resolve_file_path(file: &str, base_path: Option<&Path>) -> Result<PathBuf> {
    let path = PathBuf::from(file);

    // If absolute path, use as-is
    if path.is_absolute() {
        return Ok(path);
    }

    // If base_path provided, resolve relative to it
    if let Some(base) = base_path {
        let resolved = base.join(&path);
        if resolved.exists() {
            return Ok(resolved);
        }
    }

    // Try relative to current directory
    if let Ok(cwd) = std::env::current_dir() {
        let resolved = cwd.join(&path);
        if resolved.exists() {
            return Ok(resolved);
        }
    }

    // Return the path as-is (caller will handle file not found)
    Ok(path)
}

/// Validate a prompt template.
///
/// Checks for common issues like:
/// - Empty prompts
/// - Missing required placeholders
/// - Invalid syntax
///
/// # Arguments
///
/// * `template` - The template to validate
/// * `required_placeholders` - Optional list of required placeholders
///
/// # Errors
///
/// Returns error if validation fails.
pub fn validate_prompt(
    template: &PromptTemplate,
    required_placeholders: Option<&[&str]>,
) -> Result<()> {
    let content = template.content();

    // Check for empty prompt
    if content.trim().is_empty() {
        return Err(ProcessingError::Validation("prompt is empty".to_string()));
    }

    // Check for minimum length
    if content.trim().len() < 10 {
        return Err(ProcessingError::Validation(
            "prompt is too short (minimum 10 characters)".to_string(),
        ));
    }

    // Check for required placeholders
    if let Some(required) = required_placeholders {
        let found_placeholders = template.list_placeholders();
        for required_ph in required {
            if !found_placeholders.contains(&required_ph.to_string()) {
                return Err(ProcessingError::Validation(format!(
                    "missing required placeholder: {}",
                    required_ph
                )));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn test_inject_file_contents() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.md");
        fs::write(&test_file, "# Test Content\n\nThis is test content.").unwrap();

        let template_content = "Hello! agent[input:test.md]";
        let result = inject_file_contents(template_content, Some(temp_dir.path())).unwrap();

        assert!(result.contains("# Test Content"));
        assert!(result.contains("This is test content"));
    }

    #[test]
    fn test_inject_multiple_files() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("file1.md");
        let file2 = temp_dir.path().join("file2.md");
        fs::write(&file1, "Content 1").unwrap();
        fs::write(&file2, "Content 2").unwrap();

        let template_content = "agent[input:file1.md,file2.md]";
        let result = inject_file_contents(template_content, Some(temp_dir.path())).unwrap();

        assert!(result.contains("Content 1"));
        assert!(result.contains("Content 2"));
    }

    #[test]
    fn test_validate_prompt_empty() {
        let template = PromptTemplate::from_string("");
        let result = validate_prompt(&template, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_prompt_too_short() {
        let template = PromptTemplate::from_string("short");
        let result = validate_prompt(&template, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_prompt_valid() {
        let template = PromptTemplate::from_string("This is a valid prompt template with enough content.");
        let result = validate_prompt(&template, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_prompt_required_placeholder() {
        let template = PromptTemplate::from_string("Hello {{name}}!");
        let result = validate_prompt(&template, Some(&["name"]));
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_prompt_missing_required_placeholder() {
        let template = PromptTemplate::from_string("Hello!");
        let result = validate_prompt(&template, Some(&["name"]));
        assert!(result.is_err());
    }
}
