//! OpenAI model implementation.
//!
//! This module provides an implementation of the `Model` trait for OpenAI's API.

use async_trait::async_trait;
use radium_abstraction::{
    ChatMessage, Model, ModelError, ModelParameters, ModelResponse, ModelUsage,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{debug, error};

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
        let content =
            openai_response.choices.first().map(|c| c.message.content.clone()).ok_or_else(
                || {
                    error!("No content in OpenAI API response");
                    ModelError::ModelResponseError("No content in API response".to_string())
                },
            )?;

        // Extract usage information
        let usage = openai_response.usage.map(|u| ModelUsage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        });

        Ok(ModelResponse { content, model_id: Some(self.model_id.clone()), usage })
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }
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
    fn test_role_conversion() {
        assert_eq!(OpenAIModel::role_to_openai("user"), "user");
        assert_eq!(OpenAIModel::role_to_openai("assistant"), "assistant");
        assert_eq!(OpenAIModel::role_to_openai("system"), "system");
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
