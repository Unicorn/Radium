//! Task data structures for Radium Core.
//!
//! This module defines the core data structures for tasks, including
//! the Task struct itself, task results, task queues, runtime state, and
//! conversion utilities for working with gRPC protocol definitions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::models::proto_convert;
use crate::proto;

/// Runtime state of a task.
///
/// Tracks the current execution state of a task, allowing the system
/// to monitor and manage task lifecycle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum TaskState {
    /// Task is queued and waiting to be executed.
    #[default]
    Queued,
    /// Task is currently being executed.
    Running,
    /// Task execution has been paused.
    Paused,
    /// Task encountered an error during execution.
    Error(String),
    /// Task execution completed successfully.
    Completed,
    /// Task execution was cancelled.
    Cancelled,
}

/// Result of a task execution.
///
/// Stores the output and metadata from a completed task execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskResult {
    /// Whether this is a partial result (task was cancelled).
    #[serde(default)]
    pub is_partial: bool,
    /// The output produced by the task (JSON value).
    pub output: Value,
    /// Optional error message if the task failed.
    pub error: Option<String>,
    /// Timestamp when the task started execution.
    pub started_at: DateTime<Utc>,
    /// Timestamp when the task completed execution.
    pub completed_at: Option<DateTime<Utc>>,
    /// Duration of execution in milliseconds.
    pub duration_ms: Option<u64>,
    /// Timestamp when the task was cancelled (if applicable).
    pub cancelled_at: Option<DateTime<Utc>>,
    /// Reason for cancellation (if applicable).
    pub cancellation_reason: Option<String>,
}

impl TaskResult {
    /// Creates a new task result with the given output.
    ///
    /// # Arguments
    /// * `output` - The output value from the task
    /// * `started_at` - When the task started
    ///
    /// # Returns
    /// A new `TaskResult` with no error and no completion time.
    pub fn new(output: Value, started_at: DateTime<Utc>) -> Self {
        Self {
            is_partial: false,
            output,
            error: None,
            started_at,
            completed_at: None,
            duration_ms: None,
            cancelled_at: None,
            cancellation_reason: None,
        }
    }

    /// Creates a task result representing a successful completion.
    ///
    /// # Arguments
    /// * `output` - The output value from the task
    /// * `started_at` - When the task started
    /// * `completed_at` - When the task completed
    ///
    /// # Returns
    /// A new `TaskResult` with calculated duration.
    pub fn success(output: Value, started_at: DateTime<Utc>, completed_at: DateTime<Utc>) -> Self {
        let duration_ms =
            completed_at.signed_duration_since(started_at).num_milliseconds().max(0) as u64;

        Self {
            is_partial: false,
            output,
            error: None,
            started_at,
            completed_at: Some(completed_at),
            duration_ms: Some(duration_ms),
            cancelled_at: None,
            cancellation_reason: None,
        }
    }

    /// Creates a task result representing a failed execution.
    ///
    /// # Arguments
    /// * `error` - The error message
    /// * `started_at` - When the task started
    /// * `completed_at` - When the task failed
    ///
    /// # Returns
    /// A new `TaskResult` with the error and calculated duration.
    pub fn failure(error: String, started_at: DateTime<Utc>, completed_at: DateTime<Utc>) -> Self {
        let duration_ms =
            completed_at.signed_duration_since(started_at).num_milliseconds().max(0) as u64;

        Self {
            is_partial: false,
            output: Value::Null,
            error: Some(error),
            started_at,
            completed_at: Some(completed_at),
            duration_ms: Some(duration_ms),
            cancelled_at: None,
            cancellation_reason: None,
        }
    }

    /// Creates a partial task result for a cancelled task.
    ///
    /// # Arguments
    /// * `output` - The partial output value from the task
    /// * `started_at` - When the task started
    /// * `cancelled_at` - When the task was cancelled
    /// * `reason` - Reason for cancellation
    ///
    /// # Returns
    /// A new `TaskResult` marked as partial with cancellation metadata.
    pub fn partial(
        output: Value,
        started_at: DateTime<Utc>,
        cancelled_at: DateTime<Utc>,
        reason: Option<String>,
    ) -> Self {
        let duration_ms =
            cancelled_at.signed_duration_since(started_at).num_milliseconds().max(0) as u64;

        Self {
            is_partial: true,
            output,
            error: None,
            started_at,
            completed_at: Some(cancelled_at),
            duration_ms: Some(duration_ms),
            cancelled_at: Some(cancelled_at),
            cancellation_reason: reason,
        }
    }

    /// Returns whether the task result represents a success.
    pub fn is_success(&self) -> bool {
        self.error.is_none()
    }

    /// Returns whether the task result represents a failure.
    pub fn is_failure(&self) -> bool {
        self.error.is_some()
    }

    /// Returns whether the task result is partial (cancelled).
    pub fn is_partial_result(&self) -> bool {
        self.is_partial
    }
}

/// Core task data structure.
///
/// Represents a task in the Radium system, which is a unit of work
/// to be executed by an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique identifier for the task.
    pub id: String,
    /// Human-readable name for the task.
    pub name: String,
    /// Description of what the task does.
    pub description: String,
    /// ID of the agent that will execute this task.
    pub agent_id: String,
    /// Input data for the task (JSON value).
    pub input: Value,
    /// Current runtime state of the task.
    pub state: TaskState,
    /// Result of the task execution, if available.
    pub result: Option<TaskResult>,
    /// Timestamp when the task was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the task was last updated.
    pub updated_at: DateTime<Utc>,
}

impl Task {
    /// Creates a new task with the specified properties.
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the task
    /// * `name` - Human-readable name
    /// * `description` - Description of the task
    /// * `agent_id` - ID of the agent to execute the task
    /// * `input` - Input data for the task
    ///
    /// # Returns
    /// A new `Task` with `Queued` state and current timestamps.
    pub fn new(
        id: String,
        name: String,
        description: String,
        agent_id: String,
        input: Value,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            description,
            agent_id,
            input,
            state: TaskState::default(),
            result: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Validates the task data.
    ///
    /// # Returns
    /// `Ok(())` if the task is valid, or a `TaskError` if invalid.
    ///
    /// # Errors
    /// * `TaskError::InvalidTask` - If the task data is invalid
    pub fn validate(&self) -> Result<(), TaskError> {
        if self.id.is_empty() {
            return Err(TaskError::InvalidTask("id cannot be empty".to_string()));
        }

        if self.name.is_empty() {
            return Err(TaskError::InvalidTask("name cannot be empty".to_string()));
        }

        if self.agent_id.is_empty() {
            return Err(TaskError::InvalidTask("agent_id cannot be empty".to_string()));
        }

        Ok(())
    }

    /// Updates the task's state and sets the updated_at timestamp.
    ///
    /// # Arguments
    /// * `state` - The new state for the task
    pub fn set_state(&mut self, state: TaskState) {
        self.state = state.clone();
        self.updated_at = Utc::now();

        // If transitioning to completed or error, ensure we have a result
        if matches!(state, TaskState::Completed | TaskState::Error(_)) && self.result.is_none() {
            // Create a placeholder result if none exists
            let now = Utc::now();
            if let TaskState::Error(err) = state {
                self.result = Some(TaskResult::failure(err, self.created_at, now));
            } else {
                self.result = Some(TaskResult::success(Value::Null, self.created_at, now));
            }
        }
    }

    /// Sets the task result and updates the state to completed.
    ///
    /// # Arguments
    /// * `result` - The result of the task execution
    pub fn set_result(&mut self, result: TaskResult) {
        self.result = Some(result);
        self.state = TaskState::Completed;
        self.updated_at = Utc::now();
    }

    /// Returns whether the task has partial results (was cancelled).
    ///
    /// # Returns
    /// `true` if the task has a partial result, `false` otherwise.
    pub fn has_partial_result(&self) -> bool {
        self.result.as_ref().map_or(false, |r| r.is_partial_result())
    }
}

/// Builder for creating `Task` instances.
///
/// Provides a fluent interface for constructing tasks with optional fields.
#[derive(Debug, Default)]
pub struct TaskBuilder {
    id: Option<String>,
    name: Option<String>,
    description: Option<String>,
    agent_id: Option<String>,
    input: Option<Value>,
    state: Option<TaskState>,
    result: Option<TaskResult>,
}

impl TaskBuilder {
    /// Creates a new `TaskBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the task ID.
    #[must_use]
    pub fn id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the task name.
    #[must_use]
    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets the task description.
    #[must_use]
    pub fn description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Sets the agent ID.
    #[must_use]
    pub fn agent_id(mut self, agent_id: String) -> Self {
        self.agent_id = Some(agent_id);
        self
    }

    /// Sets the task input.
    #[must_use]
    pub fn input(mut self, input: Value) -> Self {
        self.input = Some(input);
        self
    }

    /// Sets the initial task state.
    #[must_use]
    pub fn state(mut self, state: TaskState) -> Self {
        self.state = Some(state);
        self
    }

    /// Sets the task result.
    #[must_use]
    pub fn result(mut self, result: TaskResult) -> Self {
        self.result = Some(result);
        self
    }

    /// Builds the `Task` from the builder.
    ///
    /// # Returns
    /// `Ok(Task)` if all required fields are set, or a `TaskError` if validation fails.
    ///
    /// # Errors
    /// * `TaskError::InvalidTask` - If required fields are missing or invalid
    pub fn build(self) -> Result<Task, TaskError> {
        let id = self.id.ok_or_else(|| TaskError::InvalidTask("id is required".to_string()))?;
        let name =
            self.name.ok_or_else(|| TaskError::InvalidTask("name is required".to_string()))?;
        let description = self
            .description
            .ok_or_else(|| TaskError::InvalidTask("description is required".to_string()))?;
        let agent_id = self
            .agent_id
            .ok_or_else(|| TaskError::InvalidTask("agent_id is required".to_string()))?;
        let input =
            self.input.ok_or_else(|| TaskError::InvalidTask("input is required".to_string()))?;

        let now = Utc::now();
        let task = Task {
            id,
            name,
            description,
            agent_id,
            input,
            state: self.state.unwrap_or_default(),
            result: self.result,
            created_at: now,
            updated_at: now,
        };

        task.validate()?;
        Ok(task)
    }
}

/// A queue for managing tasks.
///
/// Provides a way to organize and manage multiple tasks, typically
/// for batch processing or workflow execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskQueue {
    /// Unique identifier for the queue.
    pub id: String,
    /// Human-readable name for the queue.
    pub name: String,
    /// Description of the queue's purpose.
    pub description: String,
    /// Tasks in the queue.
    pub tasks: Vec<String>, // Task IDs
    /// Maximum number of concurrent tasks from this queue.
    pub max_concurrent: Option<u32>,
    /// Timestamp when the queue was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the queue was last updated.
    pub updated_at: DateTime<Utc>,
}

impl TaskQueue {
    /// Creates a new task queue with the specified properties.
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the queue
    /// * `name` - Human-readable name
    /// * `description` - Description of the queue
    ///
    /// # Returns
    /// A new `TaskQueue` with no tasks and current timestamps.
    pub fn new(id: String, name: String, description: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            description,
            tasks: Vec::new(),
            max_concurrent: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Adds a task ID to the queue.
    ///
    /// # Arguments
    /// * `task_id` - The ID of the task to add
    pub fn add_task(&mut self, task_id: String) {
        if !self.tasks.contains(&task_id) {
            self.tasks.push(task_id);
            self.updated_at = Utc::now();
        }
    }

    /// Removes a task ID from the queue.
    ///
    /// # Arguments
    /// * `task_id` - The ID of the task to remove
    ///
    /// # Returns
    /// `true` if the task was found and removed, `false` otherwise.
    pub fn remove_task(&mut self, task_id: &str) -> bool {
        if let Some(pos) = self.tasks.iter().position(|id| id == task_id) {
            self.tasks.remove(pos);
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// Validates the task queue.
    ///
    /// # Returns
    /// `Ok(())` if the queue is valid, or a `TaskError` if invalid.
    ///
    /// # Errors
    /// * `TaskError::InvalidQueue` - If the queue data is invalid
    pub fn validate(&self) -> Result<(), TaskError> {
        if self.id.is_empty() {
            return Err(TaskError::InvalidQueue("id cannot be empty".to_string()));
        }

        if self.name.is_empty() {
            return Err(TaskError::InvalidQueue("name cannot be empty".to_string()));
        }

        if let Some(max_concurrent) = self.max_concurrent {
            if max_concurrent == 0 {
                return Err(TaskError::InvalidQueue(
                    "max_concurrent must be greater than 0".to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// Errors that can occur when working with tasks.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum TaskError {
    /// Invalid task data.
    #[error("Invalid task: {0}")]
    InvalidTask(String),

    /// Invalid task queue.
    #[error("Invalid queue: {0}")]
    InvalidQueue(String),

    /// Error during proto conversion.
    #[error("Proto conversion error: {0}")]
    ProtoConversion(String),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(String),
}

impl From<serde_json::Error> for TaskError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err.to_string())
    }
}

// Conversion from proto::Task to Task
impl TryFrom<proto::Task> for Task {
    type Error = TaskError;

    fn try_from(proto_task: proto::Task) -> Result<Self, Self::Error> {
        let input = proto_convert::json_from_str(&proto_task.input_json)?;
        let state = proto_convert::json_from_str(&proto_task.state)?;
        let result = proto_convert::optional_json_from_str(&proto_task.result_json)?;
        let created_at =
            proto_convert::parse_rfc3339_timestamp(&proto_task.created_at, "created_at")
                .map_err(|e| TaskError::ProtoConversion(e))?;
        let updated_at =
            proto_convert::parse_rfc3339_timestamp(&proto_task.updated_at, "updated_at")
                .map_err(|e| TaskError::ProtoConversion(e))?;

        Ok(Task {
            id: proto_task.id,
            name: proto_task.name,
            description: proto_task.description,
            agent_id: proto_task.agent_id,
            input,
            state,
            result,
            created_at,
            updated_at,
        })
    }
}

// Conversion from Task to proto::Task
impl From<Task> for proto::Task {
    fn from(task: Task) -> Self {
        let input_json = proto_convert::json_to_string(&task.input, "null");
        let state = proto_convert::json_to_string(&task.state, "");
        let result_json =
            task.result.map(|r| proto_convert::json_to_string(&r, "null")).unwrap_or_default();

        proto::Task {
            id: task.id,
            name: task.name,
            description: task.description,
            agent_id: task.agent_id,
            input_json,
            state,
            result_json,
            created_at: proto_convert::format_rfc3339_timestamp(&task.created_at),
            updated_at: proto_convert::format_rfc3339_timestamp(&task.updated_at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_state_default() {
        let state = TaskState::default();
        assert_eq!(state, TaskState::Queued);
    }

    #[test]
    fn test_task_result_new() {
        let now = Utc::now();
        let result = TaskResult::new(Value::String("output".to_string()), now);
        assert!(result.is_success());
        assert!(!result.is_failure());
        assert!(result.error.is_none());
        assert!(result.completed_at.is_none());
    }

    #[test]
    fn test_task_result_success() {
        let started = Utc::now();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let completed = Utc::now();
        let result = TaskResult::success(Value::String("output".to_string()), started, completed);
        assert!(result.is_success());
        assert!(result.completed_at.is_some());
        assert!(result.duration_ms.is_some());
        assert!(result.duration_ms.unwrap() > 0);
    }

    #[test]
    fn test_task_result_failure() {
        let started = Utc::now();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let completed = Utc::now();
        let result = TaskResult::failure("Error message".to_string(), started, completed);
        assert!(result.is_failure());
        assert_eq!(result.error, Some("Error message".to_string()));
        assert!(result.completed_at.is_some());
        assert!(result.duration_ms.is_some());
    }

    #[test]
    fn test_task_new() {
        let task = Task::new(
            "task-1".to_string(),
            "Test Task".to_string(),
            "A test task".to_string(),
            "agent-1".to_string(),
            Value::String("input".to_string()),
        );

        assert_eq!(task.id, "task-1");
        assert_eq!(task.name, "Test Task");
        assert_eq!(task.agent_id, "agent-1");
        assert_eq!(task.state, TaskState::Queued);
        assert!(task.result.is_none());
    }

    #[test]
    fn test_task_validate_success() {
        let task = Task::new(
            "task-1".to_string(),
            "Test Task".to_string(),
            "A test task".to_string(),
            "agent-1".to_string(),
            Value::Null,
        );
        assert!(task.validate().is_ok());
    }

    #[test]
    fn test_task_validate_empty_id() {
        let task = Task::new(
            "".to_string(),
            "Test Task".to_string(),
            "A test task".to_string(),
            "agent-1".to_string(),
            Value::Null,
        );
        assert!(task.validate().is_err());
    }

    #[test]
    fn test_task_validate_empty_agent_id() {
        let task = Task::new(
            "task-1".to_string(),
            "Test Task".to_string(),
            "A test task".to_string(),
            "".to_string(),
            Value::Null,
        );
        assert!(task.validate().is_err());
    }

    #[test]
    fn test_task_set_state() {
        let mut task = Task::new(
            "task-1".to_string(),
            "Test Task".to_string(),
            "A test task".to_string(),
            "agent-1".to_string(),
            Value::Null,
        );

        let initial_updated_at = task.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(10));
        task.set_state(TaskState::Running);

        assert_eq!(task.state, TaskState::Running);
        assert!(task.updated_at > initial_updated_at);
    }

    #[test]
    fn test_task_set_result() {
        let mut task = Task::new(
            "task-1".to_string(),
            "Test Task".to_string(),
            "A test task".to_string(),
            "agent-1".to_string(),
            Value::Null,
        );

        let now = Utc::now();
        let result = TaskResult::success(Value::String("output".to_string()), now, now);
        task.set_result(result.clone());

        assert_eq!(task.state, TaskState::Completed);
        assert_eq!(task.result, Some(result));
    }

    #[test]
    fn test_task_queue_new() {
        let queue = TaskQueue::new(
            "queue-1".to_string(),
            "Test Queue".to_string(),
            "A test queue".to_string(),
        );

        assert_eq!(queue.id, "queue-1");
        assert_eq!(queue.name, "Test Queue");
        assert!(queue.tasks.is_empty());
        assert!(queue.max_concurrent.is_none());
    }

    #[test]
    fn test_task_queue_add_task() {
        let mut queue = TaskQueue::new(
            "queue-1".to_string(),
            "Test Queue".to_string(),
            "A test queue".to_string(),
        );

        queue.add_task("task-1".to_string());
        assert_eq!(queue.tasks.len(), 1);
        assert_eq!(queue.tasks[0], "task-1");
    }

    #[test]
    fn test_task_queue_add_duplicate_task() {
        let mut queue = TaskQueue::new(
            "queue-1".to_string(),
            "Test Queue".to_string(),
            "A test queue".to_string(),
        );

        queue.add_task("task-1".to_string());
        queue.add_task("task-1".to_string()); // Duplicate
        assert_eq!(queue.tasks.len(), 1); // Should not add duplicate
    }

    #[test]
    fn test_task_queue_remove_task() {
        let mut queue = TaskQueue::new(
            "queue-1".to_string(),
            "Test Queue".to_string(),
            "A test queue".to_string(),
        );

        queue.add_task("task-1".to_string());
        assert!(queue.remove_task("task-1"));
        assert!(queue.tasks.is_empty());
    }

    #[test]
    fn test_task_queue_remove_nonexistent_task() {
        let mut queue = TaskQueue::new(
            "queue-1".to_string(),
            "Test Queue".to_string(),
            "A test queue".to_string(),
        );

        assert!(!queue.remove_task("task-1"));
    }

    #[test]
    fn test_task_queue_validate_success() {
        let queue = TaskQueue::new(
            "queue-1".to_string(),
            "Test Queue".to_string(),
            "A test queue".to_string(),
        );
        assert!(queue.validate().is_ok());
    }

    #[test]
    fn test_task_queue_validate_empty_id() {
        let queue =
            TaskQueue::new("".to_string(), "Test Queue".to_string(), "A test queue".to_string());
        assert!(queue.validate().is_err());
    }

    #[test]
    fn test_task_queue_validate_zero_max_concurrent() {
        let mut queue = TaskQueue::new(
            "queue-1".to_string(),
            "Test Queue".to_string(),
            "A test queue".to_string(),
        );
        queue.max_concurrent = Some(0);
        assert!(queue.validate().is_err());
    }

    #[test]
    fn test_proto_task_to_task() {
        let state = serde_json::to_string(&TaskState::Queued).unwrap();

        let proto_task = proto::Task {
            id: "task-1".to_string(),
            name: "Test Task".to_string(),
            description: "A test task".to_string(),
            agent_id: "agent-1".to_string(),
            input_json: r#"{"key": "value"}"#.to_string(),
            state,
            result_json: "".to_string(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
        };

        let task = Task::try_from(proto_task).unwrap();
        assert_eq!(task.id, "task-1");
        assert_eq!(task.name, "Test Task");
        assert_eq!(task.agent_id, "agent-1");
        assert!(task.input.is_object());
    }

    #[test]
    fn test_proto_task_to_task_missing_id() {
        let state = serde_json::to_string(&TaskState::Queued).unwrap();

        let proto_task = proto::Task {
            id: "".to_string(),
            name: "Test Task".to_string(),
            description: "A test task".to_string(),
            agent_id: "agent-1".to_string(),
            input_json: "{}".to_string(),
            state,
            result_json: "".to_string(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
        };

        // Task with empty ID should still parse from proto, but validation would fail
        let task = Task::try_from(proto_task).unwrap();
        assert!(task.validate().is_err());
    }

    #[test]
    fn test_task_to_proto_task() {
        let task = Task::new(
            "task-1".to_string(),
            "Test Task".to_string(),
            "A test task".to_string(),
            "agent-1".to_string(),
            Value::Object(serde_json::Map::new()),
        );

        let proto_task = proto::Task::from(task);
        assert_eq!(proto_task.id, "task-1");
        assert_eq!(proto_task.name, "Test Task");
        assert_eq!(proto_task.agent_id, "agent-1");
    }

    #[test]
    fn test_task_proto_round_trip() {
        let original_task = Task::new(
            "task-1".to_string(),
            "Test Task".to_string(),
            "A test task".to_string(),
            "agent-1".to_string(),
            Value::String("input".to_string()),
        );

        let proto_task = proto::Task::from(original_task.clone());
        let converted_task = Task::try_from(proto_task).unwrap();

        assert_eq!(original_task.id, converted_task.id);
        assert_eq!(original_task.name, converted_task.name);
        assert_eq!(original_task.agent_id, converted_task.agent_id);
    }

    #[test]
    fn test_task_builder_minimal() {
        let task = TaskBuilder::new()
            .id("test-task".to_string())
            .name("Test Task".to_string())
            .description("A test task".to_string())
            .agent_id("agent-1".to_string())
            .input(Value::Null)
            .build()
            .expect("Should build task successfully");

        assert_eq!(task.id, "test-task");
        assert_eq!(task.name, "Test Task");
        assert_eq!(task.description, "A test task");
        assert_eq!(task.agent_id, "agent-1");
        assert_eq!(task.state, TaskState::Queued);
    }

    #[test]
    fn test_task_builder_with_state() {
        let task = TaskBuilder::new()
            .id("test-task".to_string())
            .name("Test Task".to_string())
            .description("A test task".to_string())
            .agent_id("agent-1".to_string())
            .input(Value::Null)
            .state(TaskState::Running)
            .build()
            .expect("Should build task successfully");

        assert_eq!(task.state, TaskState::Running);
    }

    #[test]
    fn test_task_builder_missing_id() {
        let result = TaskBuilder::new()
            .name("Test Task".to_string())
            .description("A test task".to_string())
            .agent_id("agent-1".to_string())
            .input(Value::Null)
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("id is required"));
    }

    #[test]
    fn test_task_builder_missing_name() {
        let result = TaskBuilder::new()
            .id("test-task".to_string())
            .description("A test task".to_string())
            .agent_id("agent-1".to_string())
            .input(Value::Null)
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("name is required"));
    }

    #[test]
    fn test_task_builder_missing_agent_id() {
        let result = TaskBuilder::new()
            .id("test-task".to_string())
            .name("Test Task".to_string())
            .description("A test task".to_string())
            .input(Value::Null)
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("agent_id is required"));
    }

    #[test]
    fn test_task_builder_validation() {
        let result = TaskBuilder::new()
            .id("".to_string()) // Empty ID should fail validation
            .name("Test Task".to_string())
            .description("A test task".to_string())
            .agent_id("agent-1".to_string())
            .input(Value::Null)
            .build();

        assert!(result.is_err());
    }
}
