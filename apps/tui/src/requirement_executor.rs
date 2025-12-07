//! Async requirement executor for non-blocking TUI execution.
//!
//! Spawns requirement execution tasks asynchronously with live progress updates.

use std::sync::Arc;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::JoinHandle;

use radium_core::workflow::RequirementExecutor as CoreRequirementExecutor;
use radium_core::agents::registry::AgentRegistry;
use radium_core::storage::Database;
use radium_abstraction::Model;
use radium_orchestrator::{AgentExecutor, Orchestrator};

use crate::progress_channel::{create_progress_channel, ExecutionResult, ProgressMessage, TaskStatus};

/// Async requirement executor for TUI.
pub struct RequirementExecutor {
    /// Project ID for Braingrid.
    project_id: String,
    /// Orchestrator reference.
    orchestrator_ref: Arc<Orchestrator>,
    /// Agent executor reference.
    executor_ref: Arc<AgentExecutor>,
    /// Database reference.
    db: Arc<std::sync::Mutex<Database>>,
    /// Agent registry.
    agent_registry: Arc<AgentRegistry>,
    /// AI model for execution.
    model: Arc<dyn Model>,
}

impl RequirementExecutor {
    /// Creates a new async requirement executor.
    pub fn new(
        project_id: impl Into<String>,
        orchestrator_ref: Arc<Orchestrator>,
        executor_ref: Arc<AgentExecutor>,
        db: Arc<std::sync::Mutex<Database>>,
        agent_registry: Arc<AgentRegistry>,
        model: Arc<dyn Model>,
    ) -> Self {
        Self {
            project_id: project_id.into(),
            orchestrator_ref,
            executor_ref,
            db,
            agent_registry,
            model,
        }
    }

    /// Spawns an async task to execute a requirement.
    ///
    /// Returns a tuple of (task_handle, progress_receiver) where:
    /// - task_handle: JoinHandle for the spawned task
    /// - progress_receiver: Channel receiver for live progress updates
    pub fn spawn_requirement_task(
        &self,
        requirement_id: String,
    ) -> (
        JoinHandle<Result<ExecutionResult, String>>,
        UnboundedReceiver<ProgressMessage>,
    ) {
        let (progress_tx, progress_rx) = create_progress_channel();

        let project_id = self.project_id.clone();
        let orchestrator_ref = Arc::clone(&self.orchestrator_ref);
        let executor_ref = Arc::clone(&self.executor_ref);
        let db = Arc::clone(&self.db);
        let agent_registry = Arc::clone(&self.agent_registry);
        let model = Arc::clone(&self.model);

        let req_id = requirement_id.clone();

        let handle = tokio::spawn(async move {
            // Create core executor
            let executor = CoreRequirementExecutor::new(
                &project_id,
                &orchestrator_ref,
                &executor_ref,
                &db,
                agent_registry,
                model,
            ).map_err(|e| format!("Failed to create executor: {}", e))?;

            // Send initial status
            let _ = progress_tx.send(ProgressMessage::StatusChange {
                task_id: req_id.clone(),
                task_title: format!("Requirement {}", req_id),
                status: TaskStatus::Running,
            });

            // Execute requirement
            let start_time = std::time::Instant::now();

            match executor.execute_requirement(&req_id).await {
                Ok(result) => {
                    // Send completion message
                    let _ = progress_tx.send(ProgressMessage::RequirementComplete {
                        requirement_id: req_id.clone(),
                        result: ExecutionResult {
                            requirement_id: result.requirement_id.clone(),
                            tasks_completed: result.tasks_completed,
                            tasks_failed: result.tasks_failed,
                            execution_time_secs: result.execution_time_secs,
                        },
                    });

                    Ok(ExecutionResult {
                        requirement_id: result.requirement_id,
                        tasks_completed: result.tasks_completed,
                        tasks_failed: result.tasks_failed,
                        execution_time_secs: start_time.elapsed().as_secs(),
                    })
                }
                Err(e) => {
                    // Send failure message
                    let _ = progress_tx.send(ProgressMessage::TaskFailed {
                        task_id: req_id.clone(),
                        error: e.to_string(),
                    });

                    Err(format!("Execution failed: {}", e))
                }
            }
        });

        (handle, progress_rx)
    }
}
