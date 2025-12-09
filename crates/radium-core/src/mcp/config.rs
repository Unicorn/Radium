//! MCP server configuration management.

use crate::mcp::{McpError, McpServerConfig, Result, TransportType};
use std::path::{Path, PathBuf};
use toml::Value as TomlValue;

/// MCP server configuration manager.
pub struct McpConfigManager {
    /// Configuration file path.
    config_path: PathBuf,
    /// Loaded server configurations.
    servers: Vec<McpServerConfig>,
}

impl McpConfigManager {
    /// Create a new configuration manager.
    pub fn new(config_path: PathBuf) -> Self {
        Self { config_path, servers: Vec::new() }
    }

    /// Get the default configuration file path.
    pub fn default_config_path(workspace_root: &Path) -> PathBuf {
        workspace_root.join(".radium").join("mcp-servers.toml")
    }

    /// Load server configurations from the config file.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration cannot be loaded or parsed.
    pub fn load(&mut self) -> Result<()> {
        if !self.config_path.exists() {
            // No config file is not an error - just return empty config
            self.servers = Vec::new();
            return Ok(());
        }

        let content = std::fs::read_to_string(&self.config_path).map_err(|e| {
            McpError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Failed to read MCP config file at {}: {}\n\nSuggestion: Ensure the file exists and has read permissions. You can create a new config file using 'rad mcp setup' or by manually creating {}",
                    self.config_path.display(),
                    e,
                    self.config_path.display()
                ),
            ))
        })?;

        let toml: toml::Table = toml::from_str(&content).map_err(|e| McpError::TomlParse(e))?;

        self.servers = Self::parse_servers(&toml)?;

        Ok(())
    }

    /// Parse server configurations from TOML.
    ///
    /// This is public to allow extension integration to parse TOML MCP configs.
    pub fn parse_servers(toml: &toml::Table) -> Result<Vec<McpServerConfig>> {
        let mut servers = Vec::new();

        if let Some(servers_array) = toml.get("servers").and_then(|s| s.as_array()) {
            for server_value in servers_array {
                if let Some(server_table) = server_value.as_table() {
                    let server = Self::parse_server_config(server_table)?;
                    servers.push(server);
                }
            }
        }

        Ok(servers)
    }

    /// Parse a single server configuration from TOML.
    pub fn parse_server_config(server_table: &toml::Table) -> Result<McpServerConfig> {
        let name = server_table
            .get("name")
            .and_then(|n| n.as_str())
            .ok_or_else(|| {
                McpError::config(
                    "Server configuration missing 'name' field",
                    "Add a 'name' field to your server configuration. Example:\n  [[servers]]\n  name = \"my-server\"\n  transport = \"stdio\"\n  command = \"mcp-server\"",
                )
            })?
            .to_string();

        let transport_str =
            server_table.get("transport").and_then(|t| t.as_str()).ok_or_else(|| {
                McpError::config(
                    "Server configuration missing 'transport' field",
                    "Add a 'transport' field to your server configuration. Valid values are: 'stdio', 'sse', or 'http'. Example:\n  [[servers]]\n  name = \"my-server\"\n  transport = \"stdio\"\n  command = \"mcp-server\"",
                )
            })?;

        let transport = match transport_str {
            "stdio" => TransportType::Stdio,
            "sse" => TransportType::Sse,
            "http" => TransportType::Http,
            _ => {
                return Err(McpError::config(
                    format!("Invalid transport type: '{}'", transport_str),
                    format!(
                        "Valid transport types are: 'stdio', 'sse', or 'http'.\n  - 'stdio': For local command-line MCP servers\n  - 'sse': For Server-Sent Events (SSE) endpoints\n  - 'http': For HTTP-based MCP servers\n\nExample:\n  transport = \"stdio\""
                    ),
                ));
            }
        };

        let command = server_table.get("command").and_then(|c| c.as_str()).map(|s| s.to_string());

        let args = server_table
            .get("args")
            .and_then(|a| a.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect());

        let url = server_table.get("url").and_then(|u| u.as_str()).map(|s| s.to_string());

        // Validate transport-specific requirements
        match transport {
            TransportType::Stdio => {
                if command.is_none() {
                    return Err(McpError::config(
                        format!("Stdio transport requires 'command' field for server '{}'", name),
                        format!(
                            "Add a 'command' field specifying the executable to run. Example:\n  [[servers]]\n  name = \"{}\"\n  transport = \"stdio\"\n  command = \"mcp-server\"\n  args = [\"--config\", \"config.json\"]",
                            name
                        ),
                    ));
                }
            }
            TransportType::Sse | TransportType::Http => {
                if url.is_none() {
                    return Err(McpError::config(
                        format!("{} transport requires 'url' field for server '{}'", transport_str, name),
                        format!(
                            "Add a 'url' field specifying the server endpoint. Example:\n  [[servers]]\n  name = \"{}\"\n  transport = \"{}\"\n  url = \"http://localhost:8080/mcp\"",
                            name, transport_str
                        ),
                    ));
                }
            }
        }

        // Parse auth configuration if present
        let auth = server_table.get("auth").and_then(|a| {
            a.as_table().map(|auth_table| {
                let auth_type = auth_table
                    .get("auth_type")
                    .and_then(|t| t.as_str())
                    .unwrap_or("oauth")
                    .to_string();

                let mut params = std::collections::HashMap::new();
                for (key, value) in auth_table {
                    if key != "auth_type" {
                        if let Some(str_val) = value.as_str() {
                            params.insert(key.clone(), str_val.to_string());
                        }
                    }
                }

                crate::mcp::McpAuthConfig { auth_type, params }
            })
        });

        Ok(McpServerConfig { name, transport, command, args, url, auth })
    }

    /// Get all server configurations.
    pub fn get_servers(&self) -> &[McpServerConfig] {
        &self.servers
    }

    /// Get mutable reference to all server configurations.
    pub fn get_servers_mut(&mut self) -> &mut Vec<McpServerConfig> {
        &mut self.servers
    }

    /// Get a server configuration by name.
    pub fn get_server(&self, name: &str) -> Option<&McpServerConfig> {
        self.servers.iter().find(|s| s.name == name)
    }

    /// Save server configurations to the config file.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration cannot be saved.
    pub fn save(&self) -> Result<()> {
        let mut toml_table = toml::Table::new();

        let mut servers_array = Vec::new();
        for server in &self.servers {
            let mut server_table = toml::Table::new();
            server_table.insert("name".to_string(), TomlValue::String(server.name.clone()));
            server_table.insert(
                "transport".to_string(),
                TomlValue::String(match server.transport {
                    TransportType::Stdio => "stdio".to_string(),
                    TransportType::Sse => "sse".to_string(),
                    TransportType::Http => "http".to_string(),
                }),
            );

            if let Some(ref command) = server.command {
                server_table.insert("command".to_string(), TomlValue::String(command.clone()));
            }

            if let Some(ref args) = server.args {
                let args_array: Vec<TomlValue> =
                    args.iter().map(|a| TomlValue::String(a.clone())).collect();
                server_table.insert("args".to_string(), TomlValue::Array(args_array));
            }

            if let Some(ref url) = server.url {
                server_table.insert("url".to_string(), TomlValue::String(url.clone()));
            }

            if let Some(ref auth) = server.auth {
                let mut auth_table = toml::Table::new();
                auth_table
                    .insert("auth_type".to_string(), TomlValue::String(auth.auth_type.clone()));
                for (key, value) in &auth.params {
                    auth_table.insert(key.clone(), TomlValue::String(value.clone()));
                }
                server_table.insert("auth".to_string(), TomlValue::Table(auth_table));
            }

            servers_array.push(TomlValue::Table(server_table));
        }

        toml_table.insert("servers".to_string(), TomlValue::Array(servers_array));

        let content = toml::to_string_pretty(&toml_table)
            .map_err(|e| McpError::config(
                format!("Failed to serialize configuration: {}", e),
                format!(
                    "This is an internal error. Please check that your server configurations are valid.\nConfig path: {}\nIf the problem persists, try removing the config file and recreating it.",
                    self.config_path.display()
                ),
            ))?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                McpError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "Failed to create config directory at {}: {}\n\nSuggestion: Ensure you have write permissions for the parent directory. You may need to create the directory manually or run with appropriate permissions.",
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
                    "Failed to write config file at {}: {}\n\nSuggestion: Ensure you have write permissions for the config file location. Check that the file is not locked by another process.",
                    self.config_path.display(),
                    e
                ),
            ))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_parse_stdio_server_config() {
        let mut table = toml::Table::new();
        table.insert("name".to_string(), TomlValue::String("test-server".to_string()));
        table.insert("transport".to_string(), TomlValue::String("stdio".to_string()));
        table.insert("command".to_string(), TomlValue::String("mcp-server".to_string()));
        table.insert(
            "args".to_string(),
            TomlValue::Array(vec![
                TomlValue::String("--config".to_string()),
                TomlValue::String("config.json".to_string()),
            ]),
        );

        let config = McpConfigManager::parse_server_config(&table).unwrap();
        assert_eq!(config.name, "test-server");
        assert_eq!(config.transport, TransportType::Stdio);
        assert_eq!(config.command, Some("mcp-server".to_string()));
    }

    #[test]
    fn test_parse_sse_server_config() {
        let mut table = toml::Table::new();
        table.insert("name".to_string(), TomlValue::String("sse-server".to_string()));
        table.insert("transport".to_string(), TomlValue::String("sse".to_string()));
        table.insert("url".to_string(), TomlValue::String("http://localhost:8080/sse".to_string()));

        let config = McpConfigManager::parse_server_config(&table).unwrap();
        assert_eq!(config.name, "sse-server");
        assert_eq!(config.transport, TransportType::Sse);
        assert_eq!(config.url, Some("http://localhost:8080/sse".to_string()));
    }

    #[test]
    fn test_config_manager_load_and_save() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("mcp-servers.toml");
        let mut manager = McpConfigManager::new(config_path.clone());

        // Create a test config
        let server = McpServerConfig {
            name: "test-server".to_string(),
            transport: TransportType::Stdio,
            command: Some("mcp-server".to_string()),
            args: Some(vec!["--config".to_string(), "config.json".to_string()]),
            url: None,
            auth: None,
        };
        manager.servers.push(server);

        // Save and reload
        manager.save().unwrap();
        let mut new_manager = McpConfigManager::new(config_path);
        new_manager.load().unwrap();

        assert_eq!(new_manager.get_servers().len(), 1);
        assert_eq!(new_manager.get_server("test-server").unwrap().name, "test-server");
    }

    #[test]
    fn test_parse_http_server_config() {
        let mut table = toml::Table::new();
        table.insert("name".to_string(), TomlValue::String("http-server".to_string()));
        table.insert("transport".to_string(), TomlValue::String("http".to_string()));
        table.insert(
            "url".to_string(),
            TomlValue::String("https://api.example.com/mcp".to_string()),
        );

        let config = McpConfigManager::parse_server_config(&table).unwrap();
        assert_eq!(config.name, "http-server");
        assert_eq!(config.transport, TransportType::Http);
        assert_eq!(config.url, Some("https://api.example.com/mcp".to_string()));
    }

    #[test]
    fn test_parse_server_config_missing_name() {
        let mut table = toml::Table::new();
        table.insert("transport".to_string(), TomlValue::String("stdio".to_string()));
        table.insert("command".to_string(), TomlValue::String("mcp-server".to_string()));

        let result = McpConfigManager::parse_server_config(&table);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("name"));
    }

    #[test]
    fn test_parse_server_config_missing_transport() {
        let mut table = toml::Table::new();
        table.insert("name".to_string(), TomlValue::String("test-server".to_string()));

        let result = McpConfigManager::parse_server_config(&table);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("transport"));
    }

    #[test]
    fn test_parse_server_config_invalid_transport() {
        let mut table = toml::Table::new();
        table.insert("name".to_string(), TomlValue::String("test-server".to_string()));
        table.insert("transport".to_string(), TomlValue::String("invalid".to_string()));

        let result = McpConfigManager::parse_server_config(&table);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid transport"));
    }

    #[test]
    fn test_parse_server_config_stdio_missing_command() {
        let mut table = toml::Table::new();
        table.insert("name".to_string(), TomlValue::String("test-server".to_string()));
        table.insert("transport".to_string(), TomlValue::String("stdio".to_string()));

        let result = McpConfigManager::parse_server_config(&table);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("command"));
    }

    #[test]
    fn test_parse_server_config_sse_missing_url() {
        let mut table = toml::Table::new();
        table.insert("name".to_string(), TomlValue::String("test-server".to_string()));
        table.insert("transport".to_string(), TomlValue::String("sse".to_string()));

        let result = McpConfigManager::parse_server_config(&table);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("url"));
    }

    #[test]
    fn test_parse_server_config_http_missing_url() {
        let mut table = toml::Table::new();
        table.insert("name".to_string(), TomlValue::String("test-server".to_string()));
        table.insert("transport".to_string(), TomlValue::String("http".to_string()));

        let result = McpConfigManager::parse_server_config(&table);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("url"));
    }

    #[test]
    fn test_parse_server_config_with_auth() {
        let mut table = toml::Table::new();
        table.insert("name".to_string(), TomlValue::String("auth-server".to_string()));
        table.insert("transport".to_string(), TomlValue::String("http".to_string()));
        table.insert(
            "url".to_string(),
            TomlValue::String("https://api.example.com/mcp".to_string()),
        );

        let mut auth_table = toml::Table::new();
        auth_table.insert("auth_type".to_string(), TomlValue::String("oauth".to_string()));
        auth_table.insert("client_id".to_string(), TomlValue::String("test-id".to_string()));
        auth_table
            .insert("client_secret".to_string(), TomlValue::String("test-secret".to_string()));
        table.insert("auth".to_string(), TomlValue::Table(auth_table));

        let config = McpConfigManager::parse_server_config(&table).unwrap();
        assert!(config.auth.is_some());
        assert_eq!(config.auth.as_ref().unwrap().auth_type, "oauth");
        assert_eq!(
            config.auth.as_ref().unwrap().params.get("client_id"),
            Some(&"test-id".to_string())
        );
    }

    #[test]
    fn test_config_manager_load_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nonexistent.toml");
        let mut manager = McpConfigManager::new(config_path);

        // Loading a missing file should not error, just return empty config
        manager.load().unwrap();
        assert_eq!(manager.get_servers().len(), 0);
    }

    #[test]
    fn test_config_manager_load_invalid_toml() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("mcp-servers.toml");
        std::fs::write(&config_path, "[invalid toml").unwrap();

        let mut manager = McpConfigManager::new(config_path);
        let result = manager.load();
        assert!(result.is_err());
    }

    #[test]
    fn test_config_manager_multiple_servers() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("mcp-servers.toml");
        let mut manager = McpConfigManager::new(config_path.clone());

        // Add multiple servers
        manager.servers.push(McpServerConfig {
            name: "server1".to_string(),
            transport: TransportType::Stdio,
            command: Some("mcp1".to_string()),
            args: None,
            url: None,
            auth: None,
        });

        manager.servers.push(McpServerConfig {
            name: "server2".to_string(),
            transport: TransportType::Http,
            command: None,
            args: None,
            url: Some("http://localhost:8080".to_string()),
            auth: None,
        });

        manager.save().unwrap();

        let mut new_manager = McpConfigManager::new(config_path);
        new_manager.load().unwrap();

        assert_eq!(new_manager.get_servers().len(), 2);
        assert!(new_manager.get_server("server1").is_some());
        assert!(new_manager.get_server("server2").is_some());
    }

    #[test]
    fn test_config_manager_get_server_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("mcp-servers.toml");
        let manager = McpConfigManager::new(config_path);

        assert!(manager.get_server("nonexistent").is_none());
    }

    #[test]
    fn test_config_manager_parse_servers_empty_array() {
        let mut table = toml::Table::new();
        table.insert("servers".to_string(), TomlValue::Array(Vec::new()));

        let servers = McpConfigManager::parse_servers(&table).unwrap();
        assert_eq!(servers.len(), 0);
    }

    #[test]
    fn test_config_manager_parse_servers_missing_key() {
        let table = toml::Table::new();
        let servers = McpConfigManager::parse_servers(&table).unwrap();
        assert_eq!(servers.len(), 0);
    }

    #[test]
    fn test_config_manager_parse_servers_invalid_entry() {
        let mut table = toml::Table::new();
        table.insert(
            "servers".to_string(),
            TomlValue::Array(vec![TomlValue::String("not a table".to_string())]),
        );

        let servers = McpConfigManager::parse_servers(&table).unwrap();
        // Invalid entries should be skipped
        assert_eq!(servers.len(), 0);
    }

    #[test]
    fn test_config_manager_save_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("subdir").join("mcp-servers.toml");
        let mut manager = McpConfigManager::new(config_path.clone());

        manager.servers.push(McpServerConfig {
            name: "test-server".to_string(),
            transport: TransportType::Stdio,
            command: Some("mcp-server".to_string()),
            args: None,
            url: None,
            auth: None,
        });

        manager.save().unwrap();
        assert!(config_path.exists());
    }
}
