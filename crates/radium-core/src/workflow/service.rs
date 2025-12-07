//! Workflow service layer.
//!
//! This module provides a high-level service for workflow operations,
//! bridging between gRPC service and workflow engine.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info};

use radium_orchestrator::{AgentExecutor, Orchestrator};

use crate::models::WorkflowState;
use crate::monitoring::MonitoringService;
use crate::storage::{Database, SqliteWorkflowRepository, StorageError, WorkflowRepository};
use crate::workspace::Workspace;

use super::engine::{ExecutionContext, WorkflowEngine, WorkflowEngineError};
use super::executor::WorkflowExecutor;

/// Execution history entry for a workflow run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    /// Unique identifier for this execution.
    pub execution_id: String,
    /// Workflow ID that was executed.
    pub workflow_id: String,
    /// Execution context with step results.
    pub context: ExecutionContext,
    /// Timestamp when execution started.
    pub started_at: DateTime<Utc>,
    /// Timestamp when execution completed (if finished).
    pub completed_at: Option<DateTime<Utc>>,
    /// Final workflow state.
    pub final_state: WorkflowState,
}

impl WorkflowExecution {
    /// Creates a new workflow execution record.
    ///
    /// # Arguments
    /// * `execution_id` - Unique identifier for this execution
    /// * `workflow_id` - The workflow ID
    /// * `context` - The execution context
    /// * `final_state` - The final workflow state
    ///
    /// # Returns
    /// A new `WorkflowExecution` record.
    pub fn new(
        execution_id: String,
        workflow_id: String,
        context: ExecutionContext,
        final_state: WorkflowState,
    ) -> Self {
        Self {
            execution_id,
            workflow_id,
            completed_at: context.completed_at,
            started_at: context.started_at,
            context,
            final_state,
        }
    }
}

/// High-level service for workflow operations.
pub struct WorkflowService {
    /// Agent orchestrator.
    orchestrator: Arc<Orchestrator>,
    /// Agent executor.
    executor: Arc<AgentExecutor>,
    /// Database for repository access.
    db: Arc<std::sync::Mutex<Database>>,
    /// Execution history (in-memory for now).
    execution_history: Arc<tokio::sync::Mutex<HashMap<String, WorkflowExecution>>>,
    /// Monitoring service for agent lifecycle tracking.
    monitoring: Option<Arc<std::sync::Mutex<MonitoringService>>>,
}

impl WorkflowService {
    /// Creates a new workflow service.
    ///
    /// # Arguments
    /// * `orchestrator` - The agent orchestrator
    /// * `executor` - The agent executor
    /// * `db` - The database (wrapped in Arc<Mutex>)
    ///
    /// # Returns
    /// A new `WorkflowService` instance.
    pub fn new(
        orchestrator: &Arc<Orchestrator>,
        executor: &Arc<AgentExecutor>,
        db: &Arc<std::sync::Mutex<Database>>,
    ) -> Self {
        // Try to initialize monitoring service from workspace
        let monitoring = Workspace::discover().ok().and_then(|ws| {
            let monitoring_path = ws.radium_dir().join("monitoring.db");
            MonitoringService::open(monitoring_path)
                .ok()
                .map(|svc| Arc::new(std::sync::Mutex::new(svc)))
        });

        Self {
            orchestrator: Arc::clone(orchestrator),
            executor: Arc::clone(executor),
            db: Arc::clone(db),
            execution_history: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            monitoring,
        }
    }

    /// Executes a workflow.
    ///
    /// # Arguments
    /// * `workflow_id` - The ID of the workflow to execute
    /// * `use_parallel` - Whether to use parallel execution for steps with same order
    ///
    /// # Returns
    /// `Ok(WorkflowExecution)` if execution succeeded, or `WorkflowEngineError` if it failed.
    #[allow(clippy::unused_async)]
    pub async fn execute_workflow(
        &self,
        workflow_id: &str,
        use_parallel: bool,
    ) -> Result<WorkflowExecution, WorkflowEngineError> {
        info!(
            workflow_id = %workflow_id,
            use_parallel = use_parallel,
            "Starting workflow execution"
        );

        // Load workflow
        let mut workflow = {
            let mut db = self.db.lock().map_err(|e| {
                error!(
                    workflow_id = %workflow_id,
                    error = %e,
                    "Failed to acquire database lock"
                );
                WorkflowEngineError::Storage(StorageError::InvalidData(format!(
                    "Database lock error: {}",
                    e
                )))
            })?;
            let workflow_repo = SqliteWorkflowRepository::new(&mut *db);
            workflow_repo.get_by_id(workflow_id).map_err(|e| {
                error!(
                    workflow_id = %workflow_id,
                    error = %e,
                    "Failed to load workflow"
                );
                match e {
                    StorageError::NotFound(_) => WorkflowEngineError::Validation(format!(
                        "Workflow {} not found",
                        workflow_id
                    )),
                    _ => WorkflowEngineError::Storage(e),
                }
            })?
        };

        // Execute workflow
        // Use the shared orchestrator which contains registered agents
        let executor = WorkflowExecutor::new(
            Arc::clone(&self.orchestrator),
            Arc::clone(&self.executor),
            self.monitoring.clone(),
        );

        // Use the new DB-aware execution method
        let context = executor.execute_workflow(&mut workflow, Arc::clone(&self.db)).await?;

        // Create execution record
        let execution_id = uuid::Uuid::new_v4().to_string();
        let execution = WorkflowExecution::new(
            execution_id.clone(),
            workflow_id.to_string(),
            context,
            workflow.state.clone(),
        );

        // Store execution history
        let mut history = self.execution_history.lock().await;
        history.insert(execution_id.clone(), execution.clone());

        info!(
            workflow_id = %workflow_id,
            execution_id = %execution_id,
            "Workflow execution completed"
        );

        Ok(execution)
    }

    /// Gets workflow execution history.
    ///
    /// # Arguments
    /// * `workflow_id` - Optional workflow ID to filter by
    ///
    /// # Returns
    /// Vector of workflow executions.
    pub async fn get_execution_history(&self, workflow_id: Option<&str>) -> Vec<WorkflowExecution> {
        let history = self.execution_history.lock().await;
        if let Some(wf_id) = workflow_id {
            history.values().filter(|exec| exec.workflow_id == wf_id).cloned().collect()
        } else {
            history.values().cloned().collect()
        }
    }

    /// Gets a specific workflow execution.
    ///
    /// # Arguments
    /// * `execution_id` - The execution ID
    ///
    /// # Returns
    /// `Some(WorkflowExecution)` if found, `None` otherwise.
    pub async fn get_execution(&self, execution_id: &str) -> Option<WorkflowExecution> {
        let history = self.execution_history.lock().await;
        history.get(execution_id).cloned()
    }

    /// Stops a running workflow execution.
    ///
    /// Note: This is a placeholder implementation. Full implementation would
    /// require tracking running executions and cancelling them.
    ///
    /// # Arguments
    /// * `workflow_id` - The workflow ID to stop
    /// * `workflow_repo` - Repository for updating workflow state (must be Send)
    ///
    /// # Returns
    /// `Ok(())` if stopped successfully, or `WorkflowEngineError` if it failed.
    pub fn stop_workflow(
        &self,
        workflow_id: &str,
        workflow_repo: &mut (dyn WorkflowRepository + Send),
    ) -> Result<(), WorkflowEngineError> {
        info!(
            workflow_id = %workflow_id,
            "Stopping workflow execution"
        );

        let mut workflow = workflow_repo.get_by_id(workflow_id).map_err(|e| {
            error!(
                workflow_id = %workflow_id,
                error = %e,
                "Failed to load workflow for stop"
            );
            match e {
                StorageError::NotFound(_) => {
                    WorkflowEngineError::Validation(format!("Workflow {} not found", workflow_id))
                }
                _ => WorkflowEngineError::Storage(e),
            }
        })?;

        // Update workflow state to Idle if it's running
        if matches!(workflow.state, WorkflowState::Running) {
            let engine =
                WorkflowEngine::new(Arc::clone(&self.orchestrator), Arc::clone(&self.executor));
            let idle_state = WorkflowState::Idle;
            engine.update_workflow_state(&mut workflow, &idle_state, workflow_repo)?;
        }

        Ok(())
    }
}
