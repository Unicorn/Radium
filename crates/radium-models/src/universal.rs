//! Universal OpenAI-compatible model implementation.
//!
//! This module provides an implementation of the `Model` trait for any server that implements
//! the OpenAI Chat Completions API specification. This enables Radium to work with:
//!
//! - **vLLM**: High-performance LLM inference server
//! - **LocalAI**: Local inference server with OpenAI-compatible API
//! - **LM Studio**: Desktop app for running local models
//! - **Ollama**: Local model runner with OpenAI-compatible endpoints
//! - **Any OpenAI-compatible server**: Works with any server implementing the OpenAI API spec
//!
//! # Quick Start
//!
//! ```no_run
//! use radium_models::UniversalModel;
//! use radium_abstraction::{ChatMessage, Model};
//!
//! # async fn example() -> Result<(), radium_abstraction::ModelError> {
//! let model = UniversalModel::without_auth(
//!     "llama-2-7b".to_string(),
//!     "http://localhost:1234/v1".to_string(),
//! );
//!
//! let messages = vec![ChatMessage {
//!     role: "user".to_string(),
//!     content: "Say hello".to_string(),
//! }];
//!
//! let response = model.generate_chat_completion(&messages, None).await?;
//! println!("{}", response.content);
//! # Ok(())
//! # }
//! ```
//!
//! # Constructor Patterns
//!
//! - `new()` - Loads API key from `UNIVERSAL_API_KEY` or `OPENAI_COMPATIBLE_API_KEY` env vars
//! - `with_api_key()` - Explicit API key for authenticated servers
//! - `without_auth()` - No authentication (most common for local servers)
//!
//! # Streaming Support
//!
//! Use `generate_chat_completion_stream()` for real-time token generation via SSE.
//!
//! # Documentation
//!
//! For detailed setup guides, troubleshooting, and examples, see the
//! [Universal Provider Guide](../../../docs/universal-provider-guide.md).

use async_trait::async_trait;
use futures::Stream;
use radium_abstraction::{
    ChatMessage, MessageContent, Model, ModelError, ModelParameters, ModelResponse, ModelUsage,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tracing::{debug, error};

/// Universal OpenAI-compatible model implementation.
///
/// This model can connect to any server that implements the OpenAI Chat Completions API,
/// enabling support for local models and self-hosted inference servers.
#[derive(Debug, Clone)]
pub struct UniversalModel {
    /// The model identifier (e.g., "llama-3-70b", "mistral-7b").
    model_id: String,
    /// Base URL for the API endpoint (e.g., "http://localhost:8000/v1").
    base_url: String,
    /// Optional API key (some local servers don't require auth).
    api_key: Option<String>,
    /// HTTP client for requests.
    client: Client,
}

impl UniversalModel {
    /// Creates a new `UniversalModel` with the given model ID and base URL.
    ///
    /// The API key will be loaded from the `UNIVERSAL_API_KEY` or `OPENAI_COMPATIBLE_API_KEY`
    /// environment variable.
    ///
    /// # Arguments
    /// * `model_id` - The model identifier (e.g., "llama-3-70b")
    /// * `base_url` - The base URL for the API endpoint (e.g., "http://localhost:8000/v1")
    ///
    /// # Errors
    /// Returns a `ModelError` if neither `UNIVERSAL_API_KEY` nor `OPENAI_COMPATIBLE_API_KEY`
    /// environment variables are set. For servers that don't require authentication,
    /// use `without_auth()` instead.
    #[allow(clippy::disallowed_methods)] // env::var is needed for API key loading
    pub fn new(model_id: String, base_url: String) -> Result<Self, ModelError> {
        // Try UNIVERSAL_API_KEY first, then fall back to OPENAI_COMPATIBLE_API_KEY
        let api_key = env::var("UNIVERSAL_API_KEY")
            .or_else(|_| env::var("OPENAI_COMPATIBLE_API_KEY"))
            .map_err(|_| {
                ModelError::UnsupportedModelProvider(
                    "Neither UNIVERSAL_API_KEY nor OPENAI_COMPATIBLE_API_KEY environment variable is set. \
                     Use without_auth() for servers that don't require authentication.".to_string(),
                )
            })?;

        Ok(Self {
            model_id,
            base_url,
            api_key: Some(api_key),
            client: Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .map_err(|e| {
                    ModelError::RequestError(format!("Failed to create HTTP client: {}", e))
                })?,
        })
    }

    /// Creates a new `UniversalModel` with an explicit API key.
    ///
    /// # Arguments
    /// * `model_id` - The model identifier
    /// * `base_url` - The base URL for the API endpoint
    /// * `api_key` - The API key for authentication
    #[must_use]
    pub fn with_api_key(model_id: String, base_url: String, api_key: String) -> Self {
        Self {
            model_id,
            base_url,
            api_key: Some(api_key),
            client: Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .unwrap_or_else(|_| Client::new()),
        }
    }

    /// Creates a new `UniversalModel` without authentication.
    ///
    /// Use this constructor for local servers that don't require API keys,
    /// such as LM Studio or local vLLM instances without authentication.
    ///
    /// # Arguments
    /// * `model_id` - The model identifier
    /// * `base_url` - The base URL for the API endpoint
    #[must_use]
    pub fn without_auth(model_id: String, base_url: String) -> Self {
        Self {
            model_id,
            base_url,
            api_key: None,
            client: Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .unwrap_or_else(|_| Client::new()),
        }
    }

    /// Converts our ChatMessage role to OpenAI API role format.
    fn role_to_openai(role: &str) -> String {
        match role {
            "assistant" => "assistant".to_string(),
            "system" => "system".to_string(),
            "user" => "user".to_string(),
            _ => role.to_string(),
        }
    }
}

#[async_trait]
impl Model for UniversalModel {
    async fn generate_text(
        &self,
        prompt: &str,
        parameters: Option<ModelParameters>,
    ) -> Result<ModelResponse, ModelError> {
        debug!(
            model_id = %self.model_id,
            prompt_len = prompt.len(),
            parameters = ?parameters,
            "UniversalModel generating text"
        );

        // Convert single prompt to chat format
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
            "UniversalModel generating chat completion"
        );

        // Build OpenAI-compatible API request
        let url = format!("{}/chat/completions", self.base_url);

        // Convert messages to OpenAI format
        // Note: Universal model only supports text content for now
        let openai_messages: Result<Vec<OpenAIMessage>, ModelError> = messages
            .iter()
            .map(|msg| {
                let text = match &msg.content {
                    MessageContent::Text(text) => text.clone(),
                    MessageContent::Blocks(_) => {
                        return Err(ModelError::UnsupportedContentType {
                            content_type: "multimodal blocks".to_string(),
                            model: "universal".to_string(),
                        });
                    }
                };
                Ok(OpenAIMessage {
                    role: Self::role_to_openai(&msg.role),
                    content: text,
                })
            })
            .collect();
        let openai_messages = openai_messages?;

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
        let mut request = self.client.post(&url).json(&request_body);

        // Add Bearer auth if API key is present
        if let Some(ref api_key) = self.api_key {
            request = request.bearer_auth(api_key);
        }

        let response = request.send().await.map_err(|e| {
            error!(
                error = %e,
                url = %url,
                "Failed to send request to OpenAI-compatible API"
            );
            ModelError::RequestError(format!("Network error: {}", e))
        })?;

        // Check status
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                status = %status,
                error = %error_text,
                url = %url,
                "OpenAI-compatible API returned error status"
            );
            
            // Map authentication errors (401, 403) to UnsupportedModelProvider
            if status == 401 || status == 403 {
                return Err(ModelError::UnsupportedModelProvider(format!(
                    "Authentication failed ({}): {}",
                    status, error_text
                )));
            }
            
            // Map quota/rate limit errors (402, 429) to QuotaExceeded
            if status == 402 || status == 429 {
                // Check for quota-related error messages in response body
                let is_quota_error = error_text.to_lowercase().contains("exceeded your current quota")
                    || error_text.to_lowercase().contains("insufficient_quota")
                    || error_text.to_lowercase().contains("quota")
                    || error_text.to_lowercase().contains("rate limit");
                
                if is_quota_error || status == 402 {
                    return Err(ModelError::QuotaExceeded {
                        provider: "universal".to_string(),
                        message: Some(error_text),
                    });
                }
            }
            
            // For 429, treat as QuotaExceeded (rate limit)
            if status == 429 {
                return Err(ModelError::QuotaExceeded {
                    provider: "universal".to_string(),
                    message: Some(error_text),
                });
            }
            
            // Map server errors (500-599) to ModelResponseError
            if (500..=599).contains(&status.as_u16()) {
                return Err(ModelError::ModelResponseError(format!(
                    "Server error ({}): {}",
                    status, error_text
                )));
            }
            
            // Other errors (400, 404, etc.)
            return Err(ModelError::ModelResponseError(format!(
                "API error ({}): {}",
                status, error_text
            )));
        }

        // Parse response
        let openai_response: OpenAIResponse = response.json().await.map_err(|e| {
            error!(
                error = %e,
                url = %url,
                "Failed to parse OpenAI-compatible API response"
            );
            ModelError::SerializationError(format!("Failed to parse response: {}", e))
        })?;

        // Extract content from response
        let content = openai_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| {
                ModelError::ModelResponseError("No content in API response".to_string())
            })?;

        // Extract usage information
        let usage = openai_response.usage.map(|u| ModelUsage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        });

        Ok(ModelResponse {
            content,
            model_id: Some(self.model_id.clone()),
            usage,
            metadata: None,
        })
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }
}

impl UniversalModel {
    /// Generates a chat completion with streaming support (Server-Sent Events).
    ///
    /// This method returns a stream that accumulates tokens as they arrive from the server.
    /// The stream yields the accumulated content after each chunk is received.
    ///
    /// # Arguments
    /// * `messages` - The conversation history as a slice of chat messages
    /// * `parameters` - Optional parameters to control generation
    ///
    /// # Errors
    /// Returns a `ModelError` if the request fails or streaming cannot be established.
    ///
    /// # Example
    /// ```no_run
    /// use futures::StreamExt;
    /// use radium_models::UniversalModel;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let model = UniversalModel::without_auth(
    ///     "llama-2-7b".to_string(),
    ///     "http://localhost:1234/v1".to_string(),
    /// );
    ///
    /// let messages = vec![radium_abstraction::ChatMessage {
    ///     role: "user".to_string(),
    ///     content: "Say hello".to_string(),
    /// }];
    ///
    /// let mut stream = model.generate_chat_completion_stream(&messages, None).await?;
    /// while let Some(result) = stream.next().await {
    ///     let content = result?;
    ///     print!("{}", content);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn generate_chat_completion_stream(
        &self,
        messages: &[ChatMessage],
        parameters: Option<ModelParameters>,
    ) -> Result<impl Stream<Item = Result<String, ModelError>>, ModelError> {
        debug!(
            model_id = %self.model_id,
            message_count = messages.len(),
            parameters = ?parameters,
            "UniversalModel generating streaming chat completion"
        );

        // Build OpenAI-compatible API request
        let url = format!("{}/chat/completions", self.base_url);

        // Convert messages to OpenAI format
        // Note: Universal model only supports text content for now
        let openai_messages: Result<Vec<OpenAIMessage>, ModelError> = messages
            .iter()
            .map(|msg| {
                let text = match &msg.content {
                    MessageContent::Text(text) => text.clone(),
                    MessageContent::Blocks(_) => {
                        return Err(ModelError::UnsupportedContentType {
                            content_type: "multimodal blocks".to_string(),
                            model: "universal".to_string(),
                        });
                    }
                };
                Ok(OpenAIMessage {
                    role: Self::role_to_openai(&msg.role),
                    content: text,
                })
            })
            .collect();
        let openai_messages = openai_messages?;

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
        let mut request = self.client.post(&url).json(&request_body);

        // Add Bearer auth if API key is present
        if let Some(ref api_key) = self.api_key {
            request = request.bearer_auth(api_key);
        }

        let response = request.send().await.map_err(|e| {
            error!(
                error = %e,
                url = %url,
                "Failed to send streaming request to OpenAI-compatible API"
            );
            ModelError::RequestError(format!("Network error: {}", e))
        })?;

        // Check status
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                status = %status,
                error = %error_text,
                url = %url,
                "OpenAI-compatible API returned error status for streaming request"
            );
            
            // Map errors similar to non-streaming
            if status == 401 || status == 403 {
                return Err(ModelError::UnsupportedModelProvider(format!(
                    "Authentication failed ({}): {}",
                    status, error_text
                )));
            }
            
            if status == 402 || status == 429 {
                return Err(ModelError::QuotaExceeded {
                    provider: "universal".to_string(),
                    message: Some(error_text),
                });
            }
            
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

        // Create streaming response parser
        Ok(SSEStream::new(response))
    }
}

// Streaming response parser for SSE format
struct SSEStream {
    stream: Pin<Box<dyn Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send>>,
    buffer: String,
    accumulated: String,
    done: bool,
}

impl SSEStream {
    fn new(response: reqwest::Response) -> Self {
        Self {
            stream: Box::pin(response.bytes_stream()),
            buffer: String::new(),
            accumulated: String::new(),
            done: false,
        }
    }
}

impl Stream for SSEStream {
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
                            
                            // Process complete lines (SSE events are separated by \n\n)
                            while let Some(end_idx) = self.buffer.find("\n\n") {
                                let event = self.buffer[..end_idx].to_string();
                                self.buffer = self.buffer[end_idx + 2..].to_string();
                                
                                // Parse SSE event
                                if event.starts_with("data: ") {
                                    let data = &event[6..]; // Skip "data: " prefix
                                    
                                    // Check for [DONE] signal
                                    if data.trim() == "[DONE]" {
                                        self.done = true;
                                        return Poll::Ready(Some(Ok(self.accumulated.clone())));
                                    }
                                    
                                    // Parse JSON chunk
                                    match serde_json::from_str::<OpenAIStreamingResponse>(data) {
                                        Ok(streaming_response) => {
                                            // Extract delta content
                                            if let Some(choice) = streaming_response.choices.first() {
                                                if let Some(content) = &choice.delta.content {
                                                    self.accumulated.push_str(content);
                                                    return Poll::Ready(Some(Ok(self.accumulated.clone())));
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
                            
                            if let Ok(streaming_response) = serde_json::from_str::<OpenAIStreamingResponse>(data) {
                                if let Some(choice) = streaming_response.choices.first() {
                                    if let Some(content) = &choice.delta.content {
                                        self.accumulated.push_str(content);
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

// OpenAI-compatible API request/response structures
// These match the OpenAI API specification and can be used with any compatible server

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

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Debug, Deserialize)]
#[allow(clippy::struct_field_names)] // Matches API naming
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

// Streaming response structures
#[derive(Debug, Deserialize)]
struct OpenAIStreamingResponse {
    choices: Vec<OpenAIStreamingChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamingChoice {
    delta: OpenAIStreamingDelta,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamingDelta {
    content: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use radium_abstraction::ChatMessage;

    #[test]
    fn test_universal_model_with_api_key() {
        let model = UniversalModel::with_api_key(
            "test-model".to_string(),
            "http://localhost:8000/v1".to_string(),
            "test-key".to_string(),
        );
        assert_eq!(model.model_id(), "test-model");
    }

    #[test]
    fn test_universal_model_without_auth() {
        let model = UniversalModel::without_auth(
            "test-model".to_string(),
            "http://localhost:8000/v1".to_string(),
        );
        assert_eq!(model.model_id(), "test-model");
    }

    #[test]
    fn test_role_to_openai() {
        assert_eq!(UniversalModel::role_to_openai("user"), "user");
        assert_eq!(UniversalModel::role_to_openai("assistant"), "assistant");
        assert_eq!(UniversalModel::role_to_openai("system"), "system");
        assert_eq!(UniversalModel::role_to_openai("custom"), "custom");
    }

    #[tokio::test]
    async fn test_generate_chat_completion_success() {
        let mut _m = mockito::Server::new_async().await;
        let mock_url = _m.url();
        let base_url = format!("{}/v1", mock_url);

        // Mock successful response
        let mock = _m
            .mock("POST", "/v1/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "choices": [{
                    "message": {
                        "role": "assistant",
                        "content": "Hello, world!"
                    }
                }],
                "usage": {
                    "prompt_tokens": 10,
                    "completion_tokens": 20,
                    "total_tokens": 30
                }
            }"#)
            .create();

        let model = UniversalModel::without_auth(
            "test-model".to_string(),
            base_url.clone(),
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Say hello".to_string(),
        }];

        let response = model.generate_chat_completion(&messages, None).await.unwrap();

        assert_eq!(response.content, "Hello, world!");
        assert_eq!(response.model_id, Some("test-model".to_string()));
        assert!(response.usage.is_some());
        let usage = response.usage.unwrap();
        assert_eq!(usage.prompt_tokens, 10);
        assert_eq!(usage.completion_tokens, 20);
        assert_eq!(usage.total_tokens, 30);

        mock.assert();
    }

    #[tokio::test]
    async fn test_generate_chat_completion_with_auth() {
        let mut _m = mockito::Server::new_async().await;
        let mock_url = _m.url();
        let base_url = format!("{}/v1", mock_url);

        // Mock successful response with auth
        let mock = _m
            .mock("POST", "/v1/chat/completions")
            .match_header("authorization", "Bearer test-key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "choices": [{
                    "message": {
                        "role": "assistant",
                        "content": "Authenticated response"
                    }
                }]
            }"#)
            .create();

        let model = UniversalModel::with_api_key(
            "test-model".to_string(),
            base_url,
            "test-key".to_string(),
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Test".to_string(),
        }];

        let response = model.generate_chat_completion(&messages, None).await.unwrap();
        assert_eq!(response.content, "Authenticated response");

        mock.assert();
    }

    #[tokio::test]
    async fn test_generate_chat_completion_error_401() {
        let mut _m = mockito::Server::new_async().await;
        let mock_url = _m.url();
        let base_url = format!("{}/v1", mock_url);

        let mock = _m
            .mock("POST", "/v1/chat/completions")
            .with_status(401)
            .with_body(r#"{"error": "Unauthorized"}"#)
            .create();

        let model = UniversalModel::without_auth(
            "test-model".to_string(),
            base_url,
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Test".to_string(),
        }];

        let result = model.generate_chat_completion(&messages, None).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ModelError::UnsupportedModelProvider(msg) => {
                assert!(msg.contains("Authentication failed"));
            }
            _ => panic!("Expected UnsupportedModelProvider error"),
        }

        mock.assert();
    }

    #[tokio::test]
    async fn test_generate_chat_completion_error_429() {
        let mut _m = mockito::Server::new_async().await;
        let mock_url = _m.url();
        let base_url = format!("{}/v1", mock_url);

        let mock = _m
            .mock("POST", "/v1/chat/completions")
            .with_status(429)
            .with_body(r#"{"error": "Rate limit exceeded"}"#)
            .create();

        let model = UniversalModel::without_auth(
            "test-model".to_string(),
            base_url,
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Test".to_string(),
        }];

        let result = model.generate_chat_completion(&messages, None).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ModelError::QuotaExceeded { provider, .. } => {
                assert_eq!(provider, "universal");
            }
            _ => panic!("Expected QuotaExceeded error"),
        }

        mock.assert();
    }

    #[tokio::test]
    async fn test_generate_chat_completion_error_500() {
        let mut _m = mockito::Server::new_async().await;
        let mock_url = _m.url();
        let base_url = format!("{}/v1", mock_url);

        let mock = _m
            .mock("POST", "/v1/chat/completions")
            .with_status(500)
            .with_body(r#"{"error": "Internal server error"}"#)
            .create();

        let model = UniversalModel::without_auth(
            "test-model".to_string(),
            base_url,
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Test".to_string(),
        }];

        let result = model.generate_chat_completion(&messages, None).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ModelError::ModelResponseError(msg) => {
                assert!(msg.contains("Server error"));
            }
            _ => panic!("Expected ModelResponseError"),
        }

        mock.assert();
    }

    #[tokio::test]
    async fn test_generate_chat_completion_malformed_json() {
        let mut _m = mockito::Server::new_async().await;
        let mock_url = _m.url();
        let base_url = format!("{}/v1", mock_url);

        let mock = _m
            .mock("POST", "/v1/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("not json")
            .create();

        let model = UniversalModel::without_auth(
            "test-model".to_string(),
            base_url,
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Test".to_string(),
        }];

        let result = model.generate_chat_completion(&messages, None).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ModelError::SerializationError(_) => {}
            _ => panic!("Expected SerializationError"),
        }

        mock.assert();
    }

    #[tokio::test]
    async fn test_generate_chat_completion_empty_choices() {
        let mut _m = mockito::Server::new_async().await;
        let mock_url = _m.url();
        let base_url = format!("{}/v1", mock_url);

        let mock = _m
            .mock("POST", "/v1/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"choices": []}"#)
            .create();

        let model = UniversalModel::without_auth(
            "test-model".to_string(),
            base_url,
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Test".to_string(),
        }];

        let result = model.generate_chat_completion(&messages, None).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ModelError::ModelResponseError(msg) => {
                assert!(msg.contains("No content"));
            }
            _ => panic!("Expected ModelResponseError"),
        }

        mock.assert();
    }

    #[tokio::test]
    async fn test_generate_text() {
        let mut _m = mockito::Server::new_async().await;
        let mock_url = _m.url();
        let base_url = format!("{}/v1", mock_url);

        let mock = _m
            .mock("POST", "/v1/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "choices": [{
                    "message": {
                        "role": "assistant",
                        "content": "Generated text"
                    }
                }]
            }"#)
            .create();

        let model = UniversalModel::without_auth(
            "test-model".to_string(),
            base_url,
        );

        let response = model.generate_text("Prompt", None).await.unwrap();
        assert_eq!(response.content, "Generated text");

        mock.assert();
    }

    #[tokio::test]
    async fn test_generate_chat_completion_with_parameters() {
        let mut _m = mockito::Server::new_async().await;
        let mock_url = _m.url();
        let base_url = format!("{}/v1", mock_url);

        let mock = _m
            .mock("POST", "/v1/chat/completions")
            .match_body(mockito::Matcher::PartialJsonString(
                r#"{"model":"test-model","messages":[{"role":"user","content":"Test"}],"temperature":0.7,"max_tokens":100}"#.to_string(),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "choices": [{
                    "message": {
                        "role": "assistant",
                        "content": "Response"
                    }
                }]
            }"#)
            .create();

        let model = UniversalModel::without_auth(
            "test-model".to_string(),
            base_url,
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Test".to_string(),
        }];

        let params = ModelParameters {
            temperature: Some(0.7),
            max_tokens: Some(100),
            top_p: None,
            top_k: None,
            frequency_penalty: None,
            presence_penalty: None,
            response_format: None,
            stop_sequences: None,
        };

        let _response = model.generate_chat_completion(&messages, Some(params)).await.unwrap();

        mock.assert();
    }

    #[test]
    #[allow(clippy::disallowed_methods, unsafe_code)]
    fn test_universal_model_new_with_env_var() {
        unsafe {
            std::env::set_var("UNIVERSAL_API_KEY", "test-env-key");
        }
        let model = UniversalModel::new(
            "test-model".to_string(),
            "http://localhost:8000/v1".to_string(),
        );
        assert!(model.is_ok());
        let model = model.unwrap();
        assert_eq!(model.model_id(), "test-model");
        unsafe {
            std::env::remove_var("UNIVERSAL_API_KEY");
        }
    }

    #[test]
    #[allow(clippy::disallowed_methods, unsafe_code)]
    fn test_universal_model_new_without_env_var() {
        unsafe {
            std::env::remove_var("UNIVERSAL_API_KEY");
            std::env::remove_var("OPENAI_COMPATIBLE_API_KEY");
        }
        let model = UniversalModel::new(
            "test-model".to_string(),
            "http://localhost:8000/v1".to_string(),
        );
        assert!(model.is_err());
    }

    #[test]
    #[allow(clippy::disallowed_methods, unsafe_code)]
    fn test_universal_model_new_fallback_to_openai_compatible_key() {
        unsafe {
            std::env::remove_var("UNIVERSAL_API_KEY");
            std::env::set_var("OPENAI_COMPATIBLE_API_KEY", "fallback-key");
        }
        let model = UniversalModel::new(
            "test-model".to_string(),
            "http://localhost:8000/v1".to_string(),
        );
        assert!(model.is_ok());
        unsafe {
            std::env::remove_var("OPENAI_COMPATIBLE_API_KEY");
        }
    }

    #[tokio::test]
    async fn test_generate_chat_completion_error_403() {
        let mut _m = mockito::Server::new_async().await;
        let mock_url = _m.url();
        let base_url = format!("{}/v1", mock_url);

        let mock = _m
            .mock("POST", "/v1/chat/completions")
            .with_status(403)
            .with_body(r#"{"error": "Forbidden"}"#)
            .create();

        let model = UniversalModel::without_auth(
            "test-model".to_string(),
            base_url,
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Test".to_string(),
        }];

        let result = model.generate_chat_completion(&messages, None).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ModelError::UnsupportedModelProvider(msg) => {
                assert!(msg.contains("Authentication failed"));
            }
            _ => panic!("Expected UnsupportedModelProvider error"),
        }

        mock.assert();
    }

    #[tokio::test]
    async fn test_generate_chat_completion_error_402() {
        let mut _m = mockito::Server::new_async().await;
        let mock_url = _m.url();
        let base_url = format!("{}/v1", mock_url);

        let mock = _m
            .mock("POST", "/v1/chat/completions")
            .with_status(402)
            .with_body(r#"{"error": "Insufficient quota"}"#)
            .create();

        let model = UniversalModel::without_auth(
            "test-model".to_string(),
            base_url,
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Test".to_string(),
        }];

        let result = model.generate_chat_completion(&messages, None).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ModelError::QuotaExceeded { provider, .. } => {
                assert_eq!(provider, "universal");
            }
            _ => panic!("Expected QuotaExceeded error"),
        }

        mock.assert();
    }

    #[tokio::test]
    async fn test_generate_chat_completion_missing_usage() {
        let mut _m = mockito::Server::new_async().await;
        let mock_url = _m.url();
        let base_url = format!("{}/v1", mock_url);

        let mock = _m
            .mock("POST", "/v1/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "choices": [{
                    "message": {
                        "role": "assistant",
                        "content": "Response without usage"
                    }
                }]
            }"#)
            .create();

        let model = UniversalModel::without_auth(
            "test-model".to_string(),
            base_url,
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }];

        let response = model.generate_chat_completion(&messages, None).await.unwrap();

        // Should succeed even without usage info
        assert_eq!(response.content, "Response without usage");
        assert!(response.usage.is_none());
        mock.assert();
    }

    #[tokio::test]
    async fn test_generate_chat_completion_empty_content() {
        let mut _m = mockito::Server::new_async().await;
        let mock_url = _m.url();
        let base_url = format!("{}/v1", mock_url);

        let mock = _m
            .mock("POST", "/v1/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "choices": [{
                    "message": {
                        "role": "assistant",
                        "content": ""
                    }
                }],
                "usage": {
                    "prompt_tokens": 5,
                    "completion_tokens": 0,
                    "total_tokens": 5
                }
            }"#)
            .create();

        let model = UniversalModel::without_auth(
            "test-model".to_string(),
            base_url,
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }];

        let response = model.generate_chat_completion(&messages, None).await.unwrap();

        assert_eq!(response.content, "");
        mock.assert();
    }

    #[tokio::test]
    async fn test_generate_chat_completion_multiple_messages() {
        let mut _m = mockito::Server::new_async().await;
        let mock_url = _m.url();
        let base_url = format!("{}/v1", mock_url);

        let mock = _m
            .mock("POST", "/v1/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "choices": [{
                    "message": {
                        "role": "assistant",
                        "content": "Response to conversation"
                    }
                }],
                "usage": {
                    "prompt_tokens": 20,
                    "completion_tokens": 5,
                    "total_tokens": 25
                }
            }"#)
            .create();

        let model = UniversalModel::without_auth(
            "test-model".to_string(),
            base_url,
        );

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are helpful".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            },
            ChatMessage {
                role: "assistant".to_string(),
                content: "Hi!".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "How are you?".to_string(),
            },
        ];

        let response = model.generate_chat_completion(&messages, None).await.unwrap();

        assert_eq!(response.content, "Response to conversation");
        assert_eq!(response.usage.unwrap().total_tokens, 25);
        mock.assert();
    }

    // Streaming tests
    #[tokio::test]
    async fn test_streaming_success() {
        use futures::StreamExt;

        let mut _m = mockito::Server::new_async().await;
        let mock_url = _m.url();
        let base_url = format!("{}/v1", mock_url);

        // SSE format: data: <json>\n\n
        let mock_response = b"data: {\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}\n\ndata: {\"choices\":[{\"delta\":{\"content\":\" world\"}}]}\n\ndata: {\"choices\":[{\"delta\":{\"content\":\"!\"}}]}\n\ndata: [DONE]\n\n";

        let mock = _m
            .mock("POST", "/v1/chat/completions")
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body(mock_response)
            .create();

        let model = UniversalModel::without_auth(
            "test-model".to_string(),
            base_url,
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Say hello".to_string(),
        }];

        let mut stream = model
            .generate_chat_completion_stream(&messages, None)
            .await
            .unwrap();

        let mut all_contents = Vec::new();
        while let Some(result) = stream.next().await {
            let content = result.unwrap();
            all_contents.push(content);
        }

        // The stream should yield accumulated content after each chunk
        // The last value should be the complete accumulated string
        assert!(!all_contents.is_empty(), "Stream should yield at least one value");
        let final_content = all_contents.last().unwrap();
        assert_eq!(final_content, "Hello world!");
        mock.assert();
    }

    #[tokio::test]
    async fn test_streaming_with_done_signal() {
        use futures::StreamExt;

        let mut _m = mockito::Server::new_async().await;
        let mock_url = _m.url();
        let base_url = format!("{}/v1", mock_url);

        let mock_response = b"data: {\"choices\":[{\"delta\":{\"content\":\"Test\"}}]}\n\ndata: [DONE]\n\n";

        let mock = _m
            .mock("POST", "/v1/chat/completions")
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body(mock_response)
            .create();

        let model = UniversalModel::without_auth(
            "test-model".to_string(),
            base_url,
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Test".to_string(),
        }];

        let mut stream = model
            .generate_chat_completion_stream(&messages, None)
            .await
            .unwrap();

        let mut last_content = String::new();
        while let Some(result) = stream.next().await {
            let content = result.unwrap();
            last_content = content;
        }

        assert_eq!(last_content, "Test");
        mock.assert();
    }

    #[tokio::test]
    async fn test_streaming_error_handling() {
        use futures::StreamExt;

        let mut _m = mockito::Server::new_async().await;
        let mock_url = _m.url();
        let base_url = format!("{}/v1", mock_url);

        // First chunk valid, second invalid JSON
        let mock_response = b"data: {\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}\n\ndata: invalid json\n\n";

        let mock = _m
            .mock("POST", "/v1/chat/completions")
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body(mock_response)
            .create();

        let model = UniversalModel::without_auth(
            "test-model".to_string(),
            base_url,
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }];

        let mut stream = model
            .generate_chat_completion_stream(&messages, None)
            .await
            .unwrap();

        // First chunk should succeed
        let first = stream.next().await;
        assert!(first.is_some());
        assert!(first.unwrap().is_ok());

        // Second chunk should be skipped (malformed JSON)
        // Stream should continue or end gracefully
        let _second = stream.next().await;
        // Implementation skips malformed chunks, so this is acceptable
        mock.assert();
    }

    #[tokio::test]
    async fn test_streaming_authentication_error() {
        let mut _m = mockito::Server::new_async().await;
        let mock_url = _m.url();
        let base_url = format!("{}/v1", mock_url);

        let mock_response = r#"{"error": {"message": "Unauthorized"}}"#;

        let mock = _m
            .mock("POST", "/v1/chat/completions")
            .with_status(401)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create();

        let model = UniversalModel::without_auth(
            "test-model".to_string(),
            base_url,
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }];

        let result = model.generate_chat_completion_stream(&messages, None).await;

        assert!(result.is_err());
        if let Err(ModelError::UnsupportedModelProvider(msg)) = result {
            assert!(msg.contains("Authentication failed"));
        } else {
            panic!("Expected UnsupportedModelProvider error");
        }
        mock.assert();
    }
}

