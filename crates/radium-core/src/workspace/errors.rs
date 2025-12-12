//! Unified error handling for file operations.
//!
//! This module provides a comprehensive error taxonomy for all file operations,
//! ensuring consistent error reporting with actionable messages and recovery strategies.

use crate::workspace::boundary::BoundaryError;
use std::path::PathBuf;
use thiserror::Error;

/// Unified error type for all file operations.
#[derive(Debug, Error)]
pub enum FileOperationError {
    /// Path not found.
    #[error("path not found: {path} (operation: {operation})")]
    PathNotFound {
        path: String,
        operation: String,
    },

    /// Permission denied.
    #[error("permission denied: {path} (operation: {operation}, required: {required_permission})")]
    PermissionDenied {
        path: String,
        operation: String,
        required_permission: String,
    },

    /// File or directory already exists.
    #[error("path already exists: {path} (operation: {operation})")]
    AlreadyExists {
        path: String,
        operation: String,
    },

    /// Workspace boundary violation.
    #[error("workspace boundary violation: {path} (workspace root: {workspace_root}, reason: {reason})")]
    WorkspaceBoundaryViolation {
        path: String,
        workspace_root: String,
        reason: String,
    },

    /// Patch conflict detected.
    #[error("patch conflict in {file} at line {line_number}: expected context '{expected}', found '{actual}'")]
    PatchConflict {
        file: String,
        line_number: usize,
        expected: String,
        actual: String,
    },

    /// Invalid input provided.
    #[error("invalid input for {operation}: field '{field}' - {reason}")]
    InvalidInput {
        operation: String,
        field: String,
        reason: String,
    },

    /// I/O error occurred.
    #[error("I/O error on {path} during {operation}: {source}")]
    IoError {
        path: String,
        operation: String,
        source: std::io::Error,
    },

    /// Transaction failed.
    #[error("transaction failed: {reason} (operations attempted: {operations_attempted}, failed at: {failed_at})")]
    TransactionFailed {
        operations_attempted: usize,
        failed_at: String,
        reason: String,
    },
}

impl FileOperationError {
    /// Add context to an error.
    pub fn with_context(self, context: ErrorContext) -> Self {
        match self {
            FileOperationError::PathNotFound { path, operation } => {
                FileOperationError::PathNotFound {
                    path: format!("{} (context: {})", path, context.operation),
                    operation,
                }
            }
            FileOperationError::PermissionDenied { path, operation, required_permission } => {
                FileOperationError::PermissionDenied {
                    path: format!("{} (context: {})", path, context.operation),
                    operation,
                    required_permission,
                }
            }
            FileOperationError::IoError { path, operation, source } => {
                FileOperationError::IoError {
                    path: format!("{} (context: {})", path, context.operation),
                    operation,
                    source,
                }
            }
            other => other,
        }
    }

    /// Get a suggested fix for the error.
    pub fn suggest_fix(&self) -> Option<String> {
        match self {
            FileOperationError::PathNotFound { .. } => {
                Some("Check that the path exists and is spelled correctly".to_string())
            }
            FileOperationError::PermissionDenied { .. } => {
                Some("Check file permissions and ensure you have the required access".to_string())
            }
            FileOperationError::AlreadyExists { .. } => {
                Some("Use a different path or remove the existing file first".to_string())
            }
            FileOperationError::WorkspaceBoundaryViolation { .. } => {
                Some("Ensure all paths are within the workspace root directory".to_string())
            }
            FileOperationError::PatchConflict { .. } => {
                Some("Review the file content and update the patch to match current state".to_string())
            }
            FileOperationError::InvalidInput { .. } => {
                Some("Check the input format and required fields".to_string())
            }
            FileOperationError::IoError { .. } => {
                Some("Check disk space, permissions, and that the file is not locked".to_string())
            }
            FileOperationError::TransactionFailed { .. } => {
                Some("Review the failed operation and retry, or rollback the transaction".to_string())
            }
        }
    }

    /// Check if the error is recoverable.
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            FileOperationError::PathNotFound { .. }
                | FileOperationError::PermissionDenied { .. }
                | FileOperationError::PatchConflict { .. }
                | FileOperationError::InvalidInput { .. }
        )
    }

    /// Get the recovery strategy for this error.
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            FileOperationError::PathNotFound { .. } => RecoveryStrategy::UserInput(
                "Path not found. Please verify the path and try again.".to_string(),
            ),
            FileOperationError::PermissionDenied { .. } => RecoveryStrategy::Abort,
            FileOperationError::AlreadyExists { .. } => RecoveryStrategy::UserInput(
                "File already exists. Overwrite? (yes/no)".to_string(),
            ),
            FileOperationError::WorkspaceBoundaryViolation { .. } => RecoveryStrategy::Abort,
            FileOperationError::PatchConflict { .. } => RecoveryStrategy::UserInput(
                "Patch conflict detected. Review and resolve manually.".to_string(),
            ),
            FileOperationError::InvalidInput { .. } => RecoveryStrategy::Retry,
            FileOperationError::IoError { .. } => RecoveryStrategy::Retry,
            FileOperationError::TransactionFailed { .. } => RecoveryStrategy::Abort,
        }
    }

    /// Get affected paths from the error.
    pub fn affected_paths(&self) -> Vec<PathBuf> {
        match self {
            FileOperationError::PathNotFound { path, .. } => {
                vec![PathBuf::from(path)]
            }
            FileOperationError::PermissionDenied { path, .. } => {
                vec![PathBuf::from(path)]
            }
            FileOperationError::AlreadyExists { path, .. } => {
                vec![PathBuf::from(path)]
            }
            FileOperationError::WorkspaceBoundaryViolation { path, .. } => {
                vec![PathBuf::from(path)]
            }
            FileOperationError::PatchConflict { file, .. } => {
                vec![PathBuf::from(file)]
            }
            FileOperationError::IoError { path, .. } => {
                vec![PathBuf::from(path)]
            }
            _ => Vec::new(),
        }
    }
}

impl From<BoundaryError> for FileOperationError {
    fn from(err: BoundaryError) -> Self {
        match err {
            BoundaryError::OutsideBoundary { path, root } => {
                FileOperationError::WorkspaceBoundaryViolation {
                    path,
                    workspace_root: root,
                    reason: "Path resolves outside workspace root".to_string(),
                }
            }
            BoundaryError::PathTraversal(path) => {
                FileOperationError::WorkspaceBoundaryViolation {
                    path: path.clone(),
                    workspace_root: "unknown".to_string(),
                    reason: format!("Path traversal detected: {}", path),
                }
            }
            BoundaryError::AbsolutePath(path) => {
                FileOperationError::WorkspaceBoundaryViolation {
                    path: path.clone(),
                    workspace_root: "unknown".to_string(),
                    reason: format!("Absolute path not allowed: {}", path),
                }
            }
            BoundaryError::SymlinkEscape { path, resolved } => {
                FileOperationError::WorkspaceBoundaryViolation {
                    path,
                    workspace_root: "unknown".to_string(),
                    reason: format!("Symlink escape attempt: resolves to {}", resolved),
                }
            }
            BoundaryError::Io(e) => {
                FileOperationError::IoError {
                    path: "unknown".to_string(),
                    operation: "boundary validation".to_string(),
                    source: e,
                }
            }
            BoundaryError::CanonicalizationFailed(path) => {
                FileOperationError::IoError {
                    path,
                    operation: "canonicalization".to_string(),
                    source: std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Failed to canonicalize path",
                    ),
                }
            }
        }
    }
}

impl From<std::io::Error> for FileOperationError {
    fn from(err: std::io::Error) -> Self {
        FileOperationError::IoError {
            path: "unknown".to_string(),
            operation: "file operation".to_string(),
            source: err,
        }
    }
}

/// Context information for errors.
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Operation that was being performed.
    pub operation: String,
    /// Affected file paths.
    pub affected_paths: Vec<PathBuf>,
    /// Optional suggestion for fixing the error.
    pub suggestion: Option<String>,
}

impl ErrorContext {
    /// Create a new error context.
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            affected_paths: Vec::new(),
            suggestion: None,
        }
    }

    /// Add an affected path.
    pub fn with_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.affected_paths.push(path.into());
        self
    }

    /// Add multiple affected paths.
    pub fn with_paths(mut self, paths: impl IntoIterator<Item = impl Into<PathBuf>>) -> Self {
        self.affected_paths.extend(paths.into_iter().map(|p| p.into()));
        self
    }

    /// Add a suggestion.
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

/// Recovery strategy for errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryStrategy {
    /// Retry the operation.
    Retry,
    /// Skip this operation and continue.
    Skip,
    /// Abort the entire operation sequence.
    Abort,
    /// Request user input before proceeding.
    UserInput(String),
}

/// Result type for file operations.
pub type FileOperationResult<T> = std::result::Result<T, FileOperationError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_suggest_fix() {
        let err = FileOperationError::PathNotFound {
            path: "/path/to/file".to_string(),
            operation: "read_file".to_string(),
        };
        assert!(err.suggest_fix().is_some());
        assert!(err.suggest_fix().unwrap().contains("Check that the path exists"));
    }

    #[test]
    fn test_error_is_recoverable() {
        let recoverable = FileOperationError::PathNotFound {
            path: "/path".to_string(),
            operation: "read".to_string(),
        };
        assert!(recoverable.is_recoverable());

        let not_recoverable = FileOperationError::TransactionFailed {
            operations_attempted: 5,
            failed_at: "operation 3".to_string(),
            reason: "Critical failure".to_string(),
        };
        assert!(!not_recoverable.is_recoverable());
    }

    #[test]
    fn test_error_recovery_strategy() {
        let err = FileOperationError::PermissionDenied {
            path: "/path".to_string(),
            operation: "write".to_string(),
            required_permission: "write".to_string(),
        };
        assert_eq!(err.recovery_strategy(), RecoveryStrategy::Abort);

        let err = FileOperationError::PatchConflict {
            file: "file.rs".to_string(),
            line_number: 10,
            expected: "old".to_string(),
            actual: "new".to_string(),
        };
        match err.recovery_strategy() {
            RecoveryStrategy::UserInput(_) => {}
            _ => panic!("Expected UserInput strategy"),
        }
    }

    #[test]
    fn test_error_affected_paths() {
        let err = FileOperationError::PathNotFound {
            path: "/test/file.txt".to_string(),
            operation: "read".to_string(),
        };
        let paths = err.affected_paths();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], PathBuf::from("/test/file.txt"));
    }

    #[test]
    fn test_error_context() {
        let context = ErrorContext::new("apply_patch")
            .with_path("file1.txt")
            .with_path("file2.txt")
            .with_suggestion("Check paths");

        assert_eq!(context.operation, "apply_patch");
        assert_eq!(context.affected_paths.len(), 2);
        assert!(context.suggestion.is_some());
    }

    #[test]
    fn test_boundary_error_conversion() {
        let boundary_err = BoundaryError::PathTraversal("../outside".to_string());
        let file_err: FileOperationError = boundary_err.into();
        assert!(matches!(file_err, FileOperationError::WorkspaceBoundaryViolation { .. }));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let file_err: FileOperationError = io_err.into();
        assert!(matches!(file_err, FileOperationError::IoError { .. }));
    }
}
