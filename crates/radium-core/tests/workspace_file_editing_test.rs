//! Comprehensive tests for file editing tools (REQ-231).

use radium_core::workspace::{
    boundary::BoundaryValidator, errors::FileOperationError, file_ops::FileOperations,
    patch::{PatchApplicator, PatchContent, PatchInput, PatchOptions},
    transaction::FileTransaction,
};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_boundary_validation_path_traversal() {
    let temp = TempDir::new().unwrap();
    let validator = BoundaryValidator::new(temp.path()).unwrap();

    // Path traversal should be rejected
    let result = validator.validate_path("../outside", false);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        radium_core::workspace::boundary::BoundaryError::PathTraversal(_)
    ));
}

#[test]
fn test_boundary_validation_absolute_path_rejected() {
    let temp = TempDir::new().unwrap();
    let validator = BoundaryValidator::new(temp.path()).unwrap();

    // Absolute path should be rejected when allow_absolute=false
    let result = validator.validate_path("/etc/passwd", false);
    assert!(result.is_err());
}

#[test]
fn test_file_operations_create_and_delete() {
    let temp = TempDir::new().unwrap();
    let ops = FileOperations::new(temp.path()).unwrap();

    // Create file
    let path = ops.create_file("test.txt", "content").unwrap();
    assert!(path.exists());
    assert_eq!(fs::read_to_string(&path).unwrap(), "content");

    // Delete file
    let deleted = ops.delete_file("test.txt").unwrap();
    assert_eq!(deleted, path);
    assert!(!path.exists());
}

#[test]
fn test_file_operations_rename() {
    let temp = TempDir::new().unwrap();
    let ops = FileOperations::new(temp.path()).unwrap();

    // Create file
    let old_path = ops.create_file("old.txt", "content").unwrap();
    assert!(old_path.exists());

    // Rename
    let (from, to) = ops.rename_path("old.txt", "new.txt").unwrap();
    assert_eq!(from, old_path);
    assert!(to.exists());
    assert!(!from.exists());
    assert_eq!(fs::read_to_string(&to).unwrap(), "content");
}

#[test]
fn test_file_operations_create_dir_nested() {
    let temp = TempDir::new().unwrap();
    let ops = FileOperations::new(temp.path()).unwrap();

    // Create nested directory
    let path = ops.create_dir("a/b/c").unwrap();
    assert!(path.is_dir());
    assert!(path.exists());
}

#[test]
fn test_patch_apply_simple() {
    let temp = TempDir::new().unwrap();
    let applicator = PatchApplicator::new(temp.path()).unwrap();

    // Create test file
    let test_file = temp.path().join("test.txt");
    fs::write(&test_file, "line 1\nline 2\nline 3\n").unwrap();

    // Apply patch
    let patch = PatchInput {
        patch: PatchContent::UnifiedDiff {
            content: "--- a/test.txt\n+++ b/test.txt\n@@ -2,1 +2,1 @@\n-line 2\n+line 2 modified\n".to_string(),
        },
        dry_run: false,
        allow_create: true,
        expected_hash: None,
        options: PatchOptions::default(),
    };

    let result = applicator.apply(&patch);
    assert!(result.success);
    assert_eq!(result.changed_files.len(), 1);

    // Verify change
    let content = fs::read_to_string(&test_file).unwrap();
    assert!(content.contains("line 2 modified"));
}

#[test]
fn test_patch_apply_context_mismatch_rejected() {
    let temp = TempDir::new().unwrap();
    let applicator = PatchApplicator::new(temp.path()).unwrap();

    // Create file with different content
    let test_file = temp.path().join("test.txt");
    fs::write(&test_file, "different\n").unwrap();

    // Patch expects "original"
    let patch = PatchInput {
        patch: PatchContent::UnifiedDiff {
            content: "--- a/test.txt\n+++ b/test.txt\n@@ -1,1 +1,1 @@\n-original\n+modified\n".to_string(),
        },
        dry_run: false,
        allow_create: true,
        expected_hash: None,
        options: PatchOptions {
            context_lines: 3,
            ignore_whitespace: false,
            allow_fuzz: false,
            max_fuzz: 0,
        },
    };

    let result = applicator.apply(&patch);
    // Should fail due to context mismatch
    assert!(!result.success || result.has_errors());
}

#[test]
fn test_patch_apply_multi_file() {
    let temp = TempDir::new().unwrap();
    let applicator = PatchApplicator::new(temp.path()).unwrap();

    // Create multiple files
    fs::write(temp.path().join("file1.txt"), "content1\n").unwrap();
    fs::write(temp.path().join("file2.txt"), "content2\n").unwrap();

    // Multi-file patch
    let patch = PatchInput {
        patch: PatchContent::UnifiedDiff {
            content: "--- a/file1.txt\n+++ b/file1.txt\n@@ -1,1 +1,1 @@\n-content1\n+content1 modified\n--- a/file2.txt\n+++ b/file2.txt\n@@ -1,1 +1,1 @@\n-content2\n+content2 modified\n".to_string(),
        },
        dry_run: false,
        allow_create: true,
        expected_hash: None,
        options: PatchOptions::default(),
    };

    let result = applicator.apply(&patch);
    assert!(result.success);
    assert_eq!(result.changed_files.len(), 2);
}

#[test]
fn test_patch_dry_run_no_modification() {
    let temp = TempDir::new().unwrap();
    let applicator = PatchApplicator::new(temp.path()).unwrap();

    let test_file = temp.path().join("test.txt");
    fs::write(&test_file, "original\n").unwrap();
    let original = fs::read_to_string(&test_file).unwrap();

    let patch = PatchInput {
        patch: PatchContent::UnifiedDiff {
            content: "--- a/test.txt\n+++ b/test.txt\n@@ -1,1 +1,1 @@\n-original\n+modified\n".to_string(),
        },
        dry_run: true,
        allow_create: true,
        expected_hash: None,
        options: PatchOptions::default(),
    };

    let result = applicator.apply(&patch);
    assert!(result.success);

    // File should not be modified
    let content = fs::read_to_string(&test_file).unwrap();
    assert_eq!(content, original);
}

#[test]
fn test_transaction_commit_atomic() {
    let temp = TempDir::new().unwrap();
    let mut tx = FileTransaction::new(temp.path()).unwrap();

    tx.create_file("file1.txt", "content1").unwrap();
    tx.create_file("file2.txt", "content2").unwrap();

    let changed = tx.commit().unwrap();
    assert_eq!(changed.len(), 2);

    // Both files should exist
    assert!(temp.path().join("file1.txt").exists());
    assert!(temp.path().join("file2.txt").exists());
}

#[test]
fn test_transaction_rollback() {
    let temp = TempDir::new().unwrap();
    let mut tx = FileTransaction::new(temp.path()).unwrap();

    // Create original file
    let file_path = temp.path().join("original.txt");
    fs::write(&file_path, "original").unwrap();

    // Add operations
    tx.write_file("original.txt", "modified").unwrap();
    tx.create_file("new.txt", "new").unwrap();

    // Rollback
    tx.rollback().unwrap();

    // Original should be restored
    assert_eq!(fs::read_to_string(&file_path).unwrap(), "original");
    // New file should be deleted
    assert!(!temp.path().join("new.txt").exists());
}

#[test]
fn test_error_handling_path_not_found() {
    let temp = TempDir::new().unwrap();
    let ops = FileOperations::new(temp.path()).unwrap();

    let result = ops.delete_file("nonexistent.txt");
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        FileOperationError::PathNotFound { .. }
    ));
}

#[test]
fn test_error_handling_already_exists() {
    let temp = TempDir::new().unwrap();
    let ops = FileOperations::new(temp.path()).unwrap();

    ops.create_file("test.txt", "content").unwrap();

    let result = ops.create_file("test.txt", "different");
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        FileOperationError::AlreadyExists { .. }
    ));
}

#[test]
fn test_error_handling_boundary_violation() {
    let temp = TempDir::new().unwrap();
    let ops = FileOperations::new(temp.path()).unwrap();

    // Try to create file outside workspace
    let result = ops.create_file("../outside.txt", "content");
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        FileOperationError::WorkspaceBoundaryViolation { .. }
    ));
}

#[test]
fn test_error_recovery_suggestions() {
    let error = FileOperationError::PathNotFound {
        path: "/test/file.txt".to_string(),
        operation: "read_file".to_string(),
    };

    let suggestion = error.suggest_fix();
    assert!(suggestion.is_some());
    assert!(suggestion.unwrap().contains("Check that the path exists"));
}

#[test]
fn test_error_recovery_strategies() {
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
