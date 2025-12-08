//! Analytics repository for persisting permission analytics data.

use crate::monitoring::permission_analytics::PermissionEvent;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result as SqliteResult};
use std::path::Path;

/// Analytics repository for storing permission events.
pub struct AnalyticsRepository {
    conn: Connection,
}

impl AnalyticsRepository {
    /// Creates a new analytics repository.
    pub fn new<P: AsRef<Path>>(db_path: P) -> SqliteResult<Self> {
        let conn = Connection::open(db_path)?;
        let repo = Self { conn };
        repo.initialize_schema()?;
        Ok(repo)
    }

    /// Initializes the database schema.
    fn initialize_schema(&self) -> SqliteResult<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS permission_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                tool_name TEXT NOT NULL,
                args TEXT NOT NULL,
                agent_id TEXT,
                outcome TEXT NOT NULL,
                matched_rule TEXT,
                reason TEXT
            )",
            [],
        )?;
        
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_permission_events_timestamp ON permission_events(timestamp)",
            [],
        )?;
        
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_permission_events_tool_name ON permission_events(tool_name)",
            [],
        )?;
        
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_permission_events_agent_id ON permission_events(agent_id)",
            [],
        )?;
        
        Ok(())
    }

    /// Stores a permission event.
    pub fn store_event(&self, event: &PermissionEvent) -> SqliteResult<()> {
        self.conn.execute(
            "INSERT INTO permission_events (timestamp, tool_name, args, agent_id, outcome, matched_rule, reason)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                event.timestamp.timestamp(),
                event.tool_name,
                serde_json::to_string(&event.args).unwrap_or_default(),
                event.agent_id,
                format!("{:?}", event.outcome).to_lowercase(),
                event.matched_rule,
                event.reason,
            ],
        )?;
        Ok(())
    }

    /// Gets events in a date range.
    pub fn get_events_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> SqliteResult<Vec<PermissionEvent>> {
        let mut stmt = self.conn.prepare(
            "SELECT timestamp, tool_name, args, agent_id, outcome, matched_rule, reason
             FROM permission_events
             WHERE timestamp >= ?1 AND timestamp <= ?2
             ORDER BY timestamp DESC"
        )?;
        
        let rows = stmt.query_map(
            params![start.timestamp(), end.timestamp()],
            |row| {
                let args_json: String = row.get(2)?;
                let args: Vec<String> = serde_json::from_str(&args_json).unwrap_or_default();
                
                Ok(PermissionEvent {
                    timestamp: DateTime::from_timestamp(row.get::<_, i64>(0)?, 0)
                        .unwrap_or_else(Utc::now),
                    tool_name: row.get(1)?,
                    args,
                    agent_id: row.get(3)?,
                    outcome: match row.get::<_, String>(4)?.as_str() {
                        "allowed" => crate::monitoring::permission_analytics::PermissionOutcome::Allowed,
                        "denied" => crate::monitoring::permission_analytics::PermissionOutcome::Denied,
                        "asked" => crate::monitoring::permission_analytics::PermissionOutcome::Asked,
                        _ => crate::monitoring::permission_analytics::PermissionOutcome::Asked,
                    },
                    matched_rule: row.get(5)?,
                    reason: row.get(6)?,
                })
            },
        )?;
        
        let mut events = Vec::new();
        for row in rows {
            events.push(row?);
        }
        Ok(events)
    }

    /// Gets all events.
    pub fn get_all_events(&self) -> SqliteResult<Vec<PermissionEvent>> {
        let start = DateTime::from_timestamp(0, 0).unwrap_or_else(Utc::now);
        let end = Utc::now() + chrono::Duration::days(365);
        self.get_events_in_range(start, end)
    }
}

