//! Error recovery strategies and utilities.
//!
//! This module provides utilities for handling and recovering from file operation errors.

use crate::workspace::errors::{ErrorContext, FileOperationError, RecoveryStrategy};
use std::path::PathBuf;

/// Error recovery handler.
///
/// Provides utilities for determining recovery actions and handling errors.
#[derive(Debug, Clone)]
pub struct ErrorRecovery;

impl ErrorRecovery {
    /// Determine if an error can be automatically recovered.
    pub fn can_auto_recover(error: &FileOperationError) -> bool {
        matches!(
            error.recovery_strategy(),
            RecoveryStrategy::Retry | RecoveryStrategy::Skip
        )
    }

    /// Get a human-readable error message with context.
    pub fn format_error(error: &FileOperationError, context: Option<&ErrorContext>) -> String {
        let mut message = format!("{}", error);

        if let Some(context) = context {
            if !context.affected_paths.is_empty() {
                let paths: Vec<String> = context
                    .affected_paths
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect();
                message.push_str(&format!("\nAffected paths: {}", paths.join(", ")));
            }

            if let Some(suggestion) = &context.suggestion {
                message.push_str(&format!("\nSuggestion: {}", suggestion));
            }
        }

        if let Some(suggestion) = error.suggest_fix() {
            message.push_str(&format!("\nSuggested fix: {}", suggestion));
        }

        message
    }

    /// Extract all affected paths from an error and context.
    pub fn extract_affected_paths(
        error: &FileOperationError,
        context: Option<&ErrorContext>,
    ) -> Vec<PathBuf> {
        let mut paths = error.affected_paths();

        if let Some(context) = context {
            paths.extend(context.affected_paths.iter().cloned());
        }

        // Deduplicate paths
        paths.sort();
        paths.dedup();
        paths
    }

    /// Create a summary of errors for batch operations.
    pub fn summarize_errors(errors: &[FileOperationError]) -> ErrorSummary {
        let mut summary = ErrorSummary::default();

        for error in errors {
            summary.total_errors += 1;

            match error {
                FileOperationError::PathNotFound { .. } => summary.path_not_found += 1,
                FileOperationError::PermissionDenied { .. } => summary.permission_denied += 1,
                FileOperationError::AlreadyExists { .. } => summary.already_exists += 1,
                FileOperationError::WorkspaceBoundaryViolation { .. } => {
                    summary.boundary_violations += 1
                }
                FileOperationError::PatchConflict { .. } => summary.patch_conflicts += 1,
                FileOperationError::InvalidInput { .. } => summary.invalid_input += 1,
                FileOperationError::IoError { .. } => summary.io_errors += 1,
                FileOperationError::TransactionFailed { .. } => summary.transaction_failures += 1,
            }

            if error.is_recoverable() {
                summary.recoverable_errors += 1;
            } else {
                summary.fatal_errors += 1;
            }
        }

        summary
    }
}

/// Summary of errors for batch operations.
#[derive(Debug, Default, Clone)]
pub struct ErrorSummary {
    /// Total number of errors.
    pub total_errors: usize,
    /// Number of recoverable errors.
    pub recoverable_errors: usize,
    /// Number of fatal errors.
    pub fatal_errors: usize,
    /// Number of path not found errors.
    pub path_not_found: usize,
    /// Number of permission denied errors.
    pub permission_denied: usize,
    /// Number of already exists errors.
    pub already_exists: usize,
    /// Number of boundary violation errors.
    pub boundary_violations: usize,
    /// Number of patch conflict errors.
    pub patch_conflicts: usize,
    /// Number of invalid input errors.
    pub invalid_input: usize,
    /// Number of I/O errors.
    pub io_errors: usize,
    /// Number of transaction failure errors.
    pub transaction_failures: usize,
}

impl ErrorSummary {
    /// Check if all errors are recoverable.
    pub fn all_recoverable(&self) -> bool {
        self.total_errors > 0 && self.fatal_errors == 0
    }

    /// Check if any errors are fatal.
    pub fn has_fatal_errors(&self) -> bool {
        self.fatal_errors > 0
    }

    /// Get a human-readable summary.
    pub fn to_string(&self) -> String {
        if self.total_errors == 0 {
            return "No errors".to_string();
        }

        let mut parts = vec![format!("Total errors: {}", self.total_errors)];

        if self.recoverable_errors > 0 {
            parts.push(format!("Recoverable: {}", self.recoverable_errors));
        }

        if self.fatal_errors > 0 {
            parts.push(format!("Fatal: {}", self.fatal_errors));
        }

        if self.path_not_found > 0 {
            parts.push(format!("Path not found: {}", self.path_not_found));
        }

        if self.permission_denied > 0 {
            parts.push(format!("Permission denied: {}", self.permission_denied));
        }

        if self.patch_conflicts > 0 {
            parts.push(format!("Patch conflicts: {}", self.patch_conflicts));
        }

        if self.boundary_violations > 0 {
            parts.push(format!("Boundary violations: {}", self.boundary_violations));
        }

        parts.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_auto_recover() {
        let retry_error = FileOperationError::InvalidInput {
            operation: "test".to_string(),
            field: "field".to_string(),
            reason: "test".to_string(),
        };
        assert!(ErrorRecovery::can_auto_recover(&retry_error));

        let abort_error = FileOperationError::PermissionDenied {
            path: "/path".to_string(),
            operation: "write".to_string(),
            required_permission: "write".to_string(),
        };
        assert!(!ErrorRecovery::can_auto_recover(&abort_error));
    }

    #[test]
    fn test_format_error() {
        let error = FileOperationError::PathNotFound {
            path: "/test/file.txt".to_string(),
            operation: "read".to_string(),
        };

        let context = ErrorContext::new("read_file")
            .with_path("/test/file.txt")
            .with_suggestion("Check file exists");

        let formatted = ErrorRecovery::format_error(&error, Some(&context));
        assert!(formatted.contains("path not found"));
        assert!(formatted.contains("Affected paths"));
        assert!(formatted.contains("Suggested fix"));
    }

    #[test]
    fn test_extract_affected_paths() {
        let error = FileOperationError::PathNotFound {
            path: "/test/file1.txt".to_string(),
            operation: "read".to_string(),
        };

        let context = ErrorContext::new("read_file")
            .with_path("/test/file2.txt")
            .with_path("/test/file3.txt");

        let paths = ErrorRecovery::extract_affected_paths(&error, Some(&context));
        assert_eq!(paths.len(), 3);
    }

    #[test]
    fn test_summarize_errors() {
        let errors = vec![
            FileOperationError::PathNotFound {
                path: "/file1".to_string(),
                operation: "read".to_string(),
            },
            FileOperationError::PathNotFound {
                path: "/file2".to_string(),
                operation: "read".to_string(),
            },
            FileOperationError::PermissionDenied {
                path: "/file3".to_string(),
                operation: "write".to_string(),
                required_permission: "write".to_string(),
            },
        ];

        let summary = ErrorRecovery::summarize_errors(&errors);
        assert_eq!(summary.total_errors, 3);
        assert_eq!(summary.path_not_found, 2);
        assert_eq!(summary.permission_denied, 1);
        assert_eq!(summary.recoverable_errors, 2);
        assert_eq!(summary.fatal_errors, 1);
    }

    #[test]
    fn test_error_summary_all_recoverable() {
        let errors = vec![
            FileOperationError::PathNotFound {
                path: "/file1".to_string(),
                operation: "read".to_string(),
            },
            FileOperationError::InvalidInput {
                operation: "test".to_string(),
                field: "field".to_string(),
                reason: "test".to_string(),
            },
        ];

        let summary = ErrorRecovery::summarize_errors(&errors);
        assert!(summary.all_recoverable());
        assert!(!summary.has_fatal_errors());
    }

    #[test]
    fn test_error_summary_to_string() {
        let errors = vec![
            FileOperationError::PathNotFound {
                path: "/file1".to_string(),
                operation: "read".to_string(),
            },
        ];

        let summary = ErrorRecovery::summarize_errors(&errors);
        let summary_str = summary.to_string();
        assert!(summary_str.contains("Total errors: 1"));
        assert!(summary_str.contains("Path not found: 1"));
    }
}
