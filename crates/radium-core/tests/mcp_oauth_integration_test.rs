#![cfg(feature = "server")]

//! Integration tests for MCP OAuth authentication flow.

use radium_core::mcp::auth::{OAuthToken, OAuthTokenManager};
use radium_core::mcp::McpAuthConfig;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::TempDir;

/// Mock OAuth server for testing
struct MockOAuthServer {
    port: u16,
    server_handle: Option<tokio::task::JoinHandle<()>>,
}

impl MockOAuthServer {
    async fn start() -> Self {
        use http_body_util::BodyExt;
        use hyper::server::conn::http1;
        use hyper::service::service_fn;
        use hyper::{Method, Request, Response, StatusCode};
        use hyper_util::rt::TokioIo;
        use std::convert::Infallible;

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let port = addr.port();

        let server_handle = tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        let io = TokioIo::new(stream);
                        let service = service_fn(|req: Request<hyper::body::Incoming>| async move {
                            let path = req.uri().path();
                            let method = req.method();

                            if method == Method::POST && path == "/token" {
                                // Parse form data
                                let body_bytes = req.collect().await.unwrap().to_bytes();
                                let body_str = String::from_utf8_lossy(&body_bytes);
                                
                                // Check for refresh_token
                                if body_str.contains("refresh_token") {
                                    // Return new token
                                    let expires_in = 3600u64;

                                    let response = serde_json::json!({
                                        "access_token": "new_access_token_12345",
                                        "token_type": "Bearer",
                                        "refresh_token": "new_refresh_token_67890",
                                        "expires_in": expires_in,
                                        "scope": "read write"
                                    });

                                    Ok::<_, Infallible>(Response::builder()
                                        .status(StatusCode::OK)
                                        .header("Content-Type", "application/json")
                                        .body(serde_json::to_string(&response).unwrap())
                                        .unwrap())
                                } else {
                                    // Invalid request
                                    Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body("Invalid request".to_string())
                                        .unwrap())
                                }
                            } else {
                                Ok(Response::builder()
                                    .status(StatusCode::NOT_FOUND)
                                    .body("Not found".to_string())
                                    .unwrap())
                            }
                        });

                        tokio::spawn(async move {
                            if let Err(err) = http1::Builder::new()
                                .serve_connection(io, service)
                                .await
                            {
                                eprintln!("Error serving connection: {:?}", err);
                            }
                        });
                    }
                    Err(e) => {
                        eprintln!("Error accepting connection: {:?}", e);
                        break;
                    }
                }
            }
        });

        // Give server time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        Self { port, server_handle: Some(server_handle) }
    }

    fn token_url(&self) -> String {
        format!("http://127.0.0.1:{}/token", self.port)
    }

    async fn stop(self) {
        if let Some(handle) = self.server_handle {
            handle.abort();
        }
    }
}

#[tokio::test]
async fn test_oauth_token_storage_and_retrieval() {
    let temp_dir = TempDir::new().unwrap();
    let mut manager = OAuthTokenManager::new(temp_dir.path().to_path_buf());

    let token = OAuthToken {
        access_token: "test_access_token".to_string(),
        token_type: "Bearer".to_string(),
        refresh_token: Some("test_refresh_token".to_string()),
        expires_at: Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + 3600,
        ),
        scope: Some("read write".to_string()),
    };

    // Save token
    manager.save_token("test-server", token.clone()).unwrap();

    // Verify token was saved to disk
    let token_file = temp_dir.path().join("test-server.json");
    assert!(token_file.exists());

    // Load tokens from disk
    let mut new_manager = OAuthTokenManager::new(temp_dir.path().to_path_buf());
    new_manager.load_tokens().unwrap();

    // Verify token was loaded
    let retrieved = new_manager.get_token("test-server");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().access_token, "test_access_token");
    assert_eq!(retrieved.unwrap().refresh_token, Some("test_refresh_token".to_string()));
}

#[tokio::test]
async fn test_oauth_token_expiration_detection() {
    let temp_dir = TempDir::new().unwrap();
    let mut manager = OAuthTokenManager::new(temp_dir.path().to_path_buf());

    // Create expired token
    let expired_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - 3600; // 1 hour ago

    let expired_token = OAuthToken {
        access_token: "expired_token".to_string(),
        token_type: "Bearer".to_string(),
        refresh_token: Some("refresh_token".to_string()),
        expires_at: Some(expired_time),
        scope: None,
    };

    manager.save_token("expired-server", expired_token).unwrap();
    assert!(manager.is_token_expired("expired-server"));

    // Create non-expired token
    let future_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 3600; // 1 hour from now

    let valid_token = OAuthToken {
        access_token: "valid_token".to_string(),
        token_type: "Bearer".to_string(),
        refresh_token: Some("refresh_token".to_string()),
        expires_at: Some(future_time),
        scope: None,
    };

    manager.save_token("valid-server", valid_token).unwrap();
    assert!(!manager.is_token_expired("valid-server"));
}

#[tokio::test]
async fn test_oauth_token_refresh_flow() {
    let temp_dir = TempDir::new().unwrap();
    let mut manager = OAuthTokenManager::new(temp_dir.path().to_path_buf());

    // Start mock OAuth server
    let server = MockOAuthServer::start().await;
    let token_url = server.token_url();

    // Create initial token with refresh token
    let initial_token = OAuthToken {
        access_token: "old_access_token".to_string(),
        token_type: "Bearer".to_string(),
        refresh_token: Some("old_refresh_token".to_string()),
        expires_at: Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                - 100, // Expired
        ),
        scope: Some("read".to_string()),
    };

    manager.save_token("test-server", initial_token).unwrap();

    // Create auth config
    let mut auth_params = HashMap::new();
    auth_params.insert("token_url".to_string(), token_url);
    auth_params.insert("client_id".to_string(), "test_client_id".to_string());
    auth_params.insert("client_secret".to_string(), "test_client_secret".to_string());

    let auth_config = McpAuthConfig {
        auth_type: "oauth".to_string(),
        params: auth_params,
    };

    // Refresh token
    manager.refresh_token("test-server", &auth_config).await.unwrap();

    // Verify new token was saved
    let new_token = manager.get_token("test-server").unwrap();
    assert_eq!(new_token.access_token, "new_access_token_12345");
    assert_eq!(new_token.refresh_token, Some("new_refresh_token_67890".to_string()));
    assert_eq!(new_token.scope, Some("read write".to_string()));
    assert!(new_token.expires_at.is_some());
    assert!(!manager.is_token_expired("test-server"));

    server.stop().await;
}

#[tokio::test]
async fn test_oauth_token_refresh_error_scenarios() {
    let temp_dir = TempDir::new().unwrap();
    let mut manager = OAuthTokenManager::new(temp_dir.path().to_path_buf());

    // Test: No token found
    let auth_config = McpAuthConfig {
        auth_type: "oauth".to_string(),
        params: {
            let mut params = HashMap::new();
            params.insert("token_url".to_string(), "https://example.com/token".to_string());
            params
        },
    };

    let result = manager.refresh_token("nonexistent-server", &auth_config).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No token found"));

    // Test: No refresh token
    let token = OAuthToken {
        access_token: "token".to_string(),
        token_type: "Bearer".to_string(),
        refresh_token: None,
        expires_at: None,
        scope: None,
    };
    manager.save_token("no-refresh-server", token).unwrap();

    let result = manager.refresh_token("no-refresh-server", &auth_config).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No refresh token available"));

    // Test: No token_url in config
    let token_with_refresh = OAuthToken {
        access_token: "token".to_string(),
        token_type: "Bearer".to_string(),
        refresh_token: Some("refresh".to_string()),
        expires_at: None,
        scope: None,
    };
    manager.save_token("no-url-server", token_with_refresh).unwrap();

    let bad_auth_config = McpAuthConfig {
        auth_type: "oauth".to_string(),
        params: HashMap::new(), // No token_url
    };

    let result = manager.refresh_token("no-url-server", &bad_auth_config).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("token_url not found"));
}

#[tokio::test]
async fn test_oauth_token_refresh_network_error() {
    let temp_dir = TempDir::new().unwrap();
    let mut manager = OAuthTokenManager::new(temp_dir.path().to_path_buf());

    let token = OAuthToken {
        access_token: "token".to_string(),
        token_type: "Bearer".to_string(),
        refresh_token: Some("refresh".to_string()),
        expires_at: None,
        scope: None,
    };
    manager.save_token("test-server", token).unwrap();

    // Use invalid URL to simulate network error
    let mut auth_params = HashMap::new();
    auth_params.insert("token_url".to_string(), "http://192.0.2.0:9999/token".to_string()); // Invalid address

    let auth_config = McpAuthConfig {
        auth_type: "oauth".to_string(),
        params: auth_params,
    };

    let result = manager.refresh_token("test-server", &auth_config).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Failed to send token refresh request"));
}

#[tokio::test]
async fn test_oauth_token_file_permissions() {
    let temp_dir = TempDir::new().unwrap();
    let mut manager = OAuthTokenManager::new(temp_dir.path().to_path_buf());

    let token = OAuthToken {
        access_token: "test_token".to_string(),
        token_type: "Bearer".to_string(),
        refresh_token: None,
        expires_at: None,
        scope: None,
    };

    manager.save_token("test-server", token).unwrap();

    // Verify file was created
    let token_file = temp_dir.path().join("test-server.json");
    assert!(token_file.exists());

    // On Unix, verify file permissions are restrictive (0600)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(&token_file).unwrap();
        let permissions = metadata.permissions();
        let mode = permissions.mode();
        // Check that file is readable/writable by owner only (0600 = 0o600)
        assert_eq!(mode & 0o777, 0o600);
    }
}

#[tokio::test]
async fn test_oauth_token_persistence_across_restarts() {
    let temp_dir = TempDir::new().unwrap();

    // First manager instance - save token
    let mut manager1 = OAuthTokenManager::new(temp_dir.path().to_path_buf());
    let token = OAuthToken {
        access_token: "persistent_token".to_string(),
        token_type: "Bearer".to_string(),
        refresh_token: Some("persistent_refresh".to_string()),
        expires_at: Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + 7200,
        ),
        scope: Some("read".to_string()),
    };
    manager1.save_token("persistent-server", token).unwrap();

    // Simulate restart - create new manager and load tokens
    let mut manager2 = OAuthTokenManager::new(temp_dir.path().to_path_buf());
    manager2.load_tokens().unwrap();

    // Verify token was loaded
    let retrieved = manager2.get_token("persistent-server");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().access_token, "persistent_token");
    assert_eq!(retrieved.unwrap().refresh_token, Some("persistent_refresh".to_string()));
    assert_eq!(retrieved.unwrap().scope, Some("read".to_string()));
}

#[tokio::test]
async fn test_oauth_token_refresh_with_client_credentials() {
    let temp_dir = TempDir::new().unwrap();
    let mut manager = OAuthTokenManager::new(temp_dir.path().to_path_buf());

    // Start mock server
    let server = MockOAuthServer::start().await;
    let token_url = server.token_url();

    let token = OAuthToken {
        access_token: "old_token".to_string(),
        token_type: "Bearer".to_string(),
        refresh_token: Some("refresh_token".to_string()),
        expires_at: Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                - 100,
        ),
        scope: None,
    };
    manager.save_token("test-server", token).unwrap();

    // Auth config with client credentials
    let mut auth_params = HashMap::new();
    auth_params.insert("token_url".to_string(), token_url);
    auth_params.insert("client_id".to_string(), "test_client".to_string());
    auth_params.insert("client_secret".to_string(), "test_secret".to_string());

    let auth_config = McpAuthConfig {
        auth_type: "oauth".to_string(),
        params: auth_params,
    };

    // Refresh should succeed with client credentials
    manager.refresh_token("test-server", &auth_config).await.unwrap();

    let new_token = manager.get_token("test-server").unwrap();
    assert_eq!(new_token.access_token, "new_access_token_12345");

    server.stop().await;
}

