//! State management for workflow execution
//!
//! This module provides state containers for tracking workflow and activity
//! execution, including:
//!
//! - **WorkflowState** - Complete workflow execution state
//! - **ActivityState** - Individual activity execution state
//! - **StateSnapshot** - Minimal state for continue-as-new pattern
//!
//! ## Example Usage
//!
//! ```rust
//! use radium_workflow::schema::state::{
//!     WorkflowState, ActivityState, StateSnapshot,
//!     ExecutionState, ActivityStatus,
//! };
//! use radium_workflow::schema::variables::{VariableDefinition, VariableType, VariableValue};
//!
//! // Create workflow state with variable definitions
//! let definitions = vec![
//!     VariableDefinition::new("orderId", VariableType::String).required(),
//!     VariableDefinition::new("processedCount", VariableType::Integer),
//! ];
//!
//! let mut state = WorkflowState::new("wf-order-123", definitions);
//! state.transition_to(ExecutionState::Running).unwrap();
//! state.set("processedCount", VariableValue::Integer(5)).unwrap();
//!
//! // Create a snapshot for continue-as-new
//! let snapshot = StateSnapshot::from_workflow_state(&state, Some("node-5".to_string()));
//! ```

mod activity_state;
mod snapshot;
mod workflow_state;

pub use activity_state::{ActivityContext, ActivityError, ActivityState, ActivityStatus};
pub use snapshot::{BatchProgress, ContinuationInfo, ProgressMarker, StateSnapshot};
pub use workflow_state::{ExecutionState, FromVariableValue, StateError, WorkflowState};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::variables::{VariableDefinition, VariableType, VariableValue};

    /// Integration test for state management
    #[test]
    fn test_workflow_activity_lifecycle() {
        // Create workflow state
        let definitions = vec![
            VariableDefinition::new("orderId", VariableType::String).required(),
            VariableDefinition::new("itemCount", VariableType::Integer),
        ];

        let mut workflow_state = WorkflowState::new("wf-order-123", definitions);

        // Start workflow
        workflow_state.transition_to(ExecutionState::Running).unwrap();
        workflow_state.set("itemCount", VariableValue::Integer(10)).unwrap();

        // Create activity
        let ctx = ActivityContext::new("act-process", "processOrder", "wf-order-123");
        let mut activity = ActivityState::new(ctx)
            .with_params({
                let mut params = std::collections::HashMap::new();
                params.insert("orderId".to_string(), VariableValue::String("order-456".to_string()));
                params
            });

        // Execute activity
        activity.start();
        assert!(activity.context.started_at.is_some());

        // Complete activity
        activity.complete(VariableValue::Object(serde_json::json!({
            "success": true,
            "processedItems": 10
        })));

        assert!(activity.is_complete());
        assert!(activity.is_success());

        // Complete workflow
        workflow_state.set_output("result", VariableValue::String("completed".to_string())).unwrap();
        workflow_state.transition_to(ExecutionState::Completed).unwrap();

        assert!(workflow_state.is_complete());
    }

    /// Test continue-as-new flow
    #[test]
    fn test_continue_as_new_flow() {
        // Initial workflow
        let mut state = WorkflowState::new("wf-batch-001", vec![
            VariableDefinition::new("processedCount", VariableType::Integer),
            VariableDefinition::new("totalItems", VariableType::Integer),
        ]);

        state.transition_to(ExecutionState::Running).unwrap();
        state.set("processedCount", VariableValue::Integer(1000)).unwrap();
        state.set("totalItems", VariableValue::Integer(5000)).unwrap();

        // Create snapshot for continue-as-new
        let mut snapshot = StateSnapshot::from_workflow_state(&state, Some("batch-node".to_string()));

        // Record batch progress
        let batch = BatchProgress::new(5000, 1000);
        snapshot.progress = snapshot.progress.with_batch_progress(batch);

        // Simulate continuation
        let snapshot2 = StateSnapshot::continue_from(&snapshot);
        assert_eq!(snapshot2.continuation.continuation_count, 1);
        assert_eq!(snapshot2.get("processedCount"), Some(&VariableValue::Integer(1000)));
    }

    /// Test activity retry flow
    #[test]
    fn test_activity_retry_flow() {
        let ctx = ActivityContext::new("act-1", "sendEmail", "wf-123")
            .with_max_attempts(3);

        let mut activity = ActivityState::new(ctx);
        activity.start();

        // First failure
        activity.fail(ActivityError::new("NETWORK", "Connection timeout"));

        // Retry
        let retry1 = activity.retry();
        assert!(retry1.is_some());
        let mut retry1 = retry1.unwrap();
        assert_eq!(retry1.context.attempt, 2);

        retry1.start();
        retry1.fail(ActivityError::new("NETWORK", "Connection timeout again"));

        // Second retry
        let retry2 = retry1.retry();
        assert!(retry2.is_some());
        let mut retry2 = retry2.unwrap();
        assert_eq!(retry2.context.attempt, 3);

        retry2.start();
        retry2.fail(ActivityError::new("NETWORK", "Still failing"));

        // No more retries
        let retry3 = retry2.retry();
        assert!(retry3.is_none());
    }

    /// Test state validation
    #[test]
    fn test_state_type_validation() {
        let definitions = vec![
            VariableDefinition::new("count", VariableType::Integer),
        ];

        let mut state = WorkflowState::new("wf-123", definitions);

        // Valid assignment
        assert!(state.set("count", VariableValue::Integer(42)).is_ok());

        // Invalid type
        let result = state.set("count", VariableValue::String("not a number".to_string()));
        assert!(result.is_err());
    }
}
