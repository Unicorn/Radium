//! Core types for workflow behaviors.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Action that an agent can request via behavior.json.
///
/// Agents write this JSON to `radium/.radium/memory/behavior.json` to control
/// workflow execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BehaviorActionType {
    /// Repeat previous steps (loop behavior).
    Loop,
    /// Trigger another agent dynamically.
    Trigger,
    /// Pause workflow execution for manual intervention.
    Checkpoint,
    /// Continue normal execution.
    Continue,
    /// Stop the current loop.
    Stop,
    /// Request metacognitive oversight (vibe check).
    VibeCheck,
}

/// Full behavior action with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BehaviorAction {
    /// The action type to perform.
    pub action: BehaviorActionType,
    /// Human-readable reason for this action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Agent ID to trigger (required when action is Trigger).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_agent_id: Option<String>,
}

impl BehaviorAction {
    /// Creates a new behavior action.
    pub fn new(action: BehaviorActionType) -> Self {
        Self { action, reason: None, trigger_agent_id: None }
    }

    /// Adds a reason to the behavior action.
    #[must_use]
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Sets the trigger agent ID (for Trigger actions).
    #[must_use]
    pub fn with_trigger_agent(mut self, agent_id: impl Into<String>) -> Self {
        self.trigger_agent_id = Some(agent_id.into());
        self
    }

    /// Reads behavior action from a JSON file.
    ///
    /// # Arguments
    /// * `path` - Path to behavior.json
    ///
    /// # Returns
    /// `Ok(Some(BehaviorAction))` if file exists and is valid,
    /// `Ok(None)` if file doesn't exist,
    /// `Err(BehaviorError)` if file is invalid.
    pub fn read_from_file(path: impl AsRef<Path>) -> Result<Option<Self>, BehaviorError> {
        let path = path.as_ref();

        match std::fs::read_to_string(path) {
            Ok(content) => {
                let action: BehaviorAction = serde_json::from_str(&content).map_err(|e| {
                    BehaviorError::ParseError { path: path.to_path_buf(), source: e }
                })?;
                Ok(Some(action))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // No behavior file = no special behavior
                Ok(None)
            }
            Err(e) => Err(BehaviorError::IoError { path: path.to_path_buf(), source: e }),
        }
    }

    /// Writes behavior action to a JSON file.
    ///
    /// # Arguments
    /// * `path` - Path to write behavior.json
    ///
    /// # Returns
    /// `Ok(())` if successful, `Err(BehaviorError)` otherwise.
    pub fn write_to_file(&self, path: impl AsRef<Path>) -> Result<(), BehaviorError> {
        let path = path.as_ref();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| BehaviorError::IoError { path: parent.to_path_buf(), source: e })?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| BehaviorError::SerializeError { source: e })?;

        std::fs::write(path, content)
            .map_err(|e| BehaviorError::IoError { path: path.to_path_buf(), source: e })
    }

    /// Deletes the behavior file if it exists.
    ///
    /// # Arguments
    /// * `path` - Path to behavior.json
    ///
    /// # Returns
    /// `Ok(())` if successful or file doesn't exist, `Err(BehaviorError)` on other errors.
    pub fn delete_file(path: impl AsRef<Path>) -> Result<(), BehaviorError> {
        let path = path.as_ref();

        match std::fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(BehaviorError::IoError { path: path.to_path_buf(), source: e }),
        }
    }
}

/// Trait for behavior evaluators.
///
/// Each behavior type (loop, trigger, checkpoint) implements this trait
/// to evaluate whether the behavior should be triggered.
pub trait BehaviorEvaluator {
    /// Decision type returned by this evaluator.
    type Decision;

    /// Evaluates whether the behavior should be triggered.
    ///
    /// # Arguments
    /// * `behavior_file` - Path to behavior.json
    /// * `output` - Output from the agent that just executed
    /// * `context` - Additional context for evaluation
    ///
    /// # Returns
    /// `Ok(Some(Decision))` if behavior triggered,
    /// `Ok(None)` if no behavior,
    /// `Err(BehaviorError)` on evaluation error.
    fn evaluate(
        &self,
        behavior_file: &Path,
        output: &str,
        context: &dyn std::any::Any,
    ) -> Result<Option<Self::Decision>, BehaviorError>;
}

/// Errors that can occur during behavior evaluation.
#[derive(Error, Debug)]
pub enum BehaviorError {
    /// Failed to read behavior file.
    #[error("Failed to read behavior file at {path}: {source}")]
    IoError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Failed to parse behavior.json.
    #[error("Failed to parse behavior file at {path}: {source}")]
    ParseError {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    /// Failed to serialize behavior action.
    #[error("Failed to serialize behavior action: {source}")]
    SerializeError {
        #[source]
        source: serde_json::Error,
    },

    /// Invalid behavior configuration.
    #[error("Invalid behavior configuration: {0}")]
    InvalidConfig(String),

    /// Missing required field.
    #[error("Missing required field: {0}")]
    MissingField(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_behavior_action_new() {
        let action = BehaviorAction::new(BehaviorActionType::Loop);
        assert_eq!(action.action, BehaviorActionType::Loop);
        assert!(action.reason.is_none());
        assert!(action.trigger_agent_id.is_none());
    }

    #[test]
    fn test_behavior_action_with_reason() {
        let action = BehaviorAction::new(BehaviorActionType::Loop)
            .with_reason("Need to fix compilation errors");

        assert_eq!(action.action, BehaviorActionType::Loop);
        assert_eq!(action.reason.as_deref(), Some("Need to fix compilation errors"));
    }

    #[test]
    fn test_behavior_action_with_trigger_agent() {
        let action = BehaviorAction::new(BehaviorActionType::Trigger)
            .with_trigger_agent("fix-agent")
            .with_reason("Detected error");

        assert_eq!(action.action, BehaviorActionType::Trigger);
        assert_eq!(action.trigger_agent_id.as_deref(), Some("fix-agent"));
        assert_eq!(action.reason.as_deref(), Some("Detected error"));
    }

    #[test]
    fn test_behavior_action_read_nonexistent_file() {
        let result = BehaviorAction::read_from_file("/nonexistent/path/behavior.json");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_behavior_action_write_and_read() {
        let temp_dir = TempDir::new().unwrap();
        let behavior_file = temp_dir.path().join("behavior.json");

        let original = BehaviorAction::new(BehaviorActionType::Checkpoint)
            .with_reason("Manual review required");

        // Write
        original.write_to_file(&behavior_file).unwrap();

        // Read back
        let loaded = BehaviorAction::read_from_file(&behavior_file)
            .unwrap()
            .expect("Should have loaded behavior");

        assert_eq!(loaded.action, BehaviorActionType::Checkpoint);
        assert_eq!(loaded.reason.as_deref(), Some("Manual review required"));
    }

    #[test]
    fn test_behavior_action_delete_file() {
        let temp_dir = TempDir::new().unwrap();
        let behavior_file = temp_dir.path().join("behavior.json");

        // Write file
        let action = BehaviorAction::new(BehaviorActionType::Continue);
        action.write_to_file(&behavior_file).unwrap();
        assert!(behavior_file.exists());

        // Delete file
        BehaviorAction::delete_file(&behavior_file).unwrap();
        assert!(!behavior_file.exists());

        // Deleting nonexistent file should succeed
        BehaviorAction::delete_file(&behavior_file).unwrap();
    }

    #[test]
    fn test_behavior_action_serialization() {
        let action = BehaviorAction::new(BehaviorActionType::Loop).with_reason("Fixing tests");

        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("\"action\":\"loop\""));
        assert!(json.contains("\"reason\":\"Fixing tests\""));

        let deserialized: BehaviorAction = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.action, BehaviorActionType::Loop);
        assert_eq!(deserialized.reason.as_deref(), Some("Fixing tests"));
    }
}
