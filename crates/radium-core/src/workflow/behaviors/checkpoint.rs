//! Checkpoint behavior implementation.
//!
//! Allows agents to pause workflow execution for manual review or intervention.
//! Any agent can trigger a checkpoint by writing a checkpoint action to behavior.json.

use serde::{Deserialize, Serialize};
use std::path::Path;

use super::types::{BehaviorAction, BehaviorActionType, BehaviorError, BehaviorEvaluator};

/// Decision result from checkpoint evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointDecision {
    /// Whether to stop the workflow.
    pub should_stop_workflow: bool,
    /// Human-readable reason for the checkpoint.
    pub reason: Option<String>,
}

/// Context for checkpoint evaluation (checkpoint is universal, needs no config).
#[derive(Debug, Clone, Default)]
pub struct CheckpointEvaluationContext;

impl CheckpointEvaluationContext {
    /// Creates a new checkpoint evaluation context.
    pub fn new() -> Self {
        Self
    }
}

/// Evaluates checkpoint behavior based on behavior.json.
///
/// Checkpoint is universal - any agent can write a checkpoint action
/// to pause workflow execution.
pub struct CheckpointEvaluator;

impl CheckpointEvaluator {
    /// Creates a new checkpoint evaluator.
    pub fn new() -> Self {
        Self
    }

    /// Evaluates checkpoint behavior.
    ///
    /// # Arguments
    /// * `behavior_file` - Path to behavior.json
    /// * `output` - Output from agent execution
    ///
    /// # Returns
    /// `Ok(Some(CheckpointDecision))` if checkpoint should be triggered,
    /// `Ok(None)` if no checkpoint behavior,
    /// `Err(BehaviorError)` on evaluation error.
    pub fn evaluate_checkpoint(
        &self,
        behavior_file: &Path,
        _output: &str,
    ) -> Result<Option<CheckpointDecision>, BehaviorError> {
        // Check for behavior action
        let Some(action) = BehaviorAction::read_from_file(behavior_file)? else {
            return Ok(None);
        };

        // Only handle checkpoint actions
        if action.action != BehaviorActionType::Checkpoint {
            return Ok(None);
        }

        Ok(Some(CheckpointDecision { should_stop_workflow: true, reason: action.reason }))
    }
}

impl Default for CheckpointEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl BehaviorEvaluator for CheckpointEvaluator {
    type Decision = CheckpointDecision;

    fn evaluate(
        &self,
        behavior_file: &Path,
        output: &str,
        _context: &dyn std::any::Any,
    ) -> Result<Option<Self::Decision>, BehaviorError> {
        self.evaluate_checkpoint(behavior_file, output)
    }
}

/// State for managing checkpoint UI and workflow pausing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointState {
    /// Whether a checkpoint is currently active.
    pub active: bool,
    /// Reason for the checkpoint.
    pub reason: Option<String>,
    /// When the checkpoint was triggered.
    pub triggered_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl CheckpointState {
    /// Creates a new inactive checkpoint state.
    pub fn new() -> Self {
        Self { active: false, reason: None, triggered_at: None }
    }

    /// Activates a checkpoint with a reason.
    pub fn activate(&mut self, reason: Option<String>) {
        self.active = true;
        self.reason = reason;
        self.triggered_at = Some(chrono::Utc::now());
    }

    /// Deactivates the checkpoint.
    pub fn deactivate(&mut self) {
        self.active = false;
        self.reason = None;
        self.triggered_at = None;
    }

    /// Checks if a checkpoint is currently active.
    pub fn is_active(&self) -> bool {
        self.active
    }
}

impl Default for CheckpointState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_checkpoint_evaluator_no_behavior_file() {
        let temp_dir = TempDir::new().unwrap();
        let behavior_file = temp_dir.path().join("behavior.json");

        let evaluator = CheckpointEvaluator::new();
        let result = evaluator.evaluate_checkpoint(&behavior_file, "");

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_checkpoint_evaluator_checkpoint_action() {
        let temp_dir = TempDir::new().unwrap();
        let behavior_file = temp_dir.path().join("behavior.json");

        // Write checkpoint action
        let action = BehaviorAction::new(BehaviorActionType::Checkpoint)
            .with_reason("Manual review required");
        action.write_to_file(&behavior_file).unwrap();

        let evaluator = CheckpointEvaluator::new();
        let result = evaluator.evaluate_checkpoint(&behavior_file, "").unwrap();

        assert!(result.is_some());
        let decision = result.unwrap();
        assert!(decision.should_stop_workflow);
        assert_eq!(decision.reason.as_deref(), Some("Manual review required"));
    }

    #[test]
    fn test_checkpoint_evaluator_non_checkpoint_action() {
        let temp_dir = TempDir::new().unwrap();
        let behavior_file = temp_dir.path().join("behavior.json");

        // Write loop action (should not trigger checkpoint)
        let action = BehaviorAction::new(BehaviorActionType::Loop);
        action.write_to_file(&behavior_file).unwrap();

        let evaluator = CheckpointEvaluator::new();
        let result = evaluator.evaluate_checkpoint(&behavior_file, "").unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_checkpoint_state_new() {
        let state = CheckpointState::new();
        assert!(!state.is_active());
        assert!(state.reason.is_none());
        assert!(state.triggered_at.is_none());
    }

    #[test]
    fn test_checkpoint_state_activate() {
        let mut state = CheckpointState::new();
        state.activate(Some("Need approval".to_string()));

        assert!(state.is_active());
        assert_eq!(state.reason.as_deref(), Some("Need approval"));
        assert!(state.triggered_at.is_some());
    }

    #[test]
    fn test_checkpoint_state_deactivate() {
        let mut state = CheckpointState::new();
        state.activate(Some("Test".to_string()));
        assert!(state.is_active());

        state.deactivate();
        assert!(!state.is_active());
        assert!(state.reason.is_none());
        assert!(state.triggered_at.is_none());
    }

    #[test]
    fn test_checkpoint_state_serialization() {
        let mut state = CheckpointState::new();
        state.activate(Some("Review needed".to_string()));

        let json = serde_json::to_string(&state).unwrap();
        let deserialized: CheckpointState = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.active, state.active);
        assert_eq!(deserialized.reason, state.reason);
    }
}
