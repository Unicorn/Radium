//! Action component schema (formerly "activity")
//!
//! The Action component represents a Temporal activity invocation.
//! It supports retry policies, timeouts, and various configuration options.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

/// Retry policy types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum RetryPolicy {
    /// No retries
    NoRetry,
    /// Linear backoff
    Linear,
    /// Exponential backoff (default)
    #[default]
    Exponential,
    /// Custom retry configuration
    Custom,
}

impl RetryPolicy {
    /// Convert to TypeScript representation
    pub fn to_typescript(&self) -> &'static str {
        match self {
            RetryPolicy::NoRetry => "'no-retry'",
            RetryPolicy::Linear => "'linear'",
            RetryPolicy::Exponential => "'exponential'",
            RetryPolicy::Custom => "'custom'",
        }
    }
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default, Validate)]
#[serde(rename_all = "camelCase")]
pub struct RetryConfig {
    /// Maximum number of attempts
    #[serde(default = "default_max_attempts")]
    #[validate(range(min = 1, message = "Max attempts must be at least 1"))]
    pub max_attempts: u32,

    /// Initial interval between retries in milliseconds
    #[serde(default = "default_initial_interval")]
    pub initial_interval_ms: u64,

    /// Maximum interval between retries in milliseconds
    #[serde(default = "default_max_interval")]
    pub max_interval_ms: u64,

    /// Backoff coefficient for exponential backoff
    #[serde(default = "default_backoff_coefficient")]
    pub backoff_coefficient: f64,

    /// Error types that should not be retried
    #[serde(default)]
    pub non_retryable_errors: Vec<String>,
}

fn default_max_attempts() -> u32 {
    3
}

fn default_initial_interval() -> u64 {
    1000
}

fn default_max_interval() -> u64 {
    60000
}

fn default_backoff_coefficient() -> f64 {
    2.0
}

impl RetryConfig {
    /// Create a new retry config with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a no-retry config
    pub fn no_retry() -> Self {
        Self {
            max_attempts: 1,
            ..Default::default()
        }
    }

    /// Set max attempts
    pub fn with_max_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = attempts;
        self
    }

    /// Set initial interval
    pub fn with_initial_interval(mut self, ms: u64) -> Self {
        self.initial_interval_ms = ms;
        self
    }

    /// Set max interval
    pub fn with_max_interval(mut self, ms: u64) -> Self {
        self.max_interval_ms = ms;
        self
    }

    /// Set backoff coefficient
    pub fn with_backoff_coefficient(mut self, coefficient: f64) -> Self {
        self.backoff_coefficient = coefficient;
        self
    }

    /// Add non-retryable error
    pub fn add_non_retryable_error(mut self, error: impl Into<String>) -> Self {
        self.non_retryable_errors.push(error.into());
        self
    }
}

/// Timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default, Validate)]
#[serde(rename_all = "camelCase")]
pub struct TimeoutConfig {
    /// Start to close timeout (how long the activity can run)
    #[serde(default = "default_start_to_close")]
    pub start_to_close_ms: u64,

    /// Schedule to start timeout (how long to wait for worker)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule_to_start_ms: Option<u64>,

    /// Schedule to close timeout (total time including queue)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule_to_close_ms: Option<u64>,

    /// Heartbeat timeout (for long-running activities)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_ms: Option<u64>,
}

fn default_start_to_close() -> u64 {
    300000 // 5 minutes
}

impl TimeoutConfig {
    /// Create a new timeout config with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set start to close timeout
    pub fn with_start_to_close(mut self, ms: u64) -> Self {
        self.start_to_close_ms = ms;
        self
    }

    /// Set schedule to start timeout
    pub fn with_schedule_to_start(mut self, ms: u64) -> Self {
        self.schedule_to_start_ms = Some(ms);
        self
    }

    /// Set schedule to close timeout
    pub fn with_schedule_to_close(mut self, ms: u64) -> Self {
        self.schedule_to_close_ms = Some(ms);
        self
    }

    /// Set heartbeat timeout
    pub fn with_heartbeat(mut self, ms: u64) -> Self {
        self.heartbeat_ms = Some(ms);
        self
    }
}

/// Activity component input
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ActivityInput {
    /// Activity name/identifier
    #[validate(length(min = 1, message = "Activity name is required"))]
    pub activity_name: String,

    /// Task queue for the activity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_queue: Option<String>,

    /// Input parameters
    #[serde(default)]
    pub params: HashMap<String, serde_json::Value>,

    /// Retry configuration
    #[serde(default)]
    pub retry: RetryConfig,

    /// Timeout configuration
    #[serde(default)]
    pub timeouts: TimeoutConfig,

    /// Whether to wait for result
    #[serde(default = "default_true")]
    pub await_result: bool,
}

fn default_true() -> bool {
    true
}

impl ActivityInput {
    /// Create a new activity input
    pub fn new(activity_name: impl Into<String>) -> Self {
        Self {
            activity_name: activity_name.into(),
            task_queue: None,
            params: HashMap::new(),
            retry: RetryConfig::default(),
            timeouts: TimeoutConfig::default(),
            await_result: true,
        }
    }

    /// Set task queue
    pub fn with_task_queue(mut self, queue: impl Into<String>) -> Self {
        self.task_queue = Some(queue.into());
        self
    }

    /// Add a parameter
    pub fn with_param(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.params.insert(key.into(), value);
        self
    }

    /// Set all parameters
    pub fn with_params(mut self, params: HashMap<String, serde_json::Value>) -> Self {
        self.params = params;
        self
    }

    /// Set retry config
    pub fn with_retry(mut self, retry: RetryConfig) -> Self {
        self.retry = retry;
        self
    }

    /// Set timeout config
    pub fn with_timeouts(mut self, timeouts: TimeoutConfig) -> Self {
        self.timeouts = timeouts;
        self
    }

    /// Set whether to await result
    pub fn await_result(mut self, await_result: bool) -> Self {
        self.await_result = await_result;
        self
    }
}

impl Default for ActivityInput {
    fn default() -> Self {
        Self::new("activity")
    }
}

/// Activity error details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityError {
    /// Error code
    pub code: String,

    /// Error message
    pub message: String,

    /// Whether the error is retryable
    pub retryable: bool,

    /// Additional error details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ActivityError {
    /// Create a new activity error
    pub fn new(code: impl Into<String>, message: impl Into<String>, retryable: bool) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            retryable,
            details: None,
        }
    }

    /// Add details
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

/// Activity component output
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityOutput {
    /// Whether the activity succeeded
    pub success: bool,

    /// Activity result (if successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,

    /// Error information (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ActivityError>,

    /// Duration in milliseconds
    pub duration_ms: u64,

    /// Number of attempts made
    pub attempts: u32,
}

impl ActivityOutput {
    /// Create a successful activity output
    pub fn success(result: serde_json::Value, duration_ms: u64, attempts: u32) -> Self {
        Self {
            success: true,
            result: Some(result),
            error: None,
            duration_ms,
            attempts,
        }
    }

    /// Create a failed activity output
    pub fn failure(error: ActivityError, duration_ms: u64, attempts: u32) -> Self {
        Self {
            success: false,
            result: None,
            error: Some(error),
            duration_ms,
            attempts,
        }
    }
}

impl Default for ActivityOutput {
    fn default() -> Self {
        Self {
            success: true,
            result: Some(serde_json::Value::Null),
            error: None,
            duration_ms: 0,
            attempts: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_policy_serialization() {
        assert_eq!(
            serde_json::to_string(&RetryPolicy::Exponential).unwrap(),
            "\"exponential\""
        );
        assert_eq!(
            serde_json::to_string(&RetryPolicy::NoRetry).unwrap(),
            "\"noRetry\""
        );
    }

    #[test]
    fn test_retry_config() {
        let config = RetryConfig::new()
            .with_max_attempts(5)
            .with_initial_interval(2000)
            .add_non_retryable_error("INVALID_INPUT");

        assert_eq!(config.max_attempts, 5);
        assert_eq!(config.initial_interval_ms, 2000);
        assert_eq!(config.non_retryable_errors.len(), 1);
    }

    #[test]
    fn test_timeout_config() {
        let config = TimeoutConfig::new()
            .with_start_to_close(60000)
            .with_heartbeat(10000);

        assert_eq!(config.start_to_close_ms, 60000);
        assert_eq!(config.heartbeat_ms, Some(10000));
    }

    #[test]
    fn test_activity_input() {
        let input = ActivityInput::new("processOrder")
            .with_task_queue("orders")
            .with_param("orderId", serde_json::json!("123"));

        assert_eq!(input.activity_name, "processOrder");
        assert_eq!(input.task_queue, Some("orders".to_string()));
        assert!(input.params.contains_key("orderId"));
    }

    #[test]
    fn test_activity_input_validation() {
        use validator::Validate;

        let input = ActivityInput::new("valid");
        assert!(input.validate().is_ok());

        let input = ActivityInput {
            activity_name: "".to_string(),
            ..Default::default()
        };
        assert!(input.validate().is_err());
    }

    #[test]
    fn test_activity_input_serialization() {
        let input = ActivityInput::new("test")
            .with_retry(RetryConfig::new().with_max_attempts(3))
            .with_timeouts(TimeoutConfig::new().with_start_to_close(30000));

        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("activityName"));
        assert!(json.contains("retry"));
        assert!(json.contains("timeouts"));
    }

    #[test]
    fn test_activity_error() {
        let error = ActivityError::new("TIMEOUT", "Activity timed out", true)
            .with_details(serde_json::json!({"lastAttempt": 3}));

        assert_eq!(error.code, "TIMEOUT");
        assert!(error.retryable);
        assert!(error.details.is_some());
    }

    #[test]
    fn test_activity_output_success() {
        let output = ActivityOutput::success(serde_json::json!({"orderId": "123"}), 1500, 1);

        assert!(output.success);
        assert!(output.result.is_some());
        assert!(output.error.is_none());
        assert_eq!(output.duration_ms, 1500);
    }

    #[test]
    fn test_activity_output_failure() {
        let error = ActivityError::new("ERROR", "Something went wrong", false);
        let output = ActivityOutput::failure(error, 5000, 3);

        assert!(!output.success);
        assert!(output.result.is_none());
        assert!(output.error.is_some());
        assert_eq!(output.attempts, 3);
    }
}
