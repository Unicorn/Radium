//! Workspace management for Radium.
//!
//! This module provides workspace structure management for Radium.
//! The workspace contains:
//! - Stage directories: backlog, development, review, testing, docs
//! - Internal .radium directory for runtime artifacts
//! - Plan directories with REQ-XXX format
//!
//! # Example
//!
//! ```rust,no_run
//! use radium_core::workspace::{Workspace, WorkspaceConfig};
//!
//! # fn main() -> anyhow::Result<()> {
//! let workspace = Workspace::discover()?;
//! workspace.ensure_structure()?;
//! # Ok(())
//! # }
//! ```

pub mod ignore;
pub mod boundary;
pub mod error_recovery;
pub mod errors;
pub mod file_ops;
pub mod patch;
pub mod plan_discovery;
pub mod requirement_id;
pub mod structure;

use std::path::{Path, PathBuf};
use thiserror::Error;

pub use ignore::IgnoreWalker;
pub use boundary::{BoundaryError, BoundaryValidator};
pub use error_recovery::{ErrorRecovery, ErrorSummary};
pub use errors::{
    ErrorContext, FileOperationError, FileOperationResult, RecoveryStrategy,
};
pub use file_ops::FileOperations;
pub use patch::{
    ChangedFile, FilePatch, Hunk, PatchContent, PatchInput, PatchOptions, PatchResult,
    PatchSummary,
};
pub use plan_discovery::{DiscoveredPlan, PlanDiscovery, PlanDiscoveryOptions, SortBy, SortOrder};
pub use requirement_id::{RequirementId, RequirementIdError};
pub use structure::{
    DIR_INTERNALS, DIR_PLAN, DIR_RADIUM, STAGE_BACKLOG, STAGE_DEVELOPMENT, STAGE_DOCS,
    STAGE_REVIEW, STAGE_TESTING, WorkspaceStructure,
};

/// Workspace management errors.
#[derive(Debug, Error)]
pub enum WorkspaceError {
    /// Workspace not found.
    #[error("workspace not found: {0}")]
    NotFound(String),

    /// Invalid workspace structure.
    #[error("invalid workspace structure: {0}")]
    InvalidStructure(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Requirement ID error.
    #[error("requirement ID error: {0}")]
    RequirementId(#[from] RequirementIdError),

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Result type for workspace operations.
pub type Result<T> = std::result::Result<T, WorkspaceError>;

/// Configuration for workspace detection and creation.
#[derive(Debug, Clone, Default)]
pub struct WorkspaceConfig {
    /// Root directory for the workspace.
    /// If None, will search for workspace starting from current directory.
    pub root: Option<PathBuf>,

    /// Whether to create workspace if it doesn't exist.
    pub create_if_missing: bool,
}

/// Workspace manager.
///
/// Provides operations for workspace discovery, initialization, and validation.
#[derive(Debug, Clone)]
pub struct Workspace {
    /// Root directory of the workspace.
    root: PathBuf,
}

impl Workspace {
    /// Discover workspace starting from the current directory.
    ///
    /// Searches upward from current directory for a `.radium` directory,
    /// or falls back to `$HOME/radium` if not found.
    ///
    /// # Errors
    ///
    /// Returns error if workspace cannot be found or validated.
    pub fn discover() -> Result<Self> {
        Self::discover_with_config(&WorkspaceConfig::default())
    }

    /// Discover workspace with custom configuration.
    ///
    /// # Errors
    ///
    /// Returns error if workspace cannot be found or validated.
    pub fn discover_with_config(config: &WorkspaceConfig) -> Result<Self> {
        let root = if let Some(root) = &config.root {
            root.clone()
        // Allow env::var for RADIUM_WORKSPACE (workspace discovery, not app config)
        } else if let Ok(workspace) = {
            #[allow(clippy::disallowed_methods)]
            std::env::var("RADIUM_WORKSPACE")
        } {
            PathBuf::from(workspace)
        } else if let Some(workspace) = Self::find_workspace_upward()? {
            workspace
        } else {
            return Err(WorkspaceError::NotFound(
                "No Radium workspace found. Run 'rad init' to create one.".to_string(),
            ));
        };

        if !root.exists() && config.create_if_missing {
            std::fs::create_dir_all(&root)?;
        }

        let workspace = Self { root };

        if !workspace.is_valid() {
            if config.create_if_missing {
                workspace.ensure_structure()?;
            } else {
                return Err(WorkspaceError::InvalidStructure(format!(
                    "workspace at {} is not valid",
                    workspace.root.display()
                )));
            }
        }

        Ok(workspace)
    }

    /// Create a new workspace at the specified root directory.
    ///
    /// # Errors
    ///
    /// Returns error if workspace cannot be created.
    pub fn create(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        std::fs::create_dir_all(&root)?;

        let workspace = Self { root };
        workspace.ensure_structure()?;

        Ok(workspace)
    }

    /// Find workspace by searching upward from current directory.
    fn find_workspace_upward() -> Result<Option<PathBuf>> {
        let current = std::env::current_dir()?;
        let mut path = current.as_path();

        loop {
            let radium_dir = path.join(DIR_RADIUM);
            if radium_dir.exists() && radium_dir.is_dir() {
                return Ok(Some(path.to_path_buf()));
            }

            match path.parent() {
                Some(parent) => path = parent,
                None => break,
            }
        }

        Ok(None)
    }

    /// Get the root directory of the workspace.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Check if workspace has valid structure.
    pub fn is_valid(&self) -> bool {
        let radium_dir = self.root.join(DIR_RADIUM);
        radium_dir.exists() && radium_dir.is_dir()
    }

    /// Ensure workspace structure exists and is complete.
    ///
    /// Creates all required directories if they don't exist.
    ///
    /// # Errors
    ///
    /// Returns error if directories cannot be created.
    pub fn ensure_structure(&self) -> Result<()> {
        let structure = WorkspaceStructure::new(&self.root);
        structure.create_all()?;
        Ok(())
    }

    /// Get the workspace structure accessor.
    pub fn structure(&self) -> WorkspaceStructure {
        WorkspaceStructure::new(&self.root)
    }

    /// Get the path to the internal _internals directory.
    pub fn radium_dir(&self) -> PathBuf {
        self.root.join(DIR_RADIUM)
    }

    /// Get the path to a specific stage directory.
    pub fn stage_dir(&self, stage: &str) -> PathBuf {
        self.root.join(DIR_RADIUM).join(DIR_PLAN).join(stage)
    }

    /// Get all stage directories.
    pub fn stage_dirs(&self) -> Vec<PathBuf> {
        vec![
            self.stage_dir(STAGE_BACKLOG),
            self.stage_dir(STAGE_DEVELOPMENT),
            self.stage_dir(STAGE_REVIEW),
            self.stage_dir(STAGE_TESTING),
            self.stage_dir(STAGE_DOCS),
        ]
    }

    /// Check if workspace is empty (no plans).
    pub fn is_empty(&self) -> Result<bool> {
        for stage_dir in self.stage_dirs() {
            if !stage_dir.exists() {
                continue;
            }

            for entry in std::fs::read_dir(&stage_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    // Check if it looks like a plan directory (REQ-XXX format)
                    if let Some(name) = entry.file_name().to_str() {
                        if name.starts_with("REQ-") {
                            return Ok(false);
                        }
                    }
                }
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_workspace() {
        let temp = TempDir::new().unwrap();
        let workspace = Workspace::create(temp.path()).unwrap();

        assert!(workspace.is_valid());
        assert!(workspace.radium_dir().exists());
        assert!(workspace.root.join(DIR_RADIUM).exists());
    }

    #[test]
    fn test_discover_workspace() {
        let temp = TempDir::new().unwrap();
        let workspace = Workspace::create(temp.path()).unwrap();

        // Change to workspace directory
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(&workspace.root).unwrap();

        let discovered = Workspace::discover().unwrap();

        // Canonicalize paths to handle symlinks (e.g., /var vs /private/var on macOS)
        let expected = workspace.root().canonicalize().unwrap();
        let actual = discovered.root().canonicalize().unwrap();
        assert_eq!(actual, expected);

        // Restore original directory
        std::env::set_current_dir(original).unwrap();
    }

    #[test]
    fn test_workspace_structure() {
        let temp = TempDir::new().unwrap();
        let workspace = Workspace::create(temp.path()).unwrap();

        let stage_dirs = workspace.stage_dirs();
        assert_eq!(stage_dirs.len(), 5);

        for dir in stage_dirs {
            assert!(dir.exists());
            assert!(dir.to_string_lossy().contains(".radium/plan"));
        }
    }

    #[test]
    fn test_workspace_is_empty() {
        let temp = TempDir::new().unwrap();
        let workspace = Workspace::create(temp.path()).unwrap();

        assert!(workspace.is_empty().unwrap());

        // Create a plan directory
        let plan_dir = workspace.stage_dir(STAGE_BACKLOG).join("REQ-001-test");
        std::fs::create_dir_all(&plan_dir).unwrap();

        assert!(!workspace.is_empty().unwrap());
    }

    #[test]
    fn test_workspace_is_valid_false_on_nonexistent() {
        let temp = TempDir::new().unwrap();
        let workspace = Workspace { root: temp.path().join("nonexistent") };

        assert!(!workspace.is_valid());
    }

    #[test]
    fn test_workspace_is_valid_false_on_invalid() {
        let temp = TempDir::new().unwrap();
        // Create temp directory but don't create .radium subdirectory
        let workspace = Workspace { root: temp.path().to_path_buf() };

        assert!(!workspace.is_valid());
    }

    #[test]
    fn test_workspace_discover_with_create_if_missing() {
        let temp = TempDir::new().unwrap();
        let workspace_root = temp.path().join("new-workspace");

        let config =
            WorkspaceConfig { root: Some(workspace_root.clone()), create_if_missing: true };

        let workspace = Workspace::discover_with_config(&config).unwrap();

        assert!(workspace.is_valid());
        assert!(workspace_root.exists());
    }

    #[test]
    fn test_workspace_discover_without_create_if_missing_error() {
        let temp = TempDir::new().unwrap();
        let workspace_root = temp.path().join("nonexistent");

        let config = WorkspaceConfig { root: Some(workspace_root), create_if_missing: false };

        let result = Workspace::discover_with_config(&config);

        assert!(result.is_err());
    }

    #[test]
    fn test_workspace_stage_dir_for_all_stages() {
        let temp = TempDir::new().unwrap();
        let workspace = Workspace::create(temp.path()).unwrap();

        let backlog = workspace.stage_dir(STAGE_BACKLOG);
        let development = workspace.stage_dir(STAGE_DEVELOPMENT);
        let review = workspace.stage_dir(STAGE_REVIEW);
        let testing = workspace.stage_dir(STAGE_TESTING);
        let docs = workspace.stage_dir(STAGE_DOCS);

        assert!(backlog.to_string_lossy().contains("backlog"));
        assert!(development.to_string_lossy().contains("development"));
        assert!(review.to_string_lossy().contains("review"));
        assert!(testing.to_string_lossy().contains("testing"));
        assert!(docs.to_string_lossy().contains("docs"));
    }

    #[test]
    fn test_workspace_is_empty_with_non_req_directory() {
        let temp = TempDir::new().unwrap();
        let workspace = Workspace::create(temp.path()).unwrap();

        // Create a directory that doesn't start with REQ-
        let non_req_dir = workspace.stage_dir(STAGE_BACKLOG).join("other-directory");
        std::fs::create_dir_all(&non_req_dir).unwrap();

        // Workspace should still be considered empty
        assert!(workspace.is_empty().unwrap());
    }
}
