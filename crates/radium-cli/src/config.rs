use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Config file not found at {0}. Run 'radium-workflow login' first.")]
    NotFound(PathBuf),

    #[error("Profile '{0}' not found in config. Available profiles: {1}")]
    ProfileNotFound(String, String),

    #[error("Failed to read config: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("Failed to parse config: {0}")]
    ParseError(#[from] toml::de::Error),

    #[error("Failed to serialize config: {0}")]
    SerializeError(#[from] toml::ser::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Profile {
    pub api_url: String,
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Config {
    #[serde(flatten)]
    pub profiles: HashMap<String, Profile>,
}

impl Config {
    /// Returns the path to the config file: `~/.radium/config.toml`
    pub fn path() -> Result<PathBuf, ConfigError> {
        let home = dirs::home_dir().ok_or_else(|| {
            ConfigError::ReadError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not determine home directory",
            ))
        })?;
        Ok(home.join(".radium").join("config.toml"))
    }

    /// Load config from `~/.radium/config.toml`.
    pub fn load() -> Result<Self, ConfigError> {
        let path = Self::path()?;
        if !path.exists() {
            return Err(ConfigError::NotFound(path));
        }
        let content = fs::read_to_string(&path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Load config from a specific path (useful for testing).
    pub fn load_from(path: &PathBuf) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Err(ConfigError::NotFound(path.clone()));
        }
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save config to `~/.radium/config.toml`, creating the directory if needed.
    pub fn save(&self) -> Result<(), ConfigError> {
        let path = Self::path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// Save config to a specific path (useful for testing).
    pub fn save_to(&self, path: &PathBuf) -> Result<(), ConfigError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Get a profile by name, returning an error if not found.
    pub fn get_profile(&self, name: &str) -> Result<&Profile, ConfigError> {
        self.profiles.get(name).ok_or_else(|| {
            let available = self
                .profiles
                .keys()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            ConfigError::ProfileNotFound(name.to_string(), available)
        })
    }

    /// Set a profile (insert or update).
    pub fn set_profile(&mut self, name: String, profile: Profile) {
        self.profiles.insert(name, profile);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn temp_config_path(dir: &TempDir) -> PathBuf {
        dir.path().join("config.toml")
    }

    #[test]
    fn config_round_trip() {
        let dir = TempDir::new().unwrap();
        let path = temp_config_path(&dir);

        let mut config = Config::default();
        config.set_profile(
            "default".to_string(),
            Profile {
                api_url: "https://radium.example.com".to_string(),
                api_key: "rk_test_key".to_string(),
            },
        );
        config.set_profile(
            "staging".to_string(),
            Profile {
                api_url: "https://staging.radium.example.com".to_string(),
                api_key: "rk_staging_key".to_string(),
            },
        );

        config.save_to(&path).unwrap();
        let loaded = Config::load_from(&path).unwrap();

        assert_eq!(config, loaded);
        assert_eq!(loaded.profiles.len(), 2);

        let default_profile = loaded.get_profile("default").unwrap();
        assert_eq!(default_profile.api_url, "https://radium.example.com");
        assert_eq!(default_profile.api_key, "rk_test_key");

        let staging_profile = loaded.get_profile("staging").unwrap();
        assert_eq!(
            staging_profile.api_url,
            "https://staging.radium.example.com"
        );
        assert_eq!(staging_profile.api_key, "rk_staging_key");
    }

    #[test]
    fn get_missing_profile_returns_error() {
        let config = Config::default();
        let result = config.get_profile("nonexistent");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("nonexistent"));
    }

    #[test]
    fn load_missing_file_returns_error() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("does_not_exist.toml");
        let result = Config::load_from(&path);
        assert!(result.is_err());
    }

    #[test]
    fn set_profile_overwrites_existing() {
        let mut config = Config::default();
        config.set_profile(
            "default".to_string(),
            Profile {
                api_url: "https://old.example.com".to_string(),
                api_key: "old_key".to_string(),
            },
        );
        config.set_profile(
            "default".to_string(),
            Profile {
                api_url: "https://new.example.com".to_string(),
                api_key: "new_key".to_string(),
            },
        );

        let profile = config.get_profile("default").unwrap();
        assert_eq!(profile.api_url, "https://new.example.com");
        assert_eq!(profile.api_key, "new_key");
    }
}
