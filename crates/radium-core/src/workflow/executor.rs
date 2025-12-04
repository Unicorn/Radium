//! Sequential workflow execution.
//!
//! This module provides functionality for executing workflows sequentially,
//! processing steps in order and handling failures.

use std::sync::Arc;
use tracing::{debug, error, info};

use radium_orchestrator::{AgentExecutor, Orchestrator};

use crate::models::{Workflow, WorkflowState};
use crate::storage::{TaskRepository};

use super::control_flow::{StepCondition, should_execute_step};
use super::engine::{ExecutionContext, StepResult, WorkflowEngine, WorkflowEngineError};
use chrono::Utc;

/// Executor for running workflows sequentially.
///
/// Executes workflow steps in order, waiting for each step to complete
/// before proceeding to the next.
pub struct WorkflowExecutor {
    /// Core workflow engine.
    engine: WorkflowEngine,
}

impl WorkflowExecutor {
    /// Creates a new workflow executor.
    ///
    /// # Arguments
    /// * `orchestrator` - The agent orchestrator
    /// * `executor` - The agent executor
    ///
    /// # Returns
    /// A new `WorkflowExecutor` instance.
    pub fn new(orchestrator: Arc<Orchestrator>, executor: Arc<AgentExecutor>) -> Self {
        Self { engine: WorkflowEngine::new(orchestrator, executor) }
    }

    /// Executes a workflow sequentially.
    ///
    /// Steps are executed in order based on `WorkflowStep.order`. Each step
    /// must complete before the next step begins. If a step fails, execution
    /// stops and the workflow state is set to `Error`.
    ///
    /// # Arguments
    /// * `workflow` - The workflow to execute (mutable reference)
    /// * `db` - Shared database access
    ///
    /// # Returns
    /// `Ok(ExecutionContext)` with execution results if successful, or
    /// `WorkflowEngineError` if execution failed.
    pub async fn execute_workflow(
        &self,
        workflow: &mut Workflow,
        db: Arc<std::sync::Mutex<crate::storage::Database>>,
    ) -> Result<ExecutionContext, WorkflowEngineError> {
        info!(
            workflow_id = %workflow.id,
            step_count = workflow.steps.len(),
            "Starting workflow execution"
        );

        // Validate workflow
        workflow.validate().map_err(|e| {
            error!(
                workflow_id = %workflow.id,
                error = %e,
                "Workflow validation failed"
            );
            WorkflowEngineError::Validation(e.to_string())
        })?;

        // Check if workflow is in a valid state to execute
        if !matches!(workflow.state, WorkflowState::Idle) {
            return Err(WorkflowEngineError::Validation(format!(
                "Workflow is not in Idle state: {:?}",
                workflow.state
            )));
        }

        // Create execution context
        let mut context = ExecutionContext::new(workflow.id.clone());

        // Sort steps by order
        let mut sorted_steps = workflow.steps.clone();
        sorted_steps.sort_by_key(|step| step.order);

        // Update workflow state to Running
        {
            let mut db_guard = db.lock().map_err(|e| {
                WorkflowEngineError::Storage(crate::storage::StorageError::InvalidData(
                    e.to_string(),
                ))
            })?;
            let mut workflow_repo = crate::storage::SqliteWorkflowRepository::new(&mut *db_guard);
            let running_state = WorkflowState::Running;
            self.engine.update_workflow_state(workflow, &running_state, &mut workflow_repo)?;
        }

        // Execute steps sequentially
        for (index, step) in sorted_steps.iter().enumerate() {
            context.current_step_index = index;

            // Check if step should execute based on conditions
            let condition = StepCondition::from_json(step.config_json.as_ref())
                .map_err(|e| WorkflowEngineError::Validation(e.to_string()))?;

            if !should_execute_step(&step.id, condition.as_ref(), &context)
                .map_err(|e| WorkflowEngineError::Validation(e.to_string()))?
            {
                debug!(
                    workflow_id = %workflow.id,
                    step_id = %step.id,
                    "Skipping step due to condition"
                );
                continue;
            }

            debug!(
                workflow_id = %workflow.id,
                step_id = %step.id,
                step_order = step.order,
                step_index = index,
                total_steps = sorted_steps.len(),
                "Executing workflow step"
            );

            // Execute the step
            // We need to load the task from DB, which requires a lock.
            // But execute_step is async (agent call), so we can't hold the lock across it.
            // Solution: Load task inside a block, then execute.
            
            let started_at = Utc::now();
            let step_result: Result<StepResult, WorkflowEngineError> = async {
                 // 1. Load task (Sync DB access)
                let task = {
                    let mut db_guard = db.lock().map_err(|e| {
                         WorkflowEngineError::Storage(crate::storage::StorageError::InvalidData(e.to_string()))
                    })?;
                    let task_repo = crate::storage::SqliteTaskRepository::new(&mut *db_guard);
                    task_repo.get_by_id(&step.task_id).map_err(|e| match e {
                        crate::storage::StorageError::NotFound(_) => WorkflowEngineError::TaskNotFound(step.task_id.clone()),
                        _ => WorkflowEngineError::Storage(e),
                    })?
                };

                // 2. Prepare execution (CPU bound)
                 let agent = self.engine.orchestrator.get_agent(&task.agent_id).await.ok_or_else(|| {
                    WorkflowEngineError::AgentNotFound(task.agent_id.clone())
                })?;

                let input_str = match &task.input {
                    serde_json::Value::String(s) => s.clone(),
                    v => serde_json::to_string(v).map_err(|e| WorkflowEngineError::InvalidInput(e.to_string()))?,
                };

                // 3. Execute Agent (Async, no DB lock)
                let execution_result = self.engine.executor.execute_agent_with_default_model(agent, &input_str).await.map_err(|e| {
                    WorkflowEngineError::Execution(e.to_string())
                })?;
                
                let completed_at = Utc::now();
                 // Convert output
                if execution_result.success {
                    let output_value = match execution_result.output {
                        radium_orchestrator::AgentOutput::Text(text) => serde_json::Value::String(text),
                        radium_orchestrator::AgentOutput::StructuredData(data) => data,
                        radium_orchestrator::AgentOutput::ToolCall { name, args } => {
                            serde_json::json!({
                                "type": "tool_call",
                                "name": name,
                                "args": args
                            })
                        }
                        radium_orchestrator::AgentOutput::Terminate => serde_json::Value::String("terminated".to_string()),
                    };
                    Ok(StepResult::success(step.id.clone(), output_value, started_at, completed_at))
                } else {
                     let error_msg = execution_result.error.unwrap_or_else(|| "Unknown execution error".to_string());
                     Ok(StepResult::failure(step.id.clone(), error_msg, started_at, completed_at))
                }
            }.await;

            let step_result = match step_result {
                Ok(res) => res,
                Err(e) => {
                     let error_msg = e.to_string();
                     let completed_at = Utc::now();
                     StepResult::failure(step.id.clone(), error_msg, started_at, completed_at)
                }
            };

            // Record step result
            context.record_step_result(step.id.clone(), step_result.clone());

            // Check if step failed
            if !step_result.success {
                let error_msg =
                    step_result.error.unwrap_or_else(|| "Step execution failed".to_string());

                error!(
                    workflow_id = %workflow.id,
                    step_id = %step.id,
                    error = %error_msg,
                    "Workflow step failed, stopping execution"
                );

                // Update workflow state to Error
                {
                     let mut db_guard = db.lock().map_err(|e| {
                        WorkflowEngineError::Storage(crate::storage::StorageError::InvalidData(
                            e.to_string(),
                        ))
                    })?;
                    let mut workflow_repo = crate::storage::SqliteWorkflowRepository::new(&mut *db_guard);
                    let error_state = WorkflowState::Error(error_msg.clone());
                    self.engine.update_workflow_state(workflow, &error_state, &mut workflow_repo)?;
                }

                return Err(WorkflowEngineError::Execution(error_msg));
            }

            info!(
                workflow_id = %workflow.id,
                step_id = %step.id,
                "Workflow step completed successfully"
            );
        }

        // All steps completed successfully
        context.completed_at = Some(chrono::Utc::now());
        context.current_step_index = sorted_steps.len();

        // Update workflow state to Completed
        {
             let mut db_guard = db.lock().map_err(|e| {
                WorkflowEngineError::Storage(crate::storage::StorageError::InvalidData(
                    e.to_string(),
                ))
            })?;
            let mut workflow_repo = crate::storage::SqliteWorkflowRepository::new(&mut *db_guard);
            let completed_state = WorkflowState::Completed;
            self.engine.update_workflow_state(workflow, &completed_state, &mut workflow_repo)?;
        }

        info!(
            workflow_id = %workflow.id,
            step_count = sorted_steps.len(),
            duration_ms = context.completed_at
                .map_or(0, |completed| completed
                    .signed_duration_since(context.started_at)
                    .num_milliseconds()),
            "Workflow execution completed successfully"
        );

        Ok(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Task, WorkflowStep};
    use crate::storage::{Database, SqliteTaskRepository, SqliteWorkflowRepository};
    use radium_orchestrator::AgentExecutor;
    use radium_orchestrator::{Orchestrator, SimpleAgent};
    use serde_json::json;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_execute_workflow_sequential() {
        // Setup - use separate databases to avoid borrowing conflicts in tests
        // In production, Database would be wrapped in Arc<Mutex<>> to allow sharing
        let mut task_db = Database::open_in_memory().unwrap();
        let mut workflow_db = Database::open_in_memory().unwrap();
        let orchestrator = Arc::new(Orchestrator::new());
        let executor = Arc::new(AgentExecutor::with_mock_model());
        let workflow_executor =
            WorkflowExecutor::new(Arc::clone(&orchestrator), Arc::clone(&executor));

        // Register an agent
        let agent = Arc::new(SimpleAgent::new("test-agent".to_string(), "Test agent".to_string()));
        orchestrator.register_agent(agent).await;

        // Create tasks in task_db
        {
            let mut task_repo = SqliteTaskRepository::new(&mut task_db);
            let task1 = Task::new(
                "task-1".to_string(),
                "Task 1".to_string(),
                "First task".to_string(),
                "test-agent".to_string(),
                json!({"input": "test1"}),
            );
            let task2 = Task::new(
                "task-2".to_string(),
                "Task 2".to_string(),
                "Second task".to_string(),
                "test-agent".to_string(),
                json!({"input": "test2"}),
            );
            task_repo.create(&task1).unwrap();
            task_repo.create(&task2).unwrap();
        }

        // Create workflow in workflow_db
        {
            let mut workflow_repo = SqliteWorkflowRepository::new(&mut workflow_db);
            let mut workflow = crate::models::Workflow::new(
                "workflow-1".to_string(),
                "Test Workflow".to_string(),
                "A test workflow".to_string(),
            );
            workflow
                .add_step(WorkflowStep::new(
                    "step-1".to_string(),
                    "Step 1".to_string(),
                    "First step".to_string(),
                    "task-1".to_string(),
                    0,
                ))
                .unwrap();
            workflow
                .add_step(WorkflowStep::new(
                    "step-2".to_string(),
                    "Step 2".to_string(),
                    "Second step".to_string(),
                    "task-2".to_string(),
                    1,
                ))
                .unwrap();
            workflow_repo.create(&workflow).unwrap();
        }

        // Execute workflow - now we can create both repos from different databases
        let mut workflow = {
            let workflow_repo = SqliteWorkflowRepository::new(&mut workflow_db);
            workflow_repo.get_by_id("workflow-1").unwrap()
        };

        let context = {
            let task_repo = SqliteTaskRepository::new(&mut task_db);
            let mut workflow_repo = SqliteWorkflowRepository::new(&mut workflow_db);
            workflow_executor
                .execute_workflow(&mut workflow, &task_repo, &mut workflow_repo)
                .await
                .unwrap()
        };

        // Verify results
        assert_eq!(context.workflow_id, "workflow-1");
        assert_eq!(context.step_results.len(), 2);
        assert!(context.step_results.get("step-1").unwrap().success);
        assert!(context.step_results.get("step-2").unwrap().success);

        // Verify workflow state
        let workflow_repo = SqliteWorkflowRepository::new(&mut workflow_db);
        let workflow = workflow_repo.get_by_id("workflow-1").unwrap();
        assert_eq!(workflow.state, WorkflowState::Completed);
    }

    #[tokio::test]
    async fn test_execute_workflow_single_step() {
        let mut task_db = Database::open_in_memory().unwrap();
        let mut workflow_db = Database::open_in_memory().unwrap();
        let orchestrator = Arc::new(Orchestrator::new());
        let executor = Arc::new(AgentExecutor::with_mock_model());
        let workflow_executor =
            WorkflowExecutor::new(Arc::clone(&orchestrator), Arc::clone(&executor));

        // Register an agent
        let agent = Arc::new(SimpleAgent::new("test-agent".to_string(), "Test agent".to_string()));
        orchestrator.register_agent(agent).await;

        // Create task
        {
            let mut task_repo = SqliteTaskRepository::new(&mut task_db);
            let task = Task::new(
                "task-1".to_string(),
                "Task 1".to_string(),
                "Single task".to_string(),
                "test-agent".to_string(),
                json!({"input": "test"}),
            );
            task_repo.create(&task).unwrap();
        }

        // Create workflow with single step
        {
            let mut workflow_repo = SqliteWorkflowRepository::new(&mut workflow_db);
            let mut workflow = crate::models::Workflow::new(
                "workflow-1".to_string(),
                "Single Step Workflow".to_string(),
                "A workflow with one step".to_string(),
            );
            workflow
                .add_step(WorkflowStep::new(
                    "step-1".to_string(),
                    "Step 1".to_string(),
                    "Only step".to_string(),
                    "task-1".to_string(),
                    0,
                ))
                .unwrap();
            workflow_repo.create(&workflow).unwrap();
        }

        // Execute workflow
        let mut workflow = {
            let workflow_repo = SqliteWorkflowRepository::new(&mut workflow_db);
            workflow_repo.get_by_id("workflow-1").unwrap()
        };

        let context = {
            let task_repo = SqliteTaskRepository::new(&mut task_db);
            let mut workflow_repo = SqliteWorkflowRepository::new(&mut workflow_db);
            workflow_executor
                .execute_workflow(&mut workflow, &task_repo, &mut workflow_repo)
                .await
                .unwrap()
        };

        assert_eq!(context.step_results.len(), 1);
        assert!(context.step_results.get("step-1").unwrap().success);
    }

    #[tokio::test]
    async fn test_execute_workflow_empty_workflow() {
        // Use separate databases to avoid borrow checker issues in tests
        let mut task_db = Database::open_in_memory().unwrap();
        let mut workflow_db = Database::open_in_memory().unwrap();
        let orchestrator = Arc::new(Orchestrator::new());
        let executor = Arc::new(AgentExecutor::with_mock_model());
        let workflow_executor =
            WorkflowExecutor::new(Arc::clone(&orchestrator), Arc::clone(&executor));

        // Create workflow with no steps (valid - will complete immediately)
        {
            let mut workflow_repo = SqliteWorkflowRepository::new(&mut workflow_db);
            let workflow = crate::models::Workflow::new(
                "workflow-1".to_string(),
                "Empty Workflow".to_string(),
                "A workflow with no steps".to_string(),
            );
            workflow_repo.create(&workflow).unwrap();
        }

        let mut workflow = {
            let workflow_repo = SqliteWorkflowRepository::new(&mut workflow_db);
            workflow_repo.get_by_id("workflow-1").unwrap()
        };

        let result = {
            let task_repo = SqliteTaskRepository::new(&mut task_db);
            let mut workflow_repo = SqliteWorkflowRepository::new(&mut workflow_db);
            workflow_executor.execute_workflow(&mut workflow, &task_repo, &mut workflow_repo).await
        };

        // Empty workflow should complete successfully with no steps
        assert!(result.is_ok());
        let context = result.unwrap();
        assert_eq!(context.step_results.len(), 0);
        assert_eq!(workflow.state, WorkflowState::Completed);
    }

    #[tokio::test]
    async fn test_execute_workflow_invalid_state() {
        // Use separate databases to avoid borrow checker issues in tests
        let mut task_db = Database::open_in_memory().unwrap();
        let mut workflow_db = Database::open_in_memory().unwrap();
        let orchestrator = Arc::new(Orchestrator::new());
        let executor = Arc::new(AgentExecutor::with_mock_model());
        let workflow_executor =
            WorkflowExecutor::new(Arc::clone(&orchestrator), Arc::clone(&executor));

        // Create workflow and set it to Running state
        {
            let mut workflow_repo = SqliteWorkflowRepository::new(&mut workflow_db);
            let mut workflow = crate::models::Workflow::new(
                "workflow-1".to_string(),
                "Running Workflow".to_string(),
                "A workflow already running".to_string(),
            );
            workflow.set_state(WorkflowState::Running);
            workflow_repo.create(&workflow).unwrap();
        }

        let mut workflow = {
            let workflow_repo = SqliteWorkflowRepository::new(&mut workflow_db);
            workflow_repo.get_by_id("workflow-1").unwrap()
        };

        let result = {
            let task_repo = SqliteTaskRepository::new(&mut task_db);
            let mut workflow_repo = SqliteWorkflowRepository::new(&mut workflow_db);
            workflow_executor.execute_workflow(&mut workflow, &task_repo, &mut workflow_repo).await
        };

        assert!(result.is_err());
        match result.unwrap_err() {
            WorkflowEngineError::Validation(_) => {}
            _ => panic!("Expected Validation error for invalid state"),
        }
    }

    #[tokio::test]
    async fn test_execute_workflow_with_dependencies() {
        // Test workflow with steps that have dependencies
        let mut task_db = Database::open_in_memory().unwrap();
        let mut workflow_db = Database::open_in_memory().unwrap();
        let orchestrator = Arc::new(Orchestrator::new());
        let executor = Arc::new(AgentExecutor::with_mock_model());
        let workflow_executor =
            WorkflowExecutor::new(Arc::clone(&orchestrator), Arc::clone(&executor));

        // Register an agent
        let agent = Arc::new(SimpleAgent::new("test-agent".to_string(), "Test agent".to_string()));
        orchestrator.register_agent(agent).await;

        // Create tasks
        {
            let mut task_repo = SqliteTaskRepository::new(&mut task_db);
            for i in 0..3 {
                let task = Task::new(
                    format!("task-{}", i),
                    format!("Task {}", i),
                    format!("Test task {}", i),
                    "test-agent".to_string(),
                    json!({"input": format!("test-{}", i)}),
                );
                task_repo.create(&task).unwrap();
            }
        }

        // Create workflow with steps that have dependencies
        {
            let mut workflow_repo = SqliteWorkflowRepository::new(&mut workflow_db);
            let mut workflow = crate::models::Workflow::new(
                "workflow-1".to_string(),
                "Dependency Workflow".to_string(),
                "A workflow with dependencies".to_string(),
            );

            // Step 1: No dependencies
            workflow
                .add_step(WorkflowStep::new(
                    "step-1".to_string(),
                    "Step 1".to_string(),
                    "First step".to_string(),
                    "task-0".to_string(),
                    0,
                ))
                .unwrap();

            // Step 2: Depends on step-1
            let mut step2 = WorkflowStep::new(
                "step-2".to_string(),
                "Step 2".to_string(),
                "Second step".to_string(),
                "task-1".to_string(),
                1,
            );
            step2.config_json = Some(
                serde_json::to_string(&serde_json::json!({
                    "dependsOn": ["step-1"]
                }))
                .unwrap(),
            );
            workflow.add_step(step2).unwrap();

            // Step 3: Depends on step-2
            let mut step3 = WorkflowStep::new(
                "step-3".to_string(),
                "Step 3".to_string(),
                "Third step".to_string(),
                "task-2".to_string(),
                2,
            );
            step3.config_json = Some(
                serde_json::to_string(&serde_json::json!({
                    "dependsOn": ["step-2"]
                }))
                .unwrap(),
            );
            workflow.add_step(step3).unwrap();

            workflow_repo.create(&workflow).unwrap();
        }

        // Execute workflow
        let mut workflow = {
            let workflow_repo = SqliteWorkflowRepository::new(&mut workflow_db);
            workflow_repo.get_by_id("workflow-1").unwrap()
        };

        let context = {
            let task_repo = SqliteTaskRepository::new(&mut task_db);
            let mut workflow_repo = SqliteWorkflowRepository::new(&mut workflow_db);
            workflow_executor
                .execute_workflow(&mut workflow, &task_repo, &mut workflow_repo)
                .await
                .unwrap()
        };

        // Verify all steps executed in order
        assert_eq!(context.step_results.len(), 3);
        assert!(context.step_results.get("step-1").unwrap().success);
        assert!(context.step_results.get("step-2").unwrap().success);
        assert!(context.step_results.get("step-3").unwrap().success);
    }

    #[tokio::test]
    async fn test_execute_workflow_agent_execution_failure() {
        // Test workflow where agent execution fails mid-workflow
        let mut task_db = Database::open_in_memory().unwrap();
        let mut workflow_db = Database::open_in_memory().unwrap();
        let orchestrator = Arc::new(Orchestrator::new());
        let executor = Arc::new(AgentExecutor::with_mock_model());
        let workflow_executor =
            WorkflowExecutor::new(Arc::clone(&orchestrator), Arc::clone(&executor));

        // Register an agent
        let agent = Arc::new(SimpleAgent::new("test-agent".to_string(), "Test agent".to_string()));
        orchestrator.register_agent(agent).await;

        // Create tasks - one will fail (we can't easily simulate this with mock model,
        // but we can test the error handling path)
        {
            let mut task_repo = SqliteTaskRepository::new(&mut task_db);
            let task1 = Task::new(
                "task-1".to_string(),
                "Task 1".to_string(),
                "First task".to_string(),
                "test-agent".to_string(),
                json!({"input": "test1"}),
            );
            task_repo.create(&task1).unwrap();

            // Create task with non-existent agent to trigger failure
            let task2 = Task::new(
                "task-2".to_string(),
                "Task 2".to_string(),
                "Second task".to_string(),
                "nonexistent-agent".to_string(),
                json!({"input": "test2"}),
            );
            task_repo.create(&task2).unwrap();
        }

        // Create workflow with two steps
        {
            let mut workflow_repo = SqliteWorkflowRepository::new(&mut workflow_db);
            let mut workflow = crate::models::Workflow::new(
                "workflow-1".to_string(),
                "Failure Workflow".to_string(),
                "A workflow with a failing step".to_string(),
            );
            workflow
                .add_step(WorkflowStep::new(
                    "step-1".to_string(),
                    "Step 1".to_string(),
                    "First step".to_string(),
                    "task-1".to_string(),
                    0,
                ))
                .unwrap();
            workflow
                .add_step(WorkflowStep::new(
                    "step-2".to_string(),
                    "Step 2".to_string(),
                    "Second step".to_string(),
                    "task-2".to_string(),
                    1,
                ))
                .unwrap();
            workflow_repo.create(&workflow).unwrap();
        }

        // Execute workflow - should fail on step 2
        let mut workflow = {
            let workflow_repo = SqliteWorkflowRepository::new(&mut workflow_db);
            workflow_repo.get_by_id("workflow-1").unwrap()
        };

        let result = {
            let task_repo = SqliteTaskRepository::new(&mut task_db);
            let mut workflow_repo = SqliteWorkflowRepository::new(&mut workflow_db);
            workflow_executor.execute_workflow(&mut workflow, &task_repo, &mut workflow_repo).await
        };

        // Should fail because agent not found for step 2
        assert!(result.is_err());

        // Verify workflow is in error state
        let workflow_repo = SqliteWorkflowRepository::new(&mut workflow_db);
        let workflow = workflow_repo.get_by_id("workflow-1").unwrap();
        assert!(matches!(workflow.state, WorkflowState::Error(_)));
    }
}
