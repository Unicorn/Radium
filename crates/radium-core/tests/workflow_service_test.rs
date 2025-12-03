//! Tests for workflow service layer.

use radium_core::workflow::service::{WorkflowExecution, WorkflowService};
use radium_core::workflow::engine::ExecutionContext;
use radium_core::models::WorkflowState;
use radium_core::storage::Database;
use radium_orchestrator::{AgentExecutor, Orchestrator};
use radium_models::factory::ModelType;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_workflow_execution_new() {
    let context = ExecutionContext::new("test-execution".to_string());
    let execution = WorkflowExecution::new(
        "exec-1".to_string(),
        "workflow-1".to_string(),
        context.clone(),
        WorkflowState::Idle,
    );

    assert_eq!(execution.execution_id, "exec-1");
    assert_eq!(execution.workflow_id, "workflow-1");
    assert_eq!(execution.final_state, WorkflowState::Idle);
    assert_eq!(execution.started_at, context.started_at);
    assert_eq!(execution.completed_at, context.completed_at);
}

#[tokio::test]
async fn test_workflow_service_new() {
    let temp_dir = TempDir::new().unwrap();
    let db = Arc::new(std::sync::Mutex::new(
        Database::open(temp_dir.path().join("test.db").to_str().unwrap()).unwrap(),
    ));
    let orchestrator = Arc::new(tokio::sync::Mutex::new(Orchestrator::new()));
    let executor = Arc::new(AgentExecutor::new(ModelType::Mock, "test-model".to_string()));

    let service = WorkflowService::new(&orchestrator, &executor, &db);
    
    // Service should be created successfully
    assert!(true); // Just verify it doesn't panic
}

#[tokio::test]
async fn test_workflow_service_get_execution_history_empty() {
    let temp_dir = TempDir::new().unwrap();
    let db = Arc::new(std::sync::Mutex::new(
        Database::open(temp_dir.path().join("test.db").to_str().unwrap()).unwrap(),
    ));
    let orchestrator = Arc::new(tokio::sync::Mutex::new(Orchestrator::new()));
    let executor = Arc::new(AgentExecutor::new(ModelType::Mock, "test-model".to_string()));

    let service = WorkflowService::new(&orchestrator, &executor, &db);
    
    // Get history with no executions
    let history = service.get_execution_history(None).await;
    assert!(history.is_empty());
    
    // Get history filtered by workflow ID
    let history = service.get_execution_history(Some("workflow-1")).await;
    assert!(history.is_empty());
}

#[tokio::test]
async fn test_workflow_service_get_execution_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    let db = Arc::new(std::sync::Mutex::new(
        Database::open(temp_dir.path().join("test.db").to_str().unwrap()).unwrap(),
    ));
    let orchestrator = Arc::new(tokio::sync::Mutex::new(Orchestrator::new()));
    let executor = Arc::new(AgentExecutor::new(ModelType::Mock, "test-model".to_string()));

    let service = WorkflowService::new(&orchestrator, &executor, &db);
    
    // Get non-existent execution
    let execution = service.get_execution("nonexistent").await;
    assert!(execution.is_none());
}

#[tokio::test]
async fn test_workflow_service_execute_workflow_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let db = Arc::new(std::sync::Mutex::new(
        Database::open(temp_dir.path().join("test.db").to_str().unwrap()).unwrap(),
    ));
    let orchestrator = Arc::new(tokio::sync::Mutex::new(Orchestrator::new()));
    let executor = Arc::new(AgentExecutor::new(ModelType::Mock, "test-model".to_string()));

    let service = WorkflowService::new(&orchestrator, &executor, &db);
    
    // Try to execute non-existent workflow
    let result = service.execute_workflow("nonexistent", false).await;
    assert!(result.is_err());
    
    // Verify it's a validation error
    if let Err(e) = result {
        assert!(matches!(e, radium_core::workflow::engine::WorkflowEngineError::Validation(_)));
    }
}

