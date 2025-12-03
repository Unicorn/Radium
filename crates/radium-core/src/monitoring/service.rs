//! Agent monitoring service for lifecycle tracking.

use super::error::{MonitoringError, Result};
use super::schema::initialize_schema;
use rusqlite::{Connection, params};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Agent status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentStatus {
    /// Agent is starting up.
    Starting,
    /// Agent is running.
    Running,
    /// Agent completed successfully.
    Completed,
    /// Agent failed with an error.
    Failed,
    /// Agent was terminated.
    Terminated,
}

impl AgentStatus {
    /// Converts status to string representation.
    pub fn as_str(&self) -> &str {
        match self {
            AgentStatus::Starting => "starting",
            AgentStatus::Running => "running",
            AgentStatus::Completed => "completed",
            AgentStatus::Failed => "failed",
            AgentStatus::Terminated => "terminated",
        }
    }

    /// Parses status from string.
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "starting" => Ok(AgentStatus::Starting),
            "running" => Ok(AgentStatus::Running),
            "completed" => Ok(AgentStatus::Completed),
            "failed" => Ok(AgentStatus::Failed),
            "terminated" => Ok(AgentStatus::Terminated),
            _ => Err(MonitoringError::InvalidStatus(s.to_string())),
        }
    }
}

/// Agent record in monitoring database.
#[derive(Debug, Clone)]
pub struct AgentRecord {
    /// Unique agent ID.
    pub id: String,
    /// Parent agent ID (if this is a child agent).
    pub parent_id: Option<String>,
    /// Plan ID this agent belongs to.
    pub plan_id: Option<String>,
    /// Agent type (architect, developer, etc.).
    pub agent_type: String,
    /// Current status.
    pub status: AgentStatus,
    /// Process ID.
    pub process_id: Option<u32>,
    /// Start timestamp (Unix epoch seconds).
    pub start_time: u64,
    /// End timestamp (Unix epoch seconds).
    pub end_time: Option<u64>,
    /// Exit code (if completed).
    pub exit_code: Option<i32>,
    /// Error message (if failed).
    pub error_message: Option<String>,
    /// Log file path.
    pub log_file: Option<String>,
}

impl AgentRecord {
    /// Creates a new agent record.
    pub fn new(id: String, agent_type: String) -> Self {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id,
            parent_id: None,
            plan_id: None,
            agent_type,
            status: AgentStatus::Starting,
            process_id: None,
            start_time,
            end_time: None,
            exit_code: None,
            error_message: None,
            log_file: None,
        }
    }

    /// Sets the parent agent ID.
    pub fn with_parent(mut self, parent_id: String) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    /// Sets the plan ID.
    pub fn with_plan(mut self, plan_id: String) -> Self {
        self.plan_id = Some(plan_id);
        self
    }

    /// Sets the process ID.
    pub fn with_process_id(mut self, process_id: u32) -> Self {
        self.process_id = Some(process_id);
        self
    }

    /// Sets the log file path.
    pub fn with_log_file(mut self, log_file: String) -> Self {
        self.log_file = Some(log_file);
        self
    }
}

/// Agent monitoring service.
pub struct MonitoringService {
    /// Database connection.
    conn: Connection,
}

impl MonitoringService {
    /// Creates a new monitoring service with an in-memory database.
    ///
    /// # Errors
    /// Returns error if database initialization fails
    pub fn new() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        initialize_schema(&conn)?;
        Ok(Self { conn })
    }

    /// Opens a monitoring service with a database file.
    ///
    /// # Arguments
    /// * `path` - Path to database file
    ///
    /// # Errors
    /// Returns error if database opening or initialization fails
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path)?;
        initialize_schema(&conn)?;
        Ok(Self { conn })
    }

    /// Registers a new agent.
    ///
    /// # Arguments
    /// * `record` - Agent record to register
    ///
    /// # Errors
    /// Returns error if insertion fails
    pub fn register_agent(&self, record: &AgentRecord) -> Result<()> {
        self.conn.execute(
            "INSERT INTO agents (id, parent_id, plan_id, agent_type, status, process_id, start_time, log_file)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                record.id,
                record.parent_id,
                record.plan_id,
                record.agent_type,
                record.status.as_str(),
                record.process_id,
                record.start_time,
                record.log_file,
            ],
        )?;
        Ok(())
    }

    /// Updates an agent's status.
    ///
    /// # Arguments
    /// * `agent_id` - Agent identifier
    /// * `status` - New status
    ///
    /// # Errors
    /// Returns error if update fails
    pub fn update_status(&self, agent_id: &str, status: AgentStatus) -> Result<()> {
        let rows_affected = self.conn.execute(
            "UPDATE agents SET status = ?1 WHERE id = ?2",
            params![status.as_str(), agent_id],
        )?;

        if rows_affected == 0 {
            return Err(MonitoringError::AgentNotFound(agent_id.to_string()));
        }

        Ok(())
    }

    /// Marks an agent as completed.
    ///
    /// # Arguments
    /// * `agent_id` - Agent identifier
    /// * `exit_code` - Exit code (0 for success)
    ///
    /// # Errors
    /// Returns error if update fails
    pub fn complete_agent(&self, agent_id: &str, exit_code: i32) -> Result<()> {
        let end_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let rows_affected = self.conn.execute(
            "UPDATE agents SET status = ?1, end_time = ?2, exit_code = ?3 WHERE id = ?4",
            params![AgentStatus::Completed.as_str(), end_time, exit_code, agent_id],
        )?;

        if rows_affected == 0 {
            return Err(MonitoringError::AgentNotFound(agent_id.to_string()));
        }

        Ok(())
    }

    /// Marks an agent as failed.
    ///
    /// # Arguments
    /// * `agent_id` - Agent identifier
    /// * `error_message` - Error message
    ///
    /// # Errors
    /// Returns error if update fails
    pub fn fail_agent(&self, agent_id: &str, error_message: &str) -> Result<()> {
        let end_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let rows_affected = self.conn.execute(
            "UPDATE agents SET status = ?1, end_time = ?2, error_message = ?3 WHERE id = ?4",
            params![AgentStatus::Failed.as_str(), end_time, error_message, agent_id],
        )?;

        if rows_affected == 0 {
            return Err(MonitoringError::AgentNotFound(agent_id.to_string()));
        }

        Ok(())
    }

    /// Gets an agent record.
    ///
    /// # Arguments
    /// * `agent_id` - Agent identifier
    ///
    /// # Returns
    /// Agent record if found
    ///
    /// # Errors
    /// Returns error if query fails or agent not found
    pub fn get_agent(&self, agent_id: &str) -> Result<AgentRecord> {
        let mut stmt = self.conn.prepare(
            "SELECT id, parent_id, plan_id, agent_type, status, process_id, start_time, end_time, exit_code, error_message, log_file
             FROM agents WHERE id = ?1"
        )?;

        let record = stmt.query_row(params![agent_id], |row| {
            Ok(AgentRecord {
                id: row.get(0)?,
                parent_id: row.get(1)?,
                plan_id: row.get(2)?,
                agent_type: row.get(3)?,
                status: AgentStatus::from_str(&row.get::<_, String>(4)?).unwrap(),
                process_id: row.get(5)?,
                start_time: row.get(6)?,
                end_time: row.get(7)?,
                exit_code: row.get(8)?,
                error_message: row.get(9)?,
                log_file: row.get(10)?,
            })
        })?;

        Ok(record)
    }

    /// Gets all child agents of a parent.
    ///
    /// # Arguments
    /// * `parent_id` - Parent agent identifier
    ///
    /// # Returns
    /// List of child agent records
    ///
    /// # Errors
    /// Returns error if query fails
    pub fn get_children(&self, parent_id: &str) -> Result<Vec<AgentRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, parent_id, plan_id, agent_type, status, process_id, start_time, end_time, exit_code, error_message, log_file
             FROM agents WHERE parent_id = ?1"
        )?;

        let records = stmt.query_map(params![parent_id], |row| {
            Ok(AgentRecord {
                id: row.get(0)?,
                parent_id: row.get(1)?,
                plan_id: row.get(2)?,
                agent_type: row.get(3)?,
                status: AgentStatus::from_str(&row.get::<_, String>(4)?).unwrap(),
                process_id: row.get(5)?,
                start_time: row.get(6)?,
                end_time: row.get(7)?,
                exit_code: row.get(8)?,
                error_message: row.get(9)?,
                log_file: row.get(10)?,
            })
        })?.collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(records)
    }

    /// Gets all agents for a plan.
    ///
    /// # Arguments
    /// * `plan_id` - Plan identifier
    ///
    /// # Returns
    /// List of agent records
    ///
    /// # Errors
    /// Returns error if query fails
    pub fn get_plan_agents(&self, plan_id: &str) -> Result<Vec<AgentRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, parent_id, plan_id, agent_type, status, process_id, start_time, end_time, exit_code, error_message, log_file
             FROM agents WHERE plan_id = ?1"
        )?;

        let records = stmt.query_map(params![plan_id], |row| {
            Ok(AgentRecord {
                id: row.get(0)?,
                parent_id: row.get(1)?,
                plan_id: row.get(2)?,
                agent_type: row.get(3)?,
                status: AgentStatus::from_str(&row.get::<_, String>(4)?).unwrap(),
                process_id: row.get(5)?,
                start_time: row.get(6)?,
                end_time: row.get(7)?,
                exit_code: row.get(8)?,
                error_message: row.get(9)?,
                log_file: row.get(10)?,
            })
        })?.collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(records)
    }
}

impl Default for MonitoringService {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_status_conversion() {
        assert_eq!(AgentStatus::Running.as_str(), "running");
        assert_eq!(AgentStatus::from_str("running").unwrap(), AgentStatus::Running);
    }

    #[test]
    fn test_monitoring_service_new() {
        let service = MonitoringService::new().unwrap();
        assert!(service.conn.is_autocommit());
    }

    #[test]
    fn test_register_agent() {
        let service = MonitoringService::new().unwrap();
        let record = AgentRecord::new("agent-1".to_string(), "developer".to_string());

        service.register_agent(&record).unwrap();

        let retrieved = service.get_agent("agent-1").unwrap();
        assert_eq!(retrieved.id, "agent-1");
        assert_eq!(retrieved.agent_type, "developer");
    }

    #[test]
    fn test_update_status() {
        let service = MonitoringService::new().unwrap();
        let record = AgentRecord::new("agent-1".to_string(), "developer".to_string());

        service.register_agent(&record).unwrap();
        service.update_status("agent-1", AgentStatus::Running).unwrap();

        let retrieved = service.get_agent("agent-1").unwrap();
        assert_eq!(retrieved.status, AgentStatus::Running);
    }

    #[test]
    fn test_complete_agent() {
        let service = MonitoringService::new().unwrap();
        let record = AgentRecord::new("agent-1".to_string(), "developer".to_string());

        service.register_agent(&record).unwrap();
        service.complete_agent("agent-1", 0).unwrap();

        let retrieved = service.get_agent("agent-1").unwrap();
        assert_eq!(retrieved.status, AgentStatus::Completed);
        assert_eq!(retrieved.exit_code, Some(0));
        assert!(retrieved.end_time.is_some());
    }

    #[test]
    fn test_fail_agent() {
        let service = MonitoringService::new().unwrap();
        let record = AgentRecord::new("agent-1".to_string(), "developer".to_string());

        service.register_agent(&record).unwrap();
        service.fail_agent("agent-1", "Test error").unwrap();

        let retrieved = service.get_agent("agent-1").unwrap();
        assert_eq!(retrieved.status, AgentStatus::Failed);
        assert_eq!(retrieved.error_message, Some("Test error".to_string()));
        assert!(retrieved.end_time.is_some());
    }

    #[test]
    fn test_parent_child_relationship() {
        let service = MonitoringService::new().unwrap();

        let parent = AgentRecord::new("parent-1".to_string(), "architect".to_string());
        service.register_agent(&parent).unwrap();

        let child = AgentRecord::new("child-1".to_string(), "developer".to_string())
            .with_parent("parent-1".to_string());
        service.register_agent(&child).unwrap();

        let children = service.get_children("parent-1").unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id, "child-1");
        assert_eq!(children[0].parent_id, Some("parent-1".to_string()));
    }

    #[test]
    fn test_plan_agents() {
        let service = MonitoringService::new().unwrap();

        let agent1 = AgentRecord::new("agent-1".to_string(), "architect".to_string())
            .with_plan("REQ-001".to_string());
        let agent2 = AgentRecord::new("agent-2".to_string(), "developer".to_string())
            .with_plan("REQ-001".to_string());

        service.register_agent(&agent1).unwrap();
        service.register_agent(&agent2).unwrap();

        let agents = service.get_plan_agents("REQ-001").unwrap();
        assert_eq!(agents.len(), 2);
    }
}
