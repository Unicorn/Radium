//! Agent orchestrator for Radium.
//!
//! This module defines the core agent trait and orchestration structures.

pub mod agents;
pub mod error;
pub mod executor;
pub mod lifecycle;
pub mod orchestration;
pub mod plugin;
pub mod queue;
pub mod registry;

use async_trait::async_trait;
use radium_abstraction::{Model, ModelError};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, warn};

pub use agents::{ChatAgent, SimpleAgent};
pub use executor::{
    AgentExecutor, ExecutionResult, ExecutionTelemetry, QueueProcessor, QueueProcessorConfig,
};
pub use lifecycle::{AgentLifecycle, AgentState};
pub use orchestration::{
    agent_tools::{AgentMetadata as OrchestrationAgentMetadata, AgentToolRegistry},
    context::{Message, OrchestrationContext, UserPreferences},
    tool::{Tool, ToolArguments, ToolCall, ToolHandler, ToolParameters, ToolResult},
    FinishReason, OrchestrationProvider, OrchestrationResult,
};
pub use plugin::{InMemoryPlugin, Plugin, PluginLoader, PluginMetadata};
pub use queue::{ExecutionQueue, ExecutionTask, Priority, QueueMetrics};
pub use registry::{AgentMetadata, AgentRegistry};

// Re-export orchestration error separately to avoid conflicts
pub use error::OrchestrationError;

/// Represents the context provided to an agent during its execution.
#[derive(Clone, Copy)]
pub struct AgentContext<'a> {
    /// The model to use for generation.
    pub model: &'a (dyn Model + Send + Sync),
    // Future additions: access to tools, memory, other agents, etc.
}

/// Represents the output produced by an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentOutput {
    /// The agent produced a text response.
    Text(String),
    /// The agent produced a structured data response (e.g., JSON).
    StructuredData(serde_json::Value),
    /// The agent decided to call a tool.
    ToolCall {
        /// Name of the tool to call.
        name: String,
        /// Arguments to pass to the tool.
        args: serde_json::Value,
    },
    /// The agent decided to terminate.
    Terminate,
    // Future additions: file output, command execution, etc.
}

/// A trait that defines the interface for any autonomous agent.
#[async_trait]
pub trait Agent {
    /// Returns the unique ID of the agent.
    fn id(&self) -> &str;

    /// Returns a description of the agent's purpose and capabilities.
    fn description(&self) -> &str;

    /// Executes the agent with the given input and context.
    ///
    /// # Arguments
    /// * `input` - The input to process
    /// * `context` - The execution context including the model
    ///
    /// # Errors
    /// Returns a `ModelError` if execution fails.
    async fn execute(
        &self,
        input: &str,
        context: AgentContext<'_>,
    ) -> std::result::Result<AgentOutput, ModelError>;
}

/// Orchestrator for managing agents and their execution.
#[derive(Debug)]
pub struct Orchestrator {
    /// Registry of registered agents.
    registry: Arc<AgentRegistry>,
    /// Lifecycle manager for agent states.
    lifecycle: Arc<AgentLifecycle>,
    /// Execution queue for agent tasks.
    queue: Arc<ExecutionQueue>,
    /// Agent executor for running agents.
    executor: Arc<AgentExecutor>,
    /// Queue processor for background task execution.
    processor: QueueProcessor,
}

impl Orchestrator {
    /// Creates a new Orchestrator instance.
    #[must_use]
    pub fn new() -> Self {
        let registry = Arc::new(AgentRegistry::new());
        let lifecycle = Arc::new(AgentLifecycle::new());
        let queue = Arc::new(ExecutionQueue::new());
        let executor = Arc::new(AgentExecutor::with_mock_model());
        let processor = QueueProcessor::new(
            QueueProcessorConfig::default(),
            Arc::clone(&registry),
            Arc::clone(&lifecycle),
            Arc::clone(&queue),
            Arc::clone(&executor),
        );

        Self { registry, lifecycle, queue, executor, processor }
    }

    /// Starts the queue processor to begin processing queued tasks.
    ///
    /// # Returns
    /// Returns `Ok(())` if started successfully, or an error if already running.
    pub fn start_queue_processor(&mut self) -> std::result::Result<(), String> {
        self.processor.start()
    }

    /// Stops the queue processor gracefully.
    ///
    /// # Returns
    /// Returns `Ok(())` if stopped successfully, or an error if not running.
    pub fn stop_queue_processor(&mut self) -> std::result::Result<(), String> {
        self.processor.stop()
    }

    /// Checks if the queue processor is currently running.
    ///
    /// # Returns
    /// Returns `true` if running, `false` otherwise.
    #[must_use]
    pub fn is_queue_processor_running(&self) -> bool {
        self.processor.is_running()
    }

    /// Registers an agent in the orchestrator.
    ///
    /// # Arguments
    /// * `agent` - The agent to register (wrapped in Arc for thread-safe sharing)
    ///
    /// # Returns
    /// Returns `true` if the agent was newly registered, `false` if it replaced an existing agent.
    pub async fn register_agent(&self, agent: std::sync::Arc<dyn Agent + Send + Sync>) -> bool {
        self.registry.register_agent(agent).await
    }

    /// Retrieves an agent by ID.
    ///
    /// # Arguments
    /// * `id` - The agent ID to look up
    ///
    /// # Returns
    /// Returns `Some(Arc<dyn Agent>)` if found, `None` otherwise.
    pub async fn get_agent(&self, id: &str) -> Option<std::sync::Arc<dyn Agent + Send + Sync>> {
        self.registry.get_agent(id).await
    }

    /// Lists all registered agents with their metadata.
    ///
    /// # Returns
    /// Returns a vector of agent metadata.
    pub async fn list_agents(&self) -> Vec<AgentMetadata> {
        self.registry.list_agents().await
    }

    /// Unregisters an agent from the orchestrator.
    ///
    /// # Arguments
    /// * `id` - The agent ID to unregister
    ///
    /// # Returns
    /// Returns `true` if the agent was found and removed, `false` otherwise.
    pub async fn unregister_agent(&self, id: &str) -> bool {
        self.registry.unregister_agent(id).await
    }

    /// Checks if an agent is registered.
    ///
    /// # Arguments
    /// * `id` - The agent ID to check
    ///
    /// # Returns
    /// Returns `true` if the agent is registered, `false` otherwise.
    pub async fn is_registered(&self, id: &str) -> bool {
        self.registry.is_registered(id).await
    }

    /// Returns the number of registered agents.
    ///
    /// # Returns
    /// The count of registered agents.
    pub async fn agent_count(&self) -> usize {
        self.registry.count().await
    }

    /// Starts an agent (transitions to Running state).
    ///
    /// # Arguments
    /// * `id` - The agent ID
    ///
    /// # Returns
    /// Returns `Ok(())` if successful, `Err(AgentState)` with current state if transition is invalid.
    pub async fn start_agent(&self, id: &str) -> std::result::Result<(), lifecycle::AgentState> {
        if !self.registry.is_registered(id).await {
            return Err(lifecycle::AgentState::Idle);
        }
        self.lifecycle.start_agent(id).await
    }

    /// Stops an agent (transitions to Stopped state).
    ///
    /// # Arguments
    /// * `id` - The agent ID
    ///
    /// # Returns
    /// Returns `Ok(())` if successful, `Err(AgentState)` with current state if transition is invalid.
    pub async fn stop_agent(&self, id: &str) -> std::result::Result<(), lifecycle::AgentState> {
        self.lifecycle.stop_agent(id).await
    }

    /// Pauses an agent (transitions from Running to Paused).
    ///
    /// # Arguments
    /// * `id` - The agent ID
    ///
    /// # Returns
    /// Returns `Ok(())` if successful, `Err(AgentState)` with current state if transition is invalid.
    pub async fn pause_agent(&self, id: &str) -> std::result::Result<(), lifecycle::AgentState> {
        self.lifecycle.pause_agent(id).await
    }

    /// Resumes an agent (transitions from Paused to Running).
    ///
    /// # Arguments
    /// * `id` - The agent ID
    ///
    /// # Returns
    /// Returns `Ok(())` if successful, `Err(AgentState)` with current state if transition is invalid.
    pub async fn resume_agent(&self, id: &str) -> std::result::Result<(), lifecycle::AgentState> {
        self.lifecycle.resume_agent(id).await
    }

    /// Gets the current state of an agent.
    ///
    /// # Arguments
    /// * `id` - The agent ID
    ///
    /// # Returns
    /// Returns the current agent state.
    pub async fn get_agent_state(&self, id: &str) -> lifecycle::AgentState {
        self.lifecycle.get_state(id).await
    }

    /// Enqueues a task for execution.
    ///
    /// # Arguments
    /// * `task` - The task to enqueue
    ///
    /// # Returns
    /// Returns `Ok(())` if successful, `Err` if the queue is closed.
    pub async fn enqueue_task(
        &self,
        task: ExecutionTask,
    ) -> std::result::Result<(), mpsc::error::SendError<ExecutionTask>> {
        self.queue.enqueue_task(task).await
    }

    /// Cancels a task by ID.
    ///
    /// # Arguments
    /// * `task_id` - The task ID to cancel
    ///
    /// # Returns
    /// Returns `true` if the task was found and cancelled, `false` otherwise.
    pub async fn cancel_task(&self, task_id: &str) -> bool {
        self.queue.cancel_task(task_id).await
    }

    /// Returns queue metrics.
    ///
    /// # Returns
    /// A struct containing all queue metrics.
    pub async fn queue_metrics(&self) -> QueueMetrics {
        self.queue.metrics().await
    }

    /// Executes an agent with the given input.
    ///
    /// # Arguments
    /// * `agent_id` - The ID of the agent to execute
    /// * `input` - The input for the agent
    ///
    /// # Returns
    /// Returns `Ok(ExecutionResult)` if the agent was found and executed, `Err` otherwise.
    pub async fn execute_agent(
        &self,
        agent_id: &str,
        input: &str,
    ) -> std::result::Result<ExecutionResult, ModelError> {
        // Check if agent is registered
        let agent = self.get_agent(agent_id).await.ok_or_else(|| {
            ModelError::UnsupportedModelProvider(format!("Agent not found: {}", agent_id))
        })?;

        // Check agent state
        let state = self.get_agent_state(agent_id).await;
        if state != AgentState::Idle && state != AgentState::Running {
            return Err(ModelError::UnsupportedModelProvider(format!(
                "Agent {} is in {:?} state and cannot be executed",
                agent_id, state
            )));
        }

        // Start agent if idle
        if state == AgentState::Idle {
            if let Err(current_state) = self.start_agent(agent_id).await {
                return Err(ModelError::UnsupportedModelProvider(format!(
                    "Failed to start agent {}: invalid state transition from {:?}",
                    agent_id, current_state
                )));
            }
        }

        // Execute the agent
        let result = self.executor.execute_agent_with_default_model(agent, input).await?;

        // Mark agent as idle if execution completed
        if result.success {
            let _ = self.lifecycle.set_state(agent_id, AgentState::Idle).await;
        } else {
            let _ = self.lifecycle.mark_error(agent_id).await;
        }

        Ok(result)
    }

    /// Executes an agent with a custom model.
    ///
    /// # Arguments
    /// * `agent_id` - The ID of the agent to execute
    /// * `input` - The input for the agent
    /// * `model_type` - The type of model to use
    /// * `model_id` - The model ID to use
    ///
    /// # Returns
    /// Returns `Ok(ExecutionResult)` if the agent was found and executed, `Err` otherwise.
    pub async fn execute_agent_with_model(
        &self,
        agent_id: &str,
        input: &str,
        model_type: radium_models::ModelType,
        model_id: String,
    ) -> std::result::Result<ExecutionResult, ModelError> {
        // Check if agent is registered
        let agent = self.get_agent(agent_id).await.ok_or_else(|| {
            ModelError::UnsupportedModelProvider(format!("Agent not found: {}", agent_id))
        })?;

        // Check agent state
        let state = self.get_agent_state(agent_id).await;
        if state != AgentState::Idle && state != AgentState::Running {
            return Err(ModelError::UnsupportedModelProvider(format!(
                "Agent {} is in {:?} state and cannot be executed",
                agent_id, state
            )));
        }

        // Start agent if idle
        if state == AgentState::Idle {
            if let Err(current_state) = self.start_agent(agent_id).await {
                return Err(ModelError::UnsupportedModelProvider(format!(
                    "Failed to start agent {}: invalid state transition from {:?}",
                    agent_id, current_state
                )));
            }
        }

        // Execute the agent
        let result =
            self.executor.execute_agent_with_model(agent, input, model_type, model_id).await?;

        // Mark agent as idle if execution completed
        if result.success {
            let _ = self.lifecycle.set_state(agent_id, AgentState::Idle).await;
        } else {
            let _ = self.lifecycle.mark_error(agent_id).await;
        }

        Ok(result)
    }

    /// Loads agents from a plugin.
    ///
    /// # Arguments
    /// * `plugin` - The plugin to load agents from
    ///
    /// # Returns
    /// Returns the number of agents successfully registered.
    pub async fn load_plugin(&self, plugin: Box<dyn Plugin>) -> usize {
        let metadata = plugin.metadata();
        debug!(
            plugin_name = %metadata.name,
            plugin_version = %metadata.version,
            agent_count = metadata.agent_ids.len(),
            "Loading plugin"
        );

        let mut registered_count = 0;
        for agent_id in metadata.agent_ids {
            match plugin.create_agent(&agent_id) {
                Some(agent) => {
                    if self.register_agent(agent).await {
                        registered_count += 1;
                        debug!(plugin_name = %metadata.name, agent_id = %agent_id, "Registered agent from plugin");
                    } else {
                        warn!(plugin_name = %metadata.name, agent_id = %agent_id, "Agent already registered, replaced");
                    }
                }
                _ => {
                    error!(plugin_name = %metadata.name, agent_id = %agent_id, "Plugin failed to create agent");
                }
            }
        }

        debug!(
            plugin_name = %metadata.name,
            registered = registered_count,
            "Plugin loading completed"
        );

        registered_count
    }
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self::new()
    }
}

/// A simple agent that echoes its input as output.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EchoAgent {
    id: String,
    description: String,
}

impl EchoAgent {
    /// Creates a new `EchoAgent` with the given ID and description.
    #[must_use]
    pub const fn new(id: String, description: String) -> Self {
        Self { id, description }
    }
}

#[async_trait]
impl Agent for EchoAgent {
    fn id(&self) -> &str {
        &self.id
    }

    fn description(&self) -> &str {
        &self.description
    }

    async fn execute(
        &self,
        input: &str,
        _context: AgentContext<'_>,
    ) -> std::result::Result<AgentOutput, ModelError> {
        debug!(agent_id = %self.id, input = %input, "EchoAgent executing");
        Ok(AgentOutput::Text(format!("Echo from {}: {input}", self.id)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_orchestrator_new() {
        let orchestrator = Orchestrator::new();
        assert_eq!(orchestrator.agent_count().await, 0);
        assert!(!orchestrator.is_queue_processor_running());
    }

    #[tokio::test]
    async fn test_orchestrator_default() {
        let orchestrator = Orchestrator::default();
        assert_eq!(orchestrator.agent_count().await, 0);
    }

    #[tokio::test]
    async fn test_orchestrator_register_agent() {
        let orchestrator = Orchestrator::new();
        let agent = Arc::new(EchoAgent::new("test-agent".to_string(), "Test agent".to_string()));
        let registered = orchestrator.register_agent(agent).await;
        assert!(registered);
        assert_eq!(orchestrator.agent_count().await, 1);
    }

    #[tokio::test]
    async fn test_orchestrator_register_duplicate_agent() {
        let orchestrator = Orchestrator::new();
        let agent1 = Arc::new(EchoAgent::new("test-agent".to_string(), "Test agent".to_string()));
        let agent2 =
            Arc::new(EchoAgent::new("test-agent".to_string(), "Different agent".to_string()));

        let registered1 = orchestrator.register_agent(agent1).await;
        assert!(registered1);

        let registered2 = orchestrator.register_agent(agent2).await;
        assert!(!registered2); // Should return false for duplicate
        assert_eq!(orchestrator.agent_count().await, 1);
    }

    #[tokio::test]
    async fn test_orchestrator_get_agent() {
        let orchestrator = Orchestrator::new();
        let agent = Arc::new(EchoAgent::new("test-agent".to_string(), "Test agent".to_string()));
        orchestrator.register_agent(agent).await;

        let retrieved = orchestrator.get_agent("test-agent").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id(), "test-agent");
    }

    #[tokio::test]
    async fn test_orchestrator_get_nonexistent_agent() {
        let orchestrator = Orchestrator::new();
        let retrieved = orchestrator.get_agent("nonexistent").await;
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_orchestrator_list_agents() {
        let orchestrator = Orchestrator::new();
        let agent1 = Arc::new(EchoAgent::new("agent-1".to_string(), "Agent 1".to_string()));
        let agent2 = Arc::new(SimpleAgent::new("agent-2".to_string(), "Agent 2".to_string()));

        orchestrator.register_agent(agent1).await;
        orchestrator.register_agent(agent2).await;

        let agents = orchestrator.list_agents().await;
        assert_eq!(agents.len(), 2);
        assert!(agents.iter().any(|a| a.id == "agent-1"));
        assert!(agents.iter().any(|a| a.id == "agent-2"));
    }

    #[tokio::test]
    async fn test_orchestrator_unregister_agent() {
        let orchestrator = Orchestrator::new();
        let agent = Arc::new(EchoAgent::new("test-agent".to_string(), "Test agent".to_string()));
        orchestrator.register_agent(agent).await;
        assert_eq!(orchestrator.agent_count().await, 1);

        let unregistered = orchestrator.unregister_agent("test-agent").await;
        assert!(unregistered);
        assert_eq!(orchestrator.agent_count().await, 0);
    }

    #[tokio::test]
    async fn test_orchestrator_unregister_nonexistent_agent() {
        let orchestrator = Orchestrator::new();
        let unregistered = orchestrator.unregister_agent("nonexistent").await;
        assert!(!unregistered);
    }

    #[tokio::test]
    async fn test_orchestrator_is_registered() {
        let orchestrator = Orchestrator::new();
        let agent = Arc::new(EchoAgent::new("test-agent".to_string(), "Test agent".to_string()));
        orchestrator.register_agent(agent).await;

        assert!(orchestrator.is_registered("test-agent").await);
        assert!(!orchestrator.is_registered("nonexistent").await);
    }

    #[tokio::test]
    async fn test_orchestrator_agent_count() {
        let orchestrator = Orchestrator::new();
        assert_eq!(orchestrator.agent_count().await, 0);

        for i in 0..5 {
            let agent = Arc::new(EchoAgent::new(format!("agent-{}", i), format!("Agent {}", i)));
            orchestrator.register_agent(agent).await;
        }

        assert_eq!(orchestrator.agent_count().await, 5);
    }

    #[tokio::test]
    async fn test_orchestrator_start_agent() {
        let orchestrator = Orchestrator::new();
        let agent = Arc::new(EchoAgent::new("test-agent".to_string(), "Test agent".to_string()));
        orchestrator.register_agent(agent).await;

        let result = orchestrator.start_agent("test-agent").await;
        assert!(result.is_ok());
        assert_eq!(orchestrator.get_agent_state("test-agent").await, AgentState::Running);
    }

    #[tokio::test]
    async fn test_orchestrator_start_nonexistent_agent() {
        let orchestrator = Orchestrator::new();
        let result = orchestrator.start_agent("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_orchestrator_stop_agent() {
        let orchestrator = Orchestrator::new();
        let agent = Arc::new(EchoAgent::new("test-agent".to_string(), "Test agent".to_string()));
        orchestrator.register_agent(agent).await;
        orchestrator.start_agent("test-agent").await.unwrap();

        let result = orchestrator.stop_agent("test-agent").await;
        assert!(result.is_ok());
        assert_eq!(orchestrator.get_agent_state("test-agent").await, AgentState::Stopped);
    }

    #[tokio::test]
    async fn test_orchestrator_pause_and_resume_agent() {
        let orchestrator = Orchestrator::new();
        let agent = Arc::new(EchoAgent::new("test-agent".to_string(), "Test agent".to_string()));
        orchestrator.register_agent(agent).await;
        orchestrator.start_agent("test-agent").await.unwrap();

        let result = orchestrator.pause_agent("test-agent").await;
        assert!(result.is_ok());
        assert_eq!(orchestrator.get_agent_state("test-agent").await, AgentState::Paused);

        let result = orchestrator.resume_agent("test-agent").await;
        assert!(result.is_ok());
        assert_eq!(orchestrator.get_agent_state("test-agent").await, AgentState::Running);
    }

    #[tokio::test]
    async fn test_orchestrator_execute_agent() {
        let orchestrator = Orchestrator::new();
        let agent = Arc::new(EchoAgent::new("test-agent".to_string(), "Test agent".to_string()));
        orchestrator.register_agent(agent).await;

        let result = orchestrator.execute_agent("test-agent", "test input").await;
        assert!(result.is_ok());
        let execution_result = result.unwrap();
        assert!(execution_result.success);
    }

    #[tokio::test]
    async fn test_orchestrator_execute_agent_with_model() {
        let orchestrator = Orchestrator::new();
        let agent = Arc::new(SimpleAgent::new("test-agent".to_string(), "Test agent".to_string()));
        orchestrator.register_agent(agent).await;

        let result = orchestrator
            .execute_agent_with_model(
                "test-agent",
                "test input",
                radium_models::ModelType::Mock,
                "mock-model".to_string(),
            )
            .await;
        assert!(result.is_ok());
        let execution_result = result.unwrap();
        assert!(execution_result.success);
    }

    #[tokio::test]
    async fn test_orchestrator_execute_nonexistent_agent() {
        let orchestrator = Orchestrator::new();
        let result = orchestrator.execute_agent("nonexistent", "test input").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_orchestrator_queue_processor_lifecycle() {
        let mut orchestrator = Orchestrator::new();

        // Start processor
        let result = orchestrator.start_queue_processor();
        assert!(result.is_ok());
        assert!(orchestrator.is_queue_processor_running());

        // Stop processor
        let result = orchestrator.stop_queue_processor();
        assert!(result.is_ok());
        assert!(!orchestrator.is_queue_processor_running());
    }

    #[tokio::test]
    async fn test_orchestrator_queue_metrics() {
        let orchestrator = Orchestrator::new();
        let metrics = orchestrator.queue_metrics().await;
        assert_eq!(metrics.pending, 0);
        assert_eq!(metrics.completed, 0);
        assert_eq!(metrics.running, 0);
    }
}
