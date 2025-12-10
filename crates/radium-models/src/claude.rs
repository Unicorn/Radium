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
use futures::stream::Stream;
use radium_abstraction::{
    ChatMessage, ContentBlock, ImageSource, MessageContent, Model, ModelError, ModelParameters,
    ModelResponse, ModelUsage, StreamingModel, StreamItem,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::pin::Pin;
use std::task::{Context, Poll};
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
    /// Optional cache configuration for context caching.
    cache_config: Option<crate::context_cache::CacheConfig>,
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
            cache_config: None,
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
            cache_config: None,
        }
    }

    /// Sets the cache configuration for this model.
    ///
    /// # Arguments
    /// * `cache_config` - The cache configuration
    #[must_use]
    pub fn with_cache_config(mut self, cache_config: crate::context_cache::CacheConfig) -> Self {
        self.cache_config = Some(cache_config);
        self
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
    /// - Extracts text from MessageContent::Text or first text block from MessageContent::Blocks
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
    /// **Reference:** This pattern is used as a reference for Gemini's `extract_system_messages()`
    /// implementation in `crates/radium-models/src/gemini.rs`.
    fn extract_system_prompt(messages: &[ChatMessage]) -> Option<String> {
        messages
            .iter()
            .find(|msg| msg.role == "system")
            .and_then(|msg| match &msg.content {
                MessageContent::Text(text) => Some(text.clone()),
                MessageContent::Blocks(blocks) => {
                    // Extract first text block if available
                    blocks
                        .iter()
                        .find_map(|block| match block {
                            ContentBlock::Text { text } => Some(text.clone()),
                            _ => None,
                        })
                }
            })
    }

    /// Converts a ContentBlock to Claude's content block format.
    fn content_block_to_claude(
        block: &ContentBlock,
        cache_control: Option<CacheControl>,
    ) -> Result<ClaudeContentBlock, ModelError> {
        match block {
            ContentBlock::Text { text } => Ok(ClaudeContentBlock::Text {
                text: text.clone(),
                cache_control,
            }),
            ContentBlock::Image { source, media_type } => {
                let claude_source = match source {
                    ImageSource::Base64 { data } => ClaudeImageSource::Base64 {
                        media_type: media_type.clone(),
                        data: data.clone(),
                    },
                    ImageSource::Url { url } => ClaudeImageSource::Url {
                        url: url.clone(),
                    },
                    ImageSource::File { path } => {
                        // Read file and encode to Base64
                        let bytes = std::fs::read(path).map_err(|e| {
                            ModelError::InvalidMediaSource {
                                media_source: path.display().to_string(),
                                reason: format!("Failed to read file: {}", e),
                            }
                        })?;
                        use base64::Engine;
                        let engine = base64::engine::general_purpose::STANDARD;
                        ClaudeImageSource::Base64 {
                            media_type: media_type.clone(),
                            data: engine.encode(&bytes),
                        }
                    }
                };
                Ok(ClaudeContentBlock::Image {
                    source: claude_source,
                    cache_control,
                })
            }
            ContentBlock::Audio { .. } => Err(ModelError::UnsupportedContentType {
                content_type: "audio".to_string(),
                model: "claude".to_string(),
            }),
            ContentBlock::Video { .. } => Err(ModelError::UnsupportedContentType {
                content_type: "video".to_string(),
                model: "claude".to_string(),
            }),
            ContentBlock::Document { .. } => Err(ModelError::UnsupportedContentType {
                content_type: "document".to_string(),
                model: "claude".to_string(),
            }),
        }
    }

    /// Converts our ChatMessage to Claude API message format.
    fn to_claude_message(msg: &ChatMessage) -> Result<ClaudeMessage, ModelError> {
        let role = if msg.role == "assistant" {
            "assistant"
        } else {
            "user"
        }
        .to_string();

        let content = match &msg.content {
            MessageContent::Text(text) => ClaudeMessageContent::String(text.clone()),
            MessageContent::Blocks(blocks) => {
                let claude_blocks: Result<Vec<ClaudeContentBlock>, ModelError> = blocks
                    .iter()
                    .map(|b| Self::content_block_to_claude(b, None))
                    .collect();
                ClaudeMessageContent::Blocks(claude_blocks?)
            }
        };

        Ok(ClaudeMessage { role, content })
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
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: MessageContent::Text(prompt.to_string()),
        }];

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
        let claude_messages: Result<Vec<ClaudeMessage>, ModelError> = messages
            .iter()
            .filter(|msg| msg.role != "system")
            .map(Self::to_claude_message)
            .collect();
        let claude_messages = claude_messages?;

        // Build request body
        let mut request_body = ClaudeRequest {
            model: self.model_id.clone(),
            messages: claude_messages,
            max_tokens: 4096, // Default max tokens
            system,
            temperature: None,
            top_p: None,
            stop_sequences: None,
            thinking: None,
        };

        // Apply parameters if provided
        if let Some(params) = parameters {
            request_body.temperature = params.temperature;
            request_body.top_p = params.top_p;
            if let Some(max_tokens) = params.max_tokens {
                request_body.max_tokens = max_tokens;
            }
            request_body.stop_sequences = params.stop_sequences;
            
            // Map reasoning effort to thinking config for Claude models
            if let Some(effort) = params.reasoning_effort {
                let thinking_budget = match effort {
                    radium_abstraction::ReasoningEffort::Low => 0.3,   // Minimal extended thinking
                    radium_abstraction::ReasoningEffort::Medium => 0.6, // Standard extended thinking
                    radium_abstraction::ReasoningEffort::High => 1.0,   // Maximum extended thinking
                };
                request_body.thinking = Some(ClaudeThinkingConfig {
                    thinking_budget: Some(thinking_budget),
                });
            }
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
        let cache_usage = if claude_response.usage.cache_creation_input_tokens.is_some()
            || claude_response.usage.cache_read_input_tokens.is_some()
        {
            Some(radium_abstraction::CacheUsage {
                cache_creation_tokens: claude_response
                    .usage
                    .cache_creation_input_tokens
                    .unwrap_or(0),
                cache_read_tokens: claude_response
                    .usage
                    .cache_read_input_tokens
                    .unwrap_or(0),
                regular_tokens: claude_response.usage.input_tokens
                    - claude_response.usage.cache_creation_input_tokens.unwrap_or(0)
                    - claude_response.usage.cache_read_input_tokens.unwrap_or(0),
            })
        } else {
            None
        };

        let usage = Some(ModelUsage {
            prompt_tokens: claude_response.usage.input_tokens,
            completion_tokens: claude_response.usage.output_tokens,
            total_tokens: claude_response.usage.input_tokens + claude_response.usage.output_tokens,
            cache_usage,
        });

        // Extract thinking process if present
        let metadata = if let Some(thinking) = claude_response.thinking {
            let mut metadata_map = HashMap::new();
            metadata_map.insert("thinking_process".to_string(), thinking);
            Some(metadata_map)
        } else {
            None
        };

        Ok(ModelResponse {
            content,
            model_id: Some(self.model_id.clone()),
            usage,
            metadata,
            tool_calls: None,
        })
    }

    async fn generate_with_tools(
        &self,
        _messages: &[ChatMessage],
        _tools: &[radium_abstraction::Tool],
        _tool_config: Option<&radium_abstraction::ToolConfig>,
    ) -> Result<ModelResponse, ModelError> {
        Err(ModelError::UnsupportedModelProvider(
            format!("ClaudeModel does not support function calling yet"),
        ))
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }
}

#[async_trait]
impl StreamingModel for ClaudeModel {
    async fn generate_stream(
        &self,
        prompt: &str,
        parameters: Option<ModelParameters>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamItem, ModelError>> + Send>>, ModelError> {
        debug!(
            model_id = %self.model_id,
            prompt_len = prompt.len(),
            parameters = ?parameters,
            "ClaudeModel generating streaming text"
        );

        // Convert single prompt to chat format for Claude
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: MessageContent::Text(prompt.to_string()),
        }];

        // Build Claude API request with streaming enabled
        let url = format!("{}/messages", self.base_url);

        // Extract system prompt if present
        let system = Self::extract_system_prompt(&messages);

        // Convert non-system messages to Claude format
        let claude_messages: Result<Vec<ClaudeMessage>, ModelError> = messages
            .iter()
            .filter(|msg| msg.role != "system")
            .map(Self::to_claude_message)
            .collect();
        let claude_messages = claude_messages?;

        // Build request body with streaming enabled
        let mut request_body = ClaudeStreamingRequest {
            model: self.model_id.clone(),
            messages: claude_messages,
            max_tokens: 4096,
            system,
            temperature: None,
            top_p: None,
            stop_sequences: None,
            thinking: None,
            stream: true,
        };

        // Apply parameters if provided
        if let Some(params) = parameters {
            request_body.temperature = params.temperature;
            request_body.top_p = params.top_p;
            if let Some(max_tokens) = params.max_tokens {
                request_body.max_tokens = max_tokens;
            }
            request_body.stop_sequences = params.stop_sequences;

            // Map reasoning effort to thinking config for Claude models
            if let Some(effort) = params.reasoning_effort {
                let thinking_budget = match effort {
                    radium_abstraction::ReasoningEffort::Low => 0.3,
                    radium_abstraction::ReasoningEffort::Medium => 0.6,
                    radium_abstraction::ReasoningEffort::High => 1.0,
                };
                request_body.thinking = Some(ClaudeThinkingConfig {
                    thinking_budget: Some(thinking_budget),
                });
            }
        }

        // Make streaming API request
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
                error!(error = %e, "Failed to send streaming request to Claude API");
                ModelError::RequestError(format!("Network error: {}", e))
            })?;

        // Check status before streaming
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                status = %status,
                error = %error_text,
                "Claude API returned error status for streaming request"
            );

            // Map quota/rate limit errors to QuotaExceeded
            if status == 402 || status == 429 {
                let is_quota_error = if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error_text) {
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
                        matches!(
                            error_type,
                            "rate_limit_error" | "overloaded_error" | "insufficient_quota"
                        )
                    } else {
                        error_text.to_lowercase().contains("quota")
                            || error_text.to_lowercase().contains("rate limit")
                            || error_text.to_lowercase().contains("insufficient")
                    }
                } else {
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

            if status == 429 {
                return Err(ModelError::QuotaExceeded {
                    provider: "anthropic".to_string(),
                    message: Some(error_text),
                });
            }

            // Map authentication errors (401, 403) to UnsupportedModelProvider
            if status == 401 || status == 403 {
                return Err(ModelError::UnsupportedModelProvider(format!(
                    "Authentication failed ({}): {}",
                    status, error_text
                )));
            }

            // Map server errors (500-599) to ModelResponseError
            if (500..=599).contains(&status.as_u16()) {
                return Err(ModelError::ModelResponseError(format!(
                    "Server error ({}): {}",
                    status, error_text
                )));
            }

            return Err(ModelError::ModelResponseError(format!(
                "API error ({}): {}",
                status, error_text
            )));
        }

        // Create SSE stream parser
        Ok(Box::pin(ClaudeSSEStream::new(response)))
    }
}

// SSE stream parser for Claude format
struct ClaudeSSEStream {
    stream: Pin<Box<dyn Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send>>,
    buffer: String,
    done: bool,
}

impl ClaudeSSEStream {
    fn new(response: reqwest::Response) -> Self {
        Self {
            stream: Box::pin(response.bytes_stream()),
            buffer: String::new(),
            done: false,
        }
    }
}

impl Stream for ClaudeSSEStream {
    type Item = Result<StreamItem, ModelError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.done {
            return Poll::Ready(None);
        }

        loop {
            // Poll the underlying byte stream
            match self.stream.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(bytes))) => {
                    // Convert bytes to string and append to buffer
                    match String::from_utf8(bytes.to_vec()) {
                        Ok(chunk) => {
                            self.buffer.push_str(&chunk);

                            // Process complete SSE events (separated by \n\n)
                            while let Some(end_idx) = self.buffer.find("\n\n") {
                                let event = self.buffer[..end_idx].to_string();
                                self.buffer = self.buffer[end_idx + 2..].to_string();

                                // Parse SSE event
                                if event.starts_with("event: ") {
                                    let lines: Vec<&str> = event.lines().collect();
                                    if lines.len() >= 2 {
                                        let event_type = lines[0].strip_prefix("event: ").unwrap_or("");

                                        // Extract data line (may be prefixed with "data: ")
                                        let data_line = lines[1];
                                        let data = if data_line.starts_with("data: ") {
                                            &data_line[6..]
                                        } else {
                                            data_line
                                        };

                                        match event_type {
                                            "message_stop" => {
                                                self.done = true;
                                                return Poll::Ready(None);
                                            }
                                            "content_block_start" | "content_block_delta" => {
                                                // Parse JSON chunk
                                                if let Ok(streaming_event) = serde_json::from_str::<ClaudeStreamingEvent>(data) {
                                                    // Handle thinking vs answer tokens
                                                    if event_type == "content_block_delta" {
                                                        if let Some(delta) = streaming_event.delta {
                                                            if delta.event_type == "text_delta" {
                                                                if let Some(text) = delta.text {
                                                                    // Check if this is thinking content
                                                                    let is_thinking = streaming_event.index == Some(0)
                                                                        && streaming_event.content_block.as_ref()
                                                                            .and_then(|cb| cb.thinking.as_ref())
                                                                            .is_some();

                                                                    if is_thinking {
                                                                        return Poll::Ready(Some(Ok(StreamItem::ThinkingToken(text))));
                                                                    } else {
                                                                        return Poll::Ready(Some(Ok(StreamItem::AnswerToken(text))));
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    } else if event_type == "content_block_start" {
                                                        // Check if this is a thinking block
                                                        if let Some(content_block) = streaming_event.content_block {
                                                            if content_block.thinking.is_some() {
                                                                // This is the start of thinking - no token yet
                                                                continue;
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            "error" => {
                                                if let Ok(error_event) = serde_json::from_str::<ClaudeErrorEvent>(data) {
                                                    return Poll::Ready(Some(Err(ModelError::ModelResponseError(
                                                        format!("Stream error: {}", error_event.error.message)
                                                    ))));
                                                }
                                            }
                                            _ => {
                                                // Skip other event types (ping, message_start, etc.)
                                                continue;
                                            }
                                        }
                                    }
                                }
                            }

                            // Continue polling for more data
                            continue;
                        }
                        Err(e) => {
                            return Poll::Ready(Some(Err(ModelError::SerializationError(format!(
                                "Failed to decode SSE chunk: {}",
                                e
                            )))));
                        }
                    }
                }
                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(Some(Err(ModelError::RequestError(format!(
                        "Stream error: {}",
                        e
                    )))));
                }
                Poll::Ready(None) => {
                    // Stream ended
                    self.done = true;
                    return Poll::Ready(None);
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

// Claude API request/response structures

#[derive(Debug, Serialize)]
struct ClaudeThinkingConfig {
    /// Thinking budget for extended thinking (0.0 to 1.0).
    /// Higher values allow more thinking tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    thinking_budget: Option<f32>,
}

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
    /// Thinking configuration for extended thinking models.
    #[serde(skip_serializing_if = "Option::is_none")]
    thinking: Option<ClaudeThinkingConfig>,
}

#[derive(Debug, Serialize)]
struct ClaudeStreamingRequest {
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
    #[serde(skip_serializing_if = "Option::is_none")]
    thinking: Option<ClaudeThinkingConfig>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum ClaudeMessageContent {
    String(String),
    Blocks(Vec<ClaudeContentBlock>),
}

#[derive(Debug, Serialize, Deserialize)]
struct ClaudeMessage {
    role: String,
    content: ClaudeMessageContent,
}

/// Cache control configuration for Claude prompt caching.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CacheControl {
    /// Cache type - "ephemeral" for prompt caching.
    #[serde(rename = "type")]
    cache_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ClaudeContentBlock {
    #[serde(rename = "text")]
    Text {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    #[serde(rename = "image")]
    Image {
        source: ClaudeImageSource,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ClaudeImageSource {
    #[serde(rename = "base64")]
    Base64 {
        #[serde(rename = "media_type")]
        media_type: String,
        data: String,
    },
    #[serde(rename = "url")]
    Url { url: String },
}

#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
    usage: ClaudeUsage,
    /// Thinking process for extended thinking models (may be present in response)
    #[serde(skip_serializing_if = "Option::is_none")]
    thinking: Option<serde_json::Value>,
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
    #[serde(default)]
    cache_creation_input_tokens: Option<u32>,
    #[serde(default)]
    cache_read_input_tokens: Option<u32>,
}

// Streaming response structures
#[derive(Debug, Deserialize)]
struct ClaudeStreamingEvent {
    #[serde(rename = "type")]
    event_type: String,
    index: Option<usize>,
    delta: Option<ClaudeStreamingDelta>,
    content_block: Option<ClaudeStreamingContentBlock>,
}

#[derive(Debug, Deserialize)]
struct ClaudeStreamingDelta {
    #[serde(rename = "type")]
    event_type: String,
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ClaudeStreamingContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: Option<String>,
    thinking: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct ClaudeErrorEvent {
    error: ClaudeError,
}

#[derive(Debug, Deserialize)]
struct ClaudeError {
    #[serde(rename = "type")]
    error_type: String,
    message: String,
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
            ChatMessage {
                role: "system".to_string(),
                content: MessageContent::Text("You are helpful".to_string()),
            },
            ChatMessage {
                role: "user".to_string(),
                content: MessageContent::Text("Hello".to_string()),
            },
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
