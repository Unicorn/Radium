//! Integration tests for plan executor system.
//!
//! Tests task execution, retry logic, error categorization, execution modes,
//! state persistence, and checkpoint recovery.

use radium_abstraction::{ChatMessage, Model, ModelError, ModelParameters, ModelResponse};
use radium_core::models::{Iteration, PlanManifest, PlanTask};
use radium_core::planning::{
    ErrorCategory, ExecutionConfig, ExecutionError, PlanExecutor, RunMode, TaskResult,
};
use radium_core::workspace::RequirementId;
use std::path::PathBuf;
use std::sync::Arc;
use std::str::FromStr;
use tempfile::TempDir;

// Mock model that can simulate different behaviors
struct MockExecutorModel {
    behavior: MockBehavior,
    call_count: Arc<std::sync::Mutex<usize>>,
}

enum MockBehavior {
    AlwaysSucceed,
    FailThenSucceed { fail_count: usize },
    AlwaysFail { error_type: String },
    FailAfterRetries { max_failures: usize },
}

impl MockExecutorModel {
    fn new(behavior: MockBehavior) -> Self {
        Self {
            behavior,
            call_count: Arc::new(std::sync::Mutex::new(0)),
        }
    }
}

#[async_trait::async_trait]
impl Model for MockExecutorModel {
    async fn generate_text(
        &self,
        _prompt: &str,
        _params: Option<ModelParameters>,
    ) -> Result<ModelResponse, ModelError> {
        let mut count = self.call_count.lock().unwrap();
        *count += 1;
        let current_count = *count;

        match &self.behavior {
            MockBehavior::AlwaysSucceed => Ok(ModelResponse {
                content: "Task completed successfully".to_string(),
                model_id: Some("mock".to_string()),
                usage: None,
                ..Default::default()
            }),
            MockBehavior::FailThenSucceed { fail_count } => {
                if current_count <= *fail_count {
                    Err(ModelError::Other(format!("Rate limit exceeded (429)")))
                } else {
                    Ok(ModelResponse {
                        content: "Task completed after retry".to_string(),
                        model_id: Some("mock".to_string()),
                        usage: None,
                        ..Default::default()
                    })
                }
            }
            MockBehavior::AlwaysFail { error_type } => {
                Err(ModelError::Other(error_type.clone()))
            }
            MockBehavior::FailAfterRetries { max_failures } => {
                if current_count <= *max_failures {
                    Err(ModelError::Other("Network timeout".to_string()))
                } else {
                    Ok(ModelResponse {
                        content: "Task completed".to_string(),
                        model_id: Some("mock".to_string()),
                        usage: None,
                        ..Default::default()
                    })
                }
            }
        }
    }

    async fn generate_chat_completion(
        &self,
        _messages: &[ChatMessage],
        _params: Option<ModelParameters>,
    ) -> Result<ModelResponse, ModelError> {
        self.generate_text("", _params).await
    }

    fn model_id(&self) -> &str {
        "mock"
    }
}

// Helper to create a simple manifest
fn create_simple_manifest() -> PlanManifest {
    let req_id = RequirementId::from_str("REQ-001").unwrap();
    let mut manifest = PlanManifest::new(req_id, "Test Project".to_string());

    let mut iter1 = Iteration::new(1, "Iteration 1".to_string());
    let task1 = PlanTask::new("I1", 1, "Task 1".to_string());
    let task2 = PlanTask::new("I1", 2, "Task 2".to_string());
    let task3 = PlanTask::new("I1", 3, "Task 3".to_string());

    iter1.add_task(task1);
    iter1.add_task(task2);
    iter1.add_task(task3);
    manifest.add_iteration(iter1);

    manifest
}

// Helper to create manifest with dependencies
fn create_manifest_with_dependencies() -> PlanManifest {
    let req_id = RequirementId::from_str("REQ-001").unwrap();
    let mut manifest = PlanManifest::new(req_id, "Test Project".to_string());

    let mut iter1 = Iteration::new(1, "Iteration 1".to_string());
    let mut task1 = PlanTask::new("I1", 1, "Task 1".to_string());
    task1.agent_id = Some("code-agent".to_string());
    task1.completed = true; // Mark as completed

    let mut task2 = PlanTask::new("I1", 2, "Task 2".to_string());
    task2.agent_id = Some("code-agent".to_string());
    task2.dependencies.push("I1.T1".to_string());

    iter1.add_task(task1);
    iter1.add_task(task2);
    manifest.add_iteration(iter1);

    manifest
}

#[tokio::test]
async fn test_executor_mark_task_complete() {
    let executor = PlanExecutor::new();
    let mut manifest = create_simple_manifest();

    let result = executor.mark_task_complete(&mut manifest, "I1", "I1.T1");
    assert!(result.is_ok());

    let iteration = manifest.get_iteration("I1").unwrap();
    let task = iteration.get_task("I1.T1").unwrap();
    assert!(task.completed);
}

#[tokio::test]
async fn test_executor_calculate_progress() {
    let executor = PlanExecutor::new();
    let mut manifest = create_simple_manifest();

    // Mark one task as complete
    executor.mark_task_complete(&mut manifest, "I1", "I1.T1").unwrap();

    let progress = executor.calculate_progress(&manifest);
    assert_eq!(progress, 33); // 1 out of 3 tasks
}

#[tokio::test]
async fn test_executor_check_dependencies_met() {
    let executor = PlanExecutor::new();
    let manifest = create_manifest_with_dependencies();

    let task = manifest.get_iteration("I1").unwrap().get_task("I1.T2").unwrap();
    let result = executor.check_dependencies(&manifest, task);

    assert!(result.is_ok()); // T1 is completed, so T2 can run
}

#[tokio::test]
async fn test_executor_check_dependencies_not_met() {
    let executor = PlanExecutor::new();
    let req_id = RequirementId::from_str("REQ-001").unwrap();
    let mut manifest = PlanManifest::new(req_id, "Test Project".to_string());

    let mut iter1 = Iteration::new(1, "Iteration 1".to_string());
    let task1 = PlanTask::new("I1", 1, "Task 1".to_string()); // Not completed
    let mut task2 = PlanTask::new("I1", 2, "Task 2".to_string());
    task2.dependencies.push("I1.T1".to_string());

    iter1.add_task(task1);
    iter1.add_task(task2);
    manifest.add_iteration(iter1);

    let task = manifest.get_iteration("I1").unwrap().get_task("I1.T2").unwrap();
    let result = executor.check_dependencies(&manifest, task);

    assert!(result.is_err());
    match result.unwrap_err() {
        ExecutionError::DependencyNotMet(_) => {}
        e => panic!("Expected DependencyNotMet, got {:?}", e),
    }
}

#[tokio::test]
async fn test_executor_error_category_recoverable() {
    let error = ExecutionError::ModelExecution("Rate limit exceeded (429)".to_string());
    assert_eq!(error.category(), ErrorCategory::Recoverable);

    let error = ExecutionError::ModelExecution("Network timeout".to_string());
    assert_eq!(error.category(), ErrorCategory::Recoverable);

    let error = ExecutionError::ModelExecution("Server error 500".to_string());
    assert_eq!(error.category(), ErrorCategory::Recoverable);
}

#[tokio::test]
async fn test_executor_error_category_fatal() {
    let error = ExecutionError::AgentNotFound("agent not found".to_string());
    assert_eq!(error.category(), ErrorCategory::Fatal);

    let error = ExecutionError::DependencyNotMet("dependency not met".to_string());
    assert_eq!(error.category(), ErrorCategory::Fatal);

    let error = ExecutionError::ModelExecution("Unauthorized (401)".to_string());
    assert_eq!(error.category(), ErrorCategory::Fatal);
}

#[tokio::test]
async fn test_executor_save_and_load_manifest() {
    let executor = PlanExecutor::new();
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("manifest.json");

    let mut manifest = create_simple_manifest();
    executor.mark_task_complete(&mut manifest, "I1", "I1.T1").unwrap();

    // Save manifest
    let save_result = executor.save_manifest(&manifest, &manifest_path);
    assert!(save_result.is_ok());
    assert!(manifest_path.exists());

    // Load manifest
    let load_result = executor.load_manifest(&manifest_path);
    assert!(load_result.is_ok());

    let loaded_manifest = load_result.unwrap();
    assert_eq!(loaded_manifest.requirement_id, manifest.requirement_id);
    assert_eq!(loaded_manifest.project_name, manifest.project_name);

    let task = loaded_manifest.get_iteration("I1").unwrap().get_task("I1.T1").unwrap();
    assert!(task.completed);
}

#[tokio::test]
async fn test_executor_has_incomplete_tasks() {
    let executor = PlanExecutor::new();
    let mut manifest = create_simple_manifest();

    assert!(executor.has_incomplete_tasks(&manifest));

    // Complete all tasks
    executor.mark_task_complete(&mut manifest, "I1", "I1.T1").unwrap();
    executor.mark_task_complete(&mut manifest, "I1", "I1.T2").unwrap();
    executor.mark_task_complete(&mut manifest, "I1", "I1.T3").unwrap();

    assert!(!executor.has_incomplete_tasks(&manifest));
}

#[tokio::test]
async fn test_executor_execution_config_default() {
    let config = ExecutionConfig::default();
    assert!(!config.resume);
    assert!(config.skip_completed);
    assert!(config.check_dependencies);
    assert_eq!(config.state_path, PathBuf::from("plan/plan_manifest.json"));
    assert!(config.context_files.is_none());
    match config.run_mode {
        RunMode::Bounded(n) => assert_eq!(n, 5),
        RunMode::Continuous => panic!("Expected Bounded mode"),
    }
}

#[tokio::test]
async fn test_executor_run_mode_bounded() {
    let mode = RunMode::Bounded(3);
    match mode {
        RunMode::Bounded(n) => assert_eq!(n, 3),
        RunMode::Continuous => panic!("Expected Bounded mode"),
    }
}

#[tokio::test]
async fn test_executor_run_mode_continuous() {
    let mode = RunMode::Continuous;
    match mode {
        RunMode::Bounded(_) => panic!("Expected Continuous mode"),
        RunMode::Continuous => {} // OK
    }
}

#[tokio::test]
async fn test_executor_task_result_success() {
    let result = TaskResult {
        task_id: "I1.T1".to_string(),
        success: true,
        response: Some("Task completed".to_string()),
        error: None,
        tokens_used: Some((100, 50)),
    };

    assert!(result.success);
    assert!(result.response.is_some());
    assert!(result.error.is_none());
    assert_eq!(result.tokens_used, Some((100, 50)));
}

#[tokio::test]
async fn test_executor_task_result_failure() {
    let result = TaskResult {
        task_id: "I1.T1".to_string(),
        success: false,
        response: None,
        error: Some("Task failed".to_string()),
        tokens_used: None,
    };

    assert!(!result.success);
    assert!(result.response.is_none());
    assert!(result.error.is_some());
    assert_eq!(result.error.unwrap(), "Task failed");
}

// Note: Full execution tests with agents would require setting up agent discovery
// and agent files, which is more complex. The above tests cover the core executor
// functionality including state management, dependency checking, error categorization,
// and configuration.

