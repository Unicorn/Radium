//! Core types for workflow behaviors.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use thiserror::Error;
use tokio::sync::watch;
use tokio::time::sleep;
use tracing::{debug, warn};

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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

    /// File watching error.
    #[error("File watching error: {0}")]
    WatchError(String),
}

/// File watcher for behavior.json with debouncing and hot reload.
///
/// Watches for changes to behavior.json and automatically reloads
/// the behavior action with a 100ms debounce to handle rapid changes.
pub struct BehaviorFileWatcher {
    /// Path to behavior.json file.
    file_path: PathBuf,
    /// Current behavior state (thread-safe).
    behavior_state: Arc<RwLock<Option<BehaviorAction>>>,
    /// Event receiver for file change notifications.
    _receiver: watch::Receiver<()>,
    /// Event sender for file change notifications.
    _sender: watch::Sender<()>,
    /// Background task handle.
    _task_handle: Option<tokio::task::JoinHandle<()>>,
    /// Notify watcher (must be kept alive).
    _watcher: Option<std::sync::Mutex<Box<dyn std::any::Any + Send>>>,
}

impl BehaviorFileWatcher {
    /// Creates a new behavior file watcher.
    ///
    /// # Arguments
    /// * `file_path` - Path to behavior.json file
    ///
    /// # Returns
    /// New BehaviorFileWatcher instance (not started yet).
    pub fn new(file_path: impl Into<PathBuf>) -> Self {
        let file_path = file_path.into();
        let behavior_state = Arc::new(RwLock::new(None));
        let (sender, receiver) = watch::channel(());

        Self {
            file_path,
            behavior_state,
            _receiver: receiver,
            _sender: sender,
            _task_handle: None,
            _watcher: None,
        }
    }

    /// Starts watching the behavior file for changes.
    ///
    /// This spawns a background task that watches for file changes,
    /// debounces them, and reloads the behavior action.
    ///
    /// # Returns
    /// `Ok(())` if watcher started successfully, or error if initialization failed.
    pub fn start(&mut self) -> Result<(), BehaviorError> {
        use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
        use std::sync::mpsc;

        let file_path = self.file_path.clone();
        let behavior_state = Arc::clone(&self.behavior_state);

        // Create channel for file events
        let (tx, rx) = mpsc::channel();

        // Create watcher with config
        let mut watcher = RecommendedWatcher::new(
            move |result: notify::Result<notify::Event>| {
                match result {
                    Ok(event) => {
                        // Only process Create and Modify events for the behavior file
                        match event.kind {
                            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                                if event.paths.iter().any(|p| p.ends_with("behavior.json")) {
                                    let _ = tx.send(());
                                }
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        warn!(error = %e, "Error watching behavior.json file");
                    }
                }
            },
            Config::default(),
        )
        .map_err(|e| BehaviorError::WatchError(format!("Failed to create watcher: {}", e)))?;

        // Start watching the parent directory (behavior.json may not exist yet)
        if let Some(parent) = self.file_path.parent() {
            watcher
                .watch(parent, RecursiveMode::NonRecursive)
                .map_err(|e| {
                    BehaviorError::WatchError(format!("Failed to watch directory: {}", e))
                })?;
        }

        // Spawn background task for debouncing and reloading
        let behavior_state_clone = Arc::clone(&behavior_state);
        let file_path_clone = self.file_path.clone();
        let (mut tx_watch, mut rx_watch) = watch::channel(());
        
        // Spawn task to forward file system events to tokio
        let watcher_tx = tx_watch.clone();
        tokio::spawn(async move {
            while let Ok(()) = rx.recv() {
                let _ = watcher_tx.send(());
            }
        });

        let task_handle = tokio::spawn(async move {
            let mut last_event = None::<tokio::time::Instant>;

            loop {
                tokio::select! {
                    // Wait for file change event
                    _ = rx_watch.changed() => {
                        let now = tokio::time::Instant::now();
                        last_event = Some(now);

                        // Wait for debounce period
                        sleep(Duration::from_millis(100)).await;

                        // Check if this is still the most recent event
                        if let Some(event_time) = last_event {
                            if event_time.elapsed() >= Duration::from_millis(100) {
                                // Reload behavior file
                                Self::reload_behavior(&file_path_clone, &behavior_state_clone);
                            }
                        }
                    }
                }
            }
        });

        self._task_handle = Some(task_handle);
        self._sender = tx_watch;
        
        // Store watcher to keep it alive (wrap in Any to allow different watcher types)
        self._watcher = Some(std::sync::Mutex::new(Box::new(watcher) as Box<dyn std::any::Any + Send>));

        // Store watcher to keep it alive
        // Note: In a real implementation, we'd store the watcher in the struct
        // For now, we'll let it be dropped but the channel receiver keeps it alive

        // Load initial behavior if file exists
        Self::reload_behavior(&self.file_path, &self.behavior_state);

        debug!(file_path = %self.file_path.display(), "Started watching behavior.json file");
        Ok(())
    }

    /// Stops watching the behavior file.
    pub fn stop(&mut self) {
        if let Some(handle) = self._task_handle.take() {
            handle.abort();
        }
        debug!(file_path = %self.file_path.display(), "Stopped watching behavior.json file");
    }

    /// Reloads behavior from file (called internally after debounce).
    fn reload_behavior(file_path: &Path, behavior_state: &Arc<RwLock<Option<BehaviorAction>>>) {
        match BehaviorAction::read_from_file(file_path) {
            Ok(Some(new_action)) => {
                let mut state = behavior_state.write().unwrap();
                let old_action = state.clone();
                *state = Some(new_action.clone());

                drop(state); // Release lock before logging

                if let Some(ref old) = old_action {
                    if old != &new_action {
                        debug!(
                            file_path = %file_path.display(),
                            old_action = ?old.action,
                            new_action = ?new_action.action,
                            "Behavior file changed"
                        );
                    }
                } else {
                    debug!(
                        file_path = %file_path.display(),
                        action = ?new_action.action,
                        "Behavior file created"
                    );
                }
            }
            Ok(None) => {
                // File doesn't exist or was deleted - revert to None (Continue behavior)
                let mut state = behavior_state.write().unwrap();
                if state.is_some() {
                    debug!(
                        file_path = %file_path.display(),
                        "Behavior file deleted, reverting to default"
                    );
                    *state = None;
                }
            }
            Err(e) => {
                warn!(
                    file_path = %file_path.display(),
                    error = %e,
                    "Failed to reload behavior file, keeping previous state"
                );
            }
        }
    }

    /// Gets the current behavior state.
    ///
    /// # Returns
    /// Current behavior action, or None if no behavior file exists.
    pub fn get_current_behavior(&self) -> Option<BehaviorAction> {
        self.behavior_state.read().unwrap().clone()
    }
}

impl Drop for BehaviorFileWatcher {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Helper function to record behavior metrics to telemetry.
///
/// # Arguments
/// * `monitoring_service` - Optional monitoring service
/// * `agent_id` - Agent ID executing the behavior
/// * `behavior_type` - Type of behavior (loop, trigger, checkpoint, vibecheck)
/// * `invocation_count` - Number of times behavior was invoked
/// * `duration_ms` - Evaluation duration in milliseconds
/// * `outcome` - Outcome of the behavior evaluation (triggered, skipped, failed, etc.)
pub async fn record_behavior_metrics(
    monitoring_service: Option<&Arc<std::sync::Mutex<crate::monitoring::service::MonitoringService>>>,
    agent_id: String,
    behavior_type: String,
    invocation_count: Option<u64>,
    duration_ms: Option<u64>,
    outcome: Option<String>,
) {
    if let Some(monitoring) = monitoring_service {
        use crate::monitoring::telemetry::TelemetryRecord;
        
        let record = TelemetryRecord::new(agent_id)
            .with_behavior_metrics(behavior_type, invocation_count, duration_ms, outcome);
        
        if let Ok(mut service) = monitoring.lock() {
            let _ = service.record_telemetry(&record).await;
        }
    }
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
