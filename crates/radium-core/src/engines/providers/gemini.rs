//! Gemini engine provider wrapper implementation.

use crate::auth::{CredentialStore, ProviderType};
use crate::engines::engine_trait::{
    Engine, EngineMetadata, ExecutionRequest, ExecutionResponse, TokenUsage,
};
use crate::engines::error::{EngineError, Result};
use async_trait::async_trait;
use radium_abstraction::{ModelParameters, ModelUsage};
use radium_models::GeminiModel;
use std::sync::Arc;

/// Gemini engine wrapper around radium-models::GeminiModel.
pub struct GeminiEngine {
    /// Engine metadata.
    metadata: EngineMetadata,
    /// Credential store for API key retrieval.
    credential_store: Arc<CredentialStore>,
}

impl GeminiEngine {
    /// Creates a new Gemini engine.
    pub fn new() -> Self {
        let metadata = EngineMetadata::new(
            "gemini".to_string(),
            "Gemini".to_string(),
            "Google Gemini AI engine".to_string(),
        )
        .with_models(vec![
            "gemini-pro".to_string(),
            "gemini-pro-vision".to_string(),
            "gemini-2.0-flash-exp".to_string(),
            "gemini-1.5-pro".to_string(),
            "gemini-1.5-flash".to_string(),
        ])
        .with_auth_required(true);

        // CredentialStore::new() can fail if HOME is not set, fallback to temp path
        let credential_store = CredentialStore::new().unwrap_or_else(|_| {
            let temp_path = std::env::temp_dir().join("radium_credentials.json");
            CredentialStore::with_path(temp_path)
        });

        Self {
            metadata,
            credential_store: Arc::new(credential_store),
        }
    }

    /// Gets the API key from credential store.
    fn get_api_key(&self) -> Result<String> {
        self.credential_store
            .get(ProviderType::Gemini)
            .map_err(|e| EngineError::AuthenticationFailed(format!("Failed to get API key: {}", e)))
    }

    /// Creates a GeminiModel instance with the given model ID.
    fn create_model(&self, model_id: &str) -> Result<GeminiModel> {
        let api_key = self.get_api_key()?;
        Ok(GeminiModel::with_api_key(model_id.to_string(), api_key))
    }

    /// Converts ModelError to EngineError.
    fn convert_error(error: radium_abstraction::ModelError) -> EngineError {
        match error {
            radium_abstraction::ModelError::RequestError(msg) => {
                EngineError::ExecutionError(format!("Request error: {}", msg))
            }
            radium_abstraction::ModelError::ModelResponseError(msg) => {
                EngineError::ExecutionError(format!("Model response error: {}", msg))
            }
            radium_abstraction::ModelError::SerializationError(msg) => {
                EngineError::ExecutionError(format!("Serialization error: {}", msg))
            }
            radium_abstraction::ModelError::UnsupportedModelProvider(msg) => {
                EngineError::InvalidConfig(format!("Unsupported provider: {}", msg))
            }
            radium_abstraction::ModelError::QuotaExceeded { provider, message } => {
                EngineError::ExecutionError(format!(
                    "Quota exceeded for {}: {}",
                    provider,
                    message.unwrap_or_else(|| "Unknown".to_string())
                ))
            }
            radium_abstraction::ModelError::Other(msg) => {
                EngineError::ExecutionError(format!("Other error: {}", msg))
            }
        }
    }

    /// Converts ModelUsage to TokenUsage.
    fn convert_usage(usage: Option<ModelUsage>) -> Option<TokenUsage> {
        usage.map(|u| TokenUsage {
            input_tokens: u.prompt_tokens as u64,
            output_tokens: u.completion_tokens as u64,
            total_tokens: u.total_tokens as u64,
        })
    }

    /// Converts ExecutionRequest to ModelParameters.
    fn convert_parameters(request: &ExecutionRequest) -> Option<ModelParameters> {
        if request.temperature.is_some() || request.max_tokens.is_some() {
            Some(ModelParameters {
                temperature: request.temperature,
                top_p: None,
                max_tokens: request.max_tokens.map(|t| t as u32),
                top_k: None,
                frequency_penalty: None,
                presence_penalty: None,
                response_format: None,
                stop_sequences: None,
            })
        } else {
            None
        }
    }
}

impl Default for GeminiEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Engine for GeminiEngine {
    fn metadata(&self) -> &EngineMetadata {
        &self.metadata
    }

    async fn is_available(&self) -> bool {
        // Gemini is API-based, always available if authenticated
        true
    }

    async fn is_authenticated(&self) -> Result<bool> {
        match self.get_api_key() {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn execute(&self, request: ExecutionRequest) -> Result<ExecutionResponse> {
        // Create model instance
        let model = self.create_model(&request.model)?;

        // Convert parameters
        let parameters = Self::convert_parameters(&request);

        // Build prompt (include system message if present)
        // Note: Gemini doesn't support system messages directly, so we prepend it to the prompt
        let prompt = if let Some(system) = &request.system {
            format!("{}\n\n{}", system, request.prompt)
        } else {
            request.prompt
        };

        // Execute via radium-models (Model trait)
        use radium_abstraction::Model;
        let response = model
            .generate_text(&prompt, parameters)
            .await
            .map_err(Self::convert_error)?;

        // Convert response
        Ok(ExecutionResponse {
            content: response.content,
            usage: Self::convert_usage(response.usage),
            model: response.model_id.unwrap_or_else(|| request.model.clone()),
            raw: None, // radium-models doesn't provide raw response
            execution_duration: None, // Cloud models use token-based costing
        })
    }

    fn default_model(&self) -> String {
        "gemini-2.0-flash-exp".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gemini_engine_metadata() {
        let engine = GeminiEngine::new();
        let metadata = engine.metadata();

        assert_eq!(metadata.id, "gemini");
        assert_eq!(metadata.name, "Gemini");
        assert!(metadata.requires_auth);
        assert_eq!(metadata.models.len(), 5);
    }

    #[tokio::test]
    async fn test_gemini_engine_is_available() {
        let engine = GeminiEngine::new();
        assert!(engine.is_available().await);
    }

    #[tokio::test]
    async fn test_gemini_engine_default_model() {
        let engine = GeminiEngine::new();
        assert_eq!(engine.default_model(), "gemini-2.0-flash-exp");
    }
}

