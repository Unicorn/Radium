//! Repository implementations for data persistence.
//!
//! This module provides the Repository pattern implementation for agents,
//! workflows, and tasks using SQLite as the backing store.

use crate::storage::database::Database;
use crate::storage::error::{StorageError, StorageResult};
use chrono::{DateTime, Utc};
use rusqlite::{Row, params};
use tracing::{debug, info};

use crate::models::{
    Agent, AgentConfig, AgentState, Task, TaskResult, TaskState, Workflow, WorkflowState,
    WorkflowStep,
};

// ============================================================================
// Generic Repository Infrastructure
// ============================================================================

/// Trait for entities that can be stored in a repository.
///
/// This trait defines how an entity maps to and from database rows,
/// including table name, validation, and serialization.
pub trait Entity: Clone + Send + Sync {
    /// Error type for entity-specific validation errors.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Returns the table name for this entity.
    fn table_name() -> &'static str;

    /// Returns the ID of this entity.
    fn id(&self) -> &str;

    /// Validates the entity.
    fn validate(&self) -> Result<(), Self::Error>;
}

/// Generic repository trait for CRUD operations.
pub trait GenericRepository<T: Entity> {
    /// Creates a new entity in storage.
    fn create(&mut self, entity: &T) -> StorageResult<()>;

    /// Retrieves an entity by ID.
    fn get_by_id(&self, id: &str) -> StorageResult<T>;

    /// Retrieves all entities.
    fn get_all(&self) -> StorageResult<Vec<T>>;

    /// Updates an existing entity.
    fn update(&mut self, entity: &T) -> StorageResult<()>;

    /// Deletes an entity by ID.
    fn delete(&mut self, id: &str) -> StorageResult<()>;
}

// ============================================================================
// Row Parsing Helpers
// ============================================================================

/// Parses a JSON field from a row into a deserializable type.
///
/// # Arguments
/// * `row` - The database row
/// * `idx` - The column index
/// * `column_name` - The name of the column (for error messages)
///
/// # Errors
/// Returns a `rusqlite::Error::InvalidColumnType` if parsing fails.
fn parse_json_field<T>(row: &Row, idx: usize, column_name: &str) -> rusqlite::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let json_str: String = row.get(idx)?;
    serde_json::from_str(&json_str).map_err(|_| {
        rusqlite::Error::InvalidColumnType(
            idx,
            column_name.to_string(),
            rusqlite::types::Type::Text,
        )
    })
}

/// Parses an optional JSON field from a row into a deserializable type.
///
/// # Arguments
/// * `row` - The database row
/// * `idx` - The column index
/// * `column_name` - The name of the column (for error messages)
///
/// # Errors
/// Returns a `rusqlite::Error::InvalidColumnType` if parsing fails.
fn parse_optional_json_field<T>(
    row: &Row,
    idx: usize,
    column_name: &str,
) -> rusqlite::Result<Option<T>>
where
    T: serde::de::DeserializeOwned,
{
    let json_str: Option<String> = row.get(idx)?;
    match json_str {
        Some(s) => serde_json::from_str(&s).map(Some).map_err(|_| {
            rusqlite::Error::InvalidColumnType(
                idx,
                column_name.to_string(),
                rusqlite::types::Type::Text,
            )
        }),
        None => Ok(None),
    }
}

/// Parses an RFC3339 timestamp string from a row into a `DateTime<Utc>`.
///
/// # Arguments
/// * `row` - The database row
/// * `idx` - The column index
/// * `column_name` - The name of the column (for error messages)
///
/// # Errors
/// Returns a `rusqlite::Error::InvalidColumnType` if parsing fails.
fn parse_timestamp(row: &Row, idx: usize, column_name: &str) -> rusqlite::Result<DateTime<Utc>> {
    let timestamp_str: String = row.get(idx)?;
    DateTime::parse_from_rfc3339(&timestamp_str).map(|dt| dt.with_timezone(&Utc)).map_err(|_| {
        rusqlite::Error::InvalidColumnType(
            idx,
            column_name.to_string(),
            rusqlite::types::Type::Text,
        )
    })
}

// ============================================================================
// Entity Implementations
// ============================================================================

impl Entity for Agent {
    type Error = crate::models::AgentError;

    fn table_name() -> &'static str {
        "agents"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn validate(&self) -> Result<(), Self::Error> {
        self.validate()
    }
}

impl Entity for Workflow {
    type Error = crate::models::WorkflowError;

    fn table_name() -> &'static str {
        "workflows"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn validate(&self) -> Result<(), Self::Error> {
        self.validate()
    }
}

impl Entity for Task {
    type Error = crate::models::TaskError;

    fn table_name() -> &'static str {
        "tasks"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn validate(&self) -> Result<(), Self::Error> {
        self.validate()
    }
}

// ============================================================================
// Generic Repository Implementation Helpers
// ============================================================================

/// Generic helper for validating an entity before operations.
fn validate_entity<T: Entity>(entity: &T) -> StorageResult<()> {
    entity.validate().map_err(|e| StorageError::InvalidData(e.to_string()))
}

/// Generic helper for handling NotFound errors.
fn not_found_error<T: Entity>(id: &str) -> StorageError {
    StorageError::NotFound(format!("{} with id {} not found", T::table_name(), id))
}

// ============================================================================
// Repository Traits
// ============================================================================

/// Repository trait for agent operations.
pub trait AgentRepository {
    /// Creates a new agent in storage.
    fn create(&mut self, agent: &Agent) -> StorageResult<()>;

    /// Retrieves an agent by ID.
    fn get_by_id(&self, id: &str) -> StorageResult<Agent>;

    /// Retrieves all agents.
    fn get_all(&self) -> StorageResult<Vec<Agent>>;

    /// Updates an existing agent.
    fn update(&mut self, agent: &Agent) -> StorageResult<()>;

    /// Deletes an agent by ID.
    fn delete(&mut self, id: &str) -> StorageResult<()>;
}

/// Repository trait for workflow operations.
pub trait WorkflowRepository {
    /// Creates a new workflow in storage.
    fn create(&mut self, workflow: &Workflow) -> StorageResult<()>;

    /// Retrieves a workflow by ID.
    fn get_by_id(&self, id: &str) -> StorageResult<Workflow>;

    /// Retrieves all workflows.
    fn get_all(&self) -> StorageResult<Vec<Workflow>>;

    /// Updates an existing workflow.
    fn update(&mut self, workflow: &Workflow) -> StorageResult<()>;

    /// Deletes a workflow by ID.
    fn delete(&mut self, id: &str) -> StorageResult<()>;
}

/// Repository trait for task operations.
pub trait TaskRepository {
    /// Creates a new task in storage.
    fn create(&mut self, task: &Task) -> StorageResult<()>;

    /// Retrieves a task by ID.
    fn get_by_id(&self, id: &str) -> StorageResult<Task>;

    /// Retrieves all tasks.
    fn get_all(&self) -> StorageResult<Vec<Task>>;

    /// Retrieves tasks by agent ID.
    fn get_by_agent_id(&self, agent_id: &str) -> StorageResult<Vec<Task>>;

    /// Updates an existing task.
    fn update(&mut self, task: &Task) -> StorageResult<()>;

    /// Deletes a task by ID.
    fn delete(&mut self, id: &str) -> StorageResult<()>;
}

// ============================================================================
// SQLite Agent Repository
// ============================================================================

/// SQLite implementation of AgentRepository.
pub struct SqliteAgentRepository<'a> {
    db: &'a mut Database,
}

impl<'a> SqliteAgentRepository<'a> {
    /// Creates a new SQLite agent repository.
    pub fn new(db: &'a mut Database) -> Self {
        Self { db }
    }
}

impl AgentRepository for SqliteAgentRepository<'_> {
    fn create(&mut self, agent: &Agent) -> StorageResult<()> {
        validate_entity(agent)?;
        let config_json = serde_json::to_string(&agent.config)?;
        let state_json = serde_json::to_string(&agent.state)?;
        self.db.conn_mut().execute(
            "INSERT INTO agents (id, name, description, config_json, state, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![agent.id, agent.name, agent.description, config_json, state_json, agent.created_at.to_rfc3339(), agent.updated_at.to_rfc3339()],
        )?;
        info!(agent_id = %agent.id, "Created agent");
        Ok(())
    }

    fn get_by_id(&self, id: &str) -> StorageResult<Agent> {
        let mut stmt = self.db.conn().prepare(
            "SELECT id, name, description, config_json, state, created_at, updated_at FROM agents WHERE id = ?1"
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let description: String = row.get(2)?;
            let config: AgentConfig = parse_json_field(row, 3, "config_json")?;
            let state: AgentState = parse_json_field(row, 4, "state")?;
            let created_at = parse_timestamp(row, 5, "created_at")?;
            let updated_at = parse_timestamp(row, 6, "updated_at")?;

            Ok(Agent { id, name, description, config, state, created_at, updated_at })
        })?;
        match rows.next() {
            Some(Ok(agent)) => Ok(agent),
            Some(Err(e)) => Err(e.into()),
            None => Err(not_found_error::<Agent>(id)),
        }
    }

    fn get_all(&self) -> StorageResult<Vec<Agent>> {
        let mut stmt = self.db.conn().prepare(
            "SELECT id, name, description, config_json, state, created_at, updated_at FROM agents ORDER BY created_at DESC"
        )?;
        let agents = stmt
            .query_map([], |row| {
                let config: AgentConfig = parse_json_field(row, 3, "config_json")?;
                let state: AgentState = parse_json_field(row, 4, "state")?;
                let created_at = parse_timestamp(row, 5, "created_at")?;
                let updated_at = parse_timestamp(row, 6, "updated_at")?;
                Ok(Agent {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    config,
                    state,
                    created_at,
                    updated_at,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(agents)
    }

    fn update(&mut self, agent: &Agent) -> StorageResult<()> {
        validate_entity(agent)?;
        let config_json = serde_json::to_string(&agent.config)?;
        let state_json = serde_json::to_string(&agent.state)?;
        let rows_affected = self.db.conn_mut().execute(
            "UPDATE agents SET name = ?2, description = ?3, config_json = ?4, state = ?5, updated_at = ?6 WHERE id = ?1",
            params![agent.id, agent.name, agent.description, config_json, state_json, agent.updated_at.to_rfc3339()],
        )?;
        if rows_affected == 0 {
            return Err(not_found_error::<Agent>(&agent.id));
        }
        debug!(agent_id = %agent.id, "Updated agent");
        Ok(())
    }

    fn delete(&mut self, id: &str) -> StorageResult<()> {
        let rows_affected =
            self.db.conn_mut().execute("DELETE FROM agents WHERE id = ?1", params![id])?;
        if rows_affected == 0 {
            return Err(not_found_error::<Agent>(id));
        }
        info!(agent_id = %id, "Deleted agent");
        Ok(())
    }
}

// ============================================================================
// SQLite Workflow Repository
// ============================================================================

/// SQLite implementation of WorkflowRepository.
pub struct SqliteWorkflowRepository<'a> {
    db: &'a mut Database,
}

impl<'a> SqliteWorkflowRepository<'a> {
    /// Creates a new SQLite workflow repository.
    pub fn new(db: &'a mut Database) -> Self {
        Self { db }
    }
}

impl SqliteWorkflowRepository<'_> {
    /// Inserts workflow steps into the database.
    fn create_workflow_steps(
        &mut self,
        workflow_id: &str,
        steps: &[WorkflowStep],
    ) -> StorageResult<()> {
        for step in steps {
            let config_json = step.config_json.as_deref().unwrap_or("null");
            self.db.conn_mut().execute(
                "INSERT INTO workflow_steps (id, workflow_id, name, description, task_id, config_json, step_order) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![step.id, workflow_id, step.name, step.description, step.task_id, config_json, step.order],
            )?;
        }
        Ok(())
    }

    /// Loads workflow steps from the database.
    fn load_workflow_steps(&self, workflow_id: &str) -> StorageResult<Vec<WorkflowStep>> {
        let mut stmt = self.db.conn().prepare(
            "SELECT id, name, description, task_id, config_json, step_order FROM workflow_steps WHERE workflow_id = ?1 ORDER BY step_order"
        )?;
        let steps: Vec<WorkflowStep> = stmt
            .query_map(params![workflow_id], |row| {
                let config_json: Option<String> = row.get(4)?;
                Ok(WorkflowStep {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    task_id: row.get(3)?,
                    config_json,
                    order: row.get(5)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(steps)
    }
}

impl WorkflowRepository for SqliteWorkflowRepository<'_> {
    fn create(&mut self, workflow: &Workflow) -> StorageResult<()> {
        validate_entity(workflow)?;
        let state_json = serde_json::to_string(&workflow.state)?;
        self.db.conn_mut().execute(
            "INSERT INTO workflows (id, name, description, state, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![workflow.id, workflow.name, workflow.description, state_json, workflow.created_at.to_rfc3339(), workflow.updated_at.to_rfc3339()],
        )?;
        self.create_workflow_steps(&workflow.id, &workflow.steps)?;
        info!(workflow_id = %workflow.id, "Created workflow");
        Ok(())
    }

    fn get_by_id(&self, id: &str) -> StorageResult<Workflow> {
        let mut stmt = self.db.conn().prepare(
            "SELECT id, name, description, state, created_at, updated_at FROM workflows WHERE id = ?1"
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            let id_str: String = row.get(0)?;
            let name_str: String = row.get(1)?;
            let description_str: String = row.get(2)?;
            let state: WorkflowState = parse_json_field(row, 3, "state")?;
            let created_at = parse_timestamp(row, 4, "created_at")?;
            let updated_at = parse_timestamp(row, 5, "updated_at")?;

            Ok((id_str, name_str, description_str, state, created_at, updated_at))
        })?;

        let (id, name, description, state, created_at, updated_at) = match rows.next() {
            Some(Ok(row)) => row,
            Some(Err(e)) => return Err(e.into()),
            None => return Err(not_found_error::<Workflow>(id)),
        };
        let steps = self.load_workflow_steps(&id)?;
        Ok(Workflow { id, name, description, steps, state, created_at, updated_at })
    }

    fn get_all(&self) -> StorageResult<Vec<Workflow>> {
        let mut stmt =
            self.db.conn().prepare("SELECT id FROM workflows ORDER BY created_at DESC")?;
        let workflow_ids: Vec<String> =
            stmt.query_map([], |row| row.get(0))?.collect::<std::result::Result<Vec<_>, _>>()?;
        let mut workflows = Vec::new();
        for id in workflow_ids {
            workflows.push(self.get_by_id(&id)?);
        }
        Ok(workflows)
    }

    fn update(&mut self, workflow: &Workflow) -> StorageResult<()> {
        validate_entity(workflow)?;
        let state_json = serde_json::to_string(&workflow.state)?;
        let rows_affected = self.db.conn_mut().execute(
            "UPDATE workflows SET name = ?2, description = ?3, state = ?4, updated_at = ?5 WHERE id = ?1",
            params![workflow.id, workflow.name, workflow.description, state_json, workflow.updated_at.to_rfc3339()],
        )?;
        if rows_affected == 0 {
            return Err(not_found_error::<Workflow>(&workflow.id));
        }
        self.db
            .conn_mut()
            .execute("DELETE FROM workflow_steps WHERE workflow_id = ?1", params![workflow.id])?;
        self.create_workflow_steps(&workflow.id, &workflow.steps)?;
        debug!(workflow_id = %workflow.id, "Updated workflow");
        Ok(())
    }

    fn delete(&mut self, id: &str) -> StorageResult<()> {
        let rows_affected =
            self.db.conn_mut().execute("DELETE FROM workflows WHERE id = ?1", params![id])?;
        if rows_affected == 0 {
            return Err(not_found_error::<Workflow>(id));
        }
        info!(workflow_id = %id, "Deleted workflow");
        Ok(())
    }
}

// ============================================================================
// SQLite Task Repository
// ============================================================================

/// SQLite implementation of TaskRepository.
pub struct SqliteTaskRepository<'a> {
    db: &'a mut Database,
}

impl<'a> SqliteTaskRepository<'a> {
    /// Creates a new SQLite task repository.
    pub fn new(db: &'a mut Database) -> Self {
        Self { db }
    }
}

impl TaskRepository for SqliteTaskRepository<'_> {
    fn create(&mut self, task: &Task) -> StorageResult<()> {
        validate_entity(task)?;
        let input_json = serde_json::to_string(&task.input)?;
        let state_json = serde_json::to_string(&task.state)?;
        let result_json = task.result.as_ref().map(|r| serde_json::to_string(r)).transpose()?;
        self.db.conn_mut().execute(
            "INSERT INTO tasks (id, name, description, agent_id, input_json, state, result_json, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![task.id, task.name, task.description, task.agent_id, input_json, state_json, result_json, task.created_at.to_rfc3339(), task.updated_at.to_rfc3339()],
        )?;
        info!(task_id = %task.id, "Created task");
        Ok(())
    }

    fn get_by_id(&self, id: &str) -> StorageResult<Task> {
        let mut stmt = self.db.conn().prepare(
            "SELECT id, name, description, agent_id, input_json, state, result_json, created_at, updated_at FROM tasks WHERE id = ?1"
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            let input: serde_json::Value = parse_json_field(row, 4, "input_json")?;
            let state: TaskState = parse_json_field(row, 5, "state")?;
            let result: Option<TaskResult> = parse_optional_json_field(row, 6, "result_json")?;
            let created_at = parse_timestamp(row, 7, "created_at")?;
            let updated_at = parse_timestamp(row, 8, "updated_at")?;
            Ok(Task {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                agent_id: row.get(3)?,
                input,
                state,
                result,
                created_at,
                updated_at,
            })
        })?;
        match rows.next() {
            Some(Ok(task)) => Ok(task),
            Some(Err(e)) => Err(e.into()),
            None => Err(not_found_error::<Task>(id)),
        }
    }

    fn get_all(&self) -> StorageResult<Vec<Task>> {
        let mut stmt = self.db.conn().prepare(
            "SELECT id, name, description, agent_id, input_json, state, result_json, created_at, updated_at FROM tasks ORDER BY created_at DESC"
        )?;
        let tasks = stmt
            .query_map([], |row| {
                let input: serde_json::Value = parse_json_field(row, 4, "input_json")?;
                let state: TaskState = parse_json_field(row, 5, "state")?;
                let result: Option<TaskResult> = parse_optional_json_field(row, 6, "result_json")?;
                let created_at = parse_timestamp(row, 7, "created_at")?;
                let updated_at = parse_timestamp(row, 8, "updated_at")?;
                Ok(Task {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    agent_id: row.get(3)?,
                    input,
                    state,
                    result,
                    created_at,
                    updated_at,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(tasks)
    }

    fn get_by_agent_id(&self, agent_id: &str) -> StorageResult<Vec<Task>> {
        let mut stmt = self.db.conn().prepare(
            "SELECT id, name, description, agent_id, input_json, state, result_json, created_at, updated_at FROM tasks WHERE agent_id = ?1 ORDER BY created_at DESC"
        )?;
        let tasks = stmt
            .query_map(params![agent_id], |row| {
                let input: serde_json::Value = parse_json_field(row, 4, "input_json")?;
                let state: TaskState = parse_json_field(row, 5, "state")?;
                let result: Option<TaskResult> = parse_optional_json_field(row, 6, "result_json")?;
                let created_at = parse_timestamp(row, 7, "created_at")?;
                let updated_at = parse_timestamp(row, 8, "updated_at")?;
                Ok(Task {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    agent_id: row.get(3)?,
                    input,
                    state,
                    result,
                    created_at,
                    updated_at,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(tasks)
    }

    fn update(&mut self, task: &Task) -> StorageResult<()> {
        validate_entity(task)?;
        let input_json = serde_json::to_string(&task.input)?;
        let state_json = serde_json::to_string(&task.state)?;
        let result_json = task.result.as_ref().map(|r| serde_json::to_string(r)).transpose()?;
        let rows_affected = self.db.conn_mut().execute(
            "UPDATE tasks SET name = ?2, description = ?3, agent_id = ?4, input_json = ?5, state = ?6, result_json = ?7, updated_at = ?8 WHERE id = ?1",
            params![task.id, task.name, task.description, task.agent_id, input_json, state_json, result_json, task.updated_at.to_rfc3339()],
        )?;
        if rows_affected == 0 {
            return Err(not_found_error::<Task>(&task.id));
        }
        debug!(task_id = %task.id, "Updated task");
        Ok(())
    }

    fn delete(&mut self, id: &str) -> StorageResult<()> {
        let rows_affected =
            self.db.conn_mut().execute("DELETE FROM tasks WHERE id = ?1", params![id])?;
        if rows_affected == 0 {
            return Err(not_found_error::<Task>(id));
        }
        info!(task_id = %id, "Deleted task");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        Agent, AgentConfig, AgentState, Task, TaskState, Workflow, WorkflowState, WorkflowStep,
    };
    use serde_json::Value;

    fn setup_db() -> Database {
        Database::open_in_memory().unwrap()
    }

    #[test]
    fn test_agent_repository_create_and_get() {
        let mut db = setup_db();
        let mut repo = SqliteAgentRepository::new(&mut db);

        let config = AgentConfig::new("test-model".to_string());
        let agent = Agent::new(
            "agent-1".to_string(),
            "Test Agent".to_string(),
            "A test agent".to_string(),
            config,
        );

        repo.create(&agent).unwrap();
        let retrieved = repo.get_by_id("agent-1").unwrap();

        assert_eq!(retrieved.id, agent.id);
        assert_eq!(retrieved.name, agent.name);
        assert_eq!(retrieved.config.model_id, agent.config.model_id);
    }

    #[test]
    fn test_agent_repository_get_all() {
        let mut db = setup_db();
        let mut repo = SqliteAgentRepository::new(&mut db);

        let config1 = AgentConfig::new("model-1".to_string());
        let agent1 = Agent::new(
            "agent-1".to_string(),
            "Agent 1".to_string(),
            "First agent".to_string(),
            config1,
        );

        let config2 = AgentConfig::new("model-2".to_string());
        let agent2 = Agent::new(
            "agent-2".to_string(),
            "Agent 2".to_string(),
            "Second agent".to_string(),
            config2,
        );

        repo.create(&agent1).unwrap();
        repo.create(&agent2).unwrap();

        let all = repo.get_all().unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_agent_repository_update() {
        let mut db = setup_db();
        let mut repo = SqliteAgentRepository::new(&mut db);

        let config = AgentConfig::new("test-model".to_string());
        let mut agent = Agent::new(
            "agent-1".to_string(),
            "Test Agent".to_string(),
            "A test agent".to_string(),
            config,
        );

        repo.create(&agent).unwrap();
        agent.set_state(AgentState::Running);
        repo.update(&agent).unwrap();

        let retrieved = repo.get_by_id("agent-1").unwrap();
        assert_eq!(retrieved.state, AgentState::Running);
    }

    #[test]
    fn test_agent_repository_delete() {
        let mut db = setup_db();
        let mut repo = SqliteAgentRepository::new(&mut db);

        let config = AgentConfig::new("test-model".to_string());
        let agent = Agent::new(
            "agent-1".to_string(),
            "Test Agent".to_string(),
            "A test agent".to_string(),
            config,
        );

        repo.create(&agent).unwrap();
        repo.delete("agent-1").unwrap();

        assert!(repo.get_by_id("agent-1").is_err());
    }

    #[test]
    fn test_workflow_repository_create_and_get() {
        let mut db = setup_db();
        let mut repo = SqliteWorkflowRepository::new(&mut db);

        let mut workflow = Workflow::new(
            "workflow-1".to_string(),
            "Test Workflow".to_string(),
            "A test workflow".to_string(),
        );

        let step = WorkflowStep::new(
            "step-1".to_string(),
            "Step 1".to_string(),
            "First step".to_string(),
            "task-1".to_string(),
            0,
        );
        workflow.add_step(step).unwrap();

        repo.create(&workflow).unwrap();
        let retrieved = repo.get_by_id("workflow-1").unwrap();

        assert_eq!(retrieved.id, workflow.id);
        assert_eq!(retrieved.steps.len(), 1);
        assert_eq!(retrieved.steps[0].task_id, "task-1");
    }

    #[test]
    fn test_workflow_repository_update() {
        let mut db = setup_db();
        let mut repo = SqliteWorkflowRepository::new(&mut db);

        let mut workflow = Workflow::new(
            "workflow-1".to_string(),
            "Test Workflow".to_string(),
            "A test workflow".to_string(),
        );

        repo.create(&workflow).unwrap();
        workflow.set_state(WorkflowState::Running);
        repo.update(&workflow).unwrap();

        let retrieved = repo.get_by_id("workflow-1").unwrap();
        assert_eq!(retrieved.state, WorkflowState::Running);
    }

    #[test]
    fn test_task_repository_create_and_get() {
        let mut db = setup_db();
        let mut repo = SqliteTaskRepository::new(&mut db);

        let task = Task::new(
            "task-1".to_string(),
            "Test Task".to_string(),
            "A test task".to_string(),
            "agent-1".to_string(),
            Value::String("input".to_string()),
        );

        repo.create(&task).unwrap();
        let retrieved = repo.get_by_id("task-1").unwrap();

        assert_eq!(retrieved.id, task.id);
        assert_eq!(retrieved.agent_id, "agent-1");
    }

    #[test]
    fn test_task_repository_get_by_agent_id() {
        let mut db = setup_db();
        let mut repo = SqliteTaskRepository::new(&mut db);

        let task1 = Task::new(
            "task-1".to_string(),
            "Task 1".to_string(),
            "First task".to_string(),
            "agent-1".to_string(),
            Value::Null,
        );

        let task2 = Task::new(
            "task-2".to_string(),
            "Task 2".to_string(),
            "Second task".to_string(),
            "agent-1".to_string(),
            Value::Null,
        );

        let task3 = Task::new(
            "task-3".to_string(),
            "Task 3".to_string(),
            "Third task".to_string(),
            "agent-2".to_string(),
            Value::Null,
        );

        repo.create(&task1).unwrap();
        repo.create(&task2).unwrap();
        repo.create(&task3).unwrap();

        let agent1_tasks = repo.get_by_agent_id("agent-1").unwrap();
        assert_eq!(agent1_tasks.len(), 2);

        let agent2_tasks = repo.get_by_agent_id("agent-2").unwrap();
        assert_eq!(agent2_tasks.len(), 1);
    }

    #[test]
    fn test_task_repository_update() {
        let mut db = setup_db();
        let mut repo = SqliteTaskRepository::new(&mut db);

        let mut task = Task::new(
            "task-1".to_string(),
            "Test Task".to_string(),
            "A test task".to_string(),
            "agent-1".to_string(),
            Value::Null,
        );

        repo.create(&task).unwrap();
        task.set_state(TaskState::Running);
        repo.update(&task).unwrap();

        let retrieved = repo.get_by_id("task-1").unwrap();
        assert_eq!(retrieved.state, TaskState::Running);
    }

    #[test]
    fn test_workflow_repository_delete() {
        let mut db = setup_db();
        let mut repo = SqliteWorkflowRepository::new(&mut db);

        let workflow = Workflow::new(
            "workflow-1".to_string(),
            "Test Workflow".to_string(),
            "A test workflow".to_string(),
        );

        repo.create(&workflow).unwrap();
        repo.delete("workflow-1").unwrap();

        assert!(repo.get_by_id("workflow-1").is_err());
    }

    #[test]
    fn test_workflow_repository_get_all() {
        let mut db = setup_db();
        let mut repo = SqliteWorkflowRepository::new(&mut db);

        let workflow1 = Workflow::new(
            "workflow-1".to_string(),
            "Workflow 1".to_string(),
            "First workflow".to_string(),
        );

        let workflow2 = Workflow::new(
            "workflow-2".to_string(),
            "Workflow 2".to_string(),
            "Second workflow".to_string(),
        );

        repo.create(&workflow1).unwrap();
        repo.create(&workflow2).unwrap();

        let all = repo.get_all().unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_task_repository_delete() {
        let mut db = setup_db();
        let mut repo = SqliteTaskRepository::new(&mut db);

        let task = Task::new(
            "task-1".to_string(),
            "Test Task".to_string(),
            "A test task".to_string(),
            "agent-1".to_string(),
            Value::Null,
        );

        repo.create(&task).unwrap();
        repo.delete("task-1").unwrap();

        assert!(repo.get_by_id("task-1").is_err());
    }

    #[test]
    fn test_task_repository_get_all() {
        let mut db = setup_db();
        let mut repo = SqliteTaskRepository::new(&mut db);

        let task1 = Task::new(
            "task-1".to_string(),
            "Task 1".to_string(),
            "First task".to_string(),
            "agent-1".to_string(),
            Value::Null,
        );

        let task2 = Task::new(
            "task-2".to_string(),
            "Task 2".to_string(),
            "Second task".to_string(),
            "agent-2".to_string(),
            Value::Null,
        );

        repo.create(&task1).unwrap();
        repo.create(&task2).unwrap();

        let all = repo.get_all().unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_agent_repository_get_nonexistent() {
        let mut db = setup_db();
        let repo = SqliteAgentRepository::new(&mut db);

        let result = repo.get_by_id("nonexistent-agent");
        assert!(result.is_err());
    }

    #[test]
    fn test_workflow_repository_update_nonexistent() {
        let mut db = setup_db();
        let mut repo = SqliteWorkflowRepository::new(&mut db);

        let workflow = Workflow::new(
            "nonexistent-workflow".to_string(),
            "Test".to_string(),
            "Test".to_string(),
        );

        let result = repo.update(&workflow);
        assert!(result.is_err());
    }

    #[test]
    fn test_task_repository_delete_nonexistent() {
        let mut db = setup_db();
        let mut repo = SqliteTaskRepository::new(&mut db);

        let result = repo.delete("nonexistent-task");
        assert!(result.is_err());
    }

    #[test]
    fn test_agent_repository_create_duplicate_id() {
        let mut db = setup_db();
        let mut repo = SqliteAgentRepository::new(&mut db);

        let config = AgentConfig::new("test-model".to_string());
        let agent = Agent::new(
            "duplicate-agent".to_string(),
            "Agent 1".to_string(),
            "First".to_string(),
            config.clone(),
        );

        repo.create(&agent).unwrap();

        // Try to create another agent with the same ID
        let agent2 = Agent::new(
            "duplicate-agent".to_string(),
            "Agent 2".to_string(),
            "Second".to_string(),
            config,
        );

        let result = repo.create(&agent2);
        assert!(result.is_err());
    }

    #[test]
    fn test_agent_repository_unicode_name() {
        let mut db = setup_db();
        let mut repo = SqliteAgentRepository::new(&mut db);

        let config = AgentConfig::new("test-model".to_string());
        let agent = Agent::new(
            "unicode-agent".to_string(),
            "エージェント 🤖".to_string(),
            "描述文本 with émojis 🎉".to_string(),
            config,
        );

        repo.create(&agent).unwrap();
        let retrieved = repo.get_by_id("unicode-agent").unwrap();

        assert_eq!(retrieved.name, "エージェント 🤖");
        assert_eq!(retrieved.description, "描述文本 with émojis 🎉");
    }

    #[test]
    fn test_agent_repository_complex_config() {
        use radium_abstraction::ModelParameters;

        let mut db = setup_db();
        let mut repo = SqliteAgentRepository::new(&mut db);

        let mut config = AgentConfig::new("test-model".to_string());
        config.model_parameters = Some(ModelParameters {
            temperature: Some(0.7),
            top_p: Some(0.9),
            max_tokens: Some(2000),
            stop_sequences: None,
        });
        config.max_iterations = Some(10);
        config.timeout_seconds = Some(300);

        let agent = Agent::new(
            "complex-agent".to_string(),
            "Complex Agent".to_string(),
            "Agent with complex config".to_string(),
            config.clone(),
        );

        repo.create(&agent).unwrap();
        let retrieved = repo.get_by_id("complex-agent").unwrap();

        assert!(retrieved.config.model_parameters.is_some());
        let params = retrieved.config.model_parameters.unwrap();
        assert_eq!(params.max_tokens, Some(2000));
        assert_eq!(params.temperature, Some(0.7));
        assert_eq!(params.top_p, Some(0.9));
        assert_eq!(retrieved.config.max_iterations, Some(10));
        assert_eq!(retrieved.config.timeout_seconds, Some(300));
    }

    #[test]
    fn test_agent_repository_get_all_empty() {
        let mut db = setup_db();
        let repo = SqliteAgentRepository::new(&mut db);

        let all = repo.get_all().unwrap();
        assert_eq!(all.len(), 0);
    }

    #[test]
    fn test_agent_repository_update_nonexistent() {
        let mut db = setup_db();
        let mut repo = SqliteAgentRepository::new(&mut db);

        let config = AgentConfig::new("test-model".to_string());
        let agent = Agent::new(
            "nonexistent-agent".to_string(),
            "Test".to_string(),
            "Test".to_string(),
            config,
        );

        let result = repo.update(&agent);
        assert!(result.is_err());
    }

    #[test]
    fn test_workflow_repository_get_nonexistent() {
        let mut db = setup_db();
        let repo = SqliteWorkflowRepository::new(&mut db);

        let result = repo.get_by_id("nonexistent-workflow");
        assert!(result.is_err());
    }

    #[test]
    fn test_workflow_repository_delete_nonexistent() {
        let mut db = setup_db();
        let mut repo = SqliteWorkflowRepository::new(&mut db);

        let result = repo.delete("nonexistent-workflow");
        assert!(result.is_err());
    }

    #[test]
    fn test_workflow_repository_with_multiple_steps() {
        let mut db = setup_db();
        let mut repo = SqliteWorkflowRepository::new(&mut db);

        let mut workflow = Workflow::new(
            "multi-step-workflow".to_string(),
            "Multi-Step Workflow".to_string(),
            "Workflow with multiple steps".to_string(),
        );

        // Add multiple steps
        for i in 0..5 {
            let step = WorkflowStep::new(
                format!("step-{}", i),
                format!("Step {}", i),
                format!("Description {}", i),
                format!("task-{}", i),
                i,
            );
            workflow.add_step(step).unwrap();
        }

        repo.create(&workflow).unwrap();
        let retrieved = repo.get_by_id("multi-step-workflow").unwrap();

        assert_eq!(retrieved.steps.len(), 5);
        // Verify steps are in order
        for (i, step) in retrieved.steps.iter().enumerate() {
            assert_eq!(step.order, i as u32);
        }
    }

    #[test]
    fn test_workflow_repository_update_steps() {
        let mut db = setup_db();
        let mut repo = SqliteWorkflowRepository::new(&mut db);

        let mut workflow = Workflow::new(
            "update-steps-workflow".to_string(),
            "Update Steps".to_string(),
            "Test updating steps".to_string(),
        );

        let step1 = WorkflowStep::new(
            "step-1".to_string(),
            "Step 1".to_string(),
            "First step".to_string(),
            "task-1".to_string(),
            0,
        );
        workflow.add_step(step1).unwrap();

        repo.create(&workflow).unwrap();

        // Update workflow with different steps
        workflow.steps.clear();
        let step2 = WorkflowStep::new(
            "step-2".to_string(),
            "Step 2".to_string(),
            "Second step".to_string(),
            "task-2".to_string(),
            0,
        );
        workflow.add_step(step2).unwrap();

        repo.update(&workflow).unwrap();
        let retrieved = repo.get_by_id("update-steps-workflow").unwrap();

        assert_eq!(retrieved.steps.len(), 1);
        assert_eq!(retrieved.steps[0].id, "step-2");
    }

    #[test]
    fn test_workflow_repository_empty_steps() {
        let mut db = setup_db();
        let mut repo = SqliteWorkflowRepository::new(&mut db);

        let workflow = Workflow::new(
            "empty-workflow".to_string(),
            "Empty Workflow".to_string(),
            "Workflow with no steps".to_string(),
        );

        repo.create(&workflow).unwrap();
        let retrieved = repo.get_by_id("empty-workflow").unwrap();

        assert_eq!(retrieved.steps.len(), 0);
    }

    #[test]
    fn test_task_repository_get_nonexistent() {
        let mut db = setup_db();
        let repo = SqliteTaskRepository::new(&mut db);

        let result = repo.get_by_id("nonexistent-task");
        assert!(result.is_err());
    }

    #[test]
    fn test_task_repository_update_nonexistent() {
        let mut db = setup_db();
        let mut repo = SqliteTaskRepository::new(&mut db);

        let task = Task::new(
            "nonexistent-task".to_string(),
            "Test".to_string(),
            "Test".to_string(),
            "agent-1".to_string(),
            Value::Null,
        );

        let result = repo.update(&task);
        assert!(result.is_err());
    }

    #[test]
    fn test_task_repository_get_by_agent_id_empty() {
        let mut db = setup_db();
        let repo = SqliteTaskRepository::new(&mut db);

        let tasks = repo.get_by_agent_id("nonexistent-agent").unwrap();
        assert_eq!(tasks.len(), 0);
    }

    #[test]
    fn test_task_repository_with_result() {
        let mut db = setup_db();
        let mut repo = SqliteTaskRepository::new(&mut db);

        let mut task = Task::new(
            "task-with-result".to_string(),
            "Task With Result".to_string(),
            "Task that has a result".to_string(),
            "agent-1".to_string(),
            Value::String("input data".to_string()),
        );

        repo.create(&task).unwrap();

        // Update with result
        let started = chrono::Utc::now();
        let completed = started + chrono::Duration::seconds(5);
        task.result = Some(crate::models::TaskResult {
            output: Value::String("output data".to_string()),
            error: None,
            started_at: started,
            completed_at: Some(completed),
            duration_ms: Some(5000),
        });
        task.set_state(TaskState::Completed);
        repo.update(&task).unwrap();

        let retrieved = repo.get_by_id("task-with-result").unwrap();
        assert!(retrieved.result.is_some());
        assert_eq!(retrieved.state, TaskState::Completed);
    }

    #[test]
    fn test_task_repository_complex_input() {
        let mut db = setup_db();
        let mut repo = SqliteTaskRepository::new(&mut db);

        let complex_input = serde_json::json!({
            "prompt": "Test prompt",
            "parameters": {
                "temperature": 0.7,
                "max_tokens": 100
            },
            "metadata": {
                "user_id": "user-123",
                "session": "session-456"
            }
        });

        let task = Task::new(
            "complex-task".to_string(),
            "Complex Task".to_string(),
            "Task with complex JSON input".to_string(),
            "agent-1".to_string(),
            complex_input.clone(),
        );

        repo.create(&task).unwrap();
        let retrieved = repo.get_by_id("complex-task").unwrap();

        assert_eq!(retrieved.input, complex_input);
    }

    #[test]
    fn test_agent_repository_get_all_ordering() {
        let mut db = setup_db();
        let mut repo = SqliteAgentRepository::new(&mut db);

        let config = AgentConfig::new("test-model".to_string());

        // Create agents with different timestamps
        for i in 0..3 {
            let agent = Agent::new(
                format!("agent-{}", i),
                format!("Agent {}", i),
                "Description".to_string(),
                config.clone(),
            );
            repo.create(&agent).unwrap();
            // Small delay to ensure different timestamps
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        let all = repo.get_all().unwrap();
        // Should be ordered by created_at DESC (newest first)
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].id, "agent-2");
        assert_eq!(all[1].id, "agent-1");
        assert_eq!(all[2].id, "agent-0");
    }

    #[test]
    fn test_workflow_repository_get_all_empty() {
        let mut db = setup_db();
        let repo = SqliteWorkflowRepository::new(&mut db);

        let all = repo.get_all().unwrap();
        assert_eq!(all.len(), 0);
    }

    #[test]
    fn test_task_repository_get_all_empty() {
        let mut db = setup_db();
        let repo = SqliteTaskRepository::new(&mut db);

        let all = repo.get_all().unwrap();
        assert_eq!(all.len(), 0);
    }
}
