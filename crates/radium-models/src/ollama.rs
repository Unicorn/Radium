//! Ollama model implementation.
//!
//! This module provides an implementation of the `Model` trait for Ollama's local API.

use async_trait::async_trait;
use radium_abstraction::{
    ChatMessage, Model, ModelError, ModelParameters, ModelResponse, ModelUsage,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

/// Ollama model implementation.
#[derive(Debug, Clone)]
pub struct OllamaModel {
    /// The model ID (e.g., "llama2", "codellama:13b").
    model_id: String,
    /// The base URL for the Ollama API (default: "http://localhost:11434").
    base_url: String,
    /// HTTP client for making requests.
    client: Client,
}

impl OllamaModel {
    /// Creates a new `OllamaModel` with the given model ID.
    ///
    /// Uses the default Ollama server URL: `http://localhost:11434`
    ///
    /// # Arguments
    /// * `model_id` - The Ollama model ID to use (e.g., "llama2")
    ///
    /// # Errors
    /// Returns a `ModelError` if the HTTP client cannot be created.
    pub fn new(model_id: String) -> Result<Self, ModelError> {
        Self::with_base_url(model_id, "http://localhost:11434".to_string())
    }

    /// Creates a new `OllamaModel` with a custom base URL.
    ///
    /// # Arguments
    /// * `model_id` - The Ollama model ID to use
    /// * `base_url` - The base URL for the Ollama API (e.g., "http://192.168.1.100:11434")
    ///
    /// # Errors
    /// Returns a `ModelError` if the HTTP client cannot be created.
    pub fn with_base_url(model_id: String, base_url: String) -> Result<Self, ModelError> {
        let client = Client::new();
        Ok(Self {
            model_id,
            base_url,
            client,
        })
    }
}

// Ollama API request/response structures
#[derive(Serialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<u32>, // max_tokens equivalent
}

#[derive(Serialize, Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
    done: bool,
    #[serde(rename = "prompt_eval_count")]
    prompt_eval_count: Option<u32>,
    #[serde(rename = "eval_count")]
    eval_count: Option<u32>,
    #[serde(skip)]
    error: Option<String>,
}

#[derive(Deserialize)]
struct OllamaError {
    error: String,
}

impl OllamaModel {
    /// Build OllamaOptions from ModelParameters
    fn build_options(parameters: Option<ModelParameters>) -> Option<OllamaOptions> {
        parameters.map(|p| OllamaOptions {
            temperature: p.temperature,
            num_predict: p.max_tokens,
        })
    }

    /// Convert ChatMessage to OllamaMessage format
    fn to_ollama_message(msg: &ChatMessage) -> OllamaMessage {
        OllamaMessage {
            role: msg.role.clone(),
            content: msg.content.clone(),
        }
    }
}

#[async_trait]
impl Model for OllamaModel {
    async fn generate_text(
        &self,
        prompt: &str,
        parameters: Option<ModelParameters>,
    ) -> Result<ModelResponse, ModelError> {
        debug!(
            model_id = %self.model_id,
            prompt_len = prompt.len(),
            parameters = ?parameters,
            "OllamaModel generating text"
        );

        let url = format!("{}/api/generate", self.base_url);

        let request_body = OllamaGenerateRequest {
            model: self.model_id.clone(),
            prompt: prompt.to_string(),
            stream: false,
            options: Self::build_options(parameters),
        };

        // Make API request
        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, base_url = %self.base_url, "Failed to connect to Ollama");
                if e.is_connect() {
                    ModelError::RequestError(format!(
                        "Ollama server not reachable at {}. Start it with 'ollama serve'.",
                        self.base_url
                    ))
                } else {
                    ModelError::RequestError(format!("Network error: {}", e))
                }
            })?;

        // Check status
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                status = %status,
                error = %error_text,
                "Ollama API returned error status"
            );

            // Try to parse error JSON
            if let Ok(error_json) = serde_json::from_str::<OllamaError>(&error_text) {
                if error_json.error.contains("model") && error_json.error.contains("not found") {
                    return Err(ModelError::ModelResponseError(format!(
                        "Model '{}' not found. Pull it with 'ollama pull {}'.",
                        self.model_id, self.model_id
                    )));
                }
                if error_json.error.contains("out of memory") || error_json.error.contains("OOM") {
                    return Err(ModelError::ModelResponseError(
                        "Insufficient memory to load model. Try a smaller variant.".to_string(),
                    ));
                }
            }

            if status == 404 {
                return Err(ModelError::ModelResponseError(format!(
                    "Model '{}' not found. Pull it with 'ollama pull {}'.",
                    self.model_id, self.model_id
                )));
            }

            return Err(ModelError::ModelResponseError(format!(
                "API error ({}): {}",
                status, error_text
            )));
        }

        // Parse response
        let ollama_response: OllamaResponse = response.json().await.map_err(|e| {
            error!(error = %e, "Failed to parse Ollama API response");
            ModelError::SerializationError(format!("Failed to parse response: {}", e))
        })?;

        // Extract usage information
        let prompt_tokens = ollama_response.prompt_eval_count.unwrap_or(0);
        let completion_tokens = ollama_response.eval_count.unwrap_or(0);
        let total_tokens = prompt_tokens + completion_tokens;

        Ok(ModelResponse {
            content: ollama_response.response,
            model_id: Some(self.model_id.clone()),
            usage: Some(ModelUsage {
                prompt_tokens,
                completion_tokens,
                total_tokens,
            }),
        })
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
            "OllamaModel generating chat completion"
        );

        let url = format!("{}/api/chat", self.base_url);

        // Convert messages to Ollama format
        let ollama_messages: Vec<OllamaMessage> = messages
            .iter()
            .map(Self::to_ollama_message)
            .collect();

        let request_body = OllamaChatRequest {
            model: self.model_id.clone(),
            messages: ollama_messages,
            stream: false,
            options: Self::build_options(parameters),
        };

        // Make API request
        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, base_url = %self.base_url, "Failed to connect to Ollama");
                if e.is_connect() {
                    ModelError::RequestError(format!(
                        "Ollama server not reachable at {}. Start it with 'ollama serve'.",
                        self.base_url
                    ))
                } else {
                    ModelError::RequestError(format!("Network error: {}", e))
                }
            })?;

        // Check status
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                status = %status,
                error = %error_text,
                "Ollama API returned error status"
            );

            // Try to parse error JSON
            if let Ok(error_json) = serde_json::from_str::<OllamaError>(&error_text) {
                if error_json.error.contains("model") && error_json.error.contains("not found") {
                    return Err(ModelError::ModelResponseError(format!(
                        "Model '{}' not found. Pull it with 'ollama pull {}'.",
                        self.model_id, self.model_id
                    )));
                }
                if error_json.error.contains("out of memory") || error_json.error.contains("OOM") {
                    return Err(ModelError::ModelResponseError(
                        "Insufficient memory to load model. Try a smaller variant.".to_string(),
                    ));
                }
            }

            if status == 404 {
                return Err(ModelError::ModelResponseError(format!(
                    "Model '{}' not found. Pull it with 'ollama pull {}'.",
                    self.model_id, self.model_id
                )));
            }

            return Err(ModelError::ModelResponseError(format!(
                "API error ({}): {}",
                status, error_text
            )));
        }

        // Parse response
        let ollama_response: OllamaResponse = response.json().await.map_err(|e| {
            error!(error = %e, "Failed to parse Ollama API response");
            ModelError::SerializationError(format!("Failed to parse response: {}", e))
        })?;

        // Extract usage information
        let prompt_tokens = ollama_response.prompt_eval_count.unwrap_or(0);
        let completion_tokens = ollama_response.eval_count.unwrap_or(0);
        let total_tokens = prompt_tokens + completion_tokens;

        Ok(ModelResponse {
            content: ollama_response.response,
            model_id: Some(self.model_id.clone()),
            usage: Some(ModelUsage {
                prompt_tokens,
                completion_tokens,
                total_tokens,
            }),
        })
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_model_new() {
        let model = OllamaModel::new("llama2".to_string()).unwrap();
        assert_eq!(model.model_id(), "llama2");
    }

    #[test]
    fn test_ollama_model_with_base_url() {
        let model = OllamaModel::with_base_url(
            "llama2".to_string(),
            "http://192.168.1.100:11434".to_string(),
        )
        .unwrap();
        assert_eq!(model.model_id(), "llama2");
    }
}

