#![cfg(feature = "workflow")]

//! Tests for parallel workflow step execution.

use radium_core::models::{Task, Workflow, WorkflowStep};
use radium_core::storage::{Database, SqliteTaskRepository, TaskRepository};
use radium_core::workflow::engine::{ExecutionContext, WorkflowEngine, WorkflowEngineError};
use radium_core::workflow::parallel::{execute_parallel_steps, group_steps_by_order};
use radium_orchestrator::{AgentExecutor, Orchestrator, SimpleAgent};
use serde_json::json;
use std::sync::Arc;

#[test]
fn test_group_steps_by_order_same_order() {
    let steps = vec![
        WorkflowStep::new(
            "step-1".to_string(),
            "Step 1".to_string(),
            "Desc".to_string(),
            "task-1".to_string(),
            0,
        ),
        WorkflowStep::new(
            "step-2".to_string(),
            "Step 2".to_string(),
            "Desc".to_string(),
            "task-2".to_string(),
            0,
        ),
        WorkflowStep::new(
            "step-3".to_string(),
            "Step 3".to_string(),
            "Desc".to_string(),
            "task-3".to_string(),
            0,
        ),
    ];

    let groups = group_steps_by_order(&steps);
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].len(), 3);
}

#[test]
fn test_group_steps_by_order_different_orders() {
    let steps = vec![
        WorkflowStep::new(
            "step-1".to_string(),
            "Step 1".to_string(),
            "Desc".to_string(),
            "task-1".to_string(),
            0,
        ),
        WorkflowStep::new(
            "step-2".to_string(),
            "Step 2".to_string(),
            "Desc".to_string(),
            "task-2".to_string(),
            1,
        ),
        WorkflowStep::new(
            "step-3".to_string(),
            "Step 3".to_string(),
            "Desc".to_string(),
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

#[test]
fn test_group_steps_by_order_empty() {
    let steps = vec![];
    let groups = group_steps_by_order(&steps);
    assert_eq!(groups.len(), 0);
}

#[test]
fn test_group_steps_by_order_mixed() {
    let steps = vec![
        WorkflowStep::new(
            "step-1".to_string(),
            "Step 1".to_string(),
            "Desc".to_string(),
            "task-1".to_string(),
            0,
        ),
        WorkflowStep::new(
            "step-2".to_string(),
            "Step 2".to_string(),
            "Desc".to_string(),
            "task-2".to_string(),
            0,
        ),
        WorkflowStep::new(
            "step-3".to_string(),
            "Step 3".to_string(),
            "Desc".to_string(),
            "task-3".to_string(),
            1,
        ),
        WorkflowStep::new(
            "step-4".to_string(),
            "Step 4".to_string(),
            "Desc".to_string(),
            "task-4".to_string(),
            1,
        ),
        WorkflowStep::new(
            "step-5".to_string(),
            "Step 5".to_string(),
            "Desc".to_string(),
            "task-5".to_string(),
            2,
        ),
    ];

    let groups = group_steps_by_order(&steps);
    assert_eq!(groups.len(), 3);
    assert_eq!(groups[0].len(), 2); // order 0
    assert_eq!(groups[1].len(), 2); // order 1
    assert_eq!(groups[2].len(), 1); // order 2
}

#[test]
fn test_group_steps_by_order_large_values() {
    let steps = vec![
        WorkflowStep::new(
            "step-1".to_string(),
            "Step 1".to_string(),
            "Desc".to_string(),
            "task-1".to_string(),
            100,
        ),
        WorkflowStep::new(
            "step-2".to_string(),
            "Step 2".to_string(),
            "Desc".to_string(),
            "task-2".to_string(),
            100,
        ),
        WorkflowStep::new(
            "step-3".to_string(),
            "Step 3".to_string(),
            "Desc".to_string(),
            "task-3".to_string(),
            200,
        ),
    ];

    let groups = group_steps_by_order(&steps);
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].len(), 2); // order 100
    assert_eq!(groups[1].len(), 1); // order 200
}

#[tokio::test]
async fn test_execute_parallel_steps_all_succeed() {
    let mut db = Database::open_in_memory().unwrap();
    let orchestrator = Arc::new(Orchestrator::new());
    let executor = Arc::new(AgentExecutor::with_mock_model());
    let engine = WorkflowEngine::new(Arc::clone(&orchestrator), Arc::clone(&executor));

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

    // Create steps with same order (can run in parallel)
    let steps = vec![
        WorkflowStep::new(
            "step-1".to_string(),
            "Step 1".to_string(),
            "Desc".to_string(),
            "task-0".to_string(),
            0,
        ),
        WorkflowStep::new(
            "step-2".to_string(),
            "Step 2".to_string(),
            "Desc".to_string(),
            "task-1".to_string(),
            0,
        ),
        WorkflowStep::new(
            "step-3".to_string(),
            "Step 3".to_string(),
            "Desc".to_string(),
            "task-2".to_string(),
            0,
        ),
    ];

    let context = ExecutionContext::new("workflow-1".to_string());
    let task_repo = SqliteTaskRepository::new(&mut db);
    let step_indices = vec![0, 1, 2];

    let result = execute_parallel_steps(&engine, &steps, &step_indices, &context, &task_repo).await;
    assert!(result.is_ok());
    let step_results = result.unwrap();
    assert_eq!(step_results.len(), 3);
    assert!(step_results.iter().all(|r| r.success));
}

#[tokio::test]
async fn test_execute_parallel_steps_some_fail() {
    let mut db = Database::open_in_memory().unwrap();
    let orchestrator = Arc::new(Orchestrator::new());
    let executor = Arc::new(AgentExecutor::with_mock_model());
    let engine = WorkflowEngine::new(Arc::clone(&orchestrator), Arc::clone(&executor));

    // Register agent
    let agent = Arc::new(SimpleAgent::new("test-agent".to_string(), "Test agent".to_string()));
    orchestrator.register_agent(agent).await;

    // Create tasks - one with non-existent agent to trigger failure
    {
        let mut task_repo = SqliteTaskRepository::new(&mut db);
        let task1 = Task::new(
            "task-1".to_string(),
            "Task 1".to_string(),
            "Test task".to_string(),
            "test-agent".to_string(),
            json!({"input": "test1"}),
        );
        task_repo.create(&task1).unwrap();

        let task2 = Task::new(
            "task-2".to_string(),
            "Task 2".to_string(),
            "Test task".to_string(),
            "nonexistent-agent".to_string(),
            json!({"input": "test2"}),
        );
        task_repo.create(&task2).unwrap();
    }

    let steps = vec![
        WorkflowStep::new(
            "step-1".to_string(),
            "Step 1".to_string(),
            "Desc".to_string(),
            "task-1".to_string(),
            0,
        ),
        WorkflowStep::new(
            "step-2".to_string(),
            "Step 2".to_string(),
            "Desc".to_string(),
            "task-2".to_string(),
            0,
        ),
    ];

    let context = ExecutionContext::new("workflow-1".to_string());
    let task_repo = SqliteTaskRepository::new(&mut db);
    let step_indices = vec![0, 1];

    let result = execute_parallel_steps(&engine, &steps, &step_indices, &context, &task_repo).await;
    // Should return error because step-2 fails
    assert!(result.is_err());
}

#[tokio::test]
async fn test_execute_parallel_steps_with_conditions() {
    let mut db = Database::open_in_memory().unwrap();
    let orchestrator = Arc::new(Orchestrator::new());
    let executor = Arc::new(AgentExecutor::with_mock_model());
    let engine = WorkflowEngine::new(Arc::clone(&orchestrator), Arc::clone(&executor));

    // Register agent
    let agent = Arc::new(SimpleAgent::new("test-agent".to_string(), "Test agent".to_string()));
    orchestrator.register_agent(agent).await;

    // Create tasks
    {
        let mut task_repo = SqliteTaskRepository::new(&mut db);
        for i in 0..2 {
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

    // Create context with step-1 already completed
    let mut context = ExecutionContext::new("workflow-1".to_string());
    context.record_step_result(
        "step-1".to_string(),
        radium_core::workflow::engine::StepResult::success(
            "step-1".to_string(),
            json!("output"),
            chrono::Utc::now(),
            chrono::Utc::now(),
        ),
    );

    // Step 2 depends on step-1, step 3 doesn't
    let mut step2 = WorkflowStep::new(
        "step-2".to_string(),
        "Step 2".to_string(),
        "Desc".to_string(),
        "task-0".to_string(),
        0,
    );
    step2.config_json = Some(
        serde_json::to_string(&serde_json::json!({
            "dependsOn": ["step-1"]
        }))
        .unwrap(),
    );

    let step3 = WorkflowStep::new(
        "step-3".to_string(),
        "Step 3".to_string(),
        "Desc".to_string(),
        "task-1".to_string(),
        0,
    );

    let steps = vec![step2, step3];
    let task_repo = SqliteTaskRepository::new(&mut db);
    let step_indices = vec![0, 1];

    let result = execute_parallel_steps(&engine, &steps, &step_indices, &context, &task_repo).await;
    assert!(result.is_ok());
    let step_results = result.unwrap();
    // Both steps should execute (step-2 has dependency satisfied)
    assert_eq!(step_results.len(), 2);
}

#[tokio::test]
async fn test_execute_parallel_steps_task_not_found() {
    let mut db = Database::open_in_memory().unwrap();
    let orchestrator = Arc::new(Orchestrator::new());
    let executor = Arc::new(AgentExecutor::with_mock_model());
    let engine = WorkflowEngine::new(Arc::clone(&orchestrator), Arc::clone(&executor));

    // Register agent
    let agent = Arc::new(SimpleAgent::new("test-agent".to_string(), "Test agent".to_string()));
    orchestrator.register_agent(agent).await;

    // Don't create tasks - step references non-existent task
    let steps = vec![WorkflowStep::new(
        "step-1".to_string(),
        "Step 1".to_string(),
        "Desc".to_string(),
        "nonexistent-task".to_string(),
        0,
    )];

    let context = ExecutionContext::new("workflow-1".to_string());
    let task_repo = SqliteTaskRepository::new(&mut db);
    let step_indices = vec![0];

    let result = execute_parallel_steps(&engine, &steps, &step_indices, &context, &task_repo).await;
    // Should return an error when task is not found
    assert!(result.is_err());
}
