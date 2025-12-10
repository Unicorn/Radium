//! Model abstraction layer for Radium.
//!
//! This module defines the core traits and types for interacting with AI models.

use async_trait::async_trait;
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use thiserror::Error;

/// Represents an error that can occur when interacting with an AI model.
#[derive(Error, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelError {
    /// An error occurred during the API request (e.g., network issues, invalid request).
    #[error("Request Error: {0}")]
    RequestError(String),

    /// The model returned an error (e.g., invalid input, rate limiting).
    #[error("Model Response Error: {0}")]
    ModelResponseError(String),

    /// An error occurred during serialization or deserialization.
    #[error("Serialization Error: {0}")]
    SerializationError(String),

    /// The model provider is not supported or configured.
    #[error("Unsupported Model Provider: {0}")]
    UnsupportedModelProvider(String),

    /// Provider quota exceeded or rate limit hit (hard stop error).
    #[error("Provider '{provider}' quota exceeded{}", message.as_ref().map(|m| format!(": {}", m)).unwrap_or_default())]
    QuotaExceeded {
        /// The provider name (e.g., "openai", "gemini").
        provider: String,
        /// Optional error message from the provider.
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },

    /// Other unexpected errors.
    #[error("Other Model Error: {0}")]
    Other(String),
}

impl ModelError {
    /// Formats the message part for QuotaExceeded error display.
    /// This method is used by the thiserror Display implementation.
    #[allow(dead_code)] // Used by thiserror macro via format string
    fn message_formatted(&self) -> String {
        match self {
            Self::QuotaExceeded { message, .. } => {
                message.as_ref().map(|m| format!(": {}", m)).unwrap_or_default()
            }
            _ => String::new(),
        }
    }
}

/// Represents a message in a conversation with a chat model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    /// The role of the message sender (e.g., "user", "assistant", "system").
    pub role: String,
    /// The content of the message.
    pub content: String,
}

/// Parameters for controlling the model's generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelParameters {
    /// What sampling temperature to use, between 0 and 2.
    /// Higher values mean the model will take more risks.
    pub temperature: Option<f32>,

    /// An alternative to sampling with temperature, called nucleus sampling,
    /// where the model considers the results of the tokens with `top_p` probability mass.
    pub top_p: Option<f32>,

    /// The maximum number of tokens to generate in the chat completion.
    pub max_tokens: Option<u32>,

    /// The number of highest probability tokens to consider for sampling.
    /// Valid range: 1-100 (provider-specific limits apply).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,

    /// Reduces likelihood of repeating tokens based on frequency.
    /// Valid range: -2.0 to 2.0 (provider-specific).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,

    /// Reduces likelihood of repeating tokens regardless of frequency.
    /// Valid range: -2.0 to 2.0 (provider-specific).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,

    /// Format for model response output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,

    /// Up to 4 sequences where the API will stop generating further tokens.
    pub stop_sequences: Option<Vec<String>>,
}

impl Default for ModelParameters {
    fn default() -> Self {
        Self {
            temperature: Some(0.7),
            top_p: Some(1.0),
            max_tokens: Some(512),
            top_k: None,
            frequency_penalty: None,
            presence_penalty: None,
            response_format: None,
            stop_sequences: None,
        }
    }
}

/// Format for model response output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseFormat {
    /// Plain text output (default).
    Text,
    /// JSON-formatted output without schema validation.
    Json,
    /// JSON output conforming to the provided schema.
    JsonSchema(String),
}

/// The response from a text generation or chat completion model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResponse {
    /// The generated content.
    pub content: String,

    /// Optional: The ID of the model used to generate the response.
    pub model_id: Option<String>,

    /// Optional: Usage statistics for the request.
    pub usage: Option<ModelUsage>,
}

/// Usage statistics for a model request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsage {
    /// Number of tokens in the prompt.
    pub prompt_tokens: u32,

    /// Number of tokens in the completion.
    pub completion_tokens: u32,

    /// Total number of tokens used.
    pub total_tokens: u32,
}

/// A trait for interacting with different AI models.
///
/// All models must be `Send + Sync` to allow concurrent use across threads.
#[async_trait]
pub trait Model: Send + Sync {
    /// Generates a text completion based on the given prompt.
    ///
    /// # Arguments
    /// * `prompt` - The input prompt for text generation
    /// * `parameters` - Optional parameters to control generation
    ///
    /// # Errors
    /// Returns a `ModelError` if generation fails.
    async fn generate_text(
        &self,
        prompt: &str,
        parameters: Option<ModelParameters>,
    ) -> Result<ModelResponse, ModelError>;

    /// Generates a chat completion based on the given conversation history.
    ///
    /// # Arguments
    /// * `messages` - The conversation history as a slice of chat messages
    /// * `parameters` - Optional parameters to control generation
    ///
    /// # Errors
    /// Returns a `ModelError` if generation fails.
    async fn generate_chat_completion(
        &self,
        messages: &[ChatMessage],
        parameters: Option<ModelParameters>,
    ) -> Result<ModelResponse, ModelError>;

    /// Returns the ID of the model.
    fn model_id(&self) -> &str;
}

/// A trait for models that support streaming text generation.
///
/// This trait enables real-time token-by-token streaming of model responses,
/// allowing consumers to display output as it's generated rather than waiting
/// for the complete response.
///
/// # When to Use Streaming vs Non-Streaming
///
/// - **Use `StreamingModel::generate_stream()`** when you need real-time output
///   display (e.g., in a TUI or CLI where users should see tokens as they arrive)
/// - **Use `Model::generate_text()`** when you need the complete response before
///   processing (e.g., for batch processing or when the full text is required)
///
/// # Consuming the Stream
///
/// The stream yields `Result<String, ModelError>` items where each `String` is
/// a token or chunk of the response. Errors within the stream indicate issues
/// during generation (e.g., network interruptions, API errors).
///
/// ```rust,no_run
/// use radium_abstraction::StreamingModel;
/// use futures::StreamExt;
///
/// # async fn example(model: impl StreamingModel) -> Result<(), Box<dyn std::error::Error>> {
/// let mut stream = model.generate_stream("Hello", None).await?;
/// while let Some(token_result) = stream.next().await {
///     match token_result {
///         Ok(token) => print!("{}", token),
///         Err(e) => eprintln!("Error: {}", e),
///     }
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Error Handling
///
/// Errors can occur at two points:
/// 1. **Stream creation**: Returns `ModelError` if the request cannot be initiated
/// 2. **During streaming**: Individual `Result` items in the stream may contain errors
///
/// Consumers should handle both cases appropriately.
#[async_trait]
pub trait StreamingModel: Send + Sync {
    /// Generates a streaming text completion based on the given prompt.
    ///
    /// Returns an async stream that yields tokens as they're generated by the model.
    /// Each item in the stream is a `Result<String, ModelError>` where:
    /// - `Ok(token)` contains a token or chunk of the response
    /// - `Err(error)` indicates an error during generation
    ///
    /// # Arguments
    /// * `prompt` - The input prompt for text generation
    /// * `parameters` - Optional parameters to control generation
    ///
    /// # Returns
    /// A pinned, boxed stream of token results. The stream must be `Send` to allow
    /// use across async boundaries.
    ///
    /// # Errors
    /// Returns a `ModelError` if the stream cannot be created (e.g., connection
    /// failure, invalid request). Errors during streaming are yielded as items
    /// in the stream itself.
    async fn generate_stream(
        &self,
        prompt: &str,
        parameters: Option<ModelParameters>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, ModelError>> + Send>>, ModelError>;
}
