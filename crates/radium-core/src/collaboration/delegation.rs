//! Task delegation system for supervisor-worker patterns.

use crate::collaboration::error::{CollaborationError, Result};
use crate::collaboration::message_bus::{MessageBus, MessageType};
use crate::storage::database::Database;
use crate::storage::error::StorageError;
use radium_orchestrator::Orchestrator;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error, warn};

/// Status of a worker agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerStatus {
    /// Worker has been spawned but not yet started.
    Spawned,
    /// Worker is currently running.
    Running,
    /// Worker completed successfully.
    Completed,
    /// Worker failed with an error.
    Failed,
    /// Worker was cancelled.
    Cancelled,
}

impl WorkerStatus {
    /// Converts a string to a WorkerStatus.
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "Spawned" => Ok(WorkerStatus::Spawned),
            "Running" => Ok(WorkerStatus::Running),
            "Completed" => Ok(WorkerStatus::Completed),
            "Failed" => Ok(WorkerStatus::Failed),
            "Cancelled" => Ok(WorkerStatus::Cancelled),
            _ => Err(CollaborationError::WorkerSpawnError {
                worker_id: "unknown".to_string(),
                reason: format!("Invalid status: {}", s),
            }),
        }
    }

    /// Converts a WorkerStatus to a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            WorkerStatus::Spawned => "Spawned",
            WorkerStatus::Running => "Running",
            WorkerStatus::Completed => "Completed",
            WorkerStatus::Failed => "Failed",
            WorkerStatus::Cancelled => "Cancelled",
        }
    }
}

/// Information about a delegation relationship.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationInfo {
    /// ID of the supervisor agent.
    pub supervisor_id: String,
    /// ID of the worker agent.
    pub worker_id: String,
    /// Timestamp when worker was spawned (Unix epoch seconds).
    pub spawned_at: i64,
    /// Timestamp when worker completed (Unix epoch seconds, None if not completed).
    pub completed_at: Option<i64>,
    /// Current status of the worker.
    pub status: WorkerStatus,
}

/// Repository trait for delegation persistence.
pub trait DelegationRepository: Send + Sync {
    /// Stores a delegation relationship.
    fn store_delegation(&self, delegation: &DelegationInfo) -> Result<()>;

    /// Retrieves a delegation by worker ID.
    fn get_delegation(&self, worker_id: &str) -> Result<Option<DelegationInfo>>;

    /// Retrieves all delegations for a supervisor.
    fn get_supervisor_delegations(&self, supervisor_id: &str) -> Result<Vec<DelegationInfo>>;

    /// Updates a delegation's status and completion time.
    fn update_delegation(
        &self,
        worker_id: &str,
        status: WorkerStatus,
        completed_at: Option<i64>,
    ) -> Result<()>;
}

/// Database-backed delegation repository.
pub struct DatabaseDelegationRepository {
    db: Arc<StdMutex<Database>>,
}

impl DatabaseDelegationRepository {
    /// Creates a new database delegation repository.
    pub fn new(db: Arc<StdMutex<Database>>) -> Self {
        Self { db }
    }
}

impl DelegationRepository for DatabaseDelegationRepository {
    fn store_delegation(&self, delegation: &DelegationInfo) -> Result<()> {
        let delegation = delegation.clone();
        let mut db = self.db.lock().map_err(|e| {
            CollaborationError::DatabaseError(StorageError::InvalidData(format!(
                "Database lock error: {}",
                e
            )))
        })?;
        let conn = db.conn_mut();

        conn.execute(
            "INSERT OR REPLACE INTO agent_delegations (supervisor_id, worker_id, spawned_at, completed_at, status) VALUES (?, ?, ?, ?, ?)",
            rusqlite::params![
                delegation.supervisor_id,
                delegation.worker_id,
                delegation.spawned_at,
                delegation.completed_at,
                delegation.status.as_str()
            ],
        )
        .map_err(|e| CollaborationError::DatabaseError(StorageError::Connection(e)))?;

        Ok(())
    }

    fn get_delegation(&self, worker_id: &str) -> Result<Option<DelegationInfo>> {
        let worker_id = worker_id.to_string();
        let db = self.db.lock().map_err(|e| {
            CollaborationError::DatabaseError(StorageError::InvalidData(format!(
                "Database lock error: {}",
                e
            )))
        })?;
        let conn = db.conn();

        let mut stmt = conn
            .prepare(
                "SELECT supervisor_id, worker_id, spawned_at, completed_at, status FROM agent_delegations WHERE worker_id = ?",
            )
            .map_err(|e| CollaborationError::DatabaseError(StorageError::Connection(e)))?;

        match stmt.query_row([worker_id], |row| {
            let status_str: String = row.get(4)?;
            let status = WorkerStatus::from_str(&status_str).map_err(|_| {
                rusqlite::Error::InvalidColumnType(
                    4,
                    "status".to_string(),
                    rusqlite::types::Type::Text,
                )
            })?;

            Ok(DelegationInfo {
                supervisor_id: row.get(0)?,
                worker_id: row.get(1)?,
                spawned_at: row.get(2)?,
                completed_at: row.get(3)?,
                status,
            })
        }) {
            Ok(delegation) => Ok(Some(delegation)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(CollaborationError::DatabaseError(StorageError::Connection(e))),
        }
    }

    fn get_supervisor_delegations(&self, supervisor_id: &str) -> Result<Vec<DelegationInfo>> {
        let supervisor_id = supervisor_id.to_string();
        let db = self.db.lock().map_err(|e| {
            CollaborationError::DatabaseError(StorageError::InvalidData(format!(
                "Database lock error: {}",
                e
            )))
        })?;
        let conn = db.conn();

        let mut stmt = conn
            .prepare(
                "SELECT supervisor_id, worker_id, spawned_at, completed_at, status FROM agent_delegations WHERE supervisor_id = ?",
            )
            .map_err(|e| CollaborationError::DatabaseError(StorageError::Connection(e)))?;

        let delegations = stmt
            .query_map([supervisor_id], |row| {
                let status_str: String = row.get(4)?;
                let status = WorkerStatus::from_str(&status_str).map_err(|_| {
                    rusqlite::Error::InvalidColumnType(
                        4,
                        "status".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })?;

                Ok(DelegationInfo {
                    supervisor_id: row.get(0)?,
                    worker_id: row.get(1)?,
                    spawned_at: row.get(2)?,
                    completed_at: row.get(3)?,
                    status,
                })
            })
            .map_err(|e| CollaborationError::DatabaseError(StorageError::Connection(e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| CollaborationError::DatabaseError(StorageError::Connection(e)))?;

        Ok(delegations)
    }

    fn update_delegation(
        &self,
        worker_id: &str,
        status: WorkerStatus,
        completed_at: Option<i64>,
    ) -> Result<()> {
        let worker_id = worker_id.to_string();
        let mut db = self.db.lock().map_err(|e| {
            CollaborationError::DatabaseError(StorageError::InvalidData(format!(
                "Database lock error: {}",
                e
            )))
        })?;
        let conn = db.conn_mut();

        conn.execute(
            "UPDATE agent_delegations SET status = ?, completed_at = ? WHERE worker_id = ?",
            rusqlite::params![status.as_str(), completed_at, worker_id],
        )
        .map_err(|e| CollaborationError::DatabaseError(StorageError::Connection(e)))?;

        Ok(())
    }
}

/// Maximum delegation depth allowed.
const MAX_DELEGATION_DEPTH: usize = 3;

/// Delegation manager for supervisor-worker patterns.
pub struct DelegationManager {
    /// Database repository for delegation persistence.
    repository: Arc<dyn DelegationRepository>,
    /// Message bus for sending events to supervisors.
    message_bus: Arc<MessageBus>,
    /// Orchestrator for spawning worker agents.
    orchestrator: Arc<Orchestrator>,
}

impl DelegationManager {
    /// Creates a new delegation manager.
    pub fn new(
        db: Arc<StdMutex<Database>>,
        message_bus: Arc<MessageBus>,
        orchestrator: Arc<Orchestrator>,
    ) -> Self {
        let repository: Arc<dyn DelegationRepository> =
            Arc::new(DatabaseDelegationRepository::new(db));
        Self {
            repository,
            message_bus,
            orchestrator,
        }
    }

    /// Spawns a worker agent for a supervisor.
    ///
    /// # Arguments
    /// * `supervisor_id` - ID of the supervisor agent
    /// * `worker_agent_id` - ID of the worker agent to spawn
    /// * `task_input` - Input for the worker task
    /// * `delegation_depth` - Current delegation depth
    ///
    /// # Returns
    /// Returns the worker ID if successful.
    pub async fn spawn_worker(
        &self,
        supervisor_id: &str,
        worker_agent_id: &str,
        task_input: &str,
        delegation_depth: usize,
    ) -> Result<String> {
        // Check delegation depth
        if delegation_depth >= MAX_DELEGATION_DEPTH {
            return Err(CollaborationError::MaxDelegationDepthExceeded {
                current_depth: delegation_depth,
                max_depth: MAX_DELEGATION_DEPTH,
            });
        }

        // Check if worker agent exists
        if self.orchestrator.get_agent(worker_agent_id).await.is_none() {
            return Err(CollaborationError::WorkerSpawnError {
                worker_id: worker_agent_id.to_string(),
                reason: "Worker agent not found in orchestrator".to_string(),
            });
        }

        let spawned_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let delegation = DelegationInfo {
            supervisor_id: supervisor_id.to_string(),
            worker_id: worker_agent_id.to_string(),
            spawned_at,
            completed_at: None,
            status: WorkerStatus::Spawned,
        };

        // Store delegation
        self.repository.store_delegation(&delegation)?;

        // Update status to Running
        self.repository
            .update_delegation(worker_agent_id, WorkerStatus::Running, None)?;

        // Spawn worker via orchestrator (this is a simplified version - actual implementation
        // would need to pass context with parent_agent_id and delegation_depth)
        // For now, we'll just execute the agent and handle completion asynchronously
        let orchestrator = Arc::clone(&self.orchestrator);
        let message_bus = Arc::clone(&self.message_bus);
        let supervisor_id = supervisor_id.to_string();
        let worker_id = worker_agent_id.to_string();
        let repository = Arc::clone(&self.repository);

        tokio::spawn(async move {
            debug!(
                supervisor_id = %supervisor_id,
                worker_id = %worker_id,
                "Spawning worker agent"
            );

            // Execute worker agent
            match orchestrator.execute_agent(&worker_id, task_input).await {
                Ok(result) => {
                    let completed_at = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64;

                    // Update delegation status
                    let _ = repository.update_delegation(
                        &worker_id,
                        WorkerStatus::Completed,
                        Some(completed_at),
                    );

                    // Send completion event to supervisor
                    let payload = serde_json::json!({
                        "worker_id": worker_id,
                        "success": result.success,
                        "output": result.output,
                    });

                    let _ = message_bus
                        .send_message(
                            &worker_id,
                            &supervisor_id,
                            MessageType::TaskResponse,
                            payload,
                        )
                        .await;
                }
                Err(e) => {
                    let completed_at = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64;

                    // Update delegation status
                    let _ = repository.update_delegation(
                        &worker_id,
                        WorkerStatus::Failed,
                        Some(completed_at),
                    );

                    // Send failure event to supervisor
                    let payload = serde_json::json!({
                        "worker_id": worker_id,
                        "error": e.to_string(),
                    });

                    let _ = message_bus
                        .send_message(
                            &worker_id,
                            &supervisor_id,
                            MessageType::TaskResponse,
                            payload,
                        )
                        .await;
                }
            }
        });

        Ok(worker_agent_id.to_string())
    }

    /// Gets the status of a worker agent.
    ///
    /// # Arguments
    /// * `worker_id` - ID of the worker agent
    ///
    /// # Returns
    /// Returns the worker status if found.
    pub async fn get_worker_status(&self, worker_id: &str) -> Result<WorkerStatus> {
        let delegation = self.repository.get_delegation(worker_id)?;
        delegation
            .map(|d| d.status)
            .ok_or_else(|| CollaborationError::DelegationNotFound {
                worker_id: worker_id.to_string(),
            })
    }

    /// Cancels a worker agent.
    ///
    /// # Arguments
    /// * `worker_id` - ID of the worker agent to cancel
    pub async fn cancel_worker(&self, worker_id: &str) -> Result<()> {
        // Update status to Cancelled
        self.repository
            .update_delegation(worker_id, WorkerStatus::Cancelled, None)?;

        // Stop the agent in orchestrator
        let _ = self.orchestrator.stop_agent(worker_id).await;

        debug!(worker_id = %worker_id, "Worker cancelled");

        Ok(())
    }

    /// Cancels all workers for a supervisor.
    ///
    /// # Arguments
    /// * `supervisor_id` - ID of the supervisor agent
    ///
    /// # Returns
    /// Returns a list of cancelled worker IDs.
    pub async fn cancel_all_workers(&self, supervisor_id: &str) -> Result<Vec<String>> {
        let delegations = self.repository.get_supervisor_delegations(supervisor_id)?;

        let mut cancelled_ids = Vec::new();
        for delegation in delegations {
            if delegation.status == WorkerStatus::Running || delegation.status == WorkerStatus::Spawned {
                if let Err(e) = self.cancel_worker(&delegation.worker_id).await {
                    warn!(
                        worker_id = %delegation.worker_id,
                        error = %e,
                        "Failed to cancel worker"
                    );
                } else {
                    cancelled_ids.push(delegation.worker_id);
                }
            }
        }

        Ok(cancelled_ids)
    }

    /// Gets the output from a completed worker.
    ///
    /// # Arguments
    /// * `worker_id` - ID of the worker agent
    ///
    /// # Returns
    /// Returns the worker output if available, None if worker hasn't completed or failed.
    pub async fn get_worker_output(&self, worker_id: &str) -> Result<Option<String>> {
        let delegation = self.repository.get_delegation(worker_id)?;
        match delegation {
            Some(d) if d.status == WorkerStatus::Completed => {
                // In a real implementation, we'd retrieve the actual output from the execution result
                // For now, we return None as the output is sent via message bus
                Ok(None)
            }
            Some(_) => Ok(None),
            None => Err(CollaborationError::DelegationNotFound {
                worker_id: worker_id.to_string(),
            }),
        }
    }
}

