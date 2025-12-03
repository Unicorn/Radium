//! Step tracking system for workflow resume capability.
//!
//! Tracks step completion status to enable resuming workflows from checkpoints.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Status of a workflow step execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StepStatus {
    /// Step has not been started.
    Pending,
    /// Step is currently executing.
    InProgress,
    /// Step completed successfully.
    Completed,
    /// Step failed with an error.
    Failed,
    /// Step was skipped (e.g., during loop).
    Skipped,
}

/// Record of a step execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepRecord {
    /// Step identifier (typically index or agent ID).
    pub step_id: String,
    /// Status of the step.
    pub status: StepStatus,
    /// When the step was started.
    pub started_at: Option<DateTime<Utc>>,
    /// When the step completed or failed.
    pub completed_at: Option<DateTime<Utc>>,
    /// Error message if step failed.
    pub error: Option<String>,
    /// Number of times this step has been executed (for loops).
    pub execution_count: usize,
}

impl StepRecord {
    /// Creates a new step record in pending state.
    pub fn new(step_id: impl Into<String>) -> Self {
        Self {
            step_id: step_id.into(),
            status: StepStatus::Pending,
            started_at: None,
            completed_at: None,
            error: None,
            execution_count: 0,
        }
    }

    /// Marks the step as started.
    pub fn mark_started(&mut self) {
        self.status = StepStatus::InProgress;
        self.started_at = Some(Utc::now());
        self.execution_count += 1;
    }

    /// Marks the step as completed.
    pub fn mark_completed(&mut self) {
        self.status = StepStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.error = None;
    }

    /// Marks the step as failed with an error message.
    pub fn mark_failed(&mut self, error: impl Into<String>) {
        self.status = StepStatus::Failed;
        self.completed_at = Some(Utc::now());
        self.error = Some(error.into());
    }

    /// Marks the step as skipped.
    pub fn mark_skipped(&mut self) {
        self.status = StepStatus::Skipped;
        self.completed_at = Some(Utc::now());
    }

    /// Checks if the step is completed.
    pub fn is_completed(&self) -> bool {
        self.status == StepStatus::Completed
    }

    /// Checks if the step is in progress.
    pub fn is_in_progress(&self) -> bool {
        self.status == StepStatus::InProgress
    }
}

/// Tracks step completion for workflow resumability.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepTracker {
    /// Map of step ID to step record.
    records: HashMap<String, StepRecord>,
    /// List of completed step IDs in order.
    completed_steps: Vec<String>,
    /// List of step IDs that haven't been completed.
    not_completed_steps: Vec<String>,
}

impl StepTracker {
    /// Creates a new step tracker.
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
            completed_steps: Vec::new(),
            not_completed_steps: Vec::new(),
        }
    }

    /// Marks a step as started.
    pub fn mark_started(&mut self, step_id: impl Into<String>) {
        let step_id = step_id.into();
        let record =
            self.records.entry(step_id.clone()).or_insert_with(|| StepRecord::new(&step_id));
        record.mark_started();
    }

    /// Marks a step as completed.
    pub fn mark_completed(&mut self, step_id: impl Into<String>) {
        let step_id = step_id.into();
        let record =
            self.records.entry(step_id.clone()).or_insert_with(|| StepRecord::new(&step_id));
        record.mark_completed();

        // Add to completed list if not already there
        if !self.completed_steps.contains(&step_id) {
            self.completed_steps.push(step_id.clone());
        }

        // Remove from not completed list
        self.not_completed_steps.retain(|id| id != &step_id);
    }

    /// Marks a step as failed.
    pub fn mark_failed(&mut self, step_id: impl Into<String>, error: impl Into<String>) {
        let step_id = step_id.into();
        let record =
            self.records.entry(step_id.clone()).or_insert_with(|| StepRecord::new(&step_id));
        record.mark_failed(error);

        // Add to not completed list if not already there
        if !self.not_completed_steps.contains(&step_id) {
            self.not_completed_steps.push(step_id);
        }
    }

    /// Marks a step as skipped.
    pub fn mark_skipped(&mut self, step_id: impl Into<String>) {
        let step_id = step_id.into();
        let record =
            self.records.entry(step_id.clone()).or_insert_with(|| StepRecord::new(&step_id));
        record.mark_skipped();
    }

    /// Removes a step from the not completed list.
    pub fn remove_from_not_completed(&mut self, step_id: &str) {
        self.not_completed_steps.retain(|id| id != step_id);
    }

    /// Gets the record for a step.
    pub fn get_record(&self, step_id: &str) -> Option<&StepRecord> {
        self.records.get(step_id)
    }

    /// Checks if a step has been completed.
    pub fn is_completed(&self, step_id: &str) -> bool {
        self.records.get(step_id).map(|r| r.is_completed()).unwrap_or(false)
    }

    /// Gets all completed step IDs.
    pub fn get_completed_steps(&self) -> &[String] {
        &self.completed_steps
    }

    /// Gets all not completed step IDs.
    pub fn get_not_completed_steps(&self) -> &[String] {
        &self.not_completed_steps
    }

    /// Calculates the resume index (where to start execution).
    ///
    /// Returns the index after the last completed step, or 0 if no steps completed.
    pub fn get_resume_index(&self, total_steps: usize) -> usize {
        if self.completed_steps.is_empty() {
            return 0;
        }

        // Find the highest step index that's been completed
        let completed_indices: HashSet<usize> =
            self.completed_steps.iter().filter_map(|id| id.parse::<usize>().ok()).collect();

        // Find the first gap
        for i in 0..total_steps {
            if !completed_indices.contains(&i) {
                return i;
            }
        }

        total_steps // All completed
    }

    /// Saves step tracker to a JSON file.
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<(), StepTrackingError> {
        let path = path.as_ref();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| StepTrackingError::IoError {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| StepTrackingError::SerializeError { source: e })?;

        std::fs::write(path, content)
            .map_err(|e| StepTrackingError::IoError { path: path.to_path_buf(), source: e })
    }

    /// Loads step tracker from a JSON file.
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, StepTrackingError> {
        let path = path.as_ref();

        let content = std::fs::read_to_string(path)
            .map_err(|e| StepTrackingError::IoError { path: path.to_path_buf(), source: e })?;

        let tracker: StepTracker = serde_json::from_str(&content)
            .map_err(|e| StepTrackingError::ParseError { path: path.to_path_buf(), source: e })?;

        Ok(tracker)
    }
}

impl Default for StepTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during step tracking.
#[derive(Error, Debug)]
pub enum StepTrackingError {
    /// Failed to read or write tracking file.
    #[error("I/O error at {path}: {source}")]
    IoError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Failed to parse tracking file.
    #[error("Failed to parse tracking file at {path}: {source}")]
    ParseError {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    /// Failed to serialize tracker.
    #[error("Failed to serialize tracker: {source}")]
    SerializeError {
        #[source]
        source: serde_json::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_step_record_new() {
        let record = StepRecord::new("step-1");
        assert_eq!(record.step_id, "step-1");
        assert_eq!(record.status, StepStatus::Pending);
        assert!(record.started_at.is_none());
        assert!(record.completed_at.is_none());
        assert_eq!(record.execution_count, 0);
    }

    #[test]
    fn test_step_record_mark_started() {
        let mut record = StepRecord::new("step-1");
        record.mark_started();

        assert_eq!(record.status, StepStatus::InProgress);
        assert!(record.started_at.is_some());
        assert_eq!(record.execution_count, 1);

        record.mark_started();
        assert_eq!(record.execution_count, 2);
    }

    #[test]
    fn test_step_record_mark_completed() {
        let mut record = StepRecord::new("step-1");
        record.mark_started();
        record.mark_completed();

        assert_eq!(record.status, StepStatus::Completed);
        assert!(record.completed_at.is_some());
        assert!(record.is_completed());
    }

    #[test]
    fn test_step_record_mark_failed() {
        let mut record = StepRecord::new("step-1");
        record.mark_started();
        record.mark_failed("Test error");

        assert_eq!(record.status, StepStatus::Failed);
        assert!(record.completed_at.is_some());
        assert_eq!(record.error.as_deref(), Some("Test error"));
        assert!(!record.is_completed());
    }

    #[test]
    fn test_step_tracker_mark_completed() {
        let mut tracker = StepTracker::new();
        tracker.mark_started("step-1");
        tracker.mark_completed("step-1");

        assert!(tracker.is_completed("step-1"));
        assert_eq!(tracker.get_completed_steps(), &["step-1"]);
        assert!(tracker.get_not_completed_steps().is_empty());
    }

    #[test]
    fn test_step_tracker_mark_failed() {
        let mut tracker = StepTracker::new();
        tracker.mark_started("step-1");
        tracker.mark_failed("step-1", "Error");

        assert!(!tracker.is_completed("step-1"));
        assert!(tracker.get_completed_steps().is_empty());
        assert_eq!(tracker.get_not_completed_steps(), &["step-1"]);
    }

    #[test]
    fn test_step_tracker_get_resume_index() {
        let mut tracker = StepTracker::new();
        tracker.mark_completed("0");
        tracker.mark_completed("1");
        tracker.mark_completed("2");

        assert_eq!(tracker.get_resume_index(5), 3);
    }

    #[test]
    fn test_step_tracker_get_resume_index_with_gap() {
        let mut tracker = StepTracker::new();
        tracker.mark_completed("0");
        tracker.mark_completed("2");

        // Should resume at step 1 (first gap)
        assert_eq!(tracker.get_resume_index(5), 1);
    }

    #[test]
    fn test_step_tracker_get_resume_index_empty() {
        let tracker = StepTracker::new();
        assert_eq!(tracker.get_resume_index(5), 0);
    }

    #[test]
    fn test_step_tracker_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let tracking_file = temp_dir.path().join("tracking.json");

        let mut tracker = StepTracker::new();
        tracker.mark_started("step-1");
        tracker.mark_completed("step-1");
        tracker.mark_started("step-2");
        tracker.mark_failed("step-2", "Error");

        // Save
        tracker.save_to_file(&tracking_file).unwrap();

        // Load
        let loaded = StepTracker::load_from_file(&tracking_file).unwrap();

        assert_eq!(loaded.get_completed_steps(), &["step-1"]);
        assert_eq!(loaded.get_not_completed_steps(), &["step-2"]);
        assert!(loaded.is_completed("step-1"));
        assert!(!loaded.is_completed("step-2"));
    }

    #[test]
    fn test_step_tracker_serialization() {
        let mut tracker = StepTracker::new();
        tracker.mark_completed("step-1");
        tracker.mark_completed("step-2");

        let json = serde_json::to_string(&tracker).unwrap();
        let deserialized: StepTracker = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.get_completed_steps(), tracker.get_completed_steps());
    }
}
