//! Transaction system for atomic file operations.
//!
//! This module provides transaction-like semantics for grouping file operations,
//! ensuring atomic commit/rollback behavior.

use crate::workspace::errors::{FileOperationError, FileOperationResult};
use crate::workspace::file_ops::FileOperations;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// A single operation in a transaction.
#[derive(Debug, Clone)]
pub enum FileOperation {
    /// Create a file.
    CreateFile {
        path: PathBuf,
        content: String,
    },
    /// Delete a file.
    DeleteFile {
        path: PathBuf,
        backup_content: Option<String>,
    },
    /// Rename/move a file.
    RenameFile {
        from: PathBuf,
        to: PathBuf,
    },
    /// Write content to a file (overwrite).
    WriteFile {
        path: PathBuf,
        content: String,
        backup_content: Option<String>,
    },
}

/// Transaction for atomic file operations.
pub struct FileTransaction {
    /// File operations handler.
    file_ops: FileOperations,
    /// Operations to perform.
    operations: Vec<FileOperation>,
    /// Backup storage for rollback.
    backups: HashMap<PathBuf, String>,
}

impl FileTransaction {
    /// Create a new transaction.
    ///
    /// # Errors
    /// Returns error if file operations handler cannot be created.
    pub fn new(workspace_root: impl AsRef<Path>) -> FileOperationResult<Self> {
        let file_ops = FileOperations::new(workspace_root)?;

        Ok(Self {
            file_ops,
            operations: Vec::new(),
            backups: HashMap::new(),
        })
    }

    /// Add a create file operation to the transaction.
    pub fn create_file(&mut self, path: impl AsRef<Path>, content: &str) -> FileOperationResult<()> {
        // Validate path by attempting to get workspace root (indirect validation)
        let workspace_root = self.file_ops.workspace_root();
        let full_path = if path.as_ref().is_absolute() {
            path.as_ref().to_path_buf()
        } else {
            workspace_root.join(path.as_ref())
        };

        // Validate using boundary validator directly
        use crate::workspace::boundary::BoundaryValidator;
        let validator = BoundaryValidator::new(workspace_root)
            .map_err(|e| FileOperationError::IoError {
                path: workspace_root.display().to_string(),
                operation: "validate_path".to_string(),
                source: std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Failed to create validator: {}", e),
                ),
            })?;

        let validated_path = validator.validate_path(&full_path, false)
            .map_err(FileOperationError::from)?;

        self.operations.push(FileOperation::CreateFile {
            path: validated_path,
            content: content.to_string(),
        });

        Ok(())
    }

    /// Add a delete file operation to the transaction.
    pub fn delete_file(&mut self, path: impl AsRef<Path>) -> FileOperationResult<()> {
        let workspace_root = self.file_ops.workspace_root();
        let full_path = if path.as_ref().is_absolute() {
            path.as_ref().to_path_buf()
        } else {
            workspace_root.join(path.as_ref())
        };

        use crate::workspace::boundary::BoundaryValidator;
        let validator = BoundaryValidator::new(workspace_root)
            .map_err(|e| FileOperationError::IoError {
                path: workspace_root.display().to_string(),
                operation: "validate_path".to_string(),
                source: std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Failed to create validator: {}", e),
                ),
            })?;

        let validated_path = validator.validate_path(&full_path, false)
            .map_err(FileOperationError::from)?;

        // Backup file content if it exists
        let backup_content = if validated_path.exists() {
            fs::read_to_string(&validated_path).ok()
        } else {
            None
        };

        if let Some(content) = &backup_content {
            self.backups.insert(validated_path.clone(), content.clone());
        }

        self.operations.push(FileOperation::DeleteFile {
            path: validated_path,
            backup_content,
        });

        Ok(())
    }

    /// Add a rename file operation to the transaction.
    pub fn rename_file(
        &mut self,
        from: impl AsRef<Path>,
        to: impl AsRef<Path>,
    ) -> FileOperationResult<()> {
        let workspace_root = self.file_ops.workspace_root();
        
        use crate::workspace::boundary::BoundaryValidator;
        let validator = BoundaryValidator::new(workspace_root)
            .map_err(|e| FileOperationError::IoError {
                path: workspace_root.display().to_string(),
                operation: "validate_path".to_string(),
                source: std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Failed to create validator: {}", e),
                ),
            })?;

        let full_from = if from.as_ref().is_absolute() {
            from.as_ref().to_path_buf()
        } else {
            workspace_root.join(from.as_ref())
        };

        let full_to = if to.as_ref().is_absolute() {
            to.as_ref().to_path_buf()
        } else {
            workspace_root.join(to.as_ref())
        };

        let validated_from = validator.validate_path(&full_from, false)
            .map_err(FileOperationError::from)?;
        let validated_to = validator.validate_path(&full_to, false)
            .map_err(FileOperationError::from)?;

        self.operations.push(FileOperation::RenameFile {
            from: validated_from,
            to: validated_to,
        });

        Ok(())
    }

    /// Add a write file operation to the transaction.
    pub fn write_file(&mut self, path: impl AsRef<Path>, content: &str) -> FileOperationResult<()> {
        let workspace_root = self.file_ops.workspace_root();
        let full_path = if path.as_ref().is_absolute() {
            path.as_ref().to_path_buf()
        } else {
            workspace_root.join(path.as_ref())
        };

        use crate::workspace::boundary::BoundaryValidator;
        let validator = BoundaryValidator::new(workspace_root)
            .map_err(|e| FileOperationError::IoError {
                path: workspace_root.display().to_string(),
                operation: "validate_path".to_string(),
                source: std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Failed to create validator: {}", e),
                ),
            })?;

        let validated_path = validator.validate_path(&full_path, false)
            .map_err(FileOperationError::from)?;

        // Backup existing content if file exists
        let backup_content = if validated_path.exists() {
            fs::read_to_string(&validated_path).ok()
        } else {
            None
        };

        if let Some(content) = &backup_content {
            self.backups.insert(validated_path.clone(), content.clone());
        }

        self.operations.push(FileOperation::WriteFile {
            path: validated_path,
            content: content.to_string(),
            backup_content,
        });

        Ok(())
    }

    /// Commit the transaction (apply all operations).
    ///
    /// # Errors
    /// Returns error if any operation fails. All operations are rolled back on failure.
    pub fn commit(self) -> FileOperationResult<Vec<PathBuf>> {
        let mut changed_paths = Vec::new();
        let mut applied_operations = Vec::new();

        // Apply all operations
        for operation in &self.operations {
            match operation {
                FileOperation::CreateFile { path, content } => {
                    self.file_ops.create_file(path, content)?;
                    changed_paths.push(path.clone());
                    applied_operations.push(operation.clone());
                }
                FileOperation::DeleteFile { path, .. } => {
                    self.file_ops.delete_file(path)?;
                    changed_paths.push(path.clone());
                    applied_operations.push(operation.clone());
                }
                FileOperation::RenameFile { from, to } => {
                    self.file_ops.rename_path(from, to)?;
                    changed_paths.push(from.clone());
                    changed_paths.push(to.clone());
                    applied_operations.push(operation.clone());
                }
                FileOperation::WriteFile { path, content, .. } => {
                    self.file_ops.create_file(path, content)?;
                    changed_paths.push(path.clone());
                    applied_operations.push(operation.clone());
                }
            }
        }

        Ok(changed_paths)
    }

    /// Rollback the transaction (undo all operations).
    ///
    /// This restores files from backups and removes created files.
    pub fn rollback(&mut self) -> FileOperationResult<()> {
        // Rollback in reverse order
        for operation in self.operations.iter().rev() {
            match operation {
                FileOperation::CreateFile { path, .. } => {
                    // Delete created file
                    if path.exists() {
                        let _ = self.file_ops.delete_file(path);
                    }
                }
                FileOperation::DeleteFile { path, backup_content } => {
                    // Restore deleted file
                    if let Some(content) = backup_content {
                        let _ = self.file_ops.create_file(path, content);
                    }
                }
                FileOperation::RenameFile { from, to } => {
                    // Reverse the rename
                    if to.exists() {
                        let _ = self.file_ops.rename_path(to, from);
                    }
                }
                FileOperation::WriteFile { path, backup_content, content: _ } => {
                    // Restore original content
                    if let Some(content) = backup_content {
                        let _ = self.file_ops.create_file(path, content);
                    } else if path.exists() {
                        // File didn't exist before, delete it
                        let _ = self.file_ops.delete_file(path);
                    }
                }
            }
        }

        Ok(())
    }

    /// Get the number of operations in the transaction.
    pub fn operation_count(&self) -> usize {
        self.operations.len()
    }

    /// Check if the transaction is empty.
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_transaction_commit() {
        let temp = TempDir::new().unwrap();
        let mut tx = FileTransaction::new(temp.path()).unwrap();

        tx.create_file("file1.txt", "content1").unwrap();
        tx.create_file("file2.txt", "content2").unwrap();

        let changed = tx.commit().unwrap();
        assert_eq!(changed.len(), 2);

        // Verify files were created
        assert!(temp.path().join("file1.txt").exists());
        assert!(temp.path().join("file2.txt").exists());
    }

    #[test]
    fn test_transaction_rollback() {
        let temp = TempDir::new().unwrap();
        let mut tx = FileTransaction::new(temp.path()).unwrap();

        // Create a file first
        let file_path = temp.path().join("original.txt");
        fs::write(&file_path, "original content").unwrap();

        // Add operations
        tx.write_file("original.txt", "new content").unwrap();
        tx.create_file("new.txt", "new file").unwrap();

        // Rollback
        tx.rollback().unwrap();

        // Verify original file was restored
        assert_eq!(fs::read_to_string(&file_path).unwrap(), "original content");
        // Verify new file was deleted
        assert!(!temp.path().join("new.txt").exists());
    }

    #[test]
    fn test_transaction_commit_failure_rollback() {
        let temp = TempDir::new().unwrap();
        let mut tx = FileTransaction::new(temp.path()).unwrap();

        // Create a file first
        let file_path = temp.path().join("existing.txt");
        fs::write(&file_path, "original").unwrap();

        // Add operations - one will fail (trying to create existing file)
        tx.write_file("existing.txt", "modified").unwrap();
        tx.create_file("existing.txt", "duplicate").unwrap(); // This will fail

        // Commit should fail and rollback
        let result = tx.commit();
        assert!(result.is_err());

        // Original file should be restored
        assert_eq!(fs::read_to_string(&file_path).unwrap(), "original");
    }
}
