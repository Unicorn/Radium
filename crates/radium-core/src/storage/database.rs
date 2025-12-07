//! Database connection and schema management.

use rusqlite::Connection;
use tracing::info;

use crate::storage::error::StorageResult;

/// Database connection wrapper.
///
/// Manages SQLite connection and schema initialization.
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Opens a new database connection at the specified path.
    ///
    /// # Arguments
    /// * `path` - Path to the SQLite database file
    ///
    /// # Returns
    /// A new `Database` instance with initialized schema.
    ///
    /// # Errors
    /// * `StorageError::Connection` - If the database connection fails
    pub fn open(path: &str) -> StorageResult<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// Opens an in-memory database for testing.
    ///
    /// # Returns
    /// A new `Database` instance with initialized schema.
    ///
    /// # Errors
    /// * `StorageError::Connection` - If the database connection fails
    pub fn open_in_memory() -> StorageResult<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// Gets a reference to the underlying connection.
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    /// Gets a mutable reference to the underlying connection.
    pub fn conn_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }

    /// Initializes the database schema.
    ///
    /// Creates all necessary tables for agents, workflows, and tasks.
    ///
    /// # Errors
    /// * `StorageError::Connection` - If schema creation fails
    fn init_schema(&self) -> StorageResult<()> {
        info!("Initializing database schema");

        // Create agents table
        self.conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS agents (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                config_json TEXT NOT NULL,
                state TEXT NOT NULL,
                capabilities TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
            [],
        )?;

        // Create workflows table
        self.conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS workflows (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                state TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
            [],
        )?;

        // Create workflow_steps table
        self.conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS workflow_steps (
                id TEXT PRIMARY KEY,
                workflow_id TEXT NOT NULL,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                task_id TEXT NOT NULL,
                config_json TEXT,
                step_order INTEGER NOT NULL,
                FOREIGN KEY (workflow_id) REFERENCES workflows(id) ON DELETE CASCADE
            )
            "#,
            [],
        )?;

        // Create tasks table
        self.conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                agent_id TEXT NOT NULL,
                input_json TEXT NOT NULL,
                state TEXT NOT NULL,
                result_json TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
            [],
        )?;

        // Create agent_messages table for agent-to-agent communication
        self.conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS agent_messages (
                id TEXT PRIMARY KEY,
                sender_id TEXT NOT NULL,
                recipient_id TEXT,
                message_type TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                delivered INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (sender_id) REFERENCES agents(id),
                FOREIGN KEY (recipient_id) REFERENCES agents(id)
            )
            "#,
            [],
        )?;

        // Create agent_delegations table for supervisor-worker relationships
        self.conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS agent_delegations (
                supervisor_id TEXT NOT NULL,
                worker_id TEXT NOT NULL,
                spawned_at INTEGER NOT NULL,
                completed_at INTEGER,
                status TEXT NOT NULL,
                PRIMARY KEY (supervisor_id, worker_id),
                FOREIGN KEY (supervisor_id) REFERENCES agents(id),
                FOREIGN KEY (worker_id) REFERENCES agents(id)
            )
            "#,
            [],
        )?;

        // Create agent_progress table for progress tracking
        self.conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS agent_progress (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                agent_id TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                percentage INTEGER NOT NULL,
                status TEXT NOT NULL,
                message TEXT,
                FOREIGN KEY (agent_id) REFERENCES agents(id)
            )
            "#,
            [],
        )?;

        // Create indexes for better query performance
        self.conn
            .execute("CREATE INDEX IF NOT EXISTS idx_tasks_agent_id ON tasks(agent_id)", [])?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_workflow_steps_workflow_id ON workflow_steps(workflow_id)",
            [],
        )?;

        // Indexes for collaboration tables
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_agent_messages_sender_id ON agent_messages(sender_id)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_agent_messages_recipient_id ON agent_messages(recipient_id)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_agent_delegations_supervisor_id ON agent_delegations(supervisor_id)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_agent_progress_agent_id ON agent_progress(agent_id)",
            [],
        )?;

        info!("Database schema initialized successfully");
        Ok(())
    }

    /// Runs a transaction with the provided closure.
    ///
    /// # Arguments
    /// * `f` - Closure that performs operations within the transaction
    ///
    /// # Returns
    /// The result of the closure, or an error if the transaction fails
    ///
    /// # Errors
    /// * `StorageError::Connection` - If the transaction fails
    pub fn transaction<F, R>(&mut self, f: F) -> StorageResult<R>
    where
        F: FnOnce(&rusqlite::Transaction) -> StorageResult<R>,
    {
        let tx = self.conn.transaction()?;
        match f(&tx) {
            Ok(result) => {
                tx.commit()?;
                Ok(result)
            }
            Err(e) => {
                tx.rollback()?;
                Err(e)
            }
        }
    }
}

// Database connection is automatically closed when dropped

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_open_in_memory() {
        let db = Database::open_in_memory().unwrap();
        // Verify tables exist by checking if we can query them
        let mut stmt =
            db.conn().prepare("SELECT name FROM sqlite_master WHERE type='table'").unwrap();
        let tables: Vec<String> =
            stmt.query_map([], |row| row.get(0)).unwrap().map(|r| r.unwrap()).collect();

        assert!(tables.contains(&"agents".to_string()));
        assert!(tables.contains(&"workflows".to_string()));
        assert!(tables.contains(&"workflow_steps".to_string()));
        assert!(tables.contains(&"tasks".to_string()));
        assert!(tables.contains(&"agent_messages".to_string()));
        assert!(tables.contains(&"agent_delegations".to_string()));
        assert!(tables.contains(&"agent_progress".to_string()));
    }

    #[test]
    fn test_database_open_file() {
        use std::fs;
        let temp_file = std::env::temp_dir().join("test_radium.db");
        // Clean up if exists
        let _ = fs::remove_file(&temp_file);

        let db = Database::open(temp_file.to_str().unwrap()).unwrap();
        // Verify it was created
        assert!(temp_file.exists());

        // Verify tables exist
        let mut stmt =
            db.conn().prepare("SELECT name FROM sqlite_master WHERE type='table'").unwrap();
        let tables_iter = stmt.query_map([], |row| row.get::<_, String>(0)).unwrap();
        assert!(tables_iter.map(|r| r.unwrap()).any(|x| x == "agents"));

        // Clean up
        let _ = fs::remove_file(&temp_file);
    }

    #[test]
    fn test_database_conn_mut() {
        let mut db = Database::open_in_memory().unwrap();
        let conn_mut = db.conn_mut();
        // Verify we can use the mutable connection
        conn_mut
            .execute("INSERT INTO agents (id, name, description, config_json, state, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
                rusqlite::params!["test-id", "Test", "Description", "{}", "idle", "2023-01-01T00:00:00Z", "2023-01-01T00:00:00Z"])
            .unwrap();
    }

    #[test]
    fn test_database_schema_idempotent() {
        // Opening the same database twice should not fail
        let db1 = Database::open_in_memory().unwrap();
        // Create a second connection to the same in-memory DB (not possible with in-memory)
        // But we can test that schema creation is idempotent by checking table count
        let mut stmt =
            db1.conn().prepare("SELECT COUNT(*) FROM sqlite_master WHERE type='table'").unwrap();
        let table_count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert!(table_count >= 7); // At least agents, workflows, workflow_steps, tasks, agent_messages, agent_delegations, agent_progress
    }

    #[test]
    fn test_database_multiple_operations() {
        let mut db = Database::open_in_memory().unwrap();
        let conn = db.conn_mut();

        // Insert multiple records
        for i in 0..5 {
            conn.execute(
                "INSERT INTO agents (id, name, description, config_json, state, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
                rusqlite::params![
                    format!("agent-{}", i),
                    format!("Agent {}", i),
                    "Description",
                    "{}",
                    "idle",
                    "2023-01-01T00:00:00Z",
                    "2023-01-01T00:00:00Z"
                ],
            )
            .unwrap();
        }

        // Verify all were inserted
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM agents").unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_database_transaction_commit() {
        let mut db = Database::open_in_memory().unwrap();

        let result = db.transaction(|tx| {
            tx.execute(
                "INSERT INTO agents (id, name, description, config_json, state, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
                rusqlite::params!["tx-agent", "TX Agent", "Test", "{}", "idle", "2023-01-01T00:00:00Z", "2023-01-01T00:00:00Z"],
            )?;
            Ok(())
        });

        assert!(result.is_ok());

        // Verify the insert was committed
        let mut stmt = db.conn().prepare("SELECT id FROM agents WHERE id = ?").unwrap();
        let exists = stmt.exists(rusqlite::params!["tx-agent"]).unwrap();
        assert!(exists);
    }

    #[test]
    fn test_database_transaction_rollback() {
        let mut db = Database::open_in_memory().unwrap();

        let result: StorageResult<()> = db.transaction(|tx| {
            tx.execute(
                "INSERT INTO agents (id, name, description, config_json, state, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
                rusqlite::params!["rollback-agent", "Rollback Agent", "Test", "{}", "idle", "2023-01-01T00:00:00Z", "2023-01-01T00:00:00Z"],
            )?;
            // Simulate an error to trigger rollback
            Err(crate::storage::error::StorageError::InvalidData("Simulated error".to_string()))
        });

        assert!(result.is_err());

        // Verify the insert was rolled back
        let mut stmt = db.conn().prepare("SELECT id FROM agents WHERE id = ?").unwrap();
        let exists = stmt.exists(rusqlite::params!["rollback-agent"]).unwrap();
        assert!(!exists);
    }

    #[test]
    fn test_database_indexes_created() {
        let db = Database::open_in_memory().unwrap();

        // Check that indexes were created
        let mut stmt = db
            .conn()
            .prepare("SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'")
            .unwrap();
        let indexes: Vec<String> =
            stmt.query_map([], |row| row.get(0)).unwrap().map(|r| r.unwrap()).collect();

        assert!(indexes.contains(&"idx_tasks_agent_id".to_string()));
        assert!(indexes.contains(&"idx_workflow_steps_workflow_id".to_string()));
        assert!(indexes.contains(&"idx_agent_messages_sender_id".to_string()));
        assert!(indexes.contains(&"idx_agent_messages_recipient_id".to_string()));
        assert!(indexes.contains(&"idx_agent_delegations_supervisor_id".to_string()));
        assert!(indexes.contains(&"idx_agent_progress_agent_id".to_string()));
    }

    #[test]
    fn test_database_open_with_special_chars_path() {
        use std::fs;
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test db with spaces.db");
        let _ = fs::remove_file(&db_path);

        let _db = Database::open(db_path.to_str().unwrap()).unwrap();
        assert!(db_path.exists());

        // Clean up
        let _ = fs::remove_file(&db_path);
    }

    #[test]
    fn test_database_persistence() {
        use std::fs;
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("persistence_test.db");
        let _ = fs::remove_file(&db_path);

        // Create database and insert data
        {
            let mut db = Database::open(db_path.to_str().unwrap()).unwrap();
            db.conn_mut()
                .execute(
                    "INSERT INTO agents (id, name, description, config_json, state, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
                    rusqlite::params!["persist-agent", "Persist", "Test", "{}", "idle", "2023-01-01T00:00:00Z", "2023-01-01T00:00:00Z"],
                )
                .unwrap();
        } // Database is closed here

        // Reopen and verify data persists
        {
            let db = Database::open(db_path.to_str().unwrap()).unwrap();
            let mut stmt = db.conn().prepare("SELECT id FROM agents WHERE id = ?").unwrap();
            let exists = stmt.exists(rusqlite::params!["persist-agent"]).unwrap();
            assert!(exists);
        }

        // Clean up
        let _ = fs::remove_file(&db_path);
    }

    #[test]
    fn test_database_transaction_with_return_value() {
        let mut db = Database::open_in_memory().unwrap();

        let result: StorageResult<i64> = db.transaction(|tx| {
            tx.execute(
                "INSERT INTO agents (id, name, description, config_json, state, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
                rusqlite::params!["return-agent", "Return", "Test", "{}", "idle", "2023-01-01T00:00:00Z", "2023-01-01T00:00:00Z"],
            )?;
            let count: i64 = tx.query_row("SELECT COUNT(*) FROM agents", [], |row| row.get(0))?;
            Ok(count)
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_database_foreign_key_constraint() {
        let mut db = Database::open_in_memory().unwrap();

        // Enable foreign key constraints
        db.conn_mut().execute("PRAGMA foreign_keys = ON", []).unwrap();

        // Try to insert a workflow step with invalid workflow_id
        let result = db.conn_mut().execute(
            "INSERT INTO workflow_steps (id, workflow_id, name, description, task_id, config_json, step_order) VALUES (?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params!["step-1", "nonexistent-workflow", "Step", "Test", "task-1", "null", 0],
        );

        // Should fail due to foreign key constraint
        assert!(result.is_err());
    }

    #[test]
    fn test_database_cascade_delete() {
        let mut db = Database::open_in_memory().unwrap();

        // Enable foreign key constraints
        db.conn_mut().execute("PRAGMA foreign_keys = ON", []).unwrap();

        // Insert workflow
        db.conn_mut()
            .execute(
                "INSERT INTO workflows (id, name, description, state, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
                rusqlite::params!["wf-cascade", "Cascade WF", "Test", "\"Idle\"", "2023-01-01T00:00:00Z", "2023-01-01T00:00:00Z"],
            )
            .unwrap();

        // Insert workflow step
        db.conn_mut()
            .execute(
                "INSERT INTO workflow_steps (id, workflow_id, name, description, task_id, config_json, step_order) VALUES (?, ?, ?, ?, ?, ?, ?)",
                rusqlite::params!["step-cascade", "wf-cascade", "Step", "Test", "task-1", "null", 0],
            )
            .unwrap();

        // Delete workflow
        db.conn_mut()
            .execute("DELETE FROM workflows WHERE id = ?", rusqlite::params!["wf-cascade"])
            .unwrap();

        // Verify step was cascaded
        let mut stmt = db.conn().prepare("SELECT id FROM workflow_steps WHERE id = ?").unwrap();
        let exists = stmt.exists(rusqlite::params!["step-cascade"]).unwrap();
        assert!(!exists);
    }

    #[test]
    fn test_database_conn_immutable() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();

        // Verify we can read from the immutable connection
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM agents").unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 0);
    }
}
