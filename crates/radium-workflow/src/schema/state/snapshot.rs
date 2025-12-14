//! State snapshot for continue-as-new
//!
//! Provides minimal state representation for Temporal's continue-as-new
//! pattern, preserving only essential data for workflow continuation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::schema::variables::VariableValue;
use super::WorkflowState;

/// Progress marker for tracking workflow execution position
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProgressMarker {
    /// Last completed activity/node ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_completed: Option<String>,

    /// Iteration counters for loops (loop_id -> iteration count)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub iterations: HashMap<String, u64>,

    /// Batch processing progress
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_progress: Option<BatchProgress>,
}

impl ProgressMarker {
    /// Create a new progress marker
    pub fn new() -> Self {
        Self::default()
    }

    /// Set last completed node
    pub fn with_last_completed(mut self, node_id: impl Into<String>) -> Self {
        self.last_completed = Some(node_id.into());
        self
    }

    /// Record loop iteration
    pub fn record_iteration(&mut self, loop_id: impl Into<String>) {
        let loop_id = loop_id.into();
        let count = self.iterations.entry(loop_id).or_insert(0);
        *count += 1;
    }

    /// Get iteration count for a loop
    pub fn get_iteration(&self, loop_id: &str) -> u64 {
        *self.iterations.get(loop_id).unwrap_or(&0)
    }

    /// Set batch progress
    pub fn with_batch_progress(mut self, progress: BatchProgress) -> Self {
        self.batch_progress = Some(progress);
        self
    }
}

/// Batch processing progress
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchProgress {
    /// Total items to process
    pub total_items: u64,
    /// Items processed so far
    pub processed_items: u64,
    /// Current batch number (0-indexed)
    pub current_batch: u64,
    /// Batch size
    pub batch_size: u64,
}

impl BatchProgress {
    /// Create new batch progress
    pub fn new(total_items: u64, batch_size: u64) -> Self {
        Self {
            total_items,
            processed_items: 0,
            current_batch: 0,
            batch_size,
        }
    }

    /// Calculate total number of batches
    pub fn total_batches(&self) -> u64 {
        (self.total_items + self.batch_size - 1) / self.batch_size
    }

    /// Check if all items have been processed
    pub fn is_complete(&self) -> bool {
        self.processed_items >= self.total_items
    }

    /// Advance to next batch
    pub fn next_batch(&mut self, processed_in_batch: u64) {
        self.processed_items += processed_in_batch;
        self.current_batch += 1;
    }

    /// Get remaining items
    pub fn remaining_items(&self) -> u64 {
        self.total_items.saturating_sub(self.processed_items)
    }

    /// Get progress as percentage
    pub fn progress_percent(&self) -> f64 {
        if self.total_items == 0 {
            100.0
        } else {
            (self.processed_items as f64 / self.total_items as f64) * 100.0
        }
    }
}

/// Information about workflow continuation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuationInfo {
    /// Original workflow ID (from the first run)
    pub original_workflow_id: String,

    /// Number of times this workflow has been continued
    pub continuation_count: u32,

    /// When the original workflow started
    pub started_at: DateTime<Utc>,

    /// When this continuation was created
    pub continued_at: DateTime<Utc>,

    /// Reason for continuation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl ContinuationInfo {
    /// Create new continuation info
    pub fn new(original_workflow_id: impl Into<String>, started_at: DateTime<Utc>) -> Self {
        Self {
            original_workflow_id: original_workflow_id.into(),
            continuation_count: 0,
            started_at,
            continued_at: Utc::now(),
            reason: None,
        }
    }

    /// Create for a continuation
    pub fn from_previous(previous: &ContinuationInfo) -> Self {
        Self {
            original_workflow_id: previous.original_workflow_id.clone(),
            continuation_count: previous.continuation_count + 1,
            started_at: previous.started_at,
            continued_at: Utc::now(),
            reason: None,
        }
    }

    /// Set continuation reason
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Total elapsed time since original start
    pub fn total_elapsed(&self) -> chrono::Duration {
        Utc::now() - self.started_at
    }
}

/// Minimal state snapshot for continue-as-new
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateSnapshot {
    /// Variables to carry forward
    pub variables: HashMap<String, VariableValue>,

    /// Current progress marker
    pub progress: ProgressMarker,

    /// Continuation information
    pub continuation: ContinuationInfo,
}

impl StateSnapshot {
    /// Create a snapshot from workflow state
    pub fn from_workflow_state(state: &WorkflowState, last_completed: Option<String>) -> Self {
        Self {
            variables: state.variables.clone(),
            progress: ProgressMarker {
                last_completed,
                iterations: HashMap::new(),
                batch_progress: None,
            },
            continuation: ContinuationInfo::new(&state.workflow_id, state.created_at),
        }
    }

    /// Create a continuation snapshot from a previous snapshot
    pub fn continue_from(previous: &StateSnapshot) -> Self {
        Self {
            variables: previous.variables.clone(),
            progress: previous.progress.clone(),
            continuation: ContinuationInfo::from_previous(&previous.continuation),
        }
    }

    /// Get a variable value
    pub fn get(&self, name: &str) -> Option<&VariableValue> {
        self.variables.get(name)
    }

    /// Set a variable value
    pub fn set(&mut self, name: impl Into<String>, value: VariableValue) {
        self.variables.insert(name.into(), value);
    }

    /// Record loop iteration
    pub fn record_iteration(&mut self, loop_id: impl Into<String>) {
        self.progress.record_iteration(loop_id);
    }

    /// Get serialized size in bytes (approximate)
    pub fn serialized_size(&self) -> usize {
        serde_json::to_vec(self).map(|v| v.len()).unwrap_or(0)
    }

    /// Check if snapshot exceeds size limit
    pub fn exceeds_size_limit(&self, max_bytes: usize) -> bool {
        self.serialized_size() > max_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_marker() {
        let mut marker = ProgressMarker::new()
            .with_last_completed("node-5");

        assert_eq!(marker.last_completed, Some("node-5".to_string()));

        marker.record_iteration("loop-1");
        marker.record_iteration("loop-1");
        assert_eq!(marker.get_iteration("loop-1"), 2);
        assert_eq!(marker.get_iteration("loop-2"), 0);
    }

    #[test]
    fn test_batch_progress() {
        let mut batch = BatchProgress::new(100, 25);

        assert_eq!(batch.total_batches(), 4);
        assert!(!batch.is_complete());
        assert_eq!(batch.remaining_items(), 100);

        batch.next_batch(25);
        assert_eq!(batch.processed_items, 25);
        assert_eq!(batch.current_batch, 1);
        assert_eq!(batch.progress_percent(), 25.0);

        batch.next_batch(25);
        batch.next_batch(25);
        batch.next_batch(25);
        assert!(batch.is_complete());
    }

    #[test]
    fn test_continuation_info() {
        let started = Utc::now();
        let info = ContinuationInfo::new("wf-original", started)
            .with_reason("History size exceeded");

        assert_eq!(info.original_workflow_id, "wf-original");
        assert_eq!(info.continuation_count, 0);
        assert!(info.reason.is_some());

        let next = ContinuationInfo::from_previous(&info);
        assert_eq!(next.continuation_count, 1);
        assert_eq!(next.original_workflow_id, "wf-original");
    }

    #[test]
    fn test_state_snapshot_from_workflow() {
        let mut state = WorkflowState::new("wf-123", vec![]);
        state.set("counter", VariableValue::Integer(42)).unwrap();
        state.set("name", VariableValue::String("test".to_string())).unwrap();

        let snapshot = StateSnapshot::from_workflow_state(&state, Some("node-5".to_string()));

        assert_eq!(snapshot.get("counter"), Some(&VariableValue::Integer(42)));
        assert_eq!(snapshot.progress.last_completed, Some("node-5".to_string()));
        assert_eq!(snapshot.continuation.original_workflow_id, "wf-123");
    }

    #[test]
    fn test_snapshot_continuation() {
        let mut state = WorkflowState::new("wf-123", vec![]);
        state.set("counter", VariableValue::Integer(42)).unwrap();

        let snapshot1 = StateSnapshot::from_workflow_state(&state, None);
        let snapshot2 = StateSnapshot::continue_from(&snapshot1);

        assert_eq!(snapshot2.continuation.continuation_count, 1);
        assert_eq!(snapshot2.get("counter"), Some(&VariableValue::Integer(42)));
    }

    #[test]
    fn test_snapshot_size() {
        let mut state = WorkflowState::new("wf-123", vec![]);
        state.set("small", VariableValue::Integer(1)).unwrap();

        let snapshot = StateSnapshot::from_workflow_state(&state, None);
        let size = snapshot.serialized_size();
        assert!(size > 0);
        assert!(!snapshot.exceeds_size_limit(1024 * 1024)); // 1MB limit
    }

    #[test]
    fn test_serialization() {
        let mut state = WorkflowState::new("wf-123", vec![]);
        state.set("data", VariableValue::String("test".to_string())).unwrap();

        let snapshot = StateSnapshot::from_workflow_state(&state, Some("node-1".to_string()));
        let json = serde_json::to_string_pretty(&snapshot).unwrap();

        let parsed: StateSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.get("data"), snapshot.get("data"));
        assert_eq!(parsed.progress.last_completed, Some("node-1".to_string()));
    }
}
