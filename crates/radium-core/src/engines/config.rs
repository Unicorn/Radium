//! Engine configuration structures.

use super::error::{EngineError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Global engine configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalEngineConfig {
    /// Default engine ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,

    /// Per-engine configurations.
    #[serde(flatten)]
    pub engines: HashMap<String, PerEngineConfig>,
}

impl GlobalEngineConfig {
    /// Creates a new global engine configuration.
    pub fn new() -> Self {
        Self {
            default: None,
            engines: HashMap::new(),
        }
    }

    /// Gets the configuration for a specific engine.
    pub fn get_engine_config(&self, engine_id: &str) -> Option<&PerEngineConfig> {
        self.engines.get(engine_id)
    }

    /// Sets the configuration for a specific engine.
    pub fn set_engine_config(&mut self, engine_id: String, config: PerEngineConfig) {
        self.engines.insert(engine_id, config);
    }

    /// Validates the configuration.
    pub fn validate(&self) -> Result<()> {
        // Validate default engine exists in engines map if specified
        if let Some(ref default_id) = self.default {
            if !self.engines.contains_key(default_id) {
                // This is OK - engine might be registered but not configured
            }
        }

        // Validate all per-engine configs
        for (engine_id, config) in &self.engines {
            config.validate().map_err(|e| {
                EngineError::InvalidConfig(format!("Invalid config for engine {}: {}", engine_id, e))
            })?;
        }

        Ok(())
    }
}

impl Default for GlobalEngineConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Per-engine configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerEngineConfig {
    /// Default model for this engine.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_model: Option<String>,

    /// Temperature setting (0.0-1.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Maximum tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,

    /// Additional engine-specific parameters.
    #[serde(flatten)]
    pub params: HashMap<String, serde_json::Value>,
}

impl PerEngineConfig {
    /// Creates a new per-engine configuration.
    pub fn new() -> Self {
        Self {
            default_model: None,
            temperature: None,
            max_tokens: None,
            params: HashMap::new(),
        }
    }

    /// Validates the configuration.
    pub fn validate(&self) -> Result<()> {
        // Validate temperature range
        if let Some(temp) = self.temperature {
            if !(0.0..=1.0).contains(&temp) {
                return Err(EngineError::InvalidConfig(format!(
                    "Temperature must be between 0.0 and 1.0, got {}",
                    temp
                )));
            }
        }

        // Validate max_tokens
        if let Some(max) = self.max_tokens {
            if max == 0 {
                return Err(EngineError::InvalidConfig(
                    "max_tokens must be greater than 0".to_string(),
                ));
            }
        }

        Ok(())
    }
}

impl Default for PerEngineConfig {
    fn default() -> Self {
        Self::new()
    }
}
