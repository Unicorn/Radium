//! Integration tests for the workflow engine.
//!
//! These tests verify that the workflow engine correctly executes workflows,
//! handles errors, and integrates with the agent orchestrator.

use radium_orchestrator::{AgentExecutor, Orchestrator, SimpleAgent};
use radium_core::models::{Task, Workflow, WorkflowStep};
use radium_core::storage::{
    Database, SqliteTaskRepository, SqliteWorkflowRepository, TaskRepository, WorkflowRepository,
};
use radium_core::workflow::{WorkflowEngine, WorkflowExecutor};
use serde_json::json;
use std::sync::Arc;

/// Sets up a test database with sample data.
fn setup_test_db() -> Database {
    Database::open_in_memory().unwrap()
}

/// Creates a test workflow with the given steps.
fn create_test_workflow(step_count: usize) -> Workflow {
    let mut workflow = Workflow::new(
        "test-workflow".to_string(),
        "Test Workflow".to_string(),
        "A test workflow".to_string(),
    );

    for i in 0..step_count {
        workflow
            .add_step(WorkflowStep::new(
                format!("step-{}", i),
                format!("Step {}", i),
                format!("Test step {}", i),
                format!("task-{}", i),
                i as u32,
            ))
            .unwrap();
    }

    workflow
}

/// Creates test tasks for a workflow.
fn create_test_tasks(db: &mut Database, task_count: usize, agent_id: &str) {
    let mut task_repo = SqliteTaskRepository::new(db);
    for i in 0..task_count {
        let task = Task::new(
            format!("task-{}", i),
            format!("Task {}", i),
            format!("Test task {}", i),
            agent_id.to_string(),
            json!({"input": format!("test-{}", i)}),
        );
        task_repo.create(&task).unwrap();
    }
}

#[tokio::test]
async fn test_workflow_engine_executes_step() {
    // Setup
    let mut db = setup_test_db();
    let orchestrator = Arc::new(Orchestrator::new());
    let executor = Arc::new(AgentExecutor::with_mock_model());
    let engine = WorkflowEngine::new(Arc::clone(&orchestrator), Arc::clone(&executor));

    // Register agent
    let agent = Arc::new(SimpleAgent::new("test-agent".to_string(), "Test agent".to_string()));
    orchestrator.register_agent(agent).await;

    // Create task
    create_test_tasks(&mut db, 1, "test-agent");

    // Create workflow step
    let step = WorkflowStep::new(
        "step-1".to_string(),
        "Step 1".to_string(),
        "Test step".to_string(),
        "task-0".to_string(),
        0,
    );

    // Create execution context
    let context = radium_core::workflow::ExecutionContext::new("test-workflow".to_string());

    // Execute step
    let task_repo = SqliteTaskRepository::new(&mut db);
    let result = engine.execute_step(&step, &context, &task_repo).await;

    assert!(result.is_ok());
    let step_result = result.unwrap();
    assert!(step_result.success);
    assert_eq!(step_result.step_id, "step-1");
}

#[tokio::test]
async fn test_workflow_executor_sequential_execution() {
    // Setup
    let db = setup_test_db();
    let orchestrator = Arc::new(Orchestrator::new());
    let executor = Arc::new(AgentExecutor::with_mock_model());
    let workflow_executor = WorkflowExecutor::new(Arc::clone(&orchestrator), Arc::clone(&executor));

    // Register agent
    let agent = Arc::new(SimpleAgent::new("test-agent".to_string(), "Test agent".to_string()));
    orchestrator.register_agent(agent).await;

    // Use separate databases to avoid borrow conflicts
    let mut task_db = db;
    let mut workflow_db = Database::open_in_memory().unwrap();

    // Create tasks in task_db
    create_test_tasks(&mut task_db, 3, "test-agent");

    // Create workflow in workflow_db
    let workflow = create_test_workflow(3);
    {
        let mut workflow_repo = SqliteWorkflowRepository::new(&mut workflow_db);
        workflow_repo.create(&workflow).unwrap();
    }

    // Execute workflow
    let mut workflow = {
        let workflow_repo = SqliteWorkflowRepository::new(&mut workflow_db);
        workflow_repo.get_by_id("test-workflow").unwrap()
    };

    let result = {
        let task_repo = SqliteTaskRepository::new(&mut task_db);
        let mut workflow_repo = SqliteWorkflowRepository::new(&mut workflow_db);
        workflow_executor.execute_workflow(&mut workflow, &task_repo, &mut workflow_repo).await
    };

    assert!(result.is_ok());
    let context = result.unwrap();
    assert_eq!(context.step_results.len(), 3);
    assert!(context.step_results.get("step-0").unwrap().success);
    assert!(context.step_results.get("step-1").unwrap().success);
    assert!(context.step_results.get("step-2").unwrap().success);
    assert_eq!(workflow.state, radium_core::models::WorkflowState::Completed);
}

#[tokio::test]
async fn test_workflow_executor_handles_missing_task() {
    // Setup
    let mut db = setup_test_db();
    let orchestrator = Arc::new(Orchestrator::new());
    let executor = Arc::new(AgentExecutor::with_mock_model());
    let workflow_executor = WorkflowExecutor::new(Arc::clone(&orchestrator), Arc::clone(&executor));

    // Register agent
    let agent = Arc::new(SimpleAgent::new("test-agent".to_string(), "Test agent".to_string()));
    orchestrator.register_agent(agent).await;

    // Create workflow without creating tasks
    let workflow = create_test_workflow(1);
    {
        let mut workflow_repo = SqliteWorkflowRepository::new(&mut db);
        workflow_repo.create(&workflow).unwrap();
    }

    // Execute workflow - should fail because task doesn't exist
    // Use separate databases to avoid borrow checker issues
    let mut task_db = Database::open_in_memory().unwrap();
    let mut workflow_db = db;

    let mut workflow = {
        let workflow_repo = SqliteWorkflowRepository::new(&mut workflow_db);
        workflow_repo.get_by_id("test-workflow").unwrap()
    };

    let result = {
        let task_repo = SqliteTaskRepository::new(&mut task_db);
        let mut workflow_repo = SqliteWorkflowRepository::new(&mut workflow_db);
        workflow_executor.execute_workflow(&mut workflow, &task_repo, &mut workflow_repo).await
    };

    assert!(result.is_err());
    assert!(matches!(workflow.state, radium_core::models::WorkflowState::Error(_)));
}

#[tokio::test]
async fn test_workflow_executor_handles_missing_agent() {
    // Setup
    let mut db = setup_test_db();
    let orchestrator = Arc::new(Orchestrator::new());
    let executor = Arc::new(AgentExecutor::with_mock_model());
    let workflow_executor = WorkflowExecutor::new(Arc::clone(&orchestrator), Arc::clone(&executor));

    // Create task with non-existent agent
    create_test_tasks(&mut db, 1, "nonexistent-agent");

    // Create workflow
    let workflow = create_test_workflow(1);
    let mut workflow_db = Database::open_in_memory().unwrap();
    {
        let mut workflow_repo = SqliteWorkflowRepository::new(&mut workflow_db);
        workflow_repo.create(&workflow).unwrap();
    }

    // Execute workflow - should fail because agent doesn't exist
    // Use separate databases to avoid borrow checker issues
    let mut task_db = db;

    let mut workflow = {
        let workflow_repo = SqliteWorkflowRepository::new(&mut workflow_db);
        workflow_repo.get_by_id("test-workflow").unwrap()
    };

    let result = {
        let task_repo = SqliteTaskRepository::new(&mut task_db);
        let mut workflow_repo = SqliteWorkflowRepository::new(&mut workflow_db);
        workflow_executor.execute_workflow(&mut workflow, &task_repo, &mut workflow_repo).await
    };

    assert!(result.is_err());
    assert!(matches!(workflow.state, radium_core::models::WorkflowState::Error(_)));
}

#[tokio::test]
async fn test_workflow_state_transitions() {
    // Setup
    let mut db = setup_test_db();
    let orchestrator = Arc::new(Orchestrator::new());
    let executor = Arc::new(AgentExecutor::with_mock_model());
    let engine = WorkflowEngine::new(Arc::clone(&orchestrator), Arc::clone(&executor));

    // Create workflow
    let workflow = create_test_workflow(1);
    {
        let mut workflow_repo = SqliteWorkflowRepository::new(&mut db);
        workflow_repo.create(&workflow).unwrap();
    }

    // Test state transitions
    let mut workflow = {
        let workflow_repo = SqliteWorkflowRepository::new(&mut db);
        workflow_repo.get_by_id("test-workflow").unwrap()
    };
    assert_eq!(workflow.state, radium_core::models::WorkflowState::Idle);

    {
        let mut workflow_repo = SqliteWorkflowRepository::new(&mut db);
        let running_state = radium_core::models::WorkflowState::Running;
        engine.update_workflow_state(&mut workflow, &running_state, &mut workflow_repo).unwrap();
    }
    assert_eq!(workflow.state, radium_core::models::WorkflowState::Running);

    {
        let mut workflow_repo = SqliteWorkflowRepository::new(&mut db);
        let completed_state = radium_core::models::WorkflowState::Completed;
        engine.update_workflow_state(&mut workflow, &completed_state, &mut workflow_repo).unwrap();
    }

    // Verify final state
    let workflow = {
        let workflow_repo = SqliteWorkflowRepository::new(&mut db);
        workflow_repo.get_by_id("test-workflow").unwrap()
    };
    assert_eq!(workflow.state, radium_core::models::WorkflowState::Completed);
}
