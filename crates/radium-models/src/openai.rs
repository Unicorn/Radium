//! OpenAI model implementation.
//!
//! This module provides an implementation of the `Model` trait for OpenAI's API.
//!
//! ## System Message Handling
//!
//! OpenAI uses an **inline approach** for system messages, where system messages are included
//! directly in the messages array with `role: "system"`. This differs from Claude and Gemini,
//! which use dedicated system instruction fields.
//!
//! **Key characteristics:**
//! - System messages are preserved with their `role: "system"` intact
//! - System messages are included in the messages array (not filtered out)
//! - Multiple system messages are supported and sent as separate message objects
//! - System messages are typically placed at the beginning of the conversation
//!
//! **Comparison with other providers:**
//! - **Claude/Gemini**: Extract system messages and send via dedicated `system`/`systemInstruction` field
//! - **OpenAI**: Include system messages inline in the messages array
//!
//! This inline approach is the native OpenAI API pattern and is fully supported by all OpenAI models.

use async_trait::async_trait;
use futures::stream::Stream;
use radium_abstraction::{
    ChatMessage, LogProb, Model, ModelError, ModelParameters, ModelResponse, ModelUsage,
    SafetyRating, StreamingModel,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::pin::Pin;
use std::task::{Context, Poll};
use tracing::{debug, error, warn};

/// OpenAI model implementation.
#[derive(Debug, Clone)]
pub struct OpenAIModel {
    /// The model ID (e.g., "gpt-4", "gpt-3.5-turbo").
    model_id: String,
    /// The API key for authentication.
    api_key: String,
    /// The base URL for the OpenAI API.
    base_url: String,
    /// HTTP client for making requests.
    client: Client,
}

impl OpenAIModel {
    /// Creates a new `OpenAIModel` with the given model ID.
    ///
    /// # Arguments
    /// * `model_id` - The OpenAI model ID to use (e.g., "gpt-4")
    ///
    /// # Errors
    /// Returns a `ModelError` if the API key is not found in environment variables.
    #[allow(clippy::disallowed_methods)] // env::var is needed for API key loading
    pub fn new(model_id: String) -> Result<Self, ModelError> {
        let api_key = env::var("OPENAI_API_KEY").map_err(|_| {
            ModelError::UnsupportedModelProvider(
                "OPENAI_API_KEY environment variable not set".to_string(),
            )
        })?;

        Ok(Self {
            model_id,
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
            client: Client::new(),
        })
    }

    /// Creates a new `OpenAIModel` with a custom API key.
    ///
    /// # Arguments
    /// * `model_id` - The OpenAI model ID to use
    /// * `api_key` - The API key for authentication
    #[must_use]
    pub fn with_api_key(model_id: String, api_key: String) -> Self {
        Self {
            model_id,
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
            client: Client::new(),
        }
    }

    /// Converts our ChatMessage role to OpenAI API role format.
    ///
    /// OpenAI natively supports `role: "system"` messages, so system messages are preserved
    /// with their role intact. This differs from providers like Gemini that require system
    /// messages to be extracted and sent via a dedicated field.
    ///
    /// # Arguments
    /// * `role` - The message role ("system", "user", "assistant")
    ///
    /// # Returns
    /// The OpenAI API role string, preserving system role as-is.
    fn role_to_openai(role: &str) -> String {
        match role {
            "assistant" => "assistant".to_string(),
            "system" => "system".to_string(), // Preserved as-is (inline approach)
            "user" => "user".to_string(),
            _ => role.to_string(),
        }
    }
}

#[async_trait]
impl Model for OpenAIModel {
    async fn generate_text(
        &self,
        prompt: &str,
        parameters: Option<ModelParameters>,
    ) -> Result<ModelResponse, ModelError> {
        debug!(
            model_id = %self.model_id,
            prompt_len = prompt.len(),
            parameters = ?parameters,
            "OpenAIModel generating text"
        );

        // Convert single prompt to chat format for OpenAI
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
            "OpenAIModel generating chat completion"
        );

        // Build OpenAI API request
        let url = format!("{}/chat/completions", self.base_url);

        // Convert messages to OpenAI format
        // Note: OpenAI uses an inline approach - system messages are included in the messages
        // array with role: "system", unlike Claude/Gemini which extract them to dedicated fields.
        let openai_messages: Vec<OpenAIMessage> = messages
            .iter()
            .map(|msg| OpenAIMessage {
                role: Self::role_to_openai(&msg.role),
                content: msg.content.clone(),
            })
            .collect();

        // Build request body
        let mut request_body = OpenAIRequest {
            model: self.model_id.clone(),
            messages: openai_messages,
            temperature: None,
            top_p: None,
            max_tokens: None,
            stop: None,
        };

        // Apply parameters if provided
        if let Some(params) = parameters {
            request_body.temperature = params.temperature;
            request_body.top_p = params.top_p;
            request_body.max_tokens = params.max_tokens;
            request_body.stop = params.stop_sequences;
        }

        // Make API request using reqwest
        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to send request to OpenAI API");
                ModelError::RequestError(format!("Network error: {}", e))
            })?;

        // Check status
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                status = %status,
                error = %error_text,
                "OpenAI API returned error status"
            );
            
            // Map quota/rate limit errors to QuotaExceeded
            if status == 402 || status == 429 {
                // Check for quota-related error messages in response body
                let is_quota_error = error_text.to_lowercase().contains("exceeded your current quota")
                    || error_text.to_lowercase().contains("insufficient_quota")
                    || error_text.to_lowercase().contains("quota")
                    || error_text.to_lowercase().contains("rate limit");
                
                if is_quota_error || status == 402 {
                    return Err(ModelError::QuotaExceeded {
                        provider: "openai".to_string(),
                        message: Some(error_text),
                    });
                }
            }
            
            // For 429, if it's a rate limit (not quota), we still treat it as QuotaExceeded
            // after potential retries (handled by orchestrator)
            if status == 429 {
                return Err(ModelError::QuotaExceeded {
                    provider: "openai".to_string(),
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
        let openai_response: OpenAIResponse = response.json().await.map_err(|e| {
            error!(error = %e, "Failed to parse OpenAI API response");
            ModelError::SerializationError(format!("Failed to parse response: {}", e))
        })?;

        // Extract content from response
        let choice = openai_response.choices.first().ok_or_else(|| {
            error!("No content in OpenAI API response");
            ModelError::ModelResponseError("No content in API response".to_string())
        })?;

        let content = choice.message.content.clone();

        // Extract usage information
        let usage = openai_response.usage.map(|u| ModelUsage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        });

        // Extract metadata from choice and response
        let metadata = if choice.finish_reason.is_some()
            || choice.logprobs.is_some()
            || choice.content_filter_results.is_some()
            || openai_response.system_fingerprint.is_some()
        {
            let openai_meta = OpenAIMetadata {
                finish_reason: choice.finish_reason.clone(),
                logprobs: choice.logprobs.as_ref().map(|lp| {
                    lp.content.iter().map(|c| LogProb::from(c)).collect()
                }),
                content_filter_results: choice.content_filter_results.as_ref().map(|cfr| {
                    vec![SafetyRating::from(cfr)]
                }),
                model_version: openai_response.system_fingerprint.clone(),
            };
            let metadata_map: HashMap<String, serde_json::Value> = openai_meta.into();
            if metadata_map.is_empty() {
                None
            } else {
                Some(metadata_map)
            }
        } else {
            None
        };

        // Check for safety blocks (behavior will be applied at higher level)
        let safety_ratings = metadata
            .as_ref()
            .and_then(|m| m.get("safety_ratings"))
            .and_then(|v| serde_json::from_value::<Vec<SafetyRating>>(v.clone()).ok());
        
        if let Some(ref ratings) = safety_ratings {
            let blocked = ratings.iter().any(|r| r.blocked);
            if blocked {
                warn!(
                    provider = "openai",
                    "Content was filtered by safety system. Metadata contains safety_ratings."
                );
            }
        }

        Ok(ModelResponse {
            content,
            model_id: Some(self.model_id.clone()),
            usage,
            metadata,
        })
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }
}

#[async_trait]
impl StreamingModel for OpenAIModel {
    async fn generate_stream(
        &self,
        prompt: &str,
        parameters: Option<ModelParameters>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, ModelError>> + Send>>, ModelError> {
        debug!(
            model_id = %self.model_id,
            prompt_len = prompt.len(),
            parameters = ?parameters,
            "OpenAIModel generating streaming text"
        );

        // Convert single prompt to chat format for OpenAI
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }];

        // Build OpenAI streaming API request
        let url = format!("{}/chat/completions", self.base_url);

        // Convert messages to OpenAI format
        let openai_messages: Vec<OpenAIMessage> = messages
            .iter()
            .map(|msg| OpenAIMessage {
                role: Self::role_to_openai(&msg.role),
                content: msg.content.clone(),
            })
            .collect();

        // Build request body with streaming enabled
        let mut request_body = OpenAIStreamingRequest {
            model: self.model_id.clone(),
            messages: openai_messages,
            stream: true,
            temperature: None,
            top_p: None,
            max_tokens: None,
            stop: None,
        };

        // Apply parameters if provided
        if let Some(params) = parameters {
            request_body.temperature = params.temperature;
            request_body.top_p = params.top_p;
            request_body.max_tokens = params.max_tokens;
            request_body.stop = params.stop_sequences;
        }

        // Make streaming API request
        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to send streaming request to OpenAI API");
                ModelError::RequestError(format!("Network error: {}", e))
            })?;

        // Check status before streaming
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                status = %status,
                error = %error_text,
                "OpenAI API returned error status for streaming request"
            );

            // Map quota/rate limit errors to QuotaExceeded
            if status == 402 || status == 429 {
                let is_quota_error = error_text.to_lowercase().contains("exceeded your current quota")
                    || error_text.to_lowercase().contains("insufficient_quota")
                    || error_text.to_lowercase().contains("quota")
                    || error_text.to_lowercase().contains("rate limit");

                if is_quota_error || status == 402 {
                    return Err(ModelError::QuotaExceeded {
                        provider: "openai".to_string(),
                        message: Some(error_text),
                    });
                }
            }

            // For 429, if it's a rate limit (not quota), we still treat it as QuotaExceeded
            if status == 429 {
                return Err(ModelError::QuotaExceeded {
                    provider: "openai".to_string(),
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

            // Other errors
            return Err(ModelError::ModelResponseError(format!(
                "API error ({}): {}",
                status, error_text
            )));
        }

        // Create SSE stream parser
        Ok(Box::pin(OpenAISSEStream::new(response)))
    }
}

// SSE stream parser for OpenAI format
struct OpenAISSEStream {
    stream: Pin<Box<dyn Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send>>,
    buffer: String,
    accumulated: String,
    done: bool,
}

impl OpenAISSEStream {
    fn new(response: reqwest::Response) -> Self {
        Self {
            stream: Box::pin(response.bytes_stream()),
            buffer: String::new(),
            accumulated: String::new(),
            done: false,
        }
    }
}

impl Stream for OpenAISSEStream {
    type Item = Result<String, ModelError>;

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
                                if event.starts_with("data: ") {
                                    let data = &event[6..]; // Skip "data: " prefix

                                    // Check for [DONE] signal
                                    if data.trim() == "[DONE]" {
                                        self.done = true;
                                        if !self.accumulated.is_empty() {
                                            return Poll::Ready(Some(Ok(self.accumulated.clone())));
                                        }
                                        return Poll::Ready(None);
                                    }

                                    // Parse JSON chunk
                                    match serde_json::from_str::<OpenAIStreamingResponse>(data) {
                                        Ok(streaming_response) => {
                                            // Extract content from choices[0].delta.content
                                            if let Some(choice) = streaming_response.choices.first() {
                                                if let Some(content) = &choice.delta.content {
                                                    if !content.is_empty() {
                                                        self.accumulated.push_str(content);
                                                        return Poll::Ready(Some(Ok(self.accumulated.clone())));
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            // Skip malformed JSON chunks (some servers send empty chunks)
                                            debug!("Failed to parse SSE chunk: {}", e);
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
                    // Stream ended - process any remaining events in buffer
                    while let Some(end_idx) = self.buffer.find("\n\n") {
                        let event = self.buffer[..end_idx].to_string();
                        self.buffer = self.buffer[end_idx + 2..].to_string();

                        if event.starts_with("data: ") {
                            let data = &event[6..];

                            if data.trim() == "[DONE]" {
                                self.done = true;
                                if !self.accumulated.is_empty() {
                                    return Poll::Ready(Some(Ok(self.accumulated.clone())));
                                }
                                return Poll::Ready(None);
                            }

                            if let Ok(streaming_response) =
                                serde_json::from_str::<OpenAIStreamingResponse>(data)
                            {
                                if let Some(choice) = streaming_response.choices.first() {
                                    if let Some(content) = &choice.delta.content {
                                        if !content.is_empty() {
                                            self.accumulated.push_str(content);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // No more events in buffer
                    self.done = true;
                    if !self.accumulated.is_empty() {
                        return Poll::Ready(Some(Ok(self.accumulated.clone())));
                    }
                    return Poll::Ready(None);
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

// Streaming request structure for OpenAI
#[derive(Debug, Serialize)]
struct OpenAIStreamingRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
}

// Streaming response structure for OpenAI SSE
#[derive(Debug, Deserialize)]
struct OpenAIStreamingResponse {
    choices: Vec<OpenAIStreamingChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    usage: Option<OpenAIUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamingChoice {
    delta: OpenAIStreamingDelta,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamingDelta {
    content: Option<String>,
}

// OpenAI API request/response structures

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
    #[serde(rename = "system_fingerprint")]
    system_fingerprint: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
    #[serde(rename = "finish_reason")]
    finish_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    logprobs: Option<OpenAILogProbs>,
    #[serde(rename = "content_filter_results", skip_serializing_if = "Option::is_none")]
    content_filter_results: Option<OpenAIContentFilter>,
}

#[derive(Debug, Deserialize)]
#[allow(clippy::struct_field_names)] // Matches API naming
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

// OpenAI-specific metadata structures

#[derive(Debug, Deserialize)]
struct OpenAILogProbs {
    content: Vec<OpenAILogProbContent>,
}

#[derive(Debug, Deserialize)]
struct OpenAILogProbContent {
    token: String,
    logprob: f64,
    bytes: Option<Vec<u8>>,
}

#[derive(Debug, Deserialize)]
struct OpenAIContentFilter {
    category: String,
    severity: String,
    filtered: bool,
}

// Common metadata structure for OpenAI
#[derive(Debug, Clone, Serialize)]
struct OpenAIMetadata {
    finish_reason: Option<String>,
    logprobs: Option<Vec<LogProb>>,
    content_filter_results: Option<Vec<SafetyRating>>,
    model_version: Option<String>,
}

impl From<OpenAIMetadata> for HashMap<String, serde_json::Value> {
    fn from(meta: OpenAIMetadata) -> Self {
        let mut map = HashMap::new();
        if let Some(finish_reason) = meta.finish_reason {
            map.insert("finish_reason".to_string(), serde_json::Value::String(finish_reason));
        }
        if let Some(logprobs) = meta.logprobs {
            map.insert("logprobs".to_string(), serde_json::to_value(logprobs).unwrap());
        }
        if let Some(content_filter_results) = meta.content_filter_results {
            map.insert("safety_ratings".to_string(), serde_json::to_value(content_filter_results).unwrap());
        }
        if let Some(model_version) = meta.model_version {
            map.insert("model_version".to_string(), serde_json::Value::String(model_version));
        }
        map
    }
}

impl From<&OpenAILogProbContent> for LogProb {
    fn from(logprob: &OpenAILogProbContent) -> Self {
        LogProb {
            token: logprob.token.clone(),
            logprob: logprob.logprob,
            bytes: logprob.bytes.clone(),
        }
    }
}

impl From<&OpenAIContentFilter> for SafetyRating {
    fn from(filter: &OpenAIContentFilter) -> Self {
        SafetyRating {
            category: filter.category.clone(),
            probability: filter.severity.clone(),
            blocked: filter.filtered,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_conversion() {
        assert_eq!(OpenAIModel::role_to_openai("user"), "user");
        assert_eq!(OpenAIModel::role_to_openai("assistant"), "assistant");
        assert_eq!(OpenAIModel::role_to_openai("system"), "system");
    }

    #[test]
    fn test_system_message_role_preservation() {
        use radium_abstraction::ChatMessage;

        // Test that system messages preserve their role when converted to OpenAI format
        let system_msg = ChatMessage { role: "system".to_string(), content: "You are helpful.".to_string() };
        let role = OpenAIModel::role_to_openai(&system_msg.role);
        assert_eq!(role, "system");

        // Test that system messages are included in messages array (not filtered)
        let messages = vec![
            ChatMessage { role: "system".to_string(), content: "System instruction.".to_string() },
            ChatMessage { role: "user".to_string(), content: "User message.".to_string() },
            ChatMessage { role: "assistant".to_string(), content: "Assistant message.".to_string() },
        ];

        // Simulate message conversion (as done in generate_chat_completion)
        let openai_messages: Vec<_> = messages
            .iter()
            .map(|msg| (OpenAIModel::role_to_openai(&msg.role), msg.content.clone()))
            .collect();

        // Verify system message is present with correct role
        assert_eq!(openai_messages.len(), 3);
        assert_eq!(openai_messages[0].0, "system");
        assert_eq!(openai_messages[1].0, "user");
        assert_eq!(openai_messages[2].0, "assistant");
    }

    #[test]
    fn test_multiple_system_messages() {
        use radium_abstraction::ChatMessage;

        // Test that multiple system messages are preserved (OpenAI supports this)
        let messages = vec![
            ChatMessage { role: "system".to_string(), content: "First system message.".to_string() },
            ChatMessage { role: "system".to_string(), content: "Second system message.".to_string() },
            ChatMessage { role: "user".to_string(), content: "User message.".to_string() },
        ];

        // Simulate message conversion
        let openai_messages: Vec<_> = messages
            .iter()
            .map(|msg| (OpenAIModel::role_to_openai(&msg.role), msg.content.clone()))
            .collect();

        // Verify both system messages are preserved
        assert_eq!(openai_messages.len(), 3);
        assert_eq!(openai_messages[0].0, "system");
        assert_eq!(openai_messages[0].1, "First system message.");
        assert_eq!(openai_messages[1].0, "system");
        assert_eq!(openai_messages[1].1, "Second system message.");
        assert_eq!(openai_messages[2].0, "user");
    }

    #[test]
    fn test_openai_model_creation_with_api_key() {
        let model = OpenAIModel::with_api_key("gpt-4".to_string(), "test-key".to_string());
        assert_eq!(model.model_id(), "gpt-4");
    }

    #[test]
    #[ignore = "Requires API key and network access"]
    #[allow(clippy::disallowed_methods, clippy::disallowed_macros)] // Test code can use env::var and eprintln
    fn test_openai_generate_text() {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(async {
            let api_key = env::var("OPENAI_API_KEY").ok();
            if api_key.is_none() {
                eprintln!("Skipping test: OPENAI_API_KEY not set");
                return;
            }

            let model = OpenAIModel::new("gpt-3.5-turbo".to_string()).unwrap();
            let response =
                model.generate_text("Say hello", None).await.expect("Should generate text");

            assert!(!response.content.is_empty());
            assert_eq!(response.model_id, Some("gpt-3.5-turbo".to_string()));
        });
    }
}
