//! Agent usage analytics and metrics tracking.
//!
//! Provides analytics for agent usage, performance, and popularity.

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use thiserror::Error;

/// Analytics errors.
#[derive(Debug, Error)]
pub enum AnalyticsError {
    /// Database error.
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    /// Invalid query parameters.
    #[error("invalid query: {0}")]
    InvalidQuery(String),
}

/// Result type for analytics operations.
pub type Result<T> = std::result::Result<T, AnalyticsError>;

/// Agent usage record for a single execution.
#[derive(Debug, Clone)]
pub struct UsageRecord {
    /// Agent ID.
    pub agent_id: String,
    /// Execution duration in milliseconds.
    pub duration_ms: u64,
    /// Token usage.
    pub tokens: u64,
    /// Whether execution was successful.
    pub success: bool,
    /// Agent category (if available).
    pub category: Option<String>,
}

/// Aggregated agent usage statistics.
#[derive(Debug, Clone)]
pub struct AgentAnalytics {
    /// Agent ID.
    pub agent_id: String,
    /// Total number of executions.
    pub execution_count: u64,
    /// Total duration across all executions (milliseconds).
    pub total_duration_ms: u64,
    /// Average duration per execution (milliseconds).
    pub avg_duration_ms: f64,
    /// Total tokens used.
    pub total_tokens: u64,
    /// Number of successful executions.
    pub success_count: u64,
    /// Number of failed executions.
    pub failure_count: u64,
    /// Success rate (0.0 to 1.0).
    pub success_rate: f64,
    /// Last used timestamp.
    pub last_used_at: Option<DateTime<Utc>>,
    /// Agent category.
    pub category: Option<String>,
}

impl AgentAnalytics {
    /// Creates analytics from database row.
    fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        let agent_id: String = row.get(0)?;
        let execution_count: i64 = row.get(1)?;
        let total_duration_ms: i64 = row.get(2)?;
        let total_tokens: i64 = row.get(3)?;
        let success_count: i64 = row.get(4)?;
        let failure_count: i64 = row.get(5)?;
        let last_used_at: Option<i64> = row.get(6)?;
        let category: Option<String> = row.get(7)?;

        let execution_count = execution_count.max(0) as u64;
        let total_duration_ms = total_duration_ms.max(0) as u64;
        let total_tokens = total_tokens.max(0) as u64;
        let success_count = success_count.max(0) as u64;
        let failure_count = failure_count.max(0) as u64;

        let avg_duration_ms = if execution_count > 0 {
            total_duration_ms as f64 / execution_count as f64
        } else {
            0.0
        };

        let total_executions = success_count + failure_count;
        let success_rate = if total_executions > 0 {
            success_count as f64 / total_executions as f64
        } else {
            0.0
        };

        let last_used_at = last_used_at.map(|ts| {
            DateTime::from_timestamp(ts, 0).unwrap_or_else(|| Utc::now())
        });

        Ok(Self {
            agent_id,
            execution_count,
            total_duration_ms,
            avg_duration_ms,
            total_tokens,
            success_count,
            failure_count,
            success_rate,
            last_used_at,
            category,
        })
    }
}

/// Agent analytics service.
pub struct AgentAnalyticsService {
    conn: Connection,
}

impl AgentAnalyticsService {
    /// Creates a new analytics service with a database connection.
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }

    /// Records agent usage.
    pub fn record_usage(&self, record: &UsageRecord) -> Result<()> {
        let timestamp = Utc::now().timestamp();

        // Insert or update agent usage
        self.conn.execute(
            "INSERT INTO agent_usage (agent_id, execution_count, total_duration, total_tokens, 
                                     success_count, failure_count, last_used_at, category)
             VALUES (?1, 1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(agent_id) DO UPDATE SET
                 execution_count = execution_count + 1,
                 total_duration = total_duration + ?2,
                 total_tokens = total_tokens + ?3,
                 success_count = success_count + ?4,
                 failure_count = failure_count + ?5,
                 last_used_at = ?6,
                 category = COALESCE(?7, category)",
            params![
                record.agent_id,
                record.duration_ms as i64,
                record.tokens as i64,
                if record.success { 1 } else { 0 },
                if record.success { 0 } else { 1 },
                timestamp,
                record.category,
            ],
        )?;

        Ok(())
    }

    /// Gets analytics for a specific agent.
    pub fn get_agent_analytics(&self, agent_id: &str) -> Result<Option<AgentAnalytics>> {
        let mut stmt = self.conn.prepare(
            "SELECT agent_id, execution_count, total_duration, total_tokens, 
                    success_count, failure_count, last_used_at, category
             FROM agent_usage
             WHERE agent_id = ?1"
        )?;

        let result = stmt.query_row(params![agent_id], |row| {
            AgentAnalytics::from_row(row)
        });

        match result {
            Ok(analytics) => Ok(Some(analytics)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AnalyticsError::Database(e)),
        }
    }

    /// Gets analytics for all agents.
    pub fn get_all_analytics(&self) -> Result<Vec<AgentAnalytics>> {
        let mut stmt = self.conn.prepare(
            "SELECT agent_id, execution_count, total_duration, total_tokens, 
                    success_count, failure_count, last_used_at, category
             FROM agent_usage
             ORDER BY execution_count DESC"
        )?;

        let rows = stmt.query_map([], |row| AgentAnalytics::from_row(row))?;
        let mut analytics = Vec::new();
        for row in rows {
            analytics.push(row?);
        }

        Ok(analytics)
    }

    /// Gets most popular agents (by execution count).
    pub fn get_popular_agents(&self, limit: usize) -> Result<Vec<AgentAnalytics>> {
        let mut stmt = self.conn.prepare(
            "SELECT agent_id, execution_count, total_duration, total_tokens, 
                    success_count, failure_count, last_used_at, category
             FROM agent_usage
             ORDER BY execution_count DESC
             LIMIT ?1"
        )?;

        let rows = stmt.query_map(params![limit as i64], |row| AgentAnalytics::from_row(row))?;
        let mut analytics = Vec::new();
        for row in rows {
            analytics.push(row?);
        }

        Ok(analytics)
    }

    /// Gets analytics by category.
    pub fn get_analytics_by_category(&self, category: &str) -> Result<Vec<AgentAnalytics>> {
        let mut stmt = self.conn.prepare(
            "SELECT agent_id, execution_count, total_duration, total_tokens, 
                    success_count, failure_count, last_used_at, category
             FROM agent_usage
             WHERE category = ?1
             ORDER BY execution_count DESC"
        )?;

        let rows = stmt.query_map(params![category], |row| AgentAnalytics::from_row(row))?;
        let mut analytics = Vec::new();
        for row in rows {
            analytics.push(row?);
        }

        Ok(analytics)
    }

    /// Gets performance metrics (slowest agents by average duration).
    pub fn get_performance_metrics(&self, limit: usize) -> Result<Vec<AgentAnalytics>> {
        let mut stmt = self.conn.prepare(
            "SELECT agent_id, execution_count, total_duration, total_tokens, 
                    success_count, failure_count, last_used_at, category
             FROM agent_usage
             WHERE execution_count > 0
             ORDER BY (CAST(total_duration AS REAL) / execution_count) DESC
             LIMIT ?1"
        )?;

        let rows = stmt.query_map(params![limit as i64], |row| AgentAnalytics::from_row(row))?;
        let mut analytics = Vec::new();
        for row in rows {
            analytics.push(row?);
        }

        Ok(analytics)
    }

    /// Gets overall statistics.
    pub fn get_overall_stats(&self) -> Result<OverallStats> {
        let mut stmt = self.conn.prepare(
            "SELECT 
                COUNT(*) as total_agents,
                SUM(execution_count) as total_executions,
                SUM(total_duration) as total_duration,
                SUM(total_tokens) as total_tokens,
                SUM(success_count) as total_successes,
                SUM(failure_count) as total_failures
             FROM agent_usage"
        )?;

        let row = stmt.query_row([], |row| {
            Ok(OverallStats {
                total_agents: row.get::<_, i64>(0)? as u64,
                total_executions: row.get::<_, i64>(1)? as u64,
                total_duration_ms: row.get::<_, i64>(2)? as u64,
                total_tokens: row.get::<_, i64>(3)? as u64,
                total_successes: row.get::<_, i64>(4)? as u64,
                total_failures: row.get::<_, i64>(5)? as u64,
            })
        })?;

        Ok(row)
    }
}

/// Overall statistics across all agents.
#[derive(Debug, Clone)]
pub struct OverallStats {
    /// Total number of agents with usage data.
    pub total_agents: u64,
    /// Total number of executions.
    pub total_executions: u64,
    /// Total duration across all executions (milliseconds).
    pub total_duration_ms: u64,
    /// Total tokens used.
    pub total_tokens: u64,
    /// Total successful executions.
    pub total_successes: u64,
    /// Total failed executions.
    pub total_failures: u64,
}

impl OverallStats {
    /// Gets average duration per execution.
    pub fn avg_duration_ms(&self) -> f64 {
        if self.total_executions > 0 {
            self.total_duration_ms as f64 / self.total_executions as f64
        } else {
            0.0
        }
    }

    /// Gets overall success rate.
    pub fn success_rate(&self) -> f64 {
        let total = self.total_successes + self.total_failures;
        if total > 0 {
            self.total_successes as f64 / total as f64
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn create_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::monitoring::initialize_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn test_record_usage() {
        let conn = create_test_db();
        let service = AgentAnalyticsService::new(conn);

        let record = UsageRecord {
            agent_id: "test-agent".to_string(),
            duration_ms: 1000,
            tokens: 5000,
            success: true,
            category: Some("core".to_string()),
        };

        service.record_usage(&record).unwrap();

        let analytics = service.get_agent_analytics("test-agent").unwrap().unwrap();
        assert_eq!(analytics.execution_count, 1);
        assert_eq!(analytics.total_duration_ms, 1000);
        assert_eq!(analytics.total_tokens, 5000);
        assert_eq!(analytics.success_count, 1);
        assert_eq!(analytics.failure_count, 0);
    }

    #[test]
    fn test_get_popular_agents() {
        let conn = create_test_db();
        let service = AgentAnalyticsService::new(conn);

        // Record usage for multiple agents
        service.record_usage(&UsageRecord {
            agent_id: "agent-1".to_string(),
            duration_ms: 1000,
            tokens: 1000,
            success: true,
            category: None,
        }).unwrap();

        service.record_usage(&UsageRecord {
            agent_id: "agent-2".to_string(),
            duration_ms: 2000,
            tokens: 2000,
            success: true,
            category: None,
        }).unwrap();

        service.record_usage(&UsageRecord {
            agent_id: "agent-2".to_string(),
            duration_ms: 2000,
            tokens: 2000,
            success: true,
            category: None,
        }).unwrap();

        let popular = service.get_popular_agents(10).unwrap();
        assert_eq!(popular.len(), 2);
        assert_eq!(popular[0].agent_id, "agent-2"); // Most executions
        assert_eq!(popular[0].execution_count, 2);
    }
}

