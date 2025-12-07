//! Database schema for agent monitoring and telemetry.

use super::error::Result;
use rusqlite::Connection;

/// Initializes the monitoring database schema.
///
/// Creates tables for:
/// - Agent tracking (lifecycle, status, parent-child relationships)
/// - Telemetry (token usage, costs, cache statistics)
///
/// # Arguments
/// * `conn` - Database connection
///
/// # Errors
/// Returns error if schema creation fails
pub fn initialize_schema(conn: &Connection) -> Result<()> {
    // Agents table for lifecycle tracking
    conn.execute(
        "CREATE TABLE IF NOT EXISTS agents (
            id TEXT PRIMARY KEY,
            parent_id TEXT,
            plan_id TEXT,
            agent_type TEXT NOT NULL,
            status TEXT NOT NULL,
            process_id INTEGER,
            start_time INTEGER NOT NULL,
            end_time INTEGER,
            exit_code INTEGER,
            error_message TEXT,
            log_file TEXT,
            FOREIGN KEY (parent_id) REFERENCES agents(id)
        )",
        [],
    )?;

    // Telemetry table for token and cost tracking
    conn.execute(
        "CREATE TABLE IF NOT EXISTS telemetry (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            agent_id TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            input_tokens INTEGER NOT NULL DEFAULT 0,
            output_tokens INTEGER NOT NULL DEFAULT 0,
            cached_tokens INTEGER NOT NULL DEFAULT 0,
            cache_creation_tokens INTEGER NOT NULL DEFAULT 0,
            cache_read_tokens INTEGER NOT NULL DEFAULT 0,
            total_tokens INTEGER NOT NULL DEFAULT 0,
            estimated_cost REAL NOT NULL DEFAULT 0.0,
            model TEXT,
            provider TEXT,
            tool_name TEXT,
            tool_args TEXT,
            tool_approved BOOLEAN,
            tool_approval_type TEXT,
            engine_id TEXT,
            FOREIGN KEY (agent_id) REFERENCES agents(id)
        )",
        [],
    )?;

    // Indexes for efficient queries
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_agents_parent
         ON agents(parent_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_agents_plan
         ON agents(plan_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_agents_status
         ON agents(status)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_telemetry_agent
         ON telemetry(agent_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_telemetry_timestamp
         ON telemetry(timestamp)",
        [],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_initialize_schema() {
        let conn = Connection::open_in_memory().unwrap();
        initialize_schema(&conn).unwrap();

        // Verify agents table exists
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='agents'")
            .unwrap();
        let exists: bool = stmt.exists([]).unwrap();
        assert!(exists);

        // Verify telemetry table exists
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='telemetry'")
            .unwrap();
        let exists: bool = stmt.exists([]).unwrap();
        assert!(exists);
    }

    #[test]
    fn test_schema_indexes() {
        let conn = Connection::open_in_memory().unwrap();
        initialize_schema(&conn).unwrap();

        // Verify indexes exist
        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='index'").unwrap();
        let indexes: Vec<String> =
            stmt.query_map([], |row| row.get(0)).unwrap().map(|r| r.unwrap()).collect();

        assert!(indexes.iter().any(|name| name.contains("idx_agents_parent")));
        assert!(indexes.iter().any(|name| name.contains("idx_agents_plan")));
        assert!(indexes.iter().any(|name| name.contains("idx_agents_status")));
        assert!(indexes.iter().any(|name| name.contains("idx_telemetry_agent")));
    }

    #[test]
    fn test_foreign_key_constraints() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();
        initialize_schema(&conn).unwrap();

        // Insert parent agent
        conn.execute(
            "INSERT INTO agents (id, agent_type, status, start_time)
             VALUES ('parent-1', 'architect', 'running', 1000)",
            [],
        )
        .unwrap();

        // Insert child agent with valid parent
        conn.execute(
            "INSERT INTO agents (id, parent_id, agent_type, status, start_time)
             VALUES ('child-1', 'parent-1', 'developer', 'running', 1100)",
            [],
        )
        .unwrap();

        // Verify child agent exists
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM agents WHERE id = 'child-1'").unwrap();
        let count: i32 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_telemetry_insert() {
        let conn = Connection::open_in_memory().unwrap();
        initialize_schema(&conn).unwrap();

        // Insert agent first
        conn.execute(
            "INSERT INTO agents (id, agent_type, status, start_time)
             VALUES ('agent-1', 'developer', 'running', 1000)",
            [],
        )
        .unwrap();

        // Insert telemetry
        conn.execute(
            "INSERT INTO telemetry (agent_id, timestamp, input_tokens, output_tokens, total_tokens, estimated_cost, model)
             VALUES ('agent-1', 2000, 100, 200, 300, 0.05, 'gpt-4')",
            [],
        ).unwrap();

        // Verify telemetry exists
        let mut stmt =
            conn.prepare("SELECT COUNT(*) FROM telemetry WHERE agent_id = 'agent-1'").unwrap();
        let count: i32 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 1);
    }
}
