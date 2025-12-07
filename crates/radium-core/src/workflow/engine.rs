//! Core workflow execution engine.
//!
//! This module provides the core workflow engine that manages workflow
//! execution state and executes individual workflow steps.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, error, info};

use radium_orchestrator::{AgentExecutor, AgentOutput, Orchestrator};

use crate::models::{Workflow, WorkflowState, WorkflowStep};
use crate::storage::{StorageError, TaskRepository, WorkflowRepository};

/// Execution context for a workflow run.
///
/// Tracks the current execution state, variables, and step results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    /// Workflow ID being executed.
    pub workflow_id: String,
    /// Current step index being executed.
    pub current_step_index: usize,
    /// Variables available to workflow steps.
    pub variables: HashMap<String, Value>,
    /// Results from executed steps, keyed by step ID.
    pub step_results: HashMap<String, StepResult>,
    /// Timestamp when execution started.
    pub started_at: DateTime<Utc>,
    /// Timestamp when execution completed (if finished).
    pub completed_at: Option<DateTime<Utc>>,
    /// ID of the parent agent if this is a worker (for delegation).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_agent_id: Option<String>,
    /// Delegation depth (0 = root, 1 = first level worker, etc.).
    #[serde(default)]
    pub delegation_depth: usize,
    /// IDs of worker agents spawned by this agent.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub worker_ids: Vec<String>,
}

impl ExecutionContext {
    /// Creates a new execution context for a workflow.
    ///
    /// # Arguments
    /// * `workflow_id` - The ID of the workflow to execute
    ///
    /// # Returns
    /// A new `ExecutionContext` with no steps executed.
    pub fn new(workflow_id: String) -> Self {
        Self {
            workflow_id,
            current_step_index: 0,
            variables: HashMap::new(),
            step_results: HashMap::new(),
            started_at: Utc::now(),
            completed_at: None,
            parent_agent_id: None,
            delegation_depth: 0,
            worker_ids: Vec::new(),
        }
    }

    /// Records the result of a step execution.
    ///
    /// # Arguments
    /// * `step_id` - The ID of the step
    /// * `result` - The result of the step execution
    pub fn record_step_result(&mut self, step_id: String, result: StepResult) {
        self.step_results.insert(step_id, result);
    }

    /// Gets the result of a previously executed step.
    ///
    /// # Arguments
    /// * `step_id` - The ID of the step
    ///
    /// # Returns
    /// `Some(StepResult)` if the step was executed, `None` otherwise.
    pub fn get_step_result(&self, step_id: &str) -> Option<&StepResult> {
        self.step_results.get(step_id)
    }

    /// Sets a variable in the execution context.
    ///
    /// # Arguments
    /// * `name` - The variable name
    /// * `value` - The variable value
    pub fn set_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    /// Gets a variable from the execution context.
    ///
    /// # Arguments
    /// * `name` - The variable name
    ///
    /// # Returns
    /// `Some(Value)` if the variable exists, `None` otherwise.
    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }
}

/// Result of executing a workflow step.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StepResult {
    /// The step ID that was executed.
    pub step_id: String,
    /// Whether the step execution was successful.
    pub success: bool,
    /// The output from the step (if successful).
    pub output: Option<Value>,
    /// Error message if the step failed.
    pub error: Option<String>,
    /// Timestamp when the step started.
    pub started_at: DateTime<Utc>,
    /// Timestamp when the step completed.
    pub completed_at: DateTime<Utc>,
    /// Duration of step execution in milliseconds.
    pub duration_ms: u64,
}

impl StepResult {
    /// Creates a successful step result.
    ///
    /// # Arguments
    /// * `step_id` - The step ID
    /// * `output` - The output value
    /// * `started_at` - When execution started
    /// * `completed_at` - When execution completed
    ///
    /// # Returns
    /// A new `StepResult` representing success.
    pub fn success(
        step_id: String,
        output: Value,
        started_at: DateTime<Utc>,
        completed_at: DateTime<Utc>,
    ) -> Self {
        let duration_ms =
            completed_at.signed_duration_since(started_at).num_milliseconds().max(0) as u64;

        Self {
            step_id,
            success: true,
            output: Some(output),
            error: None,
            started_at,
            completed_at,
            duration_ms,
        }
    }

    /// Creates a failed step result.
    ///
    /// # Arguments
    /// * `step_id` - The step ID
    /// * `error` - The error message
    /// * `started_at` - When execution started
    /// * `completed_at` - When execution failed
    ///
    /// # Returns
    /// A new `StepResult` representing failure.
    pub fn failure(
        step_id: String,
        error: String,
        started_at: DateTime<Utc>,
        completed_at: DateTime<Utc>,
    ) -> Self {
        let duration_ms =
            completed_at.signed_duration_since(started_at).num_milliseconds().max(0) as u64;

        Self {
            step_id,
            success: false,
            output: None,
            error: Some(error),
            started_at,
            completed_at,
            duration_ms,
        }
    }
}

/// Core workflow execution engine.
///
/// Manages workflow execution state and executes individual workflow steps
/// by loading tasks and executing them via the agent orchestrator.
#[derive(Clone)]
pub struct WorkflowEngine {
    /// Agent orchestrator for executing tasks.
    pub orchestrator: Arc<Orchestrator>,
    /// Agent executor for running agents.
    pub executor: Arc<AgentExecutor>,
}

impl WorkflowEngine {
    /// Creates a new workflow engine.
    ///
    /// # Arguments
    /// * `orchestrator` - The agent orchestrator
    /// * `executor` - The agent executor
    ///
    /// # Returns
    /// A new `WorkflowEngine` instance.
    pub fn new(orchestrator: Arc<Orchestrator>, executor: Arc<AgentExecutor>) -> Self {
        Self { orchestrator, executor }
    }

    /// Executes a single workflow step.
    ///
    /// This method:
    /// 1. Loads the task by `task_id` from storage
    /// 2. Executes the task via the agent orchestrator
    /// 3. Captures the step result
    ///
    /// # Arguments
    /// * `step` - The workflow step to execute
    /// * `context` - The execution context
    /// * `task_repo` - Repository for loading tasks
    ///
    /// # Returns
    /// `Ok(StepResult)` if execution succeeded, or `WorkflowEngineError` if it failed.
    pub async fn execute_step(
        &self,
        step: &WorkflowStep,
        context: &ExecutionContext,
        task_repo: &dyn TaskRepository,
    ) -> Result<StepResult, WorkflowEngineError> {
        let started_at = Utc::now();
        debug!(
            workflow_id = %context.workflow_id,
            step_id = %step.id,
            task_id = %step.task_id,
            "Executing workflow step"
        );

        // Load the task
        let task = task_repo.get_by_id(&step.task_id).map_err(|e| {
            error!(
                workflow_id = %context.workflow_id,
                step_id = %step.id,
                task_id = %step.task_id,
                error = %e,
                "Failed to load task for workflow step"
            );
            match e {
                StorageError::NotFound(_) => {
                    WorkflowEngineError::TaskNotFound(step.task_id.clone())
                }
                _ => WorkflowEngineError::Storage(e),
            }
        })?;

        // Get the agent from orchestrator
        let agent = self.orchestrator.get_agent(&task.agent_id).await.ok_or_else(|| {
            error!(
                workflow_id = %context.workflow_id,
                step_id = %step.id,
                agent_id = %task.agent_id,
                "Agent not found for task"
            );
            WorkflowEngineError::AgentNotFound(task.agent_id.clone())
        })?;

        // Prepare input - convert task input to string for agent execution
        let input_str = match &task.input {
            Value::String(s) => s.clone(),
            v => serde_json::to_string(v).map_err(|e| {
                error!(
                    workflow_id = %context.workflow_id,
                    step_id = %step.id,
                    error = %e,
                    "Failed to serialize task input"
                );
                WorkflowEngineError::InvalidInput(e.to_string())
            })?,
        };

        // Execute the agent
        let execution_result =
            self.executor.execute_agent_with_default_model(agent, &input_str, None).await.map_err(
                |e| {
                    error!(
                        workflow_id = %context.workflow_id,
                        step_id = %step.id,
                        error = %e,
                        "Agent execution failed"
                    );
                    WorkflowEngineError::Execution(e.to_string())
                },
            )?;

        let completed_at = Utc::now();

        // Convert agent output to step result
        let step_result = if execution_result.success {
            let output_value = match execution_result.output {
                AgentOutput::Text(text) => Value::String(text),
                AgentOutput::StructuredData(data) => data,
                AgentOutput::ToolCall { name, args } => {
                    serde_json::json!({
                        "type": "tool_call",
                        "name": name,
                        "args": args
                    })
                }
                AgentOutput::Terminate => Value::String("terminated".to_string()),
            };

            info!(
                workflow_id = %context.workflow_id,
                step_id = %step.id,
                "Step executed successfully"
            );

            StepResult::success(step.id.clone(), output_value, started_at, completed_at)
        } else {
            let error_msg =
                execution_result.error.unwrap_or_else(|| "Unknown execution error".to_string());

            error!(
                workflow_id = %context.workflow_id,
                step_id = %step.id,
                error = %error_msg,
                "Step execution failed"
            );

            StepResult::failure(step.id.clone(), error_msg, started_at, completed_at)
        };

        Ok(step_result)
    }

    /// Updates the workflow state in storage.
    ///
    /// # Arguments
    /// * `workflow` - The workflow to update
    /// * `state` - The new state
    /// * `workflow_repo` - Repository for updating workflows
    ///
    /// # Returns
    /// `Ok(())` if update succeeded, or `WorkflowEngineError` if it failed.
    pub fn update_workflow_state(
        &self,
        workflow: &mut Workflow,
        state: &WorkflowState,
        workflow_repo: &mut dyn WorkflowRepository,
    ) -> Result<(), WorkflowEngineError> {
        workflow.set_state(state.clone());
        workflow_repo.update(workflow).map_err(|e| {
            error!(
                workflow_id = %workflow.id,
                state = ?state,
                error = %e,
                "Failed to update workflow state"
            );
            WorkflowEngineError::Storage(e)
        })?;

        debug!(
            workflow_id = %workflow.id,
            state = ?state,
            "Updated workflow state"
        );

        Ok(())
    }
}

/// Errors that can occur during workflow execution.
#[derive(Error, Debug)]
pub enum WorkflowEngineError {
    /// Task not found in storage.
    #[error("Task not found: {0}")]
    TaskNotFound(String),

    /// Agent not found in orchestrator.
    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    /// Storage operation failed.
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    /// Invalid input data.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Execution error.
    #[error("Execution error: {0}")]
    Execution(String),

    /// Workflow validation error.
    #[error("Workflow validation error: {0}")]
    Validation(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Task, Workflow, WorkflowState};
    use crate::storage::{Database, SqliteTaskRepository, SqliteWorkflowRepository};
    use radium_orchestrator::{AgentExecutor, Orchestrator, SimpleAgent};
    use serde_json::json;
    use std::sync::Arc;

    fn setup_test_engine() -> (WorkflowEngine, Arc<Orchestrator>, Arc<AgentExecutor>) {
        let orchestrator = Arc::new(Orchestrator::new());
        let executor = Arc::new(AgentExecutor::with_mock_model());
        let engine = WorkflowEngine::new(Arc::clone(&orchestrator), Arc::clone(&executor));
        (engine, orchestrator, executor)
    }

    #[test]
    fn test_execution_context_new() {
        let context = ExecutionContext::new("workflow-1".to_string());
        assert_eq!(context.workflow_id, "workflow-1");
        assert_eq!(context.current_step_index, 0);
        assert!(context.variables.is_empty());
        assert!(context.step_results.is_empty());
        assert!(context.completed_at.is_none());
    }

    #[test]
    fn test_execution_context_record_step_result() {
        let mut context = ExecutionContext::new("workflow-1".to_string());
        let step_result = StepResult::success(
            "step-1".to_string(),
            json!({"output": "test"}),
            Utc::now(),
            Utc::now(),
        );
        context.record_step_result("step-1".to_string(), step_result.clone());
        assert_eq!(context.get_step_result("step-1"), Some(&step_result));
    }

    #[test]
    fn test_execution_context_variables() {
        let mut context = ExecutionContext::new("workflow-1".to_string());
        context.set_variable("test_var".to_string(), json!("test_value"));
        assert_eq!(context.get_variable("test_var"), Some(&json!("test_value")));
        assert_eq!(context.get_variable("nonexistent"), None);
    }

    #[test]
    fn test_step_result_success() {
        let started = Utc::now();
        let completed = started + chrono::Duration::milliseconds(100);
        let result = StepResult::success(
            "step-1".to_string(),
            json!({"output": "test"}),
            started,
            completed,
        );
        assert!(result.success);
        assert_eq!(result.step_id, "step-1");
        assert_eq!(result.output, Some(json!({"output": "test"})));
        assert!(result.error.is_none());
        assert_eq!(result.duration_ms, 100);
    }

    #[test]
    fn test_step_result_failure() {
        let started = Utc::now();
        let completed = started + chrono::Duration::milliseconds(50);
        let result =
            StepResult::failure("step-1".to_string(), "Test error".to_string(), started, completed);
        assert!(!result.success);
        assert_eq!(result.step_id, "step-1");
        assert_eq!(result.output, None);
        assert_eq!(result.error, Some("Test error".to_string()));
        assert_eq!(result.duration_ms, 50);
    }

    #[test]
    fn test_step_result_duration_calculation() {
        let started = Utc::now();
        let completed = started + chrono::Duration::seconds(2);
        let result = StepResult::success("step-1".to_string(), json!("output"), started, completed);
        assert!(result.duration_ms >= 2000);
        assert!(result.duration_ms < 2100); // Allow some margin
    }

    #[tokio::test]
    async fn test_workflow_engine_execute_step_success() {
        let (engine, orchestrator, _executor) = setup_test_engine();
        let mut db = Database::open_in_memory().unwrap();

        // Register agent
        let agent = Arc::new(SimpleAgent::new("test-agent".to_string(), "Test agent".to_string()));
        orchestrator.register_agent(agent).await;

        // Create task
        {
            let mut task_repo = SqliteTaskRepository::new(&mut db);
            let task = Task::new(
                "task-1".to_string(),
                "Test Task".to_string(),
                "A test task".to_string(),
                "test-agent".to_string(),
                json!("test input"),
            );
            task_repo.create(&task).unwrap();
        }

        // Create step and context
        let step = WorkflowStep::new(
            "step-1".to_string(),
            "Step 1".to_string(),
            "First step".to_string(),
            "task-1".to_string(),
            0,
        );
        let context = ExecutionContext::new("workflow-1".to_string());

        // Execute step
        let task_repo = SqliteTaskRepository::new(&mut db);
        let result = engine.execute_step(&step, &context, &task_repo).await;

        assert!(result.is_ok());
        let step_result = result.unwrap();
        assert!(step_result.success);
        assert_eq!(step_result.step_id, "step-1");
    }

    #[tokio::test]
    async fn test_workflow_engine_execute_step_task_not_found() {
        let (engine, _orchestrator, _executor) = setup_test_engine();
        let mut db = Database::open_in_memory().unwrap();

        let step = WorkflowStep::new(
            "step-1".to_string(),
            "Step 1".to_string(),
            "First step".to_string(),
            "nonexistent-task".to_string(),
            0,
        );
        let context = ExecutionContext::new("workflow-1".to_string());

        let task_repo = SqliteTaskRepository::new(&mut db);
        let result = engine.execute_step(&step, &context, &task_repo).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            WorkflowEngineError::TaskNotFound(task_id) => {
                assert_eq!(task_id, "nonexistent-task");
            }
            _ => panic!("Expected TaskNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_workflow_engine_execute_step_agent_not_found() {
        let (engine, _orchestrator, _executor) = setup_test_engine();
        let mut db = Database::open_in_memory().unwrap();

        // Create task with non-existent agent
        {
            let mut task_repo = SqliteTaskRepository::new(&mut db);
            let task = Task::new(
                "task-1".to_string(),
                "Test Task".to_string(),
                "A test task".to_string(),
                "nonexistent-agent".to_string(),
                json!("test input"),
            );
            task_repo.create(&task).unwrap();
        }

        let step = WorkflowStep::new(
            "step-1".to_string(),
            "Step 1".to_string(),
            "First step".to_string(),
            "task-1".to_string(),
            0,
        );
        let context = ExecutionContext::new("workflow-1".to_string());

        let task_repo = SqliteTaskRepository::new(&mut db);
        let result = engine.execute_step(&step, &context, &task_repo).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            WorkflowEngineError::AgentNotFound(agent_id) => {
                assert_eq!(agent_id, "nonexistent-agent");
            }
            _ => panic!("Expected AgentNotFound error"),
        }
    }

    #[test]
    fn test_workflow_engine_update_workflow_state() {
        let (engine, _orchestrator, _executor) = setup_test_engine();
        let mut db = Database::open_in_memory().unwrap();

        let mut workflow = Workflow::new(
            "workflow-1".to_string(),
            "Test Workflow".to_string(),
            "A test workflow".to_string(),
        );
        {
            let mut workflow_repo = SqliteWorkflowRepository::new(&mut db);
            workflow_repo.create(&workflow).unwrap();
        }

        let mut workflow_repo = SqliteWorkflowRepository::new(&mut db);
        let running_state = WorkflowState::Running;
        let result =
            engine.update_workflow_state(&mut workflow, &running_state, &mut workflow_repo);

        assert!(result.is_ok());
        assert_eq!(workflow.state, WorkflowState::Running);
    }
}
