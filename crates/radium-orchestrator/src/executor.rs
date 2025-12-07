//! Agent execution engine.
//!
//! This module provides functionality for executing agents with proper context and error handling.

#[cfg(test)]
use crate::ExecutionTask;
use crate::{
    Agent, AgentContext, AgentLifecycle, AgentOutput, AgentRegistry, AgentState, ExecutionQueue,
};
use radium_abstraction::ModelError;
use radium_models::{ModelFactory, ModelType};
use radium_core::auth::{CredentialStore, ProviderType};
use serde_json::Value;
use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Semaphore, mpsc};
use tokio::time;
use tracing::{debug, error, info, warn};

/// Result of hook execution.
#[derive(Debug, Clone)]
pub struct HookResult {
    /// Whether execution should continue.
    pub should_continue: bool,
    /// Optional message from the hook.
    pub message: Option<String>,
    /// Optional modified data from the hook.
    pub modified_data: Option<Value>,
}

/// Trait for hook execution (to avoid circular dependency with radium-core).
#[async_trait::async_trait]
pub trait HookExecutor: Send + Sync {
    /// Execute before model call hooks.
    async fn execute_before_model(&self, agent_id: &str, input: &str) -> Result<Vec<HookResult>, String>;
    
    /// Execute after model call hooks.
    async fn execute_after_model(&self, agent_id: &str, output: &AgentOutput, success: bool) -> Result<Vec<HookResult>, String>;
    
    /// Execute error interception hooks.
    async fn execute_error_interception(&self, agent_id: &str, error_message: &str, error_type: &str, error_source: Option<&str>) -> Result<Option<String>, String>;
    
    /// Execute error transformation hooks.
    async fn execute_error_transformation(&self, agent_id: &str, error_message: &str, error_type: &str, error_source: Option<&str>) -> Result<Option<String>, String>;
    
    /// Execute error recovery hooks.
    async fn execute_error_recovery(&self, agent_id: &str, error_message: &str, error_type: &str, error_source: Option<&str>) -> Result<Option<String>, String>;
}

/// Telemetry information from model execution.
#[derive(Debug, Clone)]
pub struct ExecutionTelemetry {
    /// Input/prompt tokens.
    pub input_tokens: u64,
    /// Output/completion tokens.
    pub output_tokens: u64,
    /// Total tokens.
    pub total_tokens: u64,
    /// Model ID used.
    pub model_id: Option<String>,
}

impl ExecutionTelemetry {
    /// Creates telemetry from ModelUsage.
    pub fn from_usage(usage: &radium_abstraction::ModelUsage, model_id: Option<String>) -> Self {
        Self {
            input_tokens: u64::from(usage.prompt_tokens),
            output_tokens: u64::from(usage.completion_tokens),
            total_tokens: u64::from(usage.total_tokens),
            model_id,
        }
    }
}

/// Execution result for an agent.
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// The output produced by the agent.
    pub output: AgentOutput,
    /// Whether the execution was successful.
    pub success: bool,
    /// Optional error message if execution failed.
    pub error: Option<String>,
    /// Optional telemetry information from model execution.
    pub telemetry: Option<ExecutionTelemetry>,
}

/// Executor for running agents.
pub struct AgentExecutor {
    /// Default model type to use if not specified.
    default_model_type: ModelType,
    /// Default model ID to use if not specified.
    default_model_id: String,
}

impl fmt::Debug for AgentExecutor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AgentExecutor")
            .field("default_model_type", &self.default_model_type)
            .field("default_model_id", &self.default_model_id)
            .finish()
    }
}

impl AgentExecutor {
    /// Creates a new agent executor with default model configuration.
    ///
    /// # Arguments
    /// * `default_model_type` - Default model type to use
    /// * `default_model_id` - Default model ID to use
    #[must_use]
    pub fn new(default_model_type: ModelType, default_model_id: String) -> Self {
        Self { default_model_type, default_model_id }
    }

    /// Creates a new agent executor with Mock model as default.
    #[must_use]
    pub fn with_mock_model() -> Self {
        Self::new(ModelType::Mock, "mock-model".to_string())
    }

    /// Infers the provider name from a model instance.
    ///
    /// This is a best-effort inference based on the model ID.
    /// For more accurate tracking, the provider should be passed explicitly.
    fn infer_provider_from_model(model: &Arc<dyn radium_abstraction::Model + Send + Sync>) -> String {
        let model_id = model.model_id().to_lowercase();
        if model_id.contains("gpt") || model_id.contains("openai") {
            "openai".to_string()
        } else if model_id.contains("gemini") {
            "gemini".to_string()
        } else if model_id.contains("mock") {
            "mock".to_string()
        } else {
            // Default to openai if we can't determine
            "openai".to_string()
        }
    }

    /// Finds the next available provider for failover.
    ///
    /// Returns the next provider in the failover order (openai → gemini → mock)
    /// that is not in the exhausted set and has available credentials.
    ///
    /// # Arguments
    /// * `exhausted_providers` - Set of provider names that have been exhausted
    /// * `current_provider` - The current provider name (to determine failover order)
    ///
    /// # Returns
    /// Returns `Some((provider_name, model_type, model_id))` if a backup provider is found,
    /// or `None` if all providers are exhausted.
    fn find_next_available_provider(
        exhausted_providers: &HashSet<String>,
        current_provider: &str,
    ) -> Option<(String, ModelType, String)> {
        // Failover order: openai → gemini → mock
        let failover_order = vec![
            ("openai", ModelType::OpenAI, "gpt-3.5-turbo".to_string()),
            ("gemini", ModelType::Gemini, "gemini-pro".to_string()),
            ("mock", ModelType::Mock, "mock-model".to_string()),
        ];

        // Find current provider index to start from next in order
        let current_idx = failover_order
            .iter()
            .position(|(name, _, _)| *name == current_provider)
            .unwrap_or(0);

        // Try providers in failover order starting from next after current
        let credential_store = CredentialStore::new().ok()?;
        
        for i in 0..failover_order.len() {
            let idx = (current_idx + 1 + i) % failover_order.len();
            let (provider_name, model_type, default_model_id) = &failover_order[idx];
            
            // Skip if already exhausted
            if exhausted_providers.contains(*provider_name) {
                continue;
            }

            // Check if provider is available
            // Mock is always available (no credentials needed)
            if *provider_name == "mock" {
                return Some((provider_name.to_string(), model_type.clone(), default_model_id.clone()));
            }

            // For real providers, check credentials
            let provider_type = match *provider_name {
                "openai" => ProviderType::OpenAI,
                "gemini" => ProviderType::Gemini,
                _ => continue,
            };

            if credential_store.is_configured(provider_type) {
                return Some((provider_name.to_string(), model_type.clone(), default_model_id.clone()));
            }
        }

        None
    }

    /// Executes an agent with the given input and model.
    ///
    /// # Arguments
    /// * `agent` - The agent to execute
    /// * `input` - The input for the agent
    /// * `model` - The model to use for execution
    /// * `hook_registry` - Optional hook registry for execution interception
    ///
    /// # Returns
    /// Returns `ExecutionResult` with the agent's output or error information.
    pub async fn execute_agent(
        &self,
        agent: Arc<dyn Agent + Send + Sync>,
        input: &str,
        model: Arc<dyn radium_abstraction::Model + Send + Sync>,
        hook_executor: Option<&Arc<dyn HookExecutor>>,
    ) -> ExecutionResult {
        let agent_id = agent.id();
        debug!(agent_id = %agent_id, input_len = input.len(), "Executing agent");

        // Execute BeforeModel hooks
        let mut effective_input = input.to_string();
        if let Some(executor) = hook_executor {
            match executor.execute_before_model(agent_id, input).await {
                Ok(results) => {
                    for result in results {
                        // If hook says to stop, abort execution
                        if !result.should_continue {
                            let message = result.message.unwrap_or_else(|| "Execution aborted by hook".to_string());
                            warn!(agent_id = %agent_id, message = %message, "Execution aborted by hook");
                            return ExecutionResult {
                                output: AgentOutput::Text(format!("Execution aborted: {}", message)),
                                success: false,
                                error: Some(message),
                                telemetry: None,
                            };
                        }

                        // If hook modifies input, use the modified version
                        if let Some(modified_data) = result.modified_data {
                            if let Some(new_input) = modified_data.get("input").and_then(|v| v.as_str()) {
                                effective_input = new_input.to_string();
                                debug!(agent_id = %agent_id, "Input modified by hook");
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!(agent_id = %agent_id, error = %e, "Hook execution failed, continuing");
                }
            }
        }

        // Track exhausted providers for failover
        let mut exhausted_providers = HashSet::new();
        let mut current_model = model;
        let mut current_provider = Self::infer_provider_from_model(&current_model);

        // Retry loop with provider failover
        loop {
            // Create agent context with current model
            let context = AgentContext { model: current_model.as_ref() };

            // Execute the agent
            // Note: Telemetry capture requires modifying agents to return ModelResponse
            // For now, telemetry is None - will be captured when agents are updated
            let execution_result = match agent.execute(&effective_input, context).await {
                Ok(output) => {
                    info!(agent_id = %agent_id, output_type = ?output, provider = %current_provider, "Agent execution completed successfully");
                    ExecutionResult {
                        output: output.clone(),
                        success: true,
                        error: None,
                        telemetry: None, // Will be populated when agents capture ModelResponse
                    }
                }
                Err(e) => {
                    // Check if this is a QuotaExceeded error that should trigger failover
                    if let ModelError::QuotaExceeded { provider, .. } = &e {
                        warn!(
                            agent_id = %agent_id,
                            provider = %provider,
                            "Provider reported insufficient quota"
                        );
                        
                        // Mark provider as exhausted
                        exhausted_providers.insert(provider.clone());
                        current_provider = provider.clone();

                        // Find next available provider
                        if let Some((next_provider, next_model_type, next_model_id)) =
                            Self::find_next_available_provider(&exhausted_providers, &current_provider)
                        {
                            info!(
                                agent_id = %agent_id,
                                from_provider = %current_provider,
                                to_provider = %next_provider,
                                "Failing over to backup provider"
                            );

                            // Create new model for backup provider
                            match ModelFactory::create_from_str(
                                match next_model_type {
                                    ModelType::Mock => "mock",
                                    ModelType::Gemini => "gemini",
                                    ModelType::OpenAI => "openai",
                                },
                                next_model_id,
                            ) {
                                Ok(new_model) => {
                                    current_model = new_model;
                                    current_provider = next_provider;
                                    // Continue loop to retry with new provider
                                    continue;
                                }
                                Err(create_err) => {
                                    error!(
                                        agent_id = %agent_id,
                                        provider = %next_provider,
                                        error = %create_err,
                                        "Failed to create backup provider model"
                                    );
                                    // Fall through to error handling
                                }
                            }
                        } else {
                            // All providers exhausted
                            error!(
                                agent_id = %agent_id,
                                "All configured providers are exhausted"
                            );
                            return ExecutionResult {
                                output: AgentOutput::Text(
                                    "All configured providers are exhausted. Execution paused.".to_string()
                                ),
                                success: false,
                                error: Some("All configured providers are exhausted. Execution paused.".to_string()),
                                telemetry: None,
                            };
                        }
                    }

                    // For non-quota errors or if failover failed, handle normally
                    error!(agent_id = %agent_id, error = %e, "Agent execution failed");
                    
                    // Execute error hooks if available
                    let mut effective_error = e.to_string();
                    let mut error_handled = false;
                    
                    if let Some(executor) = hook_executor {
                        // Try error interception first
                        if let Ok(Some(handled_message)) = executor.execute_error_interception(
                            agent_id,
                            &effective_error,
                            "agent_execution_error",
                            Some("agent_executor"),
                        ).await {
                            effective_error = handled_message;
                            error_handled = true;
                        }
                        
                        // If not handled, try error transformation
                        if !error_handled {
                            if let Ok(Some(transformed_message)) = executor.execute_error_transformation(
                                agent_id,
                                &effective_error,
                                "agent_execution_error",
                                Some("agent_executor"),
                            ).await {
                                effective_error = transformed_message;
                            }
                        }
                        
                        // Try error recovery
                        if let Ok(Some(recovered_message)) = executor.execute_error_recovery(
                            agent_id,
                            &effective_error,
                            "agent_execution_error",
                            Some("agent_executor"),
                        ).await {
                            // If recovery succeeded, we might want to retry or return a different result
                            // For now, we'll use the recovered message as the error
                            effective_error = recovered_message;
                        }
                    }
                    
                    ExecutionResult {
                        output: AgentOutput::Text(format!("Execution error: {}", effective_error)),
                        success: false,
                        error: Some(effective_error),
                        telemetry: None,
                    }
                }
            };

            // Execute AfterModel hooks if execution succeeded or failed with non-quota error
            if let Some(executor) = hook_executor {
                match executor.execute_after_model(agent_id, &execution_result.output, execution_result.success).await {
                    Ok(results) => {
                        for result in results {
                            // If hook modifies output, update it
                            if let Some(modified_data) = result.modified_data {
                                if let Some(new_output) = modified_data.get("output") {
                                    if let Some(text) = new_output.get("text").and_then(|v| v.as_str()) {
                                        return ExecutionResult {
                                            output: AgentOutput::Text(text.to_string()),
                                            success: execution_result.success,
                                            error: execution_result.error,
                                            telemetry: execution_result.telemetry,
                                        };
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!(agent_id = %agent_id, error = %e, "AfterModel hook execution failed");
                    }
                }
            }

            // If we get here, execution succeeded or failed with non-quota error
            return execution_result;
        }
    }

    /// Executes an agent using the default model configuration.
    ///
    /// # Arguments
    /// * `agent` - The agent to execute
    /// * `input` - The input for the agent
    /// * `hook_registry` - Optional hook registry for execution interception
    ///
    /// # Returns
    /// Returns `ExecutionResult` with the agent's output or error information.
    ///
    /// # Errors
    /// Returns `ModelError` if model creation fails.
    pub async fn execute_agent_with_default_model(
        &self,
        agent: Arc<dyn Agent + Send + Sync>,
        input: &str,
        hook_executor: Option<&Arc<dyn HookExecutor>>,
    ) -> Result<ExecutionResult, ModelError> {
        let model = ModelFactory::create_from_str(
            match &self.default_model_type {
                ModelType::Mock => "mock",
                ModelType::Gemini => "gemini",
                ModelType::OpenAI => "openai",
            },
            self.default_model_id.clone(),
        )?;

        Ok(self.execute_agent(agent, input, model, hook_executor).await)
    }

    /// Executes an agent with a custom model type and ID.
    ///
    /// # Arguments
    /// * `agent` - The agent to execute
    /// * `input` - The input for the agent
    /// * `model_type` - The type of model to use
    /// * `model_id` - The model ID to use
    /// * `hook_registry` - Optional hook registry for execution interception
    ///
    /// # Returns
    /// Returns `ExecutionResult` with the agent's output or error information.
    ///
    /// # Errors
    /// Returns `ModelError` if model creation fails.
    pub async fn execute_agent_with_model(
        &self,
        agent: Arc<dyn Agent + Send + Sync>,
        input: &str,
        model_type: ModelType,
        model_id: String,
        hook_executor: Option<&Arc<dyn HookExecutor>>,
    ) -> Result<ExecutionResult, ModelError> {
        let model = ModelFactory::create_from_str(
            match &model_type {
                ModelType::Mock => "mock",
                ModelType::Gemini => "gemini",
                ModelType::OpenAI => "openai",
            },
            model_id,
        )?;

        Ok(self.execute_agent(agent, input, model, hook_executor).await)
    }
}

impl Default for AgentExecutor {
    fn default() -> Self {
        Self::with_mock_model()
    }
}

/// Configuration for the queue processor.
#[derive(Debug, Clone)]
pub struct QueueProcessorConfig {
    /// Maximum number of concurrent task executions.
    pub max_concurrent_tasks: usize,
    /// Timeout for individual task execution.
    pub task_timeout: Duration,
    /// Interval for polling the queue when empty.
    pub poll_interval: Duration,
}

impl Default for QueueProcessorConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 10,
            task_timeout: Duration::from_secs(30),
            poll_interval: Duration::from_millis(100),
        }
    }
}

/// Processor for executing queued agent tasks.
pub struct QueueProcessor {
    /// Configuration for the processor.
    config: QueueProcessorConfig,
    /// Semaphore for controlling concurrency.
    semaphore: Arc<Semaphore>,
    /// Registry for accessing agents.
    registry: Arc<AgentRegistry>,
    /// Lifecycle manager for agent states.
    lifecycle: Arc<AgentLifecycle>,
    /// Execution queue for tasks.
    queue: Arc<ExecutionQueue>,
    /// Executor for running agents.
    executor: Arc<AgentExecutor>,
    /// Shutdown signal sender.
    shutdown_tx: Option<mpsc::UnboundedSender<()>>,
}

impl fmt::Debug for QueueProcessor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("QueueProcessor")
            .field("config", &self.config)
            .field("max_concurrent_tasks", &self.config.max_concurrent_tasks)
            .finish_non_exhaustive()
    }
}

impl QueueProcessor {
    /// Creates a new queue processor with the given configuration.
    ///
    /// # Arguments
    /// * `config` - Configuration for the processor
    /// * `registry` - Agent registry
    /// * `lifecycle` - Lifecycle manager
    /// * `queue` - Execution queue
    /// * `executor` - Agent executor
    #[must_use]
    pub fn new(
        config: QueueProcessorConfig,
        registry: Arc<AgentRegistry>,
        lifecycle: Arc<AgentLifecycle>,
        queue: Arc<ExecutionQueue>,
        executor: Arc<AgentExecutor>,
    ) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_tasks));
        Self { config, semaphore, registry, lifecycle, queue, executor, shutdown_tx: None }
    }

    /// Starts the queue processor in a background task.
    ///
    /// # Returns
    /// Returns `Ok(())` if started successfully, or an error if already running.
    pub fn start(&mut self) -> Result<(), String> {
        if self.shutdown_tx.is_some() {
            return Err("Queue processor is already running".to_string());
        }

        let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel();
        self.shutdown_tx = Some(shutdown_tx);

        let config = self.config.clone();
        let semaphore = Arc::clone(&self.semaphore);
        let registry = Arc::clone(&self.registry);
        let lifecycle = Arc::clone(&self.lifecycle);
        let queue = Arc::clone(&self.queue);
        let executor = Arc::clone(&self.executor);

        tokio::spawn(async move {
            info!("Queue processor started");

            loop {
                tokio::select! {
                    result = shutdown_rx.recv() => {
                        match result {
                            Some(()) => {
                                info!("Queue processor shutdown signal received");
                            }
                            None => {
                                info!("Queue processor shutdown channel closed");
                            }
                        }
                        break;
                    }
                    () = time::sleep(config.poll_interval) => {
                        // Try to dequeue a task
                        if let Some(task) = queue.dequeue_task_immutable().await {
                            let task_id = task.task_id.clone().unwrap_or_else(|| format!("task-{}", uuid::Uuid::new_v4()));
                            let agent_id = task.agent_id.clone();
                            let input = task.input.clone();

                            // Acquire semaphore permit for concurrency control
                            let Ok(permit) = semaphore.clone().acquire_owned().await else {
                                error!("Semaphore closed, stopping processor");
                                break;
                            };

                            // Spawn task execution
                            let registry_clone = Arc::clone(&registry);
                            let lifecycle_clone = Arc::clone(&lifecycle);
                            let queue_clone = Arc::clone(&queue);
                            let executor_clone = Arc::clone(&executor);
                            let task_id_clone = task_id.clone();
                            let agent_id_clone = agent_id.clone();

                            tokio::spawn(async move {
                                let _permit = permit; // Hold permit for task duration

                                debug!(task_id = %task_id_clone, agent_id = %agent_id_clone, "Processing task");

                                // Get agent from registry
                                let Some(agent) = registry_clone.get_agent(&agent_id_clone).await else {
                                    error!(task_id = %task_id_clone, agent_id = %agent_id_clone, "Agent not found");
                                    // Only mark error if agent is registered (exists in lifecycle)
                                    if registry_clone.is_registered(&agent_id_clone).await {
                                        let _ = lifecycle_clone.mark_error(&agent_id_clone).await;
                                    }
                                    queue_clone.mark_completed(&task_id_clone).await;
                                    return;
                                };

                                // Check and update agent state
                                let state = lifecycle_clone.get_state(&agent_id_clone).await;
                                if state != AgentState::Idle && state != AgentState::Running {
                                    warn!(
                                        task_id = %task_id_clone,
                                        agent_id = %agent_id_clone,
                                        state = ?state,
                                        "Agent not in executable state"
                                    );
                                    let _ = lifecycle_clone.mark_error(&agent_id_clone).await;
                                    queue_clone.mark_completed(&task_id_clone).await;
                                    return;
                                }

                                // Start agent if idle
                                if state == AgentState::Idle {
                                    if let Err(current_state) = lifecycle_clone.start_agent(&agent_id_clone).await {
                                        error!(
                                            task_id = %task_id_clone,
                                            agent_id = %agent_id_clone,
                                            current_state = ?current_state,
                                            "Failed to start agent"
                                        );
                                        let _ = lifecycle_clone.mark_error(&agent_id_clone).await;
                                        queue_clone.mark_completed(&task_id_clone).await;
                                        return;
                                    }
                                }

                                // Execute with timeout
                                let execution_result = time::timeout(
                                    config.task_timeout,
                                    executor_clone.execute_agent_with_default_model(agent, &input, None as Option<&Arc<dyn HookExecutor>>),
                                )
                                .await;

                                match execution_result {
                                    Ok(Ok(result)) => {
                                        if result.success {
                                            info!(
                                                task_id = %task_id_clone,
                                                agent_id = %agent_id_clone,
                                                "Task completed successfully"
                                            );
                                            let _ = lifecycle_clone.set_state(&agent_id_clone, AgentState::Idle).await;
                                        } else {
                                            error!(
                                                task_id = %task_id_clone,
                                                agent_id = %agent_id_clone,
                                                error = ?result.error,
                                                "Task execution failed"
                                            );
                                            let _ = lifecycle_clone.mark_error(&agent_id_clone).await;
                                        }
                                    }
                                    Ok(Err(e)) => {
                                        error!(
                                            task_id = %task_id_clone,
                                            agent_id = %agent_id_clone,
                                            error = %e,
                                            "Task execution error"
                                        );
                                        let _ = lifecycle_clone.mark_error(&agent_id_clone).await;
                                    }
                                    Err(_) => {
                                        error!(
                                            task_id = %task_id_clone,
                                            agent_id = %agent_id_clone,
                                            timeout = ?config.task_timeout,
                                            "Task execution timed out"
                                        );
                                        let _ = lifecycle_clone.mark_error(&agent_id_clone).await;
                                    }
                                }

                                queue_clone.mark_completed(&task_id_clone).await;
                                debug!(task_id = %task_id_clone, "Task processing completed");
                            });
                        }
                    }
                }
            }

            info!("Queue processor stopped");
        });

        Ok(())
    }

    /// Stops the queue processor gracefully.
    ///
    /// # Returns
    /// Returns `Ok(())` if stopped successfully, or an error if not running.
    pub fn stop(&mut self) -> Result<(), String> {
        match self.shutdown_tx.take() {
            Some(shutdown_tx) => {
                let _ = shutdown_tx.send(());
                Ok(())
            }
            _ => Err("Queue processor is not running".to_string()),
        }
    }

    /// Checks if the processor is currently running.
    ///
    /// # Returns
    /// Returns `true` if running, `false` otherwise.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.shutdown_tx.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Agent, AgentContext, AgentRegistry, EchoAgent};

    #[tokio::test]
    async fn test_execute_agent_with_mock() {
        let executor = AgentExecutor::with_mock_model();
        let agent = Arc::new(EchoAgent::new("test-agent".to_string(), "Test agent".to_string()));
        let model = ModelFactory::create_from_str("mock", "mock-model".to_string()).unwrap();

        let result = executor.execute_agent(agent, "test input", model, None as Option<&Arc<dyn HookExecutor>>).await;

        assert!(result.success);
        match result.output {
            AgentOutput::Text(text) => {
                assert!(text.contains("Echo from test-agent"));
                assert!(text.contains("test input"));
            }
            _ => panic!("Expected Text output"),
        }
    }

    #[tokio::test]
    async fn test_execute_agent_with_default_model() {
        let executor = AgentExecutor::with_mock_model();
        let agent = Arc::new(EchoAgent::new("test-agent".to_string(), "Test agent".to_string()));

        let result = executor.execute_agent_with_default_model(agent, "test input", None as Option<&Arc<dyn HookExecutor>>).await.unwrap();

        assert!(result.success);
        match result.output {
            AgentOutput::Text(text) => {
                assert!(text.contains("Echo from test-agent"));
            }
            _ => panic!("Expected Text output"),
        }
    }

    #[tokio::test]
    async fn test_execute_agent_with_custom_model() {
        let executor = AgentExecutor::with_mock_model();
        let agent = Arc::new(EchoAgent::new("test-agent".to_string(), "Test agent".to_string()));

        let result = executor
            .execute_agent_with_model(
                agent,
                "test input",
                ModelType::Mock,
                "custom-model".to_string(),
                None as Option<&Arc<dyn HookExecutor>>,
            )
            .await
            .unwrap();

        assert!(result.success);
        match result.output {
            AgentOutput::Text(text) => {
                assert!(text.contains("Echo from test-agent"));
            }
            _ => panic!("Expected Text output"),
        }
    }

    #[tokio::test]
    async fn test_queue_processor_start_stop() {
        let registry = Arc::new(AgentRegistry::new());
        let lifecycle = Arc::new(AgentLifecycle::new());
        let queue = Arc::new(ExecutionQueue::new());
        let executor = Arc::new(AgentExecutor::with_mock_model());

        let mut processor = QueueProcessor::new(
            QueueProcessorConfig::default(),
            registry,
            lifecycle,
            queue,
            executor,
        );

        assert!(!processor.is_running());
        assert!(processor.start().is_ok());
        assert!(processor.is_running());

        // Wait a bit to ensure it started
        time::sleep(Duration::from_millis(50)).await;

        assert!(processor.stop().is_ok());
        assert!(!processor.is_running());

        // Should fail to start again immediately
        assert!(processor.start().is_ok());
        assert!(processor.stop().is_ok());
    }

    #[tokio::test]
    async fn test_queue_processor_processes_tasks() {
        let registry = Arc::new(AgentRegistry::new());
        let lifecycle = Arc::new(AgentLifecycle::new());
        let queue = Arc::new(ExecutionQueue::new());
        let executor = Arc::new(AgentExecutor::with_mock_model());

        // Register an agent
        let agent = Arc::new(EchoAgent::new("test-agent".to_string(), "Test agent".to_string()));
        registry.register_agent(agent).await;

        let mut processor = QueueProcessor::new(
            QueueProcessorConfig {
                max_concurrent_tasks: 2,
                task_timeout: Duration::from_secs(5),
                poll_interval: Duration::from_millis(10),
            },
            registry,
            lifecycle,
            Arc::clone(&queue),
            executor,
        );

        assert!(processor.start().is_ok());

        // Enqueue a task
        let task = ExecutionTask::new("test-agent".to_string(), "test input".to_string(), 1)
            .with_task_id("task-1".to_string());
        queue.enqueue_task(task).await.unwrap();

        // Wait for processing
        time::sleep(Duration::from_millis(200)).await;

        // Check that task was completed
        let metrics = queue.metrics().await;
        assert_eq!(metrics.completed, 1);
        assert_eq!(metrics.running, 0);
        assert_eq!(metrics.pending, 0);

        processor.stop().unwrap();
    }

    #[tokio::test]
    async fn test_queue_processor_handles_missing_agent() {
        let registry = Arc::new(AgentRegistry::new());
        let lifecycle = Arc::new(AgentLifecycle::new());
        let queue = Arc::new(ExecutionQueue::new());
        let executor = Arc::new(AgentExecutor::with_mock_model());

        let mut processor = QueueProcessor::new(
            QueueProcessorConfig {
                max_concurrent_tasks: 2,
                task_timeout: Duration::from_secs(5),
                poll_interval: Duration::from_millis(10),
            },
            registry,
            Arc::clone(&lifecycle),
            Arc::clone(&queue),
            executor,
        );

        assert!(processor.start().is_ok());

        // Enqueue a task for non-existent agent
        let task = ExecutionTask::new("nonexistent-agent".to_string(), "test input".to_string(), 1)
            .with_task_id("task-1".to_string());
        queue.enqueue_task(task).await.unwrap();

        // Wait for processing
        time::sleep(Duration::from_millis(200)).await;

        // Check that task was marked as completed (even though agent doesn't exist)
        let metrics = queue.metrics().await;
        assert_eq!(metrics.completed, 1);
        assert_eq!(metrics.running, 0);
        assert_eq!(metrics.pending, 0);

        processor.stop().unwrap();
    }

    #[tokio::test]
    async fn test_queue_processor_respects_concurrency_limit() {
        let registry = Arc::new(AgentRegistry::new());
        let lifecycle = Arc::new(AgentLifecycle::new());
        let queue = Arc::new(ExecutionQueue::new());
        let executor = Arc::new(AgentExecutor::with_mock_model());

        // Register an agent
        let agent = Arc::new(EchoAgent::new("test-agent".to_string(), "Test agent".to_string()));
        registry.register_agent(agent).await;

        let mut processor = QueueProcessor::new(
            QueueProcessorConfig {
                max_concurrent_tasks: 2,
                task_timeout: Duration::from_secs(5),
                poll_interval: Duration::from_millis(10),
            },
            registry,
            lifecycle,
            Arc::clone(&queue),
            executor,
        );

        assert!(processor.start().is_ok());

        // Enqueue multiple tasks
        for i in 0..5 {
            let task = ExecutionTask::new("test-agent".to_string(), format!("test input {}", i), 1)
                .with_task_id(format!("task-{}", i));
            queue.enqueue_task(task).await.unwrap();
        }

        // Wait a bit for processing to start
        time::sleep(Duration::from_millis(100)).await;

        // Check that at most 2 tasks are running (concurrency limit)
        let metrics = queue.metrics().await;
        assert!(metrics.running <= 2, "Running tasks should not exceed concurrency limit");

        // Wait for all tasks to complete
        time::sleep(Duration::from_secs(2)).await;

        let final_metrics = queue.metrics().await;
        assert_eq!(final_metrics.completed, 5);
        assert_eq!(final_metrics.running, 0);
        assert_eq!(final_metrics.pending, 0);

        processor.stop().unwrap();
    }

    #[tokio::test]
    async fn test_queue_processor_handles_timeout() {
        // Create a slow agent that will timeout
        struct SlowAgent {
            id: String,
            delay: Duration,
        }

        #[async_trait::async_trait]
        impl Agent for SlowAgent {
            fn id(&self) -> &str {
                &self.id
            }

            fn description(&self) -> &'static str {
                "Slow agent for testing"
            }

            async fn execute(
                &self,
                _input: &str,
                _context: AgentContext<'_>,
            ) -> Result<AgentOutput, ModelError> {
                time::sleep(self.delay).await;
                Ok(AgentOutput::Text("Done".to_string()))
            }
        }

        let registry = Arc::new(AgentRegistry::new());
        let lifecycle = Arc::new(AgentLifecycle::new());
        let queue = Arc::new(ExecutionQueue::new());
        let executor = Arc::new(AgentExecutor::with_mock_model());

        // Register slow agent
        let agent = Arc::new(SlowAgent {
            id: "slow-agent".to_string(),
            delay: Duration::from_secs(10), // Will timeout
        });
        registry.register_agent(agent).await;

        let mut processor = QueueProcessor::new(
            QueueProcessorConfig {
                max_concurrent_tasks: 1,
                task_timeout: Duration::from_millis(100), // Short timeout
                poll_interval: Duration::from_millis(10),
            },
            registry,
            Arc::clone(&lifecycle),
            Arc::clone(&queue),
            executor,
        );

        assert!(processor.start().is_ok());

        // Enqueue a task that will timeout
        let task = ExecutionTask::new("slow-agent".to_string(), "test input".to_string(), 1)
            .with_task_id("task-1".to_string());
        queue.enqueue_task(task).await.unwrap();

        // Wait for timeout
        time::sleep(Duration::from_millis(300)).await;

        // Check that agent is in error state (timeout)
        let state = lifecycle.get_state("slow-agent").await;
        assert_eq!(state, AgentState::Error);

        // Check that task was marked as completed
        let metrics = queue.metrics().await;
        assert_eq!(metrics.completed, 1);

        processor.stop().unwrap();
    }

    #[tokio::test]
    async fn test_queue_processor_processes_multiple_tasks() {
        let registry = Arc::new(AgentRegistry::new());
        let lifecycle = Arc::new(AgentLifecycle::new());
        let queue = Arc::new(ExecutionQueue::new());
        let executor = Arc::new(AgentExecutor::with_mock_model());

        // Register an agent
        let agent = Arc::new(EchoAgent::new("test-agent".to_string(), "Test agent".to_string()));
        registry.register_agent(agent).await;

        let mut processor = QueueProcessor::new(
            QueueProcessorConfig {
                max_concurrent_tasks: 2,
                task_timeout: Duration::from_secs(5),
                poll_interval: Duration::from_millis(10),
            },
            registry,
            lifecycle,
            Arc::clone(&queue),
            executor,
        );

        assert!(processor.start().is_ok());

        // Enqueue multiple tasks with different priorities
        for i in 0..5 {
            let task = ExecutionTask::new(
                "test-agent".to_string(),
                format!("input-{}", i),
                i + 1, // Different priorities
            )
            .with_task_id(format!("task-{}", i));
            queue.enqueue_task(task).await.unwrap();
        }

        // Wait for all tasks to complete
        let mut attempts = 0;
        loop {
            time::sleep(Duration::from_millis(200)).await;
            let metrics = queue.metrics().await;
            if metrics.completed == 5 && metrics.pending == 0 && metrics.running == 0 {
                break;
            }
            attempts += 1;
            assert!(attempts <= 30, "Tasks did not complete in time. Metrics: {:?}", metrics);
        }

        let metrics = queue.metrics().await;
        assert_eq!(metrics.completed, 5);
        assert_eq!(metrics.pending, 0);
        assert_eq!(metrics.running, 0);

        processor.stop().unwrap();
    }
}
