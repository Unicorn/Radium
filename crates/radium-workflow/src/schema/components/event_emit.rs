//! Event Emit component schema
//!
//! The Event Emit component publishes events to the internal event bus for
//! cross-service communication. Supports correlation tracking, idempotent
//! delivery via an idempotency key, and arbitrary metadata attachment.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

use super::behaviors::{ComponentBehaviors, RateLimitConfig};

/// Event Emit component input.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct EventEmitInput {
    /// Fully-qualified event type identifier, e.g. `"order.created"` or
    /// `"payment.failed"`. Used by consumers to filter and route events.
    #[validate(length(min = 1, message = "event_type must not be empty"))]
    pub event_type: String,

    /// Logical source of the event, e.g. `"checkout-service"` or
    /// `"inventory-worker"`. Included in the event envelope for traceability.
    #[validate(length(min = 1, message = "source must not be empty"))]
    pub source: String,

    /// Event payload. Can be any JSON-serialisable value — object, array,
    /// primitive, or null.
    pub data: serde_json::Value,

    /// Arbitrary string key/value pairs attached to the event envelope.
    /// Useful for routing hints, schema version tags, or tenant identifiers.
    #[serde(default)]
    pub metadata: HashMap<String, String>,

    /// Distributed tracing correlation ID. When provided the event bus
    /// propagates it downstream so the full event chain can be linked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,

    /// Caller-supplied key used to prevent duplicate event processing.
    /// The event bus rejects a second emission that carries the same key
    /// within the deduplication window.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,

    /// Shared component behaviors (retry, rate limit, timeout, etc.).
    #[serde(default = "event_emit_default_behaviors")]
    #[validate(nested)]
    pub behaviors: ComponentBehaviors,
}

fn event_emit_default_behaviors() -> ComponentBehaviors {
    ComponentBehaviors {
        timeout_ms: 5_000,
        rate_limit: RateLimitConfig {
            requests_per_second: 100,
            burst: 200,
            ..Default::default()
        },
        ..Default::default()
    }
}

impl Default for EventEmitInput {
    fn default() -> Self {
        Self {
            event_type: String::new(),
            source: String::new(),
            data: serde_json::Value::Null,
            metadata: HashMap::new(),
            correlation_id: None,
            idempotency_key: None,
            behaviors: event_emit_default_behaviors(),
        }
    }
}

/// Event Emit component output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EventEmitOutput {
    /// Unique identifier assigned to the event by the event bus.
    pub event_id: String,

    /// ISO 8601 timestamp recorded by the event bus when it accepted the event.
    pub timestamp: String,

    /// Whether the event bus accepted the event for delivery.
    /// `false` indicates the event was rejected (e.g. duplicate idempotency key).
    pub accepted: bool,
}

impl Default for EventEmitOutput {
    fn default() -> Self {
        Self {
            event_id: String::new(),
            timestamp: String::new(),
            accepted: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_with_defaults() {
        let input = EventEmitInput::default();
        assert!(input.event_type.is_empty());
        assert!(input.source.is_empty());
        assert_eq!(input.data, serde_json::Value::Null);
        assert!(input.metadata.is_empty());
        assert!(input.correlation_id.is_none());
        assert!(input.idempotency_key.is_none());
        assert_eq!(input.behaviors.timeout_ms, 5_000);
        assert_eq!(input.behaviors.rate_limit.requests_per_second, 100);
        assert_eq!(input.behaviors.rate_limit.burst, 200);
    }

    #[test]
    fn test_full_config_deserialization() {
        let yaml = r#"
event_type: "order.created"
source: "checkout-service"
data:
  order_id: "ord-001"
  total: 149.99
  currency: "USD"
metadata:
  schema_version: "1.0"
  tenant_id: "acme-corp"
correlation_id: "trace-abc-123"
idempotency_key: "ord-001-emit"
behaviors:
  timeout_ms: 3000
  rate_limit:
    requests_per_second: 50
    burst: 100
"#;
        let input: EventEmitInput = serde_yaml::from_str(yaml).expect("deserialize");

        assert_eq!(input.event_type, "order.created");
        assert_eq!(input.source, "checkout-service");
        assert_eq!(input.data["order_id"], "ord-001");
        assert_eq!(input.data["total"], 149.99);
        assert_eq!(input.data["currency"], "USD");
        assert_eq!(
            input.metadata.get("schema_version"),
            Some(&"1.0".to_string())
        );
        assert_eq!(
            input.metadata.get("tenant_id"),
            Some(&"acme-corp".to_string())
        );
        assert_eq!(input.correlation_id, Some("trace-abc-123".to_string()));
        assert_eq!(input.idempotency_key, Some("ord-001-emit".to_string()));
        assert_eq!(input.behaviors.timeout_ms, 3_000);
        assert_eq!(input.behaviors.rate_limit.requests_per_second, 50);
        assert_eq!(input.behaviors.rate_limit.burst, 100);
    }

    #[test]
    fn test_output_serialize_deserialize() {
        let output = EventEmitOutput {
            event_id: "evt-xyz-789".to_string(),
            timestamp: "2024-01-15T10:30:00Z".to_string(),
            accepted: true,
        };

        let yaml = serde_yaml::to_string(&output).expect("serialize");
        let restored: EventEmitOutput = serde_yaml::from_str(&yaml).expect("deserialize");

        assert_eq!(restored.event_id, output.event_id);
        assert_eq!(restored.timestamp, output.timestamp);
        assert_eq!(restored.accepted, output.accepted);

        // Verify JSON round-trip and field presence.
        let json = serde_json::to_string(&output).expect("serialize to JSON");
        assert!(json.contains("event_id"));
        assert!(json.contains("timestamp"));
        assert!(json.contains("accepted"));

        // Verify a rejected output round-trips cleanly.
        let rejected = EventEmitOutput {
            event_id: "evt-dup-001".to_string(),
            timestamp: "2024-01-15T10:30:01Z".to_string(),
            accepted: false,
        };
        let rejected_json = serde_json::to_string(&rejected).expect("serialize rejected");
        let restored_rejected: EventEmitOutput =
            serde_json::from_str(&rejected_json).expect("deserialize rejected");
        assert!(!restored_rejected.accepted);
        assert_eq!(restored_rejected.event_id, "evt-dup-001");
    }

    #[test]
    fn test_high_rate_limit_defaults() {
        let behaviors = event_emit_default_behaviors();
        // Event emit is I/O-tier and must handle high-throughput event streams.
        assert_eq!(behaviors.timeout_ms, 5_000);
        assert_eq!(behaviors.rate_limit.requests_per_second, 100);
        assert_eq!(behaviors.rate_limit.burst, 200);
        // Burst must be at least 2x the sustained rate to absorb short spikes.
        assert!(
            behaviors.rate_limit.burst >= behaviors.rate_limit.requests_per_second * 2,
            "burst ({}) should be at least 2x requests_per_second ({})",
            behaviors.rate_limit.burst,
            behaviors.rate_limit.requests_per_second
        );
        // Retry and circuit breaker defaults should be inherited from ComponentBehaviors.
        assert_eq!(behaviors.retry_policy.max_attempts, 3);
        assert_eq!(behaviors.circuit_breaker.failure_threshold, 5);
    }
}
