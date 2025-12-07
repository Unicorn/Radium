// Claude orchestration provider using tool use API
//
// This provider uses Claude's tool use API to enable
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

/// Claude tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClaudeTool {
    name: String,
    description: String,
    input_schema: Value,
}

/// Claude message content block
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ClaudeContentBlock {
    Text { text: String },
    ToolUse { id: String, name: String, input: Value },
    ToolResult { tool_use_id: String, content: String },
}

/// Claude message
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClaudeMessage {
    role: String,
    content: Vec<ClaudeContentBlock>,
}

/// Claude API request
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    messages: Vec<ClaudeMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ClaudeTool>>,
}

/// Claude API response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClaudeResponse {
    id: String,
    #[serde(rename = "type")]
    response_type: String,
    role: String,
    content: Vec<ClaudeContentBlock>,
    stop_reason: Option<String>,
}

/// Claude orchestration provider
pub struct ClaudeOrchestrator {
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
    /// Maximum output tokens
    max_tokens: u32,
}

impl ClaudeOrchestrator {
    /// Create a new Claude orchestrator
    pub fn new(model_id: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            model_id: model_id.into(),
            api_key: api_key.into(),
            client: Client::new(),
            base_url: "https://api.anthropic.com/v1".to_string(),
            temperature: 0.7,
            max_tokens: 4096,
        }
    }

    /// Set temperature
    #[must_use]
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    /// Set max tokens
    #[must_use]
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Convert tools to Claude format
    fn tools_to_claude(tools: &[Tool]) -> Vec<ClaudeTool> {
        tools
            .iter()
            .map(|tool| ClaudeTool {
                name: tool.name.clone(),
                description: tool.description.clone(),
                input_schema: serde_json::to_value(&tool.parameters).unwrap_or(Value::Null),
            })
            .collect()
    }

    /// Parse tool uses from Claude response
    fn parse_tool_uses(content: &[ClaudeContentBlock]) -> Vec<ToolCall> {
        content
            .iter()
            .filter_map(|block| {
                if let ClaudeContentBlock::ToolUse { id, name, input } = block {
                    Some(ToolCall { id: id.clone(), name: name.clone(), arguments: input.clone() })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Extract text from Claude content blocks
    fn extract_text(content: &[ClaudeContentBlock]) -> String {
        content
            .iter()
            .filter_map(|block| {
                if let ClaudeContentBlock::Text { text } = block {
                    Some(text.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Convert finish reason
    fn convert_finish_reason(reason: Option<&str>) -> FinishReason {
        match reason {
            Some("max_tokens") => FinishReason::MaxIterations,
            _ => FinishReason::Stop,
        }
    }

    /// Make API call to Claude
    async fn call_claude(&self, request: &ClaudeRequest) -> Result<ClaudeResponse> {
        let url = format!("{}/messages", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(request)
            .send()
            .await
            .map_err(|e| OrchestrationError::Other(format!("Network error: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(OrchestrationError::Other(format!("Claude API error: {}", error_text)));
        }

        let claude_response: ClaudeResponse = response
            .json()
            .await
            .map_err(|e| OrchestrationError::Other(format!("Failed to parse response: {}", e)))?;

        Ok(claude_response)
    }
}

#[async_trait]
impl OrchestrationProvider for ClaudeOrchestrator {
    async fn execute_with_tools(
        &self,
        input: &str,
        tools: &[Tool],
        context: &OrchestrationContext,
    ) -> Result<OrchestrationResult> {
        // Convert tools to Claude format
        let claude_tools = Self::tools_to_claude(tools);
        let tools_opt = if claude_tools.is_empty() { None } else { Some(claude_tools) };

        // Build message history
        let mut messages: Vec<ClaudeMessage> = Vec::new();

        // Add conversation history
        for msg in &context.conversation_history {
            // Group consecutive messages from same role
            if let Some(last_msg) = messages.last_mut() {
                if last_msg.role == msg.role {
                    // Append to existing message
                    last_msg.content.push(ClaudeContentBlock::Text { text: msg.content.clone() });
                    continue;
                }
            }

            messages.push(ClaudeMessage {
                role: msg.role.clone(),
                content: vec![ClaudeContentBlock::Text { text: msg.content.clone() }],
            });
        }

        // Add current user input
        messages.push(ClaudeMessage {
            role: "user".to_string(),
            content: vec![ClaudeContentBlock::Text { text: input.to_string() }],
        });

        // Build request
        let request = ClaudeRequest {
            model: self.model_id.clone(),
            max_tokens: self.max_tokens,
            temperature: Some(context.user_preferences.temperature),
            messages,
            tools: tools_opt,
        };

        // Call Claude
        let response = self.call_claude(&request).await?;

        // Parse tool uses
        let tool_calls = Self::parse_tool_uses(&response.content);

        // Extract text response
        let text_response = Self::extract_text(&response.content);

        // Check if we have tool uses
        if !tool_calls.is_empty() {
            // Return tool calls for execution
            return Ok(OrchestrationResult::new(text_response, tool_calls, FinishReason::Stop));
        }

        // No tool uses, return final response
        let finish_reason = Self::convert_finish_reason(response.stop_reason.as_deref());
        Ok(OrchestrationResult::new(text_response, vec![], finish_reason))
    }

    fn supports_function_calling(&self) -> bool {
        true
    }

    fn provider_name(&self) -> &'static str {
        "claude"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_orchestrator() {
        let orchestrator = ClaudeOrchestrator::new("claude-3-5-sonnet-20241022", "test-key");
        assert_eq!(orchestrator.model_id, "claude-3-5-sonnet-20241022");
        assert_eq!(orchestrator.api_key, "test-key");
        assert!((orchestrator.temperature - 0.7).abs() < f32::EPSILON);
        assert_eq!(orchestrator.max_tokens, 4096);
    }

    #[test]
    fn test_with_temperature() {
        let orchestrator =
            ClaudeOrchestrator::new("claude-3-5-sonnet-20241022", "test-key").with_temperature(0.9);
        assert!((orchestrator.temperature - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn test_with_max_tokens() {
        let orchestrator =
            ClaudeOrchestrator::new("claude-3-5-sonnet-20241022", "test-key").with_max_tokens(8192);
        assert_eq!(orchestrator.max_tokens, 8192);
    }

    #[test]
    fn test_supports_function_calling() {
        let orchestrator = ClaudeOrchestrator::new("claude-3-5-sonnet-20241022", "test-key");
        assert!(orchestrator.supports_function_calling());
    }

    #[test]
    fn test_provider_name() {
        let orchestrator = ClaudeOrchestrator::new("claude-3-5-sonnet-20241022", "test-key");
        assert_eq!(orchestrator.provider_name(), "claude");
    }

    #[test]
    fn test_parse_tool_uses() {
        let content = vec![
            ClaudeContentBlock::Text { text: "Calling tool".to_string() },
            ClaudeContentBlock::ToolUse {
                id: "toolu_1234".to_string(),
                name: "test_agent".to_string(),
                input: serde_json::json!({"task": "test task"}),
            },
        ];

        let calls = ClaudeOrchestrator::parse_tool_uses(&content);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].id, "toolu_1234");
        assert_eq!(calls[0].name, "test_agent");
    }

    #[test]
    fn test_extract_text() {
        let content = vec![
            ClaudeContentBlock::Text { text: "Part 1".to_string() },
            ClaudeContentBlock::Text { text: "Part 2".to_string() },
        ];

        let text = ClaudeOrchestrator::extract_text(&content);
        assert_eq!(text, "Part 1\nPart 2");
    }

    #[test]
    fn test_convert_finish_reason() {
        assert_eq!(ClaudeOrchestrator::convert_finish_reason(Some("end_turn")), FinishReason::Stop);
        assert_eq!(
            ClaudeOrchestrator::convert_finish_reason(Some("max_tokens")),
            FinishReason::MaxIterations
        );
        assert_eq!(ClaudeOrchestrator::convert_finish_reason(Some("tool_use")), FinishReason::Stop);
        assert_eq!(ClaudeOrchestrator::convert_finish_reason(None), FinishReason::Stop);
    }
}
