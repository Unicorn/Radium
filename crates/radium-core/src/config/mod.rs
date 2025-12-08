//! Configuration module for Radium Core.

pub mod cli_config;
pub mod routing;

use std::net::SocketAddr;

use serde::Deserialize;

/// Server configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// The address to bind the gRPC server to.
    #[serde(default = "default_address")]
    pub address: SocketAddr,
    /// The address to bind the gRPC-Web server to (optional).
    #[serde(default)]
    pub web_address: Option<SocketAddr>,
    /// Enable gRPC-Web support.
    #[serde(default = "default_true")]
    pub enable_grpc_web: bool,
}

fn default_true() -> bool {
    true
}

fn default_address() -> SocketAddr {
    // This is a compile-time constant, so unwrap is safe
    "127.0.0.1:50051".parse().expect("valid default address")
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self { address: default_address(), web_address: None, enable_grpc_web: true }
    }
}

/// Model configuration section in config file.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ModelConfigSection {
    /// The type of model (mock, gemini, openai).
    #[serde(default = "default_model_type")]
    pub model_type: String,
    /// The model ID (e.g., "gemini-pro", "gpt-4").
    pub model_id: String,
    /// Optional API key override (if not provided, will be loaded from environment).
    #[serde(default)]
    pub api_key: Option<String>,
}

fn default_model_type() -> String {
    "mock".to_string()
}

/// Checkpoint configuration section in config file.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct CheckpointConfig {
    /// Enable automatic checkpoint creation during workflow execution and file operations.
    #[serde(default = "default_auto_create")]
    pub auto_create: bool,
    /// Number of days to retain checkpoints before cleanup.
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,
    /// Maximum number of checkpoints to keep.
    #[serde(default = "default_max_checkpoints")]
    pub max_checkpoints: usize,
    /// Maximum size of checkpoint repository in GB.
    #[serde(default = "default_max_size_gb")]
    pub max_size_gb: u64,
}

fn default_auto_create() -> bool {
    true
}

fn default_retention_days() -> u32 {
    7
}

fn default_max_checkpoints() -> usize {
    50
}

fn default_max_size_gb() -> u64 {
    5
}

impl Default for CheckpointConfig {
    fn default() -> Self {
        Self {
            auto_create: default_auto_create(),
            retention_days: default_retention_days(),
            max_checkpoints: default_max_checkpoints(),
            max_size_gb: default_max_size_gb(),
        }
    }
}

impl CheckpointConfig {
    /// Validates the checkpoint configuration.
    ///
    /// # Errors
    /// Returns an error if any configuration value is invalid.
    pub fn validate(&self) -> Result<(), String> {
        if self.max_checkpoints == 0 {
            return Err("max_checkpoints must be greater than 0".to_string());
        }
        if self.max_size_gb == 0 {
            return Err("max_size_gb must be greater than 0".to_string());
        }
        if self.retention_days == 0 {
            return Err("retention_days must be greater than 0".to_string());
        }
        Ok(())
    }
}

/// Root configuration for Radium.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Config {
    /// Server configuration.
    #[serde(default)]
    pub server: ServerConfig,
    /// Model configuration.
    #[serde(default)]
    pub model: Option<ModelConfigSection>,
    /// Checkpoint configuration.
    #[serde(default)]
    pub checkpoint: CheckpointConfig,
}

impl Config {
    /// Create a new configuration with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from environment variables and config files.
    ///
    /// # Errors
    ///
    /// Returns an error if configuration loading fails.
    pub fn load() -> crate::error::Result<Self> {
        // For now, just return defaults
        // TODO: Implement config file and env var loading
        Ok(Self::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.address, "127.0.0.1:50051".parse().unwrap());
        assert!(config.enable_grpc_web);
        assert_eq!(config.web_address, None);
    }

    #[test]
    fn test_server_config_deserialize() {
        let json = r#"{"address": "127.0.0.1:8080", "enable_grpc_web": false}"#;
        let config: ServerConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.address, "127.0.0.1:8080".parse().unwrap());
        assert!(!config.enable_grpc_web);
    }

    #[test]
    fn test_server_config_with_web_address() {
        let json = r#"{"address": "127.0.0.1:50051", "web_address": "127.0.0.1:50052"}"#;
        let config: ServerConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.web_address, Some("127.0.0.1:50052".parse().unwrap()));
    }

    #[test]
    fn test_model_config_section_defaults() {
        let json = r#"{"model_id": "test-model"}"#;
        let config: ModelConfigSection = serde_json::from_str(json).unwrap();
        assert_eq!(config.model_type, "mock");
        assert_eq!(config.model_id, "test-model");
        assert_eq!(config.api_key, None);
    }

    #[test]
    fn test_model_config_section_with_all_fields() {
        let json = r#"{"model_type": "gemini", "model_id": "gemini-pro", "api_key": "test-key"}"#;
        let config: ModelConfigSection = serde_json::from_str(json).unwrap();
        assert_eq!(config.model_type, "gemini");
        assert_eq!(config.model_id, "gemini-pro");
        assert_eq!(config.api_key, Some("test-key".to_string()));
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.server.address, "127.0.0.1:50051".parse().unwrap());
        assert_eq!(config.model, None);
    }

    #[test]
    fn test_config_new() {
        let config = Config::new();
        assert_eq!(config.server.address, "127.0.0.1:50051".parse().unwrap());
    }

    #[test]
    fn test_config_load() {
        let config = Config::load().unwrap();
        assert_eq!(config.server.address, "127.0.0.1:50051".parse().unwrap());
    }

    #[test]
    fn test_config_deserialize_full() {
        let json = r#"{
            "server": {
                "address": "0.0.0.0:8080",
                "enable_grpc_web": true,
                "web_address": "0.0.0.0:8081"
            },
            "model": {
                "model_type": "openai",
                "model_id": "gpt-4",
                "api_key": "sk-test"
            }
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.server.address, "0.0.0.0:8080".parse().unwrap());
        assert_eq!(config.server.web_address, Some("0.0.0.0:8081".parse().unwrap()));
        assert!(config.model.is_some());
        let model = config.model.unwrap();
        assert_eq!(model.model_type, "openai");
        assert_eq!(model.model_id, "gpt-4");
        assert_eq!(model.api_key, Some("sk-test".to_string()));
    }

    #[test]
    fn test_config_deserialize_minimal() {
        let json = r"{}";
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.server.address, "127.0.0.1:50051".parse().unwrap());
        assert_eq!(config.model, None);
    }

    #[test]
    fn test_checkpoint_config_default() {
        let config = CheckpointConfig::default();
        assert!(config.auto_create);
        assert_eq!(config.retention_days, 7);
        assert_eq!(config.max_checkpoints, 50);
        assert_eq!(config.max_size_gb, 5);
    }

    #[test]
    fn test_checkpoint_config_deserialize() {
        let json = r#"{
            "auto_create": false,
            "retention_days": 14,
            "max_checkpoints": 100,
            "max_size_gb": 10
        }"#;
        let config: CheckpointConfig = serde_json::from_str(json).unwrap();
        assert!(!config.auto_create);
        assert_eq!(config.retention_days, 14);
        assert_eq!(config.max_checkpoints, 100);
        assert_eq!(config.max_size_gb, 10);
    }

    #[test]
    fn test_checkpoint_config_deserialize_partial() {
        let json = r#"{"max_checkpoints": 25}"#;
        let config: CheckpointConfig = serde_json::from_str(json).unwrap();
        assert!(config.auto_create); // Should use default
        assert_eq!(config.retention_days, 7); // Should use default
        assert_eq!(config.max_checkpoints, 25); // Custom value
        assert_eq!(config.max_size_gb, 5); // Should use default
    }

    #[test]
    fn test_checkpoint_config_validation() {
        let mut config = CheckpointConfig::default();
        assert!(config.validate().is_ok());

        config.max_checkpoints = 0;
        assert!(config.validate().is_err());

        config.max_checkpoints = 50;
        config.max_size_gb = 0;
        assert!(config.validate().is_err());

        config.max_size_gb = 5;
        config.retention_days = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_with_checkpoint() {
        let json = r#"{
            "server": {
                "address": "127.0.0.1:50051"
            },
            "checkpoint": {
                "auto_create": false,
                "retention_days": 14
            }
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert!(!config.checkpoint.auto_create);
        assert_eq!(config.checkpoint.retention_days, 14);
        assert_eq!(config.checkpoint.max_checkpoints, 50); // Default
        assert_eq!(config.checkpoint.max_size_gb, 5); // Default
    }
}
