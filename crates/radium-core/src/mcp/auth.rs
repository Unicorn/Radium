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
        Self { storage_dir, tokens: HashMap::new() }
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
        let content = serde_json::to_string_pretty(&token).map_err(|e| McpError::Json(e))?;

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

    /// Refresh a token using OAuth 2.0 refresh token flow.
    ///
    /// # Arguments
    /// * `server_name` - Name of the server to refresh token for
    /// * `auth_config` - Authentication configuration containing OAuth parameters
    ///
    /// # Errors
    ///
    /// Returns an error if token refresh fails.
    pub async fn refresh_token(
        &mut self,
        server_name: &str,
        auth_config: &crate::mcp::McpAuthConfig,
    ) -> Result<()> {
        // Get the current token
        let token = self
            .get_token(server_name)
            .ok_or_else(|| {
                McpError::Authentication(format!("No token found for server: {}", server_name))
            })?
            .clone();

        // Get refresh token
        let refresh_token = token.refresh_token.clone().ok_or_else(|| {
            McpError::Authentication(format!(
                "No refresh token available for server: {}",
                server_name
            ))
        })?;

        // Get OAuth token endpoint from auth config
        let token_url = auth_config
            .params
            .get("token_url")
            .ok_or_else(|| {
                McpError::Authentication(
                    "OAuth token_url not found in auth configuration".to_string(),
                )
            })?
            .clone();

        // Get client_id and client_secret (optional, some providers don't require them for refresh)
        let client_id = auth_config.params.get("client_id").cloned();
        let client_secret = auth_config.params.get("client_secret").cloned();

        // Build OAuth 2.0 token refresh request
        let mut form_params = std::collections::HashMap::new();
        form_params.insert("grant_type", "refresh_token");
        form_params.insert("refresh_token", refresh_token.as_str());

        // Add client credentials if provided (some OAuth providers require this)
        if let Some(ref cid) = client_id {
            form_params.insert("client_id", cid.as_str());
        }
        if let Some(ref cs) = client_secret {
            form_params.insert("client_secret", cs.as_str());
        }

        // Make HTTP POST request to token endpoint
        let client = reqwest::Client::new();
        let response = client
            .post(&token_url)
            .form(&form_params)
            .send()
            .await
            .map_err(|e| {
                McpError::Authentication(format!("Failed to send token refresh request: {}", e))
            })?;

        // Check response status
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(McpError::Authentication(format!(
                "Token refresh failed with status {}: {}",
                status, error_text
            )));
        }

        // Parse OAuth 2.0 token response
        let token_response: serde_json::Value = response.json().await.map_err(|e| {
            McpError::Authentication(format!("Failed to parse token response: {}", e))
        })?;

        // Extract token fields
        let access_token = token_response
            .get("access_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                McpError::Authentication("Token response missing 'access_token' field".to_string())
            })?
            .to_string();

        let token_type = token_response
            .get("token_type")
            .and_then(|v| v.as_str())
            .unwrap_or("Bearer")
            .to_string();

        // Refresh token may be returned in response (some providers rotate it)
        let new_refresh_token = token_response
            .get("refresh_token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| token.refresh_token);

        // Calculate expiration time from expires_in (seconds)
        let expires_at = token_response
            .get("expires_in")
            .and_then(|v| v.as_u64())
            .map(|expires_in| {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    + expires_in
            })
            .or_else(|| token.expires_at);

        let scope = token_response
            .get("scope")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| token.scope);

        // Create new token
        let new_token = OAuthToken {
            access_token,
            token_type,
            refresh_token: new_refresh_token,
            expires_at,
            scope,
        };

        // Save the refreshed token
        self.save_token(server_name, new_token)?;

        Ok(())
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
        let future_time =
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
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

    #[tokio::test]
    async fn test_oauth_token_refresh_no_token() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = OAuthTokenManager::new(temp_dir.path().to_path_buf());

        let auth_config = crate::mcp::McpAuthConfig {
            auth_type: "oauth".to_string(),
            params: {
                let mut params = std::collections::HashMap::new();
                params.insert("token_url".to_string(), "https://example.com/token".to_string());
                params
            },
        };

        let result = manager.refresh_token("nonexistent-server", &auth_config).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No token found for server"));
    }

    #[tokio::test]
    async fn test_oauth_token_refresh_no_refresh_token() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = OAuthTokenManager::new(temp_dir.path().to_path_buf());

        let token = OAuthToken {
            access_token: "test_token".to_string(),
            token_type: "Bearer".to_string(),
            refresh_token: None, // No refresh token
            expires_at: None,
            scope: None,
        };

        manager.tokens.insert("test-server".to_string(), token);

        let auth_config = crate::mcp::McpAuthConfig {
            auth_type: "oauth".to_string(),
            params: {
                let mut params = std::collections::HashMap::new();
                params.insert("token_url".to_string(), "https://example.com/token".to_string());
                params
            },
        };

        let result = manager.refresh_token("test-server", &auth_config).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No refresh token available"));
    }

    #[tokio::test]
    async fn test_oauth_token_refresh_no_token_url() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = OAuthTokenManager::new(temp_dir.path().to_path_buf());

        let token = OAuthToken {
            access_token: "test_token".to_string(),
            token_type: "Bearer".to_string(),
            refresh_token: Some("refresh_token".to_string()),
            expires_at: None,
            scope: None,
        };

        manager.tokens.insert("test-server".to_string(), token);

        let auth_config = crate::mcp::McpAuthConfig {
            auth_type: "oauth".to_string(),
            params: std::collections::HashMap::new(), // No token_url
        };

        let result = manager.refresh_token("test-server", &auth_config).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("token_url not found"));
    }
}
