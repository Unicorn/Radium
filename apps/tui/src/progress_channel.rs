//! Channel-based progress communication for async requirement execution.
//!
//! Provides real-time progress updates from spawned tasks to the TUI without blocking.

use std::time::Duration;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

/// Status of a task during execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    /// Task is waiting to start (○)
    Pending,
    /// Task is currently running (⠋)
    Running,
    /// Task completed successfully (●)
    Completed,
    /// Task failed with an error (✗)
    Failed,
}

impl TaskStatus {
    /// Get the visual symbol for this status.
    pub fn symbol(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "○",
            TaskStatus::Running => "⠋",
            TaskStatus::Completed => "●",
            TaskStatus::Failed => "✗",
        }
    }
}

/// Result of requirement execution.
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Requirement ID that was executed.
    pub requirement_id: String,
    /// Number of tasks completed successfully.
    pub tasks_completed: usize,
    /// Number of tasks that failed.
    pub tasks_failed: usize,
    /// Total execution time in seconds.
    pub execution_time_secs: u64,
}

/// Progress messages sent from async tasks to the TUI.
#[derive(Debug, Clone)]
pub enum ProgressMessage {
    /// Task status changed.
    StatusChange {
        task_id: String,
        task_title: String,
        status: TaskStatus,
    },
    /// Token usage updated.
    TokenUpdate {
        task_id: String,
        tokens_in: u64,
        tokens_out: u64,
    },
    /// Execution duration updated.
    DurationUpdate {
        task_id: String,
        elapsed: Duration,
    },
    /// Task completed successfully.
    TaskComplete {
        task_id: String,
        result: String,
    },
    /// Task failed with error.
    TaskFailed {
        task_id: String,
        error: String,
    },
    /// Entire requirement execution completed.
    RequirementComplete {
        requirement_id: String,
        result: ExecutionResult,
    },
}

/// Creates a new progress channel pair (sender, receiver).
pub fn create_progress_channel() -> (UnboundedSender<ProgressMessage>, UnboundedReceiver<ProgressMessage>) {
    mpsc::unbounded_channel()
}
