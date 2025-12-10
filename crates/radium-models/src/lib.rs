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
use futures::stream::{self, Stream};
use radium_abstraction::{
    ChatMessage, Model, ModelError, ModelParameters, ModelResponse, ModelUsage, StreamingModel,
};
use std::pin::Pin;
use std::time::Duration;
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

#[async_trait]
impl StreamingModel for MockModel {
    async fn generate_stream(
        &self,
        prompt: &str,
        parameters: Option<ModelParameters>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, ModelError>> + Send>>, ModelError> {
        debug!(
            model_id = %self.id,
            prompt = %prompt,
            parameters = ?parameters,
            "MockModel generating streaming text"
        );

        // Generate mock response (reuse existing logic)
        let response_content = format!(
            "Mock response for: {prompt}\nModel ID: {}\nParameters: {parameters:?}",
            self.id
        );

        // Split response into words for realistic streaming
        let words: Vec<String> = response_content
            .split_whitespace()
            .map(|w| w.to_string())
            .collect();

        // Create stream that yields accumulated content with delays
        let stream = stream::unfold((0, String::new()), move |(mut index, mut accumulated)| {
            let words = words.clone();
            async move {
                if index >= words.len() {
                    return None;
                }

                // Add 50ms delay between tokens
                tokio::time::sleep(Duration::from_millis(50)).await;

                let word = words[index].clone();
                index += 1;

                // Add space after word (except for last word)
                let token = if index < words.len() {
                    format!("{} ", word)
                } else {
                    word
                };

                accumulated.push_str(&token);

                Some((Ok(accumulated.clone()), (index, accumulated)))
            }
        });

        Ok(Box::pin(stream))
    }
}

/// Count tokens in a string (simplified: word count).
///
/// For a real implementation, this would use a proper tokenizer.
#[allow(clippy::cast_possible_truncation)]
fn count_tokens(text: &str) -> u32 {
    text.split_whitespace().count() as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;

    #[tokio::test]
    async fn test_mock_model_streaming() {
        let model = MockModel::new("test-model".to_string());
        let mut stream = model
            .generate_stream("test prompt", None)
            .await
            .expect("Should create stream");

        let mut all_content = Vec::new();
        while let Some(result) = stream.next().await {
            let content = result.expect("Stream should not error");
            all_content.push(content);
        }

        // Should have received at least one chunk
        assert!(!all_content.is_empty());
        
        // Last content should be the complete accumulated response
        let final_content = all_content.last().unwrap();
        assert!(final_content.contains("test prompt"));
        assert!(final_content.contains("test-model"));
    }

    #[tokio::test]
    async fn test_mock_model_streaming_accumulation() {
        let model = MockModel::new("test-model".to_string());
        let mut stream = model
            .generate_stream("hello", None)
            .await
            .expect("Should create stream");

        let mut last_len = 0;
        while let Some(result) = stream.next().await {
            let content = result.expect("Stream should not error");
            // Content should accumulate (each chunk should be longer or equal)
            assert!(content.len() >= last_len, "Content should accumulate");
            last_len = content.len();
        }
        
        // Final content should not be empty
        assert!(last_len > 0);
    }
}
