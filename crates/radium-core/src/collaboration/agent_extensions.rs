//! Agent extensions for collaboration features.

use crate::collaboration::delegation::DelegationManager;
use crate::collaboration::lock_manager::{LockHandle, ResourceLockManager};
use crate::collaboration::message_bus::{MessageBus, MessageType};
use crate::collaboration::progress::{ProgressStatus, ProgressTracker};
use crate::collaboration::Result;
use radium_orchestrator::AgentContext;
use std::sync::Arc;

/// Context providing access to collaboration features for agents.
pub struct CollaborationContext {
    /// Message bus for agent-to-agent communication.
    pub message_bus: Arc<MessageBus>,
    /// Lock manager for workspace coordination.
    pub lock_manager: Arc<ResourceLockManager>,
    /// Delegation manager for spawning workers.
    pub delegation_manager: Arc<DelegationManager>,
    /// Progress tracker for reporting progress.
    pub progress_tracker: Arc<ProgressTracker>,
    /// ID of the agent using this context.
    pub agent_id: String,
}

impl CollaborationContext {
    /// Creates a new collaboration context.
    pub fn new(
        agent_id: String,
        message_bus: Arc<MessageBus>,
        lock_manager: Arc<ResourceLockManager>,
        delegation_manager: Arc<DelegationManager>,
        progress_tracker: Arc<ProgressTracker>,
    ) -> Self {
        Self {
            message_bus,
            lock_manager,
            delegation_manager,
            progress_tracker,
            agent_id,
        }
    }

    /// Sends a message to a specific agent.
    ///
    /// # Arguments
    /// * `recipient_id` - ID of the recipient agent
    /// * `message_type` - Type of message
    /// * `payload` - Message payload as JSON value
    ///
    /// # Returns
    /// Returns the message ID if successful.
    pub async fn send_message(
        &self,
        recipient_id: &str,
        message_type: MessageType,
        payload: serde_json::Value,
    ) -> Result<String> {
        self.message_bus
            .send_message(&self.agent_id, recipient_id, message_type, payload)
            .await
    }

    /// Broadcasts a message to all agents.
    ///
    /// # Arguments
    /// * `message_type` - Type of message
    /// * `payload` - Message payload as JSON value
    ///
    /// # Returns
    /// Returns the message ID if successful.
    pub async fn broadcast_message(
        &self,
        message_type: MessageType,
        payload: serde_json::Value,
    ) -> Result<String> {
        self.message_bus
            .broadcast_message(&self.agent_id, message_type, payload)
            .await
    }

    /// Requests a read lock on a resource.
    ///
    /// # Arguments
    /// * `resource_path` - Path to the resource
    /// * `timeout_secs` - Optional timeout (uses default if None)
    ///
    /// # Returns
    /// Returns a `LockHandle` if the lock is acquired.
    pub async fn request_read_lock(
        &self,
        resource_path: &str,
        timeout_secs: Option<u64>,
    ) -> Result<LockHandle> {
        self.lock_manager
            .request_read_lock(&self.agent_id, resource_path, timeout_secs)
            .await
    }

    /// Requests a write lock on a resource.
    ///
    /// # Arguments
    /// * `resource_path` - Path to the resource
    /// * `timeout_secs` - Optional timeout (uses default if None)
    ///
    /// # Returns
    /// Returns a `LockHandle` if the lock is acquired.
    pub async fn request_write_lock(
        &self,
        resource_path: &str,
        timeout_secs: Option<u64>,
    ) -> Result<LockHandle> {
        self.lock_manager
            .request_write_lock(&self.agent_id, resource_path, timeout_secs)
            .await
    }

    /// Spawns a worker agent.
    ///
    /// # Arguments
    /// * `worker_agent_id` - ID of the worker agent to spawn
    /// * `task_input` - Input for the worker task
    /// * `delegation_depth` - Current delegation depth
    ///
    /// # Returns
    /// Returns the worker ID if successful.
    pub async fn spawn_worker(
        &self,
        worker_agent_id: &str,
        task_input: &str,
        delegation_depth: usize,
    ) -> Result<String> {
        self.delegation_manager
            .spawn_worker(&self.agent_id, worker_agent_id, task_input, delegation_depth)
            .await
    }

    /// Reports progress for this agent.
    ///
    /// # Arguments
    /// * `percentage` - Progress percentage (0-100)
    /// * `status` - Current status
    /// * `message` - Optional message describing the progress
    pub async fn report_progress(
        &self,
        percentage: u8,
        status: ProgressStatus,
        message: Option<String>,
    ) -> Result<()> {
        self.progress_tracker
            .report_progress(&self.agent_id, percentage, status, message)
            .await
    }
}

