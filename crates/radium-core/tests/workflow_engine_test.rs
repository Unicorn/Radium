#![cfg(feature = "workflow")]

//! Integration tests for the workflow engine.
//!
//! These tests verify that the workflow engine correctly executes workflows,
//! handles errors, and integrates with the agent orchestrator.

use radium_core::models::{Task, Workflow, WorkflowStep};
use radium_core::storage::{
    Database, SqliteTaskRepository, SqliteWorkflowRepository, TaskRepository, WorkflowRepository,
};
use radium_core::workflow::{WorkflowEngine, WorkflowExecutor};
use radium_orchestrator::{AgentExecutor, Orchestrator, SimpleAgent};
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
    let workflow_executor =
        WorkflowExecutor::new(Arc::clone(&orchestrator), Arc::clone(&executor), None);

    // Register agent
    let agent = Arc::new(SimpleAgent::new("test-agent".to_string(), "Test agent".to_string()));
    orchestrator.register_agent(agent).await;

    // Use a single database for both tasks and workflows
    let db = Arc::new(std::sync::Mutex::new(db));

    // Create tasks
    {
        let mut db_lock = db.lock().unwrap();
        create_test_tasks(&mut *db_lock, 3, "test-agent");
    }

    // Create workflow
    let workflow = create_test_workflow(3);
    {
        let mut db_lock = db.lock().unwrap();
        let mut workflow_repo = SqliteWorkflowRepository::new(&mut *db_lock);
        workflow_repo.create(&workflow).unwrap();
    }

    // Execute workflow
    let mut workflow = {
        let mut db_lock = db.lock().unwrap();
        let workflow_repo = SqliteWorkflowRepository::new(&mut *db_lock);
        workflow_repo.get_by_id("test-workflow").unwrap()
    };

    let result = workflow_executor.execute_workflow(&mut workflow, Arc::clone(&db)).await;

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
    let workflow_executor =
        WorkflowExecutor::new(Arc::clone(&orchestrator), Arc::clone(&executor), None);

    // Register agent
    let agent = Arc::new(SimpleAgent::new("test-agent".to_string(), "Test agent".to_string()));
    orchestrator.register_agent(agent).await;

    // Create workflow without creating tasks
    let workflow = create_test_workflow(1);
    let db = Arc::new(std::sync::Mutex::new(db));
    {
        let mut db_lock = db.lock().unwrap();
        let mut workflow_repo = SqliteWorkflowRepository::new(&mut *db_lock);
        workflow_repo.create(&workflow).unwrap();
    }

    // Execute workflow - should fail because task doesn't exist
    let mut workflow = {
        let mut db_lock = db.lock().unwrap();
        let workflow_repo = SqliteWorkflowRepository::new(&mut *db_lock);
        workflow_repo.get_by_id("test-workflow").unwrap()
    };

    let result = workflow_executor.execute_workflow(&mut workflow, Arc::clone(&db)).await;

    assert!(result.is_err());
    assert!(matches!(workflow.state, radium_core::models::WorkflowState::Error(_)));
}

#[tokio::test]
async fn test_workflow_executor_handles_missing_agent() {
    // Setup
    let mut db = setup_test_db();
    let orchestrator = Arc::new(Orchestrator::new());
    let executor = Arc::new(AgentExecutor::with_mock_model());
    let workflow_executor =
        WorkflowExecutor::new(Arc::clone(&orchestrator), Arc::clone(&executor), None);

    // Create task with non-existent agent
    let db = Arc::new(std::sync::Mutex::new(db));
    {
        let mut db_lock = db.lock().unwrap();
        create_test_tasks(&mut *db_lock, 1, "nonexistent-agent");
    }

    // Create workflow
    let workflow = create_test_workflow(1);
    {
        let mut db_lock = db.lock().unwrap();
        let mut workflow_repo = SqliteWorkflowRepository::new(&mut *db_lock);
        workflow_repo.create(&workflow).unwrap();
    }

    // Execute workflow - should fail because agent doesn't exist
    let mut workflow = {
        let mut db_lock = db.lock().unwrap();
        let workflow_repo = SqliteWorkflowRepository::new(&mut *db_lock);
        workflow_repo.get_by_id("test-workflow").unwrap()
    };

    let result = workflow_executor.execute_workflow(&mut workflow, Arc::clone(&db)).await;

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

#[tokio::test]
async fn test_workflow_engine_execute_step_task_not_found() {
    let mut db = setup_test_db();
    let orchestrator = Arc::new(Orchestrator::new());
    let executor = Arc::new(AgentExecutor::with_mock_model());
    let engine = WorkflowEngine::new(Arc::clone(&orchestrator), Arc::clone(&executor));

    // Register agent
    let agent = Arc::new(SimpleAgent::new("test-agent".to_string(), "Test agent".to_string()));
    orchestrator.register_agent(agent).await;

    // Create step with non-existent task
    let step = WorkflowStep::new(
        "step-1".to_string(),
        "Step 1".to_string(),
        "Test step".to_string(),
        "nonexistent-task".to_string(),
        0,
    );

    let context = radium_core::workflow::ExecutionContext::new("test-workflow".to_string());
    let task_repo = SqliteTaskRepository::new(&mut db);
    let result = engine.execute_step(&step, &context, &task_repo).await;

    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, radium_core::workflow::engine::WorkflowEngineError::TaskNotFound(_)));
    }
}

#[tokio::test]
async fn test_workflow_engine_execute_step_agent_not_found() {
    let mut db = setup_test_db();
    let orchestrator = Arc::new(Orchestrator::new());
    let executor = Arc::new(AgentExecutor::with_mock_model());
    let engine = WorkflowEngine::new(Arc::clone(&orchestrator), Arc::clone(&executor));

    // Create task with non-existent agent (don't register agent)
    create_test_tasks(&mut db, 1, "nonexistent-agent");

    let step = WorkflowStep::new(
        "step-1".to_string(),
        "Step 1".to_string(),
        "Test step".to_string(),
        "task-0".to_string(),
        0,
    );

    let context = radium_core::workflow::ExecutionContext::new("test-workflow".to_string());
    let task_repo = SqliteTaskRepository::new(&mut db);
    let result = engine.execute_step(&step, &context, &task_repo).await;

    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, radium_core::workflow::engine::WorkflowEngineError::AgentNotFound(_)));
    }
}

#[tokio::test]
async fn test_workflow_engine_execute_step_different_agent_output_types() {
    let mut db = setup_test_db();
    let orchestrator = Arc::new(Orchestrator::new());
    let executor = Arc::new(AgentExecutor::with_mock_model());
    let engine = WorkflowEngine::new(Arc::clone(&orchestrator), Arc::clone(&executor));

    // Register agent
    let agent = Arc::new(SimpleAgent::new("test-agent".to_string(), "Test agent".to_string()));
    orchestrator.register_agent(agent).await;

    // Create task with different input types to test serialization
    let mut task_repo = SqliteTaskRepository::new(&mut db);

    // Test with string input (should work directly)
    let task1 = Task::new(
        "task-1".to_string(),
        "Task 1".to_string(),
        "Test task".to_string(),
        "test-agent".to_string(),
        json!("simple string input"),
    );
    task_repo.create(&task1).unwrap();

    // Test with object input (should serialize)
    let task2 = Task::new(
        "task-2".to_string(),
        "Task 2".to_string(),
        "Test task".to_string(),
        "test-agent".to_string(),
        json!({"key": "value", "number": 42}),
    );
    task_repo.create(&task2).unwrap();

    let context = radium_core::workflow::ExecutionContext::new("test-workflow".to_string());

    // Execute step with string input
    let step1 = WorkflowStep::new(
        "step-1".to_string(),
        "Step 1".to_string(),
        "Test step".to_string(),
        "task-1".to_string(),
        0,
    );
    let result1 = engine.execute_step(&step1, &context, &task_repo).await;
    assert!(result1.is_ok());
    assert!(result1.unwrap().success);

    // Execute step with object input
    let step2 = WorkflowStep::new(
        "step-2".to_string(),
        "Step 2".to_string(),
        "Test step".to_string(),
        "task-2".to_string(),
        0,
    );
    let result2 = engine.execute_step(&step2, &context, &task_repo).await;
    assert!(result2.is_ok());
    assert!(result2.unwrap().success);
}

#[tokio::test]
async fn test_workflow_engine_update_workflow_state_error() {
    let mut db = setup_test_db();
    let orchestrator = Arc::new(Orchestrator::new());
    let executor = Arc::new(AgentExecutor::with_mock_model());
    let engine = WorkflowEngine::new(Arc::clone(&orchestrator), Arc::clone(&executor));

    // Create workflow
    let workflow = create_test_workflow(0);
    {
        let mut workflow_repo = SqliteWorkflowRepository::new(&mut db);
        workflow_repo.create(&workflow).unwrap();
    }

    // Get workflow
    let mut workflow = {
        let workflow_repo = SqliteWorkflowRepository::new(&mut db);
        workflow_repo.get_by_id("test-workflow").unwrap()
    };

    // Test that the method works correctly with valid inputs
    // The error path (line 345-353) would require a mock repository
    let mut workflow_repo = SqliteWorkflowRepository::new(&mut db);
    let running_state = radium_core::models::WorkflowState::Running;
    let result = engine.update_workflow_state(&mut workflow, &running_state, &mut workflow_repo);
    assert!(result.is_ok());
}

#[test]
fn test_execution_context_methods() {
    use chrono::Utc;
    use radium_core::workflow::engine::{ExecutionContext, StepResult};
    use serde_json::json;

    let mut context = ExecutionContext::new("test-workflow".to_string());

    // Test set_variable and get_variable
    context.set_variable("test_var".to_string(), json!("test_value"));
    assert_eq!(context.get_variable("test_var"), Some(&json!("test_value")));
    assert_eq!(context.get_variable("nonexistent"), None);

    // Test record_step_result and get_step_result
    let step_result =
        StepResult::success("step-1".to_string(), json!("output"), Utc::now(), Utc::now());
    context.record_step_result("step-1".to_string(), step_result.clone());

    let retrieved = context.get_step_result("step-1");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().step_id, "step-1");
    assert_eq!(retrieved.unwrap().success, true);

    // Test overwriting existing result
    let new_result =
        StepResult::failure("step-1".to_string(), "new error".to_string(), Utc::now(), Utc::now());
    context.record_step_result("step-1".to_string(), new_result.clone());
    let retrieved = context.get_step_result("step-1");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().success, false);

    // Test get_step_result with missing step
    assert!(context.get_step_result("nonexistent").is_none());

    // Test completed_at
    assert!(context.completed_at.is_none());
    context.completed_at = Some(Utc::now());
    assert!(context.completed_at.is_some());
}

#[test]
fn test_step_result_constructors() {
    use chrono::Utc;
    use radium_core::workflow::engine::StepResult;
    use serde_json::json;

    // Test success constructor
    let success_result =
        StepResult::success("step-1".to_string(), json!("output"), Utc::now(), Utc::now());
    assert_eq!(success_result.step_id, "step-1");
    assert_eq!(success_result.success, true);
    assert_eq!(success_result.output, Some(json!("output")));
    assert!(success_result.error.is_none());

    // Test success with different output types
    let success_with_object =
        StepResult::success("step-2".to_string(), json!({"key": "value"}), Utc::now(), Utc::now());
    assert!(success_with_object.success);
    assert_eq!(success_with_object.output, Some(json!({"key": "value"})));

    // Test failure constructor
    let failure_result = StepResult::failure(
        "step-3".to_string(),
        "error message".to_string(),
        Utc::now(),
        Utc::now(),
    );
    assert_eq!(failure_result.step_id, "step-3");
    assert_eq!(failure_result.success, false);
    assert_eq!(failure_result.error, Some("error message".to_string()));

    // Test failure with empty error message
    let failure_empty =
        StepResult::failure("step-4".to_string(), "".to_string(), Utc::now(), Utc::now());
    assert!(!failure_empty.success);
    assert_eq!(failure_empty.error, Some("".to_string()));
}
