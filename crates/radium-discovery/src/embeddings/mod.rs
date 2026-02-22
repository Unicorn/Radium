//! Embedding provider abstraction layer
//!
//! Auto-discovers the embedding provider from environment variables:
//! - If `ANTHROPIC_API_KEY` is set, uses Anthropic (Voyage AI)
//! - If `OPENAI_API_KEY` is set, uses OpenAI
//! - If neither is set, returns error

mod anthropic;
mod openai;
pub mod provider;

pub use provider::{EmbeddingError, EmbeddingProvider};

/// Auto-discover and create an embedding provider from environment variables.
/// Prefers Anthropic if both keys are set.
#[allow(clippy::disallowed_methods)] // Provider auto-discovery requires checking env vars
pub fn create_provider() -> Result<Box<dyn EmbeddingProvider>, EmbeddingError> {
    if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
        tracing::info!("Found ANTHROPIC_API_KEY, using Anthropic (Voyage AI) embeddings");
        return Ok(Box::new(anthropic::AnthropicEmbedding::new(key)));
    }

    if let Ok(key) = std::env::var("OPENAI_API_KEY") {
        tracing::info!("Found OPENAI_API_KEY, using OpenAI embeddings");
        return Ok(Box::new(openai::OpenAIEmbedding::new(key)));
    }

    Err(EmbeddingError::NotConfigured)
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)] // Tests need to manipulate env vars directly
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_create_provider_no_keys() {
        std::env::remove_var("ANTHROPIC_API_KEY");
        std::env::remove_var("OPENAI_API_KEY");
        let result = create_provider();
        assert!(result.is_err());
    }

    #[test]
    #[serial]
    fn test_create_provider_openai() {
        std::env::remove_var("ANTHROPIC_API_KEY");
        std::env::set_var("OPENAI_API_KEY", "test-key");
        let result = create_provider();
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().provider_name(),
            "openai/text-embedding-3-small"
        );
        std::env::remove_var("OPENAI_API_KEY");
    }

    #[test]
    #[serial]
    fn test_create_provider_anthropic() {
        std::env::set_var("ANTHROPIC_API_KEY", "test-key");
        std::env::remove_var("OPENAI_API_KEY");
        let result = create_provider();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().provider_name(), "anthropic/voyage-3-lite");
        std::env::remove_var("ANTHROPIC_API_KEY");
    }

    #[test]
    #[serial]
    fn test_create_provider_prefers_anthropic() {
        std::env::set_var("ANTHROPIC_API_KEY", "test-key");
        std::env::set_var("OPENAI_API_KEY", "test-key");
        let result = create_provider();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().provider_name(), "anthropic/voyage-3-lite");
        std::env::remove_var("ANTHROPIC_API_KEY");
        std::env::remove_var("OPENAI_API_KEY");
    }
}
