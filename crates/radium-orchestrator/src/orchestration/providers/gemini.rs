// Gemini orchestration provider using function calling
//
// This provider uses Gemini's function_declarations API to enable
// intelligent tool routing and multi-turn agent orchestration.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::error::{OrchestrationError, Result};
use crate::orchestration::{
    FinishReason, OrchestrationProvider, OrchestrationResult,
    context::OrchestrationContext,
    tool::{Tool, ToolCall},
};

fn build_http_client() -> Client {
    // In some sandboxed macOS environments, `Client::new()` can panic while probing
    // system proxy configuration. Prefer an explicit builder with no_proxy.
    Client::builder()
        .no_proxy()
        .build()
        .unwrap_or_else(|_| Client::new())
}

/// Categorize tools by type for better discoverability
fn categorize_tools(tools: &[Tool]) -> Vec<(&'static str, Vec<&Tool>)> {
    let mut categories: HashMap<&'static str, Vec<&Tool>> = HashMap::new();
    for tool in tools {
        let category = match tool.name.as_str() {
            "read_file" | "write_file" | "search_replace" | "list_dir" | "glob_file_search" | "read_lints" => "File Operations",
            "project_scan" | "code_analysis" => "Code Analysis",
            "grep" | "search_files" => "Search",
            "git_log" | "git_diff" | "git_blame" | "git_show" | "find_references" => "Git",
            "run_terminal_cmd" => "Command Execution",
            _ => "Other",
        };
        categories.entry(category).or_default().push(tool);
    }
    categories.into_iter().collect()
}

/// Build orchestration system prompt to guide tool usage
fn build_orchestration_system_prompt(tools: &[Tool]) -> String {
    let mut prompt = String::from(
        "You are an autonomous developer assistant with native tool calling capabilities.\n\n\
        ## CRITICAL BEHAVIORAL RULES\n\n\
        1. **USE TOOLS IMMEDIATELY** - Never ask users for information you can find yourself\n\
        2. **CALL MULTIPLE TOOLS IN PARALLEL** when they don't depend on each other\n\
        3. **READ BEFORE WRITING** - Always read files before suggesting modifications\n\
        4. **BE PROACTIVE** - If a task requires multiple steps, call all necessary tools\n\
        5. **PROVIDE EVIDENCE** - Reference specific files and line numbers (format: `file.rs:123-145`)\n\n\
        ## Tool Calling Strategy\n\n\
        - Exploring code: project_scan → search_files/grep → read_file\n\
        - Making changes: read_file → write_file → run_command (to test)\n\
        - Analyzing: multiple grep/search_files calls in parallel\n\
        - Answering questions: Find files FIRST, then answer with references\n\n\
        ## Available Tools\n\n"
    );

    // Categorize tools for better discoverability
    let categories = categorize_tools(tools);
    for (category, tools_in_cat) in categories {
        prompt.push_str(&format!("### {}\n\n", category));
        for tool in tools_in_cat {
            prompt.push_str(&format!("**{}**: {}\n", tool.name, tool.description));
        }
        prompt.push('\n');
    }

    prompt.push_str(
        "## Response Guidelines\n\n\
        - Start with ACTION, not explanation\n\
        - Provide file:line references\n\
        - Chain tools logically\n\
        - Never ask users for information tools can provide\n"
    );

    prompt
}

/// Gemini function declaration (tool definition)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiFunctionDeclaration {
    name: String,
    description: String,
    parameters: Value,
}

/// Gemini tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiTools {
    function_declarations: Vec<GeminiFunctionDeclaration>,
}

/// Gemini content part
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum GeminiPart {
    Text {
        text: String
    },
    FunctionCall {
        #[serde(rename = "functionCall")]
        function_call: GeminiFunctionCall
    },
    FunctionResponse {
        #[serde(rename = "functionResponse")]
        function_response: GeminiFunctionResponse
    },
}

/// Gemini function call
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiFunctionCall {
    name: String,
    args: Value,
}

/// Gemini function response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiFunctionResponse {
    name: String,
    response: Value,
}

/// Gemini content
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

/// Gemini generation config
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
}

/// Gemini API request
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<GeminiTools>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GeminiGenerationConfig>,
}

/// Gemini candidate
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
    finish_reason: Option<String>,
}

/// Gemini API response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

/// Gemini orchestration provider
pub struct GeminiOrchestrator {
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
    /// Maximum tool iterations
    max_iterations: u32,
}

impl GeminiOrchestrator {
    /// Create a new Gemini orchestrator
    pub fn new(model_id: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            model_id: model_id.into(),
            api_key: api_key.into(),
            client: build_http_client(),
            base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            temperature: 0.7,
            max_iterations: 5,
        }
    }

    /// Set temperature
    #[must_use]
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    /// Set max iterations
    #[must_use]
    pub fn with_max_iterations(mut self, max_iterations: u32) -> Self {
        self.max_iterations = max_iterations;
        self
    }

    /// Convert tools to Gemini function declarations
    fn tools_to_gemini(tools: &[Tool]) -> Vec<GeminiFunctionDeclaration> {
        tools
            .iter()
            .map(|tool| GeminiFunctionDeclaration {
                name: tool.name.clone(),
                description: tool.description.clone(),
                parameters: serde_json::to_value(&tool.parameters).unwrap_or(Value::Null),
            })
            .collect()
    }

    /// Parse function calls from Gemini response
    fn parse_function_calls(content: &GeminiContent) -> Vec<ToolCall> {
        let mut tool_calls = Vec::new();

        for (i, part) in content.parts.iter().enumerate() {
            if let GeminiPart::FunctionCall { function_call } = part {
                tool_calls.push(ToolCall {
                    id: format!("call_{}", i),
                    name: function_call.name.clone(),
                    arguments: function_call.args.clone(),
                });
            }
        }

        tool_calls
    }

    /// Extract text response from Gemini content
    fn extract_text(content: &GeminiContent) -> String {
        content
            .parts
            .iter()
            .filter_map(|part| {
                if let GeminiPart::Text { text } = part { Some(text.clone()) } else { None }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Convert finish reason
    fn convert_finish_reason(reason: Option<&str>) -> FinishReason {
        match reason {
            Some("MAX_TOKENS") => FinishReason::MaxIterations,
            Some("SAFETY" | "RECITATION") => FinishReason::Error,
            _ => FinishReason::Stop,
        }
    }

    /// Make API call to Gemini
    async fn call_gemini(&self, request: &GeminiRequest) -> Result<GeminiResponse> {
        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.base_url, self.model_id, self.api_key
        );

        let response = self
            .client
            .post(&url)
            .json(request)
            .send()
            .await
            .map_err(|e| OrchestrationError::Other(format!("Network error: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(OrchestrationError::Other(format!("Gemini API error: {}", error_text)));
        }

        // Get response text for debugging
        let response_text = response.text().await
            .map_err(|e| OrchestrationError::Other(format!("Failed to read response: {}", e)))?;

        // Log the raw response for debugging (only shows with RUST_LOG=debug)
        tracing::debug!("Gemini API response: {}", response_text);

        let gemini_response: GeminiResponse = serde_json::from_str(&response_text)
            .map_err(|e| OrchestrationError::Other(format!("Failed to parse response: {}. Response was: {}", e, response_text)))?;

        Ok(gemini_response)
    }
}

#[async_trait]
impl OrchestrationProvider for GeminiOrchestrator {
    async fn execute_with_tools(
        &self,
        input: &str,
        tools: &[Tool],
        context: &OrchestrationContext,
    ) -> Result<OrchestrationResult> {
        // Convert tools to Gemini format
        let function_declarations = Self::tools_to_gemini(tools);
        let gemini_tools = if function_declarations.is_empty() {
            None
        } else {
            Some(vec![GeminiTools { function_declarations }])
        };

        // Build initial message history
        let mut contents = Vec::new();

        // Add conversation history
        for msg in &context.conversation_history {
            let role = if msg.role == "assistant" { "model" } else { "user" };
            contents.push(GeminiContent {
                role: role.to_string(),
                parts: vec![GeminiPart::Text { text: msg.content.clone() }],
            });
        }

        // Add system prompt if tools are available
        // (Gemini doesn't have native system role, use first user message)
        if !tools.is_empty() {
            let system_prompt = build_orchestration_system_prompt(tools);
            contents.insert(0, GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart::Text {
                    text: format!("SYSTEM INSTRUCTIONS:\n{}\n---\n", system_prompt),
                }],
            });
        }

        // Add current user input
        contents.push(GeminiContent {
            role: "user".to_string(),
            parts: vec![GeminiPart::Text { text: input.to_string() }],
        });

        // Build request
        let request = GeminiRequest {
            contents: contents.clone(),
            tools: gemini_tools.clone(),
            generation_config: Some(GeminiGenerationConfig {
                temperature: Some(context.user_preferences.temperature),
                top_p: None,
                max_output_tokens: None,
            }),
        };

        // Call Gemini
        let response = self.call_gemini(&request).await?;

        // Get first candidate
        let candidate = response
            .candidates
            .first()
            .ok_or_else(|| OrchestrationError::Other("No candidates in response".to_string()))?;

        // Parse function calls
        let tool_calls = Self::parse_function_calls(&candidate.content);

        // Extract text response
        let text_response = Self::extract_text(&candidate.content);

        // Check if we have function calls
        if !tool_calls.is_empty() {
            // Return tool calls for execution
            return Ok(OrchestrationResult::new(text_response, tool_calls, FinishReason::Stop));
        }

        // No function calls, return final response
        let finish_reason = Self::convert_finish_reason(candidate.finish_reason.as_deref());
        Ok(OrchestrationResult::new(text_response, vec![], finish_reason))
    }

    fn supports_function_calling(&self) -> bool {
        true
    }

    fn provider_name(&self) -> &'static str {
        "gemini"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_orchestrator() {
        let orchestrator = GeminiOrchestrator::new("gemini-2.0-flash-exp", "test-key");
        assert_eq!(orchestrator.model_id, "gemini-2.0-flash-exp");
        assert_eq!(orchestrator.api_key, "test-key");
        assert!((orchestrator.temperature - 0.7).abs() < f32::EPSILON);
        assert_eq!(orchestrator.max_iterations, 5);
    }

    #[test]
    fn test_with_temperature() {
        let orchestrator =
            GeminiOrchestrator::new("gemini-2.0-flash-exp", "test-key").with_temperature(0.9);
        assert!((orchestrator.temperature - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn test_with_max_iterations() {
        let orchestrator =
            GeminiOrchestrator::new("gemini-2.0-flash-exp", "test-key").with_max_iterations(10);
        assert_eq!(orchestrator.max_iterations, 10);
    }

    #[test]
    fn test_supports_function_calling() {
        let orchestrator = GeminiOrchestrator::new("gemini-2.0-flash-exp", "test-key");
        assert!(orchestrator.supports_function_calling());
    }

    #[test]
    fn test_provider_name() {
        let orchestrator = GeminiOrchestrator::new("gemini-2.0-flash-exp", "test-key");
        assert_eq!(orchestrator.provider_name(), "gemini");
    }

    #[test]
    fn test_parse_function_calls() {
        let content = GeminiContent {
            role: "model".to_string(),
            parts: vec![
                GeminiPart::Text { text: "Calling tool".to_string() },
                GeminiPart::FunctionCall {
                    function_call: GeminiFunctionCall {
                        name: "test_agent".to_string(),
                        args: serde_json::json!({"task": "test task"}),
                    },
                },
            ],
        };

        let calls = GeminiOrchestrator::parse_function_calls(&content);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "test_agent");
    }

    #[test]
    fn test_extract_text() {
        let content = GeminiContent {
            role: "model".to_string(),
            parts: vec![
                GeminiPart::Text { text: "Part 1".to_string() },
                GeminiPart::Text { text: "Part 2".to_string() },
            ],
        };

        let text = GeminiOrchestrator::extract_text(&content);
        assert_eq!(text, "Part 1\nPart 2");
    }

    #[test]
    fn test_convert_finish_reason() {
        assert_eq!(GeminiOrchestrator::convert_finish_reason(Some("STOP")), FinishReason::Stop);
        assert_eq!(
            GeminiOrchestrator::convert_finish_reason(Some("MAX_TOKENS")),
            FinishReason::MaxIterations
        );
        assert_eq!(GeminiOrchestrator::convert_finish_reason(Some("SAFETY")), FinishReason::Error);
        assert_eq!(GeminiOrchestrator::convert_finish_reason(None), FinishReason::Stop);
    }
}
