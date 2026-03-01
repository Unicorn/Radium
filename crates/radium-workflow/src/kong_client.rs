//! Kong Admin API client for dynamic route management.
//!
//! Wraps the Kong Admin API to create and delete services, routes, and plugins
//! so that the workflow service can dynamically expose webhook/trigger endpoints
//! through the API gateway.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for connecting to the Kong Admin API.
#[derive(Debug, Clone)]
pub struct KongConfig {
    /// Base URL of the Kong Admin API (e.g. `http://localhost:8001`).
    pub admin_url: String,
}

impl KongConfig {
    /// Build configuration from environment variables.
    ///
    /// | Variable          | Default                    |
    /// |-------------------|----------------------------|
    /// | `KONG_ADMIN_URL`  | `http://localhost:8001`    |
    pub fn from_env() -> Self {
        Self {
            admin_url: std::env::var("KONG_ADMIN_URL")
                .unwrap_or_else(|_| "http://localhost:8001".to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can occur when interacting with the Kong Admin API.
#[derive(Debug)]
pub enum KongError {
    /// An HTTP-level error from `reqwest`.
    Request(reqwest::Error),
    /// Kong returned a non-success status code.
    Api {
        /// HTTP status code.
        status: u16,
        /// Response body (best-effort).
        body: String,
    },
    /// Failed to deserialise the Kong response.
    Deserialize(String),
}

impl std::fmt::Display for KongError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Request(e) => write!(f, "Kong request error: {e}"),
            Self::Api { status, body } => {
                write!(f, "Kong API error (HTTP {status}): {body}")
            }
            Self::Deserialize(msg) => write!(f, "Kong deserialization error: {msg}"),
        }
    }
}

impl std::error::Error for KongError {}

impl From<reqwest::Error> for KongError {
    fn from(err: reqwest::Error) -> Self {
        Self::Request(err)
    }
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

/// Body for `POST /services`.
#[derive(Debug, Clone, Serialize)]
pub struct CreateServiceRequest {
    /// Human-readable service name (must be unique in Kong).
    pub name: String,
    /// Upstream URL that Kong will proxy to.
    pub url: String,
}

/// Body for `POST /services/{service_id}/routes`.
#[derive(Debug, Clone, Serialize)]
pub struct CreateRouteRequest {
    /// URL path prefixes to match.
    pub paths: Vec<String>,
    /// HTTP methods to match.
    pub methods: Vec<String>,
    /// Whether Kong should strip the matched path prefix before proxying.
    pub strip_path: bool,
}

/// Body for `POST /services/{service_id}/plugins`.
#[derive(Debug, Clone, Serialize)]
pub struct CreatePluginRequest {
    /// Plugin name (e.g. `rate-limiting`, `key-auth`).
    pub name: String,
    /// Plugin-specific configuration.
    pub config: serde_json::Value,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// Partial representation of a Kong service returned by the Admin API.
#[derive(Debug, Clone, Deserialize)]
pub struct KongServiceResponse {
    /// Unique identifier assigned by Kong.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Upstream host.
    #[serde(default)]
    pub host: String,
    /// Upstream port.
    #[serde(default)]
    pub port: u16,
    /// Upstream path prefix.
    #[serde(default)]
    pub path: Option<String>,
}

/// Partial representation of a Kong route returned by the Admin API.
#[derive(Debug, Clone, Deserialize)]
pub struct KongRouteResponse {
    /// Unique identifier assigned by Kong.
    pub id: String,
    /// Matched path prefixes.
    #[serde(default)]
    pub paths: Vec<String>,
    /// Matched HTTP methods.
    #[serde(default)]
    pub methods: Vec<String>,
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// HTTP client for the Kong Admin API.
#[derive(Clone)]
pub struct KongClient {
    client: reqwest::Client,
    base_url: String,
}

impl std::fmt::Debug for KongClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KongClient")
            .field("base_url", &self.base_url)
            .finish_non_exhaustive()
    }
}

impl KongClient {
    /// Create a new client from the given configuration.
    pub fn new(config: &KongConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: config.admin_url.trim_end_matches('/').to_string(),
        }
    }

    /// Register a new service in Kong.
    ///
    /// `POST /services`
    pub async fn create_service(
        &self,
        req: &CreateServiceRequest,
    ) -> Result<KongServiceResponse, KongError> {
        let url = format!("{}/services", self.base_url);
        let resp = self.client.post(&url).json(req).send().await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(KongError::Api {
                status: status.as_u16(),
                body,
            });
        }

        let body = resp.text().await.unwrap_or_default();
        serde_json::from_str(&body).map_err(|e| KongError::Deserialize(e.to_string()))
    }

    /// Create a route attached to an existing service.
    ///
    /// `POST /services/{service_id}/routes`
    pub async fn create_route(
        &self,
        service_id: &str,
        req: &CreateRouteRequest,
    ) -> Result<KongRouteResponse, KongError> {
        let url = format!("{}/services/{}/routes", self.base_url, service_id);
        let resp = self.client.post(&url).json(req).send().await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(KongError::Api {
                status: status.as_u16(),
                body,
            });
        }

        let body = resp.text().await.unwrap_or_default();
        serde_json::from_str(&body).map_err(|e| KongError::Deserialize(e.to_string()))
    }

    /// Add a plugin to an existing service.
    ///
    /// `POST /services/{service_id}/plugins`
    pub async fn add_plugin(
        &self,
        service_id: &str,
        req: &CreatePluginRequest,
    ) -> Result<serde_json::Value, KongError> {
        let url = format!("{}/services/{}/plugins", self.base_url, service_id);
        let resp = self.client.post(&url).json(req).send().await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(KongError::Api {
                status: status.as_u16(),
                body,
            });
        }

        let body = resp.text().await.unwrap_or_default();
        serde_json::from_str(&body).map_err(|e| KongError::Deserialize(e.to_string()))
    }

    /// Delete a route by ID. Treats 404 (already gone) as success.
    ///
    /// `DELETE /routes/{route_id}`
    pub async fn delete_route(&self, route_id: &str) -> Result<(), KongError> {
        let url = format!("{}/routes/{}", self.base_url, route_id);
        let resp = self.client.delete(&url).send().await?;

        let status = resp.status();
        if status.is_success() || status.as_u16() == 404 {
            return Ok(());
        }

        let body = resp.text().await.unwrap_or_default();
        Err(KongError::Api {
            status: status.as_u16(),
            body,
        })
    }

    /// Delete a service by ID. Treats 404 (already gone) as success.
    ///
    /// `DELETE /services/{service_id}`
    pub async fn delete_service(&self, service_id: &str) -> Result<(), KongError> {
        let url = format!("{}/services/{}", self.base_url, service_id);
        let resp = self.client.delete(&url).send().await?;

        let status = resp.status();
        if status.is_success() || status.as_u16() == 404 {
            return Ok(());
        }

        let body = resp.text().await.unwrap_or_default();
        Err(KongError::Api {
            status: status.as_u16(),
            body,
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial(kong_env)]
    fn test_kong_config_from_env_defaults() {
        // Ensure the env var is unset for this test.
        std::env::remove_var("KONG_ADMIN_URL");
        let config = KongConfig::from_env();
        assert_eq!(config.admin_url, "http://localhost:8001");
    }

    #[test]
    #[serial(kong_env)]
    fn test_kong_config_from_env_custom() {
        std::env::set_var("KONG_ADMIN_URL", "http://kong.internal:9001");
        let config = KongConfig::from_env();
        assert_eq!(config.admin_url, "http://kong.internal:9001");
        // Clean up.
        std::env::remove_var("KONG_ADMIN_URL");
    }

    #[test]
    fn test_create_service_request_serialization() {
        let req = CreateServiceRequest {
            name: "my-service".to_string(),
            url: "http://upstream:3000".to_string(),
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["name"], "my-service");
        assert_eq!(json["url"], "http://upstream:3000");
    }

    #[test]
    fn test_create_route_request_serialization() {
        let req = CreateRouteRequest {
            paths: vec!["/api/v1/hooks".to_string()],
            methods: vec!["POST".to_string(), "GET".to_string()],
            strip_path: true,
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["paths"], serde_json::json!(["/api/v1/hooks"]));
        assert_eq!(json["methods"], serde_json::json!(["POST", "GET"]));
        assert_eq!(json["strip_path"], true);
    }

    #[test]
    fn test_create_plugin_request_serialization() {
        let req = CreatePluginRequest {
            name: "rate-limiting".to_string(),
            config: serde_json::json!({
                "minute": 100,
                "policy": "local"
            }),
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["name"], "rate-limiting");
        assert_eq!(json["config"]["minute"], 100);
        assert_eq!(json["config"]["policy"], "local");
    }

    #[test]
    fn test_kong_service_response_deserialization() {
        let json = serde_json::json!({
            "id": "abc-123",
            "name": "workflow-triggers",
            "host": "localhost",
            "port": 3020,
            "path": "/hooks"
        });
        let resp: KongServiceResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.id, "abc-123");
        assert_eq!(resp.name, "workflow-triggers");
        assert_eq!(resp.host, "localhost");
        assert_eq!(resp.port, 3020);
        assert_eq!(resp.path, Some("/hooks".to_string()));
    }

    #[test]
    fn test_kong_route_response_deserialization() {
        let json = serde_json::json!({
            "id": "route-456",
            "paths": ["/api/v1/hooks"],
            "methods": ["POST"]
        });
        let resp: KongRouteResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.id, "route-456");
        assert_eq!(resp.paths, vec!["/api/v1/hooks"]);
        assert_eq!(resp.methods, vec!["POST"]);
    }

    #[test]
    fn test_kong_service_response_deserialization_defaults() {
        // Minimal response -- only required fields.
        let json = serde_json::json!({
            "id": "svc-minimal",
            "name": "bare"
        });
        let resp: KongServiceResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.id, "svc-minimal");
        assert_eq!(resp.name, "bare");
        assert_eq!(resp.host, ""); // default
        assert_eq!(resp.port, 0); // default
        assert!(resp.path.is_none());
    }

    #[test]
    fn test_kong_route_response_deserialization_defaults() {
        let json = serde_json::json!({
            "id": "route-minimal"
        });
        let resp: KongRouteResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.id, "route-minimal");
        assert!(resp.paths.is_empty());
        assert!(resp.methods.is_empty());
    }

    #[test]
    fn test_kong_client_debug() {
        let client = KongClient::new(&KongConfig {
            admin_url: "http://kong:8001".to_string(),
        });
        let debug = format!("{client:?}");
        assert!(debug.contains("http://kong:8001"));
        assert!(debug.contains("KongClient"));
    }

    #[test]
    fn test_kong_client_trims_trailing_slash() {
        let client = KongClient::new(&KongConfig {
            admin_url: "http://kong:8001/".to_string(),
        });
        let debug = format!("{client:?}");
        // The trailing slash should have been stripped.
        assert!(debug.contains("\"http://kong:8001\""));
    }

    #[test]
    fn test_kong_error_display() {
        let err = KongError::Api {
            status: 409,
            body: "conflict".to_string(),
        };
        let msg = format!("{err}");
        assert!(msg.contains("409"));
        assert!(msg.contains("conflict"));

        let err = KongError::Deserialize("bad json".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("bad json"));
    }
}
