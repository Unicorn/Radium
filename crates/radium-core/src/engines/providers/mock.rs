//! Mock engine provider for testing.

use crate::engines::engine_trait::{
    Engine, EngineMetadata, ExecutionRequest, ExecutionResponse, TokenUsage,
};
use crate::engines::error::Result;
use async_trait::async_trait;
use std::time::Instant;

/// Mock engine implementation for testing.
pub struct MockEngine {
    /// Engine metadata.
    metadata: EngineMetadata,
}

impl MockEngine {
    /// Creates a new mock engine.
    pub fn new() -> Self {
        let metadata = EngineMetadata::new(
            "mock".to_string(),
            "Mock Engine".to_string(),
            "A mock AI engine for testing".to_string(),
        )
        .with_models(vec!["mock-model-1".to_string(), "mock-model-2".to_string()])
        .with_auth_required(false);

        Self { metadata }
    }
}

impl Default for MockEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Engine for MockEngine {
    fn metadata(&self) -> &EngineMetadata {
        &self.metadata
    }

    async fn is_available(&self) -> bool {
        // Mock is always available
        true
    }

    async fn is_authenticated(&self) -> Result<bool> {
        // Mock doesn't require auth
        Ok(true)
    }

    async fn execute(&self, request: ExecutionRequest) -> Result<ExecutionResponse> {
        // Capture start time for duration tracking
        let start = Instant::now();

        // Mock response that echoes the prompt
        let content = format!("Mock response to: {}", request.prompt);

        let usage = TokenUsage {
            input_tokens: request.prompt.len() as u64,
            output_tokens: content.len() as u64,
            total_tokens: (request.prompt.len() + content.len()) as u64,
        };

        // Calculate execution duration
        let duration = start.elapsed();

        Ok(ExecutionResponse {
            content,
            usage: Some(usage),
            model: request.model,
            raw: Some(format!("{{\"prompt\": \"{}\"}}", request.prompt)),
            execution_duration: Some(duration),
        })
    }

    fn default_model(&self) -> String {
        "mock-model-1".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_engine_metadata() {
        let engine = MockEngine::new();
        let metadata = engine.metadata();

        assert_eq!(metadata.id, "mock");
        assert_eq!(metadata.name, "Mock Engine");
        assert_eq!(metadata.models.len(), 2);
        assert!(!metadata.requires_auth);
    }

    #[tokio::test]
    async fn test_mock_engine_is_available() {
        let engine = MockEngine::new();
        assert!(engine.is_available().await);
    }

    #[tokio::test]
    async fn test_mock_engine_is_authenticated() {
        let engine = MockEngine::new();
        assert!(engine.is_authenticated().await.unwrap());
    }

    #[tokio::test]
    async fn test_mock_engine_execute() {
        let engine = MockEngine::new();
        let request = ExecutionRequest::new("mock-model-1".to_string(), "Hello world".to_string());

        let response = engine.execute(request).await.unwrap();

        assert!(response.content.contains("Hello world"));
        assert_eq!(response.model, "mock-model-1");
        assert!(response.usage.is_some());
        assert!(response.execution_duration.is_some());
        // Duration should be very small for mock engine, but > 0
        assert!(response.execution_duration.unwrap().as_nanos() > 0);
    }

    #[tokio::test]
    async fn test_mock_engine_duration_accuracy() {
        let engine = MockEngine::new();
        let request = ExecutionRequest::new("mock-model-1".to_string(), "Test".to_string());

        // Add a small delay to test duration tracking
        let start = Instant::now();
        let response = engine.execute(request).await.unwrap();
        let elapsed = start.elapsed();

        let reported_duration = response.execution_duration.unwrap();
        
        // Reported duration should be close to elapsed time (within 10ms tolerance)
        let diff = if elapsed > reported_duration {
            elapsed - reported_duration
        } else {
            reported_duration - elapsed
        };
        assert!(diff.as_millis() < 10, "Duration should be accurate within 10ms");
    }

    #[tokio::test]
    async fn test_mock_engine_default_model() {
        let engine = MockEngine::new();
        assert_eq!(engine.default_model(), "mock-model-1");
    }

    #[tokio::test]
    async fn test_mock_engine_available_models() {
        let engine = MockEngine::new();
        let models = engine.available_models();

        assert_eq!(models.len(), 2);
        assert!(models.contains(&"mock-model-1".to_string()));
        assert!(models.contains(&"mock-model-2".to_string()));
    }
}
