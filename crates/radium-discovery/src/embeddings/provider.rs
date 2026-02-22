//! Embedding provider trait definition

use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmbeddingError {
    #[error("No embedding provider configured. Set ANTHROPIC_API_KEY or OPENAI_API_KEY.")]
    NotConfigured,

    #[error("Embedding API request failed: {0}")]
    RequestFailed(String),

    #[error("Embedding API returned unexpected response: {0}")]
    BadResponse(String),
}

/// Trait for generating text embeddings.
/// Implementations auto-discovered from environment variables at startup.
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Generate an embedding vector for the given text.
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>;

    /// The dimensionality of vectors produced by this provider.
    fn dimension(&self) -> usize;

    /// Human-readable provider name for logging.
    fn provider_name(&self) -> &str;
}
