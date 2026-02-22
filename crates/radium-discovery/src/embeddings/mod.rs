//! Embedding provider abstraction layer (stub — implemented in Task 3)

use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmbeddingError {
    #[error("Embedding provider not configured")]
    #[allow(dead_code)] // Used when provider selection is implemented (Task 3)
    NotConfigured,
}

#[async_trait]
#[allow(dead_code)] // Trait methods used when search endpoints are implemented (Task 6)
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>;
    fn dimension(&self) -> usize;
    fn provider_name(&self) -> &str;
}

/// Stub provider for initial scaffold — replaced in Task 3
struct StubProvider;

#[async_trait]
impl EmbeddingProvider for StubProvider {
    async fn embed(&self, _text: &str) -> Result<Vec<f32>, EmbeddingError> {
        Ok(vec![0.0; 1536])
    }
    fn dimension(&self) -> usize { 1536 }
    #[allow(clippy::unnecessary_literal_bound)]
    fn provider_name(&self) -> &str { "stub" }
}

#[allow(clippy::unnecessary_wraps)] // Will return Err when no provider is configured (Task 3)
pub fn create_provider() -> Result<Box<dyn EmbeddingProvider>, EmbeddingError> {
    // Stub: always returns the stub provider
    // Task 3 replaces this with env-var auto-discovery
    Ok(Box::new(StubProvider))
}
