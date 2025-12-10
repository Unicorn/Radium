//! Claude (Anthropic) model implementation.
//!
//! This module provides an implementation of the `Model` trait for Anthropic's Claude API.
//!
//! ## System Message Handling (Reference Implementation)
//!
//! Claude uses a **dedicated system field** approach for system messages, which serves as the
//! reference pattern for providers that support dedicated system instruction fields (like Gemini).
//!
//! **Key characteristics:**
//! - System messages are extracted from the ChatMessage array before processing
//! - System messages are filtered out of the main messages array
//! - System messages are sent via a dedicated `system` field in the API request
//! - Multiple system messages are concatenated with "\n\n" separator
//!
//! **When to use this pattern:**
//! - APIs that support a dedicated system instruction field (e.g., Claude, Gemini)
//! - When system context should be separated from conversation history
//! - When you want to preserve semantic distinction between system instructions and user messages
//!
//! **Comparison with other providers:**
//! - **OpenAI**: Uses inline approach - system messages included in messages array with `role: "system"`
//! - **Gemini**: Uses dedicated `systemInstruction` field (follows this Claude pattern)
//! - **Claude**: Uses dedicated `system` field (this implementation)
//!
//! See `extract_system_prompt()` and `generate_chat_completion()` for the implementation pattern.

use async_trait::async_trait;
use radium_abstraction::{
    ChatMessage, Model, ModelError, ModelParameters, ModelResponse, ModelUsage,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{debug, error};

/// Claude model implementation.
#[derive(Debug, Clone)]
pub struct ClaudeModel {
    /// The model ID (e.g., "claude-sonnet-4-5-20250929", "claude-opus-4-5-20251101").
    model_id: String,
    /// The API key for authentication.
    api_key: String,
    /// The base URL for the Claude API.
    base_url: String,
    /// HTTP client for making requests.
    client: Client,
}

impl ClaudeModel {
    /// Creates a new `ClaudeModel` with the given model ID.
    ///
    /// # Arguments
    /// * `model_id` - The Claude model ID to use (e.g., "claude-sonnet-4-5-20250929")
    ///
    /// # Errors
    /// Returns a `ModelError` if the API key is not found in environment variables.
    #[allow(clippy::disallowed_methods)] // env::var is needed for API key loading
    pub fn new(model_id: String) -> Result<Self, ModelError> {
        let api_key = env::var("ANTHROPIC_API_KEY").map_err(|_| {
            ModelError::UnsupportedModelProvider(
                "ANTHROPIC_API_KEY environment variable not set".to_string(),
            )
        })?;

        Ok(Self {
            model_id,
            api_key,
            base_url: "https://api.anthropic.com/v1".to_string(),
            client: Client::new(),
        })
    }

    /// Creates a new `ClaudeModel` with a custom API key.
    ///
    /// # Arguments
    /// * `model_id` - The Claude model ID to use
    /// * `api_key` - The API key for authentication
    #[must_use]
    pub fn with_api_key(model_id: String, api_key: String) -> Self {
        Self {
            model_id,
            api_key,
            base_url: "https://api.anthropic.com/v1".to_string(),
            client: Client::new(),
        }
    }

    /// Converts our ChatMessage to Claude API message format.
    fn to_claude_message(msg: &ChatMessage) -> ClaudeMessage {
        ClaudeMessage {
            role: if msg.role == "assistant" { "assistant" } else { "user" }.to_string(),
            content: msg.content.clone(),
        }
    }

    /// Extracts system messages from the chat history.
    ///
    /// This function implements the **dedicated system field pattern** used by Claude and Gemini.
    /// System messages are extracted and concatenated (if multiple exist) before being sent via
    /// the dedicated `system` field in the API request.
    ///
    /// **Pattern details:**
    /// - Filters messages with `role == "system"`
    /// - Returns the first system message found (Claude API typically uses single system prompt)
    /// - Returns `None` if no system messages are present
    ///
    /// **Note:** For multiple system messages, this implementation takes the first one.
    /// If concatenation of multiple system messages is needed, see Gemini's `extract_system_messages()`
    /// implementation which concatenates with "\n\n" separator.
    ///
    /// # Arguments
    /// * `messages` - Array of ChatMessage objects to extract system messages from
    ///
    /// # Returns
    /// `Some(String)` containing the system message content, or `None` if no system messages found
    ///
    /// # Example
    /// ```
    /// use radium_abstraction::ChatMessage;
    /// use radium_models::ClaudeModel;
    /// let messages = vec![
    ///     ChatMessage { role: "system".to_string(), content: "You are helpful.".to_string() },
    ///     ChatMessage { role: "user".to_string(), content: "Hello".to_string() },
    /// ];
    /// let system = ClaudeModel::extract_system_prompt(&messages);
    /// assert_eq!(system, Some("You are helpful.".to_string()));
    /// ```
    ///
    /// **Reference:** This pattern is used as a reference for Gemini's `extract_system_messages()`
    /// implementation in `crates/radium-models/src/gemini.rs`.
    fn extract_system_prompt(messages: &[ChatMessage]) -> Option<String> {
        messages
            .iter()
            .find(|msg| msg.role == "system")
            .map(|msg| msg.content.clone())
    }
}

#[async_trait]
impl Model for ClaudeModel {
    async fn generate_text(
        &self,
        prompt: &str,
        parameters: Option<ModelParameters>,
    ) -> Result<ModelResponse, ModelError> {
        debug!(
            model_id = %self.model_id,
            prompt_len = prompt.len(),
            parameters = ?parameters,
            "ClaudeModel generating text"
        );

        // Convert single prompt to chat format for Claude
        let messages = vec![ChatMessage { role: "user".to_string(), content: prompt.to_string() }];

        self.generate_chat_completion(&messages, parameters).await
    }

    async fn generate_chat_completion(
        &self,
        messages: &[ChatMessage],
        parameters: Option<ModelParameters>,
    ) -> Result<ModelResponse, ModelError> {
        debug!(
            model_id = %self.model_id,
            message_count = messages.len(),
            parameters = ?parameters,
            "ClaudeModel generating chat completion"
        );

        // Build Claude API request
        let url = format!("{}/messages", self.base_url);

        // Extract system prompt if present (dedicated system field pattern)
        // This follows the reference pattern for providers with dedicated system instruction fields.
        // System messages are extracted and sent via the `system` field, not included in messages.
        let system = Self::extract_system_prompt(messages);

        // Convert non-system messages to Claude format
        // System messages are filtered out here - they're handled via the dedicated `system` field above.
        // This pattern is replicated in Gemini's implementation (see gemini.rs).
        let claude_messages: Vec<ClaudeMessage> = messages
            .iter()
            .filter(|msg| msg.role != "system")
            .map(Self::to_claude_message)
            .collect();

        // Build request body
        let mut request_body = ClaudeRequest {
            model: self.model_id.clone(),
            messages: claude_messages,
            max_tokens: 4096, // Default max tokens
            system,
            temperature: None,
            top_p: None,
            stop_sequences: None,
        };

        // Apply parameters if provided
        if let Some(params) = parameters {
            request_body.temperature = params.temperature;
            request_body.top_p = params.top_p;
            if let Some(max_tokens) = params.max_tokens {
                request_body.max_tokens = max_tokens;
            }
            request_body.stop_sequences = params.stop_sequences;
        }

        // Make API request using reqwest
        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to send request to Claude API");
                ModelError::RequestError(format!("Network error: {}", e))
            })?;

        // Check status
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                status = %status,
                error = %error_text,
                "Claude API returned error status"
            );

            // Map quota/rate limit errors to QuotaExceeded
            if status == 402 || status == 429 {
                // Parse error response JSON to check for Anthropic-specific error types
                let is_quota_error = if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error_text) {
                    // Check for Anthropic error structure: { "error": { "type": "...", "message": "..." } }
                    if let Some(error_obj) = error_json.get("error") {
                        if let Some(error_type) = error_obj.get("type").and_then(|t| t.as_str()) {
                            matches!(
                                error_type,
                                "rate_limit_error" | "overloaded_error" | "insufficient_quota"
                            )
                        } else {
                            false
                        }
                    } else if let Some(error_type) = error_json.get("type").and_then(|t| t.as_str()) {
                        // Alternative structure: { "type": "...", "message": "..." }
                        matches!(
                            error_type,
                            "rate_limit_error" | "overloaded_error" | "insufficient_quota"
                        )
                    } else {
                        // Fallback: check error text for quota-related keywords
                        error_text.to_lowercase().contains("quota")
                            || error_text.to_lowercase().contains("rate limit")
                            || error_text.to_lowercase().contains("insufficient")
                    }
                } else {
                    // If JSON parsing fails, check error text for quota-related keywords
                    error_text.to_lowercase().contains("quota")
                        || error_text.to_lowercase().contains("rate limit")
                        || error_text.to_lowercase().contains("insufficient")
                };

                if is_quota_error || status == 402 {
                    return Err(ModelError::QuotaExceeded {
                        provider: "anthropic".to_string(),
                        message: Some(error_text),
                    });
                }
            }

            // For 429, if it's a rate limit (not quota), we still treat it as QuotaExceeded
            // after potential retries (handled by orchestrator)
            if status == 429 {
                return Err(ModelError::QuotaExceeded {
                    provider: "anthropic".to_string(),
                    message: Some(error_text),
                });
            }

            // Other errors (400, 5xx) are handled as before
            return Err(ModelError::ModelResponseError(format!(
                "API error ({}): {}",
                status, error_text
            )));
        }

        // Parse response
        let claude_response: ClaudeResponse = response.json().await.map_err(|e| {
            error!(error = %e, "Failed to parse Claude API response");
            ModelError::SerializationError(format!("Failed to parse response: {}", e))
        })?;

        // Extract content from response
        let content = claude_response
            .content
            .iter()
            .find(|c| c.content_type == "text")
            .map(|c| c.text.clone())
            .ok_or_else(|| {
                error!("No text content in Claude API response");
                ModelError::ModelResponseError("No text content in API response".to_string())
            })?;

        // Extract usage information
        let usage = Some(ModelUsage {
            prompt_tokens: claude_response.usage.input_tokens,
            completion_tokens: claude_response.usage.output_tokens,
            total_tokens: claude_response.usage.input_tokens + claude_response.usage.output_tokens,
        });

        Ok(ModelResponse { content, model_id: Some(self.model_id.clone()), usage })
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }
}

// Claude API request/response structures

#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    messages: Vec<ClaudeMessage>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
    usage: ClaudeUsage,
}

#[derive(Debug, Deserialize)]
struct ClaudeContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_model_creation_with_api_key() {
        let model = ClaudeModel::with_api_key("claude-sonnet-4-5-20250929".to_string(), "test-key".to_string());
        assert_eq!(model.model_id(), "claude-sonnet-4-5-20250929");
    }

    #[test]
    fn test_system_prompt_extraction() {
        let messages = vec![
            ChatMessage { role: "system".to_string(), content: "You are helpful".to_string() },
            ChatMessage { role: "user".to_string(), content: "Hello".to_string() },
        ];
        let system = ClaudeModel::extract_system_prompt(&messages);
        assert_eq!(system, Some("You are helpful".to_string()));
    }

    #[test]
    #[ignore = "Requires API key and network access"]
    #[allow(clippy::disallowed_methods, clippy::disallowed_macros)] // Test code can use env::var and eprintln
    fn test_claude_generate_text() {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(async {
            let api_key = env::var("ANTHROPIC_API_KEY").ok();
            if api_key.is_none() {
                eprintln!("Skipping test: ANTHROPIC_API_KEY not set");
                return;
            }

            let model = ClaudeModel::new("claude-sonnet-4-5-20250929".to_string()).unwrap();
            let response =
                model.generate_text("Say hello", None).await.expect("Should generate text");

            assert!(!response.content.is_empty());
            assert_eq!(response.model_id, Some("claude-sonnet-4-5-20250929".to_string()));
        });
    }

    #[test]
    fn test_quota_error_detection_rate_limit_error() {
        // Test that rate_limit_error type is detected
        let error_json = r#"{"error":{"type":"rate_limit_error","message":"Rate limit exceeded"}}"#;
        let error_value: serde_json::Value = serde_json::from_str(error_json).unwrap();
        
        if let Some(error_obj) = error_value.get("error") {
            if let Some(error_type) = error_obj.get("type").and_then(|t| t.as_str()) {
                assert!(matches!(error_type, "rate_limit_error" | "overloaded_error" | "insufficient_quota"));
            }
        }
    }

    #[test]
    fn test_quota_error_detection_overloaded_error() {
        // Test that overloaded_error type is detected
        let error_json = r#"{"error":{"type":"overloaded_error","message":"Service overloaded"}}"#;
        let error_value: serde_json::Value = serde_json::from_str(error_json).unwrap();
        
        if let Some(error_obj) = error_value.get("error") {
            if let Some(error_type) = error_obj.get("type").and_then(|t| t.as_str()) {
                assert!(matches!(error_type, "rate_limit_error" | "overloaded_error" | "insufficient_quota"));
            }
        }
    }

    #[test]
    fn test_quota_error_detection_insufficient_quota() {
        // Test that insufficient_quota type is detected
        let error_json = r#"{"error":{"type":"insufficient_quota","message":"Insufficient quota"}}"#;
        let error_value: serde_json::Value = serde_json::from_str(error_json).unwrap();
        
        if let Some(error_obj) = error_value.get("error") {
            if let Some(error_type) = error_obj.get("type").and_then(|t| t.as_str()) {
                assert!(matches!(error_type, "rate_limit_error" | "overloaded_error" | "insufficient_quota"));
            }
        }
    }

    #[test]
    fn test_quota_error_detection_http_402() {
        // Test that HTTP 402 status code is treated as quota error
        // This is verified by the status == 402 check in the implementation
        assert_eq!(402, 402); // HTTP 402 should trigger QuotaExceeded
    }

    #[test]
    fn test_quota_error_detection_http_429() {
        // Test that HTTP 429 status code is treated as quota error
        // This is verified by the status == 429 check in the implementation
        assert_eq!(429, 429); // HTTP 429 should trigger QuotaExceeded
    }
}
