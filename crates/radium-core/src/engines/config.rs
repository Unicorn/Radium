//! Engine configuration structures and validation.

use super::error::{EngineError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Per-engine configuration settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerEngineConfig {
    /// Default model for this engine.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_model: Option<String>,

    /// Temperature setting (0.0-1.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Maximum tokens to generate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,

    /// Additional engine-specific parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<HashMap<String, serde_json::Value>>,
}

impl Default for PerEngineConfig {
    fn default() -> Self {
        Self {
            default_model: None,
            temperature: None,
            max_tokens: None,
            params: None,
        }
    }
}

impl PerEngineConfig {
    /// Validates the configuration.
    ///
    /// # Errors
    /// Returns error if configuration values are invalid.
    pub fn validate(&self) -> Result<()> {
        if let Some(temp) = self.temperature {
            if temp < 0.0 || temp > 1.0 {
                return Err(EngineError::InvalidConfig(format!(
                    "Temperature must be between 0.0 and 1.0, got {}",
                    temp
                )));
            }
        }

        if let Some(max) = self.max_tokens {
            if max == 0 {
                return Err(EngineError::InvalidConfig(
                    "max_tokens must be greater than 0".to_string(),
                ));
            }
            if max > 1_000_000 {
                return Err(EngineError::InvalidConfig(format!(
                    "max_tokens is unreasonably large: {}",
                    max
                )));
            }
        }

        Ok(())
    }

    /// Merges another configuration into this one.
    /// Values from `other` override values in `self`.
    pub fn merge(&mut self, other: PerEngineConfig) {
        if other.default_model.is_some() {
            self.default_model = other.default_model;
        }
        if other.temperature.is_some() {
            self.temperature = other.temperature;
        }
        if other.max_tokens.is_some() {
            self.max_tokens = other.max_tokens;
        }
        if let Some(other_params) = other.params {
            if let Some(ref mut params) = self.params {
                params.extend(other_params);
            } else {
                self.params = Some(other_params);
            }
        }
    }
}

/// Global engine configuration structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalEngineConfig {
    /// Default engine ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,

    /// Per-engine configurations.
    #[serde(flatten)]
    pub engines: HashMap<String, PerEngineConfig>,
}

impl Default for GlobalEngineConfig {
    fn default() -> Self {
        Self {
            default: None,
            engines: HashMap::new(),
        }
    }
}

impl GlobalEngineConfig {
    /// Creates a new empty configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Validates all engine configurations.
    ///
    /// # Errors
    /// Returns error if any configuration is invalid.
    pub fn validate(&self) -> Result<()> {
        for (engine_id, config) in &self.engines {
            config.validate().map_err(|e| {
                EngineError::InvalidConfig(format!("Invalid config for engine '{}': {}", engine_id, e))
            })?;
        }
        Ok(())
    }

    /// Gets configuration for a specific engine.
    pub fn get_engine_config(&self, engine_id: &str) -> Option<&PerEngineConfig> {
        self.engines.get(engine_id)
    }

    /// Gets mutable configuration for a specific engine.
    pub fn get_engine_config_mut(&mut self, engine_id: &str) -> &mut PerEngineConfig {
        self.engines.entry(engine_id.to_string()).or_insert_with(PerEngineConfig::default)
    }

    /// Sets configuration for a specific engine.
    pub fn set_engine_config(&mut self, engine_id: String, config: PerEngineConfig) {
        self.engines.insert(engine_id, config);
    }

    /// Removes configuration for a specific engine.
    pub fn remove_engine_config(&mut self, engine_id: &str) -> Option<PerEngineConfig> {
        self.engines.remove(engine_id)
    }

    /// Merges another configuration into this one.
    /// Values from `other` override values in `self`.
    pub fn merge(&mut self, other: GlobalEngineConfig) {
        if other.default.is_some() {
            self.default = other.default;
        }
        for (engine_id, other_config) in other.engines {
            if let Some(existing_config) = self.engines.get_mut(&engine_id) {
                existing_config.merge(other_config);
            } else {
                self.engines.insert(engine_id, other_config);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_per_engine_config_validation() {
        let mut config = PerEngineConfig::default();
        
        // Valid temperature
        config.temperature = Some(0.7);
        assert!(config.validate().is_ok());

        // Invalid temperature (too high)
        config.temperature = Some(1.5);
        assert!(config.validate().is_err());

        // Invalid temperature (negative)
        config.temperature = Some(-0.1);
        assert!(config.validate().is_err());

        // Valid max_tokens
        config.temperature = Some(0.7);
        config.max_tokens = Some(4096);
        assert!(config.validate().is_ok());

        // Invalid max_tokens (zero)
        config.max_tokens = Some(0);
        assert!(config.validate().is_err());

        // Invalid max_tokens (too large)
        config.max_tokens = Some(2_000_000);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_per_engine_config_merge() {
        let mut config1 = PerEngineConfig {
            default_model: Some("model-1".to_string()),
            temperature: Some(0.7),
            max_tokens: Some(1000),
            params: None,
        };

        let config2 = PerEngineConfig {
            default_model: Some("model-2".to_string()),
            temperature: Some(0.8),
            max_tokens: None,
            params: None,
        };

        config1.merge(config2);

        assert_eq!(config1.default_model, Some("model-2".to_string()));
        assert_eq!(config1.temperature, Some(0.8));
        assert_eq!(config1.max_tokens, Some(1000)); // Not overridden
    }

    #[test]
    fn test_global_engine_config() {
        let mut config = GlobalEngineConfig::new();
        
        config.default = Some("gemini".to_string());
        
        let engine_config = PerEngineConfig {
            default_model: Some("gemini-2.0-flash-exp".to_string()),
            temperature: Some(0.7),
            max_tokens: Some(4096),
            params: None,
        };
        
        config.set_engine_config("gemini".to_string(), engine_config);
        
        assert_eq!(config.default, Some("gemini".to_string()));
        assert!(config.get_engine_config("gemini").is_some());
        assert_eq!(
            config.get_engine_config("gemini").unwrap().default_model,
            Some("gemini-2.0-flash-exp".to_string())
        );
    }

    #[test]
    fn test_global_engine_config_merge() {
        let mut config1 = GlobalEngineConfig::new();
        config1.default = Some("gemini".to_string());
        
        let mut config2 = GlobalEngineConfig::new();
        config2.default = Some("openai".to_string());
        
        let engine_config = PerEngineConfig {
            default_model: Some("gpt-4".to_string()),
            temperature: Some(0.8),
            max_tokens: None,
            params: None,
        };
        config2.set_engine_config("openai".to_string(), engine_config);
        
        config1.merge(config2);
        
        assert_eq!(config1.default, Some("openai".to_string()));
        assert!(config1.get_engine_config("openai").is_some());
    }
}

