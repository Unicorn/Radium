//! MCP proxy server configuration management.
//!
//! This module handles loading, parsing, and validation of proxy server
//! configuration from TOML files.

use crate::mcp::proxy::types::{ProxyConfig, ProxyTransport, SecurityConfig, UpstreamConfig};
use crate::mcp::{McpError, McpServerConfig, Result, TransportType};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use toml::Value as TomlValue;

/// Proxy configuration manager.
pub struct ProxyConfigManager {
    /// Configuration file path.
    config_path: PathBuf,
}

impl ProxyConfigManager {
    /// Create a new proxy configuration manager.
    pub fn new(config_path: PathBuf) -> Self {
        Self { config_path }
    }

    /// Get the default configuration file path.
    pub fn default_config_path(workspace_root: &Path) -> PathBuf {
        workspace_root.join(".radium").join("mcp-proxy.toml")
    }

    /// Load proxy configuration from the config file.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration cannot be loaded or parsed.
    pub fn load(&self) -> Result<ProxyConfig> {
        if !self.config_path.exists() {
            // Return default config if file doesn't exist
            return Ok(Self::generate_default());
        }

        let content = std::fs::read_to_string(&self.config_path).map_err(|e| {
            McpError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Failed to read proxy config file at {}: {}\n\nSuggestion: Ensure the file exists and has read permissions. You can create a new config file using 'rad mcp proxy init' or by manually creating {}",
                    self.config_path.display(),
                    e,
                    self.config_path.display()
                ),
            ))
        })?;

        let toml: toml::Table = toml::from_str(&content).map_err(|e| McpError::TomlParse(e))?;

        Self::parse_config(&toml)
    }

    /// Parse proxy configuration from TOML.
    fn parse_config(toml: &toml::Table) -> Result<ProxyConfig> {
        let mut config = Self::generate_default();

        // Parse [mcp.proxy] section
        if let Some(mcp_table) = toml.get("mcp").and_then(|m| m.as_table()) {
            if let Some(proxy_table) = mcp_table.get("proxy").and_then(|p| p.as_table()) {
                if let Some(enable) = proxy_table.get("enable").and_then(|e| e.as_bool()) {
                    config.enable = enable;
                }

                if let Some(port) = proxy_table.get("port").and_then(|p| p.as_integer()) {
                    config.port = port as u16;
                }

                if let Some(transport_str) = proxy_table.get("transport").and_then(|t| t.as_str()) {
                    config.transport = match transport_str {
                        "sse" => ProxyTransport::Sse,
                        "http" => ProxyTransport::Http,
                        _ => {
                            return Err(McpError::config(
                                format!("Invalid proxy transport type: '{}'", transport_str),
                                "Valid transport types are: 'sse' or 'http'.\n  - 'sse': Server-Sent Events transport\n  - 'http': HTTP transport\n\nExample:\n  transport = \"sse\"",
                            ));
                        }
                    };
                }

                if let Some(max_conn) = proxy_table.get("max_connections").and_then(|m| m.as_integer()) {
                    config.max_connections = max_conn as u32;
                }

                // Parse [mcp.proxy.security] section
                if let Some(security_table) = proxy_table.get("security").and_then(|s| s.as_table()) {
                    if let Some(log_requests) = security_table.get("log_requests").and_then(|l| l.as_bool()) {
                        config.security.log_requests = log_requests;
                    }

                    if let Some(log_responses) = security_table.get("log_responses").and_then(|l| l.as_bool()) {
                        config.security.log_responses = log_responses;
                    }

                    if let Some(redact_patterns) = security_table.get("redact_patterns").and_then(|r| r.as_array()) {
                        config.security.redact_patterns = redact_patterns
                            .iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect();
                    }

                    if let Some(rate_limit) = security_table.get("rate_limit_per_minute").and_then(|r| r.as_integer()) {
                        config.security.rate_limit_per_minute = rate_limit as u32;
                    }
                }

                // Parse [[mcp.proxy.upstreams]] array
                if let Some(upstreams_array) = proxy_table.get("upstreams").and_then(|u| u.as_array()) {
                    config.upstreams = Vec::new();
                    for upstream_value in upstreams_array {
                        if let Some(upstream_table) = upstream_value.as_table() {
                            let upstream = Self::parse_upstream_config(upstream_table)?;
                            config.upstreams.push(upstream);
                        }
                    }
                }
            }
        }

        // Validate the parsed configuration
        Self::validate_config(&config)?;

        Ok(config)
    }

    /// Parse a single upstream configuration from TOML.
    fn parse_upstream_config(upstream_table: &toml::Table) -> Result<UpstreamConfig> {
        // Parse base server config (name, transport, command, url, etc.)
        let server = crate::mcp::config::McpConfigManager::parse_server_config(upstream_table)?;

        // Parse proxy-specific fields
        let priority = upstream_table
            .get("priority")
            .and_then(|p| p.as_integer())
            .unwrap_or(1) as u32;

        let health_check_interval = upstream_table
            .get("health_check_interval")
            .and_then(|h| h.as_integer())
            .unwrap_or(30) as u64;

        let tools = upstream_table.get("tools").and_then(|t| {
            t.as_array().map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
        });

        Ok(UpstreamConfig {
            server,
            priority,
            health_check_interval,
            tools,
        })
    }

    /// Validate proxy configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails.
    pub fn validate_config(config: &ProxyConfig) -> Result<()> {
        // Validate port
        if config.port == 0 || config.port > 65535 {
            return Err(McpError::config(
                format!("Invalid port: {}. Port must be in range 1-65535", config.port),
                format!(
                    "Specify a valid port number. Example:\n  port = {}\n\nNote: Ports below 1024 may require elevated permissions on some systems.",
                    if config.port == 0 { 3000 } else { config.port }
                ),
            ));
        }

        // Validate max_connections
        if config.max_connections == 0 {
            return Err(McpError::config(
                "max_connections must be greater than 0",
                "Specify a positive number for max_connections. Example:\n  max_connections = 100",
            ));
        }

        // Validate transport (already validated during parsing, but double-check)
        match config.transport {
            ProxyTransport::Sse | ProxyTransport::Http => {}
        }

        // Validate upstreams
        let mut upstream_names = HashSet::new();
        for upstream in &config.upstreams {
            // Check for duplicate names
            if !upstream_names.insert(upstream.server.name.clone()) {
                return Err(McpError::config(
                    format!("Duplicate upstream name: '{}'", upstream.server.name),
                    format!(
                        "Each upstream must have a unique name. Example:\n  [[mcp.proxy.upstreams]]\n  name = \"{}\"\n  ...\n\n[[mcp.proxy.upstreams]]\n  name = \"{}-backup\"\n  ...",
                        upstream.server.name, upstream.server.name
                    ),
                ));
            }

            // Validate priority is positive
            if upstream.priority == 0 {
                return Err(McpError::config(
                    format!("Upstream '{}' has invalid priority: 0. Priority must be positive", upstream.server.name),
                    format!(
                        "Set a positive priority value. Lower numbers indicate higher priority. Example:\n  priority = 1  # Primary upstream\n  priority = 2  # Backup upstream",
                    ),
                ));
            }

            // Validate health_check_interval
            if upstream.health_check_interval == 0 {
                return Err(McpError::config(
                    format!("Upstream '{}' has invalid health_check_interval: 0. Must be greater than 0", upstream.server.name),
                    format!(
                        "Set a positive health check interval in seconds. Example:\n  health_check_interval = 30  # Check every 30 seconds",
                    ),
                ));
            }

            // Validate transport-specific fields (delegate to existing validation)
            match upstream.server.transport {
                TransportType::Stdio => {
                    if upstream.server.command.is_none() {
                        return Err(McpError::config(
                            format!("Upstream '{}' uses stdio transport but is missing 'command' field", upstream.server.name),
                            format!(
                                "Add a 'command' field for stdio transport. Example:\n  [[mcp.proxy.upstreams]]\n  name = \"{}\"\n  transport = \"stdio\"\n  command = \"mcp-server\"",
                                upstream.server.name
                            ),
                        ));
                    }
                }
                TransportType::Sse | TransportType::Http => {
                    if upstream.server.url.is_none() {
                        return Err(McpError::config(
                            format!("Upstream '{}' uses {} transport but is missing 'url' field", upstream.server.name, match upstream.server.transport {
                                TransportType::Sse => "SSE",
                                TransportType::Http => "HTTP",
                                _ => unreachable!(),
                            }),
                            format!(
                                "Add a 'url' field for {} transport. Example:\n  [[mcp.proxy.upstreams]]\n  name = \"{}\"\n  transport = \"{}\"\n  url = \"http://localhost:8080/mcp\"",
                                match upstream.server.transport {
                                    TransportType::Sse => "SSE",
                                    TransportType::Http => "HTTP",
                                    _ => unreachable!(),
                                },
                                upstream.server.name,
                                match upstream.server.transport {
                                    TransportType::Sse => "sse",
                                    TransportType::Http => "http",
                                    _ => unreachable!(),
                                }
                            ),
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Save proxy configuration to the config file.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration cannot be saved.
    pub fn save(&self, config: &ProxyConfig) -> Result<()> {
        // Validate before saving
        Self::validate_config(config)?;

        let mut toml_table = toml::Table::new();
        let mut mcp_table = toml::Table::new();
        let mut proxy_table = toml::Table::new();

        // Serialize proxy settings
        proxy_table.insert("enable".to_string(), TomlValue::Boolean(config.enable));
        proxy_table.insert("port".to_string(), TomlValue::Integer(config.port as i64));
        proxy_table.insert(
            "transport".to_string(),
            TomlValue::String(match config.transport {
                ProxyTransport::Sse => "sse".to_string(),
                ProxyTransport::Http => "http".to_string(),
            }),
        );
        proxy_table.insert(
            "max_connections".to_string(),
            TomlValue::Integer(config.max_connections as i64),
        );

        // Serialize security settings
        let mut security_table = toml::Table::new();
        security_table.insert(
            "log_requests".to_string(),
            TomlValue::Boolean(config.security.log_requests),
        );
        security_table.insert(
            "log_responses".to_string(),
            TomlValue::Boolean(config.security.log_responses),
        );
        security_table.insert(
            "redact_patterns".to_string(),
            TomlValue::Array(
                config
                    .security
                    .redact_patterns
                    .iter()
                    .map(|p| TomlValue::String(p.clone()))
                    .collect(),
            ),
        );
        security_table.insert(
            "rate_limit_per_minute".to_string(),
            TomlValue::Integer(config.security.rate_limit_per_minute as i64),
        );
        proxy_table.insert("security".to_string(), TomlValue::Table(security_table));

        // Serialize upstreams
        let mut upstreams_array = Vec::new();
        for upstream in &config.upstreams {
            let mut upstream_table = toml::Table::new();
            upstream_table.insert("name".to_string(), TomlValue::String(upstream.server.name.clone()));
            upstream_table.insert(
                "transport".to_string(),
                TomlValue::String(match upstream.server.transport {
                    TransportType::Stdio => "stdio".to_string(),
                    TransportType::Sse => "sse".to_string(),
                    TransportType::Http => "http".to_string(),
                }),
            );

            if let Some(ref command) = upstream.server.command {
                upstream_table.insert("command".to_string(), TomlValue::String(command.clone()));
            }

            if let Some(ref args) = upstream.server.args {
                upstream_table.insert(
                    "args".to_string(),
                    TomlValue::Array(
                        args.iter().map(|a| TomlValue::String(a.clone())).collect(),
                    ),
                );
            }

            if let Some(ref url) = upstream.server.url {
                upstream_table.insert("url".to_string(), TomlValue::String(url.clone()));
            }

            if let Some(ref auth) = upstream.server.auth {
                let mut auth_table = toml::Table::new();
                auth_table.insert(
                    "auth_type".to_string(),
                    TomlValue::String(auth.auth_type.clone()),
                );
                for (key, value) in &auth.params {
                    auth_table.insert(key.clone(), TomlValue::String(value.clone()));
                }
                upstream_table.insert("auth".to_string(), TomlValue::Table(auth_table));
            }

            upstream_table.insert(
                "priority".to_string(),
                TomlValue::Integer(upstream.priority as i64),
            );
            upstream_table.insert(
                "health_check_interval".to_string(),
                TomlValue::Integer(upstream.health_check_interval as i64),
            );

            if let Some(ref tools) = upstream.tools {
                upstream_table.insert(
                    "tools".to_string(),
                    TomlValue::Array(
                        tools.iter().map(|t| TomlValue::String(t.clone())).collect(),
                    ),
                );
            }

            upstreams_array.push(TomlValue::Table(upstream_table));
        }
        proxy_table.insert("upstreams".to_string(), TomlValue::Array(upstreams_array));

        mcp_table.insert("proxy".to_string(), TomlValue::Table(proxy_table));
        toml_table.insert("mcp".to_string(), TomlValue::Table(mcp_table));

        let content = toml::to_string_pretty(&toml_table).map_err(|e| {
            McpError::config(
                format!("Failed to serialize proxy configuration: {}", e),
                format!(
                    "This is an internal error. Please check that your proxy configurations are valid.\nConfig path: {}",
                    self.config_path.display()
                ),
            )
        })?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                McpError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "Failed to create config directory at {}: {}\n\nSuggestion: Ensure you have write permissions for the parent directory.",
                        parent.display(),
                        e
                    ),
                ))
            })?;
        }

        std::fs::write(&self.config_path, content).map_err(|e| {
            McpError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Failed to write proxy config file at {}: {}\n\nSuggestion: Ensure you have write permissions for the config file location.",
                    self.config_path.display(),
                    e
                ),
            ))
        })?;

        Ok(())
    }

    /// Generate a default proxy configuration.
    pub fn generate_default() -> ProxyConfig {
        ProxyConfig {
            enable: false,
            port: 3000,
            transport: ProxyTransport::Sse,
            max_connections: 100,
            security: SecurityConfig::default(),
            upstreams: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_generate_default() {
        let config = ProxyConfigManager::generate_default();
        assert!(!config.enable);
        assert_eq!(config.port, 3000);
        assert_eq!(config.transport, ProxyTransport::Sse);
        assert_eq!(config.max_connections, 100);
        assert!(config.upstreams.is_empty());
    }

    #[test]
    fn test_load_valid_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("mcp-proxy.toml");

        let toml_content = r#"
[mcp.proxy]
enable = true
port = 3000
transport = "sse"
max_connections = 100

[mcp.proxy.security]
log_requests = true
log_responses = true
redact_patterns = ["api_key", "password"]
rate_limit_per_minute = 60

[[mcp.proxy.upstreams]]
name = "test-upstream"
transport = "http"
url = "http://localhost:8080/mcp"
priority = 1
health_check_interval = 30
"#;

        std::fs::write(&config_path, toml_content).unwrap();

        let manager = ProxyConfigManager::new(config_path);
        let config = manager.load().unwrap();

        assert!(config.enable);
        assert_eq!(config.port, 3000);
        assert_eq!(config.transport, ProxyTransport::Sse);
        assert_eq!(config.upstreams.len(), 1);
        assert_eq!(config.upstreams[0].server.name, "test-upstream");
    }

    #[test]
    fn test_validate_config_invalid_port() {
        let mut config = ProxyConfigManager::generate_default();
        config.port = 70000;

        let result = ProxyConfigManager::validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("port"));
    }

    #[test]
    fn test_validate_config_zero_port() {
        let mut config = ProxyConfigManager::generate_default();
        config.port = 0;

        let result = ProxyConfigManager::validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("port"));
    }

    #[test]
    fn test_validate_config_zero_max_connections() {
        let mut config = ProxyConfigManager::generate_default();
        config.max_connections = 0;

        let result = ProxyConfigManager::validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("max_connections"));
    }

    #[test]
    fn test_validate_config_duplicate_upstream_names() {
        let mut config = ProxyConfigManager::generate_default();
        config.upstreams.push(UpstreamConfig {
            server: McpServerConfig {
                name: "duplicate".to_string(),
                transport: TransportType::Http,
                command: None,
                args: None,
                url: Some("http://localhost:8080".to_string()),
                auth: None,
            },
            priority: 1,
            health_check_interval: 30,
            tools: None,
        });
        config.upstreams.push(UpstreamConfig {
            server: McpServerConfig {
                name: "duplicate".to_string(),
                transport: TransportType::Http,
                command: None,
                args: None,
                url: Some("http://localhost:8081".to_string()),
                auth: None,
            },
            priority: 2,
            health_check_interval: 30,
            tools: None,
        });

        let result = ProxyConfigManager::validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate"));
    }

    #[test]
    fn test_validate_config_upstream_missing_url() {
        let mut config = ProxyConfigManager::generate_default();
        config.upstreams.push(UpstreamConfig {
            server: McpServerConfig {
                name: "test".to_string(),
                transport: TransportType::Sse,
                command: None,
                args: None,
                url: None, // Missing URL
                auth: None,
            },
            priority: 1,
            health_check_interval: 30,
            tools: None,
        });

        let result = ProxyConfigManager::validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("url"));
    }

    #[test]
    fn test_validate_config_upstream_zero_priority() {
        let mut config = ProxyConfigManager::generate_default();
        config.upstreams.push(UpstreamConfig {
            server: McpServerConfig {
                name: "test".to_string(),
                transport: TransportType::Http,
                command: None,
                args: None,
                url: Some("http://localhost:8080".to_string()),
                auth: None,
            },
            priority: 0, // Invalid priority
            health_check_interval: 30,
            tools: None,
        });

        let result = ProxyConfigManager::validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("priority"));
    }

    #[test]
    fn test_validate_config_upstream_zero_health_check_interval() {
        let mut config = ProxyConfigManager::generate_default();
        config.upstreams.push(UpstreamConfig {
            server: McpServerConfig {
                name: "test".to_string(),
                transport: TransportType::Http,
                command: None,
                args: None,
                url: Some("http://localhost:8080".to_string()),
                auth: None,
            },
            priority: 1,
            health_check_interval: 0, // Invalid interval
            tools: None,
        });

        let result = ProxyConfigManager::validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("health_check_interval"));
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("mcp-proxy.toml");

        let manager = ProxyConfigManager::new(config_path.clone());
        let original_config = ProxyConfigManager::generate_default();
        manager.save(&original_config).unwrap();

        let loaded_config = manager.load().unwrap();

        assert_eq!(original_config.enable, loaded_config.enable);
        assert_eq!(original_config.port, loaded_config.port);
        assert_eq!(original_config.transport, loaded_config.transport);
        assert_eq!(original_config.max_connections, loaded_config.max_connections);
    }

    #[test]
    fn test_load_missing_file_returns_default() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nonexistent.toml");

        let manager = ProxyConfigManager::new(config_path);
        let config = manager.load().unwrap();

        // Should return default config
        assert_eq!(config.port, 3000);
        assert!(!config.enable);
    }
}
