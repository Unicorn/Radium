//! Model abstraction layer for Radium.
//!
//! This module defines the core traits and types for interacting with AI models.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
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

    /// Up to 4 sequences where the API will stop generating further tokens.
    pub stop_sequences: Option<Vec<String>>,
}

impl Default for ModelParameters {
    fn default() -> Self {
        Self {
            temperature: Some(0.7),
            top_p: Some(1.0),
            max_tokens: Some(512),
            stop_sequences: None,
        }
    }
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
