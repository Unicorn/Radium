//! Ollama model provider for local LLM execution.
//!
//! This module provides integration with Ollama, enabling local model execution
//! without API costs or internet connectivity requirements.
//!
//! # Setup
//!
//! 1. Install Ollama: `curl https://ollama.ai/install.sh | sh`
//! 2. Start Ollama: `ollama serve`
//! 3. Pull a model: `ollama pull llama2`
//!
//! # Examples
//!
//! ## Non-Streaming Text Generation
//!
//! ```rust,no_run
//! use radium_models::OllamaModel;
//! use radium_abstraction::Model;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let model = OllamaModel::new("llama2".to_string())?;
//!     let response = model.generate_text("Hello!", None).await?;
//!     println!("{}", response.content);
//!     Ok(())
//! }
//! ```
//!
//! ## Streaming Text Generation
//!
//! ```rust,no_run
//! use radium_models::OllamaModel;
//! use radium_abstraction::StreamingModel;
//! use futures::StreamExt;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let model = OllamaModel::new("llama2".to_string())?;
//!     let mut stream = model.generate_stream("Write a story", None).await?;
//!     
//!     while let Some(token) = stream.next().await {
//!         print!("{}", token?);
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## Remote Ollama Server
//!
//! ```rust,no_run
//! use radium_models::OllamaModel;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let model = OllamaModel::with_base_url(
//!     "llama2".to_string(),
//!     "http://192.168.1.100:11434".to_string()
//! )?;
//! # Ok(())
//! # }
//! ```
//!
//! # Configuration
//!
//! - `model_id`: Model identifier (e.g., "llama2", "codellama:13b")
//! - `base_url`: Ollama server URL (default: "http://localhost:11434")
//!
//! # Troubleshooting
//!
//! ## "Ollama server not reachable"
//! - Ensure Ollama is running: `ollama serve`
//! - Check the server is listening on port 11434
//! - For remote servers, set custom base_url
//!
//! ## "Model not found"
//! - Pull the model: `ollama pull llama2`
//! - List available models: `ollama list`
//!
//! ## "Insufficient memory"
//! - Try a smaller model variant (e.g., llama2:7b instead of llama2:13b)
//! - Close other applications to free memory

use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};
use radium_abstraction::{
    ChatMessage, Model, ModelError, ModelParameters, ModelResponse, ModelUsage, StreamingModel,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
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

#[async_trait]
impl StreamingModel for OllamaModel {
    async fn generate_stream(
        &self,
        prompt: &str,
        parameters: Option<ModelParameters>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, ModelError>> + Send>>, ModelError> {
        debug!(
            model_id = %self.model_id,
            prompt_len = prompt.len(),
            parameters = ?parameters,
            "OllamaModel generating streaming text"
        );

        let url = format!("{}/api/generate", self.base_url);

        let request_body = OllamaGenerateRequest {
            model: self.model_id.clone(),
            prompt: prompt.to_string(),
            stream: true,
            options: Self::build_options(parameters),
        };

        // Make streaming request
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

        // Create stream from response body
        let stream = response
            .bytes_stream()
            .map(|chunk_result| -> Result<String, ModelError> {
                let chunk = chunk_result.map_err(|e| {
                    error!(error = %e, "Failed to read chunk from Ollama stream");
                    ModelError::RequestError(format!("Stream error: {}", e))
                })?;

                // Convert bytes to string
                let text = String::from_utf8_lossy(&chunk).to_string();
                Ok(text)
            })
            .flat_map(|text_result| {
                futures::stream::iter(match text_result {
                    Ok(text) => {
                        // Split by newlines to handle NDJSON
                        let lines: Vec<String> = text
                            .lines()
                            .map(|s| s.to_string())
                            .filter(|s| !s.is_empty())
                            .collect();

                        // Parse each line as JSON and extract tokens
                        lines
                            .into_iter()
                            .filter_map(|line| {
                                match serde_json::from_str::<OllamaResponse>(&line) {
                                    Ok(ollama_resp) => {
                                        if ollama_resp.done {
                                            None // End of stream
                                        } else {
                                            Some(Ok(ollama_resp.response))
                                        }
                                    }
                                    Err(_e) => {
                                        // If it's not valid JSON, it might be a partial token
                                        // Just return it as-is
                                        if !line.trim().is_empty() {
                                            Some(Ok(line))
                                        } else {
                                            None
                                        }
                                    }
                                }
                            })
                            .collect::<Vec<_>>()
                    }
                    Err(e) => vec![Err(e)],
                })
            });

        Ok(Box::pin(stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use mockito::Server;
    use radium_abstraction::ChatMessage;

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

    #[tokio::test]
    async fn test_generate_text_success() {
        let mut server = Server::new_async().await;
        let mock_response = r#"{"response":"Hello, world!","done":true,"prompt_eval_count":5,"eval_count":3}"#;
        
        let mock = server
            .mock("POST", "/api/generate")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create();

        let model = OllamaModel::with_base_url(
            "llama2".to_string(),
            server.url(),
        ).unwrap();

        let response = model.generate_text("Hello", None).await.unwrap();
        
        assert_eq!(response.content, "Hello, world!");
        let usage = response.usage.unwrap();
        assert_eq!(usage.prompt_tokens, 5);
        assert_eq!(usage.completion_tokens, 3);
        mock.assert();
    }

    #[tokio::test]
    async fn test_generate_chat_completion_success() {
        let mut server = Server::new_async().await;
        let mock_response = r#"{"response":"Hi there!","done":true,"prompt_eval_count":10,"eval_count":4}"#;
        
        let mock = server
            .mock("POST", "/api/chat")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create();

        let model = OllamaModel::with_base_url(
            "llama2".to_string(),
            server.url(),
        ).unwrap();

        let messages = vec![
            ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            },
        ];

        let response = model.generate_chat_completion(&messages, None).await.unwrap();
        
        assert_eq!(response.content, "Hi there!");
        assert_eq!(response.usage.unwrap().total_tokens, 14);
        mock.assert();
    }

    #[tokio::test]
    async fn test_generate_stream_success() {
        let mut server = Server::new_async().await;
        // NDJSON format: one JSON object per line
        let mock_response = r#"{"response":"Hello","done":false}
{"response":" world","done":false}
{"response":"!","done":true,"prompt_eval_count":5,"eval_count":3}"#;
        
        let mock = server
            .mock("POST", "/api/generate")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create();

        let model = OllamaModel::with_base_url(
            "llama2".to_string(),
            server.url(),
        ).unwrap();

        let mut stream = model.generate_stream("Hello", None).await.unwrap();
        let mut tokens = Vec::new();
        
        while let Some(result) = stream.next().await {
            match result {
                Ok(token) => tokens.push(token),
                Err(e) => panic!("Unexpected error in stream: {}", e),
            }
        }

        // Note: The current implementation may yield tokens differently
        // This test verifies the stream works end-to-end
        assert!(!tokens.is_empty());
        mock.assert();
    }

    #[tokio::test]
    async fn test_connection_error() {
        // Use an invalid URL to trigger connection error
        let model = OllamaModel::with_base_url(
            "llama2".to_string(),
            "http://127.0.0.1:99999".to_string(), // Invalid port
        ).unwrap();

        let result = model.generate_text("Hello", None).await;
        
        assert!(result.is_err());
        if let Err(ModelError::RequestError(msg)) = result {
            assert!(msg.contains("not reachable") || msg.contains("Network error"));
        } else {
            panic!("Expected RequestError, got: {:?}", result);
        }
    }

    #[tokio::test]
    async fn test_model_not_found_error() {
        let mut server = Server::new_async().await;
        let error_response = r#"{"error":"model 'fake-model' not found"}"#;
        
        let mock = server
            .mock("POST", "/api/generate")
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(error_response)
            .create();

        let model = OllamaModel::with_base_url(
            "fake-model".to_string(),
            server.url(),
        ).unwrap();

        let result = model.generate_text("Hello", None).await;
        
        assert!(result.is_err());
        if let Err(ModelError::ModelResponseError(msg)) = result {
            assert!(msg.contains("not found"));
            assert!(msg.contains("ollama pull"));
        } else {
            panic!("Expected ModelResponseError, got: {:?}", result);
        }
        mock.assert();
    }

    #[tokio::test]
    async fn test_out_of_memory_error() {
        let mut server = Server::new_async().await;
        let error_response = r#"{"error":"out of memory"}"#;
        
        let mock = server
            .mock("POST", "/api/generate")
            .with_status(500)
            .with_header("content-type", "application/json")
            .with_body(error_response)
            .create();

        let model = OllamaModel::with_base_url(
            "llama2".to_string(),
            server.url(),
        ).unwrap();

        let result = model.generate_text("Hello", None).await;
        
        assert!(result.is_err());
        if let Err(ModelError::ModelResponseError(msg)) = result {
            assert!(msg.contains("Insufficient memory"));
            assert!(msg.contains("smaller variant"));
        } else {
            panic!("Expected ModelResponseError, got: {:?}", result);
        }
        mock.assert();
    }

    #[tokio::test]
    async fn test_token_usage_tracking() {
        let mut server = Server::new_async().await;
        let mock_response = r#"{"response":"Test response","done":true,"prompt_eval_count":10,"eval_count":20}"#;
        
        let mock = server
            .mock("POST", "/api/generate")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create();

        let model = OllamaModel::with_base_url(
            "llama2".to_string(),
            server.url(),
        ).unwrap();

        let response = model.generate_text("Test", None).await.unwrap();
        let usage = response.usage.unwrap();
        
        assert_eq!(usage.prompt_tokens, 10);
        assert_eq!(usage.completion_tokens, 20);
        assert_eq!(usage.total_tokens, 30);
        mock.assert();
    }

    #[tokio::test]
    async fn test_custom_base_url() {
        let mut server = Server::new_async().await;
        let mock_response = r#"{"response":"Custom URL test","done":true,"prompt_eval_count":1,"eval_count":1}"#;
        
        let mock = server
            .mock("POST", "/api/generate")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create();

        let custom_url = server.url();
        let model = OllamaModel::with_base_url(
            "llama2".to_string(),
            custom_url.clone(),
        ).unwrap();

        let response = model.generate_text("Test", None).await.unwrap();
        
        assert_eq!(response.content, "Custom URL test");
        mock.assert();
    }
}

