//! Workspace initialization and management for TUI.
//!
//! Handles automatic workspace creation and discovery on TUI startup.

use anyhow::Result;
use radium_core::workspace::{Workspace, WorkspaceConfig};
use std::path::PathBuf;

/// Workspace status information.
#[derive(Debug, Clone)]
pub struct WorkspaceStatus {
    /// Whether workspace exists
    pub exists: bool,
    /// Workspace root path
    pub root: Option<PathBuf>,
    /// Whether workspace is valid
    pub is_valid: bool,
    /// Error message if workspace check failed
    pub error: Option<String>,
}

impl WorkspaceStatus {
    /// Check workspace status.
    pub fn check() -> Self {
        match Workspace::discover() {
            Ok(workspace) => {
                let root = workspace.root().to_path_buf();
                let is_valid = workspace.is_valid();
                Self {
                    exists: true,
                    root: Some(root),
                    is_valid,
                    error: if !is_valid {
                        Some("Workspace structure is invalid".to_string())
                    } else {
                        None
                    },
                }
            }
            Err(e) => Self {
                exists: false,
                root: None,
                is_valid: false,
                error: Some(e.to_string()),
            },
        }
    }

    /// Initialize workspace at home directory.
    ///
    /// Creates `~/.radium/` structure if it doesn't exist.
    pub fn initialize_home() -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
        let radium_home = home.join(".radium");

        // Create workspace if it doesn't exist
        let workspace = if radium_home.exists() {
            Workspace::discover_with_config(&WorkspaceConfig {
                root: Some(radium_home.clone()),
                create_if_missing: false,
            })?
        } else {
            Workspace::create(&radium_home)?
        };

        // Ensure structure is complete
        workspace.ensure_structure()?;

        Ok(Self {
            exists: true,
            root: Some(workspace.root().to_path_buf()),
            is_valid: true,
            error: None,
        })
    }

    /// Get display message for workspace status.
    pub fn display_message(&self) -> String {
        if let Some(root) = &self.root {
            if self.is_valid {
                format!("Workspace: {}", root.display())
            } else {
                format!("Workspace: {} (invalid)", root.display())
            }
        } else {
            "No workspace found".to_string()
        }
    }
}

/// Initialize workspace on TUI startup.
///
/// Checks for existing workspace, creates one at ~/.radium/ if needed.
pub fn initialize_workspace() -> Result<WorkspaceStatus> {
    let status = WorkspaceStatus::check();

    // If no workspace found, create one at home directory
    if !status.exists {
        WorkspaceStatus::initialize_home()
    } else if !status.is_valid {
        // Try to fix invalid workspace
        if let Some(root) = &status.root {
            let workspace = Workspace::discover_with_config(&WorkspaceConfig {
                root: Some(root.clone()),
                create_if_missing: true,
            })?;
            workspace.ensure_structure()?;
            Ok(WorkspaceStatus {
                exists: true,
                root: Some(workspace.root().to_path_buf()),
                is_valid: true,
                error: None,
            })
        } else {
            Ok(status)
        }
    } else {
        Ok(status)
    }
}

