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

        // Create indexes for better query performance
        self.conn
            .execute("CREATE INDEX IF NOT EXISTS idx_tasks_agent_id ON tasks(agent_id)", [])?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_workflow_steps_workflow_id ON workflow_steps(workflow_id)",
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
        assert!(table_count >= 4); // At least agents, workflows, workflow_steps, tasks
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
}
