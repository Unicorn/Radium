//! TOML configuration file support for routing rules.

use super::types::{ComplexityWeights, FallbackChain, RoutingStrategy};
use radium_models::{ModelConfig, ModelType};
use serde::Deserialize;
use std::path::Path;
use thiserror::Error;

/// Errors that can occur during configuration loading.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// I/O error reading the file.
    #[error("Failed to read configuration file: {0}")]
    Io(#[from] std::io::Error),
    
    /// TOML parsing error.
    #[error("Failed to parse TOML configuration: {0}")]
    Toml(#[from] toml::de::Error),
    
    /// Configuration validation error.
    #[error("Invalid configuration: {0}")]
    Validation(String),
}

/// Result type for configuration operations.
pub type Result<T> = std::result::Result<T, ConfigError>;

/// Routing configuration loaded from TOML.
#[derive(Debug, Clone, Deserialize)]
pub struct RoutingConfig {
    /// Default routing strategy.
    #[serde(default = "default_strategy")]
    pub default_strategy: String,
    
    /// Complexity threshold for routing (optional, uses router default if not set).
    pub threshold: Option<f64>,
    
    /// Fallback chains.
    #[serde(default)]
    pub chains: Vec<FallbackChainConfig>,
    
    /// Routing rules.
    #[serde(default)]
    pub rules: Vec<RoutingRule>,
}

fn default_strategy() -> String {
    "complexity_based".to_string()
}

/// Fallback chain configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct FallbackChainConfig {
    /// Chain name.
    pub name: String,
    
    /// Ordered list of model specifications (e.g., "claude:sonnet-4.5").
    pub models: Vec<String>,
}

/// Routing rule for conditional model selection.
#[derive(Debug, Clone, Deserialize)]
pub struct RoutingRule {
    /// Minimum complexity score for this rule (optional).
    pub complexity_min: Option<f64>,
    
    /// Maximum complexity score for this rule (optional).
    pub complexity_max: Option<f64>,
    
    /// Strategy to use for this rule (optional).
    pub strategy: Option<String>,
    
    /// List of model specifications allowed for this rule.
    pub models: Vec<String>,
}

/// Configuration loader for routing settings.
pub struct RoutingConfigLoader;

impl RoutingConfigLoader {
    /// Loads routing configuration from a TOML file.
    ///
    /// # Arguments
    /// * `path` - Path to the configuration file
    ///
    /// # Errors
    /// Returns error if file cannot be read or parsed.
    pub fn load(path: &Path) -> Result<RoutingConfig> {
        let content = std::fs::read_to_string(path)?;
        let config: RoutingConfig = toml::from_str(&content)?;
        
        // Validate configuration
        Self::validate(&config)?;
        
        Ok(config)
    }
    
    /// Validates routing configuration.
    ///
    /// # Arguments
    /// * `config` - Configuration to validate
    ///
    /// # Errors
    /// Returns error if configuration is invalid.
    pub fn validate(config: &RoutingConfig) -> Result<()> {
        // Validate default strategy
        if RoutingStrategy::from_str(&config.default_strategy).is_none() {
            return Err(ConfigError::Validation(format!(
                "Invalid default strategy: {}. Valid options: complexity_based, cost_optimized, latency_optimized, quality_optimized",
                config.default_strategy
            )));
        }
        
        // Validate threshold if set
        if let Some(threshold) = config.threshold {
            if threshold < 0.0 || threshold > 100.0 {
                return Err(ConfigError::Validation(format!(
                    "Invalid threshold: {}. Must be between 0.0 and 100.0",
                    threshold
                )));
            }
        }
        
        // Validate fallback chains
        for chain in &config.chains {
            if chain.models.is_empty() {
                return Err(ConfigError::Validation(format!(
                    "Fallback chain '{}' must have at least one model",
                    chain.name
                )));
            }
            
            // Validate model spec format
            for model_spec in &chain.models {
                Self::validate_model_spec(model_spec)?;
            }
        }
        
        // Validate routing rules
        for (idx, rule) in config.rules.iter().enumerate() {
            // Validate complexity range
            if let (Some(min), Some(max)) = (rule.complexity_min, rule.complexity_max) {
                if min > max {
                    return Err(ConfigError::Validation(format!(
                        "Rule {}: complexity_min ({}) must be <= complexity_max ({})",
                        idx, min, max
                    )));
                }
            }
            
            // Validate strategy if set
            if let Some(ref strategy) = rule.strategy {
                if RoutingStrategy::from_str(strategy).is_none() {
                    return Err(ConfigError::Validation(format!(
                        "Rule {}: Invalid strategy: {}. Valid options: complexity_based, cost_optimized, latency_optimized, quality_optimized",
                        idx, strategy
                    )));
                }
            }
            
            // Validate model specs
            if rule.models.is_empty() {
                return Err(ConfigError::Validation(format!(
                    "Rule {}: Must specify at least one model",
                    idx
                )));
            }
            
            for model_spec in &rule.models {
                Self::validate_model_spec(model_spec)?;
            }
        }
        
        Ok(())
    }
    
    /// Validates model specification format.
    fn validate_model_spec(spec: &str) -> Result<()> {
        let parts: Vec<&str> = spec.split(':').collect();
        if parts.len() != 2 {
            return Err(ConfigError::Validation(format!(
                "Invalid model spec format: '{}'. Expected 'engine:model' (e.g., 'claude:sonnet-4.5')",
                spec
            )));
        }
        
        let engine = parts[0];
        let valid_engines = ["claude", "openai", "gemini", "mock"];
        if !valid_engines.contains(&engine) {
            return Err(ConfigError::Validation(format!(
                "Invalid engine '{}' in model spec '{}'. Valid engines: {}",
                engine,
                spec,
                valid_engines.join(", ")
            )));
        }
        
        Ok(())
    }
    
    /// Parses a model specification into ModelConfig.
    ///
    /// # Arguments
    /// * `spec` - Model specification (e.g., "claude:sonnet-4.5")
    ///
    /// # Errors
    /// Returns error if specification is invalid.
    pub fn parse_model_spec(spec: &str) -> Result<ModelConfig> {
        let parts: Vec<&str> = spec.split(':').collect();
        if parts.len() != 2 {
            return Err(ConfigError::Validation(format!(
                "Invalid model spec format: '{}'",
                spec
            )));
        }
        
        let engine = parts[0];
        let model_id = parts[1].to_string();
        
        let model_type = match engine {
            "claude" => ModelType::Claude,
            "openai" => ModelType::OpenAI,
            "gemini" => ModelType::Gemini,
            "mock" => ModelType::Mock,
            _ => {
                return Err(ConfigError::Validation(format!(
                    "Unsupported engine: {}",
                    engine
                )));
            }
        };
        
        Ok(ModelConfig::new(model_type, model_id))
    }
    
    /// Builds fallback chains from configuration.
    ///
    /// # Arguments
    /// * `config` - Routing configuration
    ///
    /// # Errors
    /// Returns error if chain configuration is invalid.
    pub fn build_fallback_chains(config: &RoutingConfig) -> Result<Vec<(String, FallbackChain)>> {
        let mut chains = Vec::new();
        
        for chain_config in &config.chains {
            let mut models = Vec::new();
            for model_spec in &chain_config.models {
                models.push(Self::parse_model_spec(model_spec)?);
            }
            
            chains.push((
                chain_config.name.clone(),
                FallbackChain::new(models),
            ));
        }
        
        Ok(chains)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_valid_config() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
default_strategy = "cost_optimized"
threshold = 70.0

[[chains]]
name = "default"
models = ["claude:sonnet-4.5", "openai:gpt-4", "mock:test"]

[[rules]]
complexity_min = 80.0
strategy = "quality_optimized"
models = ["claude:sonnet-4.5", "openai:gpt-4-turbo"]
"#
        )
        .unwrap();
        
        let config = RoutingConfigLoader::load(file.path()).unwrap();
        assert_eq!(config.default_strategy, "cost_optimized");
        assert_eq!(config.threshold, Some(70.0));
        assert_eq!(config.chains.len(), 1);
        assert_eq!(config.rules.len(), 1);
    }

    #[test]
    fn test_validate_invalid_strategy() {
        let config = RoutingConfig {
            default_strategy: "invalid_strategy".to_string(),
            threshold: None,
            chains: Vec::new(),
            rules: Vec::new(),
        };
        
        let result = RoutingConfigLoader::validate(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_model_spec() {
        assert!(RoutingConfigLoader::validate_model_spec("claude:sonnet-4.5").is_ok());
        assert!(RoutingConfigLoader::validate_model_spec("invalid").is_err());
        assert!(RoutingConfigLoader::validate_model_spec("invalid:model:extra").is_err());
    }

    #[test]
    fn test_parse_model_spec() {
        let config = RoutingConfigLoader::parse_model_spec("claude:sonnet-4.5").unwrap();
        assert_eq!(config.model_type, ModelType::Claude);
        assert_eq!(config.model_id, "sonnet-4.5");
    }

    #[test]
    fn test_build_fallback_chains() {
        let config = RoutingConfig {
            default_strategy: "complexity_based".to_string(),
            threshold: None,
            chains: vec![FallbackChainConfig {
                name: "test".to_string(),
                models: vec![
                    "claude:sonnet-4.5".to_string(),
                    "openai:gpt-4".to_string(),
                ],
            }],
            rules: Vec::new(),
        };
        
        let chains = RoutingConfigLoader::build_fallback_chains(&config).unwrap();
        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].0, "test");
        assert_eq!(chains[0].1.len(), 2);
    }
}

