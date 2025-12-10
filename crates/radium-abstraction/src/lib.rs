//! Model abstraction layer for Radium.
//!
//! This module defines the core traits and types for interacting with AI models.

use async_trait::async_trait;
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::pin::Pin;
use thiserror::Error;

/// Behavior for handling safety-filtered content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SafetyBlockBehavior {
    /// Return available content with metadata (default, backward compatible).
    ReturnPartial,
    /// Throw ContentFiltered error when content is blocked.
    ThrowError,
    /// Log warning and continue with available content.
    LogWarning,
}

impl Default for SafetyBlockBehavior {
    fn default() -> Self {
        Self::ReturnPartial
    }
}

/// Represents an error that can occur when interacting with an AI model.
#[derive(Error, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelError {
    /// An error occurred during the API request (e.g., network issues, invalid request).
    #[error("Request Error: {0}")]
    RequestError(String),

    /// The model returned an error (e.g., invalid input, rate limiting).
    #[error("Model Response Error: {0}")]
    ModelResponseError(String),

    /// An error occurred during serialization or deserialization.
    #[error("Serialization Error: {0}")]
    SerializationError(String),

    /// The model provider is not supported or configured.
    #[error("Unsupported Model Provider: {0}")]
    UnsupportedModelProvider(String),

    /// Provider quota exceeded or rate limit hit (hard stop error).
    #[error("Provider '{provider}' quota exceeded{}", message.as_ref().map(|m| format!(": {}", m)).unwrap_or_default())]
    QuotaExceeded {
        /// The provider name (e.g., "openai", "gemini").
        provider: String,
        /// Optional error message from the provider.
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },

    /// Content was filtered/blocked by the provider's safety system.
    #[error("Content filtered by {provider}: {reason}")]
    ContentFiltered {
        /// The provider name (e.g., "openai", "gemini").
        provider: String,
        /// Reason for filtering.
        reason: String,
        /// Safety ratings that caused the filtering (optional).
        #[serde(skip_serializing_if = "Option::is_none")]
        safety_ratings: Option<Vec<SafetyRating>>,
    },

    /// The model does not support the requested content block type.
    #[error("Unsupported content type '{content_type}' for model '{model}'")]
    UnsupportedContentType {
        /// The content block type that is not supported (e.g., "audio", "video").
        content_type: String,
        /// The model ID that does not support this content type.
        model: String,
    },

    /// Invalid media source (file path doesn't exist, URL is malformed, etc.).
    #[error("Invalid media source '{media_source}': {reason}")]
    InvalidMediaSource {
        /// The media source that is invalid (file path or URL).
        media_source: String,
        /// Reason why the source is invalid.
        reason: String,
    },

    /// Media content exceeds the model's size limit.
    #[error("Media size {size} bytes exceeds limit of {limit} bytes for {media_type}")]
    MediaSizeLimitExceeded {
        /// The actual size in bytes.
        size: usize,
        /// The maximum allowed size in bytes.
        limit: usize,
        /// The media type (e.g., "image", "audio").
        media_type: String,
    },

    /// Invalid media format (MIME type not supported).
    #[error("Invalid media format '{format}'. Expected one of: {expected}")]
    InvalidMediaFormat {
        /// The MIME type that was provided.
        format: String,
        /// List of expected MIME types (formatted as comma-separated string).
        expected: String,
    },

    /// Other unexpected errors.
    #[error("Other Model Error: {0}")]
    Other(String),
}

impl ModelError {
    /// Formats the message part for QuotaExceeded error display.
    /// This method is used by the thiserror Display implementation.
    #[allow(dead_code)] // Used by thiserror macro via format string
    fn message_formatted(&self) -> String {
        match self {
            Self::QuotaExceeded { message, .. } => {
                message.as_ref().map(|m| format!(": {}", m)).unwrap_or_default()
            }
            _ => String::new(),
        }
    }
}

/// Represents the content of a chat message, supporting both simple text and multimodal content blocks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    /// Simple text content (backward compatible).
    Text(String),
    /// Multimodal content blocks (text, images, audio, video, documents).
    Blocks(Vec<ContentBlock>),
}

impl MessageContent {
    /// Creates a new `MessageContent::Text` from a string or string-like type.
    ///
    /// # Examples
    /// ```
    /// use radium_abstraction::MessageContent;
    ///
    /// let content = MessageContent::text("Hello, world!");
    /// ```
    pub fn text<S: Into<String>>(s: S) -> Self {
        MessageContent::Text(s.into())
    }

    /// Returns `true` if this content is simple text (not multimodal blocks).
    ///
    /// # Examples
    /// ```
    /// use radium_abstraction::MessageContent;
    ///
    /// let text_content = MessageContent::text("Hello");
    /// assert!(text_content.is_text_only());
    /// ```
    pub fn is_text_only(&self) -> bool {
        matches!(self, MessageContent::Text(_))
    }

    /// Returns a reference to the text content if this is a `Text` variant, `None` otherwise.
    ///
    /// # Examples
    /// ```
    /// use radium_abstraction::MessageContent;
    ///
    /// let text_content = MessageContent::text("Hello");
    /// assert_eq!(text_content.as_text(), Some("Hello"));
    /// ```
    pub fn as_text(&self) -> Option<&str> {
        match self {
            MessageContent::Text(text) => Some(text),
            MessageContent::Blocks(_) => None,
        }
    }
}

/// Enables automatic conversion from `String` to `MessageContent` for backward compatibility.
///
/// This trait implementation allows existing code that uses `String` for message content
/// to continue working without modification. The string is automatically converted to
/// `MessageContent::Text`.
///
/// # Examples
/// ```
/// use radium_abstraction::{ChatMessage, MessageContent};
///
/// // Old code still works:
/// let msg = ChatMessage {
///     role: "user".to_string(),
///     content: "Hello".to_string().into(), // Automatic conversion
/// };
///
/// // Or with explicit conversion:
/// let content: MessageContent = "Hello".to_string().into();
/// ```
impl From<String> for MessageContent {
    fn from(text: String) -> Self {
        MessageContent::Text(text)
    }
}

impl From<&str> for MessageContent {
    fn from(text: &str) -> Self {
        MessageContent::Text(text.to_string())
    }
}

/// A content block within a multimodal message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    /// Text content block.
    #[serde(rename = "text")]
    Text {
        /// The text content.
        text: String,
    },
    /// Image content block.
    #[serde(rename = "image")]
    Image {
        /// The image source.
        source: ImageSource,
        /// The MIME type of the image (e.g., "image/jpeg", "image/png").
        media_type: String,
    },
    /// Audio content block.
    #[serde(rename = "audio")]
    Audio {
        /// The audio source.
        source: MediaSource,
        /// The MIME type of the audio (e.g., "audio/mp3", "audio/wav").
        media_type: String,
    },
    /// Video content block.
    #[serde(rename = "video")]
    Video {
        /// The video source.
        source: MediaSource,
        /// The MIME type of the video (e.g., "video/mp4", "video/webm").
        media_type: String,
    },
    /// Document content block.
    #[serde(rename = "document")]
    Document {
        /// The document source.
        source: MediaSource,
        /// The MIME type of the document (e.g., "application/pdf", "text/plain").
        media_type: String,
        /// Optional filename for the document.
        #[serde(skip_serializing_if = "Option::is_none")]
        filename: Option<String>,
    },
}

/// Source for image content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ImageSource {
    /// Base64-encoded image data.
    #[serde(rename = "base64")]
    Base64 {
        /// The base64-encoded image data.
        data: String,
    },
    /// Image from a URL.
    #[serde(rename = "url")]
    Url {
        /// The URL of the image.
        url: String,
    },
    /// Image from a local file path.
    #[serde(rename = "file")]
    File {
        /// The path to the image file.
        path: PathBuf,
    },
}

/// Source for media content (audio, video, documents).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MediaSource {
    /// Base64-encoded media data.
    #[serde(rename = "base64")]
    Base64 {
        /// The base64-encoded media data.
        data: String,
    },
    /// Media from a URL.
    #[serde(rename = "url")]
    Url {
        /// The URL of the media.
        url: String,
    },
    /// Media from a local file path.
    #[serde(rename = "file")]
    File {
        /// The path to the media file.
        path: PathBuf,
    },
    /// Media from a provider's file API (e.g., Gemini File API).
    #[serde(rename = "file_api")]
    FileApi {
        /// The file ID from the provider's file API.
        file_id: String,
    },
}

/// Constants for allowed MIME types per content block type.
pub mod mime_types {
    /// Allowed MIME types for image content blocks.
    pub const IMAGE_FORMATS: &[&str] = &["image/jpeg", "image/png", "image/gif", "image/webp"];

    /// Allowed MIME types for audio content blocks.
    pub const AUDIO_FORMATS: &[&str] = &["audio/mp3", "audio/wav", "audio/ogg"];

    /// Allowed MIME types for video content blocks.
    pub const VIDEO_FORMATS: &[&str] = &["video/mp4", "video/webm"];

    /// Allowed MIME types for document content blocks.
    pub const DOCUMENT_FORMATS: &[&str] = &["application/pdf", "text/plain", "text/markdown"];
}

/// Default size limit for Base64-encoded media content (20MB).
pub const DEFAULT_SIZE_LIMIT: usize = 20 * 1024 * 1024;

/// Validates a MIME type against allowed formats for a content block type.
///
/// # Arguments
/// * `media_type` - The MIME type to validate
/// * `content_block` - The content block type to validate against
///
/// # Returns
/// `Ok(())` if the MIME type is valid, `Err(ModelError::InvalidMediaFormat)` otherwise.
pub fn validate_mime_type(media_type: &str, content_block: &ContentBlock) -> Result<(), ModelError> {
    let allowed_formats = match content_block {
        ContentBlock::Text { .. } => return Ok(()), // Text blocks don't need MIME type validation
        ContentBlock::Image { .. } => mime_types::IMAGE_FORMATS,
        ContentBlock::Audio { .. } => mime_types::AUDIO_FORMATS,
        ContentBlock::Video { .. } => mime_types::VIDEO_FORMATS,
        ContentBlock::Document { .. } => mime_types::DOCUMENT_FORMATS,
    };

    if allowed_formats.contains(&media_type) {
        Ok(())
    } else {
        Err(ModelError::InvalidMediaFormat {
            format: media_type.to_string(),
            expected: allowed_formats.iter().map(|s| (*s).to_string()).collect::<Vec<_>>().join(", "),
        })
    }
}

/// Validates that a file path exists and is readable.
///
/// # Arguments
/// * `path` - The file path to validate
///
/// # Returns
/// `Ok(())` if the file exists and is readable, `Err(ModelError::InvalidMediaSource)` otherwise.
pub fn validate_file_path(path: &PathBuf) -> Result<(), ModelError> {
    match std::fs::metadata(path) {
        Ok(metadata) => {
            if metadata.is_file() {
                Ok(())
            } else {
                Err(ModelError::InvalidMediaSource {
                    media_source: path.display().to_string(),
                    reason: "Path is not a file".to_string(),
                })
            }
        }
        Err(e) => Err(ModelError::InvalidMediaSource {
            media_source: path.display().to_string(),
            reason: format!("File does not exist or is not readable: {}", e),
        }),
    }
}

/// Validates that a URL has a valid format (http:// or https://).
///
/// # Arguments
/// * `url` - The URL to validate
///
/// # Returns
/// `Ok(())` if the URL format is valid, `Err(ModelError::InvalidMediaSource)` otherwise.
pub fn validate_url(url: &str) -> Result<(), ModelError> {
    if url.starts_with("http://") || url.starts_with("https://") {
        Ok(())
    } else {
        Err(ModelError::InvalidMediaSource {
            media_source: url.to_string(),
            reason: "URL must start with http:// or https://".to_string(),
        })
    }
}

/// Validates that Base64-encoded data does not exceed the size limit.
///
/// # Arguments
/// * `data` - The Base64-encoded data string
/// * `limit` - The maximum allowed size in bytes (after decoding)
///
/// # Returns
/// `Ok(())` if the decoded data size is within the limit, `Err(ModelError::MediaSizeLimitExceeded)` otherwise.
pub fn validate_base64_size(data: &str, limit: usize) -> Result<(), ModelError> {
    use base64::Engine;
    let engine = base64::engine::general_purpose::STANDARD;
    
    match engine.decode(data) {
        Ok(decoded) => {
            let size = decoded.len();
            if size <= limit {
                Ok(())
            } else {
                Err(ModelError::MediaSizeLimitExceeded {
                    size,
                    limit,
                    media_type: "base64".to_string(),
                })
            }
        }
        Err(e) => Err(ModelError::InvalidMediaSource {
            media_source: "base64 data".to_string(),
            reason: format!("Invalid Base64 encoding: {}", e),
        }),
    }
}

/// Represents a message in a conversation with a chat model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    /// The role of the message sender (e.g., "user", "assistant", "system").
    pub role: String,
    /// The content of the message (supports both text and multimodal content).
    pub content: MessageContent,
}

/// Parameters for controlling the model's generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelParameters {
    /// What sampling temperature to use, between 0 and 2.
    /// Higher values mean the model will take more risks.
    pub temperature: Option<f32>,

    /// An alternative to sampling with temperature, called nucleus sampling,
    /// where the model considers the results of the tokens with `top_p` probability mass.
    pub top_p: Option<f32>,

    /// The maximum number of tokens to generate in the chat completion.
    pub max_tokens: Option<u32>,

    /// The number of highest probability tokens to consider for sampling.
    /// Valid range: 1-100 (provider-specific limits apply).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,

    /// Reduces likelihood of repeating tokens based on frequency.
    /// Valid range: -2.0 to 2.0 (provider-specific).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,

    /// Reduces likelihood of repeating tokens regardless of frequency.
    /// Valid range: -2.0 to 2.0 (provider-specific).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,

    /// Format for model response output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,

    /// Up to 4 sequences where the API will stop generating further tokens.
    pub stop_sequences: Option<Vec<String>>,
}

impl Default for ModelParameters {
    fn default() -> Self {
        Self {
            temperature: Some(0.7),
            top_p: Some(1.0),
            max_tokens: Some(512),
            top_k: None,
            frequency_penalty: None,
            presence_penalty: None,
            response_format: None,
            stop_sequences: None,
        }
    }
}

/// Format for model response output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseFormat {
    /// Plain text output (default).
    Text,
    /// JSON-formatted output without schema validation.
    Json,
    /// JSON output conforming to the provided schema.
    JsonSchema(String),
}

/// The response from a text generation or chat completion model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResponse {
    /// The generated content.
    pub content: String,

    /// Optional: The ID of the model used to generate the response.
    pub model_id: Option<String>,

    /// Optional: Usage statistics for the request.
    pub usage: Option<ModelUsage>,

    /// Optional: Provider-specific metadata (e.g., finish_reason, safety_ratings, citations, logprobs).
    /// This field enables debugging, cost tracking, safety monitoring, and citation tracking.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl ModelResponse {
    /// Gets the finish reason from metadata, if available.
    ///
    /// Common values: "stop", "length", "safety", "tool_use", "max_tokens"
    pub fn get_finish_reason(&self) -> Option<String> {
        self.metadata
            .as_ref()?
            .get("finish_reason")?
            .as_str()
            .map(String::from)
    }

    /// Gets the safety ratings from metadata, if available.
    pub fn get_safety_ratings(&self) -> Option<Vec<SafetyRating>> {
        let ratings = self.metadata.as_ref()?.get("safety_ratings")?;
        serde_json::from_value(ratings.clone()).ok()
    }

    /// Gets the citations from metadata, if available.
    pub fn get_citations(&self) -> Option<Vec<Citation>> {
        let citations = self.metadata.as_ref()?.get("citations")?;
        serde_json::from_value(citations.clone()).ok()
    }

    /// Gets the log probabilities from metadata, if available.
    pub fn get_logprobs(&self) -> Option<Vec<LogProb>> {
        let logprobs = self.metadata.as_ref()?.get("logprobs")?;
        serde_json::from_value(logprobs.clone()).ok()
    }

    /// Gets the model version from metadata, if available.
    pub fn get_model_version(&self) -> Option<String> {
        self.metadata
            .as_ref()?
            .get("model_version")?
            .as_str()
            .map(String::from)
    }

    /// Checks if content was filtered/blocked based on safety ratings.
    ///
    /// Returns `true` if any safety rating indicates blocked content.
    pub fn was_content_filtered(&self) -> bool {
        self.get_safety_ratings()
            .map(|ratings| ratings.iter().any(|r| r.blocked))
            .unwrap_or(false)
    }

    /// Gets provider-specific metadata as a typed struct.
    ///
    /// # Type Parameters
    /// * `T` - The type to deserialize into (must implement `DeserializeOwned`)
    ///
    /// # Returns
    /// `Some(T)` if metadata exists and can be deserialized, `None` otherwise.
    pub fn get_provider_metadata<T: for<'de> Deserialize<'de>>(&self) -> Option<T> {
        serde_json::from_value(serde_json::Value::Object(
            self.metadata.as_ref()?.clone().into_iter().collect(),
        ))
        .ok()
    }
}

/// Usage statistics for a model request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsage {
    /// Number of tokens in the prompt.
    pub prompt_tokens: u32,

    /// Number of tokens in the completion.
    pub completion_tokens: u32,

    /// Total number of tokens used.
    pub total_tokens: u32,
}

/// Safety rating for content filtering.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SafetyRating {
    /// The category of safety concern (e.g., "HARM_CATEGORY_HATE_SPEECH").
    pub category: String,
    /// The probability level (e.g., "NEGLIGIBLE", "LOW", "MEDIUM", "HIGH").
    pub probability: String,
    /// Whether the content was blocked due to this rating.
    pub blocked: bool,
}

/// Citation information for grounded responses.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Citation {
    /// Start index of the cited text in the response.
    pub start_index: Option<u32>,
    /// End index of the cited text in the response.
    pub end_index: Option<u32>,
    /// URI of the source document.
    pub uri: Option<String>,
    /// Title of the source document.
    pub title: Option<String>,
}

/// Log probability information for a token.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogProb {
    /// The token text.
    pub token: String,
    /// The log probability of the token.
    pub logprob: f64,
    /// The bytes representation of the token (optional).
    pub bytes: Option<Vec<u8>>,
}

/// A trait for interacting with different AI models.
///
/// All models must be `Send + Sync` to allow concurrent use across threads.
#[async_trait]
pub trait Model: Send + Sync {
    /// Generates a text completion based on the given prompt.
    ///
    /// # Arguments
    /// * `prompt` - The input prompt for text generation
    /// * `parameters` - Optional parameters to control generation
    ///
    /// # Errors
    /// Returns a `ModelError` if generation fails.
    async fn generate_text(
        &self,
        prompt: &str,
        parameters: Option<ModelParameters>,
    ) -> Result<ModelResponse, ModelError>;

    /// Generates a chat completion based on the given conversation history.
    ///
    /// # Arguments
    /// * `messages` - The conversation history as a slice of chat messages
    /// * `parameters` - Optional parameters to control generation
    ///
    /// # Errors
    /// Returns a `ModelError` if generation fails.
    async fn generate_chat_completion(
        &self,
        messages: &[ChatMessage],
        parameters: Option<ModelParameters>,
    ) -> Result<ModelResponse, ModelError>;

    /// Returns the ID of the model.
    fn model_id(&self) -> &str;
}

/// A trait for models that support streaming text generation.
///
/// This trait enables real-time token-by-token streaming of model responses,
/// allowing consumers to display output as it's generated rather than waiting
/// for the complete response.
///
/// # When to Use Streaming vs Non-Streaming
///
/// - **Use `StreamingModel::generate_stream()`** when you need real-time output
///   display (e.g., in a TUI or CLI where users should see tokens as they arrive)
/// - **Use `Model::generate_text()`** when you need the complete response before
///   processing (e.g., for batch processing or when the full text is required)
///
/// # Consuming the Stream
///
/// The stream yields `Result<String, ModelError>` items where each `String` is
/// a token or chunk of the response. Errors within the stream indicate issues
/// during generation (e.g., network interruptions, API errors).
///
/// ```rust,no_run
/// use radium_abstraction::StreamingModel;
/// use futures::StreamExt;
///
/// # async fn example(model: impl StreamingModel) -> Result<(), Box<dyn std::error::Error>> {
/// let mut stream = model.generate_stream("Hello", None).await?;
/// while let Some(token_result) = stream.next().await {
///     match token_result {
///         Ok(token) => print!("{}", token),
///         Err(e) => eprintln!("Error: {}", e),
///     }
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Error Handling
///
/// Errors can occur at two points:
/// 1. **Stream creation**: Returns `ModelError` if the request cannot be initiated
/// 2. **During streaming**: Individual `Result` items in the stream may contain errors
///
/// Consumers should handle both cases appropriately.
#[async_trait]
pub trait StreamingModel: Send + Sync {
    /// Generates a streaming text completion based on the given prompt.
    ///
    /// Returns an async stream that yields tokens as they're generated by the model.
    /// Each item in the stream is a `Result<String, ModelError>` where:
    /// - `Ok(token)` contains a token or chunk of the response
    /// - `Err(error)` indicates an error during generation
    ///
    /// # Arguments
    /// * `prompt` - The input prompt for text generation
    /// * `parameters` - Optional parameters to control generation
    ///
    /// # Returns
    /// A pinned, boxed stream of token results. The stream must be `Send` to allow
    /// use across async boundaries.
    ///
    /// # Errors
    /// Returns a `ModelError` if the stream cannot be created (e.g., connection
    /// failure, invalid request). Errors during streaming are yielded as items
    /// in the stream itself.
    async fn generate_stream(
        &self,
        prompt: &str,
        parameters: Option<ModelParameters>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, ModelError>> + Send>>, ModelError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_parameters_with_new_fields() {
        let params = ModelParameters {
            temperature: Some(0.7),
            top_p: Some(0.9),
            max_tokens: Some(100),
            top_k: Some(40),
            frequency_penalty: Some(0.5),
            presence_penalty: Some(0.3),
            response_format: Some(ResponseFormat::Json),
            stop_sequences: None,
        };

        let json = serde_json::to_string(&params).unwrap();
        let deserialized: ModelParameters = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.top_k, Some(40));
        assert_eq!(deserialized.frequency_penalty, Some(0.5));
        assert_eq!(deserialized.presence_penalty, Some(0.3));
        assert!(matches!(deserialized.response_format, Some(ResponseFormat::Json)));
    }

    #[test]
    fn test_response_format_variants() {
        let text = ResponseFormat::Text;
        let json = ResponseFormat::Json;
        let schema = ResponseFormat::JsonSchema("{\"type\":\"object\"}".to_string());

        assert!(matches!(text, ResponseFormat::Text));
        assert!(matches!(json, ResponseFormat::Json));
        assert!(matches!(schema, ResponseFormat::JsonSchema(_)));

        // Test serialization/deserialization
        let json_str = serde_json::to_string(&json).unwrap();
        let deserialized_json: ResponseFormat = serde_json::from_str(&json_str).unwrap();
        assert_eq!(deserialized_json, ResponseFormat::Json);

        let schema_str = serde_json::to_string(&schema).unwrap();
        let deserialized_schema: ResponseFormat = serde_json::from_str(&schema_str).unwrap();
        assert_eq!(deserialized_schema, schema);
    }

    #[test]
    fn test_model_parameters_default() {
        let params = ModelParameters::default();
        assert_eq!(params.top_k, None);
        assert_eq!(params.frequency_penalty, None);
        assert_eq!(params.presence_penalty, None);
        assert_eq!(params.response_format, None);
    }

    #[test]
    fn test_get_finish_reason() {
        let mut metadata = HashMap::new();
        metadata.insert("finish_reason".to_string(), serde_json::Value::String("stop".to_string()));
        
        let response = ModelResponse {
            content: "Test".to_string(),
            model_id: None,
            usage: None,
            metadata: Some(metadata),
        };

        assert_eq!(response.get_finish_reason(), Some("stop".to_string()));
    }

    #[test]
    fn test_get_finish_reason_missing() {
        let response = ModelResponse {
            content: "Test".to_string(),
            model_id: None,
            usage: None,
            metadata: None,
        };

        assert_eq!(response.get_finish_reason(), None);
    }

    #[test]
    fn test_was_content_filtered() {
        let mut metadata = HashMap::new();
        let safety_ratings = vec![
            SafetyRating {
                category: "HARM_CATEGORY_HATE_SPEECH".to_string(),
                probability: "NEGLIGIBLE".to_string(),
                blocked: false,
            },
            SafetyRating {
                category: "HARM_CATEGORY_DANGEROUS_CONTENT".to_string(),
                probability: "HIGH".to_string(),
                blocked: true,
            },
        ];
        metadata.insert("safety_ratings".to_string(), serde_json::to_value(&safety_ratings).unwrap());
        
        let response = ModelResponse {
            content: "Test".to_string(),
            model_id: None,
            usage: None,
            metadata: Some(metadata),
        };

        assert!(response.was_content_filtered());
    }

    #[test]
    fn test_was_content_filtered_not_blocked() {
        let mut metadata = HashMap::new();
        let safety_ratings = vec![
            SafetyRating {
                category: "HARM_CATEGORY_HATE_SPEECH".to_string(),
                probability: "NEGLIGIBLE".to_string(),
                blocked: false,
            },
        ];
        metadata.insert("safety_ratings".to_string(), serde_json::to_value(&safety_ratings).unwrap());
        
        let response = ModelResponse {
            content: "Test".to_string(),
            model_id: None,
            usage: None,
            metadata: Some(metadata),
        };

        assert!(!response.was_content_filtered());
    }

    #[test]
    fn test_get_safety_ratings() {
        let mut metadata = HashMap::new();
        let safety_ratings = vec![
            SafetyRating {
                category: "HARM_CATEGORY_HATE_SPEECH".to_string(),
                probability: "NEGLIGIBLE".to_string(),
                blocked: false,
            },
        ];
        metadata.insert("safety_ratings".to_string(), serde_json::to_value(&safety_ratings).unwrap());
        
        let response = ModelResponse {
            content: "Test".to_string(),
            model_id: None,
            usage: None,
            metadata: Some(metadata),
        };

        let result = response.get_safety_ratings().unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].category, "HARM_CATEGORY_HATE_SPEECH");
    }

    #[test]
    fn test_get_citations() {
        let mut metadata = HashMap::new();
        let citations = vec![
            Citation {
                start_index: Some(0),
                end_index: Some(10),
                uri: Some("https://example.com".to_string()),
                title: Some("Example".to_string()),
            },
        ];
        metadata.insert("citations".to_string(), serde_json::to_value(&citations).unwrap());
        
        let response = ModelResponse {
            content: "Test".to_string(),
            model_id: None,
            usage: None,
            metadata: Some(metadata),
        };

        let result = response.get_citations().unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].uri, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_get_model_version() {
        let mut metadata = HashMap::new();
        metadata.insert("model_version".to_string(), serde_json::Value::String("gemini-1.5-pro-001".to_string()));
        
        let response = ModelResponse {
            content: "Test".to_string(),
            model_id: None,
            usage: None,
            metadata: Some(metadata),
        };

        assert_eq!(response.get_model_version(), Some("gemini-1.5-pro-001".to_string()));
    }

    #[test]
    fn test_multimodal_error_types() {
        // Test UnsupportedContentType
        let err1 = ModelError::UnsupportedContentType {
            content_type: "audio".to_string(),
            model: "gpt-3.5-turbo".to_string(),
        };
        let err1_str = err1.to_string();
        assert!(err1_str.contains("audio"));
        assert!(err1_str.contains("gpt-3.5-turbo"));

        // Test InvalidMediaSource
        let err2 = ModelError::InvalidMediaSource {
            media_source: "/path/to/missing/file.jpg".to_string(),
            reason: "File does not exist".to_string(),
        };
        let err2_str = err2.to_string();
        assert!(err2_str.contains("/path/to/missing/file.jpg"));
        assert!(err2_str.contains("File does not exist"));

        // Test MediaSizeLimitExceeded
        let err3 = ModelError::MediaSizeLimitExceeded {
            size: 25_000_000,
            limit: 20_000_000,
            media_type: "image".to_string(),
        };
        let err3_str = err3.to_string();
        assert!(err3_str.contains("25000000"));
        assert!(err3_str.contains("20000000"));
        assert!(err3_str.contains("image"));

        // Test InvalidMediaFormat
        let err4 = ModelError::InvalidMediaFormat {
            format: "image/svg+xml".to_string(),
            expected: "image/jpeg, image/png, image/gif".to_string(),
        };
        let err4_str = err4.to_string();
        assert!(err4_str.contains("image/svg+xml"));
        assert!(err4_str.contains("image/jpeg"));
    }

    #[test]
    fn test_message_content_enum() {
        // Test Text variant
        let text_content = MessageContent::Text("Hello".to_string());
        assert!(matches!(text_content, MessageContent::Text(_)));

        // Test Blocks variant
        let blocks_content = MessageContent::Blocks(vec![
            ContentBlock::Text {
                text: "Analyze this".to_string(),
            },
            ContentBlock::Image {
                source: ImageSource::File {
                    path: PathBuf::from("test.jpg"),
                },
                media_type: "image/jpeg".to_string(),
            },
        ]);
        assert!(matches!(blocks_content, MessageContent::Blocks(_)));
    }

    #[test]
    fn test_content_block_enum() {
        // Test Text block
        let text_block = ContentBlock::Text {
            text: "Hello".to_string(),
        };
        assert!(matches!(text_block, ContentBlock::Text { .. }));

        // Test Image block with Base64
        let image_block = ContentBlock::Image {
            source: ImageSource::Base64 {
                data: "base64data".to_string(),
            },
            media_type: "image/png".to_string(),
        };
        assert!(matches!(image_block, ContentBlock::Image { .. }));

        // Test Audio block
        let audio_block = ContentBlock::Audio {
            source: MediaSource::Url {
                url: "https://example.com/audio.mp3".to_string(),
            },
            media_type: "audio/mp3".to_string(),
        };
        assert!(matches!(audio_block, ContentBlock::Audio { .. }));
    }

    #[test]
    fn test_image_source_enum() {
        let base64_source = ImageSource::Base64 {
            data: "data".to_string(),
        };
        assert!(matches!(base64_source, ImageSource::Base64 { .. }));

        let url_source = ImageSource::Url {
            url: "https://example.com/image.jpg".to_string(),
        };
        assert!(matches!(url_source, ImageSource::Url { .. }));

        let file_source = ImageSource::File {
            path: PathBuf::from("image.jpg"),
        };
        assert!(matches!(file_source, ImageSource::File { .. }));
    }

    #[test]
    fn test_media_source_enum() {
        let file_api_source = MediaSource::FileApi {
            file_id: "file123".to_string(),
        };
        assert!(matches!(file_api_source, MediaSource::FileApi { .. }));
    }

    #[test]
    fn test_chat_message_with_message_content() {
        let msg = ChatMessage {
            role: "user".to_string(),
            content: MessageContent::Text("Hello".to_string()),
        };
        assert_eq!(msg.role, "user");
        assert!(matches!(msg.content, MessageContent::Text(_)));
    }

    #[test]
    fn test_serialization_round_trip() {
        let content = MessageContent::Blocks(vec![
            ContentBlock::Text {
                text: "Test".to_string(),
            },
        ]);
        let json = serde_json::to_string(&content).unwrap();
        let deserialized: MessageContent = serde_json::from_str(&json).unwrap();
        assert_eq!(content, deserialized);
    }

    #[test]
    fn test_validate_mime_type_valid() {
        let image_block = ContentBlock::Image {
            source: ImageSource::Base64 {
                data: "data".to_string(),
            },
            media_type: "image/jpeg".to_string(),
        };
        assert!(validate_mime_type("image/jpeg", &image_block).is_ok());

        let audio_block = ContentBlock::Audio {
            source: MediaSource::Url {
                url: "https://example.com/audio.mp3".to_string(),
            },
            media_type: "audio/mp3".to_string(),
        };
        assert!(validate_mime_type("audio/mp3", &audio_block).is_ok());
    }

    #[test]
    fn test_validate_mime_type_invalid() {
        let image_block = ContentBlock::Image {
            source: ImageSource::Base64 {
                data: "data".to_string(),
            },
            media_type: "image/svg+xml".to_string(),
        };
        let result = validate_mime_type("image/svg+xml", &image_block);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ModelError::InvalidMediaFormat { .. }));
    }

    #[test]
    fn test_validate_file_path_exists() {
        // Create a temporary file for testing
        let temp_file = std::env::temp_dir().join("radium_test_file.txt");
        std::fs::write(&temp_file, "test content").unwrap();
        
        let result = validate_file_path(&temp_file);
        assert!(result.is_ok());

        // Cleanup
        std::fs::remove_file(&temp_file).ok();
    }

    #[test]
    fn test_validate_file_path_missing() {
        let missing_file = PathBuf::from("/nonexistent/path/file.jpg");
        let result = validate_file_path(&missing_file);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ModelError::InvalidMediaSource { .. }));
    }

    #[test]
    fn test_validate_url_valid() {
        assert!(validate_url("https://example.com/image.jpg").is_ok());
        assert!(validate_url("http://example.com/image.jpg").is_ok());
    }

    #[test]
    fn test_validate_url_invalid() {
        let result = validate_url("not-a-url");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ModelError::InvalidMediaSource { .. }));
    }

    #[test]
    fn test_validate_base64_size_under_limit() {
        use base64::Engine;
        // Create a small Base64 string (well under 20MB)
        let small_data = base64::engine::general_purpose::STANDARD.encode(b"small test data");
        assert!(validate_base64_size(&small_data, DEFAULT_SIZE_LIMIT).is_ok());
    }

    #[test]
    fn test_validate_base64_size_over_limit() {
        use base64::Engine;
        // Create a Base64 string that decodes to more than the limit
        // We'll use a large string that when decoded exceeds 20MB
        let large_data = base64::engine::general_purpose::STANDARD.encode(&vec![0u8; DEFAULT_SIZE_LIMIT + 1]);
        let result = validate_base64_size(&large_data, DEFAULT_SIZE_LIMIT);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ModelError::MediaSizeLimitExceeded { .. }));
    }

    #[test]
    fn test_validate_base64_invalid_encoding() {
        let invalid_base64 = "not-valid-base64!!!";
        let result = validate_base64_size(invalid_base64, DEFAULT_SIZE_LIMIT);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ModelError::InvalidMediaSource { .. }));
    }

    #[test]
    fn test_from_string_conversion() {
        let content: MessageContent = "Hello".to_string().into();
        assert!(matches!(content, MessageContent::Text(_)));
        if let MessageContent::Text(text) = content {
            assert_eq!(text, "Hello");
        }
    }

    #[test]
    fn test_from_str_conversion() {
        let content: MessageContent = "Hello".into();
        assert!(matches!(content, MessageContent::Text(_)));
    }

    #[test]
    fn test_message_content_text_constructor() {
        let content = MessageContent::text("Hello");
        assert!(matches!(content, MessageContent::Text(_)));
        if let MessageContent::Text(text) = content {
            assert_eq!(text, "Hello");
        }
    }

    #[test]
    fn test_is_text_only() {
        let text_content = MessageContent::Text("Hello".to_string());
        assert!(text_content.is_text_only());

        let blocks_content = MessageContent::Blocks(vec![
            ContentBlock::Text {
                text: "Hello".to_string(),
            },
        ]);
        assert!(!blocks_content.is_text_only());
    }

    #[test]
    fn test_as_text() {
        let text_content = MessageContent::Text("Hello".to_string());
        assert_eq!(text_content.as_text(), Some("Hello"));

        let blocks_content = MessageContent::Blocks(vec![
            ContentBlock::Text {
                text: "Hello".to_string(),
            },
        ]);
        assert_eq!(blocks_content.as_text(), None);
    }

    #[test]
    fn test_backward_compatibility_chat_message() {
        // This test verifies that existing code patterns still work
        let msg = ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string().into(), // Automatic conversion via From<String>
        };
        assert_eq!(msg.role, "user");
        assert!(matches!(msg.content, MessageContent::Text(_)));
        assert_eq!(msg.content.as_text(), Some("Hello"));
    }
}
