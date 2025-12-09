//! Configuration module for Radium Core.

pub mod cli_config;
pub mod engine_costs;
pub mod model_cache;
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

/// Custom pattern definition for privacy filtering.
#[derive(Debug, Clone, Deserialize)]
pub struct CustomPattern {
    /// Name of the pattern.
    pub name: String,
    /// Regex pattern to match.
    pub regex: String,
    /// Replacement string for matches.
    pub replacement: String,
}

/// Privacy configuration for sensitive data redaction.
#[derive(Debug, Clone, Deserialize)]
pub struct PrivacyConfig {
    /// Enable privacy mode.
    #[serde(default = "default_true")]
    pub enable: bool,
    /// Privacy mode: "auto", "strict", or "off".
    #[serde(default = "default_privacy_mode")]
    pub mode: String,
    /// Redaction style: "full", "partial", or "hash".
    #[serde(default = "default_redaction_style")]
    pub redaction_style: String,
    /// Enable audit logging of redactions.
    #[serde(default = "default_true")]
    pub audit_log: bool,
    /// Custom patterns for organization-specific sensitive data.
    #[serde(default)]
    pub custom_patterns: Vec<CustomPattern>,
}

fn default_privacy_mode() -> String {
    "auto".to_string()
}

fn default_redaction_style() -> String {
    "partial".to_string()
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            enable: true,
            mode: default_privacy_mode(),
            redaction_style: default_redaction_style(),
            audit_log: true,
            custom_patterns: Vec::new(),
        }
    }
}

/// Secret management configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct SecretManagementConfig {
    /// Enable secret redaction before sending to LLMs.
    #[serde(default = "default_true")]
    pub enable_secret_redaction: bool,
    /// Enable secret injection before tool execution.
    #[serde(default = "default_true")]
    pub enable_secret_injection: bool,
    /// Enable audit logging of secret operations.
    #[serde(default = "default_true")]
    pub enable_audit_logging: bool,
    /// Warn when hardcoded secrets are detected in workspace.
    #[serde(default = "default_true")]
    pub warn_on_hardcoded_secrets: bool,
    /// Path to the secret vault file.
    #[serde(default = "default_vault_path")]
    pub secret_vault_path: String,
    /// Path to the audit log file.
    #[serde(default = "default_audit_log_path")]
    pub audit_log_path: String,
    /// Minimum master password length.
    #[serde(default = "default_min_password_length")]
    pub master_password_min_length: usize,
}

fn default_vault_path() -> String {
    #[allow(clippy::disallowed_methods)]
    std::env::var("HOME")
        .map(|home| format!("{}/.radium/auth/secrets.vault", home))
        .unwrap_or_else(|_| "~/.radium/auth/secrets.vault".to_string())
}

fn default_audit_log_path() -> String {
    #[allow(clippy::disallowed_methods)]
    std::env::var("HOME")
        .map(|home| format!("{}/.radium/auth/audit.log", home))
        .unwrap_or_else(|_| "~/.radium/auth/audit.log".to_string())
}

fn default_min_password_length() -> usize {
    12
}

impl Default for SecretManagementConfig {
    fn default() -> Self {
        Self {
            enable_secret_redaction: true,
            enable_secret_injection: true,
            enable_audit_logging: true,
            warn_on_hardcoded_secrets: true,
            secret_vault_path: default_vault_path(),
            audit_log_path: default_audit_log_path(),
            master_password_min_length: 12,
        }
    }
}

/// Security configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct SecurityConfig {
    /// Privacy configuration.
    #[serde(default)]
    pub privacy: PrivacyConfig,
    /// Secret management configuration.
    #[serde(default)]
    pub secrets: SecretManagementConfig,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            privacy: PrivacyConfig::default(),
            secrets: SecretManagementConfig::default(),
        }
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
    /// Security configuration.
    #[serde(default)]
    pub security: SecurityConfig,
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

    #[test]
    fn test_privacy_config_default() {
        let config = PrivacyConfig::default();
        assert!(config.enable);
        assert_eq!(config.mode, "auto");
        assert_eq!(config.redaction_style, "partial");
        assert!(config.audit_log);
        assert!(config.custom_patterns.is_empty());
    }

    #[test]
    fn test_privacy_config_deserialize() {
        let json = r#"{
            "enable": false,
            "mode": "strict",
            "redaction_style": "hash",
            "audit_log": false
        }"#;
        let config: PrivacyConfig = serde_json::from_str(json).unwrap();
        assert!(!config.enable);
        assert_eq!(config.mode, "strict");
        assert_eq!(config.redaction_style, "hash");
        assert!(!config.audit_log);
    }

    #[test]
    fn test_privacy_config_deserialize_with_defaults() {
        let json = r#"{}"#;
        let config: PrivacyConfig = serde_json::from_str(json).unwrap();
        assert!(config.enable); // Default
        assert_eq!(config.mode, "auto"); // Default
        assert_eq!(config.redaction_style, "partial"); // Default
        assert!(config.audit_log); // Default
    }

    #[test]
    fn test_privacy_config_with_custom_patterns() {
        let json = r#"{
            "custom_patterns": [
                {
                    "name": "custom_token",
                    "regex": "tok_[a-zA-Z0-9]{40}",
                    "replacement": "***TOKEN***"
                }
            ]
        }"#;
        let config: PrivacyConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.custom_patterns.len(), 1);
        assert_eq!(config.custom_patterns[0].name, "custom_token");
        assert_eq!(config.custom_patterns[0].regex, "tok_[a-zA-Z0-9]{40}");
        assert_eq!(config.custom_patterns[0].replacement, "***TOKEN***");
    }

    #[test]
    fn test_security_config_default() {
        let config = SecurityConfig::default();
        assert!(config.privacy.enable);
        assert_eq!(config.privacy.mode, "auto");
    }

    #[test]
    fn test_config_with_security() {
        let json = r#"{
            "server": {
                "address": "127.0.0.1:50051"
            },
            "security": {
                "privacy": {
                    "enable": false,
                    "mode": "strict",
                    "redaction_style": "full"
                }
            }
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert!(!config.security.privacy.enable);
        assert_eq!(config.security.privacy.mode, "strict");
        assert_eq!(config.security.privacy.redaction_style, "full");
    }

    #[test]
    fn test_config_deserialize_minimal_with_security_defaults() {
        let json = r"{}";
        let config: Config = serde_json::from_str(json).unwrap();
        assert!(config.security.privacy.enable); // Default
        assert_eq!(config.security.privacy.mode, "auto"); // Default
        assert_eq!(config.security.privacy.redaction_style, "partial"); // Default
    }
}
