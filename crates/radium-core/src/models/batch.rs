//! Batch execution models for database persistence.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Batch execution status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BatchStatus {
    /// Batch is running.
    Running,
    /// Batch completed successfully.
    Completed,
    /// Batch failed.
    Failed,
    /// Batch was cancelled.
    Cancelled,
}

impl BatchStatus {
    /// Convert to string representation.
    pub fn as_str(&self) -> &str {
        match self {
            BatchStatus::Running => "RUNNING",
            BatchStatus::Completed => "COMPLETED",
            BatchStatus::Failed => "FAILED",
            BatchStatus::Cancelled => "CANCELLED",
        }
    }

    /// Parse from string.
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "RUNNING" => Ok(BatchStatus::Running),
            "COMPLETED" => Ok(BatchStatus::Completed),
            "FAILED" => Ok(BatchStatus::Failed),
            "CANCELLED" => Ok(BatchStatus::Cancelled),
            _ => Err(format!("Invalid batch status: {}", s)),
        }
    }
}

/// Batch execution record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchExecution {
    /// Unique batch ID.
    pub batch_id: String,
    /// Agent ID (optional).
    pub agent_id: Option<String>,
    /// Total number of requests.
    pub total_requests: i32,
    /// Number of completed requests.
    pub completed_requests: i32,
    /// Number of successful requests.
    pub successful_requests: i32,
    /// Number of failed requests.
    pub failed_requests: i32,
    /// Concurrency limit.
    pub concurrency_limit: i32,
    /// Start time.
    pub started_at: DateTime<Utc>,
    /// Completion time (optional).
    pub completed_at: Option<DateTime<Utc>>,
    /// Batch status.
    pub status: BatchStatus,
}

impl Default for BatchExecution {
    fn default() -> Self {
        Self {
            batch_id: String::new(),
            agent_id: None,
            total_requests: 0,
            completed_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            concurrency_limit: 5,
            started_at: Utc::now(),
            completed_at: None,
            status: BatchStatus::Running,
        }
    }
}

/// Request result status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RequestStatus {
    /// Request succeeded.
    Success,
    /// Request failed.
    Failed,
    /// Request timed out.
    Timeout,
    /// Request was cancelled.
    Cancelled,
}

impl RequestStatus {
    /// Convert to string representation.
    pub fn as_str(&self) -> &str {
        match self {
            RequestStatus::Success => "SUCCESS",
            RequestStatus::Failed => "FAILED",
            RequestStatus::Timeout => "TIMEOUT",
            RequestStatus::Cancelled => "CANCELLED",
        }
    }

    /// Parse from string.
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "SUCCESS" => Ok(RequestStatus::Success),
            "FAILED" => Ok(RequestStatus::Failed),
            "TIMEOUT" => Ok(RequestStatus::Timeout),
            "CANCELLED" => Ok(RequestStatus::Cancelled),
            _ => Err(format!("Invalid request status: {}", s)),
        }
    }
}

/// Batch request result record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRequestResult {
    /// Unique result ID.
    pub id: String,
    /// Batch ID.
    pub batch_id: String,
    /// Request index in batch.
    pub request_index: i32,
    /// Input data (JSON string).
    pub input: String,
    /// Output data (JSON string, optional).
    pub output: Option<String>,
    /// Request status.
    pub status: RequestStatus,
    /// Error message (optional).
    pub error_message: Option<String>,
    /// Duration in milliseconds.
    pub duration_ms: i64,
    /// Start time.
    pub started_at: DateTime<Utc>,
    /// Completion time (optional).
    pub completed_at: Option<DateTime<Utc>>,
}

impl Default for BatchRequestResult {
    fn default() -> Self {
        Self {
            id: String::new(),
            batch_id: String::new(),
            request_index: 0,
            input: String::new(),
            output: None,
            status: RequestStatus::Success,
            error_message: None,
            duration_ms: 0,
            started_at: Utc::now(),
            completed_at: None,
        }
    }
}

