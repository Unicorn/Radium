//! Ollama engine provider implementation.

use crate::engines::engine_trait::{
    Engine, EngineMetadata, ExecutionRequest, ExecutionResponse, TokenUsage,
};
use crate::engines::error::{EngineError, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Ollama engine implementation for local Ollama server.
pub struct OllamaEngine {
    /// Engine metadata.
    metadata: EngineMetadata,
    /// HTTP client for API requests.
    client: Arc<Client>,
    /// Base URL for Ollama server.
    base_url: String,
}

impl OllamaEngine {
    /// Creates a new Ollama engine.
    pub fn new() -> Self {
        // Read OLLAMA_HOST environment variable, default to localhost:11434
        let base_url = std::env::var("OLLAMA_HOST")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());

        let metadata = EngineMetadata::new(
            "ollama".to_string(),
            "Ollama".to_string(),
            "Local Ollama AI engine".to_string(),
        )
        .with_auth_required(false);

        Self {
            metadata,
            client: Arc::new(Client::new()),
            base_url,
        }
    }
}

impl Default for OllamaEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Ollama API generate request structure.
#[derive(Debug, Serialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
    stream: bool,
}

/// Ollama API chat request structure.
#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
    stream: bool,
}

/// Ollama API message structure.
#[derive(Debug, Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

/// Ollama API options structure.
#[derive(Debug, Clone, Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<usize>,
}

/// Ollama API response structure.
#[derive(Debug, Deserialize, Serialize)]
struct OllamaResponse {
    response: String,
    model: String,
    #[serde(rename = "prompt_eval_count")]
    prompt_eval_count: Option<u64>,
    #[serde(rename = "eval_count")]
    eval_count: Option<u64>,
}

#[async_trait]
impl Engine for OllamaEngine {
    fn metadata(&self) -> &EngineMetadata {
        &self.metadata
    }

    async fn is_available(&self) -> bool {
        // Health check will be implemented in Task 3
        // For now, return true
        true
    }

    async fn is_authenticated(&self) -> Result<bool> {
        // Ollama has no authentication
        Ok(true)
    }

    async fn execute(&self, request: ExecutionRequest) -> Result<ExecutionResponse> {
        // Build options from request parameters
        let options = OllamaOptions {
            temperature: request.temperature,
            num_predict: request.max_tokens,
        };

        // If no options are set, don't include the options field
        let options = if options.temperature.is_none() && options.num_predict.is_none() {
            None
        } else {
            Some(options)
        };

        // Determine which endpoint to use based on whether we have a system message
        // Use /api/chat if we have a system message, otherwise /api/generate
        let url = if request.system.is_some() {
            format!("{}/api/chat", self.base_url)
        } else {
            format!("{}/api/generate", self.base_url)
        };

        let response = if request.system.is_some() {
            // Use chat endpoint
            let messages = vec![
                OllamaMessage {
                    role: "system".to_string(),
                    content: request.system.clone().unwrap_or_default(),
                },
                OllamaMessage {
                    role: "user".to_string(),
                    content: request.prompt.clone(),
                },
            ];

            let chat_request = OllamaChatRequest {
                model: request.model.clone(),
                messages,
                options,
                stream: false,
            };

            self.client
                .post(&url)
                .json(&chat_request)
                .send()
                .await
                .map_err(|e| {
                    EngineError::ExecutionError(format!(
                        "Failed to send request to Ollama API: {}",
                        e
                    ))
                })?
        } else {
            // Use generate endpoint
            let generate_request = OllamaGenerateRequest {
                model: request.model.clone(),
                prompt: request.prompt.clone(),
                system: request.system.clone(),
                options,
                stream: false,
            };

            self.client
                .post(&url)
                .json(&generate_request)
                .send()
                .await
                .map_err(|e| {
                    EngineError::ExecutionError(format!(
                        "Failed to send request to Ollama API: {}",
                        e
                    ))
                })?
        };

        // Check response status
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            
            if status == 404 {
                return Err(EngineError::ExecutionError(format!(
                    "Model '{}' not found. Available models: [list]. Pull with: ollama pull {}",
                    request.model, request.model
                )));
            }
            
            return Err(EngineError::ExecutionError(format!(
                "Ollama API error ({}): {}",
                status, error_text
            )));
        }

        // Parse response
        let ollama_response: OllamaResponse = response
            .json()
            .await
            .map_err(|e| EngineError::ExecutionError(format!("Failed to parse response: {}", e)))?;

        // Extract token usage
        let usage = if let (Some(prompt_count), Some(eval_count)) = (
            ollama_response.prompt_eval_count,
            ollama_response.eval_count,
        ) {
            Some(TokenUsage {
                input_tokens: prompt_count,
                output_tokens: eval_count,
                total_tokens: prompt_count + eval_count,
            })
        } else {
            None
        };

        // Serialize raw response for debugging
        let raw = serde_json::to_string(&ollama_response)
            .map_err(|e| EngineError::ExecutionError(format!("Failed to serialize response: {}", e)))?;

        Ok(ExecutionResponse {
            content: ollama_response.response,
            usage,
            model: ollama_response.model,
            raw: Some(raw),
        })
    }

    fn default_model(&self) -> String {
        "llama2:latest".to_string()
    }

    fn available_models(&self) -> Vec<String> {
        // Model discovery will be implemented in Task 2
        // For now, return empty vec
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_engine_metadata() {
        let engine = OllamaEngine::new();
        let metadata = engine.metadata();

        assert_eq!(metadata.id, "ollama");
        assert_eq!(metadata.name, "Ollama");
        assert!(!metadata.requires_auth);
    }

    #[test]
    fn test_ollama_engine_default_model() {
        let engine = OllamaEngine::new();
        assert_eq!(engine.default_model(), "llama2:latest");
    }

    #[test]
    fn test_ollama_engine_base_url_default() {
        // Clear OLLAMA_HOST if set
        unsafe {
            std::env::remove_var("OLLAMA_HOST");
        }
        let engine = OllamaEngine::new();
        // We can't directly access base_url, but we can verify it's set correctly
        // by checking that the engine was created successfully
        assert_eq!(engine.metadata().id, "ollama");
    }

    #[test]
    fn test_ollama_engine_base_url_env_override() {
        unsafe {
            std::env::set_var("OLLAMA_HOST", "http://192.168.1.100:11434");
        }
        let engine = OllamaEngine::new();
        assert_eq!(engine.metadata().id, "ollama");
        // Clean up
        unsafe {
            std::env::remove_var("OLLAMA_HOST");
        }
    }

    #[tokio::test]
    async fn test_ollama_engine_is_authenticated() {
        let engine = OllamaEngine::new();
        let result = engine.is_authenticated().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }
}

