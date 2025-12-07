//! Unified complete command implementation.
//!
//! This module provides the `rad complete` functionality that automatically
//! detects source type, fetches content, generates a plan, and executes it
//! without user intervention (YOLO mode).

use std::path::{Path, PathBuf};
use thiserror::Error;

/// Source type detected from user input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceType {
    /// Local file source with resolved path.
    LocalFile(PathBuf),
    /// Jira ticket ID (e.g., "RAD-42").
    JiraTicket(String),
    /// Braingrid requirement ID (e.g., "REQ-2025-001").
    BraingridReq(String),
    /// Invalid source format.
    Invalid,
}

/// Errors that can occur during source detection.
#[derive(Debug, Error)]
pub enum SourceDetectionError {
    /// Invalid source format.
    #[error("Invalid source format: {0}")]
    InvalidFormat(String),
}

/// Result type for source detection operations.
pub type SourceDetectionResult<T> = std::result::Result<T, SourceDetectionError>;

/// Detects the source type from user input.
///
/// Detection order:
/// 1. Local file: Check if path exists
/// 2. Jira ticket: Match pattern `^[A-Z]+-\d+$`
/// 3. Braingrid REQ: Match pattern `^REQ-\d{4}-\d{3,}$`
/// 4. Invalid: Return error for unmatched patterns
///
/// # Arguments
///
/// * `input` - The user input string to detect
///
/// # Returns
///
/// Returns `SourceType` if detection succeeds, or an error if the format is invalid.
pub fn detect_source(input: &str) -> SourceDetectionResult<SourceType> {
    // Trim whitespace
    let input = input.trim();

    // 1. Check if it's a local file path
    let path = Path::new(input);
    if path.exists() && path.is_file() {
        return Ok(SourceType::LocalFile(path.to_path_buf()));
    }

    // 2. Check if it matches Jira ticket pattern: ^[A-Z]+-\d+$
    if let Some(captures) = regex::Regex::new(r"^[A-Z]+-\d+$")
        .unwrap()
        .find(input)
    {
        if captures.as_str() == input {
            return Ok(SourceType::JiraTicket(input.to_string()));
        }
    }

    // 3. Check if it matches Braingrid REQ pattern: ^REQ-\d{4}-\d{3,}$
    if let Some(captures) = regex::Regex::new(r"^REQ-\d{4}-\d{3,}$")
        .unwrap()
        .find(input)
    {
        if captures.as_str() == input {
            return Ok(SourceType::BraingridReq(input.to_string()));
        }
    }

    // 4. Invalid format
    Err(SourceDetectionError::InvalidFormat(format!(
        "Could not detect source type from: {}. Expected a file path, Jira ticket (e.g., RAD-42), or Braingrid REQ (e.g., REQ-2025-001).",
        input
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_detect_local_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");
        std::fs::write(&file_path, "test content").unwrap();

        let result = detect_source(file_path.to_str().unwrap()).unwrap();
        match result {
            SourceType::LocalFile(path) => {
                assert_eq!(path, file_path);
            }
            _ => panic!("Expected LocalFile"),
        }
    }

    #[test]
    fn test_detect_jira_ticket() {
        let result = detect_source("RAD-42").unwrap();
        match result {
            SourceType::JiraTicket(id) => {
                assert_eq!(id, "RAD-42");
            }
            _ => panic!("Expected JiraTicket"),
        }
    }

    #[test]
    fn test_detect_braingrid_req() {
        let result = detect_source("REQ-2025-001").unwrap();
        match result {
            SourceType::BraingridReq(id) => {
                assert_eq!(id, "REQ-2025-001");
            }
            _ => panic!("Expected BraingridReq"),
        }
    }

    #[test]
    fn test_detect_invalid_format() {
        let result = detect_source("invalid-input");
        assert!(result.is_err());
        match result.unwrap_err() {
            SourceDetectionError::InvalidFormat(_) => {}
        }
    }

    #[test]
    fn test_detect_nonexistent_file() {
        // Should not match as local file if it doesn't exist
        let result = detect_source("./nonexistent.md");
        // Should try other patterns, likely fail or match something else
        assert!(result.is_err() || matches!(result.unwrap(), SourceType::Invalid));
    }

    #[test]
    fn test_detect_jira_variations() {
        assert!(matches!(
            detect_source("PROJ-123").unwrap(),
            SourceType::JiraTicket(_)
        ));
        assert!(matches!(
            detect_source("ABC-999").unwrap(),
            SourceType::JiraTicket(_)
        ));
    }

    #[test]
    fn test_detect_braingrid_variations() {
        assert!(matches!(
            detect_source("REQ-2024-001").unwrap(),
            SourceType::BraingridReq(_)
        ));
        assert!(matches!(
            detect_source("REQ-2025-1234").unwrap(),
            SourceType::BraingridReq(_)
        ));
    }
}

