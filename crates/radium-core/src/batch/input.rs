//! Input parsing for batch processing.

use crate::batch::error::BatchError;
use crate::batch::formats::{detect_format, InputFormat};
use serde_json::Value;
use std::path::Path;

/// Represents a single batch input item.
#[derive(Debug, Clone)]
pub struct BatchInput {
    /// The prompt text.
    pub prompt: String,
    /// Optional context data (JSON value).
    pub context: Option<Value>,
}

/// Parse a batch input file.
///
/// Auto-detects format (line-delimited or JSON array) and parses accordingly.
///
/// # Arguments
/// * `file_path` - Path to the input file
///
/// # Returns
/// Vector of `BatchInput` items or error if parsing fails.
pub fn parse_input_file(file_path: &Path) -> Result<Vec<BatchInput>, BatchError> {
    // Check if file exists
    if !file_path.exists() {
        return Err(BatchError::InvalidConfig(format!(
            "File not found: {}",
            file_path.display()
        )));
    }

    // Read file content
    let content = std::fs::read_to_string(file_path).map_err(|e| {
        BatchError::InvalidConfig(format!(
            "Failed to read file {}: {}",
            file_path.display(),
            e
        ))
    })?;

    // Detect format
    let format = detect_format(&content)?;

    // Parse based on format
    match format {
        InputFormat::LineDelimited => parse_line_delimited(&content),
        InputFormat::JsonArray => parse_json_array(&content),
    }
}

/// Parse line-delimited format.
///
/// Each non-empty line becomes a prompt.
fn parse_line_delimited(content: &str) -> Result<Vec<BatchInput>, BatchError> {
    let mut inputs = Vec::new();

    for (_line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            inputs.push(BatchInput {
                prompt: trimmed.to_string(),
                context: None,
            });
        }
    }

    if inputs.is_empty() {
        Err(BatchError::InvalidConfig(
            "No valid prompts found in file".to_string(),
        ))
    } else {
        Ok(inputs)
    }
}

/// Parse JSON array format.
///
/// Expected format:
/// ```json
/// [
///   {"prompt": "Prompt 1", "context": {"key": "value"}},
///   {"prompt": "Prompt 2"}
/// ]
/// ```
fn parse_json_array(content: &str) -> Result<Vec<BatchInput>, BatchError> {
    let json: Value = serde_json::from_str(content).map_err(|e| {
        BatchError::InvalidConfig(format!("Invalid JSON: {} (at position {})", e, e.line()))
    })?;

    let array = json.as_array().ok_or_else(|| {
        BatchError::InvalidConfig("JSON root must be an array".to_string())
    })?;

    let mut inputs = Vec::new();

    for (index, item) in array.iter().enumerate() {
        let obj = item.as_object().ok_or_else(|| {
            BatchError::InvalidConfig(format!(
                "Array item {} must be an object",
                index
            ))
        })?;

        let prompt = obj
            .get("prompt")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                BatchError::InvalidConfig(format!(
                    "Array item {} missing required 'prompt' field",
                    index
                ))
            })?;

        let context = obj.get("context").cloned();

        inputs.push(BatchInput {
            prompt: prompt.to_string(),
            context,
        });
    }

    if inputs.is_empty() {
        Err(BatchError::InvalidConfig(
            "JSON array is empty".to_string(),
        ))
    } else {
        Ok(inputs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_line_delimited() {
        let content = "Prompt 1\nPrompt 2\nPrompt 3\n\nPrompt 4";
        let result = parse_line_delimited(content).unwrap();
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].prompt, "Prompt 1");
        assert_eq!(result[1].prompt, "Prompt 2");
        assert!(result[0].context.is_none());
    }

    #[test]
    fn test_parse_json_array() {
        let content = r#"[
            {"prompt": "Prompt 1", "context": {"key": "value"}},
            {"prompt": "Prompt 2"}
        ]"#;
        let result = parse_json_array(content).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].prompt, "Prompt 1");
        assert!(result[0].context.is_some());
        assert_eq!(result[1].prompt, "Prompt 2");
        assert!(result[1].context.is_none());
    }

    #[test]
    fn test_parse_input_file_line_delimited() {
        let mut file = NamedTempFile::new().unwrap();
        use std::io::Write;
        writeln!(file, "Prompt 1").unwrap();
        writeln!(file, "Prompt 2").unwrap();
        file.flush().unwrap();

        let result = parse_input_file(file.path()).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_parse_input_file_json() {
        let mut file = NamedTempFile::new().unwrap();
        use std::io::Write;
        writeln!(file, r#"[{{"prompt": "Test"}}]"#).unwrap();
        file.flush().unwrap();

        let result = parse_input_file(file.path()).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_parse_input_file_not_found() {
        let path = Path::new("/nonexistent/file.txt");
        assert!(parse_input_file(path).is_err());
    }
}

