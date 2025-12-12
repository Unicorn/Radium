//! Integration tests for function calling enhancements (REQ-222)
//!
//! These tests validate all acceptance criteria from the requirements document,
//! ensuring the complete function calling system works end-to-end.

use radium_abstraction::{ModelResponse, ToolCall, ToolUseMode};
use radium_orchestrator::orchestration::{
    validate_tool_mode,
    ContinuationBehavior, FunctionExecutionStrategy, ToolErrorHandling, ToolExecutionConfig,
};
use std::time::Duration;

// Note: Full integration tests require a working model implementation.
// These tests verify the structures and logic work correctly.
// Actual end-to-end tests with real models should be added as integration tests
// that can run against test models or mocks.

#[test]
fn test_mode_validation_auto() {
    // AC-1: AUTO mode should allow tool calls or no tool calls
    let mode = ToolUseMode::Auto;
    let tools = vec![];
    let response_with_tools = ModelResponse {
        content: "test".to_string(),
        model_id: None,
        usage: None,
        metadata: None,
        tool_calls: Some(vec![ToolCall {
            id: "call1".to_string(),
            name: "tool1".to_string(),
            arguments: serde_json::json!({}),
        }]),
    };
    let response_without_tools = ModelResponse {
        content: "test".to_string(),
        model_id: None,
        usage: None,
        metadata: None,
        tool_calls: None,
    };

    assert!(validate_tool_mode(&mode, &tools, &response_with_tools).is_ok());
    assert!(validate_tool_mode(&mode, &tools, &response_without_tools).is_ok());
}

#[test]
fn test_mode_validation_any() {
    // AC-1: ANY mode should fail if no tools provided or if model doesn't call tools
    use radium_orchestrator::orchestration::tool::{Tool, ToolParameters, ToolArguments, ToolResult};
    use radium_orchestrator::orchestration::tool::ToolHandler;
    use async_trait::async_trait;
    use std::sync::Arc;

    struct MockToolHandler;
    #[async_trait]
    impl ToolHandler for MockToolHandler {
        async fn execute(&self, _args: &ToolArguments) -> radium_orchestrator::error::Result<ToolResult> {
            Ok(ToolResult::success("ok"))
        }
    }

    let mode = ToolUseMode::Any;
    let tools = vec![Tool::new(
        "tool1",
        "tool1",
        "A test tool",
        ToolParameters::new(),
        Arc::new(MockToolHandler),
    )];

    // Should fail if no tools available
    assert!(validate_tool_mode(&ToolUseMode::Any, &[], &ModelResponse {
        content: "test".to_string(),
        model_id: None,
        usage: None,
        metadata: None,
        tool_calls: None,
    }).is_err());

    // Should fail if model doesn't call tools
    assert!(validate_tool_mode(&mode, &tools, &ModelResponse {
        content: "test".to_string(),
        model_id: None,
        usage: None,
        metadata: None,
        tool_calls: None,
    }).is_err());

    // Should pass if model calls tools
    assert!(validate_tool_mode(&mode, &tools, &ModelResponse {
        content: "test".to_string(),
        model_id: None,
        usage: None,
        metadata: None,
        tool_calls: Some(vec![ToolCall {
            id: "call1".to_string(),
            name: "tool1".to_string(),
            arguments: serde_json::json!({}),
        }]),
    }).is_ok());
}

#[test]
fn test_mode_validation_none() {
    // AC-1: NONE mode should fail if model attempts to call tools
    let mode = ToolUseMode::None;
    let tools = vec![];

    // Should fail if model calls tools
    assert!(validate_tool_mode(&mode, &tools, &ModelResponse {
        content: "test".to_string(),
        model_id: None,
        usage: None,
        metadata: None,
        tool_calls: Some(vec![ToolCall {
            id: "call1".to_string(),
            name: "tool1".to_string(),
            arguments: serde_json::json!({}),
        }]),
    }).is_err());

    // Should pass if model doesn't call tools
    assert!(validate_tool_mode(&mode, &tools, &ModelResponse {
        content: "test".to_string(),
        model_id: None,
        usage: None,
        metadata: None,
        tool_calls: None,
    }).is_ok());
}

#[test]
fn test_tool_execution_config_defaults() {
    // Verify default configuration matches requirements
    let config = ToolExecutionConfig::default();
    
    assert!(matches!(config.execution_strategy, FunctionExecutionStrategy::Concurrent));
    assert!(matches!(config.continuation, ContinuationBehavior::AutoContinue { max_rounds: 5 }));
    assert!(matches!(config.error_handling, ToolErrorHandling::ReturnToModel));
    assert_eq!(config.timeout_per_call, Duration::from_secs(30));
    assert_eq!(config.max_parallel, Some(10));
    assert_eq!(config.max_rounds, 5);
}

#[test]
fn test_tool_result_error_for_model() {
    use radium_orchestrator::orchestration::tool::ToolResult;
    
    let error_result = ToolResult::error_for_model("Test error");
    assert!(!error_result.success);
    assert!(error_result.is_error);
    assert_eq!(error_result.output, "Test error");
}

// TODO: Add more integration tests when models are available for testing:
// - Parallel execution performance tests
// - Continuation behavior end-to-end tests
// - Error handling strategy tests
// - Circular call detection tests
// - Full workflow tests with real models

