//! HTTP Request component schema
//!
//! The HTTP Request component makes HTTP requests to external services.
//! Supports various methods, authentication types, and body formats.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

/// HTTP methods
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    #[default]
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

impl HttpMethod {
    /// Convert to TypeScript representation
    pub fn to_typescript(&self) -> &'static str {
        match self {
            HttpMethod::Get => "'GET'",
            HttpMethod::Post => "'POST'",
            HttpMethod::Put => "'PUT'",
            HttpMethod::Patch => "'PATCH'",
            HttpMethod::Delete => "'DELETE'",
            HttpMethod::Head => "'HEAD'",
            HttpMethod::Options => "'OPTIONS'",
        }
    }
}

/// Body content types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum BodyType {
    /// JSON body (application/json)
    #[default]
    Json,
    /// Multipart form data
    FormData,
    /// URL-encoded form
    FormUrlencoded,
    /// Plain text
    Text,
    /// Binary data
    Binary,
    /// No body
    None,
}

impl BodyType {
    /// Get content type header value
    pub fn content_type(&self) -> Option<&'static str> {
        match self {
            BodyType::Json => Some("application/json"),
            BodyType::FormData => Some("multipart/form-data"),
            BodyType::FormUrlencoded => Some("application/x-www-form-urlencoded"),
            BodyType::Text => Some("text/plain"),
            BodyType::Binary => Some("application/octet-stream"),
            BodyType::None => None,
        }
    }
}

/// Authentication types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum AuthType {
    /// No authentication
    #[default]
    None,
    /// Basic authentication
    Basic,
    /// Bearer token
    Bearer,
    /// API key
    ApiKey,
    /// OAuth2
    OAuth2,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AuthConfig {
    /// Authentication type
    pub auth_type: AuthType,

    /// Username for basic auth
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    /// Password for basic auth
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,

    /// Token for bearer auth
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,

    /// Header name for API key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key_header: Option<String>,

    /// API key value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key_value: Option<String>,
}

impl AuthConfig {
    /// Create no-auth config
    pub fn none() -> Self {
        Self::default()
    }

    /// Create basic auth config
    pub fn basic(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            auth_type: AuthType::Basic,
            username: Some(username.into()),
            password: Some(password.into()),
            ..Default::default()
        }
    }

    /// Create bearer token config
    pub fn bearer(token: impl Into<String>) -> Self {
        Self {
            auth_type: AuthType::Bearer,
            token: Some(token.into()),
            ..Default::default()
        }
    }

    /// Create API key config
    pub fn api_key(header: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            auth_type: AuthType::ApiKey,
            api_key_header: Some(header.into()),
            api_key_value: Some(value.into()),
            ..Default::default()
        }
    }

    /// Validate auth configuration
    pub fn validate_config(&self) -> Result<(), String> {
        match self.auth_type {
            AuthType::None => Ok(()),
            AuthType::Basic => {
                if self.username.is_none() || self.password.is_none() {
                    Err("Basic auth requires username and password".to_string())
                } else {
                    Ok(())
                }
            }
            AuthType::Bearer => {
                if self.token.is_none() {
                    Err("Bearer auth requires token".to_string())
                } else {
                    Ok(())
                }
            }
            AuthType::ApiKey => {
                if self.api_key_header.is_none() || self.api_key_value.is_none() {
                    Err("API key auth requires header and value".to_string())
                } else {
                    Ok(())
                }
            }
            AuthType::OAuth2 => {
                // OAuth2 requires more complex validation
                Ok(())
            }
        }
    }
}

/// HTTP Request component input
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct HttpRequestInput {
    /// URL to request
    #[validate(url(message = "Invalid URL"))]
    pub url: String,

    /// HTTP method
    #[serde(default)]
    pub method: HttpMethod,

    /// Request headers
    #[serde(default)]
    pub headers: HashMap<String, String>,

    /// Query parameters
    #[serde(default)]
    pub query_params: HashMap<String, String>,

    /// Request body
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_json::Value>,

    /// Body content type
    #[serde(default)]
    pub body_type: BodyType,

    /// Authentication configuration
    #[serde(default)]
    pub auth: AuthConfig,

    /// Request timeout in milliseconds
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,

    /// Whether to follow redirects
    #[serde(default)]
    pub follow_redirects: bool,

    /// Whether to validate SSL certificates
    #[serde(default = "default_true")]
    pub validate_ssl: bool,

    /// Expected status codes (empty means any 2xx)
    #[serde(default)]
    pub expected_status: Vec<u16>,
}

fn default_timeout() -> u64 {
    30000
}

fn default_true() -> bool {
    true
}

impl HttpRequestInput {
    /// Create a GET request
    pub fn get(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            method: HttpMethod::Get,
            headers: HashMap::new(),
            query_params: HashMap::new(),
            body: None,
            body_type: BodyType::None,
            auth: AuthConfig::none(),
            timeout_ms: default_timeout(),
            follow_redirects: false,
            validate_ssl: true,
            expected_status: Vec::new(),
        }
    }

    /// Create a POST request
    pub fn post(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            method: HttpMethod::Post,
            headers: HashMap::new(),
            query_params: HashMap::new(),
            body: None,
            body_type: BodyType::Json,
            auth: AuthConfig::none(),
            timeout_ms: default_timeout(),
            follow_redirects: false,
            validate_ssl: true,
            expected_status: Vec::new(),
        }
    }

    /// Create a PUT request
    pub fn put(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            method: HttpMethod::Put,
            ..Self::post("")
        }
    }

    /// Create a PATCH request
    pub fn patch(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            method: HttpMethod::Patch,
            ..Self::post("")
        }
    }

    /// Create a DELETE request
    pub fn delete(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            method: HttpMethod::Delete,
            ..Self::get("")
        }
    }

    /// Add a header
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Add a query parameter
    pub fn with_query(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query_params.insert(key.into(), value.into());
        self
    }

    /// Set JSON body
    pub fn with_json_body(mut self, body: serde_json::Value) -> Self {
        self.body = Some(body);
        self.body_type = BodyType::Json;
        self
    }

    /// Set authentication
    pub fn with_auth(mut self, auth: AuthConfig) -> Self {
        self.auth = auth;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }

    /// Enable following redirects
    pub fn follow_redirects(mut self) -> Self {
        self.follow_redirects = true;
        self
    }

    /// Set expected status codes
    pub fn expect_status(mut self, codes: Vec<u16>) -> Self {
        self.expected_status = codes;
        self
    }
}

impl Default for HttpRequestInput {
    fn default() -> Self {
        Self::get("http://example.com")
    }
}

/// HTTP Request component output
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpRequestOutput {
    /// HTTP status code
    pub status: u16,

    /// Status text
    pub status_text: String,

    /// Response headers
    pub headers: HashMap<String, String>,

    /// Response body
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_json::Value>,

    /// Duration in milliseconds
    pub duration_ms: u64,

    /// Whether the request was successful (status in expected range)
    pub success: bool,
}

impl HttpRequestOutput {
    /// Create a successful response
    pub fn success(
        status: u16,
        status_text: impl Into<String>,
        body: serde_json::Value,
        duration_ms: u64,
    ) -> Self {
        Self {
            status,
            status_text: status_text.into(),
            headers: HashMap::new(),
            body: Some(body),
            duration_ms,
            success: true,
        }
    }

    /// Create a failed response
    pub fn failure(
        status: u16,
        status_text: impl Into<String>,
        duration_ms: u64,
    ) -> Self {
        Self {
            status,
            status_text: status_text.into(),
            headers: HashMap::new(),
            body: None,
            duration_ms,
            success: false,
        }
    }

    /// Add response headers
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }
}

impl Default for HttpRequestOutput {
    fn default() -> Self {
        Self::success(200, "OK", serde_json::Value::Null, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_method_serialization() {
        assert_eq!(serde_json::to_string(&HttpMethod::Get).unwrap(), "\"GET\"");
        assert_eq!(serde_json::to_string(&HttpMethod::Post).unwrap(), "\"POST\"");
    }

    #[test]
    fn test_body_type_content_type() {
        assert_eq!(BodyType::Json.content_type(), Some("application/json"));
        assert_eq!(BodyType::None.content_type(), None);
    }

    #[test]
    fn test_auth_config_basic() {
        let auth = AuthConfig::basic("user", "pass");
        assert_eq!(auth.auth_type, AuthType::Basic);
        assert!(auth.validate_config().is_ok());
    }

    #[test]
    fn test_auth_config_bearer() {
        let auth = AuthConfig::bearer("token123");
        assert_eq!(auth.auth_type, AuthType::Bearer);
        assert!(auth.validate_config().is_ok());
    }

    #[test]
    fn test_auth_config_api_key() {
        let auth = AuthConfig::api_key("X-API-Key", "secret");
        assert_eq!(auth.auth_type, AuthType::ApiKey);
        assert!(auth.validate_config().is_ok());
    }

    #[test]
    fn test_auth_config_validation() {
        let auth = AuthConfig {
            auth_type: AuthType::Basic,
            username: None,
            password: None,
            ..Default::default()
        };
        assert!(auth.validate_config().is_err());
    }

    #[test]
    fn test_http_request_get() {
        let request = HttpRequestInput::get("https://api.example.com/users")
            .with_query("page", "1")
            .with_header("Accept", "application/json");

        assert_eq!(request.method, HttpMethod::Get);
        assert!(request.query_params.contains_key("page"));
        assert!(request.headers.contains_key("Accept"));
    }

    #[test]
    fn test_http_request_post() {
        let request = HttpRequestInput::post("https://api.example.com/users")
            .with_json_body(serde_json::json!({"name": "John"}))
            .with_auth(AuthConfig::bearer("token"));

        assert_eq!(request.method, HttpMethod::Post);
        assert!(request.body.is_some());
        assert_eq!(request.auth.auth_type, AuthType::Bearer);
    }

    #[test]
    fn test_http_request_serialization() {
        let request = HttpRequestInput::get("https://example.com")
            .with_timeout(5000)
            .follow_redirects();

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("url"));
        assert!(json.contains("method"));
        assert!(json.contains("timeoutMs"));
    }

    #[test]
    fn test_http_response_success() {
        let response = HttpRequestOutput::success(200, "OK", serde_json::json!({"id": 1}), 150);
        assert!(response.success);
        assert_eq!(response.status, 200);
    }

    #[test]
    fn test_http_response_failure() {
        let response = HttpRequestOutput::failure(404, "Not Found", 50);
        assert!(!response.success);
        assert_eq!(response.status, 404);
    }
}
