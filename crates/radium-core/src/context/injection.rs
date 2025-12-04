//! Context injection syntax parsing and execution.
//!
//! Supports two injection patterns:
//! - File injection: `agent[input:file1.md,file2.md]`
//! - Tail context: `agent[tail:50]`

use super::error::{ContextError, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Parsed injection directive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InjectionDirective {
    /// File input injection: `agent[input:file1.md,file2.md]`
    FileInput { files: Vec<PathBuf> },

    /// Tail context from previous output: `agent[tail:N]`
    TailContext { lines: usize },
}

impl InjectionDirective {
    /// Parses an injection directive from a string.
    ///
    /// # Arguments
    /// * `directive` - The directive string (e.g., "input:file1.md,file2.md" or "tail:50")
    ///
    /// # Returns
    /// The parsed injection directive
    ///
    /// # Errors
    /// Returns error if the syntax is invalid
    pub fn parse(directive: &str) -> Result<Self> {
        let parts: Vec<&str> = directive.splitn(2, ':').collect();

        if parts.len() != 2 {
            return Err(ContextError::InvalidSyntax(format!(
                "Expected 'type:value' format, got: {}",
                directive
            )));
        }

        let directive_type = parts[0].trim();
        let value = parts[1].trim();

        match directive_type {
            "input" => {
                let files = value.split(',').map(|s| PathBuf::from(s.trim())).collect();
                Ok(InjectionDirective::FileInput { files })
            }
            "tail" => {
                let lines = value.parse::<usize>().map_err(|e| {
                    ContextError::InvalidTailSize(format!("Invalid tail size: {}", e))
                })?;
                Ok(InjectionDirective::TailContext { lines })
            }
            _ => Err(ContextError::InvalidSyntax(format!(
                "Unknown directive type: {}",
                directive_type
            ))),
        }
    }

    /// Extracts injection directives from agent invocation string.
    ///
    /// # Arguments
    /// * `invocation` - The agent invocation string (e.g., "agent[input:file1.md]")
    ///
    /// # Returns
    /// A tuple of (agent_name, directives)
    ///
    /// # Example
    /// ```
    /// use radium_core::context::InjectionDirective;
    ///
    /// let (agent, directives) = InjectionDirective::extract_directives("architect[input:spec.md,plan.md]").unwrap();
    /// assert_eq!(agent, "architect");
    /// assert_eq!(directives.len(), 1);
    /// ```
    pub fn extract_directives(invocation: &str) -> Result<(String, Vec<Self>)> {
        // Find opening bracket
        let Some(bracket_start) = invocation.find('[') else {
            // No directives, just return agent name
            return Ok((invocation.to_string(), vec![]));
        };

        let agent_name = invocation[..bracket_start].trim().to_string();

        // Find closing bracket
        let Some(bracket_end) = invocation.find(']') else {
            return Err(ContextError::InvalidSyntax(format!(
                "Missing closing bracket in: {}",
                invocation
            )));
        };

        let directives_str = &invocation[bracket_start + 1..bracket_end];

        // Parse directives (could be multiple separated by ';')
        let mut directives = Vec::new();
        for directive_str in directives_str.split(';') {
            let directive = Self::parse(directive_str.trim())?;
            directives.push(directive);
        }

        Ok((agent_name, directives))
    }
}

/// Context injector that executes injection directives.
pub struct ContextInjector {
    /// Base directory for resolving relative file paths.
    base_dir: PathBuf,
}

impl ContextInjector {
    /// Creates a new context injector.
    ///
    /// # Arguments
    /// * `base_dir` - Base directory for resolving relative paths
    pub fn new(base_dir: impl AsRef<Path>) -> Self {
        Self { base_dir: base_dir.as_ref().to_path_buf() }
    }

    /// Injects file contents.
    ///
    /// # Arguments
    /// * `files` - Paths to files to inject
    ///
    /// # Returns
    /// Combined file contents with file headers
    ///
    /// # Errors
    /// Returns error if any file cannot be read
    pub fn inject_files(&self, files: &[PathBuf]) -> Result<String> {
        let mut content = String::new();

        for file in files {
            let path = if file.is_absolute() { file.clone() } else { self.base_dir.join(file) };

            if !path.exists() {
                return Err(ContextError::FileNotFound(path.display().to_string()));
            }

            let file_content = fs::read_to_string(&path)?;

            // Add file header
            content.push_str(&format!("\n=== {} ===\n", path.display()));
            content.push_str(&file_content);
            content.push_str("\n\n");
        }

        Ok(content)
    }

    /// Extracts tail lines from text.
    ///
    /// # Arguments
    /// * `text` - The text to extract from
    /// * `lines` - Number of lines to extract
    ///
    /// # Returns
    /// The last N lines of text
    pub fn extract_tail(&self, text: &str, lines: usize) -> String {
        text.lines()
            .rev()
            .take(lines)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_parse_file_input_single() {
        let directive = InjectionDirective::parse("input:file1.md").unwrap();
        match directive {
            InjectionDirective::FileInput { files } => {
                assert_eq!(files.len(), 1);
                assert_eq!(files[0], PathBuf::from("file1.md"));
            }
            _ => panic!("Expected FileInput directive"),
        }
    }

    #[test]
    fn test_parse_file_input_multiple() {
        let directive = InjectionDirective::parse("input:file1.md,file2.md,file3.md").unwrap();
        match directive {
            InjectionDirective::FileInput { files } => {
                assert_eq!(files.len(), 3);
                assert_eq!(files[0], PathBuf::from("file1.md"));
                assert_eq!(files[1], PathBuf::from("file2.md"));
                assert_eq!(files[2], PathBuf::from("file3.md"));
            }
            _ => panic!("Expected FileInput directive"),
        }
    }

    #[test]
    fn test_parse_tail_context() {
        let directive = InjectionDirective::parse("tail:50").unwrap();
        match directive {
            InjectionDirective::TailContext { lines } => {
                assert_eq!(lines, 50);
            }
            _ => panic!("Expected TailContext directive"),
        }
    }

    #[test]
    fn test_parse_invalid_syntax() {
        let result = InjectionDirective::parse("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_tail_size() {
        let result = InjectionDirective::parse("tail:invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_directives_single() {
        let (agent, directives) =
            InjectionDirective::extract_directives("architect[input:spec.md]").unwrap();
        assert_eq!(agent, "architect");
        assert_eq!(directives.len(), 1);
    }

    #[test]
    fn test_extract_directives_multiple() {
        let (agent, directives) =
            InjectionDirective::extract_directives("architect[input:spec.md;tail:20]").unwrap();
        assert_eq!(agent, "architect");
        assert_eq!(directives.len(), 2);
    }

    #[test]
    fn test_extract_directives_none() {
        let (agent, directives) = InjectionDirective::extract_directives("architect").unwrap();
        assert_eq!(agent, "architect");
        assert_eq!(directives.len(), 0);
    }

    #[test]
    fn test_extract_directives_missing_bracket() {
        let result = InjectionDirective::extract_directives("architect[input:spec.md");
        assert!(result.is_err());
    }

    #[test]
    fn test_inject_files() {
        let temp_dir = TempDir::new().unwrap();
        let file1_path = temp_dir.path().join("file1.txt");
        let file2_path = temp_dir.path().join("file2.txt");

        let mut file1 = File::create(&file1_path).unwrap();
        file1.write_all(b"Content 1").unwrap();

        let mut file2 = File::create(&file2_path).unwrap();
        file2.write_all(b"Content 2").unwrap();

        let injector = ContextInjector::new(temp_dir.path());
        let content = injector
            .inject_files(&[PathBuf::from("file1.txt"), PathBuf::from("file2.txt")])
            .unwrap();

        assert!(content.contains("Content 1"));
        assert!(content.contains("Content 2"));
        assert!(content.contains("file1.txt"));
        assert!(content.contains("file2.txt"));
    }

    #[test]
    fn test_inject_files_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let injector = ContextInjector::new(temp_dir.path());
        let result = injector.inject_files(&[PathBuf::from("nonexistent.txt")]);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_tail() {
        let injector = ContextInjector::new(".");
        let text = "line1\nline2\nline3\nline4\nline5";
        let tail = injector.extract_tail(text, 3);
        assert_eq!(tail, "line3\nline4\nline5");
    }

    #[test]
    fn test_extract_tail_more_than_available() {
        let injector = ContextInjector::new(".");
        let text = "line1\nline2";
        let tail = injector.extract_tail(text, 10);
        assert_eq!(tail, "line1\nline2");
    }
}
