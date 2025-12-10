//! OpenAI engine provider wrapper implementation.

use crate::auth::{CredentialStore, ProviderType};
use crate::engines::engine_trait::{
    Engine, EngineMetadata, ExecutionRequest, ExecutionResponse, TokenUsage,
};
use crate::engines::error::{EngineError, Result};
use async_trait::async_trait;
use radium_abstraction::{ModelParameters, ModelUsage};
use radium_models::OpenAIModel;
use std::sync::Arc;

/// OpenAI engine wrapper around radium-models::OpenAIModel.
pub struct OpenAIEngine {
    /// Engine metadata.
    metadata: EngineMetadata,
    /// Credential store for API key retrieval.
    credential_store: Arc<CredentialStore>,
}

impl OpenAIEngine {
    /// Creates a new OpenAI engine.
    pub fn new() -> Self {
        let metadata = EngineMetadata::new(
            "openai".to_string(),
            "OpenAI".to_string(),
            "OpenAI GPT models engine".to_string(),
        )
        .with_models(vec![
            "gpt-4".to_string(),
            "gpt-4-turbo".to_string(),
            "gpt-4o".to_string(),
            "gpt-3.5-turbo".to_string(),
            "o1-preview".to_string(),
            "o1-mini".to_string(),
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
            .get(ProviderType::OpenAI)
            .map_err(|e| EngineError::AuthenticationFailed(format!("Failed to get API key: {}", e)))
    }

    /// Creates an OpenAIModel instance with the given model ID.
    fn create_model(&self, model_id: &str) -> Result<OpenAIModel> {
        let api_key = self.get_api_key()?;
        Ok(OpenAIModel::with_api_key(model_id.to_string(), api_key))
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
            radium_abstraction::ModelError::ContentFiltered { provider, reason, .. } => {
                EngineError::ExecutionError(format!(
                    "Content filtered by {}: {}",
                    provider, reason
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

impl Default for OpenAIEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Engine for OpenAIEngine {
    fn metadata(&self) -> &EngineMetadata {
        &self.metadata
    }

    async fn is_available(&self) -> bool {
        // OpenAI is API-based, always available if authenticated
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
        "gpt-4-turbo".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_openai_engine_metadata() {
        let engine = OpenAIEngine::new();
        let metadata = engine.metadata();

        assert_eq!(metadata.id, "openai");
        assert_eq!(metadata.name, "OpenAI");
        assert!(metadata.requires_auth);
        assert_eq!(metadata.models.len(), 6);
    }

    #[tokio::test]
    async fn test_openai_engine_is_available() {
        let engine = OpenAIEngine::new();
        assert!(engine.is_available().await);
    }

    #[tokio::test]
    async fn test_openai_engine_default_model() {
        let engine = OpenAIEngine::new();
        assert_eq!(engine.default_model(), "gpt-4-turbo");
    }
}

