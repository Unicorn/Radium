//! Execution history tracking for task executions.
//!
//! This module provides state management for tracking individual task executions
//! with full metadata including timestamps, tokens, costs, and status transitions.

use super::telemetry_state::TokenMetrics;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Execution status for a task
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    /// Task is pending execution
    Pending,
    /// Task is currently running
    Running,
    /// Task completed successfully
    Completed,
    /// Task failed
    Failed,
    /// Task was cancelled
    Cancelled,
}

impl ExecutionStatus {
    /// Returns a display string for the status.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Running => "Running",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
            Self::Cancelled => "Cancelled",
        }
    }

    /// Returns whether the status is terminal (Completed, Failed, or Cancelled).
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}

/// Execution record for a single task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    /// Task ID
    pub task_id: String,
    /// Task name/title
    pub task_name: String,
    /// Requirement ID
    pub requirement_id: String,
    /// Engine/provider (e.g., "gemini-2.0-flash-exp")
    pub engine: String,
    /// Model (e.g., "google/gemini")
    pub model: String,
    /// Current execution status
    pub status: ExecutionStatus,
    /// Start time
    pub start_time: DateTime<Utc>,
    /// End time (None if still running)
    pub end_time: Option<DateTime<Utc>>,
    /// Duration in seconds (None if still running)
    pub duration_secs: Option<u64>,
    /// Token usage metrics
    pub tokens: TokenMetrics,
    /// Cost in USD
    pub cost: f64,
    /// Number of tools used
    pub tool_count: usize,
    /// Retry attempt number (0 = first try)
    pub retry_attempt: usize,
    /// Execution cycle number
    pub cycle_number: usize,
    /// Error message (if failed)
    pub error_message: Option<String>,
}

impl ExecutionRecord {
    /// Creates a new execution record with Pending status.
    pub fn new(
        task_id: String,
        task_name: String,
        requirement_id: String,
        engine: String,
        model: String,
        retry_attempt: usize,
        cycle_number: usize,
    ) -> Self {
        Self {
            task_id,
            task_name,
            requirement_id,
            engine,
            model,
            status: ExecutionStatus::Pending,
            start_time: Utc::now(),
            end_time: None,
            duration_secs: None,
            tokens: TokenMetrics::new(),
            cost: 0.0,
            tool_count: 0,
            retry_attempt,
            cycle_number,
            error_message: None,
        }
    }

    /// Calculates duration from start and end times.
    pub fn calculate_duration(&mut self) {
        if let Some(end) = self.end_time {
            let duration = end.signed_duration_since(self.start_time);
            self.duration_secs = Some(duration.num_seconds() as u64);
        }
    }

    /// Marks the execution as running.
    pub fn mark_running(&mut self) {
        self.status = ExecutionStatus::Running;
        // Update start_time to current time for more accurate timing
        self.start_time = Utc::now();
    }

    /// Marks the execution as completed.
    pub fn mark_completed(&mut self) {
        self.status = ExecutionStatus::Completed;
        self.end_time = Some(Utc::now());
        self.calculate_duration();
    }

    /// Marks the execution as failed with an error message.
    pub fn mark_failed(&mut self, error: String) {
        self.status = ExecutionStatus::Failed;
        self.end_time = Some(Utc::now());
        self.error_message = Some(error);
        self.calculate_duration();
    }

    /// Marks the execution as cancelled.
    pub fn mark_cancelled(&mut self) {
        self.status = ExecutionStatus::Cancelled;
        self.end_time = Some(Utc::now());
        self.calculate_duration();
    }

    /// Updates token usage.
    pub fn update_tokens(&mut self, input: u64, output: u64, cached: u64) {
        self.tokens.add(input, output, cached);
    }

    /// Updates cost.
    pub fn update_cost(&mut self, cost: f64) {
        self.cost = cost;
    }

    /// Increments tool count.
    pub fn increment_tool_count(&mut self) {
        self.tool_count += 1;
    }
}

/// Aggregate statistics for a requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateStats {
    /// Total number of tasks executed
    pub total_tasks: usize,
    /// Number of completed tasks
    pub completed_tasks: usize,
    /// Number of failed tasks
    pub failed_tasks: usize,
    /// Total token usage
    pub total_tokens: TokenMetrics,
    /// Total cost in USD
    pub total_cost: f64,
    /// Total duration in seconds
    pub total_duration_secs: u64,
    /// Total number of tools used
    pub total_tools_used: usize,
}

impl AggregateStats {
    /// Creates empty aggregate stats.
    pub fn new() -> Self {
        Self {
            total_tasks: 0,
            completed_tasks: 0,
            failed_tasks: 0,
            total_tokens: TokenMetrics::new(),
            total_cost: 0.0,
            total_duration_secs: 0,
            total_tools_used: 0,
        }
    }
}

impl Default for AggregateStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Execution history manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionHistory {
    /// All execution records
    records: Vec<ExecutionRecord>,
    /// Current requirement ID being tracked
    current_requirement_id: Option<String>,
    /// Active execution records keyed by task_id (for updating during execution)
    /// Note: Active records are not persisted (they're in-memory only)
    #[serde(skip)]
    active_records: std::collections::HashMap<String, ExecutionRecord>,
}

impl ExecutionHistory {
    /// Creates a new empty execution history.
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            current_requirement_id: None,
            active_records: std::collections::HashMap::new(),
        }
    }

    /// Adds a new execution record.
    pub fn add_record(&mut self, record: ExecutionRecord) {
        self.records.push(record);
        
        // Limit to last 1000 records to prevent unbounded growth
        const MAX_RECORDS: usize = 1000;
        if self.records.len() > MAX_RECORDS {
            self.records.remove(0);
        }
    }

    /// Gets or creates an active execution record for a task.
    pub fn get_or_create_active_record(
        &mut self,
        task_id: String,
        task_name: String,
        requirement_id: String,
        engine: String,
        model: String,
        retry_attempt: usize,
        cycle_number: usize,
    ) -> &mut ExecutionRecord {
        self.active_records.entry(task_id.clone()).or_insert_with(|| {
            ExecutionRecord::new(task_id, task_name, requirement_id, engine, model, retry_attempt, cycle_number)
        })
    }

    /// Gets a mutable reference to an active record.
    pub fn get_active_record_mut(&mut self, task_id: &str) -> Option<&mut ExecutionRecord> {
        self.active_records.get_mut(task_id)
    }

    /// Finalizes an active record and moves it to history.
    pub fn finalize_active_record(&mut self, task_id: &str) {
        if let Some(record) = self.active_records.remove(task_id) {
            self.add_record(record);
        }
    }

    /// Gets all records for a specific requirement.
    pub fn get_records_for_requirement(&self, req_id: &str) -> Vec<&ExecutionRecord> {
        self.records
            .iter()
            .filter(|r| r.requirement_id == req_id)
            .collect()
    }

    /// Gets aggregate statistics for a requirement.
    pub fn get_aggregate_stats(&self, req_id: &str) -> AggregateStats {
        let records = self.get_records_for_requirement(req_id);
        
        let mut stats = AggregateStats::new();
        stats.total_tasks = records.len();
        
        for record in records {
            match record.status {
                ExecutionStatus::Completed => stats.completed_tasks += 1,
                ExecutionStatus::Failed => stats.failed_tasks += 1,
                _ => {}
            }
            
            stats.total_tokens.add(
                record.tokens.input_tokens,
                record.tokens.output_tokens,
                record.tokens.cached_tokens,
            );
            stats.total_cost += record.cost;
            
            if let Some(duration) = record.duration_secs {
                stats.total_duration_secs += duration;
            }
            
            stats.total_tools_used += record.tool_count;
        }
        
        stats
    }

    /// Gets the latest execution record.
    pub fn get_latest_record(&self) -> Option<&ExecutionRecord> {
        self.records.last()
    }

    /// Gets all execution records.
    pub fn get_all_records(&self) -> &[ExecutionRecord] {
        &self.records
    }

    /// Clears all execution history.
    pub fn clear(&mut self) {
        self.records.clear();
        self.current_requirement_id = None;
        self.active_records.clear();
    }

    /// Sets the current requirement ID.
    pub fn set_current_requirement(&mut self, req_id: Option<String>) {
        self.current_requirement_id = req_id;
    }

    /// Gets the current requirement ID.
    pub fn get_current_requirement(&self) -> Option<&String> {
        self.current_requirement_id.as_ref()
    }

    /// Gets the default history file path.
    pub fn default_history_path(workspace_root: &Path) -> PathBuf {
        workspace_root.join(".radium").join("_internals").join("execution-history.json")
    }

    /// Loads execution history from a file.
    pub fn load_from_file(path: &Path) -> Self {
        if !path.exists() {
            return Self::new();
        }

        match std::fs::read_to_string(path) {
            Ok(contents) => {
                match serde_json::from_str::<Self>(&contents) {
                    Ok(mut history) => {
                        // Reinitialize active_records (not persisted)
                        history.active_records = HashMap::new();
                        history
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to parse execution history file: {}", e);
                        Self::new()
                    }
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to read execution history file: {}", e);
                Self::new()
            }
        }
    }

    /// Saves execution history to a file.
    pub fn save_to_file(&self, path: &Path) -> std::io::Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Serialize to JSON
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        // Write to temporary file first (atomic write)
        let temp_path = path.with_extension("tmp");
        std::fs::write(&temp_path, json)?;

        // Atomically rename temp file to actual path
        std::fs::rename(&temp_path, path)?;

        Ok(())
    }

    /// Appends a record to the history file (loads, adds, saves).
    pub fn append_to_file(&mut self, path: &Path, record: &ExecutionRecord) -> std::io::Result<()> {
        // Load existing history
        *self = Self::load_from_file(path);
        
        // Add new record
        self.add_record(record.clone());
        
        // Save updated history
        self.save_to_file(path)
    }
}

impl Default for ExecutionHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_status() {
        assert_eq!(ExecutionStatus::Pending.as_str(), "Pending");
        assert!(!ExecutionStatus::Pending.is_terminal());
        assert!(ExecutionStatus::Completed.is_terminal());
        assert!(ExecutionStatus::Failed.is_terminal());
        assert!(ExecutionStatus::Cancelled.is_terminal());
    }

    #[test]
    fn test_execution_record_lifecycle() {
        let mut record = ExecutionRecord::new(
            "task-1".to_string(),
            "Test Task".to_string(),
            "req-1".to_string(),
            "gemini-2.0-flash-exp".to_string(),
            "google/gemini".to_string(),
            0,
            1,
        );

        assert_eq!(record.status, ExecutionStatus::Pending);
        assert!(record.end_time.is_none());
        assert!(record.duration_secs.is_none());

        record.mark_running();
        assert_eq!(record.status, ExecutionStatus::Running);

        record.update_tokens(100, 50, 10);
        assert_eq!(record.tokens.input_tokens, 100);
        assert_eq!(record.tokens.output_tokens, 50);
        assert_eq!(record.tokens.cached_tokens, 10);

        record.update_cost(0.01);
        assert!((record.cost - 0.01).abs() < 0.0001);

        record.increment_tool_count();
        assert_eq!(record.tool_count, 1);

        record.mark_completed();
        assert_eq!(record.status, ExecutionStatus::Completed);
        assert!(record.end_time.is_some());
        assert!(record.duration_secs.is_some());
    }

    #[test]
    fn test_execution_record_failure() {
        let mut record = ExecutionRecord::new(
            "task-1".to_string(),
            "Test Task".to_string(),
            "req-1".to_string(),
            "gemini-2.0-flash-exp".to_string(),
            "google/gemini".to_string(),
            0,
            1,
        );

        record.mark_running();
        record.mark_failed("Test error".to_string());

        assert_eq!(record.status, ExecutionStatus::Failed);
        assert_eq!(record.error_message, Some("Test error".to_string()));
        assert!(record.end_time.is_some());
    }

    #[test]
    fn test_execution_history() {
        let mut history = ExecutionHistory::new();

        let record1 = ExecutionRecord::new(
            "task-1".to_string(),
            "Task 1".to_string(),
            "req-1".to_string(),
            "gemini-2.0-flash-exp".to_string(),
            "google/gemini".to_string(),
            0,
            1,
        );

        let record2 = ExecutionRecord::new(
            "task-2".to_string(),
            "Task 2".to_string(),
            "req-1".to_string(),
            "gemini-2.0-flash-exp".to_string(),
            "google/gemini".to_string(),
            0,
            1,
        );

        history.add_record(record1);
        history.add_record(record2);

        assert_eq!(history.get_all_records().len(), 2);
        
        let req_records = history.get_records_for_requirement("req-1");
        assert_eq!(req_records.len(), 2);

        let stats = history.get_aggregate_stats("req-1");
        assert_eq!(stats.total_tasks, 2);
    }

    #[test]
    fn test_aggregate_stats() {
        let mut history = ExecutionHistory::new();

        let mut record1 = ExecutionRecord::new(
            "task-1".to_string(),
            "Task 1".to_string(),
            "req-1".to_string(),
            "gemini-2.0-flash-exp".to_string(),
            "google/gemini".to_string(),
            0,
            1,
        );
        record1.update_tokens(100, 50, 0);
        record1.update_cost(0.01);
        record1.mark_completed();

        let mut record2 = ExecutionRecord::new(
            "task-2".to_string(),
            "Task 2".to_string(),
            "req-1".to_string(),
            "gemini-2.0-flash-exp".to_string(),
            "google/gemini".to_string(),
            0,
            1,
        );
        record2.update_tokens(200, 100, 0);
        record2.update_cost(0.02);
        record2.mark_failed("Error".to_string());

        history.add_record(record1);
        history.add_record(record2);

        let stats = history.get_aggregate_stats("req-1");
        assert_eq!(stats.total_tasks, 2);
        assert_eq!(stats.completed_tasks, 1);
        assert_eq!(stats.failed_tasks, 1);
        assert_eq!(stats.total_tokens.input_tokens, 300);
        assert_eq!(stats.total_tokens.output_tokens, 150);
        assert!((stats.total_cost - 0.03).abs() < 0.0001);
    }

    #[test]
    fn test_execution_history_size_limit() {
        let mut history = ExecutionHistory::new();

        // Add more than MAX_RECORDS (1000)
        for i in 0..1005 {
            let record = ExecutionRecord::new(
                format!("task-{}", i),
                format!("Task {}", i),
                "req-1".to_string(),
                "gemini-2.0-flash-exp".to_string(),
                "google/gemini".to_string(),
                0,
                1,
            );
            history.add_record(record);
        }

        // Should be limited to 1000 records
        assert_eq!(history.get_all_records().len(), 1000);
        
        // First record should be removed
        assert!(history.get_all_records()[0].task_id != "task-0");
    }
}

