// OpenAI orchestration provider using function calling API
//
// This provider uses OpenAI's function calling API to enable
// intelligent tool routing and multi-turn agent orchestration.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{OrchestrationError, Result};
use crate::orchestration::{
    FinishReason, OrchestrationProvider, OrchestrationResult,
    context::OrchestrationContext,
    tool::{Tool, ToolCall},
};

/// OpenAI function definition
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIFunction {
    name: String,
    description: String,
    parameters: Value,
}

/// OpenAI tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAITool {
    #[serde(rename = "type")]
    tool_type: String,
    function: OpenAIFunction,
}

/// OpenAI function call
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIFunctionCall {
    name: String,
    arguments: String,
}

/// OpenAI tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIToolCall {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: OpenAIFunctionCall,
}

/// OpenAI message
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenAIToolCall>>,
}

/// OpenAI API request
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<OpenAITool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

/// OpenAI choice
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
    finish_reason: Option<String>,
}

/// OpenAI API response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIResponse {
    id: String,
    choices: Vec<OpenAIChoice>,
}

/// OpenAI orchestration provider
pub struct OpenAIOrchestrator {
    /// Model ID to use
    model_id: String,
    /// API key for authentication
    api_key: String,
    /// HTTP client
    client: Client,
    /// Base URL
    base_url: String,
    /// Temperature for generation
    temperature: f32,
}

impl OpenAIOrchestrator {
    /// Create a new OpenAI orchestrator
    pub fn new(model_id: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            model_id: model_id.into(),
            api_key: api_key.into(),
            client: Client::new(),
            base_url: "https://api.openai.com/v1".to_string(),
            temperature: 0.7,
        }
    }

    /// Set temperature
    #[must_use]
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    /// Convert tools to OpenAI format
    fn tools_to_openai(tools: &[Tool]) -> Vec<OpenAITool> {
        tools
            .iter()
            .map(|tool| OpenAITool {
                tool_type: "function".to_string(),
                function: OpenAIFunction {
                    name: tool.name.clone(),
                    description: tool.description.clone(),
                    parameters: serde_json::to_value(&tool.parameters).unwrap_or(Value::Null),
                },
            })
            .collect()
    }

    /// Parse tool calls from OpenAI response
    fn parse_tool_calls(tool_calls: &[OpenAIToolCall]) -> Result<Vec<ToolCall>> {
        tool_calls
            .iter()
            .map(|tc| {
                let arguments: Value =
                    serde_json::from_str(&tc.function.arguments).map_err(|e| {
                        OrchestrationError::Other(format!("Failed to parse tool arguments: {}", e))
                    })?;

                Ok(ToolCall { id: tc.id.clone(), name: tc.function.name.clone(), arguments })
            })
            .collect()
    }

    /// Convert finish reason
    fn convert_finish_reason(reason: Option<&str>) -> FinishReason {
        match reason {
            Some("length") => FinishReason::MaxIterations,
            Some("content_filter") => FinishReason::Error,
            _ => FinishReason::Stop,
        }
    }

    /// Make API call to OpenAI
    async fn call_openai(&self, request: &OpenAIRequest) -> Result<OpenAIResponse> {
        let url = format!("{}/chat/completions", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await
            .map_err(|e| OrchestrationError::Other(format!("Network error: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(OrchestrationError::Other(format!("OpenAI API error: {}", error_text)));
        }

        let openai_response: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| OrchestrationError::Other(format!("Failed to parse response: {}", e)))?;

        Ok(openai_response)
    }
}

#[async_trait]
impl OrchestrationProvider for OpenAIOrchestrator {
    async fn execute_with_tools(
        &self,
        input: &str,
        tools: &[Tool],
        context: &OrchestrationContext,
    ) -> Result<OrchestrationResult> {
        // Convert tools to OpenAI format
        let openai_tools = Self::tools_to_openai(tools);
        let tools_opt = if openai_tools.is_empty() { None } else { Some(openai_tools) };

        // Build message history
        let mut messages = Vec::new();

        // Add conversation history
        for msg in &context.conversation_history {
            messages.push(OpenAIMessage {
                role: msg.role.clone(),
                content: Some(msg.content.clone()),
                tool_calls: None,
            });
        }

        // Add current user input
        messages.push(OpenAIMessage {
            role: "user".to_string(),
            content: Some(input.to_string()),
            tool_calls: None,
        });

        // Build request
        let request = OpenAIRequest {
            model: self.model_id.clone(),
            messages,
            tools: tools_opt,
            temperature: Some(context.user_preferences.temperature),
        };

        // Call OpenAI
        let response = self.call_openai(&request).await?;

        // Get first choice
        let choice = response
            .choices
            .first()
            .ok_or_else(|| OrchestrationError::Other("No choices in response".to_string()))?;

        // Parse tool calls if present
        let tool_calls = if let Some(ref tc) = choice.message.tool_calls {
            Self::parse_tool_calls(tc)?
        } else {
            vec![]
        };

        // Extract text response
        let text_response = choice.message.content.clone().unwrap_or_default();

        // Check if we have tool calls
        if !tool_calls.is_empty() {
            // Return tool calls for execution
            return Ok(OrchestrationResult::new(text_response, tool_calls, FinishReason::Stop));
        }

        // No tool calls, return final response
        let finish_reason = Self::convert_finish_reason(choice.finish_reason.as_deref());
        Ok(OrchestrationResult::new(text_response, vec![], finish_reason))
    }

    fn supports_function_calling(&self) -> bool {
        true
    }

    fn provider_name(&self) -> &'static str {
        "openai"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_orchestrator() {
        let orchestrator = OpenAIOrchestrator::new("gpt-4-turbo-preview", "test-key");
        assert_eq!(orchestrator.model_id, "gpt-4-turbo-preview");
        assert_eq!(orchestrator.api_key, "test-key");
        assert!((orchestrator.temperature - 0.7).abs() < f32::EPSILON);
    }

    #[test]
    fn test_with_temperature() {
        let orchestrator =
            OpenAIOrchestrator::new("gpt-4-turbo-preview", "test-key").with_temperature(0.9);
        assert!((orchestrator.temperature - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn test_supports_function_calling() {
        let orchestrator = OpenAIOrchestrator::new("gpt-4-turbo-preview", "test-key");
        assert!(orchestrator.supports_function_calling());
    }

    #[test]
    fn test_provider_name() {
        let orchestrator = OpenAIOrchestrator::new("gpt-4-turbo-preview", "test-key");
        assert_eq!(orchestrator.provider_name(), "openai");
    }

    #[test]
    fn test_parse_tool_calls() {
        let tool_calls = vec![OpenAIToolCall {
            id: "call_abc123".to_string(),
            call_type: "function".to_string(),
            function: OpenAIFunctionCall {
                name: "test_agent".to_string(),
                arguments: r#"{"task": "test task"}"#.to_string(),
            },
        }];

        let calls = OpenAIOrchestrator::parse_tool_calls(&tool_calls).unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].id, "call_abc123");
        assert_eq!(calls[0].name, "test_agent");
    }

    #[test]
    fn test_convert_finish_reason() {
        assert_eq!(OpenAIOrchestrator::convert_finish_reason(Some("stop")), FinishReason::Stop);
        assert_eq!(
            OpenAIOrchestrator::convert_finish_reason(Some("length")),
            FinishReason::MaxIterations
        );
        assert_eq!(
            OpenAIOrchestrator::convert_finish_reason(Some("tool_calls")),
            FinishReason::Stop
        );
        assert_eq!(OpenAIOrchestrator::convert_finish_reason(None), FinishReason::Stop);
    }
}
