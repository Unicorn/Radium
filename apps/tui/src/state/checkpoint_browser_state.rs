//! Checkpoint browser state for TUI checkpoint management.

use radium_core::checkpoint::{Checkpoint, CheckpointDiff, CheckpointManager};
use radium_core::workspace::Workspace;
use std::path::PathBuf;

/// State for the checkpoint browser view.
#[derive(Debug, Clone)]
pub struct CheckpointBrowserState {
    /// All available checkpoints
    pub checkpoints: Vec<Checkpoint>,
    /// Currently selected checkpoint index
    pub selected_index: usize,
    /// Diff preview for selected checkpoint (compared to current state)
    pub diff_preview: Option<CheckpointDiff>,
    /// Whether restore confirmation dialog is shown
    pub show_restore_confirmation: bool,
    /// Error message if any
    pub error: Option<String>,
    /// Checkpoint manager instance
    checkpoint_manager: Option<CheckpointManager>,
}

impl CheckpointBrowserState {
    /// Creates a new checkpoint browser state.
    pub fn new() -> Self {
        Self {
            checkpoints: Vec::new(),
            selected_index: 0,
            diff_preview: None,
            show_restore_confirmation: false,
            error: None,
            checkpoint_manager: None,
        }
    }

    /// Initializes the checkpoint manager and loads checkpoints.
    pub fn initialize(&mut self) -> Result<(), String> {
        let workspace = Workspace::discover()
            .map_err(|e| format!("No Radium workspace found: {}", e))?;

        let checkpoint_manager = CheckpointManager::new(workspace.root())
            .map_err(|e| format!("Failed to initialize checkpoint manager: {}", e))?;

        self.checkpoint_manager = Some(checkpoint_manager);
        self.load_checkpoints()
    }

    /// Loads all checkpoints from the checkpoint manager.
    pub fn load_checkpoints(&mut self) -> Result<(), String> {
        if let Some(ref cm) = self.checkpoint_manager {
            self.checkpoints = cm
                .list_checkpoints()
                .map_err(|e| format!("Failed to list checkpoints: {}", e))?;
            // Sort by timestamp (newest first)
            self.checkpoints.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            
            // Reset selection if out of bounds
            if !self.checkpoints.is_empty() && self.selected_index >= self.checkpoints.len() {
                self.selected_index = self.checkpoints.len() - 1;
            } else if self.checkpoints.is_empty() {
                self.selected_index = 0;
            }
            
            // Load diff for selected checkpoint
            self.load_diff_preview();
        }
        Ok(())
    }

    /// Loads diff preview for the selected checkpoint.
    pub fn load_diff_preview(&mut self) {
        if let Some(ref cm) = self.checkpoint_manager {
            if let Some(checkpoint) = self.checkpoints.get(self.selected_index) {
                self.diff_preview = cm.diff_checkpoints(&checkpoint.id, "HEAD").ok();
            }
        }
    }

    /// Gets the currently selected checkpoint.
    pub fn selected_checkpoint(&self) -> Option<&Checkpoint> {
        self.checkpoints.get(self.selected_index)
    }

    /// Moves selection up.
    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.load_diff_preview();
        }
    }

    /// Moves selection down.
    pub fn select_next(&mut self) {
        if self.selected_index < self.checkpoints.len().saturating_sub(1) {
            self.selected_index += 1;
            self.load_diff_preview();
        }
    }

    /// Shows the restore confirmation dialog.
    pub fn show_restore_dialog(&mut self) {
        if self.selected_checkpoint().is_some() {
            self.show_restore_confirmation = true;
        }
    }

    /// Hides the restore confirmation dialog.
    pub fn hide_restore_dialog(&mut self) {
        self.show_restore_confirmation = false;
    }

    /// Restores the selected checkpoint.
    pub fn restore_selected(&mut self) -> Result<(), String> {
        if let Some(ref cm) = self.checkpoint_manager {
            if let Some(checkpoint) = self.selected_checkpoint() {
                cm.restore_checkpoint(&checkpoint.id)
                    .map_err(|e| format!("Failed to restore checkpoint: {}", e))?;
                self.show_restore_confirmation = false;
                // Reload checkpoints after restore
                self.load_checkpoints()?;
                Ok(())
            } else {
                Err("No checkpoint selected".to_string())
            }
        } else {
            Err("Checkpoint manager not initialized".to_string())
        }
    }

    /// Sets an error message.
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    /// Clears the error message.
    pub fn clear_error(&mut self) {
        self.error = None;
    }
}

impl Default for CheckpointBrowserState {
    fn default() -> Self {
        Self::new()
    }
}

