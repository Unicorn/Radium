//! Claude (Anthropic) engine provider implementation.

use crate::auth::{CredentialStore, ProviderType};
use crate::engines::engine_trait::{
    Engine, EngineMetadata, ExecutionRequest, ExecutionResponse, TokenUsage,
};
use crate::engines::error::{EngineError, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Claude engine implementation for Anthropic API.
pub struct ClaudeEngine {
    /// Engine metadata.
    metadata: EngineMetadata,
    /// HTTP client for API requests.
    client: Arc<Client>,
    /// Credential store for API key retrieval.
    credential_store: Arc<CredentialStore>,
}

impl ClaudeEngine {
    /// Creates a new Claude engine.
    pub fn new() -> Self {
        let metadata = EngineMetadata::new(
            "claude".to_string(),
            "Claude".to_string(),
            "Anthropic Claude AI engine".to_string(),
        )
        .with_models(vec![
            "claude-3-opus-20240229".to_string(),
            "claude-3-sonnet-20240229".to_string(),
            "claude-3-haiku-20240307".to_string(),
        ])
        .with_auth_required(true);

        // CredentialStore::new() can fail if HOME is not set, fallback to temp path
        let credential_store = CredentialStore::new().unwrap_or_else(|_| {
            // If default path fails, use temp directory (shouldn't happen in practice)
            let temp_path = std::env::temp_dir().join("radium_credentials.json");
            CredentialStore::with_path(temp_path)
        });

        Self {
            metadata,
            client: Arc::new(Client::new()),
            credential_store: Arc::new(credential_store),
        }
    }

    /// Gets the API key from credential store.
    fn get_api_key(&self) -> Result<String> {
        self.credential_store
            .get(ProviderType::Claude)
            .map_err(|e| EngineError::AuthenticationFailed(format!("Failed to get API key: {}", e)))
    }
}

impl Default for ClaudeEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Anthropic API request structure.
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: usize,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

/// Anthropic API message structure.
#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

/// Anthropic API response structure.
#[derive(Debug, Deserialize, Serialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
    #[serde(rename = "usage")]
    usage: Option<AnthropicUsage>,
    model: String,
}

/// Anthropic API content structure.
#[derive(Debug, Deserialize, Serialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

/// Anthropic API usage structure.
#[derive(Debug, Deserialize, Serialize)]
struct AnthropicUsage {
    #[serde(rename = "input_tokens")]
    input_tokens: u64,
    #[serde(rename = "output_tokens")]
    output_tokens: u64,
}

#[async_trait]
impl Engine for ClaudeEngine {
    fn metadata(&self) -> &EngineMetadata {
        &self.metadata
    }

    async fn is_available(&self) -> bool {
        // Claude is API-based, always available if authenticated
        true
    }

    async fn is_authenticated(&self) -> Result<bool> {
        match self.get_api_key() {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn execute(&self, request: ExecutionRequest) -> Result<ExecutionResponse> {
        let api_key = self.get_api_key()?;

        // Build Anthropic API request
        let max_tokens = request.max_tokens.unwrap_or(4096);
        let messages = vec![AnthropicMessage {
            role: "user".to_string(),
            content: request.prompt,
        }];

        let anthropic_request = AnthropicRequest {
            model: request.model,
            max_tokens,
            messages,
            system: request.system,
            temperature: request.temperature,
        };

        // Make API request
        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&anthropic_request)
            .send()
            .await
            .map_err(|e| {
                EngineError::ExecutionError(format!("Failed to send request to Anthropic API: {}", e))
            })?;

        // Check response status
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(EngineError::ExecutionError(format!(
                "Anthropic API error ({}): {}",
                status, error_text
            )));
        }

        // Parse response
        let anthropic_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| EngineError::ExecutionError(format!("Failed to parse response: {}", e)))?;

        // Extract content from response
        let content = anthropic_response
            .content
            .iter()
            .filter_map(|c| {
                if c.content_type == "text" {
                    Some(c.text.as_str())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("");

        // Extract token usage
        let usage = anthropic_response.usage.as_ref().map(|u| TokenUsage {
            input_tokens: u.input_tokens,
            output_tokens: u.output_tokens,
            total_tokens: u.input_tokens + u.output_tokens,
        });

        // Serialize raw response for debugging
        let raw = serde_json::to_string(&anthropic_response)
            .map_err(|e| EngineError::ExecutionError(format!("Failed to serialize response: {}", e)))?;

        Ok(ExecutionResponse {
            content,
            usage,
            model: anthropic_response.model,
            raw: Some(raw),
        })
    }

    fn default_model(&self) -> String {
        "claude-3-sonnet-20240229".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_claude_engine_metadata() {
        let engine = ClaudeEngine::new();
        let metadata = engine.metadata();

        assert_eq!(metadata.id, "claude");
        assert_eq!(metadata.name, "Claude");
        assert!(metadata.requires_auth);
        assert_eq!(metadata.models.len(), 3);
    }

    #[tokio::test]
    async fn test_claude_engine_is_available() {
        let engine = ClaudeEngine::new();
        assert!(engine.is_available().await);
    }

    #[tokio::test]
    async fn test_claude_engine_default_model() {
        let engine = ClaudeEngine::new();
        assert_eq!(engine.default_model(), "claude-3-sonnet-20240229");
    }
}

