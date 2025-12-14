//! Activity state container
//!
//! Defines the state structure for individual activity execution,
//! including parameters, results, and execution context.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::schema::variables::VariableValue;

/// Activity execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ActivityStatus {
    /// Activity is pending execution
    #[default]
    Pending,
    /// Activity is currently executing
    Running,
    /// Activity completed successfully
    Completed,
    /// Activity failed
    Failed,
    /// Activity timed out
    TimedOut,
    /// Activity was cancelled
    Cancelled,
    /// Activity is being retried
    Retrying,
}

impl ActivityStatus {
    /// Check if this is a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::TimedOut | Self::Cancelled
        )
    }

    /// Check if the activity is actively running
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running | Self::Retrying)
    }
}

impl std::fmt::Display for ActivityStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Running => write!(f, "running"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::TimedOut => write!(f, "timedOut"),
            Self::Cancelled => write!(f, "cancelled"),
            Self::Retrying => write!(f, "retrying"),
        }
    }
}

/// Activity execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityContext {
    /// The activity's unique identifier
    pub activity_id: String,

    /// The activity type/name
    pub activity_type: String,

    /// The workflow this activity belongs to
    pub workflow_id: String,

    /// The task queue this activity runs on
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_queue: Option<String>,

    /// Attempt number (1-based)
    pub attempt: u32,

    /// Maximum allowed attempts
    pub max_attempts: u32,

    /// Scheduled time
    pub scheduled_at: DateTime<Utc>,

    /// Start time (when execution began)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<DateTime<Utc>>,

    /// Activity timeout
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_at: Option<DateTime<Utc>>,
}

impl ActivityContext {
    /// Create a new activity context
    pub fn new(
        activity_id: impl Into<String>,
        activity_type: impl Into<String>,
        workflow_id: impl Into<String>,
    ) -> Self {
        Self {
            activity_id: activity_id.into(),
            activity_type: activity_type.into(),
            workflow_id: workflow_id.into(),
            task_queue: None,
            attempt: 1,
            max_attempts: 3,
            scheduled_at: Utc::now(),
            started_at: None,
            timeout_at: None,
        }
    }

    /// Set task queue
    pub fn with_task_queue(mut self, queue: impl Into<String>) -> Self {
        self.task_queue = Some(queue.into());
        self
    }

    /// Set max attempts
    pub fn with_max_attempts(mut self, max: u32) -> Self {
        self.max_attempts = max;
        self
    }

    /// Check if this is a retry attempt
    pub fn is_retry(&self) -> bool {
        self.attempt > 1
    }

    /// Check if more retries are available
    pub fn can_retry(&self) -> bool {
        self.attempt < self.max_attempts
    }

    /// Increment attempt for retry
    pub fn next_attempt(&self) -> Self {
        Self {
            attempt: self.attempt + 1,
            scheduled_at: Utc::now(),
            started_at: None,
            ..self.clone()
        }
    }
}

/// Activity error information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityError {
    /// Error type/code
    pub error_type: String,
    /// Error message
    pub message: String,
    /// Whether this error is retryable
    pub retryable: bool,
    /// Additional error details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    /// When the error occurred
    pub occurred_at: DateTime<Utc>,
}

impl ActivityError {
    /// Create a new activity error
    pub fn new(error_type: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error_type: error_type.into(),
            message: message.into(),
            retryable: true,
            details: None,
            occurred_at: Utc::now(),
        }
    }

    /// Mark as non-retryable
    pub fn non_retryable(mut self) -> Self {
        self.retryable = false;
        self
    }

    /// Add details
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

/// State container for activity execution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityState {
    /// Activity execution context
    pub context: ActivityContext,

    /// Current execution status
    pub status: ActivityStatus,

    /// Input parameters
    pub params: HashMap<String, VariableValue>,

    /// Activity result (set on completion)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<VariableValue>,

    /// Error information (set on failure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ActivityError>,

    /// Local variables scoped to this activity
    pub local_variables: HashMap<String, VariableValue>,

    /// When the activity completed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,

    /// Duration of execution in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

impl ActivityState {
    /// Create a new activity state
    pub fn new(context: ActivityContext) -> Self {
        Self {
            context,
            status: ActivityStatus::Pending,
            params: HashMap::new(),
            result: None,
            error: None,
            local_variables: HashMap::new(),
            completed_at: None,
            duration_ms: None,
        }
    }

    /// Create with parameters
    pub fn with_params(mut self, params: HashMap<String, VariableValue>) -> Self {
        self.params = params;
        self
    }

    /// Start execution
    pub fn start(&mut self) {
        self.status = ActivityStatus::Running;
        self.context.started_at = Some(Utc::now());
    }

    /// Complete successfully with result
    pub fn complete(&mut self, result: VariableValue) {
        let now = Utc::now();
        self.status = ActivityStatus::Completed;
        self.result = Some(result);
        self.completed_at = Some(now);

        // Calculate duration
        if let Some(started) = self.context.started_at {
            self.duration_ms = Some((now - started).num_milliseconds() as u64);
        }
    }

    /// Fail with error
    pub fn fail(&mut self, error: ActivityError) {
        let now = Utc::now();
        self.status = ActivityStatus::Failed;
        self.error = Some(error);
        self.completed_at = Some(now);

        // Calculate duration
        if let Some(started) = self.context.started_at {
            self.duration_ms = Some((now - started).num_milliseconds() as u64);
        }
    }

    /// Mark as retrying
    pub fn retry(&mut self) -> Option<Self> {
        if !self.context.can_retry() {
            return None;
        }

        let new_context = self.context.next_attempt();
        let mut new_state = Self::new(new_context);
        new_state.params = self.params.clone();
        new_state.status = ActivityStatus::Retrying;
        Some(new_state)
    }

    /// Get a parameter value
    pub fn get_param(&self, name: &str) -> Option<&VariableValue> {
        self.params.get(name)
    }

    /// Get a local variable
    pub fn get_local(&self, name: &str) -> Option<&VariableValue> {
        self.local_variables.get(name)
    }

    /// Set a local variable
    pub fn set_local(&mut self, name: impl Into<String>, value: VariableValue) {
        self.local_variables.insert(name.into(), value);
    }

    /// Check if activity is complete
    pub fn is_complete(&self) -> bool {
        self.status.is_terminal()
    }

    /// Check if activity succeeded
    pub fn is_success(&self) -> bool {
        self.status == ActivityStatus::Completed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_status() {
        assert!(!ActivityStatus::Pending.is_terminal());
        assert!(!ActivityStatus::Running.is_terminal());
        assert!(ActivityStatus::Completed.is_terminal());
        assert!(ActivityStatus::Failed.is_terminal());

        assert!(!ActivityStatus::Pending.is_running());
        assert!(ActivityStatus::Running.is_running());
        assert!(ActivityStatus::Retrying.is_running());
    }

    #[test]
    fn test_activity_context() {
        let ctx = ActivityContext::new("act-1", "sendEmail", "wf-123")
            .with_task_queue("emails")
            .with_max_attempts(5);

        assert_eq!(ctx.activity_id, "act-1");
        assert_eq!(ctx.task_queue, Some("emails".to_string()));
        assert_eq!(ctx.max_attempts, 5);
        assert!(!ctx.is_retry());
        assert!(ctx.can_retry());
    }

    #[test]
    fn test_activity_retry() {
        let ctx = ActivityContext::new("act-1", "sendEmail", "wf-123")
            .with_max_attempts(3);

        let next = ctx.next_attempt();
        assert_eq!(next.attempt, 2);
        assert!(next.is_retry());
        assert!(next.can_retry());

        let next = next.next_attempt();
        assert_eq!(next.attempt, 3);
        assert!(!next.can_retry());
    }

    #[test]
    fn test_activity_state_lifecycle() {
        let ctx = ActivityContext::new("act-1", "processOrder", "wf-123");
        let mut state = ActivityState::new(ctx);

        assert_eq!(state.status, ActivityStatus::Pending);
        assert!(!state.is_complete());

        // Start
        state.start();
        assert_eq!(state.status, ActivityStatus::Running);
        assert!(state.context.started_at.is_some());

        // Complete
        state.complete(VariableValue::String("success".to_string()));
        assert_eq!(state.status, ActivityStatus::Completed);
        assert!(state.is_complete());
        assert!(state.is_success());
        assert!(state.result.is_some());
        assert!(state.duration_ms.is_some());
    }

    #[test]
    fn test_activity_failure() {
        let ctx = ActivityContext::new("act-1", "processOrder", "wf-123");
        let mut state = ActivityState::new(ctx);

        state.start();
        state.fail(ActivityError::new("TIMEOUT", "Request timed out"));

        assert_eq!(state.status, ActivityStatus::Failed);
        assert!(state.is_complete());
        assert!(!state.is_success());
        assert!(state.error.is_some());
    }

    #[test]
    fn test_activity_retry_state() {
        let ctx = ActivityContext::new("act-1", "processOrder", "wf-123")
            .with_max_attempts(3);
        let mut state = ActivityState::new(ctx)
            .with_params({
                let mut p = HashMap::new();
                p.insert("orderId".to_string(), VariableValue::String("order-1".to_string()));
                p
            });

        state.start();
        state.fail(ActivityError::new("NETWORK", "Connection failed"));

        // Retry
        let retry_state = state.retry();
        assert!(retry_state.is_some());

        let retry = retry_state.unwrap();
        assert_eq!(retry.context.attempt, 2);
        assert_eq!(retry.status, ActivityStatus::Retrying);
        assert_eq!(retry.get_param("orderId"), state.get_param("orderId"));
    }

    #[test]
    fn test_local_variables() {
        let ctx = ActivityContext::new("act-1", "processOrder", "wf-123");
        let mut state = ActivityState::new(ctx);

        state.set_local("tempCounter", VariableValue::Integer(5));
        assert_eq!(state.get_local("tempCounter"), Some(&VariableValue::Integer(5)));
    }

    #[test]
    fn test_activity_error() {
        let error = ActivityError::new("VALIDATION", "Invalid input")
            .non_retryable()
            .with_details(serde_json::json!({"field": "email"}));

        assert!(!error.retryable);
        assert!(error.details.is_some());
    }

    #[test]
    fn test_serialization() {
        let ctx = ActivityContext::new("act-1", "processOrder", "wf-123");
        let state = ActivityState::new(ctx);

        let json = serde_json::to_string_pretty(&state).unwrap();
        assert!(json.contains("\"activityId\": \"act-1\""));

        let parsed: ActivityState = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.context.activity_id, "act-1");
    }
}
