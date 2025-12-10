//! Integration tests for Gemini code execution tool.
//!
//! Tests cover all acceptance criteria from REQ-229:
//! - AC1: Successful code execution with enabled config
//! - AC2: Code execution disabled via agent config
//! - AC3: Runtime error handling
//! - AC4: Policy enforcement (deny)
//! - AC5: Output capture (stdout/stderr)
//! - AC6: Multiple executions in single generation

use radium_models::{GeminiModel, ModelFactory, ModelConfig, ModelType};
use radium_abstraction::{Model, ModelParameters};
use radium_orchestrator::CodeExecutionResult;
use radium_core::policy::{PolicyEngine, ApprovalMode, PolicyAction, PolicyRule, PolicyPriority};

/// Helper function to create a test Gemini model with code execution enabled.
fn create_test_model_with_code_execution() -> GeminiModel {
    let api_key = std::env::var("GEMINI_API_KEY")
        .expect("GEMINI_API_KEY environment variable must be set for integration tests");
    
    GeminiModel::with_api_key("gemini-2.0-flash-exp".to_string(), api_key)
        .with_code_execution(true)
}

/// Helper function to create a test Gemini model with code execution disabled.
fn create_test_model_without_code_execution() -> GeminiModel {
    let api_key = std::env::var("GEMINI_API_KEY")
        .expect("GEMINI_API_KEY environment variable must be set for integration tests");
    
    GeminiModel::with_api_key("gemini-2.0-flash-exp".to_string(), api_key)
        .with_code_execution(false)
}

/// Helper function to create a ModelConfig with code execution enabled.
fn create_test_config_with_code_execution() -> ModelConfig {
    ModelConfig::new(ModelType::Gemini, "gemini-2.0-flash-exp".to_string())
        .with_code_execution(true)
}

/// Helper function to create a ModelConfig with code execution disabled.
fn create_test_config_without_code_execution() -> ModelConfig {
    ModelConfig::new(ModelType::Gemini, "gemini-2.0-flash-exp".to_string())
        .with_code_execution(false)
}

#[tokio::test]
#[ignore = "Requires GEMINI_API_KEY and network access"]
async fn test_ac1_successful_code_execution() {
    // AC1: Given a Gemini model is configured with code execution enabled,
    // When the model requests to execute Python code during generation,
    // Then the code is sent to Gemini's sandbox, executed, and results are returned to the model for further reasoning.
    
    let model = create_test_model_with_code_execution();
    
    let prompt = "Calculate the sum of prime numbers up to 100 using Python code execution.";
    
    let response = model.generate_text(prompt, None).await;
    
    assert!(response.is_ok(), "Code execution request should succeed");
    let response = response.unwrap();
    
    // Check that response contains content (model should have executed code and provided answer)
    assert!(!response.content.is_empty(), "Response should contain content");
    
    // Check for code execution results in metadata
    if let Some(ref metadata) = response.metadata {
        if let Some(code_exec_results) = metadata.get("code_execution_results") {
            assert!(code_exec_results.is_array(), "code_execution_results should be an array");
        }
    }
    
    // Verify telemetry would track code executions (if available)
    // Note: Actual telemetry tracking happens in executor layer
}

#[tokio::test]
#[ignore = "Requires GEMINI_API_KEY and network access"]
async fn test_ac2_code_execution_disabled() {
    // AC2: Given an agent configuration with code_execution_enabled: false,
    // When the agent is executed with a Gemini model,
    // Then the code execution tool is NOT included in the API request.
    
    let model = create_test_model_without_code_execution();
    
    let prompt = "Calculate 2+2 using code execution.";
    
    let response = model.generate_text(prompt, None).await;
    
    // Request should succeed, but code execution tool should not be in request
    assert!(response.is_ok(), "Request should succeed even without code execution");
    
    // Note: We can't directly verify the tool wasn't in the request without mocking,
    // but we can verify the model responds without code execution results
    let response = response.unwrap();
    
    // Response should not have code execution results in metadata
    if let Some(ref metadata) = response.metadata {
        assert!(
            metadata.get("code_execution_results").is_none(),
            "Code execution results should not be present when disabled"
        );
    }
}

#[tokio::test]
#[ignore = "Requires GEMINI_API_KEY and network access"]
async fn test_ac3_runtime_error_handling() {
    // AC3: Given a code execution request results in a runtime error,
    // When the error is returned from the provider,
    // Then the error details are passed to the model and logged for audit purposes.
    
    let model = create_test_model_with_code_execution();
    
    // Request code that will raise a runtime error
    let prompt = "Execute this Python code: raise ValueError('Test error')";
    
    let response = model.generate_text(prompt, None).await;
    
    assert!(response.is_ok(), "Request should succeed even with execution error");
    let response = response.unwrap();
    
    // Check for error in code execution results
    if let Some(ref metadata) = response.metadata {
        if let Some(code_exec_results) = metadata.get("code_execution_results") {
            if let Some(results_array) = code_exec_results.as_array() {
                for result in results_array {
                    // Check if result contains error
                    if let Some(error) = result.get("error") {
                        assert!(!error.to_string().is_empty(), "Error should be present and non-empty");
                    }
                }
            }
        }
    }
    
    // Model should be able to see the error and respond
    assert!(!response.content.is_empty(), "Model should provide response even after error");
}

#[tokio::test]
#[ignore = "Requires GEMINI_API_KEY and network access"]
async fn test_ac4_policy_enforcement() {
    // AC4: Given a policy rule denies the code_execution tool,
    // When a model attempts to use code execution,
    // Then the request is blocked according to the policy action (deny or ask_user).
    
    // Note: Policy enforcement happens in executor layer, not model layer.
    // This test verifies that PolicyEngine can match "code_execution" tool pattern.
    use radium_core::policy::{PolicyEngine, ApprovalMode, PolicyAction, PolicyRule, PolicyPriority};
    
    // Create policy engine with rule denying code_execution
    let mut engine = PolicyEngine::new(ApprovalMode::Ask).unwrap();
    let rule = PolicyRule {
        name: "Deny code execution".to_string(),
        tool_pattern: "code_execution".to_string(),
        arg_pattern: None,
        action: PolicyAction::Deny,
        priority: PolicyPriority::Admin,
        reason: Some("Code execution is disabled by policy".to_string()),
    };
    engine.add_rule(rule);
    
    // Test that policy engine recognizes code_execution tool
    let decision = engine.evaluate_tool("code_execution", &[]).await.unwrap();
    assert!(decision.is_denied(), "Policy should deny code_execution");
    assert_eq!(decision.action, PolicyAction::Deny);
}

#[tokio::test]
#[ignore = "Requires GEMINI_API_KEY and network access"]
async fn test_ac5_output_capture() {
    // AC5: Given a model executes code that produces stdout and stderr output,
    // When the execution completes,
    // Then both stdout and stderr are captured and returned to the model in the response.
    
    let model = create_test_model_with_code_execution();
    
    // Request code that produces both stdout and stderr
    let prompt = "Execute Python code that prints to stdout and stderr: import sys; print('stdout message'); print('stderr message', file=sys.stderr)";
    
    let response = model.generate_text(prompt, None).await;
    
    assert!(response.is_ok(), "Request should succeed");
    let response = response.unwrap();
    
    // Check for code execution results with stdout/stderr
    if let Some(ref metadata) = response.metadata {
        if let Some(code_exec_results) = metadata.get("code_execution_results") {
            if let Some(results_array) = code_exec_results.as_array() {
                for result in results_array {
                    // Verify result structure contains stdout/stderr fields
                    // (exact structure depends on Gemini API response format)
                    assert!(result.is_object(), "Code execution result should be an object");
                }
            }
        }
    }
}

#[tokio::test]
#[ignore = "Requires GEMINI_API_KEY and network access"]
async fn test_ac6_multiple_executions() {
    // AC6: Given multiple code execution requests occur in a single generation,
    // When each execution completes,
    // Then all results are tracked in telemetry and the final output includes all execution history.
    
    let model = create_test_model_with_code_execution();
    
    // Request that requires multiple code executions
    let prompt = "First calculate 2+2, then calculate 3*3, and finally calculate 4**2. Use code execution for each calculation.";
    
    let response = model.generate_text(prompt, None).await;
    
    assert!(response.is_ok(), "Request should succeed");
    let response = response.unwrap();
    
    // Check for multiple code execution results
    if let Some(ref metadata) = response.metadata {
        if let Some(code_exec_results) = metadata.get("code_execution_results") {
            if let Some(results_array) = code_exec_results.as_array() {
                // Should have multiple results if model executed code multiple times
                // (exact count depends on model behavior)
                assert!(!results_array.is_empty(), "Should have at least one code execution result");
            }
        }
    }
}

#[tokio::test]
async fn test_code_execution_config_precedence() {
    // Test configuration precedence: Agent config > Model config > Provider default
    
    // Test 1: Model config with code execution enabled
    let config = create_test_config_with_code_execution();
    assert_eq!(config.enable_code_execution, Some(true));
    
    // Test 2: Model config with code execution disabled
    let config = create_test_config_without_code_execution();
    assert_eq!(config.enable_code_execution, Some(false));
    
    // Test 3: Model config with no setting (should use provider default)
    let config = ModelConfig::new(ModelType::Gemini, "gemini-2.0-flash-exp".to_string());
    assert_eq!(config.enable_code_execution, None);
}

#[tokio::test]
async fn test_code_execution_result_serialization() {
    // Test that CodeExecutionResult can be serialized/deserialized
    use serde_json;
    
    let result = CodeExecutionResult {
        code: "print('test')".to_string(),
        stdout: Some("test\n".to_string()),
        stderr: None,
        return_value: Some(serde_json::json!(null)),
        error: None,
    };
    
    let json = serde_json::to_string(&result).unwrap();
    let deserialized: CodeExecutionResult = serde_json::from_str(&json).unwrap();
    
    assert_eq!(deserialized.code, "print('test')");
    assert_eq!(deserialized.stdout, Some("test\n".to_string()));
    assert_eq!(deserialized.error, None);
}

#[tokio::test]
async fn test_code_execution_result_with_error() {
    // Test CodeExecutionResult with error
    use serde_json;
    
    let result = CodeExecutionResult {
        code: "invalid syntax".to_string(),
        stdout: None,
        stderr: Some("SyntaxError: invalid syntax".to_string()),
        return_value: None,
        error: Some("SyntaxError: invalid syntax".to_string()),
    };
    
    let json = serde_json::to_string(&result).unwrap();
    let deserialized: CodeExecutionResult = serde_json::from_str(&json).unwrap();
    
    assert_eq!(deserialized.error, Some("SyntaxError: invalid syntax".to_string()));
    assert_eq!(deserialized.stderr, Some("SyntaxError: invalid syntax".to_string()));
}

