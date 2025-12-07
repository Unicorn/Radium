//! Agent validation system.
//!
//! Provides comprehensive validation for agent configurations and prompt files.

use crate::agents::config::{AgentConfig, AgentConfigError};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Validation errors.
#[derive(Debug, Error)]
pub enum ValidationError {
    /// Configuration validation error.
    #[error("configuration error: {0}")]
    Config(#[from] AgentConfigError),

    /// Prompt file error.
    #[error("prompt file error: {0}")]
    PromptFile(String),

    /// Missing required field.
    #[error("missing required field: {0}")]
    MissingField(String),

    /// Invalid field value.
    #[error("invalid field value: {field} - {reason}")]
    InvalidValue { field: String, reason: String },

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for validation operations.
pub type Result<T> = std::result::Result<T, ValidationError>;

/// Agent validator trait.
pub trait AgentValidator {
    /// Validates an agent configuration.
    fn validate(&self, config: &AgentConfig) -> Result<()>;
}

/// Configuration validator.
pub struct ConfigValidator;

impl AgentValidator for ConfigValidator {
    fn validate(&self, config: &AgentConfig) -> Result<()> {
        // Validate required fields
        if config.id.is_empty() {
            return Err(ValidationError::MissingField("id".to_string()));
        }

        // Validate agent ID format (kebab-case)
        if !config.id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
            return Err(ValidationError::InvalidValue {
                field: "id".to_string(),
                reason: "must be in kebab-case (lowercase letters, numbers, hyphens only)".to_string(),
            });
        }

        if config.id.starts_with('-') || config.id.ends_with('-') {
            return Err(ValidationError::InvalidValue {
                field: "id".to_string(),
                reason: "cannot start or end with hyphen".to_string(),
            });
        }

        if config.name.is_empty() {
            return Err(ValidationError::MissingField("name".to_string()));
        }

        if config.prompt_path.as_os_str().is_empty() {
            return Err(ValidationError::MissingField("prompt_path".to_string()));
        }

        Ok(())
    }
}

/// Prompt file validator.
pub struct PromptValidator {
    /// Base directory for resolving relative paths.
    base_dir: Option<PathBuf>,
}

impl PromptValidator {
    /// Creates a new prompt validator.
    pub fn new() -> Self {
        Self { base_dir: None }
    }

    /// Creates a new prompt validator with a base directory.
    pub fn with_base_dir(base_dir: impl AsRef<Path>) -> Self {
        Self {
            base_dir: Some(base_dir.as_ref().to_path_buf()),
        }
    }

    /// Validates that a prompt file exists and is readable.
    pub fn validate_prompt_path(&self, prompt_path: &Path, config_path: Option<&Path>) -> Result<()> {
        let resolved_path = if prompt_path.is_absolute() {
            prompt_path.to_path_buf()
        } else if let Some(config_dir) = config_path.and_then(|p| p.parent()) {
            config_dir.join(prompt_path)
        } else if let Some(ref base) = self.base_dir {
            base.join(prompt_path)
        } else {
            prompt_path.to_path_buf()
        };

        if !resolved_path.exists() {
            return Err(ValidationError::PromptFile(format!(
                "prompt file not found: {}",
                resolved_path.display()
            )));
        }

        if !resolved_path.is_file() {
            return Err(ValidationError::PromptFile(format!(
                "prompt path is not a file: {}",
                resolved_path.display()
            )));
        }

        // Try to read the file to ensure it's readable
        std::fs::read_to_string(&resolved_path)?;

        Ok(())
    }
}

impl Default for PromptValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Comprehensive agent validator that validates both config and prompt.
pub struct AgentValidatorImpl {
    config_validator: ConfigValidator,
    prompt_validator: PromptValidator,
}

impl AgentValidatorImpl {
    /// Creates a new comprehensive validator.
    pub fn new() -> Self {
        Self {
            config_validator: ConfigValidator,
            prompt_validator: PromptValidator::new(),
        }
    }

    /// Creates a new validator with a base directory for prompt resolution.
    pub fn with_base_dir(base_dir: impl AsRef<Path>) -> Self {
        Self {
            config_validator: ConfigValidator,
            prompt_validator: PromptValidator::with_base_dir(base_dir),
        }
    }

    /// Validates an agent configuration and its prompt file.
    pub fn validate(&self, config: &AgentConfig, config_path: Option<&Path>) -> Result<()> {
        // Validate configuration
        self.config_validator.validate(config)?;

        // Validate prompt file
        self.prompt_validator
            .validate_prompt_path(&config.prompt_path, config_path)?;

        Ok(())
    }
}

impl Default for AgentValidatorImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::config::AgentCapabilities;

    fn create_test_config() -> AgentConfig {
        AgentConfig {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            description: "Test description".to_string(),
            prompt_path: PathBuf::from("test.md"),
            mirror_path: None,
            engine: None,
            model: None,
            reasoning_effort: None,
            loop_behavior: None,
            trigger_behavior: None,
            category: None,
            file_path: None,
            capabilities: AgentCapabilities::default(),
            persona_config: None,
            sandbox: None,
        }
    }

    #[test]
    fn test_config_validator_valid() {
        let validator = ConfigValidator;
        let config = create_test_config();
        assert!(validator.validate(&config).is_ok());
    }

    #[test]
    fn test_config_validator_empty_id() {
        let validator = ConfigValidator;
        let mut config = create_test_config();
        config.id = String::new();
        assert!(matches!(
            validator.validate(&config),
            Err(ValidationError::MissingField(_))
        ));
    }

    #[test]
    fn test_config_validator_invalid_id_format() {
        let validator = ConfigValidator;
        let mut config = create_test_config();
        config.id = "Invalid_ID".to_string();
        assert!(matches!(
            validator.validate(&config),
            Err(ValidationError::InvalidValue { .. })
        ));
    }
}

