//! Input format detection and parsing for batch processing.

use crate::batch::error::BatchError;
use serde_json::Value;

/// Supported input formats for batch processing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputFormat {
    /// Line-delimited format (one prompt per line).
    LineDelimited,
    /// JSON array format with optional context.
    JsonArray,
}

/// Detects the input format based on file content.
///
/// # Arguments
/// * `content` - File content as string
///
/// # Returns
/// Detected format or error if content is empty.
pub fn detect_format(content: &str) -> Result<InputFormat, BatchError> {
    let trimmed = content.trim();
    
    if trimmed.is_empty() {
        return Err(BatchError::InvalidConfig("File is empty".to_string()));
    }
    
    // Check if it starts with '[' which indicates JSON array
    if trimmed.starts_with('[') {
        Ok(InputFormat::JsonArray)
    } else {
        Ok(InputFormat::LineDelimited)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_format_line_delimited() {
        let content = "Prompt 1\nPrompt 2\nPrompt 3";
        assert_eq!(detect_format(content).unwrap(), InputFormat::LineDelimited);
    }

    #[test]
    fn test_detect_format_json_array() {
        let content = "[{\"prompt\": \"test\"}]";
        assert_eq!(detect_format(content).unwrap(), InputFormat::JsonArray);
    }

    #[test]
    fn test_detect_format_empty() {
        let content = "";
        assert!(detect_format(content).is_err());
    }
}

