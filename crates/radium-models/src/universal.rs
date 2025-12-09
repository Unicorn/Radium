//! Universal OpenAI-compatible model implementation.
//!
//! This module provides an implementation of the `Model` trait for any server that implements
//! the OpenAI API specification, including:
//! - vLLM: High-performance LLM inference server
//! - LocalAI: Local inference server with OpenAI-compatible API
//! - LM Studio: Desktop app for running local models
//! - Ollama: Local model runner with OpenAI-compatible endpoints
//!
//! # Examples
//!
//! ## vLLM
//! ```no_run
//! use radium_models::UniversalModel;
//!
//! let model = UniversalModel::new(
//!     "meta-llama/Llama-3-70b".to_string(),
//!     "http://localhost:8000/v1".to_string(),
//! )?;
//! # Ok::<(), radium_abstraction::ModelError>(())
//! ```
//!
//! ## LocalAI with authentication
//! ```no_run
//! use radium_models::UniversalModel;
//!
//! let model = UniversalModel::with_api_key(
//!     "gpt-3.5-turbo".to_string(),
//!     "http://localhost:8080/v1".to_string(),
//!     "local-api-key".to_string(),
//! );
//! ```
//!
//! ## LM Studio (no authentication)
//! ```no_run
//! use radium_models::UniversalModel;
//!
//! let model = UniversalModel::without_auth(
//!     "llama-2-7b".to_string(),
//!     "http://localhost:1234/v1".to_string(),
//! );
//! ```

use async_trait::async_trait;
use radium_abstraction::{
    ChatMessage, Model, ModelError, ModelParameters, ModelResponse, ModelUsage,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;
use tracing::debug;

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
            content: prompt.to_string(),
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
        let mut request = self.client.post(&url).json(&request_body);

        // Add Bearer auth if API key is present
        if let Some(ref api_key) = self.api_key {
            request = request.bearer_auth(api_key);
        }

        let response = request.send().await.map_err(|e| {
            ModelError::RequestError(format!("Network error: {}", e))
        })?;

        // Check status
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            
            // Map quota/rate limit errors to QuotaExceeded
            if status == 402 || status == 429 {
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
            
            // For 429, treat as QuotaExceeded
            if status == 429 {
                return Err(ModelError::QuotaExceeded {
                    provider: "universal".to_string(),
                    message: Some(error_text),
                });
            }
            
            // Other errors
            return Err(ModelError::ModelResponseError(format!(
                "API error ({}): {}",
                status, error_text
            )));
        }

        // Parse response
        let openai_response: OpenAIResponse = response.json().await.map_err(|e| {
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
        })
    }

    fn model_id(&self) -> &str {
        &self.model_id
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_universal_model_with_api_key() {
        let model = UniversalModel::with_api_key(
            "test-model".to_string(),
            "http://localhost:8000/v1".to_string(),
            "test-key".to_string(),
        );
        assert_eq!(model.model_id, "test-model");
        assert_eq!(model.base_url, "http://localhost:8000/v1");
        assert_eq!(model.api_key, Some("test-key".to_string()));
    }

    #[test]
    fn test_universal_model_without_auth() {
        let model = UniversalModel::without_auth(
            "test-model".to_string(),
            "http://localhost:8000/v1".to_string(),
        );
        assert_eq!(model.model_id, "test-model");
        assert_eq!(model.base_url, "http://localhost:8000/v1");
        assert_eq!(model.api_key, None);
    }
}

