//! Integration tests for workflow execution.

use radium_core::models::{Task, Workflow, WorkflowState, WorkflowStep};
use radium_core::storage::{
    Database, SqliteTaskRepository, SqliteWorkflowRepository, TaskRepository, WorkflowRepository,
};
use radium_core::workflow::WorkflowExecutor;
use radium_orchestrator::{AgentExecutor, Orchestrator, SimpleAgent};
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn test_workflow_execution_end_to_end() {
    // Use a single database wrapped in Arc<Mutex<>>
    let db = Arc::new(std::sync::Mutex::new(Database::open_in_memory().unwrap()));
    let orchestrator = Arc::new(Orchestrator::new());
    let executor = Arc::new(AgentExecutor::with_mock_model());
    let workflow_executor = WorkflowExecutor::new(Arc::clone(&orchestrator), Arc::clone(&executor));

    // Register agent
    let agent = Arc::new(SimpleAgent::new("test-agent".to_string(), "Test agent".to_string()));
    orchestrator.register_agent(agent).await;

    // Create tasks
    {
        let mut db_lock = db.lock().unwrap();
        let mut task_repo = SqliteTaskRepository::new(&mut *db_lock);
        for i in 1..=3 {
            let task = Task::new(
                format!("task-{}", i),
                format!("Task {}", i),
                format!("Task {} description", i),
                "test-agent".to_string(),
                json!({"input": format!("input-{}", i)}),
            );
            task_repo.create(&task).unwrap();
        }
    }

    // Create workflow with multiple steps
    {
        let mut db_lock = db.lock().unwrap();
        let mut workflow_repo = SqliteWorkflowRepository::new(&mut *db_lock);
        let mut workflow = Workflow::new(
            "workflow-1".to_string(),
            "Multi-step Workflow".to_string(),
            "A workflow with multiple steps".to_string(),
        );
        for i in 0..3 {
            workflow
                .add_step(WorkflowStep::new(
                    format!("step-{}", i + 1),
                    format!("Step {}", i + 1),
                    format!("Step {} description", i + 1),
                    format!("task-{}", i + 1),
                    i,
                ))
                .unwrap();
        }
        workflow_repo.create(&workflow).unwrap();
    }

    // Execute workflow
    let mut workflow = {
        let mut db_lock = db.lock().unwrap();
        let workflow_repo = SqliteWorkflowRepository::new(&mut *db_lock);
        workflow_repo.get_by_id("workflow-1").unwrap()
    };

    let context = workflow_executor
        .execute_workflow(&mut workflow, Arc::clone(&db))
        .await
        .unwrap();

    // Verify execution results
    assert_eq!(context.workflow_id, "workflow-1");
    assert_eq!(context.step_results.len(), 3);
    assert!(context.step_results.get("step-1").unwrap().success);
    assert!(context.step_results.get("step-2").unwrap().success);
    assert!(context.step_results.get("step-3").unwrap().success);
    assert!(context.completed_at.is_some());

    // Verify workflow state persisted
    let mut db_lock = db.lock().unwrap();
    let workflow_repo = SqliteWorkflowRepository::new(&mut *db_lock);
    let workflow = workflow_repo.get_by_id("workflow-1").unwrap();
    assert_eq!(workflow.state, WorkflowState::Completed);
}

#[tokio::test]
async fn test_workflow_error_recovery() {
    let db = Arc::new(std::sync::Mutex::new(Database::open_in_memory().unwrap()));
    let orchestrator = Arc::new(Orchestrator::new());
    let executor = Arc::new(AgentExecutor::with_mock_model());
    let workflow_executor = WorkflowExecutor::new(Arc::clone(&orchestrator), Arc::clone(&executor));

    // Register agent
    let agent = Arc::new(SimpleAgent::new("test-agent".to_string(), "Test agent".to_string()));
    orchestrator.register_agent(agent).await;

    // Create task that will fail (using non-existent agent in task)
    {
        let mut db_lock = db.lock().unwrap();
        let mut task_repo = SqliteTaskRepository::new(&mut *db_lock);
        let task = Task::new(
            "task-1".to_string(),
            "Task 1".to_string(),
            "Task that will fail".to_string(),
            "nonexistent-agent".to_string(), // This will cause failure
            json!({"input": "test"}),
        );
        task_repo.create(&task).unwrap();
    }

    // Create workflow
    {
        let mut db_lock = db.lock().unwrap();
        let mut workflow_repo = SqliteWorkflowRepository::new(&mut *db_lock);
        let mut workflow = Workflow::new(
            "workflow-1".to_string(),
            "Failing Workflow".to_string(),
            "A workflow that will fail".to_string(),
        );
        workflow
            .add_step(WorkflowStep::new(
                "step-1".to_string(),
                "Step 1".to_string(),
                "Failing step".to_string(),
                "task-1".to_string(),
                0,
            ))
            .unwrap();
        workflow_repo.create(&workflow).unwrap();
    }

    // Execute workflow - should fail
    let mut workflow = {
        let mut db_lock = db.lock().unwrap();
        let workflow_repo = SqliteWorkflowRepository::new(&mut *db_lock);
        workflow_repo.get_by_id("workflow-1").unwrap()
    };

    let result = workflow_executor.execute_workflow(&mut workflow, Arc::clone(&db)).await;

    assert!(result.is_err());

    // Verify workflow state is Error
    let mut db_lock = db.lock().unwrap();
    let workflow_repo = SqliteWorkflowRepository::new(&mut *db_lock);
    let workflow = workflow_repo.get_by_id("workflow-1").unwrap();
    match &workflow.state {
        WorkflowState::Error(_) => {}
        _ => panic!("Expected workflow to be in Error state"),
    }
}

#[tokio::test]
async fn test_workflow_state_persistence() {
    let db = Arc::new(std::sync::Mutex::new(Database::open_in_memory().unwrap()));
    let orchestrator = Arc::new(Orchestrator::new());
    let executor = Arc::new(AgentExecutor::with_mock_model());
    let workflow_executor = WorkflowExecutor::new(Arc::clone(&orchestrator), Arc::clone(&executor));

    // Register agent
    let agent = Arc::new(SimpleAgent::new("test-agent".to_string(), "Test agent".to_string()));
    orchestrator.register_agent(agent).await;

    // Create task
    {
        let mut db_lock = db.lock().unwrap();
        let mut task_repo = SqliteTaskRepository::new(&mut *db_lock);
        let task = Task::new(
            "task-1".to_string(),
            "Task 1".to_string(),
            "Test task".to_string(),
            "test-agent".to_string(),
            json!({"input": "test"}),
        );
        task_repo.create(&task).unwrap();
    }

    // Create workflow
    {
        let mut db_lock = db.lock().unwrap();
        let mut workflow_repo = SqliteWorkflowRepository::new(&mut *db_lock);
        let mut workflow = Workflow::new(
            "workflow-1".to_string(),
            "State Test Workflow".to_string(),
            "Testing state persistence".to_string(),
        );
        workflow
            .add_step(WorkflowStep::new(
                "step-1".to_string(),
                "Step 1".to_string(),
                "Test step".to_string(),
                "task-1".to_string(),
                0,
            ))
            .unwrap();
        workflow_repo.create(&workflow).unwrap();
    }

    // Verify initial state
    {
        let mut db_lock = db.lock().unwrap();
        let workflow_repo = SqliteWorkflowRepository::new(&mut *db_lock);
        let workflow = workflow_repo.get_by_id("workflow-1").unwrap();
        assert_eq!(workflow.state, WorkflowState::Idle);
    }

    // Execute workflow
    let mut workflow = {
        let mut db_lock = db.lock().unwrap();
        let workflow_repo = SqliteWorkflowRepository::new(&mut *db_lock);
        workflow_repo.get_by_id("workflow-1").unwrap()
    };

    let _context = workflow_executor
        .execute_workflow(&mut workflow, Arc::clone(&db))
        .await
        .unwrap();

    // Verify final state persisted
    {
        let mut db_lock = db.lock().unwrap();
        let workflow_repo = SqliteWorkflowRepository::new(&mut *db_lock);
        let workflow = workflow_repo.get_by_id("workflow-1").unwrap();
        assert_eq!(workflow.state, WorkflowState::Completed);
    }
}
