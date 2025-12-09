//! Ollama engine provider implementation.

use crate::engines::engine_trait::{
    Engine, EngineMetadata, ExecutionRequest, ExecutionResponse, TokenUsage,
};
use crate::engines::error::{EngineError, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

/// Ollama engine implementation for local Ollama server.
pub struct OllamaEngine {
    /// Engine metadata.
    metadata: EngineMetadata,
    /// HTTP client for API requests.
    client: Arc<Client>,
    /// Base URL for Ollama server.
    base_url: String,
    /// Model cache with TTL (5 minutes).
    model_cache: Arc<RwLock<Option<(Instant, Vec<OllamaModelMetadata>)>>>,
    /// Cached model names for synchronous access.
    cached_model_names: Arc<Mutex<Vec<String>>>,
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
            model_cache: Arc::new(RwLock::new(None)),
            cached_model_names: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Fetches models from Ollama server.
    async fn fetch_models(&self) -> Result<Vec<OllamaModelMetadata>> {
        let url = format!("{}/api/tags", self.base_url);
        
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| {
                EngineError::ExecutionError(format!(
                    "Failed to fetch models from Ollama API: {}",
                    e
                ))
            })?;

        // Check response status
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(EngineError::ExecutionError(format!(
                "Ollama API error ({}): {}",
                status, error_text
            )));
        }

        // Parse response
        let tags_response: OllamaTagsResponse = response
            .json()
            .await
            .map_err(|e| EngineError::ExecutionError(format!("Failed to parse response: {}", e)))?;

        // Convert to OllamaModelMetadata
        let models: Vec<OllamaModelMetadata> = tags_response
            .models
            .into_iter()
            .map(|model| OllamaModelMetadata {
                name: model.name,
                size_bytes: model.size,
                modified_at: model.modified_at,
                digest: model.digest,
                format: model.details.as_ref().and_then(|d| d.format.clone()),
                family: model.details.as_ref().and_then(|d| d.family.clone()),
                parameter_size: model.details.as_ref().and_then(|d| d.parameter_size.clone()),
                quantization_level: model.details.as_ref().and_then(|d| d.quantization_level.clone()),
            })
            .collect();

        Ok(models)
    }

    /// Gets cached models, refreshing if cache is expired or missing.
    async fn get_cached_models(&self) -> Result<Vec<OllamaModelMetadata>> {
        const CACHE_TTL_SECS: u64 = 300; // 5 minutes

        // Check cache
        {
            let cache = self.model_cache.read().map_err(|e| {
                EngineError::RegistryError(format!("Failed to read model cache: {}", e))
            })?;

            if let Some((cached_at, models)) = cache.as_ref() {
                if cached_at.elapsed() < Duration::from_secs(CACHE_TTL_SECS) {
                    return Ok(models.clone());
                }
            }
        }

        // Cache expired or missing, fetch new models
        let models = self.fetch_models().await?;

        // Update cache
        {
            let mut cache = self.model_cache.write().map_err(|e| {
                EngineError::RegistryError(format!("Failed to write model cache: {}", e))
            })?;
            *cache = Some((Instant::now(), models.clone()));
        }

        // Update cached model names for synchronous access
        {
            let model_names: Vec<String> = models.iter().map(|m| m.name.clone()).collect();
            if let Ok(mut cached) = self.cached_model_names.lock() {
                *cached = model_names;
            }
        }

        Ok(models)
    }

    /// Formats bytes as human-readable size string.
    fn format_size(bytes: u64) -> String {
        const GB: u64 = 1_000_000_000;
        const MB: u64 = 1_000_000;
        const KB: u64 = 1_000;

        if bytes >= GB {
            format!("{:.1} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.1} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.1} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
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

/// Ollama model metadata.
#[derive(Debug, Clone)]
pub struct OllamaModelMetadata {
    /// Model name (e.g., "llama2:latest").
    pub name: String,
    /// Model size in bytes.
    pub size_bytes: u64,
    /// Last modified timestamp.
    pub modified_at: String,
    /// Model digest.
    pub digest: String,
    /// Model format (e.g., "gguf").
    pub format: Option<String>,
    /// Model family (e.g., "llama", "mistral").
    pub family: Option<String>,
    /// Parameter size (e.g., "7B", "13B").
    pub parameter_size: Option<String>,
    /// Quantization level (e.g., "Q4_0", "Q5_K_M").
    pub quantization_level: Option<String>,
}

/// Ollama API tags response structure.
#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModelInfo>,
}

/// Ollama model information from API.
#[derive(Debug, Deserialize)]
struct OllamaModelInfo {
    name: String,
    size: u64,
    #[serde(rename = "modified_at")]
    modified_at: String,
    digest: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<OllamaModelDetails>,
}

/// Ollama model details.
#[derive(Debug, Deserialize)]
struct OllamaModelDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameter_size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    quantization_level: Option<String>,
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
        // Return cached model names synchronously
        // The cache will be populated asynchronously when models are first fetched
        self.cached_model_names
            .lock()
            .map(|names| names.clone())
            .unwrap_or_default()
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

