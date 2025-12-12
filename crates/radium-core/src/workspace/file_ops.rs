//! File mutation operations with workspace boundary validation.
//!
//! This module provides safe file operations (create, delete, rename, create_dir)
//! that are constrained to the workspace boundary.

use crate::workspace::boundary::BoundaryValidator;
use crate::workspace::errors::{FileOperationError, FileOperationResult};
use std::fs;
use std::path::{Path, PathBuf};

/// File operation handler with boundary validation.
pub struct FileOperations {
    /// Workspace root path.
    workspace_root: PathBuf,
    /// Boundary validator.
    boundary_validator: BoundaryValidator,
}

impl FileOperations {
    /// Create a new file operations handler.
    ///
    /// # Errors
    /// Returns error if workspace root cannot be canonicalized.
    pub fn new(workspace_root: impl AsRef<Path>) -> FileOperationResult<Self> {
        let root = workspace_root.as_ref().to_path_buf();
        let validator = BoundaryValidator::new(&root)
            .map_err(|e| FileOperationError::IoError {
                path: root.display().to_string(),
                operation: "initialize file operations".to_string(),
                source: std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Failed to create boundary validator: {}", e),
                ),
            })?;

        Ok(Self {
            workspace_root: root,
            boundary_validator: validator,
        })
    }

    /// Create a new file with content.
    ///
    /// # Arguments
    /// * `path` - Path to the file (relative to workspace root)
    /// * `content` - Content to write to the file
    ///
    /// # Returns
    /// The canonicalized path of the created file
    ///
    /// # Errors
    /// Returns error if path is outside workspace, file already exists, or I/O fails.
    pub fn create_file(
        &self,
        path: impl AsRef<Path>,
        content: &str,
    ) -> FileOperationResult<PathBuf> {
        let validated_path = self
            .boundary_validator
            .validate_path(path, false)
            .map_err(FileOperationError::from)?;

        // Check if file already exists
        if validated_path.exists() {
            return Err(FileOperationError::AlreadyExists {
                path: validated_path.display().to_string(),
                operation: "create_file".to_string(),
            });
        }

        // Ensure parent directory exists
        if let Some(parent) = validated_path.parent() {
            fs::create_dir_all(parent).map_err(|e| FileOperationError::IoError {
                path: parent.display().to_string(),
                operation: "create_parent_dir".to_string(),
                source: e,
            })?;
        }

        // Write file
        fs::write(&validated_path, content).map_err(|e| FileOperationError::IoError {
            path: validated_path.display().to_string(),
            operation: "write_file".to_string(),
            source: e,
        })?;

        Ok(validated_path)
    }

    /// Delete a file.
    ///
    /// # Arguments
    /// * `path` - Path to the file (relative to workspace root)
    ///
    /// # Returns
    /// The canonicalized path of the deleted file
    ///
    /// # Errors
    /// Returns error if path is outside workspace, file doesn't exist, or deletion fails.
    pub fn delete_file(&self, path: impl AsRef<Path>) -> FileOperationResult<PathBuf> {
        let validated_path = self
            .boundary_validator
            .validate_path(path, false)
            .map_err(FileOperationError::from)?;

        // Check if file exists
        if !validated_path.exists() {
            return Err(FileOperationError::PathNotFound {
                path: validated_path.display().to_string(),
                operation: "delete_file".to_string(),
            });
        }

        // Check if it's actually a file (not a directory)
        if !validated_path.is_file() {
            return Err(FileOperationError::InvalidInput {
                operation: "delete_file".to_string(),
                field: "path".to_string(),
                reason: "Path is a directory, not a file".to_string(),
            });
        }

        // Delete file
        fs::remove_file(&validated_path).map_err(|e| FileOperationError::IoError {
            path: validated_path.display().to_string(),
            operation: "delete_file".to_string(),
            source: e,
        })?;

        Ok(validated_path)
    }

    /// Rename or move a file or directory.
    ///
    /// # Arguments
    /// * `from` - Source path (relative to workspace root)
    /// * `to` - Destination path (relative to workspace root)
    ///
    /// # Returns
    /// Tuple of (old_path, new_path) both canonicalized
    ///
    /// # Errors
    /// Returns error if paths are outside workspace, source doesn't exist, or rename fails.
    pub fn rename_path(
        &self,
        from: impl AsRef<Path>,
        to: impl AsRef<Path>,
    ) -> FileOperationResult<(PathBuf, PathBuf)> {
        let validated_from = self
            .boundary_validator
            .validate_path(from, false)
            .map_err(FileOperationError::from)?;

        let validated_to = self
            .boundary_validator
            .validate_path(to, false)
            .map_err(FileOperationError::from)?;

        // Check if source exists
        if !validated_from.exists() {
            return Err(FileOperationError::PathNotFound {
                path: validated_from.display().to_string(),
                operation: "rename_path".to_string(),
            });
        }

        // Check if destination already exists
        if validated_to.exists() {
            return Err(FileOperationError::AlreadyExists {
                path: validated_to.display().to_string(),
                operation: "rename_path".to_string(),
            });
        }

        // Ensure destination parent directory exists
        if let Some(parent) = validated_to.parent() {
            fs::create_dir_all(parent).map_err(|e| FileOperationError::IoError {
                path: parent.display().to_string(),
                operation: "create_parent_dir".to_string(),
                source: e,
            })?;
        }

        // Rename/move
        fs::rename(&validated_from, &validated_to).map_err(|e| FileOperationError::IoError {
            path: validated_from.display().to_string(),
            operation: "rename_path".to_string(),
            source: e,
        })?;

        Ok((validated_from, validated_to))
    }

    /// Create a directory (and parent directories if needed).
    ///
    /// # Arguments
    /// * `path` - Path to the directory (relative to workspace root)
    ///
    /// # Returns
    /// The canonicalized path of the created directory
    ///
    /// # Errors
    /// Returns error if path is outside workspace, directory already exists, or creation fails.
    pub fn create_dir(&self, path: impl AsRef<Path>) -> FileOperationResult<PathBuf> {
        let validated_path = self
            .boundary_validator
            .validate_path(path, false)
            .map_err(FileOperationError::from)?;

        // Check if directory already exists
        if validated_path.exists() {
            if validated_path.is_dir() {
                // Directory already exists, return it
                return Ok(validated_path);
            } else {
                return Err(FileOperationError::AlreadyExists {
                    path: validated_path.display().to_string(),
                    operation: "create_dir".to_string(),
                });
            }
        }

        // Create directory (and parents)
        fs::create_dir_all(&validated_path).map_err(|e| FileOperationError::IoError {
            path: validated_path.display().to_string(),
            operation: "create_dir".to_string(),
            source: e,
        })?;

        Ok(validated_path)
    }

    /// Get the workspace root path.
    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    /// Get the boundary validator (for use by transactions).
    pub(crate) fn boundary_validator(&self) -> &BoundaryValidator {
        &self.boundary_validator
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_file() {
        let temp = TempDir::new().unwrap();
        let ops = FileOperations::new(temp.path()).unwrap();

        let result = ops.create_file("test.txt", "hello world");
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.exists());
        assert_eq!(fs::read_to_string(&path).unwrap(), "hello world");
    }

    #[test]
    fn test_create_file_already_exists() {
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
    fn test_delete_file() {
        let temp = TempDir::new().unwrap();
        let ops = FileOperations::new(temp.path()).unwrap();

        let path = ops.create_file("test.txt", "content").unwrap();
        assert!(path.exists());

        let deleted = ops.delete_file("test.txt").unwrap();
        assert_eq!(deleted, path);
        assert!(!path.exists());
    }

    #[test]
    fn test_delete_file_not_found() {
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
    fn test_rename_path() {
        let temp = TempDir::new().unwrap();
        let ops = FileOperations::new(temp.path()).unwrap();

        let old_path = ops.create_file("old.txt", "content").unwrap();
        let (from, to) = ops.rename_path("old.txt", "new.txt").unwrap();

        assert_eq!(from, old_path);
        assert!(to.exists());
        assert!(!from.exists());
        assert_eq!(fs::read_to_string(&to).unwrap(), "content");
    }

    #[test]
    fn test_create_dir() {
        let temp = TempDir::new().unwrap();
        let ops = FileOperations::new(temp.path()).unwrap();

        let path = ops.create_dir("subdir/nested").unwrap();
        assert!(path.is_dir());
        assert!(path.exists());
    }

    #[test]
    fn test_create_dir_already_exists() {
        let temp = TempDir::new().unwrap();
        let ops = FileOperations::new(temp.path()).unwrap();

        let path1 = ops.create_dir("subdir").unwrap();
        let path2 = ops.create_dir("subdir").unwrap();

        // Should return existing directory
        assert_eq!(path1, path2);
    }

    #[test]
    fn test_boundary_validation() {
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
}
