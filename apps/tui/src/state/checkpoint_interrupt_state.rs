//! State management for checkpoint and interrupt moments during workflow execution.

use std::time::SystemTime;

/// Trigger type for workflow interrupts.
#[derive(Debug, Clone, PartialEq)]
pub enum InterruptTrigger {
    /// Agent checkpoint behavior triggered
    AgentCheckpoint {
        /// Reason for the checkpoint
        reason: String,
        /// Agent ID that triggered the checkpoint
        agent_id: String,
    },
    /// Policy engine AskUser decision
    PolicyAskUser {
        /// Tool name that requires user approval
        tool_name: String,
        /// Tool arguments
        args: String,
        /// Reason for asking user
        reason: String,
    },
    /// Error condition requiring user intervention
    Error {
        /// Error message
        message: String,
    },
}

/// Available actions for resolving an interrupt.
#[derive(Debug, Clone, PartialEq)]
pub enum InterruptAction {
    /// Resume workflow execution from current state
    Continue,
    /// Restore to previous checkpoint
    Rollback {
        /// Checkpoint ID to restore
        checkpoint_id: String,
    },
    /// Terminate workflow execution
    Cancel,
}

/// State for managing a single checkpoint/interrupt moment.
#[derive(Debug, Clone)]
pub struct CheckpointInterruptState {
    /// Whether interrupt is currently active
    pub active: bool,
    /// What caused the interrupt
    pub trigger: InterruptTrigger,
    /// Current checkpoint ID if available
    pub checkpoint_id: Option<String>,
    /// Associated workflow ID
    pub workflow_id: String,
    /// Current step in workflow
    pub step_number: usize,
    /// When interrupt occurred
    pub timestamp: SystemTime,
    /// User's selected action
    pub selected_action: Option<InterruptAction>,
    /// Index of selected action in available actions list
    pub selected_action_index: usize,
    /// Whether details view is expanded
    pub show_details: bool,
    /// Whether diff view is visible
    pub show_diff: bool,
    /// IDs of checkpoints available for rollback
    pub available_checkpoints: Vec<String>,
    /// Cached diff data (will use radium_core::checkpoint::CheckpointDiff when integrated)
    pub diff_data: Option<radium_core::checkpoint::CheckpointDiff>,
    /// Scroll offset for diff view
    pub diff_scroll_offset: usize,
}

impl CheckpointInterruptState {
    /// Creates a new checkpoint interrupt state.
    pub fn new(
        trigger: InterruptTrigger,
        workflow_id: String,
        step_number: usize,
    ) -> Self {
        Self {
            active: false,
            trigger,
            checkpoint_id: None,
            workflow_id,
            step_number,
            timestamp: SystemTime::now(),
            selected_action: None,
            selected_action_index: 0,
            show_details: false,
            show_diff: false,
            available_checkpoints: Vec::new(),
            diff_data: None,
            diff_scroll_offset: 0,
        }
    }

    /// Activates the interrupt with optional checkpoint ID.
    pub fn activate(&mut self, checkpoint_id: Option<String>) {
        self.active = true;
        self.checkpoint_id = checkpoint_id;
        self.timestamp = SystemTime::now();
        // Default to Continue action
        self.selected_action = Some(InterruptAction::Continue);
        self.selected_action_index = 0;
    }

    /// Deactivates the interrupt.
    pub fn deactivate(&mut self) {
        self.active = false;
        self.selected_action = None;
        self.selected_action_index = 0;
        self.show_details = false;
        self.show_diff = false;
    }

    /// Checks if interrupt is currently active.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Gets the list of available actions based on state.
    pub fn available_actions(&self) -> Vec<InterruptAction> {
        // Policy AskUser interrupts use Approve/Deny instead of Continue/Cancel
        if matches!(self.trigger, InterruptTrigger::PolicyAskUser { .. }) {
            // For policy interrupts, we'll use Continue as Approve and Cancel as Deny
            // The UI will display them differently
            return vec![InterruptAction::Continue, InterruptAction::Cancel];
        }

        let mut actions = vec![InterruptAction::Continue];
        
        if self.can_rollback() {
            if let Some(checkpoint_id) = &self.checkpoint_id {
                actions.push(InterruptAction::Rollback {
                    checkpoint_id: checkpoint_id.clone(),
                });
            } else if let Some(first_checkpoint) = self.available_checkpoints.first() {
                actions.push(InterruptAction::Rollback {
                    checkpoint_id: first_checkpoint.clone(),
                });
            }
        }
        
        actions.push(InterruptAction::Cancel);
        actions
    }

    /// Selects an action.
    pub fn select_action(&mut self, action: InterruptAction) {
        self.selected_action = Some(action.clone());
        // Update index to match selected action
        let actions = self.available_actions();
        if let Some(index) = actions.iter().position(|a| a == &action) {
            self.selected_action_index = index;
        }
    }

    /// Selects the next action in the list.
    pub fn select_next_action(&mut self) {
        let actions = self.available_actions();
        if !actions.is_empty() {
            self.selected_action_index = (self.selected_action_index + 1) % actions.len();
            self.selected_action = Some(actions[self.selected_action_index].clone());
        }
    }

    /// Selects the previous action in the list.
    pub fn select_previous_action(&mut self) {
        let actions = self.available_actions();
        if !actions.is_empty() {
            self.selected_action_index = if self.selected_action_index == 0 {
                actions.len() - 1
            } else {
                self.selected_action_index - 1
            };
            self.selected_action = Some(actions[self.selected_action_index].clone());
        }
    }

    /// Gets the currently selected action.
    pub fn get_selected_action(&self) -> Option<&InterruptAction> {
        self.selected_action.as_ref()
    }

    /// Toggles the details view.
    pub fn toggle_details(&mut self) {
        self.show_details = !self.show_details;
    }

    /// Toggles the diff view and fetches diff if needed.
    pub fn toggle_diff(&mut self) {
        self.show_diff = !self.show_diff;
        // Diff will be fetched when needed via fetch_diff method
    }

    /// Checks if rollback is available.
    pub fn can_rollback(&self) -> bool {
        self.checkpoint_id.is_some() || !self.available_checkpoints.is_empty()
    }

    /// Fetches diff data from checkpoint manager.
    pub fn fetch_diff(&mut self, checkpoint_manager: &radium_core::checkpoint::CheckpointManager) -> Result<(), String> {
        // Get current and previous checkpoint IDs
        let current_id = match &self.checkpoint_id {
            Some(id) => id.clone(),
            None => {
                // Try to get latest checkpoint
                if let Some(latest) = checkpoint_manager.list_checkpoints().ok()
                    .and_then(|checkpoints| checkpoints.last()) {
                    latest.id.clone()
                } else {
                    return Err("No checkpoint available for diff".to_string());
                }
            }
        };

        // Get previous checkpoint (one before current)
        let checkpoints = checkpoint_manager.list_checkpoints()
            .map_err(|e| format!("Failed to list checkpoints: {}", e))?;
        
        let prev_id = if checkpoints.len() >= 2 {
            // Find current checkpoint index
            let current_idx = checkpoints.iter()
                .position(|cp| cp.id == current_id)
                .unwrap_or(checkpoints.len() - 1);
            
            if current_idx > 0 {
                Some(checkpoints[current_idx - 1].id.clone())
            } else {
                None
            }
        } else {
            None
        };

        // Get diff between checkpoints
        if let Some(prev_id) = prev_id {
            match checkpoint_manager.diff_checkpoints(&prev_id, &current_id) {
                Ok(diff) => {
                    self.diff_data = Some(diff);
                    Ok(())
                }
                Err(e) => Err(format!("Failed to get diff: {}", e)),
            }
        } else {
            // No previous checkpoint - get diff from current to workspace state
            match checkpoint_manager.diff_checkpoint(&current_id) {
                Ok(diff_text) => {
                    // Create a simple CheckpointDiff from the text
                    let diff = radium_core::checkpoint::CheckpointDiff {
                        added: Vec::new(),
                        modified: Vec::new(),
                        deleted: Vec::new(),
                        raw_diff: diff_text,
                        insertions: 0,
                        deletions: 0,
                    };
                    self.diff_data = Some(diff);
                    Ok(())
                }
                Err(e) => Err(format!("Failed to get diff: {}", e)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interrupt_state_new() {
        let state = CheckpointInterruptState::new(
            InterruptTrigger::AgentCheckpoint {
                reason: "Test".to_string(),
                agent_id: "agent-1".to_string(),
            },
            "workflow-1".to_string(),
            5,
        );

        assert!(!state.is_active());
        assert_eq!(state.workflow_id, "workflow-1");
        assert_eq!(state.step_number, 5);
        assert!(!state.show_details);
        assert!(!state.show_diff);
    }

    #[test]
    fn test_interrupt_state_activate() {
        let mut state = CheckpointInterruptState::new(
            InterruptTrigger::AgentCheckpoint {
                reason: "Test".to_string(),
                agent_id: "agent-1".to_string(),
            },
            "workflow-1".to_string(),
            5,
        );

        state.activate(Some("checkpoint-1".to_string()));
        assert!(state.is_active());
        assert_eq!(state.checkpoint_id, Some("checkpoint-1".to_string()));
        assert!(state.selected_action.is_some());
    }

    #[test]
    fn test_interrupt_state_deactivate() {
        let mut state = CheckpointInterruptState::new(
            InterruptTrigger::AgentCheckpoint {
                reason: "Test".to_string(),
                agent_id: "agent-1".to_string(),
            },
            "workflow-1".to_string(),
            5,
        );

        state.activate(Some("checkpoint-1".to_string()));
        assert!(state.is_active());

        state.deactivate();
        assert!(!state.is_active());
        assert!(state.selected_action.is_none());
    }

    #[test]
    fn test_available_actions() {
        let mut state = CheckpointInterruptState::new(
            InterruptTrigger::AgentCheckpoint {
                reason: "Test".to_string(),
                agent_id: "agent-1".to_string(),
            },
            "workflow-1".to_string(),
            5,
        );

        // Without checkpoint, should have Continue and Cancel
        let actions = state.available_actions();
        assert_eq!(actions.len(), 2);
        assert!(matches!(actions[0], InterruptAction::Continue));
        assert!(matches!(actions[1], InterruptAction::Cancel));

        // With checkpoint, should have Continue, Rollback, and Cancel
        state.activate(Some("checkpoint-1".to_string()));
        let actions = state.available_actions();
        assert_eq!(actions.len(), 3);
        assert!(matches!(actions[0], InterruptAction::Continue));
        assert!(matches!(actions[1], InterruptAction::Rollback { .. }));
        assert!(matches!(actions[2], InterruptAction::Cancel));
    }

    #[test]
    fn test_action_selection() {
        let mut state = CheckpointInterruptState::new(
            InterruptTrigger::AgentCheckpoint {
                reason: "Test".to_string(),
                agent_id: "agent-1".to_string(),
            },
            "workflow-1".to_string(),
            5,
        );

        state.activate(Some("checkpoint-1".to_string()));

        // Test next action
        state.select_next_action();
        let actions = state.available_actions();
        assert_eq!(state.selected_action_index, 1);
        assert!(matches!(state.get_selected_action(), Some(InterruptAction::Rollback { .. })));

        // Test previous action
        state.select_previous_action();
        assert_eq!(state.selected_action_index, 0);
        assert!(matches!(state.get_selected_action(), Some(InterruptAction::Continue)));
    }

    #[test]
    fn test_toggle_views() {
        let mut state = CheckpointInterruptState::new(
            InterruptTrigger::AgentCheckpoint {
                reason: "Test".to_string(),
                agent_id: "agent-1".to_string(),
            },
            "workflow-1".to_string(),
            5,
        );

        assert!(!state.show_details);
        state.toggle_details();
        assert!(state.show_details);
        state.toggle_details();
        assert!(!state.show_details);

        assert!(!state.show_diff);
        state.toggle_diff();
        assert!(state.show_diff);
    }

    #[test]
    fn test_can_rollback() {
        let mut state = CheckpointInterruptState::new(
            InterruptTrigger::AgentCheckpoint {
                reason: "Test".to_string(),
                agent_id: "agent-1".to_string(),
            },
            "workflow-1".to_string(),
            5,
        );

        assert!(!state.can_rollback());

        state.activate(Some("checkpoint-1".to_string()));
        assert!(state.can_rollback());

        state.checkpoint_id = None;
        state.available_checkpoints.push("cp-1".to_string());
        assert!(state.can_rollback());
    }
}

