//! Model cache configuration loading from workspace config.

use radium_models::CacheConfig;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur when loading cache configuration.
#[derive(Debug, Error)]
pub enum CacheConfigError {
    /// I/O error reading config file.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// TOML parsing error.
    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    /// Configuration validation error.
    #[error("Configuration validation error: {0}")]
    Validation(String),
}

/// Load cache configuration from workspace config file.
///
/// Searches for `.radium/config.toml` in the workspace root.
/// If the `[models.cache]` section is missing, returns default configuration.
///
/// # Arguments
/// * `workspace_root` - Root directory of the workspace
///
/// # Returns
/// Cache configuration (default if section is missing)
///
/// # Errors
/// Returns error if config file exists but cannot be read or parsed.
pub fn load_cache_config(workspace_root: &Path) -> Result<CacheConfig, CacheConfigError> {
    let config_path = workspace_root.join(".radium").join("config.toml");

    // If config file doesn't exist, return defaults
    if !config_path.exists() {
        return Ok(CacheConfig::default());
    }

    let content = std::fs::read_to_string(&config_path)?;
    let toml: toml::Table = toml::from_str(&content)?;

    // Try to get [models.cache] section
    if let Some(models) = toml.get("models") {
        if let Some(models_table) = models.as_table() {
            if let Some(cache) = models_table.get("cache") {
                // Convert TOML value to string and parse it
                let cache_str = toml::to_string(cache).map_err(|e| {
                    CacheConfigError::Validation(format!("Failed to serialize cache config: {}", e))
                })?;
                
                // Parse cache config from string
                let cache_config: CacheConfig = toml::from_str(&cache_str)
                    .map_err(CacheConfigError::TomlParse)?;

                // Validate the config
                cache_config.validate().map_err(|e| {
                    CacheConfigError::Validation(format!("Invalid cache configuration: {}", e))
                })?;

                return Ok(cache_config);
            }
        }
    }

    // Section not found, return defaults
    Ok(CacheConfig::default())
}

/// Get the default config file path for a workspace.
///
/// # Arguments
/// * `workspace_root` - Root directory of the workspace
///
/// # Returns
/// Path to `.radium/config.toml`
#[must_use]
pub fn default_config_path(workspace_root: &Path) -> PathBuf {
    workspace_root.join(".radium").join("config.toml")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_cache_config_default_when_missing() {
        let temp = TempDir::new().unwrap();
        let config = load_cache_config(temp.path()).unwrap();
        assert_eq!(config.enabled, true);
        assert_eq!(config.inactivity_timeout_secs, 1800);
    }

    #[test]
    fn test_load_cache_config_from_file() {
        let temp = TempDir::new().unwrap();
        let config_dir = temp.path().join(".radium");
        std::fs::create_dir_all(&config_dir).unwrap();

        let config_content = r#"
[models.cache]
enabled = true
inactivity_timeout_secs = 3600
max_cache_size = 20
cleanup_interval_secs = 600
"#;

        std::fs::write(config_dir.join("config.toml"), config_content).unwrap();

        let config = load_cache_config(temp.path()).unwrap();
        assert_eq!(config.enabled, true);
        assert_eq!(config.inactivity_timeout_secs, 3600);
        assert_eq!(config.max_cache_size, 20);
        assert_eq!(config.cleanup_interval_secs, 600);
    }

    #[test]
    fn test_load_cache_config_default_when_section_missing() {
        let temp = TempDir::new().unwrap();
        let config_dir = temp.path().join(".radium");
        std::fs::create_dir_all(&config_dir).unwrap();

        let config_content = r#"
[other.section]
value = "test"
"#;

        std::fs::write(config_dir.join("config.toml"), config_content).unwrap();

        let config = load_cache_config(temp.path()).unwrap();
        // Should return defaults
        assert_eq!(config.enabled, true);
        assert_eq!(config.inactivity_timeout_secs, 1800);
    }
}

