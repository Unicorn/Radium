//! Queue Publish component schema
//!
//! The Queue Publish component publishes messages to message queues such as
//! SQS, RabbitMQ, Redis, and NATS. Supports FIFO queues, deduplication,
//! delivery delays, and per-message attributes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

use super::behaviors::{ComponentBehaviors, RateLimitConfig};

/// Supported message queue providers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum QueueProvider {
    /// Amazon SQS (standard and FIFO queues).
    #[default]
    Sqs,
    /// RabbitMQ (AMQP).
    RabbitMq,
    /// Redis pub/sub or Redis Streams.
    Redis,
    /// NATS messaging system.
    Nats,
}

/// Queue Publish component input.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct QueuePublishInput {
    /// URL or address of the target queue. For SQS this is the queue URL;
    /// for RabbitMQ it is the exchange/routing-key; for Redis the channel
    /// name; for NATS the subject.
    #[validate(length(min = 1, message = "queue_url must not be empty"))]
    pub queue_url: String,

    /// Queue provider. Defaults to SQS.
    #[serde(default)]
    pub provider: QueueProvider,

    /// Message payload. Can be any JSON-serialisable value.
    pub message: serde_json::Value,

    /// Message group ID used for SQS FIFO queues to ensure ordering within
    /// a group.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_group_id: Option<String>,

    /// Deduplication ID used to prevent duplicate message delivery in SQS
    /// FIFO queues or similar providers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deduplication_id: Option<String>,

    /// Number of seconds to delay before the message becomes visible to
    /// consumers. SQS supports 0–900 seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delay_seconds: Option<u32>,

    /// Arbitrary key/value attributes attached to the message (e.g. SQS
    /// message attributes or AMQP headers).
    #[serde(default)]
    pub attributes: HashMap<String, String>,

    /// Shared component behaviors (retry, rate limit, timeout, etc.).
    #[serde(default = "queue_publish_default_behaviors")]
    #[validate(nested)]
    pub behaviors: ComponentBehaviors,
}

fn queue_publish_default_behaviors() -> ComponentBehaviors {
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

impl Default for QueuePublishInput {
    fn default() -> Self {
        Self {
            queue_url: String::new(),
            provider: QueueProvider::default(),
            message: serde_json::Value::Null,
            message_group_id: None,
            deduplication_id: None,
            delay_seconds: None,
            attributes: HashMap::new(),
            behaviors: queue_publish_default_behaviors(),
        }
    }
}

/// Queue Publish component output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct QueuePublishOutput {
    /// Provider-assigned message identifier.
    pub message_id: String,

    /// Sequence number assigned by the provider (e.g. SQS FIFO queues).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sequence_number: Option<String>,
}

impl Default for QueuePublishOutput {
    fn default() -> Self {
        Self {
            message_id: String::new(),
            sequence_number: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_with_defaults() {
        let input = QueuePublishInput::default();
        assert!(input.queue_url.is_empty());
        assert_eq!(input.provider, QueueProvider::Sqs);
        assert_eq!(input.message, serde_json::Value::Null);
        assert!(input.message_group_id.is_none());
        assert!(input.deduplication_id.is_none());
        assert!(input.delay_seconds.is_none());
        assert!(input.attributes.is_empty());
        assert_eq!(input.behaviors.timeout_ms, 10_000);
        assert_eq!(input.behaviors.rate_limit.requests_per_second, 50);
        assert_eq!(input.behaviors.rate_limit.burst, 100);
    }

    #[test]
    fn test_full_config_deserialization() {
        let yaml = r#"
queue_url: "https://sqs.us-east-1.amazonaws.com/123456789/my-queue.fifo"
provider: sqs
message:
  order_id: "ord-001"
  total: 99.99
message_group_id: "orders"
deduplication_id: "ord-001-publish"
delay_seconds: 30
attributes:
  source: "checkout-service"
  priority: "high"
behaviors:
  timeout_ms: 8000
  rate_limit:
    requests_per_second: 25
    burst: 50
"#;
        let input: QueuePublishInput = serde_yaml::from_str(yaml).expect("deserialize");

        assert_eq!(
            input.queue_url,
            "https://sqs.us-east-1.amazonaws.com/123456789/my-queue.fifo"
        );
        assert_eq!(input.provider, QueueProvider::Sqs);
        assert_eq!(input.message["order_id"], "ord-001");
        assert_eq!(input.message["total"], 99.99);
        assert_eq!(input.message_group_id, Some("orders".to_string()));
        assert_eq!(
            input.deduplication_id,
            Some("ord-001-publish".to_string())
        );
        assert_eq!(input.delay_seconds, Some(30));
        assert_eq!(input.attributes.get("source"), Some(&"checkout-service".to_string()));
        assert_eq!(input.attributes.get("priority"), Some(&"high".to_string()));
        assert_eq!(input.behaviors.timeout_ms, 8_000);
        assert_eq!(input.behaviors.rate_limit.requests_per_second, 25);
        assert_eq!(input.behaviors.rate_limit.burst, 50);
    }

    #[test]
    fn test_output_serialize_deserialize() {
        let output = QueuePublishOutput {
            message_id: "msg-abc-123".to_string(),
            sequence_number: Some("10000000000000000000".to_string()),
        };

        let yaml = serde_yaml::to_string(&output).expect("serialize");
        let restored: QueuePublishOutput = serde_yaml::from_str(&yaml).expect("deserialize");

        assert_eq!(restored.message_id, output.message_id);
        assert_eq!(restored.sequence_number, output.sequence_number);

        // Verify that an output without a sequence number omits the field.
        let output_no_seq = QueuePublishOutput {
            message_id: "msg-xyz-456".to_string(),
            sequence_number: None,
        };
        let json = serde_json::to_string(&output_no_seq).expect("serialize to JSON");
        assert!(!json.contains("sequence_number"));
        assert!(json.contains("message_id"));
    }

    #[test]
    fn test_queue_provider_default() {
        let provider = QueueProvider::default();
        assert_eq!(provider, QueueProvider::Sqs);

        // Verify all variants round-trip through JSON.
        let variants = [
            QueueProvider::Sqs,
            QueueProvider::RabbitMq,
            QueueProvider::Redis,
            QueueProvider::Nats,
        ];
        for variant in &variants {
            let json = serde_json::to_string(variant).expect("serialize");
            let restored: QueueProvider =
                serde_json::from_str(&json).expect("deserialize");
            assert_eq!(&restored, variant);
        }

        // Spot-check wire format.
        assert_eq!(
            serde_json::to_string(&QueueProvider::RabbitMq).unwrap(),
            "\"rabbit_mq\""
        );
        assert_eq!(
            serde_json::to_string(&QueueProvider::Nats).unwrap(),
            "\"nats\""
        );
    }
}
