//! Hook configuration.

use crate::hooks::error::{HookError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Hook configuration from TOML file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConfig {
    /// List of hooks.
    pub hooks: Vec<HookDefinition>,
    /// Whether to enable performance profiling (default: false).
    #[serde(default)]
    pub enable_profiling: bool,
}

/// Definition of a single hook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookDefinition {
    /// Name of the hook.
    pub name: String,
    /// Type of hook.
    #[serde(rename = "type")]
    pub hook_type: String,
    /// Priority of the hook.
    pub priority: Option<u32>,
    /// Whether the hook is enabled.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Script path for the hook (if using external script).
    pub script: Option<String>,
    /// Inline hook configuration.
    pub config: Option<toml::Value>,
}

fn default_enabled() -> bool {
    true
}

impl HookConfig {
    /// Load hook configuration from a TOML file.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| HookError::Io(e))?;
        Self::from_str(&content)
    }

    /// Parse hook configuration from a TOML string.
    pub fn from_str(content: &str) -> Result<Self> {
        toml::from_str(content).map_err(|e| HookError::ConfigParse(e))
    }

    /// Save hook configuration to a TOML file.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| HookError::InvalidConfig(format!("Failed to serialize config: {}", e)))?;
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent).map_err(|e| HookError::Io(e))?;
        }
        
        std::fs::write(path, content).map_err(|e| HookError::Io(e))?;
        Ok(())
    }

    /// Update the enabled state of a hook by name.
    pub fn set_hook_enabled(&mut self, name: &str, enabled: bool) -> Result<()> {
        let hook = self.hooks.iter_mut()
            .find(|h| h.name == name)
            .ok_or_else(|| HookError::NotFound(name.to_string()))?;
        hook.enabled = enabled;
        Ok(())
    }

    /// Get the enabled state of a hook by name.
    pub fn is_hook_enabled(&self, name: &str) -> Option<bool> {
        self.hooks.iter()
            .find(|h| h.name == name)
            .map(|h| h.enabled)
    }

    /// Validate the hook configuration.
    pub fn validate(&self) -> Result<()> {
        for hook in &self.hooks {
            if hook.name.is_empty() {
                return Err(HookError::InvalidConfig("Hook name cannot be empty".to_string()));
            }

            if hook.hook_type.is_empty() {
                return Err(HookError::InvalidConfig("Hook type cannot be empty".to_string()));
            }

            // Validate hook type
            let valid_types = [
                "before_model",
                "after_model",
                "before_tool",
                "after_tool",
                "tool_selection",
                "error_interception",
                "error_transformation",
                "error_recovery",
                "error_logging",
                "telemetry_collection",
                "custom_logging",
                "metrics_aggregation",
                "performance_monitoring",
            ];

            if !valid_types.contains(&hook.hook_type.as_str()) {
                return Err(HookError::InvalidConfig(format!(
                    "Invalid hook type: {}",
                    hook.hook_type
                )));
            }

            // Either script or config must be provided
            if hook.script.is_none() && hook.config.is_none() {
                return Err(HookError::InvalidConfig(
                    "Either script or config must be provided".to_string(),
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_hook_config_from_str_valid() {
        let content = r#"
            enable_profiling = true
            [[hooks]]
            name = "test-hook"
            type = "before_model"
            priority = 100
            enabled = true
        "#;
        
        let config = HookConfig::from_str(content);
        assert!(config.is_ok());
        let config = config.unwrap();
        assert!(config.enable_profiling);
        assert_eq!(config.hooks.len(), 1);
        assert_eq!(config.hooks[0].name, "test-hook");
    }

    #[test]
    fn test_hook_config_from_str_invalid_toml() {
        let content = "invalid toml {";
        let result = HookConfig::from_str(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_hook_config_from_file() {
        let temp = TempDir::new().unwrap();
        let config_file = temp.path().join("hooks.toml");
        
        let content = r#"
            [[hooks]]
            name = "test-hook"
            type = "before_model"
        "#;
        std::fs::write(&config_file, content).unwrap();
        
        let config = HookConfig::from_file(&config_file);
        assert!(config.is_ok());
    }

    #[test]
    fn test_hook_config_from_file_nonexistent() {
        let result = HookConfig::from_file("/nonexistent/path/hooks.toml");
        assert!(result.is_err());
    }

    #[test]
    fn test_hook_config_save() {
        let temp = TempDir::new().unwrap();
        let config_file = temp.path().join("hooks.toml");
        
        let mut config = HookConfig {
            hooks: vec![HookDefinition {
                name: "test-hook".to_string(),
                hook_type: "before_model".to_string(),
                priority: Some(100),
                enabled: true,
                script: None,
                config: None,
            }],
            enable_profiling: false,
        };
        
        let result = config.save(&config_file);
        assert!(result.is_ok());
        assert!(config_file.exists());
    }

    #[test]
    fn test_hook_config_save_creates_parent_dir() {
        let temp = TempDir::new().unwrap();
        let config_file = temp.path().join("subdir").join("hooks.toml");
        
        let config = HookConfig {
            hooks: vec![],
            enable_profiling: false,
        };
        
        let result = config.save(&config_file);
        assert!(result.is_ok());
        assert!(config_file.exists());
    }

    #[test]
    fn test_hook_config_set_hook_enabled() {
        let mut config = HookConfig {
            hooks: vec![HookDefinition {
                name: "test-hook".to_string(),
                hook_type: "before_model".to_string(),
                priority: None,
                enabled: true,
                script: None,
                config: None,
            }],
            enable_profiling: false,
        };
        
        let result = config.set_hook_enabled("test-hook", false);
        assert!(result.is_ok());
        assert!(!config.hooks[0].enabled);
    }

    #[test]
    fn test_hook_config_set_hook_enabled_not_found() {
        let mut config = HookConfig {
            hooks: vec![],
            enable_profiling: false,
        };
        
        let result = config.set_hook_enabled("nonexistent", true);
        assert!(result.is_err());
    }

    #[test]
    fn test_hook_config_is_hook_enabled() {
        let config = HookConfig {
            hooks: vec![HookDefinition {
                name: "test-hook".to_string(),
                hook_type: "before_model".to_string(),
                priority: None,
                enabled: true,
                script: None,
                config: None,
            }],
            enable_profiling: false,
        };
        
        assert_eq!(config.is_hook_enabled("test-hook"), Some(true));
        assert_eq!(config.is_hook_enabled("nonexistent"), None);
    }

    #[test]
    fn test_hook_config_validate_empty_name() {
        let config = HookConfig {
            hooks: vec![HookDefinition {
                name: String::new(),
                hook_type: "before_model".to_string(),
                priority: None,
                enabled: true,
                script: None,
                config: None,
            }],
            enable_profiling: false,
        };
        
        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_hook_config_validate_empty_type() {
        let config = HookConfig {
            hooks: vec![HookDefinition {
                name: "test-hook".to_string(),
                hook_type: String::new(),
                priority: None,
                enabled: true,
                script: None,
                config: None,
            }],
            enable_profiling: false,
        };
        
        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_hook_config_validate_invalid_type() {
        let config = HookConfig {
            hooks: vec![HookDefinition {
                name: "test-hook".to_string(),
                hook_type: "invalid_type".to_string(),
                priority: None,
                enabled: true,
                script: Some("script.sh".to_string()),
                config: None,
            }],
            enable_profiling: false,
        };
        
        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_hook_config_validate_no_script_or_config() {
        let config = HookConfig {
            hooks: vec![HookDefinition {
                name: "test-hook".to_string(),
                hook_type: "before_model".to_string(),
                priority: None,
                enabled: true,
                script: None,
                config: None,
            }],
            enable_profiling: false,
        };
        
        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_hook_config_validate_with_script() {
        let config = HookConfig {
            hooks: vec![HookDefinition {
                name: "test-hook".to_string(),
                hook_type: "before_model".to_string(),
                priority: None,
                enabled: true,
                script: Some("script.sh".to_string()),
                config: None,
            }],
            enable_profiling: false,
        };
        
        let result = config.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_hook_config_validate_with_config() {
        let config = HookConfig {
            hooks: vec![HookDefinition {
                name: "test-hook".to_string(),
                hook_type: "before_model".to_string(),
                priority: None,
                enabled: true,
                script: None,
                config: Some(toml::Value::String("config".to_string())),
            }],
            enable_profiling: false,
        };
        
        let result = config.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_hook_config_validate_all_valid_types() {
        let valid_types = [
            "before_model", "after_model", "before_tool", "after_tool",
            "tool_selection", "error_interception", "error_transformation",
            "error_recovery", "error_logging", "telemetry_collection",
            "custom_logging", "metrics_aggregation", "performance_monitoring",
        ];
        
        for hook_type in &valid_types {
            let config = HookConfig {
                hooks: vec![HookDefinition {
                    name: "test-hook".to_string(),
                    hook_type: hook_type.to_string(),
                    priority: None,
                    enabled: true,
                    script: Some("script.sh".to_string()),
                    config: None,
                }],
                enable_profiling: false,
            };
            
            let result = config.validate();
            assert!(result.is_ok(), "Hook type {} should be valid", hook_type);
        }
    }

    #[test]
    fn test_hook_definition_default_enabled() {
        let hook = HookDefinition {
            name: "test".to_string(),
            hook_type: "before_model".to_string(),
            priority: None,
            enabled: default_enabled(),
            script: None,
            config: None,
        };
        assert!(hook.enabled);
    }

    #[test]
    fn test_hook_config_enable_profiling_default() {
        let content = r#"
            [[hooks]]
            name = "test-hook"
            type = "before_model"
            script = "script.sh"
        "#;
        
        let config = HookConfig::from_str(content).unwrap();
        assert!(!config.enable_profiling); // Default is false
    }
}
