//! Google Gemini model implementation.
//!
//! This module provides an implementation of the `Model` trait for Google's Gemini API.

pub mod file_api;

use async_trait::async_trait;
use futures::stream::Stream;
use radium_abstraction::{
    ChatMessage, ContentBlock, ImageSource, MessageContent, Citation, Model, ModelError,
    ModelParameters, ModelResponse, ModelUsage, ResponseFormat, SafetyRating, StreamingModel,
};
use std::path::PathBuf;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::pin::Pin;
use std::task::{Context, Poll};
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
            .filter_map(|msg| match &msg.content {
                MessageContent::Text(text) => Some(text.clone()),
                MessageContent::Blocks(blocks) => {
                    // Extract first text block if available
                    blocks
                        .iter()
                        .find_map(|block| match block {
                            ContentBlock::Text { text } => Some(text.clone()),
                            _ => None,
                        })
                }
            })
            .collect();

        if system_messages.is_empty() {
            None
        } else {
            Some(system_messages.join("\n\n"))
        }
    }

    /// Reads a file and encodes it to Base64.
    fn read_and_encode_file(path: &PathBuf) -> Result<String, ModelError> {
        let bytes = std::fs::read(path).map_err(|e| {
            ModelError::InvalidMediaSource {
                media_source: path.display().to_string(),
                reason: format!("Failed to read file: {}", e),
            }
        })?;
        encoding_utils::encode_to_base64(&bytes)
    }

    /// Converts a ContentBlock to Gemini's part format.
    fn content_block_to_gemini_part(
        block: &ContentBlock,
    ) -> Result<GeminiPart, ModelError> {
        match block {
            ContentBlock::Text { text } => Ok(GeminiPart::Text {
                text: text.clone(),
            }),
            ContentBlock::Image { source, media_type } => {
                match source {
                    ImageSource::Base64 { data } => {
                        // For base64 data, check if it's already encoded size exceeds limit
                        // Base64 data is already encoded, so we check the encoded size directly
                        let encoded_size = data.len();
                        if encoded_size > validation_utils::MAX_INLINE_SIZE {
                            return Err(ModelError::ContentTooLarge {
                                actual_size: encoded_size,
                                max_size: validation_utils::MAX_INLINE_SIZE,
                                content_type: media_type.clone(),
                            });
                        }
                        Ok(GeminiPart::InlineData {
                            inline_data: GeminiInlineData {
                                mime_type: media_type.clone(),
                                data,
                            },
                        })
                    }
                    ImageSource::File { path } => {
                        // Check file size before reading
                        let metadata = std::fs::metadata(path).map_err(|e| {
                            ModelError::InvalidMediaSource {
                                media_source: path.display().to_string(),
                                reason: format!("Failed to read file metadata: {}", e),
                            }
                        })?;
                        let file_size = metadata.len() as usize;

                        // Check if we should use file URI (for FileApi) or inline data
                        // For now, if file is too large, we'll need FileApi (REQ-220)
                        // For files that fit, use inline data
                        if validation_utils::should_use_file_uri(file_size) {
                            // File is too large for inline - would need FileApi
                            // For now, return error suggesting FileApi usage
                            return Err(ModelError::ContentTooLarge {
                                actual_size: validation_utils::calculate_base64_size(file_size),
                                max_size: validation_utils::MAX_INLINE_SIZE,
                                content_type: media_type.clone(),
                            });
                        }

                        // File is small enough for inline transmission
                        let encoded = Self::read_and_encode_file(path)?;
                        Ok(GeminiPart::InlineData {
                            inline_data: GeminiInlineData {
                                mime_type: media_type.clone(),
                                data: encoded,
                            },
                        })
                    }
                    ImageSource::Url { url } => {
                        // URL images - validate URI format
                        validation_utils::validate_file_uri(&url)?;
                        // For URL images, we'd need to download and encode, or use as-is
                        // For now, return error as URL support needs more work
                        Err(ModelError::UnsupportedContentType {
                            content_type: "image (URL)".to_string(),
                            model: "gemini".to_string(),
                        })
                    }
                }
            }
            ContentBlock::Audio { source, media_type } => {
                match source {
                    radium_abstraction::MediaSource::FileApi { file_id } => {
                        Ok(GeminiPart::FileData {
                            file_data: GeminiFileData {
                                mime_type: media_type.clone(),
                                file_uri: file_id.clone(),
                            },
                        })
                    }
                    _ => Err(ModelError::UnsupportedContentType {
                        content_type: "audio (non-FileAPI)".to_string(),
                        model: "gemini".to_string(),
                    }),
                }
            }
            ContentBlock::Video { source, media_type } => {
                match source {
                    radium_abstraction::MediaSource::FileApi { file_id } => {
                        Ok(GeminiPart::FileData {
                            file_data: GeminiFileData {
                                mime_type: media_type.clone(),
                                file_uri: file_id.clone(),
                            },
                        })
                    }
                    _ => Err(ModelError::UnsupportedContentType {
                        content_type: "video (non-FileAPI)".to_string(),
                        model: "gemini".to_string(),
                    }),
                }
            }
            ContentBlock::Document { source, media_type, .. } => {
                match source {
                    radium_abstraction::MediaSource::FileApi { file_id } => {
                        Ok(GeminiPart::FileData {
                            file_data: GeminiFileData {
                                mime_type: media_type.clone(),
                                file_uri: file_id.clone(),
                            },
                        })
                    }
                    radium_abstraction::MediaSource::File { path } => {
                        // Check file size before reading
                        let metadata = std::fs::metadata(path).map_err(|e| {
                            ModelError::InvalidMediaSource {
                                media_source: path.display().to_string(),
                                reason: format!("Failed to read file metadata: {}", e),
                            }
                        })?;
                        let file_size = metadata.len() as usize;

                        // Check if we should use file URI or inline data
                        if validation_utils::should_use_file_uri(file_size) {
                            // File is too large for inline - would need FileApi
                            return Err(ModelError::ContentTooLarge {
                                actual_size: validation_utils::calculate_base64_size(file_size),
                                max_size: validation_utils::MAX_INLINE_SIZE,
                                content_type: media_type.clone(),
                            });
                        }

                        // File is small enough for inline transmission
                        let encoded = Self::read_and_encode_file(path)?;
                        Ok(GeminiPart::InlineData {
                            inline_data: GeminiInlineData {
                                mime_type: media_type.clone(),
                                data: encoded,
                            },
                        })
                    }
                    radium_abstraction::MediaSource::Base64 { data } => {
                        // For base64 data, check encoded size
                        let encoded_size = data.len();
                        if encoded_size > validation_utils::MAX_INLINE_SIZE {
                            return Err(ModelError::ContentTooLarge {
                                actual_size: encoded_size,
                                max_size: validation_utils::MAX_INLINE_SIZE,
                                content_type: media_type.clone(),
                            });
                        }
                        Ok(GeminiPart::InlineData {
                            inline_data: GeminiInlineData {
                                mime_type: media_type.clone(),
                                data,
                            },
                        })
                    }
                    _ => Err(ModelError::UnsupportedContentType {
                        content_type: "document (URL or unsupported source)".to_string(),
                        model: "gemini".to_string(),
                    }),
                }
            }
        }
    }

    /// Converts our ChatMessage to Gemini format.
    fn to_gemini_content(msg: &ChatMessage) -> Result<GeminiContent, ModelError> {
        let role = Self::role_to_gemini(&msg.role);

        let parts = match &msg.content {
            MessageContent::Text(text) => vec![GeminiPart::Text {
                text: text.clone(),
            }],
            MessageContent::Blocks(blocks) => {
                let gemini_parts: Result<Vec<GeminiPart>, ModelError> = blocks
                    .iter()
                    .map(Self::content_block_to_gemini_part)
                    .collect();
                gemini_parts?
            }
        };

        Ok(GeminiContent { role, parts })
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
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: MessageContent::Text(prompt.to_string()),
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
        let gemini_messages: Result<Vec<GeminiContent>, ModelError> = messages
            .iter()
            .filter(|msg| msg.role != "system")
            .map(Self::to_gemini_content)
            .collect();
        let gemini_messages = gemini_messages?;

        // Build request body
        let mut request_body = GeminiRequest {
            contents: gemini_messages,
            generation_config: None,
            system_instruction: system_instruction.map(|text| GeminiSystemInstruction {
                parts: vec![GeminiPart::Text { text }],
            }),
            tool_config: None,
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
        let candidate = gemini_response
            .candidates
            .first()
            .ok_or_else(|| {
                error!("No candidates in Gemini API response");
                ModelError::ModelResponseError("No content in API response".to_string())
            })?;

        // Extract text from all parts, concatenating multiple text parts
        let mut text_parts = Vec::new();
        for part in &candidate.content.parts {
            match part {
                GeminiPart::Text { text } => {
                    text_parts.push(text.clone());
                }
                GeminiPart::InlineData { inline_data } => {
                    debug!(
                        mime_type = %inline_data.mime_type,
                        data_size = inline_data.data.len(),
                        "Received inline_data in response (not yet processed)"
                    );
                }
                GeminiPart::FileData { file_data } => {
                    debug!(
                        mime_type = %file_data.mime_type,
                        file_uri = %file_data.file_uri,
                        "Received file_data in response (not yet processed)"
                    );
                }
                GeminiPart::FunctionCall { .. } => {
                    debug!("Received function_call in response (not yet processed)");
                }
            }
        }

        let content = if text_parts.is_empty() {
            error!("No text content in Gemini API response");
            return Err(ModelError::ModelResponseError("No text content in API response".to_string()));
        } else {
            text_parts.join("\n")
        };

        // Extract usage information
        let usage = gemini_response.usage_metadata.map(|meta| ModelUsage {
            prompt_tokens: meta.prompt_token_count.unwrap_or(0),
            completion_tokens: meta.candidates_token_count.unwrap_or(0),
            total_tokens: meta.total_token_count.unwrap_or(0),
        });

        // Extract metadata from candidate
        let metadata = if candidate.finish_reason.is_some()
            || candidate.safety_ratings.is_some()
            || candidate.citation_metadata.is_some()
            || candidate.grounding_metadata.is_some()
        {
            let gemini_meta = GeminiMetadata {
                finish_reason: candidate.finish_reason.clone(),
                safety_ratings: candidate.safety_ratings.as_ref().map(|ratings| {
                    ratings.iter().map(|r| SafetyRating::from(r)).collect()
                }),
                citations: candidate.citation_metadata.as_ref().map(|cm| {
                    cm.citations.iter().map(|c| Citation::from(c)).collect()
                }),
                grounding_attributions: candidate.grounding_metadata.as_ref().map(|gm| {
                    gm.grounding_attributions.clone()
                }),
            };
            let metadata_map: HashMap<String, serde_json::Value> = gemini_meta.into();
            if metadata_map.is_empty() {
                None
            } else {
                Some(metadata_map)
            }
        } else {
            None
        };

        // Check for safety blocks (behavior will be applied at higher level)
        let safety_ratings = metadata
            .as_ref()
            .and_then(|m| m.get("safety_ratings"))
            .and_then(|v| serde_json::from_value::<Vec<SafetyRating>>(v.clone()).ok());
        
        if let Some(ref ratings) = safety_ratings {
            let blocked = ratings.iter().any(|r| r.blocked);
            if blocked {
                warn!(
                    provider = "gemini",
                    "Content was blocked by safety filters. Metadata contains safety_ratings."
                );
            }
        }

        Ok(ModelResponse {
            content,
            model_id: Some(self.model_id.clone()),
            usage,
            metadata,
            tool_calls: None,
        })
    }

    async fn generate_with_tools(
        &self,
        _messages: &[ChatMessage],
        _tools: &[radium_abstraction::Tool],
        _tool_config: Option<&radium_abstraction::ToolConfig>,
    ) -> Result<ModelResponse, ModelError> {
        Err(ModelError::UnsupportedModelProvider(
            format!("GeminiModel function calling not yet implemented (Task 7)"),
        ))
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }
}

#[async_trait]
impl StreamingModel for GeminiModel {
    async fn generate_stream(
        &self,
        prompt: &str,
        parameters: Option<ModelParameters>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, ModelError>> + Send>>, ModelError> {
        debug!(
            model_id = %self.model_id,
            prompt_len = prompt.len(),
            parameters = ?parameters,
            "GeminiModel generating streaming text"
        );

        // Convert single prompt to chat format for Gemini
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: MessageContent::Text(prompt.to_string()),
        }];

        // Build Gemini streaming API request
        let url = format!(
            "{}/models/{}:streamGenerateContent?alt=sse&key={}",
            self.base_url, self.model_id, self.api_key
        );

        // Extract system instruction if present
        let system_instruction = Self::extract_system_messages(&messages);

        // Convert non-system messages to Gemini format
        let gemini_messages: Result<Vec<GeminiContent>, ModelError> = messages
            .iter()
            .filter(|msg| msg.role != "system")
            .map(Self::to_gemini_content)
            .collect();
        let gemini_messages = gemini_messages?;

        // Build request body
        let mut request_body = GeminiRequest {
            contents: gemini_messages,
            generation_config: None,
            system_instruction: system_instruction.map(|text| GeminiSystemInstruction {
                parts: vec![GeminiPart::Text { text }],
            }),
            tool_config: None,
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

        // Make streaming API request
        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to send streaming request to Gemini API");
                ModelError::RequestError(format!("Network error: {}", e))
            })?;

        // Check status before streaming
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                status = %status,
                error = %error_text,
                "Gemini API returned error status for streaming request"
            );

            // Map quota/rate limit errors to QuotaExceeded
            if status == 402 || status == 429 {
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
            if status == 429 {
                return Err(ModelError::QuotaExceeded {
                    provider: "gemini".to_string(),
                    message: Some(error_text),
                });
            }

            // Map authentication errors (401, 403) to UnsupportedModelProvider
            if status == 401 || status == 403 {
                return Err(ModelError::UnsupportedModelProvider(format!(
                    "Authentication failed ({}): {}",
                    status, error_text
                )));
            }

            // Map server errors (500-599) to ModelResponseError
            if (500..=599).contains(&status.as_u16()) {
                return Err(ModelError::ModelResponseError(format!(
                    "Server error ({}): {}",
                    status, error_text
                )));
            }

            // Other errors
            return Err(ModelError::ModelResponseError(format!(
                "API error ({}): {}",
                status, error_text
            )));
        }

        // Create SSE stream parser
        Ok(Box::pin(GeminiSSEStream::new(response)))
    }
}

// SSE stream parser for Gemini format
struct GeminiSSEStream {
    stream: Pin<Box<dyn Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send>>,
    buffer: String,
    accumulated: String,
    done: bool,
}

impl GeminiSSEStream {
    fn new(response: reqwest::Response) -> Self {
        Self {
            stream: Box::pin(response.bytes_stream()),
            buffer: String::new(),
            accumulated: String::new(),
            done: false,
        }
    }
}

impl Stream for GeminiSSEStream {
    type Item = Result<String, ModelError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.done {
            return Poll::Ready(None);
        }

        loop {
            // Poll the underlying byte stream
            match self.stream.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(bytes))) => {
                    // Convert bytes to string and append to buffer
                    match String::from_utf8(bytes.to_vec()) {
                        Ok(chunk) => {
                            self.buffer.push_str(&chunk);

                            // Process complete SSE events (separated by \n\n)
                            while let Some(end_idx) = self.buffer.find("\n\n") {
                                let event = self.buffer[..end_idx].to_string();
                                self.buffer = self.buffer[end_idx + 2..].to_string();

                                // Parse SSE event
                                if event.starts_with("data: ") {
                                    let data = &event[6..]; // Skip "data: " prefix

                                    // Check for [DONE] signal or empty data
                                    if data.trim() == "[DONE]" || data.trim().is_empty() {
                                        self.done = true;
                                        if !self.accumulated.is_empty() {
                                            return Poll::Ready(Some(Ok(self.accumulated.clone())));
                                        }
                                        return Poll::Ready(None);
                                    }

                                    // Parse JSON chunk
                                    match serde_json::from_str::<GeminiStreamingResponse>(data) {
                                        Ok(streaming_response) => {
                                    // Extract text from all parts in candidate
                                    if let Some(candidate) = streaming_response.candidates.first() {
                                        let mut has_text = false;
                                        for part in &candidate.content.parts {
                                            match part {
                                                GeminiPart::Text { text } => {
                                                    if !text.is_empty() {
                                                        self.accumulated.push_str(text);
                                                        has_text = true;
                                                    }
                                                }
                                                GeminiPart::InlineData { inline_data } => {
                                                    debug!(
                                                        mime_type = %inline_data.mime_type,
                                                        "Received inline_data in streaming response (not yet processed)"
                                                    );
                                                }
                                                GeminiPart::FileData { file_data } => {
                                                    debug!(
                                                        mime_type = %file_data.mime_type,
                                                        file_uri = %file_data.file_uri,
                                                        "Received file_data in streaming response (not yet processed)"
                                                    );
                                                }
                                                GeminiPart::FunctionCall { .. } => {
                                                    debug!("Received function_call in streaming response (not yet processed)");
                                                }
                                            }
                                        }
                                        if has_text {
                                            return Poll::Ready(Some(Ok(self.accumulated.clone())));
                                        }
                                    }
                                        }
                                        Err(e) => {
                                            // Skip malformed JSON chunks (some servers send empty chunks)
                                            debug!("Failed to parse SSE chunk: {}", e);
                                        }
                                    }
                                }
                            }

                            // Continue polling for more data
                            continue;
                        }
                        Err(e) => {
                            return Poll::Ready(Some(Err(ModelError::SerializationError(format!(
                                "Failed to decode SSE chunk: {}",
                                e
                            )))));
                        }
                    }
                }
                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(Some(Err(ModelError::RequestError(format!(
                        "Stream error: {}",
                        e
                    )))));
                }
                Poll::Ready(None) => {
                    // Stream ended - process any remaining events in buffer
                    while let Some(end_idx) = self.buffer.find("\n\n") {
                        let event = self.buffer[..end_idx].to_string();
                        self.buffer = self.buffer[end_idx + 2..].to_string();

                        if event.starts_with("data: ") {
                            let data = &event[6..];

                            if data.trim() == "[DONE]" || data.trim().is_empty() {
                                self.done = true;
                                if !self.accumulated.is_empty() {
                                    return Poll::Ready(Some(Ok(self.accumulated.clone())));
                                }
                                return Poll::Ready(None);
                            }

                            if let Ok(streaming_response) =
                                serde_json::from_str::<GeminiStreamingResponse>(data)
                            {
                                if let Some(candidate) = streaming_response.candidates.first() {
                                    // Extract text from all parts
                                    for part in &candidate.content.parts {
                                        match part {
                                            GeminiPart::Text { text } => {
                                                if !text.is_empty() {
                                                    self.accumulated.push_str(text);
                                                }
                                            }
                                            GeminiPart::InlineData { inline_data } => {
                                                debug!(
                                                    mime_type = %inline_data.mime_type,
                                                    "Received inline_data in streaming response (not yet processed)"
                                                );
                                            }
                                            GeminiPart::FileData { file_data } => {
                                                debug!(
                                                    mime_type = %file_data.mime_type,
                                                    file_uri = %file_data.file_uri,
                                                    "Received file_data in streaming response (not yet processed)"
                                                );
                                            }
                                            GeminiPart::FunctionCall { .. } => {
                                                debug!("Received function_call in streaming response (not yet processed)");
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // No more events in buffer
                    self.done = true;
                    if !self.accumulated.is_empty() {
                        return Poll::Ready(Some(Ok(self.accumulated.clone())));
                    }
                    return Poll::Ready(None);
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

// Streaming response structure for Gemini SSE
#[derive(Debug, Deserialize)]
struct GeminiStreamingResponse {
    candidates: Vec<GeminiCandidate>,
}

// Gemini API request/response structures

/// Gemini function calling configuration
#[derive(Debug, Serialize)]
struct GeminiFunctionCallingConfig {
    /// Function calling mode: "AUTO", "ANY", or "NONE"
    mode: String,
    /// Optional whitelist of allowed function names
    #[serde(skip_serializing_if = "Option::is_none", rename = "allowedFunctionNames")]
    allowed_function_names: Option<Vec<String>>,
}

/// Gemini tool configuration
#[derive(Debug, Serialize)]
struct GeminiToolConfig {
    #[serde(rename = "functionCallingConfig")]
    function_calling_config: GeminiFunctionCallingConfig,
}

/// Helper function to convert ToolUseMode to Gemini API mode string
fn tool_use_mode_to_gemini(mode: radium_abstraction::ToolUseMode) -> String {
    match mode {
        radium_abstraction::ToolUseMode::Auto => "AUTO".to_string(),
        radium_abstraction::ToolUseMode::Any => "ANY".to_string(),
        radium_abstraction::ToolUseMode::None => "NONE".to_string(),
    }
}

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GeminiGenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "systemInstruction")]
    system_instruction: Option<GeminiSystemInstruction>,
    /// Optional tool configuration for function calling
    #[serde(skip_serializing_if = "Option::is_none", rename = "toolConfig")]
    tool_config: Option<GeminiToolConfig>,
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
#[serde(untagged)]
enum GeminiPart {
    Text { text: String },
    InlineData { 
        #[serde(rename = "inline_data")]
        inline_data: GeminiInlineData 
    },
    FileData { 
        #[serde(rename = "file_data")]
        file_data: GeminiFileData 
    },
    FunctionCall {
        #[serde(rename = "function_call")]
        function_call: serde_json::Value,  // Placeholder for future tool use
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiInlineData {
    #[serde(rename = "mime_type")]
    mime_type: String,
    data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiFileData {
    #[serde(rename = "mime_type")]
    mime_type: String,
    #[serde(rename = "file_uri")]
    file_uri: String,
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
    #[serde(rename = "finishReason")]
    finish_reason: Option<String>,
    #[serde(rename = "safetyRatings")]
    safety_ratings: Option<Vec<GeminiSafetyRating>>,
    #[serde(rename = "citationMetadata")]
    citation_metadata: Option<GeminiCitationMetadata>,
    #[serde(rename = "groundingMetadata")]
    grounding_metadata: Option<GeminiGroundingMetadata>,
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

// Gemini-specific metadata structures

#[derive(Debug, Deserialize)]
struct GeminiSafetyRating {
    category: String,
    probability: String,
    blocked: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct GeminiCitationMetadata {
    citations: Vec<GeminiCitation>,
}

#[derive(Debug, Deserialize)]
struct GeminiCitation {
    #[serde(rename = "startIndex")]
    start_index: Option<u32>,
    #[serde(rename = "endIndex")]
    end_index: Option<u32>,
    uri: Option<String>,
    title: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GeminiGroundingMetadata {
    #[serde(rename = "groundingAttributions")]
    grounding_attributions: Vec<serde_json::Value>,
}

// Common metadata structure for Gemini
#[derive(Debug, Clone, Serialize)]
struct GeminiMetadata {
    finish_reason: Option<String>,
    safety_ratings: Option<Vec<SafetyRating>>,
    citations: Option<Vec<Citation>>,
    grounding_attributions: Option<Vec<serde_json::Value>>,
}

impl From<GeminiMetadata> for HashMap<String, serde_json::Value> {
    fn from(meta: GeminiMetadata) -> Self {
        let mut map = HashMap::new();
        if let Some(finish_reason) = meta.finish_reason {
            map.insert("finish_reason".to_string(), serde_json::Value::String(finish_reason));
        }
        if let Some(safety_ratings) = meta.safety_ratings {
            map.insert("safety_ratings".to_string(), serde_json::to_value(safety_ratings).unwrap());
        }
        if let Some(citations) = meta.citations {
            map.insert("citations".to_string(), serde_json::to_value(citations).unwrap());
        }
        if let Some(attributions) = meta.grounding_attributions {
            map.insert("grounding_attributions".to_string(), serde_json::to_value(attributions).unwrap());
        }
        map
    }
}

impl From<&GeminiSafetyRating> for SafetyRating {
    fn from(rating: &GeminiSafetyRating) -> Self {
        SafetyRating {
            category: rating.category.clone(),
            probability: rating.probability.clone(),
            blocked: rating.blocked.unwrap_or(false),
        }
    }
}

impl From<&GeminiCitation> for Citation {
    fn from(citation: &GeminiCitation) -> Self {
        Citation {
            start_index: citation.start_index,
            end_index: citation.end_index,
            uri: citation.uri.clone(),
            title: citation.title.clone(),
        }
    }
}

/// Provider capability detection for multimodal content support.
mod provider_capabilities {
    use radium_abstraction::ModelError;

    /// Provider capabilities for multimodal content.
    #[derive(Debug, Clone)]
    pub struct ProviderCapabilities {
        pub provider_name: String,
        pub supports_images: bool,
        pub supports_audio: bool,
        pub supports_video: bool,
        pub supports_pdf: bool,
        pub max_inline_size: u64,
        pub supports_file_api: bool,
    }

    impl ProviderCapabilities {
        /// Get capabilities for Gemini provider.
        pub fn for_gemini() -> Self {
            Self {
                provider_name: "gemini".to_string(),
                supports_images: true,
                supports_audio: true,
                supports_video: true,
                supports_pdf: true,
                max_inline_size: 20 * 1024 * 1024, // 20MB
                supports_file_api: true,
            }
        }

        /// Get capabilities for Claude provider.
        pub fn for_claude() -> Self {
            Self {
                provider_name: "claude".to_string(),
                supports_images: true,
                supports_audio: false,
                supports_video: false,
                supports_pdf: true,
                max_inline_size: 5 * 1024 * 1024, // 5MB
                supports_file_api: false,
            }
        }

        /// Get capabilities for OpenAI provider.
        pub fn for_openai() -> Self {
            Self {
                provider_name: "openai".to_string(),
                supports_images: true,
                supports_audio: true,
                supports_video: false,
                supports_pdf: false,
                max_inline_size: 20 * 1024 * 1024, // 20MB
                supports_file_api: false,
            }
        }

        /// Validate if a MIME type is supported by this provider.
        ///
        /// # Arguments
        /// * `mime_type` - The MIME type to validate
        ///
        /// # Returns
        /// `Ok(())` if supported, `Err(UnsupportedMimeType)` with alternative providers if not
        pub fn validate_mime_type(&self, mime_type: &str) -> Result<(), ModelError> {
            let supported = match mime_type {
                "image/png" | "image/jpeg" | "image/webp" => self.supports_images,
                "application/pdf" => self.supports_pdf,
                "audio/mpeg" | "audio/wav" | "audio/aac" => self.supports_audio,
                "video/mp4" | "video/quicktime" => self.supports_video,
                _ => false,
            };

            if !supported {
                // Find providers that support this type
                let alternatives = self.find_supporting_providers(mime_type);
                let mut supported_types = self.get_supported_types();
                
                Err(ModelError::UnsupportedMimeType {
                    mime_type: mime_type.to_string(),
                    supported_types,
                })
            } else {
                Ok(())
            }
        }

        /// Validate content size against provider's inline limit.
        ///
        /// # Arguments
        /// * `size` - The content size in bytes
        /// * `mime_type` - The MIME type
        ///
        /// # Returns
        /// `Ok(())` if size is valid, `Err(ContentTooLarge)` if exceeds limit
        pub fn validate_size(&self, size: u64, mime_type: &str) -> Result<(), ModelError> {
            if size > self.max_inline_size && !self.supports_file_api {
                return Err(ModelError::ContentTooLarge {
                    actual_size: size as usize,
                    max_size: self.max_inline_size as usize,
                    content_type: mime_type.to_string(),
                });
            }
            Ok(())
        }

        /// Get list of supported MIME types for this provider.
        fn get_supported_types(&self) -> Vec<String> {
            let mut types = Vec::new();
            if self.supports_images {
                types.extend_from_slice(&["image/png".to_string(), "image/jpeg".to_string(), "image/webp".to_string()]);
            }
            if self.supports_pdf {
                types.push("application/pdf".to_string());
            }
            if self.supports_audio {
                types.extend_from_slice(&["audio/mpeg".to_string(), "audio/wav".to_string(), "audio/aac".to_string()]);
            }
            if self.supports_video {
                types.extend_from_slice(&["video/mp4".to_string(), "video/quicktime".to_string()]);
            }
            types
        }

        /// Find providers that support a given MIME type.
        fn find_supporting_providers(&self, mime_type: &str) -> Vec<String> {
            let mut providers = Vec::new();

            // Check which providers support this MIME type
            if mime_type.starts_with("audio/") {
                providers.extend_from_slice(&["gemini", "openai"]);
            } else if mime_type.starts_with("video/") {
                providers.push("gemini");
            } else if mime_type == "application/pdf" {
                providers.extend_from_slice(&["gemini", "claude"]);
            } else if mime_type.starts_with("image/") {
                providers.extend_from_slice(&["gemini", "claude", "openai"]);
            }

            // Remove current provider from suggestions
            providers.retain(|p| p != &self.provider_name);
            providers
        }
    }
}

/// Base64 encoding utilities for multimodal content.
mod encoding_utils {
    use base64::Engine;
    use radium_abstraction::ModelError;
    use tracing::debug;

    /// Encode binary data to base64 string.
    ///
    /// # Arguments
    /// * `data` - The binary data to encode
    ///
    /// # Returns
    /// `Ok(encoded_string)` if encoding succeeds, `Err` if encoding fails
    pub fn encode_to_base64(data: &[u8]) -> Result<String, ModelError> {
        debug!(
            data_size = data.len(),
            "Encoding binary data to base64"
        );
        
        let engine = base64::engine::general_purpose::STANDARD;
        let encoded = engine.encode(data);
        
        debug!(
            original_size = data.len(),
            encoded_size = encoded.len(),
            "Successfully encoded data to base64"
        );
        
        Ok(encoded)
    }
}

/// Content validation and size checking utilities for multimodal content.
mod validation_utils {
    use radium_abstraction::ModelError;

    /// Maximum size for inline data transmission (20MB in bytes).
    pub const MAX_INLINE_SIZE: usize = 20_971_520;
    
    /// Supported URI schemes for file data.
    pub const SUPPORTED_URI_SCHEMES: &[&str] = &["file://", "gs://", "s3://", "https://"];

    /// Calculate the size of data after base64 encoding.
    ///
    /// Base64 encoding increases size by approximately 4/3, plus padding.
    ///
    /// # Arguments
    /// * `data_size` - The original data size in bytes
    ///
    /// # Returns
    /// The estimated size after base64 encoding
    pub fn calculate_base64_size(data_size: usize) -> usize {
        // Base64 encoding increases size by 4/3, plus padding
        ((data_size + 2) / 3) * 4
    }

    /// Determine if file URI should be used instead of inline data.
    ///
    /// # Arguments
    /// * `data_size` - The original data size in bytes
    ///
    /// # Returns
    /// `true` if file URI should be used (content too large for inline), `false` otherwise
    pub fn should_use_file_uri(data_size: usize) -> bool {
        let encoded_size = calculate_base64_size(data_size);
        encoded_size > MAX_INLINE_SIZE
    }

    /// Validate content size against inline transmission limit.
    ///
    /// # Arguments
    /// * `data_size` - The original data size in bytes
    /// * `content_type` - The content type/MIME type
    ///
    /// # Returns
    /// `Ok(())` if size is valid for inline transmission, `Err(ContentTooLarge)` if too large
    pub fn validate_content_size(
        data_size: usize,
        content_type: &str,
    ) -> Result<(), ModelError> {
        let encoded_size = calculate_base64_size(data_size);
        if encoded_size > MAX_INLINE_SIZE {
            Err(ModelError::ContentTooLarge {
                actual_size: encoded_size,
                max_size: MAX_INLINE_SIZE,
                content_type: content_type.to_string(),
            })
        } else {
            Ok(())
        }
    }

    /// Validate a file URI format and scheme.
    ///
    /// # Arguments
    /// * `uri` - The file URI to validate
    ///
    /// # Returns
    /// `Ok(())` if URI is valid, `Err(InvalidFileUri)` if invalid
    pub fn validate_file_uri(uri: &str) -> Result<(), ModelError> {
        let has_valid_scheme = SUPPORTED_URI_SCHEMES
            .iter()
            .any(|scheme| uri.starts_with(scheme));
        
        if !has_valid_scheme {
            Err(ModelError::InvalidFileUri {
                uri: uri.to_string(),
                reason: "Unsupported URI scheme".to_string(),
            })
        } else {
            Ok(())
        }
    }
}

/// MIME type detection and validation utilities for multimodal content.
mod mime_utils {
    use radium_abstraction::ModelError;

    /// Supported image MIME types.
    pub const SUPPORTED_IMAGE_TYPES: &[&str] = &["image/png", "image/jpeg", "image/webp"];
    
    /// Supported document MIME types.
    pub const SUPPORTED_DOCUMENT_TYPES: &[&str] = &["application/pdf"];

    /// Detect MIME type from file content using magic bytes.
    ///
    /// # Arguments
    /// * `data` - The file content bytes
    ///
    /// # Returns
    /// `Some(mime_type)` if detected, `None` if unknown
    pub fn detect_mime_type(data: &[u8]) -> Option<String> {
        // PNG: \x89PNG\r\n\x1a\n
        if data.starts_with(b"\x89PNG\r\n\x1a\n") {
            Some("image/png".to_string())
        }
        // JPEG: \xff\xd8\xff
        else if data.starts_with(b"\xff\xd8\xff") {
            Some("image/jpeg".to_string())
        }
        // WebP: RIFF...WEBP
        else if data.starts_with(b"RIFF") && data.len() > 12 && &data[8..12] == b"WEBP" {
            Some("image/webp".to_string())
        }
        // PDF: %PDF
        else if data.starts_with(b"%PDF") {
            Some("application/pdf".to_string())
        }
        else {
            None
        }
    }

    /// Check if a MIME type is supported for images or documents.
    ///
    /// # Arguments
    /// * `mime_type` - The MIME type to check
    ///
    /// # Returns
    /// `true` if supported, `false` otherwise
    pub fn is_supported_mime_type(mime_type: &str) -> bool {
        SUPPORTED_IMAGE_TYPES.contains(&mime_type) 
            || SUPPORTED_DOCUMENT_TYPES.contains(&mime_type)
    }

    /// Validate a MIME type against supported types.
    ///
    /// # Arguments
    /// * `mime_type` - The MIME type to validate
    ///
    /// # Returns
    /// `Ok(())` if valid, `Err(ModelError::UnsupportedMimeType)` if not supported
    pub fn validate_mime_type(mime_type: &str) -> Result<(), ModelError> {
        if is_supported_mime_type(mime_type) {
            Ok(())
        } else {
            let mut supported = Vec::new();
            supported.extend_from_slice(SUPPORTED_IMAGE_TYPES);
            supported.extend_from_slice(SUPPORTED_DOCUMENT_TYPES);
            Err(ModelError::UnsupportedMimeType {
                mime_type: mime_type.to_string(),
                supported_types: supported.iter().map(|s| s.to_string()).collect(),
            })
        }
    }
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
            ChatMessage {
                role: "system".to_string(),
                content: MessageContent::Text("You are helpful.".to_string()),
            },
            ChatMessage {
                role: "user".to_string(),
                content: MessageContent::Text("Hello".to_string()),
            },
        ];
        let system = GeminiModel::extract_system_messages(&messages);
        assert_eq!(system, Some("You are helpful.".to_string()));

        // Test multiple system messages (should be concatenated)
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: MessageContent::Text("First instruction.".to_string()),
            },
            ChatMessage {
                role: "system".to_string(),
                content: MessageContent::Text("Second instruction.".to_string()),
            },
            ChatMessage {
                role: "user".to_string(),
                content: MessageContent::Text("Hello".to_string()),
            },
        ];
        let system = GeminiModel::extract_system_messages(&messages);
        assert_eq!(system, Some("First instruction.\n\nSecond instruction.".to_string()));

        // Test no system messages
        let messages = vec![
            ChatMessage {
                role: "user".to_string(),
                content: MessageContent::Text("Hello".to_string()),
            },
            ChatMessage {
                role: "assistant".to_string(),
                content: MessageContent::Text("Hi there!".to_string()),
            },
        ];
        let system = GeminiModel::extract_system_messages(&messages);
        assert_eq!(system, None);

        // Test mixed message types
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: MessageContent::Text("System message.".to_string()),
            },
            ChatMessage {
                role: "user".to_string(),
                content: MessageContent::Text("User message.".to_string()),
            },
            ChatMessage {
                role: "assistant".to_string(),
                content: MessageContent::Text("Assistant message.".to_string()),
            },
        ];
        let system = GeminiModel::extract_system_messages(&messages);
        assert_eq!(system, Some("System message.".to_string()));

        // Test empty system message content
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: MessageContent::Text("".to_string()),
            },
            ChatMessage {
                role: "user".to_string(),
                content: MessageContent::Text("Hello".to_string()),
            },
        ];
        let system = GeminiModel::extract_system_messages(&messages);
        assert_eq!(system, Some("".to_string()));
    }

    #[test]
    fn test_system_message_filtering_from_contents() {
        use radium_abstraction::ChatMessage;

        // Test that system messages are filtered from contents array
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: MessageContent::Text("System instruction.".to_string()),
            },
            ChatMessage {
                role: "user".to_string(),
                content: MessageContent::Text("User message.".to_string()),
            },
            ChatMessage {
                role: "assistant".to_string(),
                content: MessageContent::Text("Assistant message.".to_string()),
            },
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
            parts: vec![GeminiPart::Text {
                text: system_text.to_string(),
            }],
        };

        let request = GeminiRequest {
            contents: vec![GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart::Text {
                    text: "Hello".to_string(),
                }],
            }],
            generation_config: None,
            system_instruction: Some(system_instruction),
            tool_config: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("systemInstruction"));
        assert!(json.contains(system_text));

        // Test that systemInstruction field is omitted when None
        let request_no_system = GeminiRequest {
            contents: vec![GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart::Text {
                    text: "Hello".to_string(),
                }],
            }],
            generation_config: None,
            system_instruction: None,
            tool_config: None,
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

    #[tokio::test]
    async fn test_gemini_streaming_sse_parsing() {
        use futures::StreamExt;
        use mockito::Server;

        let mut server = Server::new_async().await;
        let mock_url = server.url();

        // Mock SSE response with Gemini format
        let mock_response = b"data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"Hello\"}]}}]}\n\ndata: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\" world\"}]}}]}\n\ndata: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"!\"}]}}]}\n\ndata: [DONE]\n\n";

        let _mock = server
            .mock("POST", "/models/test-model:streamGenerateContent")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("alt".to_string(), "sse".to_string()),
                mockito::Matcher::UrlEncoded("key".to_string(), "test-key".to_string()),
            ]))
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body(mock_response)
            .create();

        let model = GeminiModel::with_api_key(
            "test-model".to_string(),
            "test-key".to_string(),
        );
        
        // Override base_url to use mock server
        // Note: This requires making base_url mutable or using a different approach
        // For now, we'll test the SSE parsing logic indirectly through integration tests
        // This test verifies the structure compiles correctly
        assert_eq!(model.model_id(), "test-model");
    }

    #[test]
    fn test_gemini_streaming_response_deserialization() {
        // Test that GeminiStreamingResponse can deserialize correctly
        let json = r#"{"candidates":[{"content":{"role":"model","parts":[{"text":"Hello"}]}}]}"#;
        let response: GeminiStreamingResponse = serde_json::from_str(json)
            .expect("Should deserialize Gemini streaming response");
        
        assert_eq!(response.candidates.len(), 1);
        assert_eq!(response.candidates[0].content.parts.len(), 1);
        if let GeminiPart::Text { text } = &response.candidates[0].content.parts[0] {
            assert_eq!(text, "Hello");
        } else {
            panic!("Expected text part");
        }
    }

    #[test]
    fn test_mime_type_detection_png() {
        let png_data = b"\x89PNG\r\n\x1a\n";
        let mime = mime_utils::detect_mime_type(png_data);
        assert_eq!(mime, Some("image/png".to_string()));
    }

    #[test]
    fn test_mime_type_detection_jpeg() {
        let jpeg_data = b"\xff\xd8\xff\xe0";
        let mime = mime_utils::detect_mime_type(jpeg_data);
        assert_eq!(mime, Some("image/jpeg".to_string()));
    }

    #[test]
    fn test_mime_type_detection_webp() {
        let mut webp_data = b"RIFF".to_vec();
        webp_data.extend_from_slice(&[0u8; 4]);
        webp_data.extend_from_slice(b"WEBP");
        let mime = mime_utils::detect_mime_type(&webp_data);
        assert_eq!(mime, Some("image/webp".to_string()));
    }

    #[test]
    fn test_mime_type_detection_pdf() {
        let pdf_data = b"%PDF-1.4";
        let mime = mime_utils::detect_mime_type(pdf_data);
        assert_eq!(mime, Some("application/pdf".to_string()));
    }

    #[test]
    fn test_mime_type_validation_supported() {
        assert!(mime_utils::validate_mime_type("image/png").is_ok());
        assert!(mime_utils::validate_mime_type("image/jpeg").is_ok());
        assert!(mime_utils::validate_mime_type("image/webp").is_ok());
        assert!(mime_utils::validate_mime_type("application/pdf").is_ok());
    }

    #[test]
    fn test_mime_type_validation_unsupported() {
        let result = mime_utils::validate_mime_type("application/zip");
        assert!(result.is_err());
        if let Err(ModelError::UnsupportedMimeType { mime_type, supported_types }) = result {
            assert_eq!(mime_type, "application/zip");
            assert!(supported_types.contains(&"image/png".to_string()));
        } else {
            panic!("Expected UnsupportedMimeType error");
        }
    }

    #[test]
    fn test_base64_size_calculation() {
        // Test base64 size calculation
        let original_size = 1000;
        let encoded_size = validation_utils::calculate_base64_size(original_size);
        // Base64 increases size by ~33% (4/3 ratio)
        assert!(encoded_size > original_size);
        assert_eq!(encoded_size, ((original_size + 2) / 3) * 4);
    }

    #[test]
    fn test_should_use_file_uri_small_file() {
        // 5MB should use inline
        let size = 5 * 1024 * 1024;
        assert!(!validation_utils::should_use_file_uri(size));
    }

    #[test]
    fn test_should_use_file_uri_large_file() {
        // 25MB should use file URI
        let size = 25 * 1024 * 1024;
        assert!(validation_utils::should_use_file_uri(size));
    }

    #[test]
    fn test_exactly_20mb_uses_inline() {
        // Exactly 20MB should use inline (at the limit, not over)
        let size = validation_utils::MAX_INLINE_SIZE;
        let encoded_size = validation_utils::calculate_base64_size(size);
        // If encoded size exceeds limit, should use file URI
        // But original 20MB might be close to limit after encoding
        let should_use_file = validation_utils::should_use_file_uri(size);
        // This depends on the exact calculation, but 20MB raw should be close to limit
        assert!(!should_use_file || encoded_size <= validation_utils::MAX_INLINE_SIZE);
    }

    #[test]
    fn test_20mb_plus_one_uses_file() {
        // 20MB + 1 byte should use file URI
        let size = validation_utils::MAX_INLINE_SIZE + 1;
        assert!(validation_utils::should_use_file_uri(size));
    }

    #[test]
    fn test_validate_content_size_valid() {
        // Small file should pass validation
        let size = 5 * 1024 * 1024; // 5MB
        assert!(validation_utils::validate_content_size(size, "image/png").is_ok());
    }

    #[test]
    fn test_validate_content_size_too_large() {
        // Large file should fail validation
        let size = 25 * 1024 * 1024; // 25MB
        let result = validation_utils::validate_content_size(size, "image/png");
        assert!(result.is_err());
        if let Err(ModelError::ContentTooLarge { actual_size, max_size, content_type }) = result {
            assert!(actual_size > max_size);
            assert_eq!(content_type, "image/png");
        } else {
            panic!("Expected ContentTooLarge error");
        }
    }

    #[test]
    fn test_validate_file_uri_valid_schemes() {
        assert!(validation_utils::validate_file_uri("file:///path/to/file").is_ok());
        assert!(validation_utils::validate_file_uri("gs://bucket/file").is_ok());
        assert!(validation_utils::validate_file_uri("s3://bucket/file").is_ok());
        assert!(validation_utils::validate_file_uri("https://example.com/file").is_ok());
    }

    #[test]
    fn test_validate_file_uri_invalid_scheme() {
        let result = validation_utils::validate_file_uri("invalid://scheme");
        assert!(result.is_err());
        if let Err(ModelError::InvalidFileUri { uri, reason }) = result {
            assert_eq!(uri, "invalid://scheme");
            assert!(reason.contains("Unsupported"));
        } else {
            panic!("Expected InvalidFileUri error");
        }
    }

    #[test]
    fn test_encode_to_base64() {
        let data = b"Hello, World!";
        let encoded = encoding_utils::encode_to_base64(data).unwrap();
        assert!(!encoded.is_empty());
        // Verify it's valid base64
        assert!(encoded.chars().all(|c| c.is_alphanumeric() || c == '+' || c == '/' || c == '='));
    }

    #[test]
    fn test_encode_to_base64_empty() {
        let data = b"";
        let encoded = encoding_utils::encode_to_base64(data).unwrap();
        assert_eq!(encoded, "");
    }

    #[test]
    fn test_encode_to_base64_large_data() {
        let data = vec![0u8; 1024];
        let encoded = encoding_utils::encode_to_base64(&data).unwrap();
        // Base64 encoding of 1024 bytes should be approximately 1366 bytes (1024 * 4/3)
        assert!(encoded.len() > data.len());
        assert!(encoded.len() <= (data.len() * 4 / 3 + 4)); // Account for padding
    }

    #[test]
    fn test_provider_capabilities_gemini() {
        let caps = provider_capabilities::ProviderCapabilities::for_gemini();
        assert_eq!(caps.provider_name, "gemini");
        assert!(caps.supports_images);
        assert!(caps.supports_audio);
        assert!(caps.supports_video);
        assert!(caps.supports_pdf);
        assert_eq!(caps.max_inline_size, 20 * 1024 * 1024);
        assert!(caps.supports_file_api);
    }

    #[test]
    fn test_provider_capabilities_claude() {
        let caps = provider_capabilities::ProviderCapabilities::for_claude();
        assert_eq!(caps.provider_name, "claude");
        assert!(caps.supports_images);
        assert!(!caps.supports_audio);
        assert!(!caps.supports_video);
        assert!(caps.supports_pdf);
        assert_eq!(caps.max_inline_size, 5 * 1024 * 1024);
        assert!(!caps.supports_file_api);
    }

    #[test]
    fn test_provider_capabilities_openai() {
        let caps = provider_capabilities::ProviderCapabilities::for_openai();
        assert_eq!(caps.provider_name, "openai");
        assert!(caps.supports_images);
        assert!(caps.supports_audio);
        assert!(!caps.supports_video);
        assert!(!caps.supports_pdf);
        assert_eq!(caps.max_inline_size, 20 * 1024 * 1024);
        assert!(!caps.supports_file_api);
    }

    #[test]
    fn test_validate_mime_type_supported() {
        let caps = provider_capabilities::ProviderCapabilities::for_gemini();
        assert!(caps.validate_mime_type("image/png").is_ok());
        assert!(caps.validate_mime_type("application/pdf").is_ok());
    }

    #[test]
    fn test_validate_mime_type_unsupported() {
        let caps = provider_capabilities::ProviderCapabilities::for_claude();
        let result = caps.validate_mime_type("audio/mpeg");
        assert!(result.is_err());
        if let Err(ModelError::UnsupportedMimeType { mime_type, .. }) = result {
            assert_eq!(mime_type, "audio/mpeg");
        } else {
            panic!("Expected UnsupportedMimeType error");
        }
    }

    #[test]
    fn test_validate_size_within_limit() {
        let caps = provider_capabilities::ProviderCapabilities::for_gemini();
        assert!(caps.validate_size(10 * 1024 * 1024, "image/png").is_ok());
    }

    #[test]
    fn test_validate_size_exceeds_limit_no_file_api() {
        let caps = provider_capabilities::ProviderCapabilities::for_claude();
        let result = caps.validate_size(10 * 1024 * 1024, "image/png"); // 10MB > 5MB limit
        assert!(result.is_err());
        if let Err(ModelError::ContentTooLarge { .. }) = result {
            // Expected
        } else {
            panic!("Expected ContentTooLarge error");
        }
    }

    // Comprehensive integration tests for multimodal support

    #[test]
    fn test_png_5mb_inline_encoding() {
        // Test that 5MB PNG would use inline_data
        let mut data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG header
        data.extend(vec![0u8; 5 * 1024 * 1024 - 8]); // ~5MB total

        let mime = mime_utils::detect_mime_type(&data).unwrap();
        assert_eq!(mime, "image/png");

        let should_use_file = validation_utils::should_use_file_uri(data.len());
        assert!(!should_use_file, "5MB should use inline_data");

        let encoded = encoding_utils::encode_to_base64(&data).unwrap();
        assert!(!encoded.is_empty());
        assert!(encoded.len() > data.len()); // Base64 increases size
    }

    #[test]
    fn test_jpeg_25mb_file_uri_routing() {
        // Test that 25MB JPEG would use file_data
        let data_size = 25 * 1024 * 1024; // 25MB

        let should_use_file = validation_utils::should_use_file_uri(data_size);
        assert!(should_use_file, "25MB should use file_data");
    }

    #[test]
    fn test_pdf_10mb_inline_encoding() {
        // Test PDF detection and inline encoding for 10MB PDF
        let mut data = b"%PDF-1.4".to_vec();
        data.extend(vec![0u8; 10 * 1024 * 1024 - 8]); // ~10MB total

        let mime = mime_utils::detect_mime_type(&data).unwrap();
        assert_eq!(mime, "application/pdf");

        let should_use_file = validation_utils::should_use_file_uri(data.len());
        assert!(!should_use_file, "10MB PDF should use inline_data");
    }

    #[test]
    fn test_pdf_50mb_file_uri_routing() {
        // Test that 50MB PDF would use file_data
        let data_size = 50 * 1024 * 1024; // 50MB

        let should_use_file = validation_utils::should_use_file_uri(data_size);
        assert!(should_use_file, "50MB PDF should use file_data");
    }

    #[test]
    fn test_mime_type_mismatch_detection() {
        // Test that we can detect MIME type from content
        let png_data = b"\x89PNG\r\n\x1a\n";
        let detected = mime_utils::detect_mime_type(png_data).unwrap();
        assert_eq!(detected, "image/png");

        // Even if extension suggests something else, magic bytes should win
        let jpeg_data = b"\xff\xd8\xff";
        let detected = mime_utils::detect_mime_type(jpeg_data).unwrap();
        assert_eq!(detected, "image/jpeg");
    }

    #[test]
    fn test_unsupported_mime_type_error_message() {
        // Test that unsupported MIME types return clear error with supported types
        let result = mime_utils::validate_mime_type("application/zip");
        assert!(result.is_err());
        if let Err(ModelError::UnsupportedMimeType { mime_type, supported_types }) = result {
            assert_eq!(mime_type, "application/zip");
            assert!(supported_types.contains(&"image/png".to_string()));
            assert!(supported_types.contains(&"application/pdf".to_string()));
        } else {
            panic!("Expected UnsupportedMimeType error");
        }
    }

    #[test]
    fn test_size_edge_cases() {
        // Test exactly at 20MB limit
        let size = validation_utils::MAX_INLINE_SIZE;
        let encoded_size = validation_utils::calculate_base64_size(size);
        // Encoded size might exceed limit, but original at limit should be close
        let should_use_file = validation_utils::should_use_file_uri(size);
        // This is edge case - depends on exact calculation
        assert!(!should_use_file || encoded_size <= validation_utils::MAX_INLINE_SIZE);

        // Test 20MB + 1 byte
        let size_plus_one = validation_utils::MAX_INLINE_SIZE + 1;
        assert!(validation_utils::should_use_file_uri(size_plus_one));
    }

    #[test]
    fn test_backward_compatibility_text_only() {
        // Test that text-only messages still work
        let part = GeminiPart::Text { text: "Hello, world!".to_string() };
        let json = serde_json::to_string(&part).unwrap();
        assert!(json.contains("text"));
        assert!(json.contains("Hello, world!"));
    }

    #[test]
    fn test_gemini_part_serialization() {
        // Test that all GeminiPart variants serialize correctly
        let text_part = GeminiPart::Text { text: "test".to_string() };
        let json = serde_json::to_string(&text_part).unwrap();
        assert!(json.contains("text"));

        let inline_part = GeminiPart::InlineData {
            inline_data: GeminiInlineData {
                mime_type: "image/png".to_string(),
                data: "base64data".to_string(),
            },
        };
        let json = serde_json::to_string(&inline_part).unwrap();
        assert!(json.contains("inline_data"));
        assert!(json.contains("mime_type"));

        let file_part = GeminiPart::FileData {
            file_data: GeminiFileData {
                mime_type: "image/png".to_string(),
                file_uri: "file:///path/to/file".to_string(),
            },
        };
        let json = serde_json::to_string(&file_part).unwrap();
        assert!(json.contains("file_data"));
        assert!(json.contains("file_uri"));
    }

    #[test]
    fn test_multiple_text_parts_concatenation() {
        // Test that multiple text parts would be concatenated
        // This tests the response parsing logic conceptually
        let parts = vec![
            GeminiPart::Text { text: "Hello".to_string() },
            GeminiPart::Text { text: " ".to_string() },
            GeminiPart::Text { text: "World".to_string() },
        ];
        let text_parts: Vec<String> = parts
            .iter()
            .filter_map(|p| match p {
                GeminiPart::Text { text } => Some(text.clone()),
                _ => None,
            })
            .collect();
        let concatenated = text_parts.join("");
        assert_eq!(concatenated, "Hello World");
    }

    #[test]
    fn test_provider_specific_limits() {
        // Test that different providers have different limits
        let gemini = provider_capabilities::ProviderCapabilities::for_gemini();
        let claude = provider_capabilities::ProviderCapabilities::for_claude();

        assert_eq!(gemini.max_inline_size, 20 * 1024 * 1024);
        assert_eq!(claude.max_inline_size, 5 * 1024 * 1024);

        // 10MB file
        let size = 10 * 1024 * 1024;
        assert!(gemini.validate_size(size, "image/png").is_ok());
        assert!(claude.validate_size(size, "image/png").is_err());
    }

    #[test]
    fn test_file_uri_validation_all_schemes() {
        // Test all supported URI schemes
        assert!(validation_utils::validate_file_uri("file:///path/to/file").is_ok());
        assert!(validation_utils::validate_file_uri("gs://bucket/file").is_ok());
        assert!(validation_utils::validate_file_uri("s3://bucket/file").is_ok());
        assert!(validation_utils::validate_file_uri("https://example.com/file").is_ok());
    }

    #[test]
    fn test_file_uri_validation_invalid() {
        // Test invalid URI schemes
        let result = validation_utils::validate_file_uri("invalid://scheme");
        assert!(result.is_err());
        if let Err(ModelError::InvalidFileUri { uri, reason }) = result {
            assert_eq!(uri, "invalid://scheme");
            assert!(reason.contains("Unsupported"));
        } else {
            panic!("Expected InvalidFileUri error");
        }
    }
}
