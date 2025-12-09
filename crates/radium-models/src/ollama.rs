//! Ollama model implementation.
//!
//! This module provides an implementation of the `Model` trait for Ollama's local API.

use async_trait::async_trait;
use radium_abstraction::{
    ChatMessage, Model, ModelError, ModelParameters, ModelResponse, ModelUsage,
};
use reqwest::Client;

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

#[async_trait]
impl Model for OllamaModel {
    /// Model trait implementation will be completed in Task 4.
    async fn generate_text(
        &self,
        _prompt: &str,
        _parameters: Option<ModelParameters>,
    ) -> Result<ModelResponse, ModelError> {
        Err(ModelError::UnsupportedModelProvider(
            "OllamaModel::generate_text() will be implemented in Task 4".to_string(),
        ))
    }

    /// Model trait implementation will be completed in Task 4.
    async fn generate_chat_completion(
        &self,
        _messages: &[ChatMessage],
        _parameters: Option<ModelParameters>,
    ) -> Result<ModelResponse, ModelError> {
        Err(ModelError::UnsupportedModelProvider(
            "OllamaModel::generate_chat_completion() will be implemented in Task 4".to_string(),
        ))
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

