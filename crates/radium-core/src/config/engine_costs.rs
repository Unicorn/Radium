//! Engine cost configuration for local model cost tracking.

use crate::monitoring::{MonitoringError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Engine cost configuration structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineCostsConfig {
    /// Path to the configuration file.
    #[serde(skip)]
    pub config_path: PathBuf,
    /// Engine cost rates by engine ID.
    #[serde(flatten)]
    pub engines: HashMap<String, EngineConfig>,
}

/// Configuration for a single engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    /// Cost per second of execution in USD.
    pub cost_per_second: f64,
    /// Minimum billable duration in seconds.
    #[serde(default = "default_min_billable_duration")]
    pub min_billable_duration: f64,
}

fn default_min_billable_duration() -> f64 {
    0.0
}

/// TOML structure for serialization/deserialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EngineCostsConfigToml {
    /// Engines section.
    #[serde(default)]
    engines: HashMap<String, EngineConfig>,
}

impl EngineCostsConfig {
    /// Load engine costs configuration from a TOML file.
    ///
    /// # Arguments
    /// * `path` - Path to the TOML configuration file
    ///
    /// # Errors
    /// Returns error if file cannot be read, parsed, or validated.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        
        // If file doesn't exist, return empty config (backward compatible)
        if !path.exists() {
            return Ok(Self {
                config_path: path.to_path_buf(),
                engines: HashMap::new(),
            });
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| MonitoringError::Io(e))?;

        let toml_config: EngineCostsConfigToml = toml::from_str(&content)
            .map_err(|e| MonitoringError::Other(format!("Failed to parse engine costs config: {}", e)))?;

        let config = Self {
            config_path: path.to_path_buf(),
            engines: toml_config.engines,
        };

        config.validate()?;
        Ok(config)
    }

    /// Validate the configuration.
    ///
    /// # Errors
    /// Returns error if any configuration value is invalid.
    pub fn validate(&self) -> Result<()> {
        for (engine_id, engine_config) in &self.engines {
            if engine_config.cost_per_second < 0.0 {
                return Err(MonitoringError::Other(format!(
                    "Invalid cost_per_second for engine '{}': must be >= 0.0",
                    engine_id
                )));
            }
            if engine_config.min_billable_duration < 0.0 {
                return Err(MonitoringError::Other(format!(
                    "Invalid min_billable_duration for engine '{}': must be >= 0.0",
                    engine_id
                )));
            }
        }
        Ok(())
    }

    /// Save the configuration to file.
    ///
    /// # Errors
    /// Returns error if file cannot be written.
    pub fn save(&self) -> Result<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| MonitoringError::Io(e))?;
        }

        // Serialize to TOML
        let toml_config = EngineCostsConfigToml {
            engines: self.engines.clone(),
        };

        let content = toml::to_string_pretty(&toml_config)
            .map_err(|e| MonitoringError::Other(format!("Failed to serialize config: {}", e)))?;

        std::fs::write(&self.config_path, content)
            .map_err(|e| MonitoringError::Io(e))?;

        Ok(())
    }

    /// Get cost rate for an engine.
    ///
    /// # Arguments
    /// * `engine_id` - Engine identifier
    ///
    /// # Returns
    /// Cost rate if configured, None otherwise
    pub fn get_rate(&self, engine_id: &str) -> Option<&EngineConfig> {
        self.engines.get(engine_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_valid_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("engine-costs.toml");

        let toml_content = r#"
[engines.ollama]
cost_per_second = 0.0001
min_billable_duration = 0.1

[engines.lm-studio]
cost_per_second = 0.00015
min_billable_duration = 0.1
"#;
        std::fs::write(&config_path, toml_content).unwrap();

        let config = EngineCostsConfig::load(&config_path).unwrap();
        assert_eq!(config.engines.len(), 2);
        
        let ollama = config.get_rate("ollama").unwrap();
        assert_eq!(ollama.cost_per_second, 0.0001);
        assert_eq!(ollama.min_billable_duration, 0.1);

        let lm_studio = config.get_rate("lm-studio").unwrap();
        assert_eq!(lm_studio.cost_per_second, 0.00015);
        assert_eq!(lm_studio.min_billable_duration, 0.1);
    }

    #[test]
    fn test_load_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nonexistent.toml");

        let config = EngineCostsConfig::load(&config_path).unwrap();
        assert_eq!(config.engines.len(), 0);
    }

    #[test]
    fn test_validate_negative_cost() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("engine-costs.toml");

        let toml_content = r#"
[engines.ollama]
cost_per_second = -0.0001
min_billable_duration = 0.1
"#;
        std::fs::write(&config_path, toml_content).unwrap();

        let result = EngineCostsConfig::load(&config_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cost_per_second"));
    }

    #[test]
    fn test_validate_negative_min_duration() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("engine-costs.toml");

        let toml_content = r#"
[engines.ollama]
cost_per_second = 0.0001
min_billable_duration = -0.1
"#;
        std::fs::write(&config_path, toml_content).unwrap();

        let result = EngineCostsConfig::load(&config_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("min_billable_duration"));
    }

    #[test]
    fn test_default_min_billable_duration() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("engine-costs.toml");

        let toml_content = r#"
[engines.ollama]
cost_per_second = 0.0001
"#;
        std::fs::write(&config_path, toml_content).unwrap();

        let config = EngineCostsConfig::load(&config_path).unwrap();
        let ollama = config.get_rate("ollama").unwrap();
        assert_eq!(ollama.min_billable_duration, 0.0);
    }

    #[test]
    fn test_save_and_reload() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("engine-costs.toml");

        let mut config = EngineCostsConfig {
            config_path: config_path.clone(),
            engines: HashMap::new(),
        };

        config.engines.insert(
            "ollama".to_string(),
            EngineConfig {
                cost_per_second: 0.0001,
                min_billable_duration: 0.1,
            },
        );

        config.save().unwrap();

        let reloaded = EngineCostsConfig::load(&config_path).unwrap();
        assert_eq!(reloaded.engines.len(), 1);
        let ollama = reloaded.get_rate("ollama").unwrap();
        assert_eq!(ollama.cost_per_second, 0.0001);
        assert_eq!(ollama.min_billable_duration, 0.1);
    }

    #[test]
    fn test_get_rate_missing() {
        let config = EngineCostsConfig {
            config_path: PathBuf::from("/tmp/test.toml"),
            engines: HashMap::new(),
        };

        assert!(config.get_rate("unknown").is_none());
    }
}

