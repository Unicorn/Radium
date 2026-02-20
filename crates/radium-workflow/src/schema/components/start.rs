//! Start (Trigger) component schema
//!
//! The Start component is the entry point of every workflow.
//! It takes no input (input comes from workflow caller) and outputs
//! the timestamp when the workflow started.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Start component input - workflow entry point
///
/// Takes no input as it's the workflow trigger.
/// The actual workflow input comes from the caller.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StartInput {
    // Empty - start receives input from workflow caller
}

/// Start component output
///
/// Contains metadata about when the workflow started.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartOutput {
    /// When the workflow started
    pub started_at: DateTime<Utc>,
}

impl StartOutput {
    /// Create a new StartOutput with the current timestamp
    pub fn now() -> Self {
        StartOutput {
            started_at: Utc::now(),
        }
    }
}

impl Default for StartOutput {
    fn default() -> Self {
        Self::now()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_input_serialization() {
        let input = StartInput::default();
        let json = serde_json::to_string(&input).unwrap();
        assert_eq!(json, "{}");
    }

    #[test]
    fn test_start_output_serialization() {
        let output = StartOutput {
            started_at: DateTime::parse_from_rfc3339("2025-01-15T10:30:00Z")
                .unwrap()
                .with_timezone(&Utc),
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("startedAt"));
        assert!(json.contains("2025-01-15T10:30:00Z"));
    }

    #[test]
    fn test_start_output_now() {
        let output = StartOutput::now();
        // Verify that started_at is within the last second
        let now = Utc::now();
        let diff = now - output.started_at;
        assert!(diff.num_seconds() < 1);
    }
}
