//! Failure detection and classification system.
//!
//! Provides functionality to classify agent execution failures into recoverable
//! and non-recoverable types, enabling intelligent recovery strategies.

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;
use thiserror::Error;

/// Types of failures that can occur during workflow execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FailureType {
    /// Transient failure that may succeed on retry (network, timeout, resource unavailable).
    Transient {
        /// Reason for the transient failure.
        reason: String,
    },
    /// Permanent failure that won't succeed on retry (validation, logic errors, missing dependencies).
    Permanent {
        /// Reason for the permanent failure.
        reason: String,
    },
    /// Agent-specific failure indicating the agent may be the problem.
    AgentFailure {
        /// Agent ID that failed.
        agent_id: String,
        /// Reason for the agent failure.
        reason: String,
    },
    /// Unknown or unclassified failure.
    Unknown {
        /// Error message.
        error: String,
    },
}

impl FailureType {
    /// Returns true if this failure type is recoverable.
    pub fn is_recoverable(&self) -> bool {
        matches!(self, FailureType::Transient { .. } | FailureType::AgentFailure { .. })
    }

    /// Returns a string description of the failure type.
    pub fn description(&self) -> String {
        match self {
            FailureType::Transient { reason } => format!("Transient: {}", reason),
            FailureType::Permanent { reason } => format!("Permanent: {}", reason),
            FailureType::AgentFailure { agent_id, reason } => {
                format!("Agent failure ({}): {}", agent_id, reason)
            }
            FailureType::Unknown { error } => format!("Unknown: {}", error),
        }
    }
}

/// Classifies errors into failure types.
pub struct FailureClassifier;

impl FailureClassifier {
    /// Creates a new failure classifier.
    pub fn new() -> Self {
        Self
    }

    /// Classifies an error into a failure type.
    ///
    /// # Arguments
    /// * `error` - The error to classify (as a string or error trait object)
    ///
    /// # Returns
    /// The classified failure type
    pub fn classify(&self, error: &dyn Error) -> FailureType {
        let error_msg = error.to_string().to_lowercase();

        // Check for transient indicators
        if Self::is_transient_error(&error_msg) {
            return FailureType::Transient {
                reason: error.to_string(),
            };
        }

        // Check for permanent indicators
        if Self::is_permanent_error(&error_msg) {
            return FailureType::Permanent {
                reason: error.to_string(),
            };
        }

        // Check for agent-specific failures
        if let Some(agent_id) = Self::extract_agent_id(&error_msg) {
            return FailureType::AgentFailure {
                agent_id,
                reason: error.to_string(),
            };
        }

        // Default to unknown
        FailureType::Unknown {
            error: error.to_string(),
        }
    }

    /// Classifies an error from a string message.
    pub fn classify_from_string(&self, error_msg: &str) -> FailureType {
        let lower_msg = error_msg.to_lowercase();

        if Self::is_transient_error(&lower_msg) {
            return FailureType::Transient {
                reason: error_msg.to_string(),
            };
        }

        if Self::is_permanent_error(&lower_msg) {
            return FailureType::Permanent {
                reason: error_msg.to_string(),
            };
        }

        if let Some(agent_id) = Self::extract_agent_id(&lower_msg) {
            return FailureType::AgentFailure {
                agent_id,
                reason: error_msg.to_string(),
            };
        }

        FailureType::Unknown {
            error: error_msg.to_string(),
        }
    }

    /// Checks if an error message indicates a transient failure.
    fn is_transient_error(msg: &str) -> bool {
        let transient_keywords = [
            "timeout",
            "connection",
            "network",
            "unavailable",
            "temporary",
            "retry",
            "rate limit",
            "quota",
            "throttle",
            "service unavailable",
            "503",
            "502",
            "504",
            "connection refused",
            "connection reset",
        ];

        transient_keywords.iter().any(|keyword| msg.contains(keyword))
    }

    /// Checks if an error message indicates a permanent failure.
    fn is_permanent_error(msg: &str) -> bool {
        let permanent_keywords = [
            "validation",
            "invalid",
            "not found",
            "missing",
            "syntax error",
            "parse error",
            "type error",
            "not implemented",
            "unsupported",
            "forbidden",
            "unauthorized",
            "401",
            "403",
            "404",
            "400",
        ];

        permanent_keywords.iter().any(|keyword| msg.contains(keyword))
    }

    /// Extracts agent ID from error message if present.
    fn extract_agent_id(msg: &str) -> Option<String> {
        // Look for patterns like "agent not found: agent-id" or "agent 'agent-id' failed"
        if let Some(start) = msg.find("agent not found:") {
            let rest = &msg[start + "agent not found:".len()..];
            if let Some(end) = rest.find(|c: char| c.is_whitespace() || c == ',' || c == '.') {
                return Some(rest[..end].trim().to_string());
            }
            return Some(rest.trim().to_string());
        }

        if let Some(start) = msg.find("agent '") {
            let rest = &msg[start + "agent '".len()..];
            if let Some(end) = rest.find('\'') {
                return Some(rest[..end].to_string());
            }
        }

        None
    }
}

impl Default for FailureClassifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Record of a single failure occurrence.
#[derive(Debug, Clone)]
pub struct FailureRecord {
    /// Timestamp when the failure occurred.
    pub timestamp: DateTime<Utc>,
    /// Type of failure.
    pub failure_type: FailureType,
    /// Error message.
    pub error_message: String,
}

/// History of failures for a specific task.
#[derive(Debug, Clone)]
pub struct FailureHistory {
    /// Task ID this history is for.
    pub task_id: String,
    /// List of failure records.
    pub failures: Vec<FailureRecord>,
    /// Number of retry attempts.
    pub retry_count: u32,
}

impl FailureHistory {
    /// Creates a new failure history for a task.
    pub fn new(task_id: String) -> Self {
        Self { task_id, failures: Vec::new(), retry_count: 0 }
    }

    /// Adds a failure record to the history.
    pub fn add_failure(&mut self, failure_type: FailureType, error_message: String) {
        self.failures.push(FailureRecord {
            timestamp: Utc::now(),
            failure_type,
            error_message,
        });
        self.retry_count += 1;
    }

    /// Gets the current retry count.
    pub fn get_retry_count(&self) -> u32 {
        self.retry_count
    }

    /// Checks if a retry should be attempted based on the failure policy.
    pub fn should_retry(&self, policy: &FailurePolicy) -> bool {
        if self.retry_count >= policy.max_retries {
            return false;
        }

        // Check if the last failure is recoverable
        if let Some(last_failure) = self.failures.last() {
            match &last_failure.failure_type {
                FailureType::Transient { .. } => true,
                FailureType::AgentFailure { .. } => true,
                FailureType::Permanent { .. } => policy.permanent_retry,
                FailureType::Unknown { .. } => false,
            }
        } else {
            false
        }
    }

    /// Gets the most recent failure type.
    pub fn last_failure_type(&self) -> Option<&FailureType> {
        self.failures.last().map(|f| &f.failure_type)
    }
}

/// Policy for handling failures and retries.
#[derive(Debug, Clone)]
pub struct FailurePolicy {
    /// Maximum number of retries allowed.
    pub max_retries: u32,
    /// Delay before retrying transient failures.
    pub transient_retry_delay: Duration,
    /// Whether to retry permanent failures.
    pub permanent_retry: bool,
}

impl Default for FailurePolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            transient_retry_delay: Duration::from_secs(5),
            permanent_retry: false,
        }
    }
}

impl FailurePolicy {
    /// Creates a new failure policy with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a custom failure policy.
    pub fn with_config(
        max_retries: u32,
        transient_retry_delay: Duration,
        permanent_retry: bool,
    ) -> Self {
        Self { max_retries, transient_retry_delay, permanent_retry }
    }

    /// Determines if a retry should be attempted based on failure history and type.
    pub fn should_retry(&self, history: &FailureHistory, failure_type: &FailureType) -> bool {
        if history.retry_count >= self.max_retries {
            return false;
        }

        match failure_type {
            FailureType::Transient { .. } => true,
            FailureType::AgentFailure { .. } => true,
            FailureType::Permanent { .. } => self.permanent_retry,
            FailureType::Unknown { .. } => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt;

    struct TestError {
        msg: String,
    }

    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.msg)
        }
    }

    impl Error for TestError {}

    #[test]
    fn test_classify_transient_error() {
        let classifier = FailureClassifier::new();
        let error = TestError { msg: "Connection timeout".to_string() };
        let failure_type = classifier.classify(&error);

        assert!(matches!(failure_type, FailureType::Transient { .. }));
        assert!(failure_type.is_recoverable());
    }

    #[test]
    fn test_classify_permanent_error() {
        let classifier = FailureClassifier::new();
        let error = TestError { msg: "Validation failed: invalid input".to_string() };
        let failure_type = classifier.classify(&error);

        assert!(matches!(failure_type, FailureType::Permanent { .. }));
        assert!(!failure_type.is_recoverable());
    }

    #[test]
    fn test_classify_agent_failure() {
        let classifier = FailureClassifier::new();
        let error = TestError {
            msg: "Agent not found: code-agent".to_string(),
        };
        let failure_type = classifier.classify(&error);

        match failure_type {
            FailureType::AgentFailure { agent_id, .. } => {
                assert_eq!(agent_id, "code-agent");
            }
            _ => panic!("Expected AgentFailure"),
        }
        assert!(failure_type.is_recoverable());
    }

    #[test]
    fn test_classify_from_string() {
        let classifier = FailureClassifier::new();
        let failure_type = classifier.classify_from_string("Network connection refused");

        assert!(matches!(failure_type, FailureType::Transient { .. }));
    }

    #[test]
    fn test_failure_history() {
        let mut history = FailureHistory::new("task-1".to_string());
        assert_eq!(history.get_retry_count(), 0);

        history.add_failure(
            FailureType::Transient { reason: "timeout".to_string() },
            "Connection timeout".to_string(),
        );
        assert_eq!(history.get_retry_count(), 1);

        let policy = FailurePolicy::default();
        assert!(history.should_retry(&policy));
    }

    #[test]
    fn test_failure_policy_retry_threshold() {
        let policy = FailurePolicy::with_config(3, Duration::from_secs(5), false);
        let mut history = FailureHistory::new("task-1".to_string());

        // Add 3 failures
        for _ in 0..3 {
            history.add_failure(
                FailureType::Transient { reason: "timeout".to_string() },
                "Timeout".to_string(),
            );
        }

        assert!(!history.should_retry(&policy));
        assert_eq!(history.get_retry_count(), 3);
    }

    #[test]
    fn test_failure_policy_should_retry() {
        let policy = FailurePolicy::default();
        let history = FailureHistory::new("task-1".to_string());

        assert!(policy.should_retry(
            &history,
            &FailureType::Transient { reason: "timeout".to_string() }
        ));
        assert!(!policy.should_retry(
            &history,
            &FailureType::Permanent { reason: "validation".to_string() }
        ));
    }

    #[test]
    fn test_failure_type_description() {
        let failure = FailureType::Transient { reason: "timeout".to_string() };
        let desc = failure.description();
        assert!(desc.contains("Transient"));
        assert!(desc.contains("timeout"));
    }
}

