//! Anthropic embedding provider (via Voyage AI, Anthropic's recommended embedding partner)

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::provider::{EmbeddingError, EmbeddingProvider};

#[allow(dead_code)] // Used by embed() when search endpoints are implemented (Task 6)
const VOYAGE_EMBEDDING_URL: &str = "https://api.voyageai.com/v1/embeddings";
#[allow(dead_code)] // Used by embed() when search endpoints are implemented (Task 6)
const MODEL: &str = "voyage-3-lite";
const DIMENSION: usize = 512;

pub struct AnthropicEmbedding {
    #[allow(dead_code)] // Used by embed() when search endpoints are implemented (Task 6)
    client: Client,
    #[allow(dead_code)] // Used by embed() when search endpoints are implemented (Task 6)
    api_key: String,
}

impl AnthropicEmbedding {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }
}

#[allow(dead_code)] // Used by embed() when search endpoints are implemented (Task 6)
#[derive(Serialize)]
struct EmbeddingRequest {
    model: String,
    input: Vec<String>,
    input_type: String,
}

#[allow(dead_code)] // Used by embed() when search endpoints are implemented (Task 6)
#[derive(Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[allow(dead_code)] // Used by embed() when search endpoints are implemented (Task 6)
#[derive(Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

#[async_trait]
impl EmbeddingProvider for AnthropicEmbedding {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let request = EmbeddingRequest {
            model: MODEL.to_string(),
            input: vec![text.to_string()],
            input_type: "document".to_string(),
        };

        let response = self
            .client
            .post(VOYAGE_EMBEDDING_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| EmbeddingError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(EmbeddingError::RequestFailed(format!(
                "Voyage AI API returned error: {body}"
            )));
        }

        let result: EmbeddingResponse = response
            .json()
            .await
            .map_err(|e| EmbeddingError::BadResponse(e.to_string()))?;

        result
            .data
            .into_iter()
            .next()
            .map(|d| d.embedding)
            .ok_or_else(|| EmbeddingError::BadResponse("Empty embedding response".to_string()))
    }

    fn dimension(&self) -> usize {
        DIMENSION
    }

    #[allow(clippy::unnecessary_literal_bound)]
    fn provider_name(&self) -> &str {
        "anthropic/voyage-3-lite"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimension() {
        let provider = AnthropicEmbedding::new("fake-key".to_string());
        assert_eq!(provider.dimension(), 512);
    }

    #[test]
    fn test_provider_name() {
        let provider = AnthropicEmbedding::new("fake-key".to_string());
        assert_eq!(provider.provider_name(), "anthropic/voyage-3-lite");
    }

    #[test]
    fn test_embedding_request_serialization() {
        let req = EmbeddingRequest {
            model: MODEL.to_string(),
            input: vec!["test text".to_string()],
            input_type: "document".to_string(),
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["model"], "voyage-3-lite");
        assert_eq!(json["input"][0], "test text");
        assert_eq!(json["input_type"], "document");
    }
}
