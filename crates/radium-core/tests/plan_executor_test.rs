//! Integration tests for plan execution with error handling and retry logic.

use radium_abstraction::{ChatMessage, Model, ModelError, ModelParameters, ModelResponse};
use radium_core::planning::{ErrorCategory, ExecutionError, PlanExecutor, TaskResult};
use radium_core::models::PlanTask;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::time::{Duration, Instant};

// Mock model that simulates recoverable errors (rate limit, network, timeout)
struct RecoverableErrorModel {
    attempts: Arc<AtomicU32>,
    max_failures: u32,
    error_message: String,
}

impl RecoverableErrorModel {
    fn new(max_failures: u32, error_message: impl Into<String>) -> Self {
        Self {
            attempts: Arc::new(AtomicU32::new(0)),
            max_failures,
            error_message: error_message.into(),
        }
    }
}

#[async_trait::async_trait]
impl Model for RecoverableErrorModel {
    async fn generate_text(
        &self,
        _prompt: &str,
        _params: Option<ModelParameters>,
    ) -> Result<ModelResponse, ModelError> {
        let attempt = self.attempts.fetch_add(1, Ordering::SeqCst);
        
        if attempt < self.max_failures {
            // Simulate recoverable error
            Err(ModelError::RequestError(format!("{}: {}", self.error_message, attempt)))
        } else {
            // Succeed after max_failures attempts
            Ok(ModelResponse {
                content: "Task completed successfully".to_string(),
                model_id: Some("mock".to_string()),
                usage: None,
            })
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

// Mock model that simulates fatal errors (auth, config)
struct FatalErrorModel {
    error_message: String,
}

impl FatalErrorModel {
    fn new(error_message: impl Into<String>) -> Self {
        Self {
            error_message: error_message.into(),
        }
    }
}

#[async_trait::async_trait]
impl Model for FatalErrorModel {
    async fn generate_text(
        &self,
        _prompt: &str,
        _params: Option<ModelParameters>,
    ) -> Result<ModelResponse, ModelError> {
        // Always fail with fatal error
        Err(ModelError::RequestError(self.error_message.clone()))
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

// Helper to create a test task
fn create_test_task() -> PlanTask {
    PlanTask::new("I1", 1, "Test Task".to_string())
}

#[tokio::test]
async fn test_retry_with_recoverable_error() {
    let executor = PlanExecutor::new();
    let task = create_test_task();
    
    // Model fails twice with rate limit, then succeeds
    let model = Arc::new(RecoverableErrorModel::new(2, "429 rate limit exceeded"));
    
    let start = Instant::now();
    let result = executor.execute_task_with_retry(&task, model, 3, 10).await; // 10ms base delay
    
    assert!(result.is_ok());
    let task_result = result.unwrap();
    assert!(task_result.success);
    
    // Verify exponential backoff was used (should take at least 10ms + 20ms = 30ms)
    let elapsed = start.elapsed();
    assert!(elapsed >= Duration::from_millis(30), "Should have delayed for retries");
}

#[tokio::test]
async fn test_retry_with_network_error() {
    let executor = PlanExecutor::new();
    let task = create_test_task();
    
    // Model fails once with network error, then succeeds
    let model = Arc::new(RecoverableErrorModel::new(1, "network connection failed"));
    
    let result = executor.execute_task_with_retry(&task, model, 3, 10).await;
    
    assert!(result.is_ok());
    let task_result = result.unwrap();
    assert!(task_result.success);
}

#[tokio::test]
async fn test_retry_exhausted() {
    let executor = PlanExecutor::new();
    let task = create_test_task();
    
    // Model always fails with recoverable error
    let model = Arc::new(RecoverableErrorModel::new(10, "429 rate limit exceeded")); // More failures than retries
    
    let result = executor.execute_task_with_retry(&task, model, 2, 10).await; // Only 2 retries
    
    // Should fail after retries are exhausted
    assert!(result.is_err());
    match result.unwrap_err() {
        ExecutionError::ModelExecution(_) => {} // Expected
        e => panic!("Expected ModelExecution error, got {:?}", e),
    }
}

#[tokio::test]
async fn test_fatal_error_no_retry() {
    let executor = PlanExecutor::new();
    let task = create_test_task();
    
    // Model always fails with fatal error
    let model = Arc::new(FatalErrorModel::new("401 unauthorized"));
    
    let start = Instant::now();
    let result = executor.execute_task_with_retry(&task, model, 3, 10).await;
    
    // Should fail immediately without retries
    assert!(result.is_err());
    
    // Should fail quickly (no retry delays)
    let elapsed = start.elapsed();
    assert!(elapsed < Duration::from_millis(50), "Should fail immediately without retries");
    
    match result.unwrap_err() {
        ExecutionError::ModelExecution(msg) => {
            assert!(msg.to_lowercase().contains("unauthorized"));
        }
        e => panic!("Expected ModelExecution error with unauthorized, got {:?}", e),
    }
}

#[tokio::test]
async fn test_fatal_error_missing_config() {
    let executor = PlanExecutor::new();
    let task = create_test_task();
    
    // Model fails with missing config error
    let model = Arc::new(FatalErrorModel::new("missing config file"));
    
    let result = executor.execute_task_with_retry(&task, model, 3, 10).await;
    
    // Should fail immediately without retries
    assert!(result.is_err());
    
    // Verify error is categorized as fatal
    match &result.unwrap_err() {
        ExecutionError::ModelExecution(msg) => {
            let error = ExecutionError::ModelExecution(msg.clone());
            assert_eq!(error.category(), ErrorCategory::Fatal);
        }
        e => panic!("Expected ModelExecution error, got {:?}", e),
    }
}

#[tokio::test]
async fn test_exponential_backoff_timing() {
    let executor = PlanExecutor::new();
    let task = create_test_task();
    
    // Model fails 3 times, then succeeds
    let model = Arc::new(RecoverableErrorModel::new(3, "timeout"));
    let base_delay_ms = 50;
    
    let start = Instant::now();
    let result = executor.execute_task_with_retry(&task, model, 5, base_delay_ms).await;
    
    assert!(result.is_ok());
    
    // Verify exponential backoff: 50ms + 100ms + 200ms = 350ms minimum
    let elapsed = start.elapsed();
    let expected_min = Duration::from_millis(350);
    assert!(
        elapsed >= expected_min,
        "Elapsed time {} should be at least {} (exponential backoff)",
        elapsed.as_millis(),
        expected_min.as_millis()
    );
}

#[tokio::test]
async fn test_error_category_recoverable_patterns() {
    let recoverable_patterns = vec![
        "429 rate limit",
        "timeout error",
        "network failure",
        "500 server error",
        "connection lost",
    ];

    for pattern in recoverable_patterns {
        let error = ExecutionError::ModelExecution(pattern.to_string());
        assert_eq!(
            error.category(),
            ErrorCategory::Recoverable,
            "Pattern '{}' should be recoverable",
            pattern
        );
    }
}

#[tokio::test]
async fn test_error_category_fatal_patterns() {
    let fatal_patterns = vec![
        "401 unauthorized",
        "403 forbidden",
        "missing config",
        "invalid input",
        "dependency not met",
    ];

    for pattern in fatal_patterns {
        let error = ExecutionError::ModelExecution(pattern.to_string());
        assert_eq!(
            error.category(),
            ErrorCategory::Fatal,
            "Pattern '{}' should be fatal",
            pattern
        );
    }
}

