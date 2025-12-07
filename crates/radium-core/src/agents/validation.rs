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

    #[test]
    fn test_config_validator_id_starts_with_hyphen() {
        let validator = ConfigValidator;
        let mut config = create_test_config();
        config.id = "-invalid-id".to_string();
        let result = validator.validate(&config);
        assert!(matches!(result, Err(ValidationError::InvalidValue { .. })));
        if let Err(ValidationError::InvalidValue { field, reason }) = result {
            assert_eq!(field, "id");
            assert!(reason.contains("hyphen"));
        }
    }

    #[test]
    fn test_config_validator_id_ends_with_hyphen() {
        let validator = ConfigValidator;
        let mut config = create_test_config();
        config.id = "invalid-id-".to_string();
        let result = validator.validate(&config);
        assert!(matches!(result, Err(ValidationError::InvalidValue { .. })));
    }

    #[test]
    fn test_config_validator_id_with_uppercase() {
        let validator = ConfigValidator;
        let mut config = create_test_config();
        config.id = "Invalid-Id".to_string();
        assert!(matches!(
            validator.validate(&config),
            Err(ValidationError::InvalidValue { .. })
        ));
    }

    #[test]
    fn test_config_validator_id_with_special_chars() {
        let validator = ConfigValidator;
        let mut config = create_test_config();
        config.id = "invalid_id".to_string(); // underscore not allowed
        assert!(matches!(
            validator.validate(&config),
            Err(ValidationError::InvalidValue { .. })
        ));
    }

    #[test]
    fn test_config_validator_id_with_spaces() {
        let validator = ConfigValidator;
        let mut config = create_test_config();
        config.id = "invalid id".to_string();
        assert!(matches!(
            validator.validate(&config),
            Err(ValidationError::InvalidValue { .. })
        ));
    }

    #[test]
    fn test_config_validator_valid_id_with_numbers() {
        let validator = ConfigValidator;
        let mut config = create_test_config();
        config.id = "agent-123".to_string();
        assert!(validator.validate(&config).is_ok());
    }

    #[test]
    fn test_config_validator_valid_id_multiple_hyphens() {
        let validator = ConfigValidator;
        let mut config = create_test_config();
        config.id = "my-test-agent-123".to_string();
        assert!(validator.validate(&config).is_ok());
    }

    #[test]
    fn test_config_validator_empty_name() {
        let validator = ConfigValidator;
        let mut config = create_test_config();
        config.name = String::new();
        assert!(matches!(
            validator.validate(&config),
            Err(ValidationError::MissingField(_))
        ));
    }

    #[test]
    fn test_config_validator_empty_prompt_path() {
        let validator = ConfigValidator;
        let mut config = create_test_config();
        config.prompt_path = PathBuf::new();
        assert!(matches!(
            validator.validate(&config),
            Err(ValidationError::MissingField(_))
        ));
    }

    #[test]
    fn test_prompt_validator_new() {
        let validator = PromptValidator::new();
        assert!(validator.base_dir.is_none());
    }

    #[test]
    fn test_prompt_validator_with_base_dir() {
        let base = PathBuf::from("/test/base");
        let validator = PromptValidator::with_base_dir(&base);
        assert_eq!(validator.base_dir, Some(base));
    }

    #[test]
    fn test_prompt_validator_nonexistent_file() {
        let validator = PromptValidator::new();
        let path = PathBuf::from("/nonexistent/file.md");
        let result = validator.validate_prompt_path(&path, None);
        assert!(matches!(result, Err(ValidationError::PromptFile(_))));
    }

    #[test]
    fn test_prompt_validator_absolute_path() {
        use tempfile::TempDir;
        use std::fs;
        
        let temp = TempDir::new().unwrap();
        let prompt_file = temp.path().join("prompt.md");
        fs::write(&prompt_file, "# Test Prompt").unwrap();

        let validator = PromptValidator::new();
        let result = validator.validate_prompt_path(&prompt_file, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_prompt_validator_relative_path_with_config_dir() {
        use tempfile::TempDir;
        use std::fs;
        
        let temp = TempDir::new().unwrap();
        let config_dir = temp.path().join("agents");
        fs::create_dir_all(&config_dir).unwrap();
        let prompt_file = config_dir.join("prompt.md");
        fs::write(&prompt_file, "# Test Prompt").unwrap();

        let validator = PromptValidator::new();
        let config_path = config_dir.join("agent.toml");
        let result = validator.validate_prompt_path(Path::new("prompt.md"), Some(&config_path));
        assert!(result.is_ok());
    }

    #[test]
    fn test_prompt_validator_relative_path_with_base_dir() {
        use tempfile::TempDir;
        use std::fs;
        
        let temp = TempDir::new().unwrap();
        let base_dir = temp.path();
        let prompt_file = base_dir.join("prompt.md");
        fs::write(&prompt_file, "# Test Prompt").unwrap();

        let validator = PromptValidator::with_base_dir(base_dir);
        let result = validator.validate_prompt_path(Path::new("prompt.md"), None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_prompt_validator_path_is_directory() {
        use tempfile::TempDir;
        
        let temp = TempDir::new().unwrap();
        let dir_path = temp.path();

        let validator = PromptValidator::new();
        let result = validator.validate_prompt_path(dir_path, None);
        assert!(matches!(result, Err(ValidationError::PromptFile(_))));
        if let Err(ValidationError::PromptFile(msg)) = result {
            assert!(msg.contains("not a file"));
        }
    }

    #[test]
    fn test_prompt_validator_unreadable_file() {
        use tempfile::TempDir;
        use std::fs;
        #[cfg(unix)]
        use std::os::unix::fs::PermissionsExt;
        
        let temp = TempDir::new().unwrap();
        let prompt_file = temp.path().join("prompt.md");
        fs::write(&prompt_file, "# Test").unwrap();
        
        #[cfg(unix)]
        {
            let mut perms = fs::metadata(&prompt_file).unwrap().permissions();
            perms.set_mode(0o000);
            fs::set_permissions(&prompt_file, perms).unwrap();
        }

        let validator = PromptValidator::new();
        let result = validator.validate_prompt_path(&prompt_file, None);
        // On Unix, this should fail with I/O error; on Windows, it might succeed
        #[cfg(unix)]
        assert!(result.is_err());
    }

    #[test]
    fn test_agent_validator_impl_new() {
        let validator = AgentValidatorImpl::new();
        assert!(validator.prompt_validator.base_dir.is_none());
    }

    #[test]
    fn test_agent_validator_impl_with_base_dir() {
        let base = PathBuf::from("/test/base");
        let validator = AgentValidatorImpl::with_base_dir(&base);
        assert_eq!(validator.prompt_validator.base_dir, Some(base));
    }

    #[test]
    fn test_agent_validator_impl_validate_config_only() {
        use tempfile::TempDir;
        use std::fs;
        
        let temp = TempDir::new().unwrap();
        let prompt_file = temp.path().join("prompt.md");
        fs::write(&prompt_file, "# Test").unwrap();

        let mut config = create_test_config();
        config.prompt_path = prompt_file.clone();

        let validator = AgentValidatorImpl::new();
        let config_path = temp.path().join("agent.toml");
        let result = validator.validate(&config, Some(&config_path));
        assert!(result.is_ok());
    }

    #[test]
    fn test_agent_validator_impl_validate_invalid_config() {
        let mut config = create_test_config();
        config.id = String::new();

        let validator = AgentValidatorImpl::new();
        let result = validator.validate(&config, None);
        assert!(matches!(result, Err(ValidationError::MissingField(_))));
    }

    #[test]
    fn test_agent_validator_impl_validate_missing_prompt() {
        let config = create_test_config();
        let validator = AgentValidatorImpl::new();
        let result = validator.validate(&config, None);
        assert!(matches!(result, Err(ValidationError::PromptFile(_))));
    }

    #[test]
    fn test_validation_error_display() {
        let error = ValidationError::MissingField("test".to_string());
        let msg = format!("{}", error);
        assert!(msg.contains("missing required field"));
        assert!(msg.contains("test"));
    }

    #[test]
    fn test_validation_error_invalid_value() {
        let error = ValidationError::InvalidValue {
            field: "id".to_string(),
            reason: "test reason".to_string(),
        };
        let msg = format!("{}", error);
        assert!(msg.contains("invalid field value"));
        assert!(msg.contains("id"));
        assert!(msg.contains("test reason"));
    }

    #[test]
    fn test_validation_error_from_config_error() {
        let config_error = AgentConfigError::Invalid("test".to_string());
        let validation_error: ValidationError = config_error.into();
        assert!(matches!(validation_error, ValidationError::Config(_)));
    }

    #[test]
    fn test_validation_error_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let validation_error: ValidationError = io_error.into();
        assert!(matches!(validation_error, ValidationError::Io(_)));
    }
}

