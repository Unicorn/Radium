//! Parallel workflow step execution.
//!
//! This module provides functionality for executing workflow steps in parallel,
//! grouping steps by order value and executing them concurrently.

use std::collections::HashMap;
use tracing::{debug, error, info};

use crate::models::{Workflow, WorkflowState};
use crate::storage::{TaskRepository, WorkflowRepository};

use super::control_flow::{StepCondition, should_execute_step};
use super::engine::{ExecutionContext, StepResult, WorkflowEngine, WorkflowEngineError};

/// Groups workflow steps by their order value for parallel execution.
///
/// Steps with the same `order` value can be executed in parallel.
///
/// # Arguments
/// * `steps` - The workflow steps to group
///
/// # Returns
/// A vector of step groups, where each group contains steps that can run in parallel.
pub fn group_steps_by_order(steps: &[crate::models::WorkflowStep]) -> Vec<Vec<usize>> {
    let mut groups: HashMap<u32, Vec<usize>> = HashMap::new();

    for (index, step) in steps.iter().enumerate() {
        groups.entry(step.order).or_default().push(index);
    }

    // Sort groups by order and return as vector of indices
    let mut sorted_groups: Vec<(u32, Vec<usize>)> = groups.into_iter().collect();
    sorted_groups.sort_by_key(|(order, _)| *order);

    sorted_groups.into_iter().map(|(_, indices)| indices).collect()
}

/// Executes a group of workflow steps in parallel.
///
/// Note: Due to trait object limitations with async, steps in a parallel group
/// are currently executed sequentially. Full parallel execution would require
/// refactoring repositories to use Arc<dyn TaskRepository + Send + Sync>.
///
/// # Arguments
/// * `engine` - The workflow engine
/// * `steps` - All workflow steps
/// * `step_indices` - Indices of steps in this group to execute
/// * `context` - The execution context
/// * `task_repo` - Repository for loading tasks
///
/// # Returns
/// `Ok(Vec<StepResult>)` with results for all steps, or `WorkflowEngineError` if any step failed.
pub async fn execute_parallel_steps(
    engine: &WorkflowEngine,
    steps: &[crate::models::WorkflowStep],
    step_indices: &[usize],
    context: &ExecutionContext,
    task_repo: &dyn TaskRepository,
) -> Result<Vec<StepResult>, WorkflowEngineError> {
    let mut step_results = Vec::new();
    let mut errors = Vec::new();

    // Execute steps in the group
    // TODO: Implement true parallel execution by refactoring repositories to Arc<dyn TaskRepository + Send + Sync>
    for &index in step_indices {
        let step = &steps[index];

        // Check if step should execute based on conditions
        let condition = StepCondition::from_json(step.config_json.as_ref())
            .map_err(|e| WorkflowEngineError::Validation(e.to_string()))?;

        if !should_execute_step(&step.id, condition.as_ref(), context)
            .map_err(|e| WorkflowEngineError::Validation(e.to_string()))?
        {
            debug!(
                step_id = %step.id,
                "Skipping step in parallel group due to condition"
            );
            continue;
        }

        debug!(
            step_id = %step.id,
            "Executing step in parallel group"
        );

        // Execute the step
        match engine.execute_step(step, context, task_repo).await {
            Ok(step_result) => {
                if !step_result.success {
                    let error_msg = step_result
                        .error
                        .clone()
                        .unwrap_or_else(|| "Step execution failed".to_string());
                    errors.push(format!("Step {} failed: {}", step_result.step_id, error_msg));
                }
                step_results.push(step_result);
            }
            Err(e) => {
                error!(
                    step_id = %step.id,
                    error = %e,
                    "Step execution error in parallel group"
                );
                errors.push(format!("Step {} execution error: {}", step.id, e));
            }
        }
    }

    // If any step failed, return error
    if !errors.is_empty() {
        return Err(WorkflowEngineError::Execution(format!(
            "Parallel step execution failed: {}",
            errors.join("; ")
        )));
    }

    Ok(step_results)
}

/// Executes a workflow with parallel step support.
///
/// Steps with the same `order` value are executed in parallel. Steps with
/// different order values execute sequentially.
///
/// # Arguments
/// * `engine` - The workflow engine
/// * `workflow` - The workflow to execute
/// * `task_repo` - Repository for loading tasks
/// * `workflow_repo` - Repository for updating workflow state
///
/// # Returns
/// `Ok(ExecutionContext)` with execution results if successful, or
/// `WorkflowEngineError` if execution failed.
pub async fn execute_workflow_parallel(
    engine: &WorkflowEngine,
    workflow: &mut Workflow,
    task_repo: &dyn TaskRepository,
    workflow_repo: &mut dyn WorkflowRepository,
) -> Result<ExecutionContext, WorkflowEngineError> {
    info!(
        workflow_id = %workflow.id,
        step_count = workflow.steps.len(),
        "Starting parallel workflow execution"
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
    let running_state = WorkflowState::Running;
    engine.update_workflow_state(workflow, &running_state, workflow_repo)?;

    // Group steps by order for parallel execution
    let step_groups = group_steps_by_order(&sorted_steps);

    // Execute groups sequentially, steps within each group in parallel
    for (group_index, step_indices) in step_groups.iter().enumerate() {
        context.current_step_index = group_index;

        debug!(
            workflow_id = %workflow.id,
            group_index = group_index,
            step_count = step_indices.len(),
            total_groups = step_groups.len(),
            "Executing parallel step group"
        );

        // Execute steps in this group in parallel
        let step_results =
            execute_parallel_steps(engine, &sorted_steps, step_indices, &context, task_repo)
                .await?;

        // Record all step results
        for step_result in step_results {
            context.record_step_result(step_result.step_id.clone(), step_result);
        }

        info!(
            workflow_id = %workflow.id,
            group_index = group_index,
            step_count = step_indices.len(),
            "Parallel step group completed successfully"
        );
    }

    // All steps completed successfully
    context.completed_at = Some(chrono::Utc::now());
    context.current_step_index = step_groups.len();

    // Update workflow state to Completed
    let completed_state = WorkflowState::Completed;
    engine.update_workflow_state(workflow, &completed_state, workflow_repo)?;

    info!(
        workflow_id = %workflow.id,
        step_count = sorted_steps.len(),
        duration_ms = context.completed_at
            .map_or(0, |completed| completed
                .signed_duration_since(context.started_at)
                .num_milliseconds()),
        "Parallel workflow execution completed successfully"
    );

    Ok(context)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Task, WorkflowStep};
    use crate::storage::repositories::TaskRepository;
    use crate::storage::{Database, SqliteTaskRepository};
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
    fn test_group_steps_by_order() {
        let steps = vec![
            WorkflowStep::new(
                "step-1".to_string(),
                "Step 1".to_string(),
                "".to_string(),
                "task-1".to_string(),
                0,
            ),
            WorkflowStep::new(
                "step-2".to_string(),
                "Step 2".to_string(),
                "".to_string(),
                "task-2".to_string(),
                0,
            ),
            WorkflowStep::new(
                "step-3".to_string(),
                "Step 3".to_string(),
                "".to_string(),
                "task-3".to_string(),
                1,
            ),
            WorkflowStep::new(
                "step-4".to_string(),
                "Step 4".to_string(),
                "".to_string(),
                "task-4".to_string(),
                1,
            ),
            WorkflowStep::new(
                "step-5".to_string(),
                "Step 5".to_string(),
                "".to_string(),
                "task-5".to_string(),
                2,
            ),
        ];

        let groups = group_steps_by_order(&steps);
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0].len(), 2); // Steps 0 and 1 (order 0)
        assert_eq!(groups[1].len(), 2); // Steps 2 and 3 (order 1)
        assert_eq!(groups[2].len(), 1); // Step 4 (order 2)
    }

    #[test]
    fn test_group_steps_by_order_single_step() {
        let steps = vec![WorkflowStep::new(
            "step-1".to_string(),
            "Step 1".to_string(),
            "".to_string(),
            "task-1".to_string(),
            0,
        )];

        let groups = group_steps_by_order(&steps);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].len(), 1);
    }

    #[test]
    fn test_group_steps_by_order_empty() {
        let steps: Vec<WorkflowStep> = vec![];
        let groups = group_steps_by_order(&steps);
        assert_eq!(groups.len(), 0);
    }

    #[test]
    fn test_group_steps_by_order_all_same_order() {
        let steps = vec![
            WorkflowStep::new(
                "step-1".to_string(),
                "Step 1".to_string(),
                "".to_string(),
                "task-1".to_string(),
                0,
            ),
            WorkflowStep::new(
                "step-2".to_string(),
                "Step 2".to_string(),
                "".to_string(),
                "task-2".to_string(),
                0,
            ),
            WorkflowStep::new(
                "step-3".to_string(),
                "Step 3".to_string(),
                "".to_string(),
                "task-3".to_string(),
                0,
            ),
        ];

        let groups = group_steps_by_order(&steps);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].len(), 3); // All steps in same group
    }

    #[test]
    fn test_group_steps_by_order_all_different_orders() {
        let steps = vec![
            WorkflowStep::new(
                "step-1".to_string(),
                "Step 1".to_string(),
                "".to_string(),
                "task-1".to_string(),
                0,
            ),
            WorkflowStep::new(
                "step-2".to_string(),
                "Step 2".to_string(),
                "".to_string(),
                "task-2".to_string(),
                1,
            ),
            WorkflowStep::new(
                "step-3".to_string(),
                "Step 3".to_string(),
                "".to_string(),
                "task-3".to_string(),
                2,
            ),
        ];

        let groups = group_steps_by_order(&steps);
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0].len(), 1);
        assert_eq!(groups[1].len(), 1);
        assert_eq!(groups[2].len(), 1);
    }

    #[tokio::test]
    async fn test_execute_parallel_steps_success() {
        let (engine, orchestrator, _executor) = setup_test_engine();
        let mut db = Database::open_in_memory().unwrap();

        // Register agent
        let agent = Arc::new(SimpleAgent::new("test-agent".to_string(), "Test agent".to_string()));
        orchestrator.register_agent(agent).await;

        // Create tasks
        {
            let mut task_repo = SqliteTaskRepository::new(&mut db);
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

        // Create steps
        let steps = vec![
            WorkflowStep::new(
                "step-0".to_string(),
                "Step 0".to_string(),
                "First step".to_string(),
                "task-0".to_string(),
                0,
            ),
            WorkflowStep::new(
                "step-1".to_string(),
                "Step 1".to_string(),
                "Second step".to_string(),
                "task-1".to_string(),
                0,
            ),
            WorkflowStep::new(
                "step-2".to_string(),
                "Step 2".to_string(),
                "Third step".to_string(),
                "task-2".to_string(),
                0,
            ),
        ];

        let context = ExecutionContext::new("workflow-1".to_string());
        let step_indices = vec![0, 1, 2];

        let task_repo = SqliteTaskRepository::new(&mut db);
        let results = execute_parallel_steps(&engine, &steps, &step_indices, &context, &task_repo)
            .await
            .unwrap();

        assert_eq!(results.len(), 3);
        assert!(results[0].success);
        assert!(results[1].success);
        assert!(results[2].success);
    }

    #[tokio::test]
    async fn test_execute_parallel_steps_single_step() {
        let (engine, orchestrator, _executor) = setup_test_engine();
        let mut db = Database::open_in_memory().unwrap();

        // Register agent
        let agent = Arc::new(SimpleAgent::new("test-agent".to_string(), "Test agent".to_string()));
        orchestrator.register_agent(agent).await;

        // Create task
        {
            let mut task_repo = SqliteTaskRepository::new(&mut db);
            let task = Task::new(
                "task-0".to_string(),
                "Task 0".to_string(),
                "Test task".to_string(),
                "test-agent".to_string(),
                json!({"input": "test"}),
            );
            task_repo.create(&task).unwrap();
        }

        let steps = vec![WorkflowStep::new(
            "step-0".to_string(),
            "Step 0".to_string(),
            "Single step".to_string(),
            "task-0".to_string(),
            0,
        )];

        let context = ExecutionContext::new("workflow-1".to_string());
        let step_indices = vec![0];

        let task_repo = SqliteTaskRepository::new(&mut db);
        let results = execute_parallel_steps(&engine, &steps, &step_indices, &context, &task_repo)
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        assert!(results[0].success);
    }

    #[tokio::test]
    async fn test_execute_parallel_steps_empty() {
        let (engine, _orchestrator, _executor) = setup_test_engine();
        let mut db = Database::open_in_memory().unwrap();

        let steps: Vec<WorkflowStep> = vec![];
        let context = ExecutionContext::new("workflow-1".to_string());
        let step_indices: Vec<usize> = vec![];

        let task_repo = SqliteTaskRepository::new(&mut db);
        let results = execute_parallel_steps(&engine, &steps, &step_indices, &context, &task_repo)
            .await
            .unwrap();

        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_execute_parallel_steps_with_step_failure() {
        let (engine, orchestrator, _executor) = setup_test_engine();
        let mut db = Database::open_in_memory().unwrap();

        // Register agent
        let agent = Arc::new(SimpleAgent::new("test-agent".to_string(), "Test agent".to_string()));
        orchestrator.register_agent(agent).await;

        // Create tasks
        {
            let mut task_repo = SqliteTaskRepository::new(&mut db);
            let task1 = Task::new(
                "task-0".to_string(),
                "Task 0".to_string(),
                "Test task".to_string(),
                "test-agent".to_string(),
                json!({"input": "test"}),
            );
            task_repo.create(&task1).unwrap();

            // Task with non-existent agent - will cause failure
            let task2 = Task::new(
                "task-1".to_string(),
                "Task 1".to_string(),
                "Failing task".to_string(),
                "nonexistent-agent".to_string(),
                json!({"input": "test"}),
            );
            task_repo.create(&task2).unwrap();
        }

        let steps = vec![
            WorkflowStep::new(
                "step-0".to_string(),
                "Step 0".to_string(),
                "First step".to_string(),
                "task-0".to_string(),
                0,
            ),
            WorkflowStep::new(
                "step-1".to_string(),
                "Step 1".to_string(),
                "Failing step".to_string(),
                "task-1".to_string(),
                0,
            ),
        ];

        let context = ExecutionContext::new("workflow-1".to_string());
        let step_indices = vec![0, 1];

        let task_repo = SqliteTaskRepository::new(&mut db);
        let result =
            execute_parallel_steps(&engine, &steps, &step_indices, &context, &task_repo).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            WorkflowEngineError::Execution(msg) => {
                assert!(msg.contains("Parallel step execution failed"));
            }
            _ => panic!("Expected Execution error"),
        }
    }

    #[tokio::test]
    async fn test_execute_parallel_steps_task_not_found() {
        let (engine, _orchestrator, _executor) = setup_test_engine();
        let mut db = Database::open_in_memory().unwrap();

        // Create step referencing non-existent task
        let step = WorkflowStep::new(
            "step-0".to_string(),
            "Step 0".to_string(),
            "Step with missing task".to_string(),
            "nonexistent-task".to_string(),
            0,
        );

        let steps = vec![step];
        let context = ExecutionContext::new("workflow-1".to_string());
        let step_indices = vec![0];

        let task_repo = SqliteTaskRepository::new(&mut db);
        let result =
            execute_parallel_steps(&engine, &steps, &step_indices, &context, &task_repo).await;

        // Should fail because task doesn't exist
        assert!(result.is_err());
        match result.unwrap_err() {
            WorkflowEngineError::Execution(msg) => {
                assert!(msg.contains("Parallel step execution failed"));
            }
            _ => panic!("Expected Execution error"),
        }
    }

    // Note: Full integration tests for execute_workflow_parallel would require Arc<Mutex<Database>>
    // pattern due to simultaneous TaskRepository and WorkflowRepository borrows.
    // See executor.rs for similar pattern. The execute_parallel_steps function is thoroughly tested above.
}
