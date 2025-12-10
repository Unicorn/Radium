//! Model implementations for Radium.
//!
//! This crate provides concrete implementations of the `Model` trait.
//!
//! # Supported Providers
//!
//! - **Mock**: Testing and development
//! - **Claude**: Anthropic's Claude models (API key required)
//! - **Gemini**: Google's Gemini models (API key required)
//! - **OpenAI**: OpenAI's GPT models (API key required)
//! - **Ollama**: Local models via Ollama (no API key, local execution)

pub mod cache;
pub mod claude;
pub mod factory;
pub mod gemini;
pub mod ollama;
pub mod openai;
pub mod universal;

use async_trait::async_trait;
use radium_abstraction::{
    ChatMessage, Model, ModelError, ModelParameters, ModelResponse, ModelUsage,
};
use tracing::debug;

pub use cache::{CacheConfig, CacheKey, CacheStats, CachedModel, ModelCache};
pub use claude::ClaudeModel;
pub use factory::{ModelConfig, ModelFactory, ModelType};
pub use gemini::GeminiModel;
pub use ollama::OllamaModel;
pub use openai::OpenAIModel;
pub use universal::UniversalModel;

/// A mock implementation of the `Model` trait for testing and demonstration.
#[derive(Debug, Default)]
pub struct MockModel {
    id: String,
}

impl MockModel {
    /// Creates a new `MockModel` with the given ID.
    #[must_use]
    pub const fn new(id: String) -> Self {
        Self { id }
    }
}

#[async_trait]
impl Model for MockModel {
    async fn generate_text(
        &self,
        prompt: &str,
        parameters: Option<ModelParameters>,
    ) -> Result<ModelResponse, ModelError> {
        debug!(
            model_id = %self.id,
            prompt = %prompt,
            parameters = ?parameters,
            "MockModel generating text"
        );

        let response_content = format!(
            "Mock response for: {prompt}\nModel ID: {}\nParameters: {parameters:?}",
            self.id
        );

        let prompt_tokens = count_tokens(prompt);
        let completion_tokens = count_tokens(&response_content);
        let total_tokens = prompt_tokens + completion_tokens;

        Ok(ModelResponse {
            content: response_content,
            model_id: Some(self.id.clone()),
            usage: Some(ModelUsage { prompt_tokens, completion_tokens, total_tokens }),
            metadata: None,
        })
    }

    async fn generate_chat_completion(
        &self,
        messages: &[ChatMessage],
        parameters: Option<ModelParameters>,
    ) -> Result<ModelResponse, ModelError> {
        use std::fmt::Write;

        debug!(
            model_id = %self.id,
            message_count = messages.len(),
            parameters = ?parameters,
            "MockModel generating chat completion"
        );

        let mut conversation_summary = String::from("Conversation Summary:\n");
        for message in messages {
            let _ = writeln!(conversation_summary, "  {}: {}", message.role, message.content);
        }

        let response_content = format!(
            "Mock chat response from {}\n{conversation_summary}\nParameters: {parameters:?}",
            self.id
        );

        let prompt_tokens = messages.iter().map(|m| count_tokens(&m.content)).sum::<u32>();
        let completion_tokens = count_tokens(&response_content);
        let total_tokens = prompt_tokens + completion_tokens;

        Ok(ModelResponse {
            content: response_content,
            model_id: Some(self.id.clone()),
            usage: Some(ModelUsage { prompt_tokens, completion_tokens, total_tokens }),
            metadata: None,
        })
    }

    fn model_id(&self) -> &str {
        &self.id
    }
}

/// Count tokens in a string (simplified: word count).
///
/// For a real implementation, this would use a proper tokenizer.
#[allow(clippy::cast_possible_truncation)]
fn count_tokens(text: &str) -> u32 {
    text.split_whitespace().count() as u32
}
