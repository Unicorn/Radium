//! Database storage for policy analytics.

use rusqlite::{params, Connection, Result as SqliteResult};
use std::sync::{Arc, Mutex};

/// Policy event record stored in the database.
#[derive(Debug, Clone)]
pub struct PolicyEvent {
    /// Event ID (auto-increment).
    pub id: Option<i64>,
    /// Timestamp of the event (Unix timestamp).
    pub timestamp: i64,
    /// Tool name that was evaluated.
    pub tool_name: String,
    /// Arguments passed to the tool (JSON).
    pub arguments: String,
    /// Policy action taken (allow, deny, ask_user, dry_run_first).
    pub action: String,
    /// Matched rule name (if any).
    pub matched_rule: Option<String>,
    /// Reason for the decision.
    pub reason: Option<String>,
    /// Optional user/agent identifier.
    pub user: Option<String>,
}

/// Repository for policy analytics storage.
pub struct PolicyAnalyticsStorage {
    conn: Arc<Mutex<Connection>>,
}

impl PolicyAnalyticsStorage {
    /// Creates a new policy analytics storage.
    ///
    /// # Arguments
    /// * `conn` - SQLite connection (shared)
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        let storage = Self { conn };
        storage.init_schema().expect("Failed to initialize policy analytics schema");
        storage
    }

    /// Initializes the database schema for policy analytics.
    fn init_schema(&self) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        
        // Create policy_events table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS policy_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                tool_name TEXT NOT NULL,
                arguments TEXT NOT NULL,
                action TEXT NOT NULL,
                matched_rule TEXT,
                reason TEXT,
                user TEXT
            )
            "#,
            [],
        )?;

        // Create rule_metrics table for aggregated metrics
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS rule_metrics (
                rule_name TEXT PRIMARY KEY,
                total_evaluations INTEGER NOT NULL DEFAULT 0,
                allow_count INTEGER NOT NULL DEFAULT 0,
                deny_count INTEGER NOT NULL DEFAULT 0,
                ask_count INTEGER NOT NULL DEFAULT 0,
                dry_run_count INTEGER NOT NULL DEFAULT 0,
                last_updated INTEGER NOT NULL
            )
            "#,
            [],
        )?;

        // Create indexes for better query performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_policy_events_timestamp ON policy_events(timestamp)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_policy_events_tool_name ON policy_events(tool_name)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_policy_events_action ON policy_events(action)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_policy_events_matched_rule ON policy_events(matched_rule)",
            [],
        )?;

        Ok(())
    }

    /// Stores a policy event.
    pub fn store_event(&self, event: &PolicyEvent) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            INSERT INTO policy_events 
            (timestamp, tool_name, arguments, action, matched_rule, reason, user)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                event.timestamp,
                event.tool_name,
                event.arguments,
                event.action,
                event.matched_rule,
                event.reason,
                event.user,
            ],
        )?;

        // Update rule metrics
        if let Some(ref rule_name) = event.matched_rule {
            self.update_rule_metrics(rule_name, &event.action)?;
        }

        Ok(())
    }

    /// Updates rule metrics for a given rule.
    fn update_rule_metrics(&self, rule_name: &str, action: &str) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        let timestamp = chrono::Utc::now().timestamp();

        // Try to update existing metrics
        let rows_affected = conn.execute(
            r#"
            UPDATE rule_metrics
            SET 
                total_evaluations = total_evaluations + 1,
                allow_count = allow_count + CASE WHEN ?1 = 'allow' THEN 1 ELSE 0 END,
                deny_count = deny_count + CASE WHEN ?1 = 'deny' THEN 1 ELSE 0 END,
                ask_count = ask_count + CASE WHEN ?1 = 'askuser' THEN 1 ELSE 0 END,
                dry_run_count = dry_run_count + CASE WHEN ?1 = 'dry_run_first' THEN 1 ELSE 0 END,
                last_updated = ?2
            WHERE rule_name = ?3
            "#,
            params![action, timestamp, rule_name],
        )?;

        // If no rows were updated, insert new metrics
        if rows_affected == 0 {
            conn.execute(
                r#"
                INSERT INTO rule_metrics 
                (rule_name, total_evaluations, allow_count, deny_count, ask_count, dry_run_count, last_updated)
                VALUES (?1, 1, 
                    CASE WHEN ?2 = 'allow' THEN 1 ELSE 0 END,
                    CASE WHEN ?2 = 'deny' THEN 1 ELSE 0 END,
                    CASE WHEN ?2 = 'askuser' THEN 1 ELSE 0 END,
                    CASE WHEN ?2 = 'dry_run_first' THEN 1 ELSE 0 END,
                    ?3)
                "#,
                params![rule_name, action, timestamp],
            )?;
        }

        Ok(())
    }

    /// Gets violation trends by day.
    ///
    /// # Arguments
    /// * `days` - Number of days to look back
    ///
    /// # Returns
    /// Vector of (date, violation_count) tuples
    pub fn get_violation_trends(&self, days: i64) -> SqliteResult<Vec<(String, i64)>> {
        let conn = self.conn.lock().unwrap();
        let start_timestamp = chrono::Utc::now().timestamp() - (days * 24 * 60 * 60);

        let mut stmt = conn.prepare(
            r#"
            SELECT 
                date(datetime(timestamp, 'unixepoch')) as date,
                COUNT(*) as violation_count
            FROM policy_events
            WHERE timestamp >= ?1 
                AND action IN ('deny', 'askuser', 'dry_run_first')
            GROUP BY date
            ORDER BY date ASC
            "#,
        )?;

        let rows = stmt.query_map(params![start_timestamp], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;

        let mut trends = Vec::new();
        for row in rows {
            trends.push(row?);
        }

        Ok(trends)
    }

    /// Gets rule effectiveness metrics.
    ///
    /// # Returns
    /// Vector of rule metrics
    pub fn get_rule_metrics(&self) -> SqliteResult<Vec<(String, i64, i64, i64, i64, i64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT 
                rule_name,
                total_evaluations,
                allow_count,
                deny_count,
                ask_count,
                dry_run_count
            FROM rule_metrics
            ORDER BY total_evaluations DESC
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
            ))
        })?;

        let mut metrics = Vec::new();
        for row in rows {
            metrics.push(row?);
        }

        Ok(metrics)
    }

    /// Exports events as JSON.
    pub fn export_json(&self, start_timestamp: Option<i64>, end_timestamp: Option<i64>) -> SqliteResult<String> {
        let conn = self.conn.lock().unwrap();
        let mut query = "SELECT * FROM policy_events WHERE 1=1".to_string();
        let mut params_vec = Vec::new();

        if let Some(start) = start_timestamp {
            query.push_str(" AND timestamp >= ?");
            params_vec.push(start.to_string());
        }

        if let Some(end) = end_timestamp {
            query.push_str(" AND timestamp <= ?");
            params_vec.push(end.to_string());
        }

        query.push_str(" ORDER BY timestamp DESC");

        let mut stmt = conn.prepare(&query)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, Option<i64>>(0)?,
                "timestamp": row.get::<_, i64>(1)?,
                "tool_name": row.get::<_, String>(2)?,
                "arguments": row.get::<_, String>(3)?,
                "action": row.get::<_, String>(4)?,
                "matched_rule": row.get::<_, Option<String>>(5)?,
                "reason": row.get::<_, Option<String>>(6)?,
                "user": row.get::<_, Option<String>>(7)?,
            }))
        })?;

        let mut events = Vec::new();
        for row in rows {
            events.push(row?);
        }

        Ok(serde_json::to_string_pretty(&events).unwrap_or_else(|_| "[]".to_string()))
    }

    /// Exports events as CSV.
    pub fn export_csv(&self, start_timestamp: Option<i64>, end_timestamp: Option<i64>) -> SqliteResult<String> {
        let conn = self.conn.lock().unwrap();
        let mut query = "SELECT * FROM policy_events WHERE 1=1".to_string();
        let mut params_vec = Vec::new();

        if let Some(start) = start_timestamp {
            query.push_str(" AND timestamp >= ?");
            params_vec.push(start.to_string());
        }

        if let Some(end) = end_timestamp {
            query.push_str(" AND timestamp <= ?");
            params_vec.push(end.to_string());
        }

        query.push_str(" ORDER BY timestamp DESC");

        let mut stmt = conn.prepare(&query)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            Ok((
                row.get::<_, Option<i64>>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<String>>(7)?,
            ))
        })?;

        let mut csv = String::from("id,timestamp,tool_name,arguments,action,matched_rule,reason,user\n");
        for row in rows {
            let (id, timestamp, tool_name, arguments, action, matched_rule, reason, user) = row?;
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{}\n",
                id.map(|i| i.to_string()).unwrap_or_default(),
                timestamp,
                escape_csv(&tool_name),
                escape_csv(&arguments),
                escape_csv(&action),
                matched_rule.as_ref().map(|s| escape_csv(s)).unwrap_or_default(),
                reason.as_ref().map(|s| escape_csv(s)).unwrap_or_default(),
                user.as_ref().map(|s| escape_csv(s)).unwrap_or_default(),
            ));
        }

        Ok(csv)
    }
}

/// Escapes CSV field values.
fn escape_csv(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_analytics_storage() {
        let conn = Connection::open_in_memory().unwrap();
        let storage = PolicyAnalyticsStorage::new(Arc::new(Mutex::new(conn)));

        let event = PolicyEvent {
            id: None,
            timestamp: chrono::Utc::now().timestamp(),
            tool_name: "run_terminal_cmd".to_string(),
            arguments: r#"["rm", "-rf", "/tmp"]"#.to_string(),
            action: "deny".to_string(),
            matched_rule: Some("deny-dangerous".to_string()),
            reason: Some("Safety policy".to_string()),
            user: Some("agent-123".to_string()),
        };

        storage.store_event(&event).unwrap();
        
        let metrics = storage.get_rule_metrics().unwrap();
        assert!(!metrics.is_empty());
    }
}

