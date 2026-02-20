//! Log activity component schema
//!
//! The Log component sends log messages to the Kong logging endpoint.
//! Used for debugging, monitoring, and audit trails in workflows.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

/// Log levels for categorizing messages
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// Debug level - detailed information for debugging
    Debug,
    /// Info level - general information (default)
    #[default]
    Info,
    /// Warn level - warning messages
    Warn,
    /// Error level - error messages
    Error,
}

impl LogLevel {
    /// Convert to TypeScript string representation
    pub fn to_typescript(&self) -> &'static str {
        match self {
            LogLevel::Debug => "'debug'",
            LogLevel::Info => "'info'",
            LogLevel::Warn => "'warn'",
            LogLevel::Error => "'error'",
        }
    }
}

/// Log activity input
///
/// Contains the message to log along with optional level and metadata.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct LogInput {
    /// The message to log
    #[validate(length(
        min = 1,
        max = 10000,
        message = "Message must be 1-10000 characters"
    ))]
    pub message: String,

    /// Log level (defaults to info)
    #[serde(default)]
    pub level: LogLevel,

    /// Optional metadata to include in log
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl LogInput {
    /// Create a new log input with just a message
    pub fn new(message: impl Into<String>) -> Self {
        LogInput {
            message: message.into(),
            level: LogLevel::default(),
            metadata: None,
        }
    }

    /// Create a new log input with message and level
    pub fn with_level(message: impl Into<String>, level: LogLevel) -> Self {
        LogInput {
            message: message.into(),
            level,
            metadata: None,
        }
    }

    /// Add metadata to the log input
    pub fn with_metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Log activity output
///
/// Contains confirmation of logging and timestamp.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogOutput {
    /// Whether the log was successfully recorded
    pub logged: bool,

    /// Timestamp when the log was recorded
    pub timestamp: DateTime<Utc>,
}

impl LogOutput {
    /// Create a successful log output
    pub fn success() -> Self {
        LogOutput {
            logged: true,
            timestamp: Utc::now(),
        }
    }

    /// Create a failed log output
    pub fn failure() -> Self {
        LogOutput {
            logged: false,
            timestamp: Utc::now(),
        }
    }
}

impl Default for LogOutput {
    fn default() -> Self {
        Self::success()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_serialization() {
        assert_eq!(
            serde_json::to_string(&LogLevel::Debug).unwrap(),
            "\"debug\""
        );
        assert_eq!(serde_json::to_string(&LogLevel::Info).unwrap(), "\"info\"");
        assert_eq!(serde_json::to_string(&LogLevel::Warn).unwrap(), "\"warn\"");
        assert_eq!(
            serde_json::to_string(&LogLevel::Error).unwrap(),
            "\"error\""
        );
    }

    #[test]
    fn test_log_level_deserialization() {
        let level: LogLevel = serde_json::from_str("\"debug\"").unwrap();
        assert_eq!(level, LogLevel::Debug);

        let level: LogLevel = serde_json::from_str("\"error\"").unwrap();
        assert_eq!(level, LogLevel::Error);
    }

    #[test]
    fn test_log_level_typescript() {
        assert_eq!(LogLevel::Debug.to_typescript(), "'debug'");
        assert_eq!(LogLevel::Info.to_typescript(), "'info'");
        assert_eq!(LogLevel::Warn.to_typescript(), "'warn'");
        assert_eq!(LogLevel::Error.to_typescript(), "'error'");
    }

    #[test]
    fn test_log_input_new() {
        let input = LogInput::new("Test message");
        assert_eq!(input.message, "Test message");
        assert_eq!(input.level, LogLevel::Info);
        assert!(input.metadata.is_none());
    }

    #[test]
    fn test_log_input_with_level() {
        let input = LogInput::with_level("Error message", LogLevel::Error);
        assert_eq!(input.message, "Error message");
        assert_eq!(input.level, LogLevel::Error);
    }

    #[test]
    fn test_log_input_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("key".to_string(), serde_json::json!("value"));

        let input = LogInput::new("Test").with_metadata(metadata.clone());
        assert!(input.metadata.is_some());
        assert_eq!(input.metadata.unwrap().get("key").unwrap(), "value");
    }

    #[test]
    fn test_log_input_serialization() {
        let input = LogInput {
            message: "Hello, workflow!".to_string(),
            level: LogLevel::Info,
            metadata: Some({
                let mut m = HashMap::new();
                m.insert("count".to_string(), serde_json::json!(42));
                m
            }),
        };

        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("message"));
        assert!(json.contains("Hello, workflow!"));
        assert!(json.contains("level"));
        assert!(json.contains("info"));
        assert!(json.contains("metadata"));
        assert!(json.contains("count"));
    }

    #[test]
    fn test_log_input_validation() {
        use validator::Validate;

        // Valid input
        let input = LogInput::new("Valid message");
        assert!(input.validate().is_ok());

        // Empty message should fail
        let input = LogInput::new("");
        assert!(input.validate().is_err());

        // Too long message should fail
        let input = LogInput::new("x".repeat(10001));
        assert!(input.validate().is_err());
    }

    #[test]
    fn test_log_output_success() {
        let output = LogOutput::success();
        assert!(output.logged);

        let now = Utc::now();
        let diff = now - output.timestamp;
        assert!(diff.num_seconds() < 1);
    }

    #[test]
    fn test_log_output_failure() {
        let output = LogOutput::failure();
        assert!(!output.logged);
    }

    #[test]
    fn test_log_output_serialization() {
        let output = LogOutput {
            logged: true,
            timestamp: DateTime::parse_from_rfc3339("2025-01-15T10:30:00Z")
                .unwrap()
                .with_timezone(&Utc),
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("logged"));
        assert!(json.contains("true"));
        assert!(json.contains("timestamp"));
        assert!(json.contains("2025-01-15T10:30:00Z"));
    }
}
