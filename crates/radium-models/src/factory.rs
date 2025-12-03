//! Model factory for creating model instances from configuration.
//!
//! This module provides functionality to create model instances based on configuration,
//! handling API key loading from environment variables.

use crate::{GeminiModel, MockModel, OpenAIModel};
use radium_abstraction::{Model, ModelError};
use std::str::FromStr;
use std::sync::Arc;
use tracing::{debug, error};

/// Model type enumeration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelType {
    /// Mock model for testing.
    Mock,
    /// Google Gemini model.
    Gemini,
    /// OpenAI model.
    OpenAI,
}

impl FromStr for ModelType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mock" => Ok(Self::Mock),
            "gemini" => Ok(Self::Gemini),
            "openai" => Ok(Self::OpenAI),
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
}

impl ModelConfig {
    /// Creates a new `ModelConfig` with the given type and model ID.
    ///
    /// # Arguments
    /// * `model_type` - The type of model
    /// * `model_id` - The model ID
    #[must_use]
    pub fn new(model_type: ModelType, model_id: String) -> Self {
        Self { model_type, model_id, api_key: None }
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
}
