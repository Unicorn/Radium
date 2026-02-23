//! OAuth Token component schema
//!
//! The OAuth Token component obtains OAuth2 access tokens from an authorization
//! server. Supports the client credentials, authorization code, and refresh
//! token grant flows.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

use super::behaviors::{ComponentBehaviors, RateLimitConfig};

/// OAuth2 grant type.
///
/// Determines which token-acquisition flow is used against the authorization
/// server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum OAuthGrantType {
    /// Machine-to-machine credential flow — no user interaction required.
    #[default]
    ClientCredentials,
    /// Authorization code flow — requires a redirect URI and authorization code.
    AuthorizationCode,
    /// Refresh token flow — exchanges an existing refresh token for a new
    /// access token.
    RefreshToken,
}

/// OAuth Token component input.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct OAuthTokenInput {
    /// Token endpoint URL for the authorization server.
    #[validate(length(min = 1, message = "token_url must not be empty"))]
    pub token_url: String,

    /// OAuth2 grant type to use when requesting the token.
    #[serde(default)]
    pub grant_type: OAuthGrantType,

    /// OAuth2 client identifier.
    #[validate(length(min = 1, message = "client_id must not be empty"))]
    pub client_id: String,

    /// Secret reference for the client secret
    /// (e.g. `"${{ secrets.OAUTH_CLIENT_SECRET }}"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_secret_ref: Option<String>,

    /// Space-delimited list of requested scopes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,

    /// Redirect URI registered with the authorization server.
    /// Required for the authorization code flow.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redirect_uri: Option<String>,

    /// Authorization code received from the authorization server.
    /// Required for the authorization code flow.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    /// Secret reference for the refresh token
    /// (e.g. `"${{ secrets.OAUTH_REFRESH_TOKEN }}"`).
    /// Required for the refresh token flow.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_token_ref: Option<String>,

    /// Additional parameters to include in the token request body.
    #[serde(default)]
    pub additional_params: HashMap<String, String>,

    /// Shared component behaviors (retry, rate limit, timeout, etc.).
    #[serde(default = "oauth_token_default_behaviors")]
    #[validate(nested)]
    pub behaviors: ComponentBehaviors,
}

fn oauth_token_default_behaviors() -> ComponentBehaviors {
    ComponentBehaviors {
        timeout_ms: 30_000,
        rate_limit: RateLimitConfig {
            requests_per_second: 5,
            burst: 10,
            ..Default::default()
        },
        ..Default::default()
    }
}

impl Default for OAuthTokenInput {
    fn default() -> Self {
        Self {
            token_url: String::new(),
            grant_type: OAuthGrantType::default(),
            client_id: String::new(),
            client_secret_ref: None,
            scope: None,
            redirect_uri: None,
            code: None,
            refresh_token_ref: None,
            additional_params: HashMap::new(),
            behaviors: oauth_token_default_behaviors(),
        }
    }
}

/// OAuth Token component output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OAuthTokenOutput {
    /// The bearer token issued by the authorization server.
    pub access_token: String,

    /// Token type — typically `"Bearer"`.
    pub token_type: String,

    /// Lifetime of the access token in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_in: Option<u64>,

    /// Refresh token that can be used to obtain new access tokens.
    /// Only present when the authorization server issues one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,

    /// Scopes actually granted by the authorization server.
    /// May differ from the requested scopes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

impl Default for OAuthTokenOutput {
    fn default() -> Self {
        Self {
            access_token: String::new(),
            token_type: "Bearer".to_string(),
            expires_in: None,
            refresh_token: None,
            scope: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_with_defaults() {
        let input = OAuthTokenInput::default();
        assert!(input.token_url.is_empty());
        assert_eq!(input.grant_type, OAuthGrantType::ClientCredentials);
        assert!(input.client_id.is_empty());
        assert!(input.client_secret_ref.is_none());
        assert!(input.scope.is_none());
        assert!(input.redirect_uri.is_none());
        assert!(input.code.is_none());
        assert!(input.refresh_token_ref.is_none());
        assert!(input.additional_params.is_empty());
        assert_eq!(input.behaviors.timeout_ms, 30_000);
        assert_eq!(input.behaviors.rate_limit.requests_per_second, 5);
        assert_eq!(input.behaviors.rate_limit.burst, 10);
    }

    #[test]
    fn test_full_config_deserialization() {
        let yaml = r#"
token_url: "https://auth.example.com/oauth2/token"
grant_type: authorization_code
client_id: "my-client-id"
client_secret_ref: "${{ secrets.OAUTH_CLIENT_SECRET }}"
scope: "read write"
redirect_uri: "https://app.example.com/callback"
code: "auth-code-abc123"
additional_params:
  audience: "https://api.example.com"
behaviors:
  timeout_ms: 15000
"#;
        let input: OAuthTokenInput = serde_yaml::from_str(yaml).expect("deserialize");
        assert_eq!(input.token_url, "https://auth.example.com/oauth2/token");
        assert_eq!(input.grant_type, OAuthGrantType::AuthorizationCode);
        assert_eq!(input.client_id, "my-client-id");
        assert_eq!(
            input.client_secret_ref,
            Some("${{ secrets.OAUTH_CLIENT_SECRET }}".to_string())
        );
        assert_eq!(input.scope, Some("read write".to_string()));
        assert_eq!(
            input.redirect_uri,
            Some("https://app.example.com/callback".to_string())
        );
        assert_eq!(input.code, Some("auth-code-abc123".to_string()));
        assert!(input.refresh_token_ref.is_none());
        assert_eq!(
            input.additional_params.get("audience").map(String::as_str),
            Some("https://api.example.com")
        );
        assert_eq!(input.behaviors.timeout_ms, 15_000);
    }

    #[test]
    fn test_output_serialize_deserialize() {
        let output = OAuthTokenOutput {
            access_token: "eyJhbGciOiJSUzI1NiJ9.payload.sig".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: Some(3600),
            refresh_token: Some("rt-xyz789".to_string()),
            scope: Some("read write".to_string()),
        };
        let yaml = serde_yaml::to_string(&output).expect("serialize");
        let restored: OAuthTokenOutput = serde_yaml::from_str(&yaml).expect("deserialize");
        assert_eq!(restored.access_token, output.access_token);
        assert_eq!(restored.token_type, output.token_type);
        assert_eq!(restored.expires_in, output.expires_in);
        assert_eq!(restored.refresh_token, output.refresh_token);
        assert_eq!(restored.scope, output.scope);
    }

    #[test]
    fn test_grant_type_default() {
        let grant_type = OAuthGrantType::default();
        assert_eq!(grant_type, OAuthGrantType::ClientCredentials);

        let serialized = serde_json::to_string(&grant_type).expect("serialize");
        assert_eq!(serialized, "\"client_credentials\"");

        let auth_code = OAuthGrantType::AuthorizationCode;
        let serialized_auth = serde_json::to_string(&auth_code).expect("serialize auth_code");
        assert_eq!(serialized_auth, "\"authorization_code\"");

        let refresh = OAuthGrantType::RefreshToken;
        let serialized_refresh = serde_json::to_string(&refresh).expect("serialize refresh");
        assert_eq!(serialized_refresh, "\"refresh_token\"");
    }
}
