//! Workspace boundary validation for file operations.
//!
//! This module provides security validation to ensure all file operations
//! are constrained to the workspace root, preventing path traversal attacks
//! and symlink-based escapes.

use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur during workspace boundary validation.
#[derive(Debug, Error)]
pub enum BoundaryError {
    /// Path is outside the workspace root.
    #[error("path outside workspace boundary: {path} (workspace root: {root})")]
    OutsideBoundary {
        path: String,
        root: String,
    },

    /// Path contains traversal attempts (../).
    #[error("path traversal detected: {0}")]
    PathTraversal(String),

    /// Path is absolute when relative was expected.
    #[error("absolute path not allowed: {0}")]
    AbsolutePath(String),

    /// Symlink detected that would escape workspace.
    #[error("symlink escape attempt detected: {path} (resolves to: {resolved})")]
    SymlinkEscape {
        path: String,
        resolved: String,
    },

    /// I/O error during validation.
    #[error("I/O error during validation: {0}")]
    Io(#[from] std::io::Error),

    /// Path cannot be canonicalized.
    #[error("failed to canonicalize path: {0}")]
    CanonicalizationFailed(String),
}

/// Result type for boundary validation operations.
pub type Result<T> = std::result::Result<T, BoundaryError>;

/// Workspace boundary validator.
///
/// Validates that all file paths are within the workspace root and
/// prevents path traversal and symlink-based escapes.
#[derive(Debug, Clone)]
pub struct BoundaryValidator {
    /// Canonicalized workspace root path.
    workspace_root: PathBuf,
}

impl BoundaryValidator {
    /// Create a new boundary validator for the given workspace root.
    ///
    /// # Arguments
    /// * `workspace_root` - The root directory of the workspace
    ///
    /// # Errors
    /// Returns error if workspace root cannot be canonicalized.
    pub fn new(workspace_root: impl AsRef<Path>) -> Result<Self> {
        let root = workspace_root.as_ref();
        let canonical_root = root
            .canonicalize()
            .map_err(|e| BoundaryError::CanonicalizationFailed(format!(
                "Failed to canonicalize workspace root {}: {}",
                root.display(),
                e
            )))?;

        Ok(Self {
            workspace_root: canonical_root,
        })
    }

    /// Validate that a path is within the workspace boundary.
    ///
    /// This method:
    /// 1. Checks for path traversal attempts (../)
    /// 2. Rejects absolute paths if a relative path was expected
    /// 3. Canonicalizes the path
    /// 4. Verifies the canonicalized path is within workspace root
    /// 5. Checks for symlink escapes
    ///
    /// # Arguments
    /// * `path` - The path to validate (can be relative or absolute)
    /// * `allow_absolute` - Whether to allow absolute paths (default: false)
    ///
    /// # Returns
    /// The canonicalized path if valid, error otherwise.
    ///
    /// # Errors
    /// Returns error if path is outside workspace, contains traversal, or has symlink escape.
    pub fn validate_path(&self, path: impl AsRef<Path>, allow_absolute: bool) -> Result<PathBuf> {
        let path = path.as_ref();

        // Check for path traversal in string representation
        let path_str = path.to_string_lossy();
        if path_str.contains("..") {
            return Err(BoundaryError::PathTraversal(path_str.to_string()));
        }

        // Check for absolute paths if not allowed
        if path.is_absolute() && !allow_absolute {
            return Err(BoundaryError::AbsolutePath(path_str.to_string()));
        }

        // Resolve the path relative to workspace root if it's relative
        let resolved_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.workspace_root.join(path)
        };

        // Canonicalize the resolved path
        let canonical_path = resolved_path
            .canonicalize()
            .map_err(|e| BoundaryError::CanonicalizationFailed(format!(
                "Failed to canonicalize path {}: {}",
                resolved_path.display(),
                e
            )))?;

        // Verify the canonicalized path is within workspace root
        if !canonical_path.starts_with(&self.workspace_root) {
            return Err(BoundaryError::OutsideBoundary {
                path: canonical_path.display().to_string(),
                root: self.workspace_root.display().to_string(),
            });
        }

        // Check for symlink escapes by comparing resolved path components
        self.check_symlink_escape(&canonical_path)?;

        Ok(canonical_path)
    }

    /// Validate multiple paths at once.
    ///
    /// # Arguments
    /// * `paths` - Iterator of paths to validate
    /// * `allow_absolute` - Whether to allow absolute paths
    ///
    /// # Returns
    /// Vector of canonicalized paths if all valid, first error otherwise.
    pub fn validate_paths<I, P>(&self, paths: I, allow_absolute: bool) -> Result<Vec<PathBuf>>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let mut validated = Vec::new();
        for path in paths {
            validated.push(self.validate_path(path, allow_absolute)?);
        }
        Ok(validated)
    }

    /// Check if a path would escape via symlinks.
    ///
    /// This validates that even if symlinks are present, the resolved
    /// path still stays within the workspace boundary.
    fn check_symlink_escape(&self, canonical_path: &Path) -> Result<()> {
        // Walk up the path components and check each segment
        let mut current = canonical_path.to_path_buf();
        
        while current != self.workspace_root && current.parent().is_some() {
            // Check if current path is a symlink
            if let Ok(metadata) = std::fs::symlink_metadata(&current) {
                if metadata.file_type().is_symlink() {
                    // Resolve the symlink
                    let symlink_target = std::fs::read_link(&current)
                        .map_err(|e| BoundaryError::Io(e))?;
                    
                    // Resolve relative to symlink's parent
                    let resolved = if symlink_target.is_absolute() {
                        symlink_target
                    } else {
                        current.parent()
                            .unwrap_or(&self.workspace_root)
                            .join(&symlink_target)
                    };

                    // Canonicalize the resolved symlink target
                    if let Ok(canonical_target) = resolved.canonicalize() {
                        // Check if the symlink target is outside workspace
                        if !canonical_target.starts_with(&self.workspace_root) {
                            return Err(BoundaryError::SymlinkEscape {
                                path: current.display().to_string(),
                                resolved: canonical_target.display().to_string(),
                            });
                        }
                    }
                }
            }
            
            // Move to parent
            if let Some(parent) = current.parent() {
                current = parent.to_path_buf();
            } else {
                break;
            }
        }

        Ok(())
    }

    /// Get the workspace root path.
    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    /// Check if a path string contains unsafe patterns.
    ///
    /// This is a quick check before attempting full validation.
    pub fn is_unsafe_path(path: &str) -> bool {
        // Check for path traversal
        if path.contains("..") {
            return true;
        }

        // Check for null bytes (potential security issue)
        if path.contains('\0') {
            return true;
        }

        // Check for absolute paths on Unix
        if path.starts_with('/') {
            return true;
        }

        // Check for absolute paths on Windows
        if path.len() > 1 && &path[1..3] == ":\\" {
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_validate_path_within_workspace() {
        let temp = TempDir::new().unwrap();
        let validator = BoundaryValidator::new(temp.path()).unwrap();

        // Create a file in workspace
        let file_path = temp.path().join("test.txt");
        std::fs::write(&file_path, "test").unwrap();

        // Validate the path
        let validated = validator.validate_path("test.txt", false).unwrap();
        assert_eq!(validated, file_path.canonicalize().unwrap());
    }

    #[test]
    fn test_validate_path_traversal_rejected() {
        let temp = TempDir::new().unwrap();
        let validator = BoundaryValidator::new(temp.path()).unwrap();

        // Path traversal should be rejected
        let result = validator.validate_path("../outside", false);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BoundaryError::PathTraversal(_)));
    }

    #[test]
    fn test_validate_path_absolute_rejected() {
        let temp = TempDir::new().unwrap();
        let validator = BoundaryValidator::new(temp.path()).unwrap();

        // Absolute path should be rejected when allow_absolute=false
        let result = validator.validate_path("/etc/passwd", false);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BoundaryError::AbsolutePath(_)));
    }

    #[test]
    fn test_validate_path_outside_workspace_rejected() {
        let temp = TempDir::new().unwrap();
        let validator = BoundaryValidator::new(temp.path()).unwrap();

        // Create a file outside workspace
        let outside_dir = temp.path().parent().unwrap();
        let outside_file = outside_dir.join("outside.txt");
        std::fs::write(&outside_file, "test").unwrap();

        // Try to access it via absolute path (if allowed)
        let result = validator.validate_path(&outside_file, true);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BoundaryError::OutsideBoundary { .. }));
    }

    #[test]
    fn test_validate_multiple_paths() {
        let temp = TempDir::new().unwrap();
        let validator = BoundaryValidator::new(temp.path()).unwrap();

        // Create multiple files
        std::fs::write(temp.path().join("file1.txt"), "test1").unwrap();
        std::fs::write(temp.path().join("file2.txt"), "test2").unwrap();

        let paths = vec!["file1.txt", "file2.txt"];
        let validated = validator.validate_paths(paths, false).unwrap();
        assert_eq!(validated.len(), 2);
    }

    #[test]
    fn test_is_unsafe_path() {
        assert!(BoundaryValidator::is_unsafe_path("../outside"));
        assert!(BoundaryValidator::is_unsafe_path("/absolute/path"));
        assert!(BoundaryValidator::is_unsafe_path("C:\\Windows\\path"));
        assert!(BoundaryValidator::is_unsafe_path("path\0with\0null"));
        assert!(!BoundaryValidator::is_unsafe_path("relative/path.txt"));
    }

    #[test]
    fn test_symlink_escape_detection() {
        let temp = TempDir::new().unwrap();
        let validator = BoundaryValidator::new(temp.path()).unwrap();

        // Create a directory inside workspace
        let inside_dir = temp.path().join("inside");
        std::fs::create_dir(&inside_dir).unwrap();

        // Create a directory outside workspace
        let outside_dir = temp.path().parent().unwrap().join("outside");
        std::fs::create_dir(&outside_dir).unwrap();

        // Create a symlink inside workspace pointing outside
        let symlink_path = inside_dir.join("escape");
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&outside_dir, &symlink_path).unwrap();
            
            // Validation should detect the symlink escape
            let result = validator.validate_path("inside/escape", false);
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), BoundaryError::SymlinkEscape { .. }));
        }

        #[cfg(windows)]
        {
            // On Windows, symlinks require special permissions
            // Skip this test on Windows for now
        }
    }

    #[test]
    fn test_validate_nested_paths() {
        let temp = TempDir::new().unwrap();
        let validator = BoundaryValidator::new(temp.path()).unwrap();

        // Create nested directories
        let nested = temp.path().join("a").join("b").join("c");
        std::fs::create_dir_all(&nested).unwrap();
        let file = nested.join("file.txt");
        std::fs::write(&file, "test").unwrap();

        // Validate nested path
        let validated = validator.validate_path("a/b/c/file.txt", false).unwrap();
        assert_eq!(validated, file.canonicalize().unwrap());
    }
}
