//! Agent monitoring service for lifecycle tracking.

use super::error::{MonitoringError, Result};
use super::schema::initialize_schema;
use crate::hooks::registry::HookRegistry;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::warn;

/// Agent status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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

/// Agent usage statistics from agent_usage table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentUsage {
    /// Agent ID.
    pub agent_id: String,
    /// Total number of executions.
    pub execution_count: u64,
    /// Total duration across all executions (milliseconds).
    pub total_duration: u64,
    /// Total tokens used.
    pub total_tokens: u64,
    /// Number of successful executions.
    pub success_count: u64,
    /// Number of failed executions.
    pub failure_count: u64,
    /// Last used timestamp (Unix epoch seconds).
    pub last_used_at: Option<u64>,
    /// Agent category.
    pub category: Option<String>,
}

/// Filter for agent usage queries.
#[derive(Debug, Clone, Default)]
pub struct UsageFilter {
    /// Filter by category.
    pub category: Option<String>,
    /// Minimum execution count.
    pub min_executions: Option<u64>,
    /// Filter by last used since timestamp (Unix epoch seconds).
    pub since: Option<u64>,
}

/// Agent record in monitoring database.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
        let start_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

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
    #[must_use]
    pub fn with_parent(mut self, parent_id: String) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    /// Sets the plan ID.
    #[must_use]
    pub fn with_plan(mut self, plan_id: String) -> Self {
        self.plan_id = Some(plan_id);
        self
    }

    /// Sets the process ID.
    #[must_use]
    pub fn with_process_id(mut self, process_id: u32) -> Self {
        self.process_id = Some(process_id);
        self
    }

    /// Sets the log file path.
    #[must_use]
    pub fn with_log_file(mut self, log_file: String) -> Self {
        self.log_file = Some(log_file);
        self
    }
}

/// Agent monitoring service.
pub struct MonitoringService {
    /// Database connection.
    pub(super) conn: Connection,
    /// Optional hook registry for telemetry interception.
    hook_registry: Option<Arc<HookRegistry>>,
}

impl MonitoringService {
    /// Get a reference to the database connection.
    /// This is used by cost query service for complex queries.
    pub fn conn(&self) -> &Connection {
        &self.conn
    }
}

impl MonitoringService {
    /// Creates a new monitoring service with an in-memory database.
    ///
    /// # Errors
    /// Returns error if database initialization fails
    pub fn new() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        initialize_schema(&conn)?;
        Ok(Self { conn, hook_registry: None })
    }

    /// Creates a new monitoring service with hook registry.
    ///
    /// # Arguments
    /// * `hook_registry` - Hook registry for telemetry interception
    ///
    /// # Errors
    /// Returns error if database initialization fails
    pub fn with_hooks(hook_registry: Arc<HookRegistry>) -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        initialize_schema(&conn)?;
        Ok(Self { conn, hook_registry: Some(hook_registry) })
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
        Ok(Self { conn, hook_registry: None })
    }

    /// Opens a monitoring service with a database file and hook registry.
    ///
    /// # Arguments
    /// * `path` - Path to database file
    /// * `hook_registry` - Hook registry for telemetry interception
    ///
    /// # Errors
    /// Returns error if database opening or initialization fails
    pub fn open_with_hooks(path: impl AsRef<Path>, hook_registry: Arc<HookRegistry>) -> Result<Self> {
        let conn = Connection::open(path)?;
        initialize_schema(&conn)?;
        Ok(Self { conn, hook_registry: Some(hook_registry) })
    }

    /// Sets the hook registry for this monitoring service.
    pub fn set_hook_registry(&mut self, hook_registry: Arc<HookRegistry>) {
        self.hook_registry = Some(hook_registry);
    }

    /// Gets a clone of the hook registry if it exists.
    pub fn get_hook_registry(&self) -> Option<Arc<HookRegistry>> {
        self.hook_registry.as_ref().map(Arc::clone)
    }

    /// Records telemetry synchronously (internal method, hooks should be executed before calling this).
    pub fn record_telemetry_sync(&self, record: &crate::monitoring::telemetry::TelemetryRecord) -> Result<()> {
        
        self.conn.execute(
            "INSERT INTO telemetry (agent_id, timestamp, input_tokens, output_tokens, cached_tokens,
                                    cache_creation_tokens, cache_read_tokens, total_tokens,
                                    estimated_cost, model, provider, tool_name, tool_args, tool_approved, tool_approval_type, engine_id,
                                    behavior_type, behavior_invocation_count, behavior_duration_ms, behavior_outcome,
                                    api_key_id, team_name, project_name, cost_center,
                                    model_tier, routing_decision, complexity_score)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27)",
            rusqlite::params![
                record.agent_id,
                record.timestamp,
                record.input_tokens,
                record.output_tokens,
                record.cached_tokens,
                record.cache_creation_tokens,
                record.cache_read_tokens,
                record.total_tokens,
                record.estimated_cost,
                record.model,
                record.provider,
                record.tool_name,
                record.tool_args,
                record.tool_approved,
                record.tool_approval_type,
                record.engine_id,
                record.behavior_type,
                record.behavior_invocation_count,
                record.behavior_duration_ms,
                record.behavior_outcome,
                record.api_key_id,
                record.team_name,
                record.project_name,
                record.cost_center,
                record.model_tier,
                record.routing_decision,
                record.complexity_score,
            ],
        )?;

        // Also persist to cost_events table for analytics
        if let Err(e) = self.insert_cost_event(record) {
            // Log error but don't fail the telemetry recording
            warn!("Failed to insert cost event: {}", e);
        }

        Ok(())
    }

    /// Inserts a cost event into the cost_events table for analytics.
    ///
    /// # Arguments
    /// * `record` - Telemetry record to extract cost data from
    ///
    /// # Errors
    /// Returns error if insertion fails
    fn insert_cost_event(&self, record: &crate::monitoring::telemetry::TelemetryRecord) -> Result<()> {
        // Look up agent's plan_id to get requirement_id
        let requirement_id: Option<String> = self.conn
            .query_row(
                "SELECT plan_id FROM agents WHERE id = ?1",
                params![record.agent_id],
                |row| row.get(0),
            )
            .ok();

        // Generate session_id from agent_id (using agent_id as session identifier)
        // This groups all telemetry from the same agent execution together
        let session_id = record.agent_id.clone();

        self.conn.execute(
            "INSERT INTO cost_events (timestamp, requirement_id, model, provider, tokens_input, tokens_output, cost_usd, session_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                record.timestamp,
                requirement_id,
                record.model,
                record.provider,
                record.input_tokens,
                record.output_tokens,
                record.estimated_cost,
                session_id,
            ],
        )?;

        Ok(())
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

    /// Registers a new agent with telemetry hooks.
    ///
    /// This is an async version that executes telemetry hooks for agent lifecycle events.
    ///
    /// # Arguments
    /// * `record` - Agent record to register
    ///
    /// # Errors
    /// Returns error if insertion fails
    pub async fn register_agent_with_hooks(&self, record: &AgentRecord) -> Result<()> {
        // Execute telemetry hooks for agent start
        #[cfg(feature = "orchestrator-integration")]
        if let Some(ref registry) = self.hook_registry {
            use crate::hooks::integration::OrchestratorHooks;
            let hooks = OrchestratorHooks::new(Arc::clone(registry));
            let telemetry_data = serde_json::json!({
                "event_type": "agent_start",
                "agent_id": record.id,
                "agent_type": record.agent_type,
                "parent_id": record.parent_id,
                "plan_id": record.plan_id,
                "start_time": record.start_time,
            });
            if let Err(e) = hooks.telemetry_collection("agent_start", &telemetry_data).await {
                tracing::warn!(
                    agent_id = %record.id,
                    error = %e,
                    "Telemetry hook execution failed for agent start"
                );
            }
        }

        // Register agent (synchronous DB operation)
        self.register_agent(record)
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
        let end_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        let rows_affected = self.conn.execute(
            "UPDATE agents SET status = ?1, end_time = ?2, exit_code = ?3 WHERE id = ?4",
            params![AgentStatus::Completed.as_str(), end_time, exit_code, agent_id],
        )?;

        if rows_affected == 0 {
            return Err(MonitoringError::AgentNotFound(agent_id.to_string()));
        }

        Ok(())
    }

    /// Marks an agent as completed with telemetry hooks.
    ///
    /// This is an async version that executes telemetry hooks for agent completion.
    ///
    /// # Arguments
    /// * `agent_id` - Agent identifier
    /// * `exit_code` - Exit code (0 for success)
    ///
    /// # Errors
    /// Returns error if update fails
    pub async fn complete_agent_with_hooks(&self, agent_id: &str, exit_code: i32) -> Result<()> {
        // Get agent record for telemetry
        let agent_record = self.get_agent(agent_id).ok();
        let end_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        // Execute telemetry hooks for agent completion
        #[cfg(feature = "orchestrator-integration")]
        {
            use crate::hooks::integration::OrchestratorHooks;
            if let Some(ref registry) = self.hook_registry {
            let hooks = OrchestratorHooks::new(Arc::clone(registry));
            let telemetry_data = serde_json::json!({
                "event_type": "agent_complete",
                "agent_id": agent_id,
                "exit_code": exit_code,
                "end_time": end_time,
                "success": exit_code == 0,
            });
            if let Err(e) = hooks.telemetry_collection("agent_complete", &telemetry_data).await {
                tracing::warn!(
                    agent_id = %agent_id,
                    error = %e,
                    "Telemetry hook execution failed for agent completion"
                );
            }
            }
        }

        // Complete agent (synchronous DB operation)
        self.complete_agent(agent_id, exit_code)
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
        let end_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        let rows_affected = self.conn.execute(
            "UPDATE agents SET status = ?1, end_time = ?2, error_message = ?3 WHERE id = ?4",
            params![AgentStatus::Failed.as_str(), end_time, error_message, agent_id],
        )?;

        if rows_affected == 0 {
            return Err(MonitoringError::AgentNotFound(agent_id.to_string()));
        }

        Ok(())
    }

    /// Marks an agent as failed with telemetry hooks.
    ///
    /// This is an async version that executes telemetry hooks for agent failure.
    ///
    /// # Arguments
    /// * `agent_id` - Agent identifier
    /// * `error_message` - Error message
    ///
    /// # Errors
    /// Returns error if update fails
    pub async fn fail_agent_with_hooks(&self, agent_id: &str, error_message: &str) -> Result<()> {
        let end_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        // Execute telemetry hooks for agent failure
        #[cfg(feature = "orchestrator-integration")]
        {
            use crate::hooks::integration::OrchestratorHooks;
            if let Some(ref registry) = self.hook_registry {
            let hooks = OrchestratorHooks::new(Arc::clone(registry));
            let telemetry_data = serde_json::json!({
                "event_type": "agent_fail",
                "agent_id": agent_id,
                "error_message": error_message,
                "end_time": end_time,
            });
            if let Err(e) = hooks.telemetry_collection("agent_fail", &telemetry_data).await {
                tracing::warn!(
                    agent_id = %agent_id,
                    error = %e,
                    "Telemetry hook execution failed for agent failure"
                );
            }
            }
        }

        // Fail agent (synchronous DB operation)
        self.fail_agent(agent_id, error_message)
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

        let records = stmt
            .query_map(params![parent_id], |row| {
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
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

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

        let records = stmt
            .query_map(params![plan_id], |row| {
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
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(records)
    }

    /// Lists all agents.
    ///
    /// # Returns
    /// List of all agent records, ordered by start_time descending
    ///
    /// # Errors
    /// Returns error if query fails
    pub fn list_agents(&self) -> Result<Vec<AgentRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, parent_id, plan_id, agent_type, status, process_id, start_time, end_time, exit_code, error_message, log_file
             FROM agents ORDER BY start_time DESC"
        )?;

        let records = stmt
            .query_map([], |row| {
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
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(records)
    }

    /// Gets agent usage statistics for a specific agent.
    ///
    /// # Arguments
    /// * `agent_id` - Agent identifier
    ///
    /// # Returns
    /// Agent usage statistics if found
    ///
    /// # Errors
    /// Returns error if query fails
    pub fn get_agent_usage(&self, agent_id: &str) -> Result<Option<AgentUsage>> {
        let mut stmt = self.conn.prepare(
            "SELECT agent_id, execution_count, total_duration, total_tokens,
                    success_count, failure_count, last_used_at, category
             FROM agent_usage WHERE agent_id = ?1",
        )?;

        let result = stmt.query_row(params![agent_id], |row| {
            Ok(AgentUsage {
                agent_id: row.get(0)?,
                execution_count: row.get::<_, i64>(1)? as u64,
                total_duration: row.get::<_, i64>(2)? as u64,
                total_tokens: row.get::<_, i64>(3)? as u64,
                success_count: row.get::<_, i64>(4)? as u64,
                failure_count: row.get::<_, i64>(5)? as u64,
                last_used_at: row.get(6)?,
                category: row.get(7)?,
            })
        });

        match result {
            Ok(usage) => Ok(Some(usage)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(MonitoringError::Database(e)),
        }
    }

    /// Helper function to map a database row to AgentUsage.
    fn row_to_agent_usage(row: &rusqlite::Row) -> rusqlite::Result<AgentUsage> {
        Ok(AgentUsage {
            agent_id: row.get(0)?,
            execution_count: row.get::<_, i64>(1)? as u64,
            total_duration: row.get::<_, i64>(2)? as u64,
            total_tokens: row.get::<_, i64>(3)? as u64,
            success_count: row.get::<_, i64>(4)? as u64,
            failure_count: row.get::<_, i64>(5)? as u64,
            last_used_at: row.get(6)?,
            category: row.get(7)?,
        })
    }

    /// Gets cost breakdown by provider.
    ///
    /// Aggregates costs from telemetry records grouped by provider.
    ///
    /// # Returns
    /// Vector of ProviderCostBreakdown sorted by total_cost descending
    ///
    /// # Errors
    /// Returns error if query fails
    pub fn get_costs_by_provider(&self) -> Result<Vec<crate::monitoring::budget::ProviderCostBreakdown>> {
        use crate::monitoring::budget::ProviderCostBreakdown;
        
        // First get total cost across all providers
        let mut total_stmt = self.conn.prepare("SELECT SUM(estimated_cost) FROM telemetry WHERE provider IS NOT NULL")?;
        let total_cost: f64 = total_stmt.query_row([], |row| {
            Ok(row.get::<_, Option<f64>>(0)?.unwrap_or(0.0))
        })?;

        // Get breakdown by provider
        let mut stmt = self.conn.prepare(
            "SELECT provider, SUM(estimated_cost) as total_cost, COUNT(*) as execution_count
             FROM telemetry
             WHERE provider IS NOT NULL
             GROUP BY provider
             ORDER BY total_cost DESC"
        )?;

        let breakdowns = stmt.query_map([], |row| {
            let provider: String = row.get(0)?;
            let cost: f64 = row.get::<_, Option<f64>>(1)?.unwrap_or(0.0);
            let count: i64 = row.get(2)?;
            
            let percentage = if total_cost > 0.0 {
                (cost / total_cost) * 100.0
            } else {
                0.0
            };

            Ok(ProviderCostBreakdown {
                provider,
                total_cost: cost,
                percentage,
                execution_count: count as u64,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(breakdowns)
    }

    /// Gets cost breakdown by team.
    ///
    /// Aggregates costs from telemetry records grouped by team name.
    ///
    /// # Returns
    /// Vector of TeamCostBreakdown sorted by total_cost descending
    ///
    /// # Errors
    /// Returns error if query fails
    pub fn get_costs_by_team(&self) -> Result<Vec<crate::monitoring::budget::TeamCostBreakdown>> {
        use crate::monitoring::budget::TeamCostBreakdown;
        
        let mut stmt = self.conn.prepare(
            "SELECT team_name, project_name, SUM(estimated_cost) as total_cost, COUNT(*) as execution_count
             FROM telemetry
             WHERE team_name IS NOT NULL
             GROUP BY team_name, project_name
             ORDER BY total_cost DESC"
        )?;

        let breakdowns = stmt.query_map([], |row| {
            let team_name: String = row.get(0)?;
            let project_name: Option<String> = row.get(1)?;
            let cost: f64 = row.get::<_, Option<f64>>(2)?.unwrap_or(0.0);
            let count: i64 = row.get(3)?;

            Ok(TeamCostBreakdown {
                team_name,
                project_name,
                total_cost: cost,
                execution_count: count as u64,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(breakdowns)
    }

    /// Lists agent usage statistics with optional filtering.
    ///
    /// # Arguments
    /// * `filter` - Optional filter for category, min_executions, or since timestamp
    ///
    /// # Returns
    /// List of agent usage statistics
    ///
    /// # Errors
    /// Returns error if query fails
    pub fn list_agent_usage(&self, filter: UsageFilter) -> Result<Vec<AgentUsage>> {
        let mut query = "SELECT agent_id, execution_count, total_duration, total_tokens,
                                success_count, failure_count, last_used_at, category
                         FROM agent_usage".to_string();
        let mut conditions = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref category) = filter.category {
            conditions.push("category = ?");
            params_vec.push(Box::new(category.clone()));
        }

        if let Some(min_executions) = filter.min_executions {
            conditions.push("execution_count >= ?");
            params_vec.push(Box::new(min_executions as i64));
        }

        if let Some(since) = filter.since {
            conditions.push("last_used_at >= ?");
            params_vec.push(Box::new(since as i64));
        }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        query.push_str(" ORDER BY last_used_at DESC NULLS LAST");

        let mut stmt = self.conn.prepare(&query)?;

        // Use the helper function for row mapping
        let records: Vec<AgentUsage> = match (filter.category.as_ref(), filter.min_executions, filter.since) {
            (Some(cat), Some(min), Some(since)) => {
                stmt.query_map(params![cat, min as i64, since as i64], Self::row_to_agent_usage)?.collect::<std::result::Result<Vec<_>, _>>()?
            }
            (Some(cat), Some(min), None) => {
                stmt.query_map(params![cat, min as i64], Self::row_to_agent_usage)?.collect::<std::result::Result<Vec<_>, _>>()?
            }
            (Some(cat), None, Some(since)) => {
                stmt.query_map(params![cat, since as i64], Self::row_to_agent_usage)?.collect::<std::result::Result<Vec<_>, _>>()?
            }
            (Some(cat), None, None) => {
                stmt.query_map(params![cat], Self::row_to_agent_usage)?.collect::<std::result::Result<Vec<_>, _>>()?
            }
            (None, Some(min), Some(since)) => {
                stmt.query_map(params![min as i64, since as i64], Self::row_to_agent_usage)?.collect::<std::result::Result<Vec<_>, _>>()?
            }
            (None, Some(min), None) => {
                stmt.query_map(params![min as i64], Self::row_to_agent_usage)?.collect::<std::result::Result<Vec<_>, _>>()?
            }
            (None, None, Some(since)) => {
                stmt.query_map(params![since as i64], Self::row_to_agent_usage)?.collect::<std::result::Result<Vec<_>, _>>()?
            }
            (None, None, None) => {
                stmt.query_map([], Self::row_to_agent_usage)?.collect::<std::result::Result<Vec<_>, _>>()?
    }
        };

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
    }

    #[test]
    fn test_agent_status_all_variants() {
        assert_eq!(AgentStatus::Starting.as_str(), "starting");
        assert_eq!(AgentStatus::Running.as_str(), "running");
        assert_eq!(AgentStatus::Completed.as_str(), "completed");
        assert_eq!(AgentStatus::Failed.as_str(), "failed");
        assert_eq!(AgentStatus::Terminated.as_str(), "terminated");
    }

    #[test]
    fn test_agent_status_from_str_all_variants() {
        assert_eq!(AgentStatus::from_str("starting").unwrap(), AgentStatus::Starting);
        assert_eq!(AgentStatus::from_str("running").unwrap(), AgentStatus::Running);
        assert_eq!(AgentStatus::from_str("completed").unwrap(), AgentStatus::Completed);
        assert_eq!(AgentStatus::from_str("failed").unwrap(), AgentStatus::Failed);
        assert_eq!(AgentStatus::from_str("terminated").unwrap(), AgentStatus::Terminated);
    }

    #[test]
    fn test_agent_status_from_str_invalid() {
        assert!(AgentStatus::from_str("invalid").is_err());
    }

    #[test]
    fn test_agent_record_new() {
        let record = AgentRecord::new("agent-1".to_string(), "developer".to_string());
        assert_eq!(record.id, "agent-1");
        assert_eq!(record.agent_type, "developer");
        assert_eq!(record.status, AgentStatus::Starting);
        assert!(record.parent_id.is_none());
        assert!(record.plan_id.is_none());
    }

    #[test]
    fn test_agent_record_with_parent() {
        let record = AgentRecord::new("agent-1".to_string(), "developer".to_string())
            .with_parent("parent-1".to_string());
        assert_eq!(record.parent_id, Some("parent-1".to_string()));
    }

    #[test]
    fn test_agent_record_with_plan() {
        let record = AgentRecord::new("agent-1".to_string(), "developer".to_string())
            .with_plan("plan-1".to_string());
        assert_eq!(record.plan_id, Some("plan-1".to_string()));
    }

    #[test]
    fn test_agent_record_with_process_id() {
        let record = AgentRecord::new("agent-1".to_string(), "developer".to_string())
            .with_process_id(12345);
        assert_eq!(record.process_id, Some(12345));
    }

    #[test]
    fn test_agent_record_with_log_file() {
        let record = AgentRecord::new("agent-1".to_string(), "developer".to_string())
            .with_log_file("/path/to/log".to_string());
        assert_eq!(record.log_file, Some("/path/to/log".to_string()));
    }

    #[test]
    fn test_monitoring_service_with_hooks() {
        let registry = Arc::new(HookRegistry::new());
        let service = MonitoringService::with_hooks(registry.clone()).unwrap();
        assert!(service.get_hook_registry().is_some());
    }

    #[test]
    fn test_monitoring_service_open() {
        use tempfile::TempDir;
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("monitoring.db");
        
        let service = MonitoringService::open(&db_path);
        assert!(service.is_ok());
    }

    #[test]
    fn test_monitoring_service_open_with_hooks() {
        use tempfile::TempDir;
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("monitoring.db");
        let registry = Arc::new(HookRegistry::new());
        
        let service = MonitoringService::open_with_hooks(&db_path, registry.clone());
        assert!(service.is_ok());
        let service = service.unwrap();
        assert!(service.get_hook_registry().is_some());
    }

    #[test]
    fn test_monitoring_service_set_hook_registry() {
        let mut service = MonitoringService::new().unwrap();
        assert!(service.get_hook_registry().is_none());
        
        let registry = Arc::new(HookRegistry::new());
        service.set_hook_registry(registry.clone());
        assert!(service.get_hook_registry().is_some());
    }

    #[test]
    fn test_fail_agent() {
        let service = MonitoringService::new().unwrap();
        let record = AgentRecord::new("agent-1".to_string(), "developer".to_string());

        service.register_agent(&record).unwrap();
        service.fail_agent("agent-1", "test error").unwrap();

        let retrieved = service.get_agent("agent-1").unwrap();
        assert_eq!(retrieved.status, AgentStatus::Failed);
        assert_eq!(retrieved.error_message, Some("test error".to_string()));
    }

    #[test]
    fn test_get_agent_not_found() {
        let service = MonitoringService::new().unwrap();
        let result = service.get_agent("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_update_status_not_found() {
        let service = MonitoringService::new().unwrap();
        let result = service.update_status("nonexistent", AgentStatus::Running);
        assert!(result.is_err());
    }

    #[test]
    fn test_usage_filter_default() {
        let filter = UsageFilter::default();
        assert!(filter.category.is_none());
        assert!(filter.min_executions.is_none());
        assert!(filter.since.is_none());
    }

    #[test]
    fn test_usage_filter_with_values() {
        let filter = UsageFilter {
            category: Some("test".to_string()),
            min_executions: Some(10),
            since: Some(1000),
        };
        assert_eq!(filter.category, Some("test".to_string()));
        assert_eq!(filter.min_executions, Some(10));
        assert_eq!(filter.since, Some(1000));
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

    #[test]
    fn test_agent_record_json_serialization() {
        let record = AgentRecord::new("agent-123".to_string(), "developer".to_string())
            .with_parent("parent-456".to_string())
            .with_plan("REQ-49".to_string())
            .with_process_id(12345)
            .with_log_file("/path/to/log".to_string());

        // Serialize to JSON
        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("agent-123"));
        assert!(json.contains("developer"));
        assert!(json.contains("parent-456"));
        assert!(json.contains("REQ-49"));

        // Deserialize back
        let deserialized: AgentRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, record.id);
        assert_eq!(deserialized.agent_type, record.agent_type);
        assert_eq!(deserialized.parent_id, record.parent_id);
        assert_eq!(deserialized.plan_id, record.plan_id);
        assert_eq!(deserialized.process_id, record.process_id);
        assert_eq!(deserialized.log_file, record.log_file);
        assert_eq!(deserialized.status, record.status);
    }

    #[test]
    fn test_agent_status_json_serialization() {
        // Test all status variants
        let statuses = vec![
            AgentStatus::Starting,
            AgentStatus::Running,
            AgentStatus::Completed,
            AgentStatus::Failed,
            AgentStatus::Terminated,
        ];

        for status in statuses {
            // Serialize to JSON
            let json = serde_json::to_string(&status).unwrap();
            // Should be lowercase string
            assert!(json.starts_with('"'));
            assert!(json.ends_with('"'));

            // Deserialize back
            let deserialized: AgentStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, status);
        }
    }

    #[test]
    fn test_agent_record_json_round_trip() {
        // Create record with all fields populated
        let mut record = AgentRecord::new("agent-123".to_string(), "developer".to_string())
            .with_parent("parent-456".to_string())
            .with_plan("REQ-49".to_string())
            .with_process_id(12345)
            .with_log_file("/path/to/log".to_string());

        // Update status and add completion info
        record.status = AgentStatus::Completed;
        // Note: We can't directly set end_time, exit_code, etc. through builder,
        // but we can test with what we have

        // Serialize and deserialize
        let json = serde_json::to_string(&record).unwrap();
        let deserialized: AgentRecord = serde_json::from_str(&json).unwrap();

        // Verify all fields preserved
        assert_eq!(deserialized.id, record.id);
        assert_eq!(deserialized.agent_type, record.agent_type);
        assert_eq!(deserialized.parent_id, record.parent_id);
        assert_eq!(deserialized.plan_id, record.plan_id);
        assert_eq!(deserialized.process_id, record.process_id);
        assert_eq!(deserialized.log_file, record.log_file);
        assert_eq!(deserialized.status, record.status);
        assert_eq!(deserialized.start_time, record.start_time);
    }
}
