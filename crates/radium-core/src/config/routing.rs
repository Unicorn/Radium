//! Routing configuration for Smart/Eco model routing system.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;

/// Routing configuration for model tier selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    /// Smart tier model specification (format: "engine:model").
    #[serde(default = "default_smart_model")]
    pub smart_model: String,
    
    /// Eco tier model specification (format: "engine:model").
    #[serde(default = "default_eco_model")]
    pub eco_model: String,
    
    /// Whether auto-routing is enabled.
    #[serde(default = "default_true")]
    pub auto_route: bool,
    
    /// Complexity threshold for routing (0-100).
    #[serde(default = "default_threshold")]
    pub complexity_threshold: f64,
    
    /// Complexity estimation weights.
    #[serde(default)]
    pub weights: ComplexityWeights,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            smart_model: default_smart_model(),
            eco_model: default_eco_model(),
            auto_route: default_true(),
            complexity_threshold: default_threshold(),
            weights: ComplexityWeights::default(),
        }
    }
}

fn default_smart_model() -> String {
    "claude:claude-sonnet-4.5".to_string()
}

fn default_eco_model() -> String {
    "claude:claude-haiku-4.5".to_string()
}

fn default_true() -> bool {
    true
}

fn default_threshold() -> f64 {
    60.0
}

/// Complexity estimation weights.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityWeights {
    /// Weight for token count factor (0-1).
    #[serde(default = "default_token_weight")]
    pub token_count: f64,
    
    /// Weight for task type factor (0-1).
    #[serde(default = "default_task_type_weight")]
    pub task_type: f64,
    
    /// Weight for reasoning factor (0-1).
    #[serde(default = "default_reasoning_weight")]
    pub reasoning: f64,
    
    /// Weight for context complexity factor (0-1).
    #[serde(default = "default_context_weight")]
    pub context: f64,
}

impl Default for ComplexityWeights {
    fn default() -> Self {
        Self {
            token_count: default_token_weight(),
            task_type: default_task_type_weight(),
            reasoning: default_reasoning_weight(),
            context: default_context_weight(),
        }
    }
}

fn default_token_weight() -> f64 {
    0.3
}

fn default_task_type_weight() -> f64 {
    0.4
}

fn default_reasoning_weight() -> f64 {
    0.2
}

fn default_context_weight() -> f64 {
    0.1
}

/// Errors that can occur during configuration loading or validation.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),
    
    #[error("Validation error: {0}")]
    Validation(String),
}

impl RoutingConfig {
    /// Loads routing configuration from file.
    ///
    /// Searches in order:
    /// 1. `./radium.toml` (workspace config)
    /// 2. `~/.radium/config.toml` (home directory config)
    ///
    /// Falls back to defaults if no file is found.
    ///
    /// # Errors
    /// Returns `ConfigError` if file exists but parsing fails or validation fails.
    pub fn load() -> Result<Self, ConfigError> {
        // Try workspace config first
        let workspace_config = Path::new("./radium.toml");
        if workspace_config.exists() {
            if let Ok(config) = Self::load_from_file(workspace_config) {
                return Ok(config);
            }
        }
        
        // Try home directory config
        if let Ok(home) = std::env::var("HOME") {
            let home_config = PathBuf::from(home).join(".radium/config.toml");
            if home_config.exists() {
                if let Ok(config) = Self::load_from_file(&home_config) {
                    return Ok(config);
                }
            }
        }
        
        // Fall back to defaults
        Ok(Self::default())
    }
    
    /// Loads configuration from a specific file.
    ///
    /// # Arguments
    /// * `path` - Path to TOML configuration file
    ///
    /// # Errors
    /// Returns `ConfigError` if file cannot be read, parsed, or validated.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let contents = fs::read_to_string(path)?;
        let config: RoutingConfig = toml::from_str(&contents)?;
        config.validate()?;
        Ok(config)
    }
    
    /// Validates the configuration.
    ///
    /// Checks:
    /// - Threshold is in range [0.0, 100.0]
    /// - Weights sum to approximately 1.0 (with tolerance)
    /// - Model strings are non-empty
    ///
    /// # Errors
    /// Returns `ConfigError::Validation` if any check fails.
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate threshold
        if !(0.0..=100.0).contains(&self.complexity_threshold) {
            return Err(ConfigError::Validation(format!(
                "complexity_threshold must be between 0.0 and 100.0, got {}",
                self.complexity_threshold
            )));
        }
        
        // Validate weights sum to ~1.0
        let weight_sum = self.weights.token_count
            + self.weights.task_type
            + self.weights.reasoning
            + self.weights.context;
        
        const TOLERANCE: f64 = 0.01;
        if (weight_sum - 1.0).abs() > TOLERANCE {
            return Err(ConfigError::Validation(format!(
                "Complexity weights must sum to 1.0 (±{}), got {}",
                TOLERANCE, weight_sum
            )));
        }
        
        // Validate model strings
        if self.smart_model.is_empty() {
            return Err(ConfigError::Validation(
                "smart_model cannot be empty".to_string()
            ));
        }
        
        if self.eco_model.is_empty() {
            return Err(ConfigError::Validation(
                "eco_model cannot be empty".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Parses model string into engine and model parts.
    ///
    /// # Arguments
    /// * `model_spec` - Model specification in format "engine:model"
    ///
    /// # Returns
    /// Tuple of (engine, model_id) or error if format is invalid.
    pub fn parse_model_spec(&self, model_spec: &str) -> Result<(String, String), ConfigError> {
        let parts: Vec<&str> = model_spec.split(':').collect();
        if parts.len() != 2 {
            return Err(ConfigError::Validation(format!(
                "Invalid model format '{}', expected 'engine:model'",
                model_spec
            )));
        }
        Ok((parts[0].to_string(), parts[1].to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_default_config() {
        let config = RoutingConfig::default();
        assert_eq!(config.smart_model, "claude:claude-sonnet-4.5");
        assert_eq!(config.eco_model, "claude:claude-haiku-4.5");
        assert!(config.auto_route);
        assert_eq!(config.complexity_threshold, 60.0);
    }
    
    #[test]
    fn test_load_from_toml() {
        let toml = r#"
            smart_model = "claude:claude-sonnet-4.5"
            eco_model = "claude:claude-haiku-4.5"
            auto_route = true
            complexity_threshold = 70.0
            
            [weights]
            token_count = 0.3
            task_type = 0.4
            reasoning = 0.2
            context = 0.1
        "#;
        
        let file = NamedTempFile::new().unwrap();
        std::fs::write(file.path(), toml).unwrap();
        
        let config = RoutingConfig::load_from_file(file.path()).unwrap();
        assert_eq!(config.complexity_threshold, 70.0);
        assert_eq!(config.weights.token_count, 0.3);
    }
    
    #[test]
    fn test_validation_threshold_range() {
        let mut config = RoutingConfig::default();
        config.complexity_threshold = 150.0;
        
        assert!(config.validate().is_err());
        
        config.complexity_threshold = -10.0;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_validation_weights_sum() {
        let mut config = RoutingConfig::default();
        config.weights.token_count = 0.5;
        config.weights.task_type = 0.5;
        config.weights.reasoning = 0.5;
        config.weights.context = 0.5;
        
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_parse_model_spec() {
        let config = RoutingConfig::default();
        let (engine, model) = config.parse_model_spec("claude:claude-sonnet-4.5").unwrap();
        assert_eq!(engine, "claude");
        assert_eq!(model, "claude-sonnet-4.5");
    }
    
    #[test]
    fn test_parse_model_spec_invalid() {
        let config = RoutingConfig::default();
        assert!(config.parse_model_spec("invalid").is_err());
        assert!(config.parse_model_spec("too:many:parts").is_err());
    }
}
