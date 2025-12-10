//! Google Gemini model implementation.
//!
//! This module provides an implementation of the `Model` trait for Google's Gemini API.

use async_trait::async_trait;
use radium_abstraction::{
    ChatMessage, Model, ModelError, ModelParameters, ModelResponse, ModelUsage, ResponseFormat,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{debug, error, warn};

/// Google Gemini model implementation.
#[derive(Debug, Clone)]
pub struct GeminiModel {
    /// The model ID (e.g., "gemini-pro", "gemini-1.5-pro").
    model_id: String,
    /// The API key for authentication.
    api_key: String,
    /// The base URL for the Gemini API.
    base_url: String,
    /// HTTP client for making requests.
    client: Client,
}

impl GeminiModel {
    /// Creates a new `GeminiModel` with the given model ID.
    ///
    /// # Arguments
    /// * `model_id` - The Gemini model ID to use (e.g., "gemini-pro")
    ///
    /// # Errors
    /// Returns a `ModelError` if the API key is not found in environment variables.
    #[allow(clippy::disallowed_methods)] // env::var is needed for API key loading
    pub fn new(model_id: String) -> Result<Self, ModelError> {
        let api_key = env::var("GEMINI_API_KEY").map_err(|_| {
            ModelError::UnsupportedModelProvider(
                "GEMINI_API_KEY environment variable not set".to_string(),
            )
        })?;

        Ok(Self {
            model_id,
            api_key,
            base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            client: Client::new(),
        })
    }

    /// Creates a new `GeminiModel` with a custom API key.
    ///
    /// # Arguments
    /// * `model_id` - The Gemini model ID to use
    /// * `api_key` - The API key for authentication
    #[must_use]
    pub fn with_api_key(model_id: String, api_key: String) -> Self {
        Self {
            model_id,
            api_key,
            base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            client: Client::new(),
        }
    }

    /// Converts our ChatMessage role to Gemini API role format.
    ///
    /// Note: System messages should be filtered out before calling this function,
    /// as they are handled separately via the `systemInstruction` field.
    fn role_to_gemini(role: &str) -> String {
        match role {
            "assistant" => "model".to_string(),
            "user" => "user".to_string(),
            // System messages are filtered out before this function is called
            _ => role.to_string(),
        }
    }

    /// Extracts system messages from the chat history and concatenates them.
    ///
    /// Multiple system messages are joined with "\n\n" separator.
    /// Returns `None` if no system messages are present.
    ///
    /// This follows the same pattern as Claude's `extract_system_prompt()` function.
    fn extract_system_messages(messages: &[ChatMessage]) -> Option<String> {
        let system_messages: Vec<String> = messages
            .iter()
            .filter(|msg| msg.role == "system")
            .map(|msg| msg.content.clone())
            .collect();

        if system_messages.is_empty() {
            None
        } else {
            Some(system_messages.join("\n\n"))
        }
    }
}

#[async_trait]
impl Model for GeminiModel {
    async fn generate_text(
        &self,
        prompt: &str,
        parameters: Option<ModelParameters>,
    ) -> Result<ModelResponse, ModelError> {
        debug!(
            model_id = %self.model_id,
            prompt_len = prompt.len(),
            parameters = ?parameters,
            "GeminiModel generating text"
        );

        // Convert single prompt to chat format for Gemini
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
            "GeminiModel generating chat completion"
        );

        // Build Gemini API request
        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.base_url, self.model_id, self.api_key
        );

        // Extract system instruction if present
        let system_instruction = Self::extract_system_messages(messages);

        // Convert non-system messages to Gemini format
        let gemini_messages: Vec<GeminiContent> = messages
            .iter()
            .filter(|msg| msg.role != "system")
            .map(|msg| GeminiContent {
                role: Self::role_to_gemini(&msg.role),
                parts: vec![GeminiPart { text: msg.content.clone() }],
            })
            .collect();

        // Build request body
        let mut request_body = GeminiRequest {
            contents: gemini_messages,
            generation_config: None,
            system_instruction: system_instruction.map(|text| GeminiSystemInstruction {
                parts: vec![GeminiPart { text }],
            }),
        };

        // Apply parameters if provided
        if let Some(params) = parameters {
            // Handle response format (mime type and schema)
            let (mime_type, schema) = match &params.response_format {
                Some(ResponseFormat::Json) => (Some("application/json".to_string()), None),
                Some(ResponseFormat::JsonSchema(schema_str)) => {
                    match serde_json::from_str::<serde_json::Value>(schema_str) {
                        Ok(parsed_schema) => {
                            (Some("application/json".to_string()), Some(parsed_schema))
                        }
                        Err(e) => {
                            error!(error = %e, schema = schema_str, "Invalid JSON schema in response_format");
                            return Err(ModelError::SerializationError(format!(
                                "Invalid JSON schema: {}",
                                e
                            )));
                        }
                    }
                }
                _ => (None, None),
            };

            // Clamp penalty values to Gemini's 0.0-2.0 range
            let frequency_penalty = params.frequency_penalty.map(|p| {
                let clamped = p.clamp(0.0, 2.0);
                if clamped != p {
                    warn!(
                        original = p,
                        clamped = clamped,
                        "Clamping frequency_penalty to Gemini range [0.0, 2.0]"
                    );
                }
                clamped
            });

            let presence_penalty = params.presence_penalty.map(|p| {
                let clamped = p.clamp(0.0, 2.0);
                if clamped != p {
                    warn!(
                        original = p,
                        clamped = clamped,
                        "Clamping presence_penalty to Gemini range [0.0, 2.0]"
                    );
                }
                clamped
            });

            request_body.generation_config = Some(GeminiGenerationConfig {
                temperature: params.temperature,
                top_p: params.top_p,
                max_output_tokens: params.max_tokens,
                top_k: params.top_k,
                frequency_penalty,
                presence_penalty,
                response_mime_type: mime_type,
                response_schema: schema,
                stop_sequences: params.stop_sequences,
            });
        }

        // Make API request using reqwest
        let response = self.client.post(&url).json(&request_body).send().await.map_err(|e| {
            error!(error = %e, "Failed to send request to Gemini API");
            ModelError::RequestError(format!("Network error: {}", e))
        })?;

        // Check status
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                status = %status,
                error = %error_text,
                "Gemini API returned error status"
            );
            
            // Map quota/rate limit errors to QuotaExceeded
            if status == 402 || status == 429 {
                // Check for Gemini-specific quota error patterns
                let is_quota_error = error_text.to_uppercase().contains("RESOURCE_EXHAUSTED")
                    || error_text.to_lowercase().contains("quota exceeded")
                    || error_text.to_lowercase().contains("quota")
                    || error_text.to_lowercase().contains("rate limit");
                
                if is_quota_error || status == 402 {
                    return Err(ModelError::QuotaExceeded {
                        provider: "gemini".to_string(),
                        message: Some(error_text),
                    });
                }
            }
            
            // For 429, if it's a rate limit (not quota), we still treat it as QuotaExceeded
            // after potential retries (handled by orchestrator)
            if status == 429 {
                return Err(ModelError::QuotaExceeded {
                    provider: "gemini".to_string(),
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
        let gemini_response: GeminiResponse = response.json().await.map_err(|e| {
            error!(error = %e, "Failed to parse Gemini API response");
            ModelError::SerializationError(format!("Failed to parse response: {}", e))
        })?;

        // Extract content from response
        let content = gemini_response
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .ok_or_else(|| {
                error!("No content in Gemini API response");
                ModelError::ModelResponseError("No content in API response".to_string())
            })?;

        // Extract usage information
        let usage = gemini_response.usage_metadata.map(|meta| ModelUsage {
            prompt_tokens: meta.prompt_token_count.unwrap_or(0),
            completion_tokens: meta.candidates_token_count.unwrap_or(0),
            total_tokens: meta.total_token_count.unwrap_or(0),
        });

        Ok(ModelResponse { content, model_id: Some(self.model_id.clone()), usage })
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }
}

// Gemini API request/response structures

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GeminiGenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "systemInstruction")]
    system_instruction: Option<GeminiSystemInstruction>,
}

#[derive(Debug, Clone, Serialize)]
struct GeminiSystemInstruction {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiPart {
    text: String,
}

#[derive(Debug, Serialize)]
struct GeminiGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
    #[serde(rename = "topK", skip_serializing_if = "Option::is_none")]
    top_k: Option<u32>,
    #[serde(rename = "frequencyPenalty", skip_serializing_if = "Option::is_none")]
    frequency_penalty: Option<f32>,
    #[serde(rename = "presencePenalty", skip_serializing_if = "Option::is_none")]
    presence_penalty: Option<f32>,
    #[serde(rename = "responseMimeType", skip_serializing_if = "Option::is_none")]
    response_mime_type: Option<String>,
    #[serde(rename = "responseSchema", skip_serializing_if = "Option::is_none")]
    response_schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<GeminiUsageMetadata>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}

#[derive(Debug, Deserialize)]
#[allow(clippy::struct_field_names)] // Matches API naming
struct GeminiUsageMetadata {
    #[serde(rename = "promptTokenCount")]
    prompt_token_count: Option<u32>,
    #[serde(rename = "candidatesTokenCount")]
    candidates_token_count: Option<u32>,
    #[serde(rename = "totalTokenCount")]
    total_token_count: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_conversion() {
        assert_eq!(GeminiModel::role_to_gemini("user"), "user");
        assert_eq!(GeminiModel::role_to_gemini("assistant"), "model");
        // Note: System messages are filtered out before role_to_gemini() is called,
        // so they are handled separately via systemInstruction field
    }

    #[test]
    fn test_extract_system_messages() {
        use radium_abstraction::ChatMessage;

        // Test single system message
        let messages = vec![
            ChatMessage { role: "system".to_string(), content: "You are helpful.".to_string() },
            ChatMessage { role: "user".to_string(), content: "Hello".to_string() },
        ];
        let system = GeminiModel::extract_system_messages(&messages);
        assert_eq!(system, Some("You are helpful.".to_string()));

        // Test multiple system messages (should be concatenated)
        let messages = vec![
            ChatMessage { role: "system".to_string(), content: "First instruction.".to_string() },
            ChatMessage { role: "system".to_string(), content: "Second instruction.".to_string() },
            ChatMessage { role: "user".to_string(), content: "Hello".to_string() },
        ];
        let system = GeminiModel::extract_system_messages(&messages);
        assert_eq!(system, Some("First instruction.\n\nSecond instruction.".to_string()));

        // Test no system messages
        let messages = vec![
            ChatMessage { role: "user".to_string(), content: "Hello".to_string() },
            ChatMessage { role: "assistant".to_string(), content: "Hi there!".to_string() },
        ];
        let system = GeminiModel::extract_system_messages(&messages);
        assert_eq!(system, None);

        // Test mixed message types
        let messages = vec![
            ChatMessage { role: "system".to_string(), content: "System message.".to_string() },
            ChatMessage { role: "user".to_string(), content: "User message.".to_string() },
            ChatMessage { role: "assistant".to_string(), content: "Assistant message.".to_string() },
        ];
        let system = GeminiModel::extract_system_messages(&messages);
        assert_eq!(system, Some("System message.".to_string()));

        // Test empty system message content
        let messages = vec![
            ChatMessage { role: "system".to_string(), content: "".to_string() },
            ChatMessage { role: "user".to_string(), content: "Hello".to_string() },
        ];
        let system = GeminiModel::extract_system_messages(&messages);
        assert_eq!(system, Some("".to_string()));
    }

    #[test]
    fn test_system_message_filtering_from_contents() {
        use radium_abstraction::ChatMessage;

        // Test that system messages are filtered from contents array
        let messages = vec![
            ChatMessage { role: "system".to_string(), content: "System instruction.".to_string() },
            ChatMessage { role: "user".to_string(), content: "User message.".to_string() },
            ChatMessage { role: "assistant".to_string(), content: "Assistant message.".to_string() },
        ];

        // Filter system messages (simulating what happens in generate_chat_completion)
        let filtered: Vec<&ChatMessage> = messages.iter().filter(|msg| msg.role != "system").collect();

        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].role, "user");
        assert_eq!(filtered[1].role, "assistant");
        assert!(!filtered.iter().any(|msg| msg.role == "system"));
    }

    #[test]
    fn test_request_serialization_with_system_instruction() {
        use serde_json;

        // Test that systemInstruction field is included when system messages are present
        let system_text = "You are a helpful assistant.";
        let system_instruction = GeminiSystemInstruction {
            parts: vec![GeminiPart { text: system_text.to_string() }],
        };

        let request = GeminiRequest {
            contents: vec![GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart { text: "Hello".to_string() }],
            }],
            generation_config: None,
            system_instruction: Some(system_instruction),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("systemInstruction"));
        assert!(json.contains(system_text));

        // Test that systemInstruction field is omitted when None
        let request_no_system = GeminiRequest {
            contents: vec![GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart { text: "Hello".to_string() }],
            }],
            generation_config: None,
            system_instruction: None,
        };

        let json_no_system = serde_json::to_string(&request_no_system).unwrap();
        assert!(!json_no_system.contains("systemInstruction"));
    }

    #[test]
    fn test_gemini_model_creation_with_api_key() {
        let model = GeminiModel::with_api_key("gemini-pro".to_string(), "test-key".to_string());
        assert_eq!(model.model_id(), "gemini-pro");
    }

    #[test]
    #[ignore = "Requires API key and network access"]
    #[allow(clippy::disallowed_methods, clippy::disallowed_macros)] // Test code can use env::var and eprintln
    fn test_gemini_generate_text() {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(async {
            let api_key = env::var("GEMINI_API_KEY").ok();
            if api_key.is_none() {
                eprintln!("Skipping test: GEMINI_API_KEY not set");
                return;
            }

            let model = GeminiModel::new("gemini-pro".to_string()).unwrap();
            let response =
                model.generate_text("Say hello", None).await.expect("Should generate text");

            assert!(!response.content.is_empty());
            assert_eq!(response.model_id, Some("gemini-pro".to_string()));
        });
    }
}
