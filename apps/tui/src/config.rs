//! TUI configuration management.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// TUI configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuiConfig {
    /// Theme configuration
    pub theme: ThemeConfig,
    /// Performance configuration
    #[serde(default)]
    pub performance: PerformanceConfig,
    /// Animation configuration
    #[serde(default)]
    pub animations: AnimationConfig,
}

/// Performance configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Maximum conversation history to keep in memory (default: 500)
    #[serde(default = "default_max_conversation_history")]
    pub max_conversation_history: usize,
}

fn default_max_conversation_history() -> usize {
    500
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_conversation_history: 500,
        }
    }
}

/// Animation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationConfig {
    /// Whether animations are enabled (default: true)
    #[serde(default = "default_animations_enabled")]
    pub enabled: bool,
    /// Animation duration multiplier (default: 1.0)
    #[serde(default = "default_duration_multiplier")]
    pub duration_multiplier: f64,
    /// Whether to use reduced motion (default: false)
    #[serde(default = "default_reduced_motion")]
    pub reduced_motion: bool,
}

fn default_animations_enabled() -> bool {
    true
}

fn default_duration_multiplier() -> f64 {
    1.0
}

fn default_reduced_motion() -> bool {
    false
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            duration_multiplier: 1.0,
            reduced_motion: false,
        }
    }
}

/// Theme configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Theme preset: "dark", "light", or "custom"
    #[serde(default = "default_preset")]
    pub preset: String,
    /// Custom colors (only used if preset = "custom")
    #[serde(default)]
    pub colors: Option<CustomColors>,
}

/// Custom color configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomColors {
    pub primary: Option<String>,
    pub secondary: Option<String>,
    pub success: Option<String>,
    pub warning: Option<String>,
    pub error: Option<String>,
    pub info: Option<String>,
    pub text: Option<String>,
    pub text_muted: Option<String>,
    pub text_dim: Option<String>,
    pub bg_primary: Option<String>,
    pub bg_panel: Option<String>,
    pub bg_element: Option<String>,
    pub border: Option<String>,
    pub border_active: Option<String>,
    pub border_subtle: Option<String>,
}

fn default_preset() -> String {
    "dark".to_string()
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            preset: "dark".to_string(),
            colors: None,
        }
    }
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            theme: ThemeConfig::default(),
            performance: PerformanceConfig::default(),
            animations: AnimationConfig::default(),
        }
    }
}

impl TuiConfig {
    /// Get the config file path.
    pub fn config_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
        Ok(home.join(".radium").join("config.toml"))
    }

    /// Load configuration from file, or return default if file doesn't exist.
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            // Generate default config file
            let default_config = Self::default();
            default_config.save()?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

        let config: TuiConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", config_path.display()))?;

        Ok(config)
    }

    /// Save configuration to file.
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
        }

        // Generate TOML with comments
        let mut toml = String::new();
        toml.push_str("# Radium TUI Configuration\n");
        toml.push_str("# This file allows you to customize the TUI appearance\n\n");
        toml.push_str("[theme]\n");
        toml.push_str("# Theme preset: \"dark\" (default), \"light\", or \"custom\"\n");
        toml.push_str(&format!("preset = \"{}\"\n\n", self.theme.preset));

        toml.push_str("[performance]\n");
        toml.push_str("# Maximum conversation history to keep in memory (default: 500)\n");
        toml.push_str(&format!("max_conversation_history = {}\n\n", self.performance.max_conversation_history));

        toml.push_str("[animations]\n");
        toml.push_str("# Whether animations are enabled (default: true)\n");
        toml.push_str(&format!("enabled = {}\n", self.animations.enabled));
        toml.push_str("# Animation duration multiplier (default: 1.0)\n");
        toml.push_str(&format!("duration_multiplier = {}\n", self.animations.duration_multiplier));
        toml.push_str("# Whether to use reduced motion (default: false)\n");
        toml.push_str(&format!("reduced_motion = {}\n\n", self.animations.reduced_motion));

        if let Some(ref colors) = self.theme.colors {
            toml.push_str("# Custom colors (only used if preset = \"custom\")\n");
            toml.push_str("# Colors should be in hex format: \"#RRGGBB\"\n");
            toml.push_str("[theme.colors]\n");

            if let Some(ref c) = colors.primary {
                toml.push_str(&format!("primary = \"{}\"\n", c));
            }
            if let Some(ref c) = colors.secondary {
                toml.push_str(&format!("secondary = \"{}\"\n", c));
            }
            if let Some(ref c) = colors.success {
                toml.push_str(&format!("success = \"{}\"\n", c));
            }
            if let Some(ref c) = colors.warning {
                toml.push_str(&format!("warning = \"{}\"\n", c));
            }
            if let Some(ref c) = colors.error {
                toml.push_str(&format!("error = \"{}\"\n", c));
            }
            if let Some(ref c) = colors.info {
                toml.push_str(&format!("info = \"{}\"\n", c));
            }
            if let Some(ref c) = colors.text {
                toml.push_str(&format!("text = \"{}\"\n", c));
            }
            if let Some(ref c) = colors.text_muted {
                toml.push_str(&format!("text_muted = \"{}\"\n", c));
            }
            if let Some(ref c) = colors.text_dim {
                toml.push_str(&format!("text_dim = \"{}\"\n", c));
            }
            if let Some(ref c) = colors.bg_primary {
                toml.push_str(&format!("bg_primary = \"{}\"\n", c));
            }
            if let Some(ref c) = colors.bg_panel {
                toml.push_str(&format!("bg_panel = \"{}\"\n", c));
            }
            if let Some(ref c) = colors.bg_element {
                toml.push_str(&format!("bg_element = \"{}\"\n", c));
            }
            if let Some(ref c) = colors.border {
                toml.push_str(&format!("border = \"{}\"\n", c));
            }
            if let Some(ref c) = colors.border_active {
                toml.push_str(&format!("border_active = \"{}\"\n", c));
            }
            if let Some(ref c) = colors.border_subtle {
                toml.push_str(&format!("border_subtle = \"{}\"\n", c));
            }
        }

        fs::write(&config_path, toml)
            .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;

        Ok(())
    }

    /// Reload configuration from file.
    pub fn reload() -> Result<Self> {
        Self::load()
    }
}

