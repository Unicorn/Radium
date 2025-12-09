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
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content_blocks: Option<Vec<AnthropicContentBlock>>,
}

/// Anthropic API content block for multimodal support.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum AnthropicContentBlock {
    Text {
        #[serde(rename = "type")]
        content_type: String,
        text: String,
    },
    Image {
        #[serde(rename = "type")]
        content_type: String,
        source: AnthropicImageSource,
    },
}

/// Anthropic API image source.
#[derive(Debug, Serialize, Deserialize)]
struct AnthropicImageSource {
    #[serde(rename = "type")]
    source_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    media_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<String>, // base64 encoded
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
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
        
        // Check for vision/image content in params
        let mut content_blocks: Vec<AnthropicContentBlock> = Vec::new();
        
        // Add text content
        if !request.prompt.is_empty() {
            content_blocks.push(AnthropicContentBlock::Text {
                content_type: "text".to_string(),
                text: request.prompt.clone(),
            });
        }
        
        // Check for images in params
        if let Some(images) = request.params.get("images") {
            if let Some(image_array) = images.as_array() {
                for image_value in image_array {
                    if let Some(image_obj) = image_value.as_object() {
                        if let Some(image_type) = image_obj.get("type").and_then(|v| v.as_str()) {
                            match image_type {
                                "base64" => {
                                    if let (Some(media_type), Some(data)) = (
                                        image_obj.get("media_type").and_then(|v| v.as_str()),
                                        image_obj.get("data").and_then(|v| v.as_str()),
                                    ) {
                                        content_blocks.push(AnthropicContentBlock::Image {
                                            content_type: "image".to_string(),
                                            source: AnthropicImageSource {
                                                source_type: "base64".to_string(),
                                                media_type: Some(media_type.to_string()),
                                                data: Some(data.to_string()),
                                                url: None,
                                            },
                                        });
                                    }
                                }
                                "url" => {
                                    if let Some(url) = image_obj.get("url").and_then(|v| v.as_str()) {
                                        content_blocks.push(AnthropicContentBlock::Image {
                                            content_type: "image".to_string(),
                                            source: AnthropicImageSource {
                                                source_type: "url".to_string(),
                                                media_type: None,
                                                data: None,
                                                url: Some(url.to_string()),
                                            },
                                        });
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
        
        // Build message - use content_blocks if we have images, otherwise use simple content
        let message = if content_blocks.len() > 1 || (content_blocks.len() == 1 && matches!(content_blocks[0], AnthropicContentBlock::Image { .. })) {
            // Use content_blocks for multimodal
            AnthropicMessage {
                role: "user".to_string(),
                content: None,
                content_blocks: Some(content_blocks),
            }
        } else {
            // Simple text message
            AnthropicMessage {
                role: "user".to_string(),
                content: Some(request.prompt),
                content_blocks: None,
            }
        };
        
        let messages = vec![message];

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

        // Check response status and handle errors
        let status = response.status();
        if !status.is_success() {
            // Get headers before consuming the response body
            let retry_after = if status == 429 {
                response.headers()
                    .get("retry-after")
                    .and_then(|h| h.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
            } else {
                None
            };
            
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            
            // Enhanced error handling
            if status == 429 {
                // Rate limit error
                
                let mut error_msg = "Rate limit exceeded".to_string();
                if let Some(seconds) = retry_after {
                    error_msg.push_str(&format!(". Retry after {} seconds", seconds));
                }
                error_msg.push_str(". Consider reducing request frequency or upgrading your plan.");
                
                return Err(EngineError::ExecutionError(error_msg));
            } else if status == 402 || status == 403 {
                // Quota or authentication error
                return Err(EngineError::AuthenticationFailed(format!(
                    "Quota exceeded or authentication failed. Check your API key and billing status. Error: {}",
                    error_text
                )));
            } else if status == 400 {
                // Bad request - try to parse error details
                if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error_text) {
                    if let Some(error_detail) = error_json.get("error").and_then(|e| e.get("message")) {
                        return Err(EngineError::ExecutionError(format!(
                            "Invalid request: {}",
                            error_detail
                        )));
                    }
                }
            }
            
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
            execution_duration: None, // Cloud models use token-based costing
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

