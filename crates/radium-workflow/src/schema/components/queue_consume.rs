//! Queue Consume component schema
//!
//! The Queue Consume component pulls/consumes messages from message queues.
//! Supports SQS, RabbitMQ, Redis, and NATS with configurable polling,
//! long-poll wait times, visibility timeouts, and auto-acknowledgement.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

use super::behaviors::{ComponentBehaviors, RateLimitConfig};

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Supported message queue providers.
///
/// Defined locally to avoid cross-module dependency on the queue_publish module.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum QueueProvider {
    /// Amazon Simple Queue Service.
    #[default]
    Sqs,
    /// RabbitMQ AMQP broker.
    RabbitMq,
    /// Redis streams or list-based queues.
    Redis,
    /// NATS JetStream or core subjects.
    Nats,
}

// ---------------------------------------------------------------------------
// Supporting output struct
// ---------------------------------------------------------------------------

/// A single message pulled from the queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct QueueMessage {
    /// Provider-assigned unique message identifier.
    pub message_id: String,

    /// Message body (arbitrary JSON value).
    pub body: serde_json::Value,

    /// Provider-specific message attributes (e.g. content-type, delay seconds).
    #[serde(default)]
    pub attributes: HashMap<String, String>,

    /// Opaque receipt handle used to delete or extend visibility.
    /// Absent for providers that do not use receipt handles (e.g. NATS).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receipt_handle: Option<String>,
}

// ---------------------------------------------------------------------------
// Input
// ---------------------------------------------------------------------------

/// Queue Consume component input.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct QueueConsumeInput {
    /// URL or name identifying the queue to consume from.
    /// For SQS this is the full queue URL; for NATS, the subject name.
    #[validate(length(min = 1, message = "queue_url must not be empty"))]
    pub queue_url: String,

    /// Queue provider backend.
    #[serde(default)]
    pub provider: QueueProvider,

    /// Maximum number of messages to pull per invocation.
    /// SQS supports up to 10; other providers may vary.
    #[serde(default = "default_max_messages")]
    pub max_messages: u32,

    /// Long-polling wait time in seconds.
    /// When set, the receive call blocks up to this duration waiting for messages.
    /// Reduces empty-receive API calls and associated costs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wait_time_seconds: Option<u32>,

    /// Visibility timeout in seconds.
    /// Hides a consumed message from other consumers for this duration,
    /// giving the handler time to process and delete it before it becomes
    /// visible again.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility_timeout: Option<u32>,

    /// Whether the component automatically acknowledges (deletes/acks)
    /// each message after successful execution.
    /// Set to `false` to manage acknowledgement in a downstream step.
    #[serde(default = "default_true")]
    pub auto_acknowledge: bool,

    /// Shared component behaviors (retry, rate limit, timeout, etc.).
    #[serde(default = "queue_consume_default_behaviors")]
    #[validate(nested)]
    pub behaviors: ComponentBehaviors,
}

fn default_max_messages() -> u32 {
    1
}

fn default_true() -> bool {
    true
}

/// I/O tier defaults: 10 s timeout, 50 req/s sustained with burst 100.
fn queue_consume_default_behaviors() -> ComponentBehaviors {
    ComponentBehaviors {
        timeout_ms: 10_000,
        rate_limit: RateLimitConfig {
            requests_per_second: 50,
            burst: 100,
            ..Default::default()
        },
        ..Default::default()
    }
}

impl Default for QueueConsumeInput {
    fn default() -> Self {
        Self {
            queue_url: String::new(),
            provider: QueueProvider::default(),
            max_messages: default_max_messages(),
            wait_time_seconds: None,
            visibility_timeout: None,
            auto_acknowledge: true,
            behaviors: queue_consume_default_behaviors(),
        }
    }
}

// ---------------------------------------------------------------------------
// Output
// ---------------------------------------------------------------------------

/// Queue Consume component output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct QueueConsumeOutput {
    /// Messages pulled from the queue during this invocation.
    pub messages: Vec<QueueMessage>,

    /// Number of messages returned (convenience alias for `messages.len()`).
    pub count: u32,
}

impl Default for QueueConsumeOutput {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            count: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_with_defaults() {
        let input = QueueConsumeInput::default();

        assert!(input.queue_url.is_empty());
        assert_eq!(input.provider, QueueProvider::Sqs);
        assert_eq!(input.max_messages, 1);
        assert!(input.wait_time_seconds.is_none());
        assert!(input.visibility_timeout.is_none());
        assert!(input.auto_acknowledge);

        // Behavior tier: I/O (10 s timeout, 50 req/s, burst 100)
        assert_eq!(input.behaviors.timeout_ms, 10_000);
        assert_eq!(input.behaviors.rate_limit.requests_per_second, 50);
        assert_eq!(input.behaviors.rate_limit.burst, 100);
    }

    #[test]
    fn test_full_config_deserialization() {
        let yaml = r#"
queue_url: "https://sqs.us-east-1.amazonaws.com/123456789/my-queue"
provider: sqs
max_messages: 10
wait_time_seconds: 20
visibility_timeout: 30
auto_acknowledge: false
behaviors:
  timeout_ms: 15000
"#;
        let input: QueueConsumeInput = serde_yaml::from_str(yaml).expect("deserialize");

        assert_eq!(
            input.queue_url,
            "https://sqs.us-east-1.amazonaws.com/123456789/my-queue"
        );
        assert_eq!(input.provider, QueueProvider::Sqs);
        assert_eq!(input.max_messages, 10);
        assert_eq!(input.wait_time_seconds, Some(20));
        assert_eq!(input.visibility_timeout, Some(30));
        assert!(!input.auto_acknowledge);
        assert_eq!(input.behaviors.timeout_ms, 15_000);
    }

    #[test]
    fn test_output_serialize_deserialize() {
        let output = QueueConsumeOutput {
            messages: vec![QueueMessage {
                message_id: "msg-001".to_string(),
                body: serde_json::json!({"event": "order.created", "order_id": "ord-42"}),
                attributes: HashMap::new(),
                receipt_handle: Some("rh-abc123".to_string()),
            }],
            count: 1,
        };

        let yaml = serde_yaml::to_string(&output).expect("serialize");
        let restored: QueueConsumeOutput = serde_yaml::from_str(&yaml).expect("deserialize");

        assert_eq!(restored.count, 1);
        assert_eq!(restored.messages.len(), 1);
        assert_eq!(restored.messages[0].message_id, "msg-001");
        assert_eq!(
            restored.messages[0].body["event"],
            "order.created"
        );
        assert_eq!(
            restored.messages[0].receipt_handle,
            Some("rh-abc123".to_string())
        );
    }

    #[test]
    fn test_queue_message_structure() {
        // Minimal message — no receipt handle, no attributes
        let minimal = QueueMessage {
            message_id: "nats-seq-7".to_string(),
            body: serde_json::json!("plain string payload"),
            attributes: HashMap::new(),
            receipt_handle: None,
        };

        assert_eq!(minimal.message_id, "nats-seq-7");
        assert!(minimal.receipt_handle.is_none());
        assert!(minimal.attributes.is_empty());

        // Full message with attributes and receipt handle
        let mut attrs = HashMap::new();
        attrs.insert("ApproximateReceiveCount".to_string(), "1".to_string());
        attrs.insert("SenderId".to_string(), "AIDAI23N7MQRQN7EXAMPLE".to_string());

        let full = QueueMessage {
            message_id: "sqs-uuid-001".to_string(),
            body: serde_json::json!({"key": "value"}),
            attributes: attrs,
            receipt_handle: Some("AQEBwJnKyrHigUMZj6reyNurzkIR...".to_string()),
        };

        assert_eq!(full.attributes.len(), 2);
        assert!(full.receipt_handle.is_some());

        // Round-trip through JSON
        let json = serde_json::to_string(&full).expect("serialize");
        let restored: QueueMessage = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.message_id, full.message_id);
        assert_eq!(restored.attributes.len(), 2);
        assert_eq!(restored.receipt_handle, full.receipt_handle);
    }
}
