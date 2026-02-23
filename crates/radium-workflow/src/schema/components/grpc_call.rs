//! gRPC Call component schema
//!
//! The gRPC Call component makes unary gRPC calls to external services.
//! Supports fully-qualified service and method names, JSON-encoded request
//! messages, gRPC metadata headers, optional TLS, and per-call deadline
//! overrides.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

use super::behaviors::{ComponentBehaviors, RateLimitConfig};

/// gRPC Call component input.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct GrpcCallInput {
    /// gRPC server endpoint in `host:port` format (e.g. `"localhost:50051"`).
    #[validate(length(min = 1, message = "endpoint must not be empty"))]
    pub endpoint: String,

    /// Fully-qualified gRPC service name
    /// (e.g. `"com.example.OrderService"`).
    #[validate(length(min = 1, message = "service must not be empty"))]
    pub service: String,

    /// RPC method name to invoke (e.g. `"CreateOrder"`).
    #[validate(length(min = 1, message = "method must not be empty"))]
    pub method: String,

    /// Request message serialised as JSON.  The runtime encodes this into the
    /// appropriate protobuf wire format before transmission.
    #[serde(default)]
    pub message: serde_json::Value,

    /// gRPC metadata key-value pairs sent alongside the request (analogous to
    /// HTTP headers).
    #[serde(default)]
    pub metadata: HashMap<String, String>,

    /// Whether to establish a TLS-secured connection.  Defaults to `false`
    /// (plaintext).
    #[serde(default)]
    pub use_tls: bool,

    /// Secret reference for a PEM-encoded TLS certificate used to authenticate
    /// the client or verify the server (e.g. `"${{ secrets.GRPC_TLS_CERT }}"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls_cert_ref: Option<String>,

    /// Per-call gRPC deadline in milliseconds.  When set this overrides the
    /// `behaviors.timeout_ms` value for deadline propagation at the gRPC layer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline_ms: Option<u64>,

    /// Shared component behaviors (retry, rate limit, timeout, etc.).
    #[serde(default = "grpc_call_default_behaviors")]
    #[validate(nested)]
    pub behaviors: ComponentBehaviors,
}

fn grpc_call_default_behaviors() -> ComponentBehaviors {
    ComponentBehaviors {
        timeout_ms: 30_000,
        rate_limit: RateLimitConfig {
            requests_per_second: 20,
            burst: 40,
            ..Default::default()
        },
        ..Default::default()
    }
}

impl Default for GrpcCallInput {
    fn default() -> Self {
        Self {
            endpoint: String::new(),
            service: String::new(),
            method: String::new(),
            message: serde_json::Value::Null,
            metadata: HashMap::new(),
            use_tls: false,
            tls_cert_ref: None,
            deadline_ms: None,
            behaviors: grpc_call_default_behaviors(),
        }
    }
}

/// gRPC Call component output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GrpcCallOutput {
    /// Response message deserialised from protobuf into JSON.
    pub response: serde_json::Value,

    /// gRPC status code.  `0` represents `OK`; non-zero values follow the
    /// canonical gRPC status code definitions.
    pub status_code: u32,

    /// Human-readable gRPC status message accompanying a non-OK status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_message: Option<String>,

    /// Trailing gRPC metadata returned by the server.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl Default for GrpcCallOutput {
    fn default() -> Self {
        Self {
            response: serde_json::Value::Null,
            status_code: 0,
            status_message: None,
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_with_defaults() {
        let input = GrpcCallInput::default();
        assert!(input.endpoint.is_empty());
        assert!(input.service.is_empty());
        assert!(input.method.is_empty());
        assert_eq!(input.message, serde_json::Value::Null);
        assert!(input.metadata.is_empty());
        assert!(!input.use_tls);
        assert!(input.tls_cert_ref.is_none());
        assert!(input.deadline_ms.is_none());
        assert_eq!(input.behaviors.timeout_ms, 30_000);
        assert_eq!(input.behaviors.rate_limit.requests_per_second, 20);
        assert_eq!(input.behaviors.rate_limit.burst, 40);
    }

    #[test]
    fn test_full_config_deserialization() {
        let yaml = r#"
endpoint: "grpc.example.com:443"
service: "com.example.OrderService"
method: "CreateOrder"
message:
  customer_id: "cust-42"
  item_ids:
    - "item-1"
    - "item-2"
metadata:
  x-request-id: "req-abc123"
  authorization: "Bearer ${{ secrets.GRPC_TOKEN }}"
use_tls: true
tls_cert_ref: "${{ secrets.GRPC_TLS_CERT }}"
deadline_ms: 5000
behaviors:
  timeout_ms: 10000
  rate_limit:
    requests_per_second: 5
    burst: 10
"#;
        let input: GrpcCallInput = serde_yaml::from_str(yaml).expect("deserialize");
        assert_eq!(input.endpoint, "grpc.example.com:443");
        assert_eq!(input.service, "com.example.OrderService");
        assert_eq!(input.method, "CreateOrder");
        assert_eq!(input.message["customer_id"], "cust-42");
        assert_eq!(input.metadata.len(), 2);
        assert!(input.use_tls);
        assert_eq!(
            input.tls_cert_ref.as_deref(),
            Some("${{ secrets.GRPC_TLS_CERT }}")
        );
        assert_eq!(input.deadline_ms, Some(5_000));
        assert_eq!(input.behaviors.timeout_ms, 10_000);
        assert_eq!(input.behaviors.rate_limit.requests_per_second, 5);
        assert_eq!(input.behaviors.rate_limit.burst, 10);
    }

    #[test]
    fn test_output_serialize_deserialize() {
        let output = GrpcCallOutput {
            response: serde_json::json!({"order_id": "ord-99", "status": "CREATED"}),
            status_code: 0,
            status_message: None,
            metadata: {
                let mut m = HashMap::new();
                m.insert("x-request-id".to_string(), "req-abc123".to_string());
                m
            },
        };
        let yaml = serde_yaml::to_string(&output).expect("serialize");
        let restored: GrpcCallOutput = serde_yaml::from_str(&yaml).expect("deserialize");
        assert_eq!(restored.status_code, 0);
        assert!(restored.status_message.is_none());
        assert_eq!(restored.response["order_id"], "ord-99");
        assert_eq!(restored.response["status"], "CREATED");
        assert_eq!(restored.metadata.get("x-request-id").map(String::as_str), Some("req-abc123"));
    }

    #[test]
    fn test_default_tls_disabled() {
        let input = GrpcCallInput::default();
        assert!(!input.use_tls, "TLS must be disabled by default");
        assert!(
            input.tls_cert_ref.is_none(),
            "tls_cert_ref must be absent when TLS is disabled"
        );

        // Verify that use_tls=false is preserved through a serialization round-trip.
        let yaml = serde_yaml::to_string(&input).expect("serialize");
        let restored: GrpcCallInput = serde_yaml::from_str(&yaml).expect("deserialize");
        assert!(!restored.use_tls);
        assert!(restored.tls_cert_ref.is_none());
    }
}
