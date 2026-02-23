//! WebSocket component schema
//!
//! The WebSocket component sends and receives messages over WebSocket connections.
//! Supports connect, send, receive, and close lifecycle actions with configurable
//! subprotocols, headers, and message types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

use super::behaviors::{ComponentBehaviors, RateLimitConfig};

/// Action to perform on the WebSocket connection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum WebSocketAction {
    /// Open a new WebSocket connection.
    Connect,
    /// Send a message over an existing connection.
    #[default]
    Send,
    /// Wait for and receive a message from the server.
    Receive,
    /// Close an existing WebSocket connection.
    Close,
}

/// Message encoding for WebSocket frames.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum WebSocketMessageType {
    /// UTF-8 text frame.
    #[default]
    Text,
    /// Raw binary frame.
    Binary,
}

/// WebSocket component input.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct WebSocketInput {
    /// WebSocket endpoint URL (e.g. `"wss://example.com/ws"`).
    #[validate(length(min = 1, message = "url must not be empty"))]
    pub url: String,

    /// Lifecycle action to perform on the connection.
    #[serde(default)]
    pub action: WebSocketAction,

    /// Message payload to transmit (required when `action` is `Send`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Encoding of the outbound message frame.
    #[serde(default)]
    pub message_type: WebSocketMessageType,

    /// HTTP headers forwarded during the WebSocket upgrade handshake.
    #[serde(default)]
    pub headers: HashMap<String, String>,

    /// Requested WebSocket sub-protocols (e.g. `["graphql-ws"]`).
    #[serde(default)]
    pub subprotocols: Vec<String>,

    /// Maximum milliseconds to wait for an inbound message when
    /// `action` is `Receive`. `None` defers to the component timeout.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receive_timeout_ms: Option<u64>,

    /// Shared component behaviors (retry, rate limit, timeout, etc.).
    #[serde(default = "websocket_default_behaviors")]
    #[validate(nested)]
    pub behaviors: ComponentBehaviors,
}

fn websocket_default_behaviors() -> ComponentBehaviors {
    ComponentBehaviors {
        timeout_ms: 60_000,
        heartbeat_interval_ms: Some(10_000),
        rate_limit: RateLimitConfig {
            requests_per_second: 10,
            burst: 20,
            ..Default::default()
        },
        ..Default::default()
    }
}

impl Default for WebSocketInput {
    fn default() -> Self {
        Self {
            url: String::new(),
            action: WebSocketAction::default(),
            message: None,
            message_type: WebSocketMessageType::default(),
            headers: HashMap::new(),
            subprotocols: Vec::new(),
            receive_timeout_ms: None,
            behaviors: websocket_default_behaviors(),
        }
    }
}

/// WebSocket component output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WebSocketOutput {
    /// Whether the WebSocket connection is currently open.
    pub connected: bool,

    /// Message received from the server (populated when `action` is `Receive`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Encoding of the received message frame.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_type: Option<WebSocketMessageType>,

    /// WebSocket close status code (populated when `action` is `Close`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub close_code: Option<u16>,

    /// Human-readable reason accompanying the close code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub close_reason: Option<String>,
}

impl Default for WebSocketOutput {
    fn default() -> Self {
        Self {
            connected: false,
            message: None,
            message_type: None,
            close_code: None,
            close_reason: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_with_defaults() {
        let input = WebSocketInput::default();
        assert!(input.url.is_empty());
        assert_eq!(input.action, WebSocketAction::Send);
        assert!(input.message.is_none());
        assert_eq!(input.message_type, WebSocketMessageType::Text);
        assert!(input.headers.is_empty());
        assert!(input.subprotocols.is_empty());
        assert!(input.receive_timeout_ms.is_none());
        assert_eq!(input.behaviors.timeout_ms, 60_000);
        assert_eq!(input.behaviors.heartbeat_interval_ms, Some(10_000));
        assert_eq!(input.behaviors.rate_limit.requests_per_second, 10);
        assert_eq!(input.behaviors.rate_limit.burst, 20);
    }

    #[test]
    fn test_full_config_deserialization() {
        let yaml = r#"
url: "wss://stream.example.com/feed"
action: receive
message: null
message_type: binary
headers:
  Authorization: "Bearer ${{ secrets.WS_TOKEN }}"
  X-Client-ID: "workflow-runner"
subprotocols:
  - "graphql-ws"
  - "v1.protocol"
receive_timeout_ms: 5000
behaviors:
  timeout_ms: 30000
  rate_limit:
    requests_per_second: 5
    burst: 10
"#;
        let input: WebSocketInput = serde_yaml::from_str(yaml).expect("deserialize");
        assert_eq!(input.url, "wss://stream.example.com/feed");
        assert_eq!(input.action, WebSocketAction::Receive);
        assert!(input.message.is_none());
        assert_eq!(input.message_type, WebSocketMessageType::Binary);
        assert_eq!(
            input.headers.get("Authorization").map(String::as_str),
            Some("Bearer ${{ secrets.WS_TOKEN }}")
        );
        assert_eq!(
            input.headers.get("X-Client-ID").map(String::as_str),
            Some("workflow-runner")
        );
        assert_eq!(input.subprotocols.len(), 2);
        assert_eq!(input.subprotocols[0], "graphql-ws");
        assert_eq!(input.receive_timeout_ms, Some(5_000));
        assert_eq!(input.behaviors.timeout_ms, 30_000);
        assert_eq!(input.behaviors.rate_limit.requests_per_second, 5);
        assert_eq!(input.behaviors.rate_limit.burst, 10);
    }

    #[test]
    fn test_output_serialize_deserialize() {
        let output = WebSocketOutput {
            connected: true,
            message: Some("hello from server".to_string()),
            message_type: Some(WebSocketMessageType::Text),
            close_code: None,
            close_reason: None,
        };
        let yaml = serde_yaml::to_string(&output).expect("serialize");
        let restored: WebSocketOutput = serde_yaml::from_str(&yaml).expect("deserialize");
        assert!(restored.connected);
        assert_eq!(restored.message, Some("hello from server".to_string()));
        assert_eq!(restored.message_type, Some(WebSocketMessageType::Text));
        assert!(restored.close_code.is_none());
        assert!(restored.close_reason.is_none());

        // Verify close fields round-trip when populated
        let close_output = WebSocketOutput {
            connected: false,
            message: None,
            message_type: None,
            close_code: Some(1000),
            close_reason: Some("Normal closure".to_string()),
        };
        let close_yaml = serde_yaml::to_string(&close_output).expect("serialize close");
        let close_restored: WebSocketOutput =
            serde_yaml::from_str(&close_yaml).expect("deserialize close");
        assert!(!close_restored.connected);
        assert_eq!(close_restored.close_code, Some(1000));
        assert_eq!(
            close_restored.close_reason,
            Some("Normal closure".to_string())
        );
    }

    #[test]
    fn test_action_and_message_type_defaults() {
        // Action default is Send
        let action = WebSocketAction::default();
        assert_eq!(action, WebSocketAction::Send);
        let serialized = serde_json::to_string(&action).expect("serialize action");
        assert_eq!(serialized, "\"send\"");

        // All action variants round-trip correctly
        let connect: WebSocketAction =
            serde_json::from_str("\"connect\"").expect("deserialize connect");
        assert_eq!(connect, WebSocketAction::Connect);
        let receive: WebSocketAction =
            serde_json::from_str("\"receive\"").expect("deserialize receive");
        assert_eq!(receive, WebSocketAction::Receive);
        let close: WebSocketAction =
            serde_json::from_str("\"close\"").expect("deserialize close");
        assert_eq!(close, WebSocketAction::Close);

        // MessageType default is Text
        let msg_type = WebSocketMessageType::default();
        assert_eq!(msg_type, WebSocketMessageType::Text);
        let serialized_type = serde_json::to_string(&msg_type).expect("serialize message_type");
        assert_eq!(serialized_type, "\"text\"");

        // Binary variant round-trips correctly
        let binary: WebSocketMessageType =
            serde_json::from_str("\"binary\"").expect("deserialize binary");
        assert_eq!(binary, WebSocketMessageType::Binary);
    }
}
