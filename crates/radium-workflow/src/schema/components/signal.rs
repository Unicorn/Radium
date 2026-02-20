//! Signal component schema
//!
//! The Signal component handles sending and receiving Temporal signals.
//! Enables inter-workflow communication.

use serde::{Deserialize, Serialize};
use validator::Validate;

/// Signal direction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum SignalDirection {
    /// Send a signal to another workflow
    Send,
    /// Receive a signal (wait for it)
    #[default]
    Receive,
}

impl SignalDirection {
    /// Convert to TypeScript representation
    pub fn to_typescript(&self) -> &'static str {
        match self {
            SignalDirection::Send => "'send'",
            SignalDirection::Receive => "'receive'",
        }
    }
}

/// Signal component input
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SignalInput {
    /// Signal name
    #[validate(length(min = 1, message = "Signal name is required"))]
    pub signal_name: String,

    /// Direction (send or receive)
    #[serde(default)]
    pub direction: SignalDirection,

    /// Target workflow ID (for send)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_workflow_id: Option<String>,

    /// Target run ID (optional, for send)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_run_id: Option<String>,

    /// Payload to send
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,

    /// Timeout for receive (0 = wait forever)
    #[serde(default)]
    pub timeout_ms: u64,

    /// Variable to store received signal
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_variable: Option<String>,
}

impl SignalInput {
    /// Create a receive signal input
    pub fn receive(signal_name: impl Into<String>) -> Self {
        Self {
            signal_name: signal_name.into(),
            direction: SignalDirection::Receive,
            target_workflow_id: None,
            target_run_id: None,
            payload: None,
            timeout_ms: 0,
            output_variable: None,
        }
    }

    /// Create a send signal input
    pub fn send(
        signal_name: impl Into<String>,
        target_workflow_id: impl Into<String>,
    ) -> Self {
        Self {
            signal_name: signal_name.into(),
            direction: SignalDirection::Send,
            target_workflow_id: Some(target_workflow_id.into()),
            target_run_id: None,
            payload: None,
            timeout_ms: 0,
            output_variable: None,
        }
    }

    /// Set payload
    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = Some(payload);
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }

    /// Set output variable
    pub fn with_output_variable(mut self, variable: impl Into<String>) -> Self {
        self.output_variable = Some(variable.into());
        self
    }

    /// Set target run ID
    pub fn with_target_run_id(mut self, run_id: impl Into<String>) -> Self {
        self.target_run_id = Some(run_id.into());
        self
    }

    /// Validate signal configuration
    pub fn validate_config(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.direction == SignalDirection::Send && self.target_workflow_id.is_none() {
            errors.push("Send signal requires target_workflow_id".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Default for SignalInput {
    fn default() -> Self {
        Self::receive("signal")
    }
}

/// Signal component output
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignalOutput {
    /// Signal name
    pub signal_name: String,

    /// Whether signal was sent (for send direction)
    pub sent: bool,

    /// Whether signal was received (for receive direction)
    pub received: bool,

    /// Signal payload
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,

    /// Sender workflow ID (for received signals)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_workflow_id: Option<String>,

    /// Whether timeout occurred
    #[serde(default)]
    pub timed_out: bool,
}

impl SignalOutput {
    /// Create a sent signal output
    pub fn sent(signal_name: impl Into<String>) -> Self {
        Self {
            signal_name: signal_name.into(),
            sent: true,
            received: false,
            payload: None,
            sender_workflow_id: None,
            timed_out: false,
        }
    }

    /// Create a received signal output
    pub fn received(
        signal_name: impl Into<String>,
        payload: serde_json::Value,
        sender: Option<String>,
    ) -> Self {
        Self {
            signal_name: signal_name.into(),
            sent: false,
            received: true,
            payload: Some(payload),
            sender_workflow_id: sender,
            timed_out: false,
        }
    }

    /// Create a timeout output
    pub fn timeout(signal_name: impl Into<String>) -> Self {
        Self {
            signal_name: signal_name.into(),
            sent: false,
            received: false,
            payload: None,
            sender_workflow_id: None,
            timed_out: true,
        }
    }
}

impl Default for SignalOutput {
    fn default() -> Self {
        Self::received("signal", serde_json::Value::Null, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_direction_serialization() {
        assert_eq!(
            serde_json::to_string(&SignalDirection::Send).unwrap(),
            "\"send\""
        );
        assert_eq!(
            serde_json::to_string(&SignalDirection::Receive).unwrap(),
            "\"receive\""
        );
    }

    #[test]
    fn test_signal_receive() {
        let signal = SignalInput::receive("approval")
            .with_timeout(30000)
            .with_output_variable("approvalResult");

        assert_eq!(signal.signal_name, "approval");
        assert_eq!(signal.direction, SignalDirection::Receive);
        assert_eq!(signal.timeout_ms, 30000);
        assert!(signal.validate_config().is_ok());
    }

    #[test]
    fn test_signal_send() {
        let signal = SignalInput::send("notify", "target-workflow-123")
            .with_payload(serde_json::json!({"message": "Hello"}));

        assert_eq!(signal.signal_name, "notify");
        assert_eq!(signal.direction, SignalDirection::Send);
        assert!(signal.target_workflow_id.is_some());
        assert!(signal.validate_config().is_ok());
    }

    #[test]
    fn test_signal_validation() {
        let signal = SignalInput {
            direction: SignalDirection::Send,
            target_workflow_id: None,
            ..SignalInput::default()
        };
        assert!(signal.validate_config().is_err());
    }

    #[test]
    fn test_signal_output_sent() {
        let output = SignalOutput::sent("notify");
        assert!(output.sent);
        assert!(!output.received);
    }

    #[test]
    fn test_signal_output_received() {
        let output = SignalOutput::received(
            "approval",
            serde_json::json!({"approved": true}),
            Some("sender-123".to_string()),
        );

        assert!(output.received);
        assert!(output.payload.is_some());
        assert!(output.sender_workflow_id.is_some());
    }

    #[test]
    fn test_signal_output_timeout() {
        let output = SignalOutput::timeout("approval");
        assert!(output.timed_out);
        assert!(!output.received);
    }

    #[test]
    fn test_serialization() {
        let input = SignalInput::send("update", "wf-123")
            .with_payload(serde_json::json!({"status": "complete"}));

        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("signalName"));
        assert!(json.contains("direction"));
        assert!(json.contains("targetWorkflowId"));
    }
}
