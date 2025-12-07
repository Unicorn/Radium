//! Error types for agent collaboration features.

use crate::storage::error::StorageError;

/// Errors that can occur during agent collaboration operations.
#[derive(Debug, thiserror::Error)]
pub enum CollaborationError {
    /// Message delivery failed.
    #[error("message delivery failed to recipient {recipient_id}: {reason}")]
    MessageDeliveryError {
        /// ID of the recipient agent.
        recipient_id: String,
        /// Reason for delivery failure.
        reason: String,
    },

    /// Resource lock acquisition timed out.
    #[error("lock timeout for resource {resource_path} after {timeout_secs}s (held by: {})", holder_agent_id.as_ref().map(|id| id.as_str()).unwrap_or("unknown"))]
    LockTimeout {
        /// Path to the resource that couldn't be locked.
        resource_path: String,
        /// ID of the agent currently holding the lock, if any.
        holder_agent_id: Option<String>,
        /// Timeout duration in seconds.
        timeout_secs: u64,
    },

    /// Worker agent spawn failed.
    #[error("failed to spawn worker agent {worker_id}: {reason}")]
    WorkerSpawnError {
        /// ID of the worker agent that failed to spawn.
        worker_id: String,
        /// Reason for spawn failure.
        reason: String,
    },

    /// Maximum delegation depth exceeded.
    #[error("maximum delegation depth exceeded: current {current_depth}, max {max_depth}")]
    MaxDelegationDepthExceeded {
        /// Current delegation depth.
        current_depth: usize,
        /// Maximum allowed delegation depth.
        max_depth: usize,
    },

    /// Delegation relationship not found.
    #[error("delegation not found for worker {worker_id}")]
    DelegationNotFound {
        /// ID of the worker agent.
        worker_id: String,
    },

    /// Progress reporting failed.
    #[error("progress report failed for agent {agent_id}: {reason}")]
    ProgressReportError {
        /// ID of the agent reporting progress.
        agent_id: String,
        /// Reason for failure.
        reason: String,
    },

    /// Invalid message type.
    #[error("invalid message type: {message_type}")]
    InvalidMessageType {
        /// The invalid message type string.
        message_type: String,
    },

    /// Database operation error.
    #[error("database error: {0}")]
    DatabaseError(#[from] StorageError),
}

/// Result type for collaboration operations.
pub type Result<T> = std::result::Result<T, CollaborationError>;

