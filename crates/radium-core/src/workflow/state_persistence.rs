//! State persistence and resume capability for interrupted executions.
//!
//! This module provides functionality to save execution state to disk and
//! resume from the last completed task when execution is interrupted.

use crate::workflow::execution_state::{ExecutionState, TaskResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Persisted execution state for resuming interrupted executions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedExecutionState {
    /// Requirement ID.
    pub requirement_id: String,
    /// Requirement title.
    pub requirement_title: String,
    /// Execution start time.
    pub started_at: DateTime<Utc>,
    /// Last checkpoint time.
    pub last_checkpoint_at: DateTime<Utc>,
    /// List of completed task IDs.
    pub completed_tasks: Vec<String>,
    /// List of failed task IDs.
    pub failed_tasks: Vec<String>,
    /// Task results map.
    pub task_results: HashMap<String, TaskResult>,
    /// Next tasks ready to execute.
    pub next_tasks: Vec<String>,
}

/// State persistence manager.
pub struct StatePersistence {
    /// Workspace root path.
    workspace_path: PathBuf,
}

impl StatePersistence {
    /// Creates a new state persistence manager.
    ///
    /// # Arguments
    /// * `workspace_path` - Path to the workspace root
    pub fn new(workspace_path: impl AsRef<Path>) -> Self {
        Self {
            workspace_path: workspace_path.as_ref().to_path_buf(),
        }
    }

    /// Saves execution state to disk.
    ///
    /// # Arguments
    /// * `req_id` - The requirement ID
    /// * `requirement_title` - The requirement title
    /// * `execution_state` - The execution state to save
    /// * `started_at` - When execution started
    /// * `next_tasks` - Tasks ready to execute next
    ///
    /// # Returns
    /// Ok(()) on success, error on failure
    pub fn save_state(
        &self,
        req_id: &str,
        requirement_title: &str,
        execution_state: &ExecutionState,
        started_at: DateTime<Utc>,
        next_tasks: Vec<String>,
    ) -> Result<(), std::io::Error> {
        // Create executions directory
        let executions_dir = self
            .workspace_path
            .join(".radium")
            .join("_internals")
            .join("executions");
        std::fs::create_dir_all(&executions_dir)?;

        // Extract task results
        let mut task_results = HashMap::new();
        for task_id in execution_state.completed_tasks().iter().chain(execution_state.failed_tasks().iter()) {
            if let Some(result) = execution_state.get_result(task_id) {
                task_results.insert(task_id.clone(), result);
            }
        }

        // Build persisted state
        let persisted_state = PersistedExecutionState {
            requirement_id: req_id.to_string(),
            requirement_title: requirement_title.to_string(),
            started_at,
            last_checkpoint_at: Utc::now(),
            completed_tasks: execution_state.completed_tasks(),
            failed_tasks: execution_state.failed_tasks(),
            task_results,
            next_tasks,
        };

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&persisted_state)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        // Write to file atomically (write to temp, then rename)
        let filename = format!("{}.json", req_id);
        let file_path = executions_dir.join(&filename);
        let temp_path = executions_dir.join(format!("{}.tmp", req_id));
        
        std::fs::write(&temp_path, json)?;
        std::fs::rename(&temp_path, &file_path)?;

        Ok(())
    }

    /// Loads execution state from disk.
    ///
    /// # Arguments
    /// * `req_id` - The requirement ID
    ///
    /// # Returns
    /// Some(persisted_state) if state exists, None otherwise
    pub fn load_state(&self, req_id: &str) -> Result<Option<PersistedExecutionState>, std::io::Error> {
        let executions_dir = self
            .workspace_path
            .join(".radium")
            .join("_internals")
            .join("executions");
        
        let filename = format!("{}.json", req_id);
        let file_path = executions_dir.join(&filename);

        if !file_path.exists() {
            return Ok(None);
        }

        // Read and parse JSON
        let json = std::fs::read_to_string(&file_path)?;
        let state: PersistedExecutionState = serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Failed to parse state: {}", e)))?;

        Ok(Some(state))
    }

    /// Deletes execution state from disk.
    ///
    /// # Arguments
    /// * `req_id` - The requirement ID
    ///
    /// # Returns
    /// Ok(()) on success, error on failure (ignores missing file)
    pub fn delete_state(&self, req_id: &str) -> Result<(), std::io::Error> {
        let executions_dir = self
            .workspace_path
            .join(".radium")
            .join("_internals")
            .join("executions");
        
        let filename = format!("{}.json", req_id);
        let file_path = executions_dir.join(&filename);

        if file_path.exists() {
            std::fs::remove_file(&file_path)?;
        }

        Ok(())
    }

    /// Lists all resumable requirement IDs.
    ///
    /// # Returns
    /// Vector of requirement IDs that have saved state
    pub fn list_resumable(&self) -> Result<Vec<String>, std::io::Error> {
        let executions_dir = self
            .workspace_path
            .join(".radium")
            .join("_internals")
            .join("executions");

        if !executions_dir.exists() {
            return Ok(vec![]);
        }

        let mut resumable = Vec::new();
        for entry in std::fs::read_dir(&executions_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    resumable.push(stem.to_string());
                }
            }
        }

        Ok(resumable)
    }
}

