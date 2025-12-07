//! Progress tracking and reporting system.

use crate::collaboration::error::{CollaborationError, Result};
use crate::storage::database::Database;
use crate::storage::error::StorageError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex as StdMutex};
use tracing::debug;

/// Status of an agent's progress.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProgressStatus {
    /// Agent is idle.
    Idle,
    /// Agent is working.
    Working,
    /// Agent is waiting for something.
    Waiting,
    /// Agent is blocked.
    Blocked,
    /// Agent has completed.
    Complete,
    /// Agent encountered an error.
    Error,
}

impl ProgressStatus {
    /// Converts a string to a ProgressStatus.
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "Idle" => Ok(ProgressStatus::Idle),
            "Working" => Ok(ProgressStatus::Working),
            "Waiting" => Ok(ProgressStatus::Waiting),
            "Blocked" => Ok(ProgressStatus::Blocked),
            "Complete" => Ok(ProgressStatus::Complete),
            "Error" => Ok(ProgressStatus::Error),
            _ => Err(CollaborationError::ProgressReportError {
                agent_id: "unknown".to_string(),
                reason: format!("Invalid status: {}", s),
            }),
        }
    }

    /// Converts a ProgressStatus to a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            ProgressStatus::Idle => "Idle",
            ProgressStatus::Working => "Working",
            ProgressStatus::Waiting => "Waiting",
            ProgressStatus::Blocked => "Blocked",
            ProgressStatus::Complete => "Complete",
            ProgressStatus::Error => "Error",
        }
    }
}

/// A snapshot of an agent's progress at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressSnapshot {
    /// ID of the agent.
    pub agent_id: String,
    /// Progress percentage (0-100).
    pub percentage: u8,
    /// Current status.
    pub status: ProgressStatus,
    /// Optional message describing the progress.
    pub message: Option<String>,
    /// Timestamp when this snapshot was taken (Unix epoch seconds).
    pub timestamp: i64,
}

/// Aggregated progress across multiple agents.
#[derive(Debug, Clone)]
pub struct AggregatedProgress {
    /// Average progress percentage across all agents.
    pub average_percentage: f64,
    /// Count of agents in each status.
    pub status_counts: HashMap<ProgressStatus, usize>,
    /// Individual worker statuses.
    pub worker_statuses: Vec<ProgressSnapshot>,
}

/// Repository trait for progress persistence.
pub trait ProgressRepository: Send + Sync {
    /// Stores a progress snapshot.
    fn store_progress(&self, snapshot: &ProgressSnapshot) -> Result<()>;

    /// Retrieves the latest progress for an agent.
    fn get_latest_progress(&self, agent_id: &str) -> Result<Option<ProgressSnapshot>>;

    /// Retrieves progress for multiple agents.
    fn get_progress_for_agents(&self, agent_ids: &[String]) -> Result<Vec<ProgressSnapshot>>;
}

/// Database-backed progress repository.
pub struct DatabaseProgressRepository {
    db: Arc<StdMutex<Database>>,
}

impl DatabaseProgressRepository {
    /// Creates a new database progress repository.
    pub fn new(db: Arc<StdMutex<Database>>) -> Self {
        Self { db }
    }
}

impl ProgressRepository for DatabaseProgressRepository {
    fn store_progress(&self, snapshot: &ProgressSnapshot) -> Result<()> {
        let snapshot = snapshot.clone();
        let mut db = self.db.lock().map_err(|e| {
            CollaborationError::DatabaseError(StorageError::InvalidData(format!(
                "Database lock error: {}",
                e
            )))
        })?;
        let conn = db.conn_mut();

        conn.execute(
            "INSERT INTO agent_progress (agent_id, timestamp, percentage, status, message) VALUES (?, ?, ?, ?, ?)",
            rusqlite::params![
                snapshot.agent_id,
                snapshot.timestamp,
                snapshot.percentage as i64,
                snapshot.status.as_str(),
                snapshot.message
            ],
        )
        .map_err(|e| CollaborationError::DatabaseError(StorageError::Connection(e)))?;

        Ok(())
    }

    fn get_latest_progress(&self, agent_id: &str) -> Result<Option<ProgressSnapshot>> {
        let agent_id = agent_id.to_string();
        let db = self.db.lock().map_err(|e| {
            CollaborationError::DatabaseError(StorageError::InvalidData(format!(
                "Database lock error: {}",
                e
            )))
        })?;
        let conn = db.conn();

        let mut stmt = conn
            .prepare(
                "SELECT agent_id, timestamp, percentage, status, message FROM agent_progress WHERE agent_id = ? ORDER BY timestamp DESC LIMIT 1",
            )
            .map_err(|e| CollaborationError::DatabaseError(StorageError::Connection(e)))?;

        match stmt.query_row([agent_id], |row| {
            let status_str: String = row.get(3)?;
            let status = ProgressStatus::from_str(&status_str).map_err(|_| {
                rusqlite::Error::InvalidColumnType(
                    3,
                    "status".to_string(),
                    rusqlite::types::Type::Text,
                )
            })?;

            Ok(ProgressSnapshot {
                agent_id: row.get(0)?,
                timestamp: row.get(1)?,
                percentage: row.get::<_, i64>(2)? as u8,
                status,
                message: row.get(4)?,
            })
        }) {
            Ok(snapshot) => Ok(Some(snapshot)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(CollaborationError::DatabaseError(StorageError::Connection(e))),
        }
    }

    fn get_progress_for_agents(&self, agent_ids: &[String]) -> Result<Vec<ProgressSnapshot>> {
        if agent_ids.is_empty() {
            return Ok(Vec::new());
        }

        let db = self.db.lock().map_err(|e| {
            CollaborationError::DatabaseError(StorageError::InvalidData(format!(
                "Database lock error: {}",
                e
            )))
        })?;
        let conn = db.conn();

        // Build query with placeholders
        let placeholders = vec!["?"; agent_ids.len()].join(",");
        let query = format!(
            "SELECT agent_id, timestamp, percentage, status, message FROM agent_progress WHERE agent_id IN ({}) AND id IN (SELECT MAX(id) FROM agent_progress WHERE agent_id IN ({}) GROUP BY agent_id)",
            placeholders, placeholders
        );

        let mut stmt = conn
            .prepare(&query)
            .map_err(|e| CollaborationError::DatabaseError(StorageError::Connection(e)))?;

        // Create params array
        let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();
        for id in agent_ids {
            params.push(id);
        }
        for id in agent_ids {
            params.push(id);
        }

        let snapshots = stmt
            .query_map(rusqlite::params_from_iter(params.iter()), |row| {
                let status_str: String = row.get(3)?;
                let status = ProgressStatus::from_str(&status_str).map_err(|_| {
                    rusqlite::Error::InvalidColumnType(
                        3,
                        "status".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })?;

                Ok(ProgressSnapshot {
                    agent_id: row.get(0)?,
                    timestamp: row.get(1)?,
                    percentage: row.get::<_, i64>(2)? as u8,
                    status,
                    message: row.get(4)?,
                })
            })
            .map_err(|e| CollaborationError::DatabaseError(StorageError::Connection(e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| CollaborationError::DatabaseError(StorageError::Connection(e)))?;

        Ok(snapshots)
    }
}

/// Progress tracker for monitoring agent execution progress.
pub struct ProgressTracker {
    /// Database repository for progress persistence.
    repository: Arc<dyn ProgressRepository>,
}

impl ProgressTracker {
    /// Creates a new progress tracker.
    pub fn new(db: Arc<StdMutex<Database>>) -> Self {
        let repository: Arc<dyn ProgressRepository> = Arc::new(DatabaseProgressRepository::new(db));
        Self { repository }
    }

    /// Reports progress for an agent.
    ///
    /// # Arguments
    /// * `agent_id` - ID of the agent
    /// * `percentage` - Progress percentage (0-100)
    /// * `status` - Current status
    /// * `message` - Optional message describing the progress
    pub async fn report_progress(
        &self,
        agent_id: &str,
        percentage: u8,
        status: ProgressStatus,
        message: Option<String>,
    ) -> Result<()> {
        if percentage > 100 {
            return Err(CollaborationError::ProgressReportError {
                agent_id: agent_id.to_string(),
                reason: "Percentage must be between 0 and 100".to_string(),
            });
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let snapshot = ProgressSnapshot {
            agent_id: agent_id.to_string(),
            percentage,
            status,
            message,
            timestamp,
        };

        self.repository.store_progress(&snapshot)?;

        debug!(
            agent_id = %agent_id,
            percentage = percentage,
            status = ?status,
            "Progress reported"
        );

        Ok(())
    }

    /// Gets the latest progress for an agent.
    ///
    /// # Arguments
    /// * `agent_id` - ID of the agent
    ///
    /// # Returns
    /// Returns the latest progress snapshot, or None if no progress has been reported.
    pub async fn get_progress(&self, agent_id: &str) -> Result<Option<ProgressSnapshot>> {
        self.repository.get_latest_progress(agent_id)
    }

    /// Gets aggregated progress across multiple agents.
    ///
    /// # Arguments
    /// * `agent_ids` - IDs of the agents to aggregate
    ///
    /// # Returns
    /// Returns aggregated progress information.
    pub async fn get_aggregated_progress(
        &self,
        agent_ids: &[String],
    ) -> Result<AggregatedProgress> {
        let snapshots = self.repository.get_progress_for_agents(agent_ids)?;

        if snapshots.is_empty() {
            return Ok(AggregatedProgress {
                average_percentage: 0.0,
                status_counts: HashMap::new(),
                worker_statuses: Vec::new(),
            });
        }

        let total_percentage: u64 = snapshots.iter().map(|s| s.percentage as u64).sum();
        let average_percentage = total_percentage as f64 / snapshots.len() as f64;

        let mut status_counts = HashMap::new();
        for snapshot in &snapshots {
            *status_counts.entry(snapshot.status).or_insert(0) += 1;
        }

        Ok(AggregatedProgress {
            average_percentage,
            status_counts,
            worker_statuses: snapshots,
        })
    }
}

