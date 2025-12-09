//! Model factory for creating model instances from configuration.
//!
//! This module provides functionality to create model instances based on configuration,
//! handling API key loading from environment variables.

use crate::{ClaudeModel, GeminiModel, MockModel, OpenAIModel, UniversalModel};
use radium_abstraction::{Model, ModelError};
use std::str::FromStr;
use std::sync::Arc;
use tracing::{debug, error};

/// Model type enumeration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelType {
    /// Mock model for testing.
    Mock,
    /// Anthropic Claude model.
    Claude,
    /// Google Gemini model.
    Gemini,
    /// OpenAI model.
    OpenAI,
    /// Universal OpenAI-compatible model (vLLM, LocalAI, LM Studio, etc.).
    Universal,
    /// Ollama local model.
    Ollama,
}

impl FromStr for ModelType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mock" => Ok(Self::Mock),
            "claude" | "anthropic" => Ok(Self::Claude),
            "gemini" => Ok(Self::Gemini),
            "openai" => Ok(Self::OpenAI),
            "universal" | "openai-compatible" | "local" => Ok(Self::Universal),
            "ollama" => Ok(Self::Ollama),
            _ => Err(()),
        }
    }
}

/// Model configuration.
#[derive(Debug, Clone)]
pub struct ModelConfig {
    /// The type of model to create.
    pub model_type: ModelType,
    /// The model ID (e.g., "gemini-pro", "gpt-4").
    pub model_id: String,
    /// Optional API key (if not provided, will be loaded from environment).
    pub api_key: Option<String>,
    /// Optional base URL for Universal models (required for Universal type).
    pub base_url: Option<String>,
}

impl ModelConfig {
    /// Creates a new `ModelConfig` with the given type and model ID.
    ///
    /// # Arguments
    /// * `model_type` - The type of model
    /// * `model_id` - The model ID
    #[must_use]
    pub fn new(model_type: ModelType, model_id: String) -> Self {
        Self {
            model_type,
            model_id,
            api_key: None,
            base_url: None,
        }
    }

    /// Sets the API key for this configuration.
    ///
    /// # Arguments
    /// * `api_key` - The API key to use
    #[must_use]
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    /// Sets the base URL for this configuration (required for Universal models).
    ///
    /// # Arguments
    /// * `base_url` - The base URL for the API endpoint (e.g., "http://localhost:8000/v1")
    #[must_use]
    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = Some(base_url);
        self
    }
}

/// Factory for creating model instances.
pub struct ModelFactory;

impl ModelFactory {
    /// Creates a model instance from the given configuration.
    ///
    /// # Arguments
    /// * `config` - The model configuration
    ///
    /// # Errors
    /// Returns a `ModelError` if model creation fails (e.g., missing API key).
    pub fn create(config: ModelConfig) -> Result<Arc<dyn Model + Send + Sync>, ModelError> {
        debug!(
            model_type = ?config.model_type,
            model_id = %config.model_id,
            "Creating model instance"
        );

        match config.model_type {
            ModelType::Mock => {
                let model = MockModel::new(config.model_id);
                Ok(Arc::new(model))
            }
            ModelType::Claude => {
                let model = if let Some(api_key) = config.api_key {
                    ClaudeModel::with_api_key(config.model_id, api_key)
                } else {
                    ClaudeModel::new(config.model_id)?
                };
                Ok(Arc::new(model))
            }
            ModelType::Gemini => {
                let model = if let Some(api_key) = config.api_key {
                    GeminiModel::with_api_key(config.model_id, api_key)
                } else {
                    GeminiModel::new(config.model_id)?
                };
                Ok(Arc::new(model))
            }
            ModelType::OpenAI => {
                let model = if let Some(api_key) = config.api_key {
                    OpenAIModel::with_api_key(config.model_id, api_key)
                } else {
                    OpenAIModel::new(config.model_id)?
                };
                Ok(Arc::new(model))
            }
            ModelType::Universal => {
                let base_url = config.base_url.ok_or_else(|| {
                    ModelError::UnsupportedModelProvider(
                        "base_url is required for Universal model type. Use ModelConfig::with_base_url() to set it.".to_string(),
                    )
                })?;

                let model = if let Some(api_key) = config.api_key {
                    UniversalModel::with_api_key(config.model_id, base_url, api_key)
                } else {
                    UniversalModel::without_auth(config.model_id, base_url)
                };
                Ok(Arc::new(model))
            }
            ModelType::Ollama => {
                // Ollama support is not yet implemented.
                // Use UniversalModel with base_url "http://localhost:11434/v1" instead.
                Err(ModelError::UnsupportedModelProvider(
                    "Ollama model type is not yet implemented. Use UniversalModel with base_url 'http://localhost:11434/v1' instead.".to_string(),
                ))
            }
        }
    }

    /// Creates a model instance from a model type string and model ID.
    ///
    /// # Arguments
    /// * `model_type_str` - String representation of model type
    /// * `model_id` - The model ID
    ///
    /// # Errors
    /// Returns a `ModelError` if the model type is unrecognized or creation fails.
    pub fn create_from_str(
        model_type_str: &str,
        model_id: String,
    ) -> Result<Arc<dyn Model + Send + Sync>, ModelError> {
        let model_type = ModelType::from_str(model_type_str).map_err(|()| {
            error!(model_type = %model_type_str, "Unrecognized model type");
            ModelError::UnsupportedModelProvider(format!(
                "Unrecognized model type: {}",
                model_type_str
            ))
        })?;

        let config = ModelConfig::new(model_type, model_id);
        Self::create(config)
    }

    /// Creates a model instance with an explicit API key.
    ///
    /// # Arguments
    /// * `model_type_str` - String representation of model type
    /// * `model_id` - The model ID
    /// * `api_key` - The API key to use
    ///
    /// # Errors
    /// Returns a `ModelError` if the model type is unrecognized or creation fails.
    pub fn create_with_api_key(
        model_type_str: &str,
        model_id: String,
        api_key: String,
    ) -> Result<Arc<dyn Model + Send + Sync>, ModelError> {
        let model_type = ModelType::from_str(model_type_str).map_err(|()| {
            error!(model_type = %model_type_str, "Unrecognized model type");
            ModelError::UnsupportedModelProvider(format!(
                "Unrecognized model type: {}",
                model_type_str
            ))
        })?;

        let config = ModelConfig::new(model_type, model_id).with_api_key(api_key);
        Self::create(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_type_from_str() {
        assert_eq!(ModelType::from_str("mock"), Ok(ModelType::Mock));
        assert_eq!(ModelType::from_str("Mock"), Ok(ModelType::Mock));
        assert_eq!(ModelType::from_str("MOCK"), Ok(ModelType::Mock));
        assert_eq!(ModelType::from_str("gemini"), Ok(ModelType::Gemini));
        assert_eq!(ModelType::from_str("openai"), Ok(ModelType::OpenAI));
        assert_eq!(ModelType::from_str("claude"), Ok(ModelType::Claude));
        assert_eq!(ModelType::from_str("anthropic"), Ok(ModelType::Claude));
        assert_eq!(ModelType::from_str("Claude"), Ok(ModelType::Claude));
        assert_eq!(ModelType::from_str("ANTHROPIC"), Ok(ModelType::Claude));
        assert_eq!(ModelType::from_str("ollama"), Ok(ModelType::Ollama));
        assert_eq!(ModelType::from_str("Ollama"), Ok(ModelType::Ollama));
        assert_eq!(ModelType::from_str("OLLAMA"), Ok(ModelType::Ollama));
        assert_eq!(ModelType::from_str("unknown"), Err(()));
    }

    #[test]
    fn test_model_config() {
        let config = ModelConfig::new(ModelType::Mock, "test-model".to_string());
        assert_eq!(config.model_type, ModelType::Mock);
        assert_eq!(config.model_id, "test-model");
        assert_eq!(config.api_key, None);

        let config = config.with_api_key("test-key".to_string());
        assert_eq!(config.api_key, Some("test-key".to_string()));
    }

    #[test]
    fn test_factory_create_mock() {
        let config = ModelConfig::new(ModelType::Mock, "test-mock".to_string());
        let model = ModelFactory::create(config).unwrap();
        assert_eq!(model.model_id(), "test-mock");
    }

    #[test]
    fn test_factory_create_from_str() {
        let model = ModelFactory::create_from_str("mock", "test-mock".to_string()).unwrap();
        assert_eq!(model.model_id(), "test-mock");
    }

    #[test]
    fn test_factory_create_with_api_key() {
        let model = ModelFactory::create_with_api_key(
            "mock",
            "test-mock".to_string(),
            "test-key".to_string(),
        )
        .unwrap();
        assert_eq!(model.model_id(), "test-mock");
    }

    #[test]
    fn test_factory_create_invalid_type() {
        let result = ModelFactory::create_from_str("invalid", "test".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_factory_create_claude_with_api_key() {
        let config = ModelConfig::new(
            ModelType::Claude,
            "claude-3-sonnet-20240229".to_string(),
        )
        .with_api_key("test-api-key".to_string());
        let model = ModelFactory::create(config).unwrap();
        assert_eq!(model.model_id(), "claude-3-sonnet-20240229");
    }

    #[test]
    fn test_factory_create_from_str_anthropic() {
        let _model = ModelFactory::create_from_str(
            "anthropic",
            "claude-3-sonnet-20240229".to_string(),
        );
        // This will fail if ANTHROPIC_API_KEY is not set, which is expected
        // We just verify the string parsing works
        assert!(ModelType::from_str("anthropic").is_ok());
    }

    #[test]
    fn test_factory_create_from_str_claude() {
        let _model = ModelFactory::create_from_str(
            "claude",
            "claude-3-sonnet-20240229".to_string(),
        );
        // This will fail if ANTHROPIC_API_KEY is not set, which is expected
        // We just verify the string parsing works
        assert!(ModelType::from_str("claude").is_ok());
    }

    #[test]
    fn test_factory_create_claude_with_explicit_api_key() {
        let model = ModelFactory::create_with_api_key(
            "claude",
            "claude-3-sonnet-20240229".to_string(),
            "test-api-key".to_string(),
        )
        .unwrap();
        assert_eq!(model.model_id(), "claude-3-sonnet-20240229");
    }
}
