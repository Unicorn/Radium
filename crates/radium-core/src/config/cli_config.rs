//! CLI configuration file support.
//!
//! Provides configuration structure and loading for CLI settings.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// CLI configuration structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    /// Default engine to use
    #[serde(default)]
    pub engine: Option<String>,

    /// Default model to use
    #[serde(default)]
    pub model: Option<String>,

    /// Default workspace path
    #[serde(default)]
    pub workspace: Option<String>,

    /// Command aliases
    #[serde(default)]
    pub aliases: std::collections::HashMap<String, String>,

    /// Output format preferences
    #[serde(default)]
    pub output: OutputConfig,

    /// Log level
    #[serde(default)]
    pub log_level: Option<String>,
}

/// Output format configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Default output format (human, json)
    #[serde(default = "default_output_format")]
    pub format: String,

    /// Always use JSON output
    #[serde(default)]
    pub always_json: bool,
}

fn default_output_format() -> String {
    "human".to_string()
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: "human".to_string(),
            always_json: false,
        }
    }
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            engine: None,
            model: None,
            workspace: None,
            aliases: std::collections::HashMap::new(),
            output: OutputConfig::default(),
            log_level: None,
        }
    }
}

/// Errors that can occur during configuration loading.
#[derive(Debug, Error)]
pub enum CliConfigError {
    /// Configuration file not found.
    #[error("Configuration file not found: {0}")]
    NotFound(String),

    /// Failed to read configuration file.
    #[error("Failed to read configuration file: {0}")]
    ReadError(String),

    /// Failed to parse configuration file.
    #[error("Failed to parse configuration file: {0}")]
    ParseError(String),

    /// Invalid configuration value.
    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),
}

/// Result type for configuration operations.
pub type CliConfigResult<T> = std::result::Result<T, CliConfigError>;

impl CliConfig {
    /// Load configuration from a TOML file.
    pub fn load_from_file(path: &Path) -> CliConfigResult<Self> {
        if !path.exists() {
            return Err(CliConfigError::NotFound(path.display().to_string()));
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| CliConfigError::ReadError(format!("{}: {}", path.display(), e)))?;

        toml::from_str(&content)
            .map_err(|e| CliConfigError::ParseError(format!("{}: {}", path.display(), e)))
    }

    /// Save configuration to a TOML file.
    pub fn save_to_file(&self, path: &Path) -> CliConfigResult<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| CliConfigError::ParseError(format!("Failed to serialize: {}", e)))?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| CliConfigError::ReadError(format!("Failed to create directory: {}", e)))?;
        }

        std::fs::write(path, content)
            .map_err(|e| CliConfigError::ReadError(format!("Failed to write file: {}", e)))?;

        Ok(())
    }

    /// Get default global configuration file path.
    pub fn default_global_path() -> PathBuf {
        std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".radium")
            .join("config.toml")
    }

    /// Get default local configuration file path.
    pub fn default_local_path() -> PathBuf {
        PathBuf::from(".radiumrc")
    }

    /// Discover and load configuration files.
    ///
    /// Loads configuration from:
    /// 1. Global config (~/.radium/config.toml)
    /// 2. Local config (./.radiumrc)
    ///
    /// Local config overrides global config.
    pub fn discover_and_load() -> Self {
        let mut config = Self::default();

        // Load global config
        let global_path = Self::default_global_path();
        if let Ok(global_config) = Self::load_from_file(&global_path) {
            config.merge(&global_config);
        }

        // Load local config (overrides global)
        let local_path = Self::default_local_path();
        if let Ok(local_config) = Self::load_from_file(&local_path) {
            config.merge(&local_config);
        }

        config
    }

    /// Merge another configuration into this one.
    ///
    /// Values from `other` override values in `self` if they are Some.
    pub fn merge(&mut self, other: &Self) {
        if let Some(ref engine) = other.engine {
            self.engine = Some(engine.clone());
        }
        if let Some(ref model) = other.model {
            self.model = Some(model.clone());
        }
        if let Some(ref workspace) = other.workspace {
            self.workspace = Some(workspace.clone());
        }
        if let Some(ref log_level) = other.log_level {
            self.log_level = Some(log_level.clone());
        }
        self.aliases.extend(other.aliases.clone());
        if other.output.always_json {
            self.output.always_json = true;
        }
        if other.output.format != "human" {
            self.output.format = other.output.format.clone();
        }
    }

    /// Apply configuration to environment variables if not already set.
    ///
    /// # Safety
    ///
    /// This function modifies environment variables. It should only be called
    /// from single-threaded code before spawning threads.
    pub unsafe fn apply_to_env(&self) {
        if let Some(ref engine) = self.engine {
            if std::env::var("RADIUM_ENGINE").is_err() {
                std::env::set_var("RADIUM_ENGINE", engine);
            }
        }
        if let Some(ref model) = self.model {
            if std::env::var("RADIUM_MODEL").is_err() {
                std::env::set_var("RADIUM_MODEL", model);
            }
        }
        if let Some(ref workspace) = self.workspace {
            if std::env::var("RADIUM_WORKSPACE").is_err() {
                std::env::set_var("RADIUM_WORKSPACE", workspace);
            }
        }
        if let Some(ref log_level) = self.log_level {
            if std::env::var("RUST_LOG").is_err() {
                std::env::set_var("RUST_LOG", log_level);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let config_content = r#"
engine = "claude"
model = "claude-3-opus"
workspace = "/path/to/workspace"
log_level = "debug"

[output]
format = "json"
always_json = true

[aliases]
c = "craft"
p = "plan"
"#;

        std::fs::write(&config_path, config_content).unwrap();

        let config = CliConfig::load_from_file(&config_path).unwrap();
        assert_eq!(config.engine, Some("claude".to_string()));
        assert_eq!(config.model, Some("claude-3-opus".to_string()));
        assert_eq!(config.workspace, Some("/path/to/workspace".to_string()));
        assert_eq!(config.log_level, Some("debug".to_string()));
        assert_eq!(config.output.format, "json");
        assert!(config.output.always_json);
        assert_eq!(config.aliases.get("c"), Some(&"craft".to_string()));
    }

    #[test]
    fn test_merge() {
        let mut config1 = CliConfig {
            engine: Some("mock".to_string()),
            model: None,
            ..Default::default()
        };

        let config2 = CliConfig {
            engine: Some("claude".to_string()),
            model: Some("claude-3-opus".to_string()),
            ..Default::default()
        };

        config1.merge(&config2);
        assert_eq!(config1.engine, Some("claude".to_string()));
        assert_eq!(config1.model, Some("claude-3-opus".to_string()));
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let mut config = CliConfig {
            engine: Some("claude".to_string()),
            model: Some("claude-3-opus".to_string()),
            ..Default::default()
        };

        config.save_to_file(&config_path).unwrap();
        let loaded = CliConfig::load_from_file(&config_path).unwrap();

        assert_eq!(loaded.engine, config.engine);
        assert_eq!(loaded.model, config.model);
    }
}

