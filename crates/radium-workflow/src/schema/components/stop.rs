//! Stop (End) component schema
//!
//! The Stop component is the exit point of every workflow.
//! It receives the final result and terminates the workflow execution.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Stop component input - receives final workflow result
///
/// Generic over the result type T, defaults to serde_json::Value
/// for maximum flexibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StopInput<T = serde_json::Value> {
    /// The result to return from the workflow
    pub result: T,
}

impl Default for StopInput<serde_json::Value> {
    fn default() -> Self {
        StopInput {
            result: serde_json::Value::Null,
        }
    }
}

/// Stop component output - workflow completion metadata
///
/// Contains information about when the workflow completed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StopOutput {
    /// When the workflow completed
    pub completed_at: DateTime<Utc>,

    /// Whether the workflow completed successfully
    pub success: bool,
}

impl StopOutput {
    /// Create a successful completion output
    pub fn success() -> Self {
        StopOutput {
            completed_at: Utc::now(),
            success: true,
        }
    }

    /// Create a failed completion output
    pub fn failure() -> Self {
        StopOutput {
            completed_at: Utc::now(),
            success: false,
        }
    }
}

impl Default for StopOutput {
    fn default() -> Self {
        Self::success()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stop_input_serialization() {
        let input = StopInput {
            result: serde_json::json!({ "data": "test result" }),
        };
        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("result"));
        assert!(json.contains("test result"));
    }

    #[test]
    fn test_stop_input_with_typed_result() {
        #[derive(Debug, Serialize, Deserialize)]
        struct MyResult {
            count: i32,
            message: String,
        }

        let input = StopInput {
            result: MyResult {
                count: 42,
                message: "done".to_string(),
            },
        };
        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("count"));
        assert!(json.contains("42"));
    }

    #[test]
    fn test_stop_output_success() {
        let output = StopOutput::success();
        assert!(output.success);
        let now = Utc::now();
        let diff = now - output.completed_at;
        assert!(diff.num_seconds() < 1);
    }

    #[test]
    fn test_stop_output_failure() {
        let output = StopOutput::failure();
        assert!(!output.success);
    }

    #[test]
    fn test_stop_output_serialization() {
        let output = StopOutput {
            completed_at: DateTime::parse_from_rfc3339("2025-01-15T10:30:00Z")
                .unwrap()
                .with_timezone(&Utc),
            success: true,
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("completedAt"));
        assert!(json.contains("2025-01-15T10:30:00Z"));
        assert!(json.contains("success"));
    }
}
