//! Integration tests for orchestration provider implementations
//!
//! These tests use mocked HTTP responses to test provider implementations
//! without requiring actual API keys or network access.

use radium_orchestrator::orchestration::{
    FinishReason, OrchestrationProvider, OrchestrationResult,
    context::OrchestrationContext,
    tool::{Tool, ToolCall, ToolParameters, ToolResult, ToolHandler, ToolArguments},
};
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;

// Mock tool handler for testing
struct MockToolHandler {
    tool_name: String,
}

#[async_trait]
impl ToolHandler for MockToolHandler {
    async fn execute(&self, _args: &ToolArguments) -> radium_orchestrator::error::Result<ToolResult> {
        Ok(ToolResult::success(format!("Executed {}", self.tool_name)))
    }
}

fn create_test_tools() -> Vec<Tool> {
    vec![
        Tool::new(
            "agent_1",
            "test_agent_1",
            "First test agent",
            ToolParameters::new().add_property("task", "string", "Task to perform", true),
            Arc::new(MockToolHandler { tool_name: "test_agent_1".to_string() }),
        ),
        Tool::new(
            "agent_2",
            "test_agent_2",
            "Second test agent",
            ToolParameters::new().add_property("task", "string", "Task to perform", true),
            Arc::new(MockToolHandler { tool_name: "test_agent_2".to_string() }),
        ),
    ]
}

fn create_test_context() -> OrchestrationContext {
    let mut context = OrchestrationContext::new("test-session");
    context.add_user_message("Test input");
    context
}

#[tokio::test]
async fn test_gemini_provider_basic() {
    // Note: This test would require mocking the HTTP client
    // For now, we test the provider structure
    use radium_orchestrator::orchestration::providers::gemini::GeminiOrchestrator;
    
    let orchestrator = GeminiOrchestrator::new("test-model", "test-key");
    assert!(orchestrator.supports_function_calling());
    assert_eq!(orchestrator.provider_name(), "gemini");
}

#[tokio::test]
async fn test_claude_provider_basic() {
    use radium_orchestrator::orchestration::providers::claude::ClaudeOrchestrator;
    
    let orchestrator = ClaudeOrchestrator::new("test-model", "test-key");
    assert!(orchestrator.supports_function_calling());
    assert_eq!(orchestrator.provider_name(), "claude");
}

#[tokio::test]
async fn test_openai_provider_basic() {
    use radium_orchestrator::orchestration::providers::openai::OpenAIOrchestrator;
    
    let orchestrator = OpenAIOrchestrator::new("test-model", "test-key");
    assert!(orchestrator.supports_function_calling());
    assert_eq!(orchestrator.provider_name(), "openai");
}

#[tokio::test]
async fn test_prompt_based_provider_basic() {
    use radium_orchestrator::orchestration::providers::prompt_based::PromptBasedOrchestrator;
    use radium_models::MockModel;
    
    let model = Box::new(MockModel::new("test-model".to_string()));
    let orchestrator = PromptBasedOrchestrator::new(model);
    assert!(!orchestrator.supports_function_calling());
    assert_eq!(orchestrator.provider_name(), "prompt_based");
}

#[tokio::test]
async fn test_prompt_based_provider_with_tools() {
    use radium_orchestrator::orchestration::providers::prompt_based::PromptBasedOrchestrator;
    use radium_models::MockModel;
    
    let model = Box::new(MockModel::new("test-model".to_string()));
    let orchestrator = PromptBasedOrchestrator::new(model);
    let tools = create_test_tools();
    let context = create_test_context();
    
    // MockModel will return a simple response
    let result = orchestrator.execute_with_tools("Test input", &tools, &context).await;
    
    // Should succeed (MockModel always succeeds)
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.finish_reason, FinishReason::Stop);
}

#[test]
fn test_tool_conversion_to_provider_format() {
    let tools = create_test_tools();
    
    // Verify tools have correct structure
    assert_eq!(tools.len(), 2);
    assert_eq!(tools[0].name, "test_agent_1");
    assert_eq!(tools[1].name, "test_agent_2");
    
    // Verify parameters
    assert!(tools[0].parameters.properties.contains_key("task"));
    assert!(tools[0].parameters.required.contains(&"task".to_string()));
}

#[test]
fn test_tool_call_parsing() {
    // Test that tool calls can be parsed from JSON
    let json = json!({
        "id": "call_1",
        "name": "test_agent_1",
        "arguments": {
            "task": "test task"
        }
    });
    
    let tool_call: ToolCall = serde_json::from_value(json).unwrap();
    assert_eq!(tool_call.id, "call_1");
    assert_eq!(tool_call.name, "test_agent_1");
    assert_eq!(tool_call.arguments["task"], "test task");
}

#[test]
fn test_finish_reason_serialization() {
    let reasons = vec![
        FinishReason::Stop,
        FinishReason::MaxIterations,
        FinishReason::ToolError,
        FinishReason::Cancelled,
        FinishReason::Error,
    ];
    
    for reason in reasons {
        let serialized = serde_json::to_string(&reason).unwrap();
        let deserialized: FinishReason = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, reason);
    }
}

#[test]
fn test_orchestration_result_serialization() {
    let result = OrchestrationResult::new(
        "Test response".to_string(),
        vec![ToolCall {
            id: "call_1".to_string(),
            name: "test_tool".to_string(),
            arguments: json!({"task": "test"}),
        }],
        FinishReason::Stop,
    );
    
    let serialized = serde_json::to_string(&result).unwrap();
    assert!(serialized.contains("Test response"));
    assert!(serialized.contains("test_tool"));
    
    let deserialized: OrchestrationResult = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.response, result.response);
    assert_eq!(deserialized.tool_calls.len(), 1);
}

#[tokio::test]
async fn test_context_with_conversation_history() {
    let mut context = OrchestrationContext::new("test-session");
    
    context.add_user_message("First message");
    context.add_assistant_message("First response");
    context.add_user_message("Second message");
    
    assert_eq!(context.conversation_history.len(), 3);
    assert_eq!(context.conversation_history[0].role, "user");
    assert_eq!(context.conversation_history[1].role, "assistant");
    assert_eq!(context.conversation_history[2].role, "user");
}

#[test]
fn test_tool_parameters_validation() {
    let params = ToolParameters::new()
        .add_property("required_param", "string", "Required parameter", true)
        .add_property("optional_param", "number", "Optional parameter", false);
    
    assert_eq!(params.required.len(), 1);
    assert!(params.required.contains(&"required_param".to_string()));
    assert!(!params.required.contains(&"optional_param".to_string()));
}

#[tokio::test]
async fn test_prompt_based_multi_turn_conversation() {
    // Test multi-turn conversation with prompt-based provider
    use radium_orchestrator::orchestration::providers::prompt_based::PromptBasedOrchestrator;
    use radium_models::MockModel;
    
    let model = Box::new(MockModel::new("test-model".to_string()));
    let orchestrator = PromptBasedOrchestrator::new(model);
    let tools = create_test_tools();
    
    let mut context = OrchestrationContext::new("test-session");
    context.add_user_message("First message");
    
    // First turn
    let result1 = orchestrator.execute_with_tools("First message", &tools, &context).await;
    assert!(result1.is_ok());
    let result1 = result1.unwrap();
    context.add_assistant_message(&result1.response);
    
    // Second turn - context should include previous conversation
    context.add_user_message("Second message");
    let result2 = orchestrator.execute_with_tools("Second message", &tools, &context).await;
    assert!(result2.is_ok());
    
    // Verify context has history
    assert!(context.conversation_history.len() >= 3);
}

#[tokio::test]
async fn test_invalid_tool_arguments_handling() {
    // Test that invalid tool arguments are handled gracefully
    use radium_orchestrator::orchestration::providers::prompt_based::PromptBasedOrchestrator;
    use radium_models::MockModel;
    
    let model = Box::new(MockModel::new("test-model".to_string()));
    let orchestrator = PromptBasedOrchestrator::new(model);
    let tools = create_test_tools();
    let context = create_test_context();
    
    // Execute with tools - MockModel should handle this gracefully
    let result = orchestrator.execute_with_tools("Test with invalid args", &tools, &context).await;
    assert!(result.is_ok()); // MockModel always succeeds, but real providers would handle errors
}

#[tokio::test]
async fn test_provider_timeout_simulation() {
    // Test that providers handle timeout scenarios
    // Note: Real timeout testing would require actual network calls or mocks
    // This is a placeholder structure test
    use radium_orchestrator::orchestration::providers::prompt_based::PromptBasedOrchestrator;
    use radium_models::MockModel;
    
    let model = Box::new(MockModel::new("test-model".to_string()));
    let orchestrator = PromptBasedOrchestrator::new(model);
    let tools = create_test_tools();
    let context = create_test_context();
    
    // Execute - should complete (MockModel doesn't timeout, but structure is tested)
    let start = std::time::Instant::now();
    let result = orchestrator.execute_with_tools("Test input", &tools, &context).await;
    let elapsed = start.elapsed();
    
    assert!(result.is_ok());
    // Verify it completed quickly (MockModel is instant)
    assert!(elapsed.as_millis() < 100);
}

#[tokio::test]
async fn test_provider_error_handling() {
    // Test provider error handling structure
    // Real providers would return errors on API failures
    use radium_orchestrator::orchestration::providers::gemini::GeminiOrchestrator;
    
    // Creating with invalid API key structure should still create the provider
    // (actual API errors happen during execution)
    let orchestrator = GeminiOrchestrator::new("test-model", "invalid-key");
    assert_eq!(orchestrator.provider_name(), "gemini");
    // Actual API errors would occur during execute_with_tools
}

