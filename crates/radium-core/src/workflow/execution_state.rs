//! Execution state tracking for parallel task execution.
//!
//! This module provides state management for tracking task execution progress,
//! including task status, results, and timing information.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Task execution status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskExecutionStatus {
    /// Task is pending execution.
    Pending,
    /// Task is currently running.
    Running,
    /// Task completed successfully.
    Completed,
    /// Task failed during execution.
    Failed,
    /// Task is blocked by failed dependencies.
    Blocked,
}

/// Task execution result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Agent output/logs.
    pub output: String,
    /// Git commit hashes created during execution.
    pub commits: Vec<String>,
    /// Test execution results (if available).
    pub test_results: Option<serde_json::Value>,
    /// Task start time.
    pub started_at: DateTime<Utc>,
    /// Task completion time.
    pub completed_at: DateTime<Utc>,
    /// Agent ID that executed the task.
    pub agent_id: String,
    /// Error message if task failed.
    pub error_message: Option<String>,
}

impl TaskResult {
    /// Creates a new successful task result.
    pub fn success(
        output: String,
        commits: Vec<String>,
        test_results: Option<serde_json::Value>,
        started_at: DateTime<Utc>,
        completed_at: DateTime<Utc>,
        agent_id: String,
    ) -> Self {
        Self {
            output,
            commits,
            test_results,
            started_at,
            completed_at,
            agent_id,
            error_message: None,
        }
    }

    /// Creates a new failed task result.
    pub fn failure(
        output: String,
        started_at: DateTime<Utc>,
        completed_at: DateTime<Utc>,
        agent_id: String,
        error_message: String,
    ) -> Self {
        Self {
            output,
            commits: vec![],
            test_results: None,
            started_at,
            completed_at,
            agent_id,
            error_message: Some(error_message),
        }
    }

    /// Gets the execution duration in seconds.
    pub fn duration_secs(&self) -> u64 {
        (self.completed_at - self.started_at).num_seconds() as u64
    }
}

/// Execution state for tracking task progress.
///
/// Thread-safe state tracking using Arc<RwLock<...>> for concurrent access.
pub struct ExecutionState {
    /// Task status map (task_id -> status).
    status_map: Arc<RwLock<HashMap<String, TaskExecutionStatus>>>,
    /// Task results map (task_id -> result).
    results_map: Arc<RwLock<HashMap<String, TaskResult>>>,
    /// Set of completed task IDs.
    completed_set: Arc<RwLock<std::collections::HashSet<String>>>,
    /// Set of failed task IDs.
    failed_set: Arc<RwLock<std::collections::HashSet<String>>>,
}

impl ExecutionState {
    /// Creates a new execution state.
    ///
    /// # Arguments
    /// * `task_ids` - Vector of all task IDs to track
    pub fn new(task_ids: Vec<String>) -> Self {
        let mut status_map = HashMap::new();
        for task_id in &task_ids {
            status_map.insert(task_id.clone(), TaskExecutionStatus::Pending);
        }

        Self {
            status_map: Arc::new(RwLock::new(status_map)),
            results_map: Arc::new(RwLock::new(HashMap::new())),
            completed_set: Arc::new(RwLock::new(std::collections::HashSet::new())),
            failed_set: Arc::new(RwLock::new(std::collections::HashSet::new())),
        }
    }

    /// Marks a task as running.
    ///
    /// # Arguments
    /// * `task_id` - The task ID to mark as running
    pub fn mark_running(&self, task_id: &str) {
        let mut status_map = self.status_map.write().unwrap();
        status_map.insert(task_id.to_string(), TaskExecutionStatus::Running);
    }

    /// Marks a task as completed with a result.
    ///
    /// # Arguments
    /// * `task_id` - The task ID to mark as completed
    /// * `result` - The task execution result
    pub fn mark_completed(&self, task_id: &str, result: TaskResult) {
        let mut status_map = self.status_map.write().unwrap();
        let mut results_map = self.results_map.write().unwrap();
        let mut completed_set = self.completed_set.write().unwrap();

        status_map.insert(task_id.to_string(), TaskExecutionStatus::Completed);
        results_map.insert(task_id.to_string(), result);
        completed_set.insert(task_id.to_string());
    }

    /// Marks a task as failed with an error message.
    ///
    /// # Arguments
    /// * `task_id` - The task ID to mark as failed
    /// * `result` - The task execution result (with error)
    pub fn mark_failed(&self, task_id: &str, result: TaskResult) {
        let mut status_map = self.status_map.write().unwrap();
        let mut results_map = self.results_map.write().unwrap();
        let mut failed_set = self.failed_set.write().unwrap();

        status_map.insert(task_id.to_string(), TaskExecutionStatus::Failed);
        results_map.insert(task_id.to_string(), result);
        failed_set.insert(task_id.to_string());
    }

    /// Marks a task as blocked (due to failed dependencies).
    ///
    /// # Arguments
    /// * `task_id` - The task ID to mark as blocked
    pub fn mark_blocked(&self, task_id: &str) {
        let mut status_map = self.status_map.write().unwrap();
        status_map.insert(task_id.to_string(), TaskExecutionStatus::Blocked);
    }

    /// Gets the execution status of a task.
    ///
    /// # Arguments
    /// * `task_id` - The task ID to get status for
    ///
    /// # Returns
    /// The task execution status, or Pending if not found
    pub fn get_status(&self, task_id: &str) -> TaskExecutionStatus {
        let status_map = self.status_map.read().unwrap();
        status_map
            .get(task_id)
            .copied()
            .unwrap_or(TaskExecutionStatus::Pending)
    }

    /// Gets the execution result for a task.
    ///
    /// # Arguments
    /// * `task_id` - The task ID to get result for
    ///
    /// # Returns
    /// Some(result) if task has completed or failed, None otherwise
    pub fn get_result(&self, task_id: &str) -> Option<TaskResult> {
        let results_map = self.results_map.read().unwrap();
        results_map.get(task_id).cloned()
    }

    /// Gets all completed task IDs.
    pub fn completed_tasks(&self) -> Vec<String> {
        let completed_set = self.completed_set.read().unwrap();
        completed_set.iter().cloned().collect()
    }

    /// Gets all failed task IDs.
    pub fn failed_tasks(&self) -> Vec<String> {
        let failed_set = self.failed_set.read().unwrap();
        failed_set.iter().cloned().collect()
    }

    /// Checks if a task is completed.
    pub fn is_completed(&self, task_id: &str) -> bool {
        let completed_set = self.completed_set.read().unwrap();
        completed_set.contains(task_id)
    }

    /// Checks if a task has failed.
    pub fn is_failed(&self, task_id: &str) -> bool {
        let failed_set = self.failed_set.read().unwrap();
        failed_set.contains(task_id)
    }

    /// Gets the count of completed tasks.
    pub fn completed_count(&self) -> usize {
        let completed_set = self.completed_set.read().unwrap();
        completed_set.len()
    }

    /// Gets the count of failed tasks.
    pub fn failed_count(&self) -> usize {
        let failed_set = self.failed_set.read().unwrap();
        failed_set.len()
    }

    /// Gets the count of running tasks.
    pub fn running_count(&self) -> usize {
        let status_map = self.status_map.read().unwrap();
        status_map
            .values()
            .filter(|&&status| status == TaskExecutionStatus::Running)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_state_creation() {
        let state = ExecutionState::new(vec!["task1".to_string(), "task2".to_string()]);
        assert_eq!(state.get_status("task1"), TaskExecutionStatus::Pending);
        assert_eq!(state.get_status("task2"), TaskExecutionStatus::Pending);
    }

    #[test]
    fn test_mark_running() {
        let state = ExecutionState::new(vec!["task1".to_string()]);
        state.mark_running("task1");
        assert_eq!(state.get_status("task1"), TaskExecutionStatus::Running);
    }

    #[test]
    fn test_mark_completed() {
        let state = ExecutionState::new(vec!["task1".to_string()]);
        let result = TaskResult::success(
            "Output".to_string(),
            vec!["abc123".to_string()],
            None,
            Utc::now(),
            Utc::now(),
            "code-agent".to_string(),
        );
        state.mark_completed("task1", result);
        assert_eq!(state.get_status("task1"), TaskExecutionStatus::Completed);
        assert!(state.is_completed("task1"));
        assert_eq!(state.completed_count(), 1);
    }

    #[test]
    fn test_mark_failed() {
        let state = ExecutionState::new(vec!["task1".to_string()]);
        let result = TaskResult::failure(
            "Error output".to_string(),
            Utc::now(),
            Utc::now(),
            "code-agent".to_string(),
            "Task failed".to_string(),
        );
        state.mark_failed("task1", result);
        assert_eq!(state.get_status("task1"), TaskExecutionStatus::Failed);
        assert!(state.is_failed("task1"));
        assert_eq!(state.failed_count(), 1);
    }

    #[test]
    fn test_mark_blocked() {
        let state = ExecutionState::new(vec!["task1".to_string()]);
        state.mark_blocked("task1");
        assert_eq!(state.get_status("task1"), TaskExecutionStatus::Blocked);
    }
}

