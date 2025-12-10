//! Batch repository for database persistence.

use crate::models::batch::{BatchExecution, BatchRequestResult, BatchStatus, RequestStatus};
use crate::storage::error::{StorageError, StorageResult};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Row};
use std::path::Path;

/// Repository trait for batch execution data.
pub trait BatchRepository {
    /// Create a new batch execution record.
    fn create_batch(&mut self, batch: &BatchExecution) -> StorageResult<()>;

    /// Update an existing batch execution record.
    fn update_batch(&mut self, batch: &BatchExecution) -> StorageResult<()>;

    /// Get a batch execution by ID.
    fn get_batch(&self, batch_id: &str) -> StorageResult<Option<BatchExecution>>;

    /// List batch executions with pagination.
    fn list_batches(&self, limit: usize, offset: usize) -> StorageResult<Vec<BatchExecution>>;

    /// Add a request result to a batch.
    fn add_result(&mut self, result: &BatchRequestResult) -> StorageResult<()>;

    /// Get all results for a batch.
    fn get_results(&self, batch_id: &str) -> StorageResult<Vec<BatchRequestResult>>;
}

/// SQLite implementation of batch repository.
pub struct SqliteBatchRepository {
    conn: Connection,
}

impl SqliteBatchRepository {
    /// Create a new SQLite batch repository.
    pub fn new<P: AsRef<Path>>(db_path: P) -> StorageResult<Self> {
        let conn = Connection::open(db_path)?;
        Ok(Self { conn })
    }

    /// Create from existing connection.
    pub fn from_connection(conn: Connection) -> Self {
        Self { conn }
    }

    /// Helper to convert row to BatchExecution.
    fn row_to_batch_execution(row: &Row) -> rusqlite::Result<BatchExecution> {
        let status_str: String = row.get(9)?;
        let status = BatchStatus::from_str(&status_str)
            .map_err(|e| rusqlite::Error::InvalidColumnType(9, "status".to_string(), rusqlite::types::Type::Text))?;

        let started_at_str: String = row.get(7)?;
        let started_at = DateTime::parse_from_rfc3339(&started_at_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(7, "started_at".to_string(), rusqlite::types::Type::Text))?
            .with_timezone(&Utc);

        let completed_at: Option<String> = row.get(8)?;
        let completed_at = completed_at
            .map(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok()
            })
            .flatten();

        Ok(BatchExecution {
            batch_id: row.get(0)?,
            agent_id: row.get(1)?,
            total_requests: row.get(2)?,
            completed_requests: row.get(3)?,
            successful_requests: row.get(4)?,
            failed_requests: row.get(5)?,
            concurrency_limit: row.get(6)?,
            started_at,
            completed_at,
            status,
        })
    }

    /// Helper to convert row to BatchRequestResult.
    fn row_to_batch_request_result(row: &Row) -> rusqlite::Result<BatchRequestResult> {
        let status_str: String = row.get(6)?;
        let status = RequestStatus::from_str(&status_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(6, "status".to_string(), rusqlite::types::Type::Text))?;

        let started_at_str: String = row.get(8)?;
        let started_at = DateTime::parse_from_rfc3339(&started_at_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(8, "started_at".to_string(), rusqlite::types::Type::Text))?
            .with_timezone(&Utc);

        let completed_at: Option<String> = row.get(9)?;
        let completed_at = completed_at
            .map(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok()
            })
            .flatten();

        Ok(BatchRequestResult {
            id: row.get(0)?,
            batch_id: row.get(1)?,
            request_index: row.get(2)?,
            input: row.get(3)?,
            output: row.get(4)?,
            status,
            error_message: row.get(5)?,
            duration_ms: row.get(7)?,
            started_at,
            completed_at,
        })
    }
}

impl BatchRepository for SqliteBatchRepository {
    fn create_batch(&mut self, batch: &BatchExecution) -> StorageResult<()> {
        self.conn.execute(
            r#"
            INSERT INTO batch_executions (
                batch_id, agent_id, total_requests, completed_requests,
                successful_requests, failed_requests, concurrency_limit,
                started_at, completed_at, status
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![
                batch.batch_id,
                batch.agent_id,
                batch.total_requests,
                batch.completed_requests,
                batch.successful_requests,
                batch.failed_requests,
                batch.concurrency_limit,
                batch.started_at.to_rfc3339(),
                batch.completed_at.map(|dt| dt.to_rfc3339()),
                batch.status.as_str(),
            ],
        )?;
        Ok(())
    }

    fn update_batch(&mut self, batch: &BatchExecution) -> StorageResult<()> {
        self.conn.execute(
            r#"
            UPDATE batch_executions SET
                completed_requests = ?2,
                successful_requests = ?3,
                failed_requests = ?4,
                completed_at = ?5,
                status = ?6
            WHERE batch_id = ?1
            "#,
            params![
                batch.batch_id,
                batch.completed_requests,
                batch.successful_requests,
                batch.failed_requests,
                batch.completed_at.map(|dt| dt.to_rfc3339()),
                batch.status.as_str(),
            ],
        )?;
        Ok(())
    }

    fn get_batch(&self, batch_id: &str) -> StorageResult<Option<BatchExecution>> {
        let mut stmt = self.conn.prepare(
            "SELECT batch_id, agent_id, total_requests, completed_requests, successful_requests, failed_requests, concurrency_limit, started_at, completed_at, status FROM batch_executions WHERE batch_id = ?1"
        )?;
        let mut rows = stmt.query(params![batch_id])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_batch_execution(row)?))
        } else {
            Ok(None)
        }
    }

    fn list_batches(&self, limit: usize, offset: usize) -> StorageResult<Vec<BatchExecution>> {
        let mut stmt = self.conn.prepare(
            "SELECT batch_id, agent_id, total_requests, completed_requests, successful_requests, failed_requests, concurrency_limit, started_at, completed_at, status FROM batch_executions ORDER BY started_at DESC LIMIT ?1 OFFSET ?2"
        )?;
        let mut rows = stmt.query(params![limit as i64, offset as i64])?;
        let mut batches = Vec::new();

        while let Some(row) = rows.next()? {
            batches.push(Self::row_to_batch_execution(row)?);
        }

        Ok(batches)
    }

    fn add_result(&mut self, result: &BatchRequestResult) -> StorageResult<()> {
        self.conn.execute(
            r#"
            INSERT INTO batch_request_results (
                id, batch_id, request_index, input, output, status,
                error_message, duration_ms, started_at, completed_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![
                result.id,
                result.batch_id,
                result.request_index,
                result.input,
                result.output,
                result.status.as_str(),
                result.error_message,
                result.duration_ms,
                result.started_at.to_rfc3339(),
                result.completed_at.map(|dt| dt.to_rfc3339()),
            ],
        )?;
        Ok(())
    }

    fn get_results(&self, batch_id: &str) -> StorageResult<Vec<BatchRequestResult>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, batch_id, request_index, input, output, error_message, status, duration_ms, started_at, completed_at FROM batch_request_results WHERE batch_id = ?1 ORDER BY request_index"
        )?;
        let mut rows = stmt.query(params![batch_id])?;
        let mut results = Vec::new();

        while let Some(row) = rows.next()? {
            results.push(Self::row_to_batch_request_result(row)?);
        }

        Ok(results)
    }
}

