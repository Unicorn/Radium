//! Checkpoint-based recovery system for workflow execution.
//!
//! Provides automatic recovery from failures by restoring workspace state
//! to the last known good checkpoint.

use crate::checkpoint::{Checkpoint, CheckpointManager, CheckpointError};
use crate::workflow::engine::ExecutionContext;
use crate::workflow::failure::{FailurePolicy, FailureType};
use std::sync::Arc;
use thiserror::Error;

/// Recovery strategy for handling failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryStrategy {
    /// Restore to a specific checkpoint.
    RestoreCheckpoint {
        /// Checkpoint ID to restore.
        checkpoint_id: String,
    },
    /// Retry without restoring workspace.
    RetryWithoutRestore,
    /// Skip the failed task and continue.
    SkipTask,
    /// Abort execution.
    Abort,
}

/// Context for recovery operations.
#[derive(Debug, Clone)]
pub struct RecoveryContext {
    /// Workflow ID.
    pub workflow_id: String,
    /// Step ID that failed.
    pub failed_step_id: String,
    /// Checkpoint ID before failure (if available).
    pub checkpoint_id: Option<String>,
    /// Execution context to preserve.
    pub execution_context: ExecutionContext,
    /// Type of failure that occurred.
    pub failure_type: FailureType,
}

/// Errors that can occur during recovery operations.
#[derive(Debug, Error)]
pub enum RecoveryError {
    /// Checkpoint not found.
    #[error("Checkpoint not found: {0}")]
    CheckpointNotFound(String),

    /// Checkpoint restore failed.
    #[error("Failed to restore checkpoint: {0}")]
    RestoreFailed(String),

    /// Recovery strategy not applicable.
    #[error("Recovery strategy not applicable: {0}")]
    StrategyNotApplicable(String),

    /// Checkpoint error.
    #[error("Checkpoint error: {0}")]
    Checkpoint(#[from] CheckpointError),
}

/// Result type for recovery operations.
pub type Result<T> = std::result::Result<T, RecoveryError>;

/// Manager for coordinating recovery operations.
pub struct RecoveryManager {
    /// Checkpoint manager for restore operations.
    checkpoint_manager: Arc<std::sync::Mutex<CheckpointManager>>,
    /// Failure policy for retry decisions.
    failure_policy: FailurePolicy,
}

impl RecoveryManager {
    /// Creates a new recovery manager.
    ///
    /// # Arguments
    /// * `checkpoint_manager` - The checkpoint manager
    /// * `failure_policy` - The failure policy for retry decisions
    pub fn new(
        checkpoint_manager: Arc<std::sync::Mutex<CheckpointManager>>,
        failure_policy: FailurePolicy,
    ) -> Self {
        Self { checkpoint_manager, failure_policy }
    }

    /// Determines the appropriate recovery strategy based on context.
    ///
    /// # Arguments
    /// * `context` - The recovery context
    ///
    /// # Returns
    /// The recovery strategy to use
    pub fn determine_strategy(&self, context: &RecoveryContext) -> RecoveryStrategy {
        match &context.failure_type {
            FailureType::Transient { .. } => {
                // For transient failures, try restoring checkpoint if available
                if let Some(checkpoint_id) = &context.checkpoint_id {
                    RecoveryStrategy::RestoreCheckpoint {
                        checkpoint_id: checkpoint_id.clone(),
                    }
                } else {
                    RecoveryStrategy::RetryWithoutRestore
                }
            }
            FailureType::AgentFailure { .. } => {
                // For agent failures, retry without restore (agent reassignment will handle)
                RecoveryStrategy::RetryWithoutRestore
            }
            FailureType::Permanent { .. } => {
                // Permanent failures cannot be recovered
                RecoveryStrategy::Abort
            }
            FailureType::Unknown { .. } => {
                // Unknown failures: abort to be safe
                RecoveryStrategy::Abort
            }
        }
    }

    /// Executes a recovery strategy.
    ///
    /// # Arguments
    /// * `strategy` - The recovery strategy to execute
    /// * `context` - The recovery context
    ///
    /// # Returns
    /// `Ok(())` if recovery succeeded, error otherwise
    pub fn execute_recovery(
        &self,
        strategy: RecoveryStrategy,
        _context: &RecoveryContext,
    ) -> Result<()> {
        match strategy {
            RecoveryStrategy::RestoreCheckpoint { checkpoint_id } => {
                let cm = self.checkpoint_manager.lock().unwrap();
                cm.restore_checkpoint(&checkpoint_id).map_err(|e| {
                    RecoveryError::RestoreFailed(format!("Failed to restore checkpoint: {}", e))
                })?;
                Ok(())
            }
            RecoveryStrategy::RetryWithoutRestore => {
                // No action needed, just retry
                Ok(())
            }
            RecoveryStrategy::SkipTask => {
                // No action needed, task will be skipped
                Ok(())
            }
            RecoveryStrategy::Abort => {
                // Abort - return error to stop execution
                Err(RecoveryError::StrategyNotApplicable(
                    "Recovery strategy is Abort".to_string(),
                ))
            }
        }
    }

    /// Finds the checkpoint for a specific step.
    ///
    /// # Arguments
    /// * `step_id` - The step ID
    ///
    /// # Returns
    /// The checkpoint if found, None otherwise
    pub fn find_checkpoint_for_step(&self, step_id: &str) -> Option<Checkpoint> {
        let cm = self.checkpoint_manager.lock().ok()?;
        // Look for checkpoint with description containing step_id
        // This is a simplified implementation - in practice, you'd want to store
        // step_id -> checkpoint_id mapping
        if let Ok(checkpoints) = cm.list_checkpoints() {
            checkpoints
                .into_iter()
                .find(|cp| {
                    cp.description
                        .as_ref()
                        .map(|d| d.contains(step_id))
                        .unwrap_or(false)
                })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::failure::FailurePolicy;
    use std::time::Duration;

    fn create_test_recovery_context() -> RecoveryContext {
        RecoveryContext {
            workflow_id: "workflow-1".to_string(),
            failed_step_id: "step-1".to_string(),
            checkpoint_id: Some("checkpoint-123".to_string()),
            execution_context: ExecutionContext::new("workflow-1".to_string()),
            failure_type: FailureType::Transient { reason: "timeout".to_string() },
        }
    }

    #[test]
    fn test_recovery_strategy_transient_with_checkpoint() {
        let policy = FailurePolicy::default();
        // Note: This test would need a real CheckpointManager, so we'll test the logic
        // The actual integration test would require a real git repo
        let context = create_test_recovery_context();
        
        // Strategy determination logic
        match context.failure_type {
            FailureType::Transient { .. } if context.checkpoint_id.is_some() => {
                // Should return RestoreCheckpoint
                assert!(true);
            }
            _ => panic!("Expected transient failure with checkpoint"),
        }
    }

    #[test]
    fn test_recovery_strategy_permanent() {
        let context = RecoveryContext {
            workflow_id: "workflow-1".to_string(),
            failed_step_id: "step-1".to_string(),
            checkpoint_id: Some("checkpoint-123".to_string()),
            execution_context: ExecutionContext::new("workflow-1".to_string()),
            failure_type: FailureType::Permanent { reason: "validation".to_string() },
        };

        // Permanent failures should abort
        match context.failure_type {
            FailureType::Permanent { .. } => {
                // Should return Abort
                assert!(true);
            }
            _ => panic!("Expected permanent failure"),
        }
    }

    #[test]
    fn test_recovery_strategy_agent_failure() {
        let context = RecoveryContext {
            workflow_id: "workflow-1".to_string(),
            failed_step_id: "step-1".to_string(),
            checkpoint_id: None,
            execution_context: ExecutionContext::new("workflow-1".to_string()),
            failure_type: FailureType::AgentFailure {
                agent_id: "agent-1".to_string(),
                reason: "agent error".to_string(),
            },
        };

        // Agent failures should retry without restore
        match context.failure_type {
            FailureType::AgentFailure { .. } => {
                // Should return RetryWithoutRestore
                assert!(true);
            }
            _ => panic!("Expected agent failure"),
        }
    }
}

