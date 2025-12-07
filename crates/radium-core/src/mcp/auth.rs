//! OAuth authentication for MCP servers.

use crate::mcp::{McpError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// OAuth token storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    /// Access token.
    pub access_token: String,
    /// Token type (usually "Bearer").
    pub token_type: String,
    /// Refresh token (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    /// Token expiration timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<u64>,
    /// Scope (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

/// OAuth token manager.
pub struct OAuthTokenManager {
    /// Token storage directory.
    storage_dir: PathBuf,
    /// In-memory token cache.
    tokens: HashMap<String, OAuthToken>,
}

impl OAuthTokenManager {
    /// Create a new OAuth token manager.
    pub fn new(storage_dir: PathBuf) -> Self {
        Self {
            storage_dir,
            tokens: HashMap::new(),
        }
    }

    /// Get the default storage directory.
    pub fn default_storage_dir() -> PathBuf {
        #[allow(clippy::disallowed_methods)]
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".radium").join("mcp_tokens")
    }

    /// Load tokens from storage.
    ///
    /// # Errors
    ///
    /// Returns an error if token loading fails.
    pub fn load_tokens(&mut self) -> Result<()> {
        if !self.storage_dir.exists() {
            std::fs::create_dir_all(&self.storage_dir).map_err(|e| {
                McpError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to create token storage directory: {}", e),
                ))
            })?;
            return Ok(());
        }

        for entry in std::fs::read_dir(&self.storage_dir).map_err(|e| {
            McpError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to read token storage directory: {}", e),
            ))
        })? {
            let entry = entry.map_err(|e| {
                McpError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to read directory entry: {}", e),
                ))
            })?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(token) = serde_json::from_str::<OAuthToken>(&content) {
                        let server_name = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown")
                            .to_string();
                        self.tokens.insert(server_name, token);
                    }
                }
            }
        }

        Ok(())
    }

    /// Save a token to storage.
    ///
    /// # Errors
    ///
    /// Returns an error if token saving fails.
    pub fn save_token(&mut self, server_name: &str, token: OAuthToken) -> Result<()> {
        self.tokens.insert(server_name.to_string(), token.clone());

        if !self.storage_dir.exists() {
            std::fs::create_dir_all(&self.storage_dir).map_err(|e| {
                McpError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to create token storage directory: {}", e),
                ))
            })?;
        }

        let token_file = self.storage_dir.join(format!("{}.json", server_name));
        let content = serde_json::to_string_pretty(&token).map_err(|e| {
            McpError::Json(e)
        })?;

        std::fs::write(&token_file, content).map_err(|e| {
            McpError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to write token file: {}", e),
            ))
        })?;

        Ok(())
    }

    /// Get a token for a server.
    pub fn get_token(&self, server_name: &str) -> Option<&OAuthToken> {
        self.tokens.get(server_name)
    }

    /// Check if a token is expired.
    pub fn is_token_expired(&self, server_name: &str) -> bool {
        if let Some(token) = self.get_token(server_name) {
            if let Some(expires_at) = token.expires_at {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                return now >= expires_at;
            }
        }
        false
    }

    /// Refresh a token (placeholder - actual implementation would call OAuth endpoint).
    ///
    /// # Errors
    ///
    /// Returns an error if token refresh fails.
    pub async fn refresh_token(&mut self, _server_name: &str) -> Result<()> {
        // TODO: Implement actual OAuth refresh flow
        Err(McpError::Authentication(
            "Token refresh not yet implemented".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_oauth_token_serialization() {
        let token = OAuthToken {
            access_token: "test_token".to_string(),
            token_type: "Bearer".to_string(),
            refresh_token: Some("refresh_token".to_string()),
            expires_at: Some(1234567890),
            scope: Some("read write".to_string()),
        };

        let json = serde_json::to_string(&token).unwrap();
        assert!(json.contains("test_token"));
        assert!(json.contains("Bearer"));
    }

    #[test]
    fn test_oauth_token_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = OAuthTokenManager::new(temp_dir.path().to_path_buf());
        assert_eq!(manager.tokens.len(), 0);
    }

    #[test]
    fn test_oauth_token_manager_save_and_get() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = OAuthTokenManager::new(temp_dir.path().to_path_buf());

        let token = OAuthToken {
            access_token: "test_token".to_string(),
            token_type: "Bearer".to_string(),
            refresh_token: None,
            expires_at: None,
            scope: None,
        };

        manager.save_token("test-server", token.clone()).unwrap();
        let retrieved = manager.get_token("test-server");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().access_token, "test_token");
    }

    #[test]
    fn test_oauth_token_expiration() {
        let temp_dir = TempDir::new().unwrap();
        let _manager = OAuthTokenManager::new(temp_dir.path().to_path_buf());

        // Token with future expiration
        let future_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600; // 1 hour from now

        let mut manager = OAuthTokenManager::new(temp_dir.path().to_path_buf());
        let token = OAuthToken {
            access_token: "test_token".to_string(),
            token_type: "Bearer".to_string(),
            refresh_token: None,
            expires_at: Some(future_time),
            scope: None,
        };

        manager.tokens.insert("test-server".to_string(), token);
        assert!(!manager.is_token_expired("test-server"));
    }
}

