//! Sandbox configuration types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Sandbox type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SandboxType {
    /// No sandboxing (default).
    None,
    /// Docker container sandboxing.
    Docker,
    /// Podman container sandboxing.
    Podman,
    /// macOS Seatbelt sandboxing.
    #[serde(rename = "seatbelt")]
    Seatbelt,
}

impl Default for SandboxType {
    fn default() -> Self {
        Self::None
    }
}

impl std::fmt::Display for SandboxType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SandboxType::None => write!(f, "none"),
            SandboxType::Docker => write!(f, "docker"),
            SandboxType::Podman => write!(f, "podman"),
            SandboxType::Seatbelt => write!(f, "seatbelt"),
        }
    }
}

/// Sandbox network configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NetworkMode {
    /// Network fully open.
    Open,
    /// Network fully closed.
    Closed,
    /// Network proxied through host.
    Proxied,
}

impl Default for NetworkMode {
    fn default() -> Self {
        Self::Open
    }
}

/// Sandbox profile type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SandboxProfile {
    /// Permissive profile (minimal restrictions).
    Permissive,
    /// Restrictive profile (maximum restrictions).
    Restrictive,
    /// Custom profile from file.
    Custom(String),
}

impl Default for SandboxProfile {
    fn default() -> Self {
        Self::Permissive
    }
}

/// Sandbox configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Sandbox type to use.
    #[serde(default)]
    pub sandbox_type: SandboxType,

    /// Sandbox profile.
    #[serde(default)]
    pub profile: SandboxProfile,

    /// Network mode.
    #[serde(default)]
    pub network: NetworkMode,

    /// Custom sandbox flags (for Docker/Podman).
    #[serde(default)]
    pub custom_flags: Vec<String>,

    /// Environment variables to pass to sandbox.
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Working directory inside sandbox.
    #[serde(default)]
    pub working_dir: Option<String>,

    /// Volumes to mount (host:container format).
    #[serde(default)]
    pub volumes: Vec<String>,

    /// Container image (for Docker/Podman).
    #[serde(default)]
    pub image: Option<String>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            sandbox_type: SandboxType::None,
            profile: SandboxProfile::Permissive,
            network: NetworkMode::Open,
            custom_flags: Vec::new(),
            env: HashMap::new(),
            working_dir: None,
            volumes: Vec::new(),
            image: None,
        }
    }
}

impl SandboxConfig {
    /// Creates a new sandbox configuration.
    pub fn new(sandbox_type: SandboxType) -> Self {
        Self { sandbox_type, ..Default::default() }
    }

    /// Sets the sandbox profile.
    pub fn with_profile(mut self, profile: SandboxProfile) -> Self {
        self.profile = profile;
        self
    }

    /// Sets the network mode.
    pub fn with_network(mut self, network: NetworkMode) -> Self {
        self.network = network;
        self
    }

    /// Adds custom flags.
    pub fn with_flags(mut self, flags: Vec<String>) -> Self {
        self.custom_flags = flags;
        self
    }

    /// Adds environment variables.
    pub fn with_env(mut self, env: HashMap<String, String>) -> Self {
        self.env = env;
        self
    }

    /// Sets the working directory.
    pub fn with_working_dir(mut self, dir: String) -> Self {
        self.working_dir = Some(dir);
        self
    }

    /// Sets volumes to mount.
    pub fn with_volumes(mut self, volumes: Vec<String>) -> Self {
        self.volumes = volumes;
        self
    }

    /// Sets the container image.
    pub fn with_image(mut self, image: String) -> Self {
        self.image = Some(image);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_config_default() {
        let config = SandboxConfig::default();
        assert_eq!(config.sandbox_type, SandboxType::None);
        assert_eq!(config.profile, SandboxProfile::Permissive);
        assert_eq!(config.network, NetworkMode::Open);
    }

    #[test]
    fn test_sandbox_config_builder() {
        let mut env = HashMap::new();
        env.insert("KEY".to_string(), "value".to_string());

        let config = SandboxConfig::new(SandboxType::Docker)
            .with_profile(SandboxProfile::Restrictive)
            .with_network(NetworkMode::Closed)
            .with_env(env.clone())
            .with_working_dir("/app".to_string())
            .with_image("rust:latest".to_string());

        assert_eq!(config.sandbox_type, SandboxType::Docker);
        assert_eq!(config.profile, SandboxProfile::Restrictive);
        assert_eq!(config.network, NetworkMode::Closed);
        assert_eq!(config.env.get("KEY"), Some(&"value".to_string()));
        assert_eq!(config.working_dir, Some("/app".to_string()));
        assert_eq!(config.image, Some("rust:latest".to_string()));
    }

    #[test]
    fn test_sandbox_type_serde() {
        let json = r#"{"sandbox_type":"docker"}"#;
        let config: serde_json::Value = serde_json::from_str(json).unwrap();
        assert_eq!(config["sandbox_type"], "docker");
    }

    #[test]
    fn test_sandbox_type_all_variants() {
        assert_eq!(SandboxType::None.to_string(), "none");
        assert_eq!(SandboxType::Docker.to_string(), "docker");
        assert_eq!(SandboxType::Podman.to_string(), "podman");
        assert_eq!(SandboxType::Seatbelt.to_string(), "seatbelt");
    }

    #[test]
    fn test_network_mode_serialization() {
        let open = serde_json::to_string(&NetworkMode::Open).unwrap();
        assert_eq!(open, r#""open""#);

        let closed = serde_json::to_string(&NetworkMode::Closed).unwrap();
        assert_eq!(closed, r#""closed""#);

        let proxied = serde_json::to_string(&NetworkMode::Proxied).unwrap();
        assert_eq!(proxied, r#""proxied""#);
    }

    #[test]
    fn test_sandbox_profile_serialization() {
        let permissive = serde_json::to_string(&SandboxProfile::Permissive).unwrap();
        assert_eq!(permissive, r#""permissive""#);

        let restrictive = serde_json::to_string(&SandboxProfile::Restrictive).unwrap();
        assert_eq!(restrictive, r#""restrictive""#);
    }

    #[test]
    fn test_sandbox_config_with_flags() {
        let flags = vec!["--flag1".to_string(), "--flag2".to_string()];
        let config = SandboxConfig::new(SandboxType::Docker).with_flags(flags.clone());

        assert_eq!(config.custom_flags, flags);
    }

    #[test]
    fn test_sandbox_config_multiple_env_vars() {
        let mut env = HashMap::new();
        env.insert("VAR1".to_string(), "value1".to_string());
        env.insert("VAR2".to_string(), "value2".to_string());
        env.insert("VAR3".to_string(), "value3".to_string());

        let config = SandboxConfig::new(SandboxType::Docker).with_env(env.clone());

        assert_eq!(config.env.len(), 3);
        assert_eq!(config.env.get("VAR1"), Some(&"value1".to_string()));
        assert_eq!(config.env.get("VAR2"), Some(&"value2".to_string()));
        assert_eq!(config.env.get("VAR3"), Some(&"value3".to_string()));
    }

    #[test]
    fn test_sandbox_config_builder_chaining() {
        let config = SandboxConfig::new(SandboxType::Seatbelt)
            .with_profile(SandboxProfile::Restrictive)
            .with_network(NetworkMode::Proxied)
            .with_working_dir("/workspace".to_string());

        assert_eq!(config.sandbox_type, SandboxType::Seatbelt);
        assert_eq!(config.profile, SandboxProfile::Restrictive);
        assert_eq!(config.network, NetworkMode::Proxied);
        assert_eq!(config.working_dir, Some("/workspace".to_string()));
    }

    #[test]
    fn test_sandbox_config_empty_env() {
        let config = SandboxConfig::new(SandboxType::None);
        assert!(config.env.is_empty());
    }

    #[test]
    fn test_sandbox_config_empty_flags() {
        let config = SandboxConfig::new(SandboxType::None);
        assert!(config.custom_flags.is_empty());
    }
}
