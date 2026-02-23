//! GraphQL Request component schema
//!
//! The GraphQL Request component executes GraphQL queries and mutations against
//! any GraphQL endpoint. Supports variables, operation name selection for
//! multi-operation documents, custom headers, and bearer token authentication.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

use super::behaviors::{ComponentBehaviors, RateLimitConfig};

/// A single location within a GraphQL document referenced by an error.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct GraphQlErrorLocation {
    /// Line number (1-based) within the document.
    pub line: u32,

    /// Column number (1-based) within the line.
    pub column: u32,
}

/// A single error entry returned in the GraphQL response `errors` array.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GraphQlError {
    /// Human-readable description of the error.
    pub message: String,

    /// Source locations within the query document associated with this error.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locations: Option<Vec<GraphQlErrorLocation>>,

    /// Path into the response data where the error occurred.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<Vec<serde_json::Value>>,

    /// Implementation-defined extensions attached to the error.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<serde_json::Value>,
}

/// GraphQL Request component input.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct GraphQlRequestInput {
    /// The URL of the GraphQL endpoint.
    #[validate(length(min = 1, message = "endpoint must not be empty"))]
    pub endpoint: String,

    /// The GraphQL query or mutation string to execute.
    #[validate(length(min = 1, message = "query must not be empty"))]
    pub query: String,

    /// Optional variables passed alongside the query.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variables: Option<serde_json::Value>,

    /// Optional operation name when the document contains multiple operations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_name: Option<String>,

    /// Custom HTTP headers to include in the request.
    #[serde(default)]
    pub headers: HashMap<String, String>,

    /// Optional bearer token sent as `Authorization: Bearer <token>`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,

    /// Shared component behaviors (retry, rate limit, timeout, etc.).
    #[serde(default = "graphql_request_default_behaviors")]
    #[validate(nested)]
    pub behaviors: ComponentBehaviors,
}

fn graphql_request_default_behaviors() -> ComponentBehaviors {
    ComponentBehaviors {
        timeout_ms: 30_000,
        rate_limit: RateLimitConfig {
            requests_per_second: 10,
            burst: 20,
            ..Default::default()
        },
        ..Default::default()
    }
}

impl Default for GraphQlRequestInput {
    fn default() -> Self {
        Self {
            endpoint: String::new(),
            query: String::new(),
            variables: None,
            operation_name: None,
            headers: HashMap::new(),
            auth_token: None,
            behaviors: graphql_request_default_behaviors(),
        }
    }
}

/// GraphQL Request component output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GraphQlRequestOutput {
    /// The `data` field from the GraphQL response body.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,

    /// Any GraphQL errors returned alongside (or instead of) data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<GraphQlError>>,

    /// HTTP status code received from the server.
    pub status_code: u16,
}

impl Default for GraphQlRequestOutput {
    fn default() -> Self {
        Self {
            data: None,
            errors: None,
            status_code: 200,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_with_defaults() {
        let input = GraphQlRequestInput::default();
        assert!(input.endpoint.is_empty());
        assert!(input.query.is_empty());
        assert!(input.variables.is_none());
        assert!(input.operation_name.is_none());
        assert!(input.headers.is_empty());
        assert!(input.auth_token.is_none());
        assert_eq!(input.behaviors.timeout_ms, 30_000);
        assert_eq!(input.behaviors.rate_limit.requests_per_second, 10);
        assert_eq!(input.behaviors.rate_limit.burst, 20);
    }

    #[test]
    fn test_full_config_deserialization() {
        let yaml = r#"
endpoint: "https://api.example.com/graphql"
query: "query GetUser($id: ID!) { user(id: $id) { name email } }"
variables:
  id: "42"
operation_name: "GetUser"
headers:
  X-Client-Name: "radium"
  Accept: "application/json"
auth_token: "my-secret-token"
behaviors:
  timeout_ms: 15000
  rate_limit:
    requests_per_second: 5
    burst: 10
"#;
        let input: GraphQlRequestInput = serde_yaml::from_str(yaml).expect("deserialize");
        assert_eq!(input.endpoint, "https://api.example.com/graphql");
        assert!(input.query.contains("GetUser"));
        assert!(input.variables.is_some());
        assert_eq!(input.operation_name, Some("GetUser".to_string()));
        assert_eq!(input.headers.len(), 2);
        assert_eq!(input.auth_token, Some("my-secret-token".to_string()));
        assert_eq!(input.behaviors.timeout_ms, 15_000);
        assert_eq!(input.behaviors.rate_limit.requests_per_second, 5);
        assert_eq!(input.behaviors.rate_limit.burst, 10);
    }

    #[test]
    fn test_output_serialize_deserialize() {
        let output = GraphQlRequestOutput {
            data: Some(serde_json::json!({"user": {"name": "Alice", "email": "alice@example.com"}})),
            errors: None,
            status_code: 200,
        };
        let yaml = serde_yaml::to_string(&output).expect("serialize");
        let restored: GraphQlRequestOutput = serde_yaml::from_str(&yaml).expect("deserialize");
        assert_eq!(restored.status_code, output.status_code);
        assert!(restored.errors.is_none());
        let data = restored.data.expect("data present");
        assert_eq!(data["user"]["name"], "Alice");
    }

    #[test]
    fn test_graphql_error_structure() {
        let error = GraphQlError {
            message: "Cannot query field 'foo' on type 'Query'.".to_string(),
            locations: Some(vec![GraphQlErrorLocation { line: 2, column: 3 }]),
            path: Some(vec![serde_json::json!("user"), serde_json::json!(0)]),
            extensions: Some(serde_json::json!({"code": "GRAPHQL_VALIDATION_FAILED"})),
        };

        let yaml = serde_yaml::to_string(&error).expect("serialize");
        let restored: GraphQlError = serde_yaml::from_str(&yaml).expect("deserialize");

        assert_eq!(restored.message, error.message);
        let locs = restored.locations.expect("locations present");
        assert_eq!(locs.len(), 1);
        assert_eq!(locs[0].line, 2);
        assert_eq!(locs[0].column, 3);
        let path = restored.path.expect("path present");
        assert_eq!(path.len(), 2);
        let ext = restored.extensions.expect("extensions present");
        assert_eq!(ext["code"], "GRAPHQL_VALIDATION_FAILED");
    }
}
