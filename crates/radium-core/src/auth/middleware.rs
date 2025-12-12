//! Authentication middleware for gRPC.

use crate::auth::token::TokenStore;
use anyhow::Result;
use std::net::IpAddr;
use std::path::Path;
use std::sync::Arc;
use tonic::{Request, Status};
use tracing::{debug, warn};

/// Authentication configuration.
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// Whether authentication is enabled
    pub enable_auth: bool,
    /// Whether to enforce localhost-only connections when auth is disabled
    pub localhost_only: bool,
    /// Token store for token validation
    pub token_store: Option<Arc<TokenStore>>,
}

impl AuthConfig {
    /// Create a new auth configuration.
    pub fn new(workspace_root: &Path, enable_auth: bool, localhost_only: bool) -> Result<Self> {
        let token_store = if enable_auth {
            Some(Arc::new(TokenStore::new(workspace_root)?))
        } else {
            None
        };

        Ok(Self {
            enable_auth,
            localhost_only,
            token_store,
        })
    }
}

/// Check if a peer address is localhost.
fn is_localhost(peer_addr: &str) -> bool {
    // Parse the peer address (format: "ip:port")
    if let Some(ip_part) = peer_addr.split(':').next() {
        if let Ok(ip) = ip_part.parse::<IpAddr>() {
            return matches!(ip, IpAddr::V4(addr) if addr.is_loopback())
                || matches!(ip, IpAddr::V6(addr) if addr.is_loopback());
        }
    }
    false
}

/// Authenticate a gRPC request.
///
/// # Arguments
/// * `request` - The gRPC request
/// * `config` - Authentication configuration
///
/// # Returns
/// Ok(()) if authenticated, Err(Status) if authentication fails
pub fn authenticate_request<T>(request: &Request<T>, config: &AuthConfig) -> Result<(), Status> {
    // Get peer address
    let peer_addr = request
        .remote_addr()
        .map(|addr| addr.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    debug!(peer_addr = %peer_addr, "Authenticating request");

    // If localhost-only mode is enabled and auth is disabled, check peer address
    if !config.enable_auth && config.localhost_only {
        if !is_localhost(&peer_addr) {
            return Err(Status::permission_denied(
                "Remote connections are not allowed. This daemon only accepts localhost connections.",
            ));
        }
        return Ok(()); // Localhost connection, no auth required
    }

    // If auth is enabled, validate token
    if config.enable_auth {
        let token_store = config
            .token_store
            .as_ref()
            .ok_or_else(|| Status::internal("Token store not configured"))?;

        // Extract token from metadata
        let metadata = request.metadata();
        let auth_header = metadata
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| Status::unauthenticated("Missing authorization header"))?;

        // Parse "Bearer <token>" format
        let token_parts: Vec<&str> = auth_header.split_whitespace().collect();
        if token_parts.len() != 2 || token_parts[0] != "Bearer" {
            return Err(Status::unauthenticated(
                "Invalid authorization header format. Expected: Bearer <token>",
            ));
        }

        let token_str = token_parts[1];
        // Token format: "<token_id>:<secret>"
        let token_parts: Vec<&str> = token_str.split(':').collect();
        if token_parts.len() != 2 {
            return Err(Status::unauthenticated("Invalid token format"));
        }

        let token_id = token_parts[0];
        let secret = token_parts[1];

        // Validate token
        match token_store.validate_token(token_id, secret) {
            Ok(true) => {
                debug!(token_id = %token_id, "Token validated successfully");
                Ok(())
            }
            Ok(false) => Err(Status::unauthenticated("Invalid token")),
            Err(e) => {
                warn!("Token validation error: {}", e);
                Err(Status::internal("Token validation failed"))
            }
        }
    } else {
        // No auth required
        Ok(())
    }
}
