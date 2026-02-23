//! Webhook Send component schema
//!
//! The Webhook Send component fires HTTP webhook callbacks to external endpoints.
//! Supports configurable HTTP methods, custom headers, JSON payloads, and HMAC
//! signing via a secret reference for secure delivery verification.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

use super::behaviors::{ComponentBehaviors, RateLimitConfig};

/// HTTP methods available for outbound webhook delivery.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum WebhookMethod {
    /// HTTP GET — typically used for notification-only callbacks.
    Get,
    /// HTTP POST — the standard method for webhook delivery.
    #[default]
    Post,
    /// HTTP PUT — replaces the target resource.
    Put,
    /// HTTP PATCH — partially updates the target resource.
    Patch,
}

/// HMAC signing algorithm used to generate the request signature.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SigningAlgorithm {
    /// HMAC-SHA256 — widely supported default.
    #[default]
    HmacSha256,
    /// HMAC-SHA512 — stronger digest for higher-security endpoints.
    HmacSha512,
}

/// Webhook Send component input.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct WebhookSendInput {
    /// Destination URL for the webhook callback.
    #[validate(length(min = 1, message = "url must not be empty"))]
    pub url: String,

    /// HTTP method to use for the webhook request.
    #[serde(default)]
    pub method: WebhookMethod,

    /// Additional HTTP headers to include in the request.
    #[serde(default)]
    pub headers: HashMap<String, String>,

    /// Optional JSON payload to send as the request body.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_json::Value>,

    /// Secret reference (e.g. `${{ secrets.WEBHOOK_SECRET }}`) used for
    /// HMAC signing. When present the runtime computes the signature over
    /// the serialised body and attaches it in `signature_header`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signing_secret_ref: Option<String>,

    /// HMAC algorithm used to compute the request signature.
    #[serde(default)]
    pub signing_algorithm: SigningAlgorithm,

    /// Name of the header that carries the computed HMAC signature.
    #[serde(default = "default_signature_header")]
    pub signature_header: String,

    /// Whether to retry the webhook delivery on transient failures.
    #[serde(default = "default_true")]
    pub retry_on_failure: bool,

    /// Shared component behaviors (retry, rate limit, timeout, etc.).
    #[serde(default = "webhook_send_default_behaviors")]
    #[validate(nested)]
    pub behaviors: ComponentBehaviors,
}

fn default_signature_header() -> String {
    "X-Webhook-Signature".to_string()
}

fn default_true() -> bool {
    true
}

fn webhook_send_default_behaviors() -> ComponentBehaviors {
    ComponentBehaviors {
        timeout_ms: 15_000,
        rate_limit: RateLimitConfig {
            requests_per_second: 10,
            burst: 20,
            ..Default::default()
        },
        ..Default::default()
    }
}

impl Default for WebhookSendInput {
    fn default() -> Self {
        Self {
            url: String::new(),
            method: WebhookMethod::default(),
            headers: HashMap::new(),
            body: None,
            signing_secret_ref: None,
            signing_algorithm: SigningAlgorithm::default(),
            signature_header: default_signature_header(),
            retry_on_failure: true,
            behaviors: webhook_send_default_behaviors(),
        }
    }
}

/// Webhook Send component output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WebhookSendOutput {
    /// HTTP status code returned by the webhook endpoint.
    pub status_code: u16,

    /// Optional response body returned by the webhook endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_body: Option<String>,

    /// Whether the webhook was successfully delivered (2xx status code).
    pub delivered: bool,
}

impl Default for WebhookSendOutput {
    fn default() -> Self {
        Self {
            status_code: 200,
            response_body: None,
            delivered: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_with_defaults() {
        let input = WebhookSendInput::default();
        assert!(input.url.is_empty());
        assert_eq!(input.method, WebhookMethod::Post);
        assert!(input.headers.is_empty());
        assert!(input.body.is_none());
        assert!(input.signing_secret_ref.is_none());
        assert_eq!(input.signing_algorithm, SigningAlgorithm::HmacSha256);
        assert_eq!(input.signature_header, "X-Webhook-Signature");
        assert!(input.retry_on_failure);
        assert_eq!(input.behaviors.timeout_ms, 15_000);
        assert_eq!(input.behaviors.rate_limit.requests_per_second, 10);
        assert_eq!(input.behaviors.rate_limit.burst, 20);
    }

    #[test]
    fn test_full_config_deserialization() {
        let yaml = r#"
url: "https://hooks.example.com/webhook"
method: put
headers:
  Content-Type: "application/json"
  X-Custom-Header: "radium"
body:
  event: "order.created"
  order_id: "ord-9876"
signing_secret_ref: "${{ secrets.WEBHOOK_SECRET }}"
signing_algorithm: hmac_sha512
signature_header: "X-Signature-256"
retry_on_failure: false
behaviors:
  timeout_ms: 10000
"#;
        let input: WebhookSendInput = serde_yaml::from_str(yaml).expect("deserialize");
        assert_eq!(input.url, "https://hooks.example.com/webhook");
        assert_eq!(input.method, WebhookMethod::Put);
        assert_eq!(input.headers.len(), 2);
        assert!(input.body.is_some());
        assert_eq!(
            input.signing_secret_ref.as_deref(),
            Some("${{ secrets.WEBHOOK_SECRET }}")
        );
        assert_eq!(input.signing_algorithm, SigningAlgorithm::HmacSha512);
        assert_eq!(input.signature_header, "X-Signature-256");
        assert!(!input.retry_on_failure);
        assert_eq!(input.behaviors.timeout_ms, 10_000);
    }

    #[test]
    fn test_output_serialize_deserialize() {
        let output = WebhookSendOutput {
            status_code: 200,
            response_body: Some("OK".to_string()),
            delivered: true,
        };
        let yaml = serde_yaml::to_string(&output).expect("serialize");
        let restored: WebhookSendOutput = serde_yaml::from_str(&yaml).expect("deserialize");
        assert_eq!(restored.status_code, output.status_code);
        assert_eq!(restored.response_body, output.response_body);
        assert_eq!(restored.delivered, output.delivered);
    }

    #[test]
    fn test_signing_algorithm_default() {
        let algo = SigningAlgorithm::default();
        assert_eq!(algo, SigningAlgorithm::HmacSha256);

        // Verify round-trip serialization of both variants
        let sha256 = serde_json::to_string(&SigningAlgorithm::HmacSha256).unwrap();
        let sha512 = serde_json::to_string(&SigningAlgorithm::HmacSha512).unwrap();
        assert_eq!(sha256, "\"hmac_sha256\"");
        assert_eq!(sha512, "\"hmac_sha512\"");

        let restored: SigningAlgorithm =
            serde_json::from_str(&sha256).expect("deserialize hmac_sha256");
        assert_eq!(restored, SigningAlgorithm::HmacSha256);
    }
}
