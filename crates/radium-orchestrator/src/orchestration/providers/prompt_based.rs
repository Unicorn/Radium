// Prompt-based orchestration provider
//
// This provider uses prompt engineering to enable orchestration for models
// that don't have native function calling support. It instructs the model
// to output JSON-formatted tool calls.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Write;

use crate::error::{OrchestrationError, Result};
use crate::orchestration::{
    FinishReason, OrchestrationProvider, OrchestrationResult,
    context::OrchestrationContext,
    tool::{Tool, ToolCall},
};
use radium_abstraction::{ChatMessage, Model, ModelParameters};

/// Tool call request in JSON format
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ToolCallRequest {
    tool: String,
    arguments: Value,
}

/// Response format for prompt-based orchestration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum PromptResponse {
    ToolCalls { tool_calls: Vec<ToolCallRequest> },
    Text { response: String },
}

/// Prompt-based orchestration provider
pub struct PromptBasedOrchestrator {
    /// Underlying model to use
    model: Box<dyn Model>,
}

impl PromptBasedOrchestrator {
    /// Create a new prompt-based orchestrator
    pub fn new(model: Box<dyn Model>) -> Self {
        Self { model }
    }

    /// Build system prompt with tool definitions
    fn build_system_prompt(tools: &[Tool]) -> String {
        let mut prompt = String::from(
            "You are an intelligent assistant that can use tools to help users.\n\n\
            When you need to use a tool, respond ONLY with a JSON object in this exact format:\n\
            {\"tool_calls\": [{\"tool\": \"tool_name\", \"arguments\": {\"arg1\": \"value1\"}}]}\n\n\
            When you don't need to use any tools, respond normally with text.\n\n\
            Available tools:\n\n",
        );

        for tool in tools {
            let _ = write!(
                &mut prompt,
                "Tool: {}\nDescription: {}\nParameters: {}\n\n",
                tool.name,
                tool.description,
                serde_json::to_string_pretty(&tool.parameters).unwrap_or_default()
            );
        }

        prompt.push_str(
            "Remember:\n\
            - Use tools when you need to delegate tasks to specialists\n\
            - Respond with JSON tool_calls when using tools\n\
            - Respond with plain text otherwise\n\
            - Only use one tool call at a time for clarity",
        );

        prompt
    }

    /// Parse model response to extract tool calls or text
    fn parse_response(response: &str) -> PromptResponse {
        // Try to parse as JSON first
        if let Ok(parsed) = serde_json::from_str::<PromptResponse>(response) {
            return parsed;
        }

        // If not JSON, treat as text response
        PromptResponse::Text { response: response.to_string() }
    }

    /// Convert tool call requests to ToolCall format
    fn convert_tool_calls(requests: Vec<ToolCallRequest>) -> Vec<ToolCall> {
        requests
            .into_iter()
            .enumerate()
            .map(|(i, req)| ToolCall {
                id: format!("call_{}", i),
                name: req.tool,
                arguments: req.arguments,
            })
            .collect()
    }
}

#[async_trait]
impl OrchestrationProvider for PromptBasedOrchestrator {
    async fn execute_with_tools(
        &self,
        input: &str,
        tools: &[Tool],
        context: &OrchestrationContext,
    ) -> Result<OrchestrationResult> {
        // Build system prompt with tool definitions
        let system_prompt = Self::build_system_prompt(tools);

        // Build message history
        let mut messages = vec![ChatMessage { role: "system".to_string(), content: system_prompt.into() }];

        // Add conversation history
        for msg in &context.conversation_history {
            messages.push(ChatMessage { role: msg.role.clone(), content: msg.content.clone().into() });
        }

        // Add current user input
        messages.push(ChatMessage { role: "user".to_string(), content: input.to_string().into() });

        // Create parameters
        let parameters = ModelParameters {
            temperature: Some(context.user_preferences.temperature),
            ..Default::default()
        };

        // Call model
        let response = self
            .model
            .generate_chat_completion(&messages, Some(parameters))
            .await
            .map_err(|e| OrchestrationError::Other(format!("Model error: {}", e)))?;

        // Parse response
        match Self::parse_response(&response.content) {
            PromptResponse::ToolCalls { tool_calls } => {
                // Convert and return tool calls
                let converted_calls = Self::convert_tool_calls(tool_calls);
                Ok(OrchestrationResult::new(String::new(), converted_calls, FinishReason::Stop))
            }
            PromptResponse::Text { response: text } => {
                // Return text response
                Ok(OrchestrationResult::new(text, vec![], FinishReason::Stop))
            }
        }
    }

    fn supports_function_calling(&self) -> bool {
        false
    }

    fn provider_name(&self) -> &'static str {
        "prompt_based"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use radium_models::MockModel;

    #[test]
    fn test_build_system_prompt() {
        use crate::orchestration::tool::ToolParameters;
        use std::sync::Arc;

        struct DummyHandler;
        #[async_trait]
        impl crate::orchestration::tool::ToolHandler for DummyHandler {
            async fn execute(
                &self,
                _args: &crate::orchestration::tool::ToolArguments,
            ) -> Result<crate::orchestration::tool::ToolResult> {
                Ok(crate::orchestration::tool::ToolResult::success("test"))
            }
        }

        let tools = vec![Tool::new(
            "agent_1",
            "test_tool",
            "A test tool",
            ToolParameters::new().add_property("task", "string", "Task description", true),
            Arc::new(DummyHandler),
        )];

        let prompt = PromptBasedOrchestrator::build_system_prompt(&tools);
        assert!(prompt.contains("test_tool"));
        assert!(prompt.contains("A test tool"));
        assert!(prompt.contains("tool_calls"));
    }

    #[test]
    fn test_parse_response_tool_calls() {
        let json_response =
            r#"{"tool_calls": [{"tool": "test_tool", "arguments": {"task": "test"}}]}"#;
        let parsed = PromptBasedOrchestrator::parse_response(json_response);

        match parsed {
            PromptResponse::ToolCalls { tool_calls } => {
                assert_eq!(tool_calls.len(), 1);
                assert_eq!(tool_calls[0].tool, "test_tool");
            }
            PromptResponse::Text { .. } => panic!("Expected ToolCalls variant"),
        }
    }

    #[test]
    fn test_parse_response_text() {
        let text_response = "This is a plain text response";
        let parsed = PromptBasedOrchestrator::parse_response(text_response);

        match parsed {
            PromptResponse::Text { response } => {
                assert_eq!(response, text_response);
            }
            PromptResponse::ToolCalls { .. } => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn test_convert_tool_calls() {
        let requests = vec![ToolCallRequest {
            tool: "test_tool".to_string(),
            arguments: serde_json::json!({"task": "test"}),
        }];

        let calls = PromptBasedOrchestrator::convert_tool_calls(requests);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].id, "call_0");
        assert_eq!(calls[0].name, "test_tool");
    }

    #[test]
    fn test_supports_function_calling() {
        let model = Box::new(MockModel::new("test-model".to_string()));
        let orchestrator = PromptBasedOrchestrator::new(model);
        assert!(!orchestrator.supports_function_calling());
    }

    #[test]
    fn test_provider_name() {
        let model = Box::new(MockModel::new("test-model".to_string()));
        let orchestrator = PromptBasedOrchestrator::new(model);
        assert_eq!(orchestrator.provider_name(), "prompt_based");
    }
}
