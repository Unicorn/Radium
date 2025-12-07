//! Agent prompt template linter.
//!
//! Analyzes prompt templates for structure, completeness, and best practices.

use std::path::Path;
use thiserror::Error;

/// Linting errors and warnings.
#[derive(Debug, Error)]
pub enum LintError {
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Missing required section.
    #[error("missing required section: {0}")]
    MissingSection(String),

    /// Section content too short.
    #[error("section '{0}' content too short (minimum {1} characters, found {2})")]
    SectionTooShort(String, usize, usize),
}

/// Lint result for a prompt template.
#[derive(Debug, Clone)]
pub struct LintResult {
    /// Whether the prompt passed all linting rules.
    pub valid: bool,
    /// List of errors found.
    pub errors: Vec<String>,
    /// List of warnings found.
    pub warnings: Vec<String>,
}

impl LintResult {
    /// Creates a new lint result.
    pub fn new() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Adds an error.
    pub fn add_error(&mut self, error: String) {
        self.valid = false;
        self.errors.push(error);
    }

    /// Adds a warning.
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

impl Default for LintResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Agent linter trait.
pub trait AgentLinter {
    /// Lints a prompt template file.
    fn lint(&self, path: impl AsRef<Path>) -> Result<LintResult, LintError>;
}

/// Prompt template linter.
pub struct PromptLinter {
    /// Minimum content length per section (default: 50).
    min_section_length: usize,
    /// Whether to check for examples (default: true).
    check_examples: bool,
}

impl PromptLinter {
    /// Creates a new prompt linter with default settings.
    pub fn new() -> Self {
        Self {
            min_section_length: 50,
            check_examples: true,
        }
    }

    /// Sets the minimum section length.
    pub fn with_min_section_length(mut self, length: usize) -> Self {
        self.min_section_length = length;
        self
    }

    /// Sets whether to check for examples.
    pub fn with_check_examples(mut self, check: bool) -> Self {
        self.check_examples = check;
        self
    }

    /// Extracts a section from markdown content.
    fn extract_section(content: &str, section_name: &str) -> Option<String> {
        let pattern = format!("## {}", section_name);
        let pattern_alt = format!("### {}", section_name);

        if let Some(start) = content.find(&pattern) {
            let section_start = start + pattern.len();
            // Find the next ## heading or end of content
            let remaining = &content[section_start..];
            let end = remaining
                .find("\n## ")
                .or_else(|| remaining.find("\n### "))
                .unwrap_or(remaining.len());

            Some(remaining[..end].trim().to_string())
        } else if let Some(start) = content.find(&pattern_alt) {
            let section_start = start + pattern_alt.len();
            let remaining = &content[section_start..];
            let end = remaining
                .find("\n## ")
                .or_else(|| remaining.find("\n### "))
                .unwrap_or(remaining.len());

            Some(remaining[..end].trim().to_string())
        } else {
            None
        }
    }

    /// Checks if a section exists and has sufficient content.
    fn check_section(
        &self,
        content: &str,
        section_name: &str,
        required: bool,
        result: &mut LintResult,
    ) {
        if let Some(section_content) = Self::extract_section(content, section_name) {
            let content_len = section_content.len();
            if content_len < self.min_section_length {
                if required {
                    result.add_error(format!(
                        "Section '{}' is too short (minimum {} characters, found {})",
                        section_name, self.min_section_length, content_len
                    ));
                } else {
                    result.add_warning(format!(
                        "Section '{}' is quite short ({} characters), consider adding more detail",
                        section_name, content_len
                    ));
                }
            }
        } else if required {
            result.add_error(format!("Missing required section: {}", section_name));
        }
    }
}

impl AgentLinter for PromptLinter {
    fn lint(&self, path: impl AsRef<Path>) -> Result<LintResult, LintError> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let mut result = LintResult::new();

        // Check for required sections
        self.check_section(&content, "Role", true, &mut result);
        self.check_section(&content, "Capabilities", true, &mut result);
        self.check_section(&content, "Input", true, &mut result);
        self.check_section(&content, "Output", true, &mut result);
        self.check_section(&content, "Instructions", true, &mut result);

        // Check for optional but recommended sections
        self.check_section(&content, "Examples", false, &mut result);
        self.check_section(&content, "Notes", false, &mut result);

        // Check for examples if enabled
        if self.check_examples {
            if Self::extract_section(&content, "Examples").is_none() {
                result.add_warning(
                    "No Examples section found. Examples help clarify agent behavior.".to_string(),
                );
            }
        }

        Ok(result)
    }
}

impl Default for PromptLinter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_lint_valid_prompt() {
        let content = r#"# Test Agent

## Role

You are a test agent with a comprehensive role description that provides clear guidance on what the agent should do and how it should behave in various scenarios.

## Capabilities

- Capability 1: Detailed description of what this capability enables
- Capability 2: Another capability with sufficient detail to meet minimum requirements
- Capability 3: Third capability description

## Input

This section describes the inputs the agent expects, including context, parameters, and any required data structures or formats.

## Output

This section describes what the agent produces, including output formats, key deliverables, and success criteria.

## Instructions

Step-by-step instructions for the agent to follow, including detailed guidance on how to process inputs and generate outputs.

## Examples

### Example 1

Sample example content here.
"#;

        let file = NamedTempFile::new().unwrap();
        std::fs::write(file.path(), content).unwrap();

        let linter = PromptLinter::new();
        let result = linter.lint(file.path()).unwrap();

        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_lint_missing_section() {
        let content = r#"# Test Agent

## Role

Role description here.
"#;

        let file = NamedTempFile::new().unwrap();
        std::fs::write(file.path(), content).unwrap();

        let linter = PromptLinter::new();
        let result = linter.lint(file.path()).unwrap();

        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("Capabilities")));
    }

    #[test]
    fn test_lint_short_section() {
        let content = r#"# Test Agent

## Role

Short.

## Capabilities

- Cap 1

## Input

Input here with sufficient content to meet minimum requirements.

## Output

Output description with enough detail to pass validation.

## Instructions

Instructions with sufficient content.
"#;

        let file = NamedTempFile::new().unwrap();
        std::fs::write(file.path(), content).unwrap();

        let linter = PromptLinter::new();
        let result = linter.lint(file.path()).unwrap();

        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("Role") && e.contains("too short")));
    }
}

