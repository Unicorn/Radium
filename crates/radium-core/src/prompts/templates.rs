//! Prompt template loading and processing.
//!
//! Implements markdown-based prompt templates with placeholder replacement.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
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
#[derive(Debug, Clone, Default)]
pub struct RenderOptions {
    /// Strict mode: error if placeholder is missing.
    pub strict: bool,

    /// Default value for missing placeholders (only used if not strict).
    pub default_value: Option<String>,
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
        let content = r"
# Agent Prompt

Hello {{name}}!

Your task is to {{task}}.

Please complete this by {{deadline}}.
";
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
    fn test_context_remove() {
        let mut context = PromptContext::new();
        context.set("key1", "value1");
        context.set("key2", "value2");

        assert_eq!(context.remove("key1"), Some("value1".to_string()));
        assert!(context.get("key1").is_none());
        assert_eq!(context.get("key2"), Some("value2"));
    }

    #[test]
    fn test_context_clear() {
        let mut context = PromptContext::new();
        context.set("key1", "value1");
        context.set("key2", "value2");

        context.clear();
        assert!(context.get("key1").is_none());
        assert!(context.get("key2").is_none());
        assert!(!context.contains("key1"));
    }

    #[test]
    fn test_context_empty_key() {
        let mut context = PromptContext::new();
        context.set("", "empty key");

        assert_eq!(context.get(""), Some("empty key"));
        assert!(context.contains(""));
    }

    #[test]
    fn test_context_unicode() {
        let mut context = PromptContext::new();
        context.set("greeting", "こんにちは");
        context.set("emoji", "🎉");

        assert_eq!(context.get("greeting"), Some("こんにちは"));
        assert_eq!(context.get("emoji"), Some("🎉"));
    }

    #[test]
    fn test_template_load_not_found() {
        let result = PromptTemplate::load("/nonexistent/path/template.md");
        assert!(result.is_err());
        match result {
            Err(PromptError::NotFound(_)) => (),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_template_consecutive_placeholders() {
        let template = PromptTemplate::from_string("{{first}}{{second}}{{third}}");
        let mut context = PromptContext::new();
        context.set("first", "A");
        context.set("second", "B");
        context.set("third", "C");

        let result = template.render(&context).unwrap();
        assert_eq!(result, "ABC");
    }

    #[test]
    fn test_template_placeholders_at_boundaries() {
        let template = PromptTemplate::from_string("{{start}} middle {{end}}");
        let mut context = PromptContext::new();
        context.set("start", "Beginning");
        context.set("end", "Finish");

        let result = template.render(&context).unwrap();
        assert_eq!(result, "Beginning middle Finish");
    }

    #[test]
    fn test_template_placeholder_only() {
        let template = PromptTemplate::from_string("{{content}}");
        let mut context = PromptContext::new();
        context.set("content", "Complete replacement");

        let result = template.render(&context).unwrap();
        assert_eq!(result, "Complete replacement");
    }

    #[test]
    fn test_template_no_placeholders() {
        let template = PromptTemplate::from_string("This is plain text with no placeholders.");
        let context = PromptContext::new();

        let result = template.render(&context).unwrap();
        assert_eq!(result, "This is plain text with no placeholders.");
    }

    #[test]
    fn test_template_unclosed_placeholder() {
        let template = PromptTemplate::from_string("Hello {{name! Missing closing braces");
        let placeholders = template.list_placeholders();
        assert_eq!(placeholders.len(), 0);
    }

    #[test]
    fn test_template_triple_braces() {
        let template = PromptTemplate::from_string("{{{name}}}");
        let placeholders = template.list_placeholders();
        // Parser finds {{ and reads until }}, so it captures {name (without closing brace)
        assert_eq!(placeholders.len(), 1);
        assert_eq!(placeholders[0], "{name");
    }

    #[test]
    fn test_template_mixed_valid_invalid() {
        let template = PromptTemplate::from_string("{{valid}} {invalid} {{another}}");
        let placeholders = template.list_placeholders();
        assert_eq!(placeholders.len(), 2);
        assert!(placeholders.contains(&"valid".to_string()));
        assert!(placeholders.contains(&"another".to_string()));
    }

    #[test]
    fn test_template_render_empty_context() {
        let template = PromptTemplate::from_string("Hello {{name}}! Age: {{age}}");
        let context = PromptContext::new();

        let result = template.render(&context).unwrap();
        assert_eq!(result, "Hello ! Age: ");
    }

    #[test]
    fn test_template_partial_replacement() {
        let template = PromptTemplate::from_string("{{a}} {{b}} {{c}}");
        let mut context = PromptContext::new();
        context.set("a", "first");
        context.set("c", "third");

        let result = template.render(&context).unwrap();
        assert_eq!(result, "first  third");
    }

    #[test]
    fn test_template_same_placeholder_multiple_times() {
        let template = PromptTemplate::from_string("{{name}} and {{name}} and {{name}}");
        let mut context = PromptContext::new();
        context.set("name", "Alice");

        let result = template.render(&context).unwrap();
        assert_eq!(result, "Alice and Alice and Alice");
    }

    #[test]
    fn test_template_special_characters_in_value() {
        let template = PromptTemplate::from_string("Code: {{code}}");
        let mut context = PromptContext::new();
        context.set("code", "fn main() { println!(\"Hello\"); }");

        let result = template.render(&context).unwrap();
        assert!(result.contains("fn main()"));
        assert!(result.contains("println!"));
    }

    #[test]
    fn test_render_options_strict_mode() {
        let template = PromptTemplate::from_string("{{required}}");
        let context = PromptContext::new();

        let options = RenderOptions { strict: true, default_value: None };
        let result = template.render_with_options(&context, &options);
        assert!(result.is_err());
    }

    #[test]
    fn test_render_options_default_value_overrides_empty() {
        let template = PromptTemplate::from_string("{{missing}}");
        let context = PromptContext::new();

        let options = RenderOptions { strict: false, default_value: Some("DEFAULT".to_string()) };
        let result = template.render_with_options(&context, &options).unwrap();
        assert_eq!(result, "DEFAULT");
    }

    #[test]
    fn test_context_overwrite_value() {
        let mut context = PromptContext::new();
        context.set("key", "first");
        context.set("key", "second");

        assert_eq!(context.get("key"), Some("second"));
    }

    #[test]
    fn test_template_file_path_preserved() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"content").unwrap();
        file.flush().unwrap();

        let template = PromptTemplate::load(file.path()).unwrap();
        assert!(template.file_path().is_some());
        assert_eq!(template.file_path().unwrap(), file.path());
    }

    #[test]
    fn test_template_empty_content() {
        let template = PromptTemplate::from_string("");
        let context = PromptContext::new();

        let result = template.render(&context).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_find_placeholders_newlines() {
        let template = PromptTemplate::from_string("Line 1: {{first}}\nLine 2: {{second}}");
        let placeholders = template.list_placeholders();
        assert_eq!(placeholders.len(), 2);
        assert!(placeholders.contains(&"first".to_string()));
        assert!(placeholders.contains(&"second".to_string()));
    }

    #[test]
    fn test_nested_placeholder_syntax() {
        // Test deeply nested braces (should not be treated as nested placeholders)
        let template = PromptTemplate::from_string("{{{{VAR}}}}");
        let placeholders = template.list_placeholders();
        // Should find {{VAR}} as a placeholder (outer braces are part of syntax)
        assert_eq!(placeholders.len(), 1);
        assert_eq!(placeholders[0], "{VAR");
    }

    #[test]
    fn test_placeholder_with_special_characters() {
        // Test placeholders with special characters in names
        let template = PromptTemplate::from_string("{{user-name}} {{user_email}} {{user.name}}");
        let placeholders = template.list_placeholders();
        assert_eq!(placeholders.len(), 3);
        assert!(placeholders.contains(&"user-name".to_string()));
        assert!(placeholders.contains(&"user_email".to_string()));
        assert!(placeholders.contains(&"user.name".to_string()));
    }

    #[test]
    fn test_empty_placeholder_name() {
        // Test empty placeholder {{}}
        let template = PromptTemplate::from_string("Hello {{}} world");
        let placeholders = template.list_placeholders();
        // Should handle empty placeholder name
        assert!(placeholders.is_empty() || placeholders.contains(&"".to_string()));
    }

    #[test]
    fn test_placeholder_with_whitespace() {
        // Test placeholders with whitespace {{ KEY }}
        let template = PromptTemplate::from_string("Hello {{ name }} world");
        let placeholders = template.list_placeholders();
        assert_eq!(placeholders.len(), 1);
        // Whitespace should be trimmed
        assert_eq!(placeholders[0], "name");
    }

    #[test]
    fn test_strict_mode_missing_placeholder() {
        let template = PromptTemplate::from_string("{{required}} {{optional}}");
        let mut context = PromptContext::new();
        context.set("required", "value");

        let options = RenderOptions { strict: true, default_value: None };
        let result = template.render_with_options(&context, &options);
        // Should fail in strict mode when optional is missing
        assert!(result.is_err());
    }

    #[test]
    fn test_placeholder_at_start_and_end() {
        let template = PromptTemplate::from_string("{{start}} middle {{end}}");
        let mut context = PromptContext::new();
        context.set("start", "BEGIN");
        context.set("end", "END");

        let result = template.render(&context).unwrap();
        assert_eq!(result, "BEGIN middle END");
    }
}
