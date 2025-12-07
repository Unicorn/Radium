//! Unified complete command implementation.
//!
//! This module provides the `rad complete` functionality that automatically
//! detects source type, fetches content, generates a plan, and executes it
//! without user intervention (YOLO mode).

use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

use crate::context::{BraingridReader, JiraReader, LocalFileReader, SourceError, SourceReader};
use crate::planning::{PlanExecutor, PlanGenerator, ExecutionConfig};
use crate::workspace::{RequirementId, Workspace};
use crate::models::plan::{Plan, PlanManifest, Iteration, PlanTask};
use crate::planning::parser::ParsedPlan;
use radium_abstraction::Model;
use radium_models::ModelFactory;
use tokio::sync::mpsc;

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

/// Checks if input matches Jira ticket pattern: ^[A-Z]+-\d+$
fn is_jira_ticket(input: &str) -> bool {
    // Must have at least one uppercase letter, a dash, and at least one digit
    let parts: Vec<&str> = input.split('-').collect();
    if parts.len() != 2 {
        return false;
    }

    let prefix = parts[0];
    let suffix = parts[1];

    // Prefix must be all uppercase letters
    if prefix.is_empty() || !prefix.chars().all(|c| c.is_ascii_uppercase()) {
        return false;
    }

    // Suffix must be all digits
    suffix.chars().all(|c| c.is_ascii_digit())
}

/// Checks if input matches Braingrid REQ pattern: ^REQ-\d{4}-\d{3,}$
fn is_braingrid_req(input: &str) -> bool {
    // Must start with "REQ-"
    if !input.starts_with("REQ-") {
        return false;
    }

    let rest = &input[4..];
    let parts: Vec<&str> = rest.split('-').collect();
    if parts.len() != 2 {
        return false;
    }

    let year = parts[0];
    let number = parts[1];

    // Year must be exactly 4 digits
    if year.len() != 4 || !year.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    // Number must be 3 or more digits
    number.len() >= 3 && number.chars().all(|c| c.is_ascii_digit())
}

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
    if is_jira_ticket(input) {
        return Ok(SourceType::JiraTicket(input.to_string()));
    }

    // 3. Check if it matches Braingrid REQ pattern: ^REQ-\d{4}-\d{3,}$
    if is_braingrid_req(input) {
        return Ok(SourceType::BraingridReq(input.to_string()));
    }

    // 4. Invalid format
    Err(SourceDetectionError::InvalidFormat(format!(
        "Could not detect source type from: {}. Expected a file path, Jira ticket (e.g., RAD-42), or Braingrid REQ (e.g., REQ-2025-001).",
        input
    )))
}

/// Errors that can occur during source content fetching.
#[derive(Debug, Error)]
pub enum SourceFetchError {
    /// Source not found.
    #[error("Source not found: {0}")]
    NotFound(String),

    /// Missing credentials for authentication.
    #[error("Missing {0} credentials. Please run `rad auth login {1}`.")]
    MissingCredentials(String, String),

    /// Network or other error.
    #[error("Failed to fetch from {0}: {1}")]
    FetchError(String, String),
}

impl From<SourceError> for SourceFetchError {
    fn from(err: SourceError) -> Self {
        match err {
            SourceError::NotFound(msg) => SourceFetchError::NotFound(msg),
            SourceError::Unauthorized(msg) => {
                // Try to extract provider name from error message
                let provider = if msg.contains("Jira") || msg.contains("JIRA") {
                    ("Jira", "jira")
                } else if msg.contains("Braingrid") || msg.contains("BRAINGRID") {
                    ("Braingrid", "braingrid")
                } else {
                    ("credentials", "credentials")
                };
                SourceFetchError::MissingCredentials(provider.0.to_string(), provider.1.to_string())
            }
            SourceError::NetworkError(msg) => SourceFetchError::FetchError("network".to_string(), msg),
            SourceError::IoError(e) => SourceFetchError::FetchError("I/O".to_string(), e.to_string()),
            SourceError::InvalidUri(msg) => SourceFetchError::FetchError("invalid URI".to_string(), msg),
            SourceError::Other(msg) => SourceFetchError::FetchError("source".to_string(), msg),
        }
    }
}

/// Result type for source content fetching operations.
pub type SourceFetchResult<T> = std::result::Result<T, SourceFetchError>;

/// Fetches content from a detected source.
///
/// # Arguments
///
/// * `source` - The detected source type
///
/// # Returns
///
/// Returns the content as a string, or an error if fetching fails.
pub async fn fetch_source_content(source: SourceType) -> SourceFetchResult<String> {
    match source {
        SourceType::LocalFile(path) => {
            // Convert path to file:// URI
            let uri = if path.is_absolute() {
                format!("file://{}", path.display())
            } else {
                format!("file://{}", path.display())
            };

            let reader = LocalFileReader::new();
            reader.fetch(&uri).await.map_err(SourceFetchError::from)
        }
        SourceType::JiraTicket(ticket_id) => {
            // Convert ticket ID to jira:// URI
            let uri = format!("jira://{}", ticket_id);

            let reader = JiraReader::new();
            match reader.fetch(&uri).await {
                Ok(content) => Ok(content),
                Err(SourceError::Unauthorized(_)) => {
                    Err(SourceFetchError::MissingCredentials("Jira".to_string(), "jira".to_string()))
                }
                Err(SourceError::NotFound(msg)) => Err(SourceFetchError::NotFound(msg)),
                Err(e) => Err(SourceFetchError::from(e)),
            }
        }
        SourceType::BraingridReq(req_id) => {
            // Convert REQ ID to braingrid:// URI
            let uri = format!("braingrid://{}", req_id);

            let reader = BraingridReader::new();
            match reader.fetch(&uri).await {
                Ok(content) => Ok(content),
                Err(SourceError::Unauthorized(_)) => {
                    Err(SourceFetchError::MissingCredentials("Braingrid".to_string(), "braingrid".to_string()))
                }
                Err(SourceError::NotFound(msg)) => Err(SourceFetchError::NotFound(msg)),
                Err(e) => Err(SourceFetchError::from(e)),
            }
        }
        SourceType::Invalid => Err(SourceFetchError::FetchError(
            "invalid source".to_string(),
            "Source type is invalid".to_string(),
        )),
    }
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
        // Should try other patterns, likely fail
        assert!(result.is_err());
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

    #[test]
    fn test_is_jira_ticket() {
        assert!(is_jira_ticket("RAD-42"));
        assert!(is_jira_ticket("PROJ-123"));
        assert!(is_jira_ticket("ABC-999"));
        assert!(!is_jira_ticket("rad-42")); // lowercase
        assert!(!is_jira_ticket("RAD-42-EXTRA")); // too many parts
        assert!(!is_jira_ticket("RAD")); // no dash
    }

    #[test]
    fn test_is_braingrid_req() {
        assert!(is_braingrid_req("REQ-2024-001"));
        assert!(is_braingrid_req("REQ-2025-1234"));
        assert!(!is_braingrid_req("REQ-24-001")); // year too short
        assert!(!is_braingrid_req("REQ-2024-12")); // number too short
        assert!(!is_braingrid_req("req-2024-001")); // lowercase
    }

    #[tokio::test]
    async fn test_fetch_local_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");
        let content = "Test specification content";
        std::fs::write(&file_path, content).unwrap();

        let source = SourceType::LocalFile(file_path);
        let result = fetch_source_content(source).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), content);
    }

    #[tokio::test]
    async fn test_fetch_nonexistent_file() {
        let source = SourceType::LocalFile(PathBuf::from("./nonexistent-file-12345.md"));
        let result = fetch_source_content(source).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            SourceFetchError::NotFound(_) => {}
            _ => panic!("Expected NotFound error"),
        }
    }
}

