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
