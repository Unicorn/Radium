//! Agent execution engine.
//!
//! This module provides functionality for executing agents with proper context and error handling.

#[cfg(test)]
use crate::ExecutionTask;
use crate::{
    Agent, AgentContext, AgentLifecycle, AgentOutput, AgentRegistry, AgentState, ExecutionQueue,
};
use radium_abstraction::ModelError;
use radium_models::{ModelConfig, ModelFactory, ModelType};
use crate::routing::RoutingStrategy;
use serde_json::Value;
use std::collections::HashSet;
use std::fmt;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Semaphore, mpsc, RwLock};
use tokio::time;
use tokio_util::sync::CancellationToken;
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
    /// Optional routing decision metadata (for Smart/Eco tier routing).
    pub routing_decision: Option<crate::routing::RoutingDecision>,
}

/// Context tracking failover attempts for error messages.
#[derive(Debug, Clone)]
struct FailoverContext {
    /// List of (provider, model) combinations that were attempted.
    attempted_combinations: Vec<(String, String)>,
    /// List of error types encountered (e.g., "rate_limit", "quota_exhausted").
    error_types: Vec<String>,
    /// Budget status if budget manager is available (formatted string).
    budget_status: Option<String>,
}

/// Trait for budget management to avoid circular dependency with radium-core.
pub trait BudgetManagerTrait: Send + Sync {
    /// Check if estimated cost is within budget.
    fn check_budget_available(&self, estimated_cost: f64) -> Result<(), BudgetCheckResult>;
    
    /// Record an actual cost after execution.
    fn record_cost(&self, actual_cost: f64);
    
    /// Get budget status as a formatted string.
    fn get_budget_status_string(&self) -> Option<String>;
}

/// Result of budget check.
#[derive(Debug, Clone)]
pub enum BudgetCheckResult {
    /// Budget limit exceeded.
    BudgetExceeded {
        spent: f64,
        limit: f64,
        requested: f64,
    },
    /// Budget warning threshold reached.
    BudgetWarning {
        spent: f64,
        limit: f64,
        percentage: f64,
    },
}

/// Trait for sandbox operations to avoid circular dependency with radium-core.
#[async_trait::async_trait]
pub trait SandboxManager: Send + Sync {
    /// Initialize sandbox for an agent.
    async fn initialize_sandbox(&self, agent_id: &str) -> Result<(), String>;
    
    /// Cleanup sandbox for an agent.
    async fn cleanup_sandbox(&self, agent_id: &str);
    
    /// Get active sandbox for an agent (if any).
    fn get_active_sandbox(&self, agent_id: &str) -> Option<Box<dyn std::any::Any + Send + Sync>>;
}

/// Executor for running agents.
pub struct AgentExecutor {
    /// Default model type to use if not specified.
    default_model_type: ModelType,
    /// Default model ID to use if not specified.
    default_model_id: String,
    /// Optional sandbox manager for sandbox operations.
    sandbox_manager: Option<Arc<dyn SandboxManager>>,
    /// Optional budget manager for cost tracking and enforcement.
    budget_manager: Option<Arc<dyn BudgetManagerTrait>>,
    /// Optional model router for Smart/Eco tier selection.
    model_router: Option<Arc<crate::routing::ModelRouter>>,
    /// Optional tier override for manual routing control.
    tier_override: Option<crate::routing::RoutingTier>,
}

impl fmt::Debug for AgentExecutor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AgentExecutor")
            .field("default_model_type", &self.default_model_type)
            .field("default_model_id", &self.default_model_id)
            .field("sandbox_manager", &if self.sandbox_manager.is_some() { "Some" } else { "None" })
            .field("budget_manager", &"<optional>")
            .field("model_router", &if self.model_router.is_some() { "Some" } else { "None" })
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
        Self {
            default_model_type,
            default_model_id,
            sandbox_manager: None,
            #[allow(unused_mut)]
            budget_manager: None,
            model_router: None,
            tier_override: None,
        }
    }

    /// Creates a new agent executor with sandbox manager.
    ///
    /// # Arguments
    /// * `default_model_type` - Default model type to use
    /// * `default_model_id` - Default model ID to use
    /// * `sandbox_manager` - Optional sandbox manager for sandbox operations
    #[must_use]
    pub fn with_sandbox_manager(
        default_model_type: ModelType,
        default_model_id: String,
        sandbox_manager: Option<Arc<dyn SandboxManager>>,
    ) -> Self {
        Self {
            default_model_type,
            default_model_id,
            sandbox_manager,
            #[allow(unused_mut)]
            budget_manager: None,
            model_router: None,
            tier_override: None,
        }
    }

    /// Sets the sandbox manager.
    ///
    /// # Arguments
    /// * `manager` - The sandbox manager to use
    pub fn set_sandbox_manager(&mut self, manager: Arc<dyn SandboxManager>) {
        self.sandbox_manager = Some(manager);
    }

    /// Sets the budget manager.
    ///
    /// # Arguments
    /// * `manager` - The budget manager to use
    pub fn set_budget_manager(&mut self, manager: Arc<dyn BudgetManagerTrait>) {
        self.budget_manager = Some(manager);
    }

    /// Creates a new agent executor with Mock model as default.
    #[must_use]
    pub fn with_mock_model() -> Self {
        Self::new(ModelType::Mock, "mock-model".to_string())
    }

    /// Sets the model router for automatic Smart/Eco tier selection.
    ///
    /// # Arguments
    /// * `router` - The model router to use
    pub fn set_model_router(&mut self, router: Arc<crate::routing::ModelRouter>) {
        self.model_router = Some(router);
    }

    /// Sets a tier override for manual routing control.
    ///
    /// # Arguments
    /// * `tier` - The tier to override with (Smart, Eco, or Auto for automatic)
    pub fn set_tier_override(&mut self, tier: Option<crate::routing::RoutingTier>) {
        self.tier_override = tier;
    }


    /// Creates a checkpoint when all providers are exhausted.
    ///
    /// This is a simplified checkpoint creation that uses git directly
    /// to avoid circular dependency with radium-core.
    ///
    /// # Arguments
    /// * `workspace_root` - Root directory of the workspace
    /// * `description` - Optional description for the checkpoint
    ///
    /// # Returns
    /// Returns the checkpoint ID/hash if successful, or None if checkpoint creation failed.
    fn create_checkpoint_on_exhaustion(
        workspace_root: &Path,
        description: Option<&str>,
    ) -> Option<String> {
        // Check if workspace is a git repository
        if !workspace_root.join(".git").exists() {
            warn!("Workspace is not a git repository, skipping checkpoint creation");
            return None;
        }

        // Get current commit hash
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(workspace_root)
            .output();

        let commit_hash = match output {
            Ok(output) if output.status.success() => {
                String::from_utf8_lossy(&output.stdout).trim().to_string()
            }
            _ => {
                warn!("Failed to get current commit hash, skipping checkpoint creation");
                return None;
            }
        };

        // Generate checkpoint ID
        let checkpoint_id = format!("checkpoint-exhaustion-{}", &commit_hash[..8]);

        // Create git tag for checkpoint
        let mut tag_args = vec!["tag", "-a", &checkpoint_id, "-m"];
        let tag_message = description
            .unwrap_or("Provider exhaustion checkpoint")
            .to_string();
        tag_args.push(&tag_message);

        let tag_output = Command::new("git")
            .args(&tag_args)
            .current_dir(workspace_root)
            .output();

        match tag_output {
            Ok(output) if output.status.success() => {
                info!(
                    checkpoint_id = %checkpoint_id,
                    commit_hash = %commit_hash,
                    "Workspace checkpoint created"
                );
                Some(checkpoint_id)
            }
            _ => {
                warn!("Failed to create checkpoint tag, but commit hash is: {}", commit_hash);
                // Return commit hash as fallback
                Some(commit_hash)
            }
        }
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

    /// Formats a comprehensive error message for quota exhaustion scenarios.
    fn format_quota_exhaustion_error(context: &FailoverContext) -> String {
        let mut message = String::from("All AI providers exhausted.\n\n");
        
        // List attempted providers and models
        if !context.attempted_combinations.is_empty() {
            message.push_str("Attempted providers/models:\n");
            for (provider, model) in &context.attempted_combinations {
                message.push_str(&format!("  - {} ({})\n", provider, model));
            }
            message.push('\n');
        }
        
        // Include budget status if available
        if let Some(ref budget_status_str) = context.budget_status {
            message.push_str("Budget status: ");
            message.push_str(budget_status_str);
            message.push('\n');
        }
        
        // Determine error types and provide actionable next steps
        let has_rate_limit = context.error_types.iter().any(|t| t == "rate_limit");
        let has_quota_exhausted = context.error_types.iter().any(|t| t == "quota_exhausted");
        
        message.push_str("Next steps:\n");
        if has_rate_limit {
            message.push_str("  - Rate limits detected: Wait a few minutes and retry\n");
        }
        if has_quota_exhausted {
            message.push_str("  - Quota exhausted: Add credits to your provider accounts\n");
        }
        message.push_str("  - Increase budget limit if budget was the constraint\n");
        message.push_str("  - Check provider status pages for service issues\n");
        
        message
    }

    /// Gets a cheaper model alternative for the given provider and model.
    ///
    /// Returns the next cheaper model in the tier, or None if no cheaper model exists.
    fn get_cheaper_model(provider: &str, model_id: &str) -> Option<String> {
        let model_lower = model_id.to_lowercase();
        
        match provider {
            "openai" => {
                if model_lower.contains("gpt-4") {
                    Some("gpt-3.5-turbo".to_string())
                } else {
                    None
                }
            }
            "claude" | "anthropic" => {
                if model_lower.contains("opus") {
                    Some("claude-3-sonnet-20240229".to_string())
                } else if model_lower.contains("sonnet") {
                    Some("claude-3-haiku-20240307".to_string())
                } else {
                    None
                }
            }
            "gemini" => {
                if model_lower.contains("ultra") {
                    Some("gemini-pro".to_string())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Finds the next available provider for failover.
    ///
    /// Returns the next provider in the failover order (openai → claude → gemini → mock)
    /// that is not fully exhausted and has available credentials.
    ///
    /// # Arguments
    /// * `exhausted_combinations` - Set of (provider, model) pairs that have been exhausted
    /// * `current_provider` - The current provider name (to determine failover order)
    ///
    /// # Returns
    /// Returns `Some((provider_name, model_type, model_id))` if a backup provider is found,
    /// or `None` if all providers are exhausted.
    fn find_next_available_provider(
        exhausted_combinations: &HashSet<(String, String)>,
        current_provider: &str,
    ) -> Option<(String, ModelType, String)> {
        // Failover order: openai → claude → gemini → mock
        let failover_order = [
            ("openai", ModelType::OpenAI, "gpt-3.5-turbo".to_string()),
            ("claude", ModelType::Claude, "claude-3-sonnet-20240229".to_string()),
            ("gemini", ModelType::Gemini, "gemini-pro".to_string()),
            ("mock", ModelType::Mock, "mock-model".to_string()),
        ];

        // Find current provider index to start from next in order
        let current_idx = failover_order
            .iter()
            .position(|(name, _, _)| *name == current_provider)
            .unwrap_or(0);

        // Try providers in failover order starting from next after current
        for i in 0..failover_order.len() {
            let idx = (current_idx + 1 + i) % failover_order.len();
            let (provider_name, model_type, default_model_id) = &failover_order[idx];
            
            // Check if this (provider, model) combination is exhausted
            if exhausted_combinations.contains(&((*provider_name).to_string(), default_model_id.clone())) {
                // Try to find a non-exhausted model for this provider
                // For now, we'll skip if the default model is exhausted
                // In a more sophisticated implementation, we could try other models
                continue;
            }

            // Check if provider is available
            // Mock is always available (no credentials needed)
            if *provider_name == "mock" {
                return Some(((*provider_name).to_string(), model_type.clone(), default_model_id.clone()));
            }

            // For real providers, check environment variables for API keys
            // Allow env::var for credential checking
            #[allow(clippy::disallowed_methods)]
            let has_credentials = match *provider_name {
                "openai" => std::env::var("OPENAI_API_KEY").is_ok(),
                "claude" | "anthropic" => std::env::var("ANTHROPIC_API_KEY").is_ok(),
                "gemini" => std::env::var("GEMINI_API_KEY").is_ok() || std::env::var("GOOGLE_API_KEY").is_ok(),
                _ => false,
            };

            if has_credentials {
                return Some(((*provider_name).to_string(), model_type.clone(), default_model_id.clone()));
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
    /// Executes an agent with optional collaboration context.
    ///
    /// # Arguments
    /// * `agent` - The agent to execute
    /// * `input` - The input for the agent
    /// * `model` - The model to use for generation
    /// * `hook_executor` - Optional hook executor for execution interception
    /// * `collaboration_context` - Optional collaboration context (passed as raw pointer to avoid circular dependency)
    pub async fn execute_agent(
        &self,
        agent: Arc<dyn Agent + Send + Sync>,
        input: &str,
        model: Arc<dyn radium_abstraction::Model + Send + Sync>,
        hook_executor: Option<&Arc<dyn HookExecutor>>,
    ) -> ExecutionResult {
        let agent_id = agent.id();
        debug!(agent_id = %agent_id, input_len = input.len(), "Executing agent");

        // Initialize sandbox for agent if sandbox manager is configured
        if let Some(ref manager) = self.sandbox_manager {
            if let Err(e) = manager.initialize_sandbox(agent_id).await {
                warn!(
                    agent_id = %agent_id,
                    error = %e,
                    "Failed to initialize sandbox, continuing without sandbox"
                );
            }
        }


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
                            // Cleanup sandbox before early return
                            if let Some(ref manager) = self.sandbox_manager {
                                manager.cleanup_sandbox(agent_id).await;
                            }
                            return ExecutionResult {
                                output: AgentOutput::Text(format!("Execution aborted: {}", message)),
                                success: false,
                                error: Some(message),
                                telemetry: None,
                                routing_decision: None,
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

        // Track exhausted (provider, model) pairs for failover
        let mut exhausted_combinations: HashSet<(String, String)> = HashSet::new();
        let mut current_model = model;
        let mut current_provider = Self::infer_provider_from_model(&current_model);
        
        // Track failover context for error messages
        let mut failover_context = FailoverContext {
            attempted_combinations: Vec::new(),
            error_types: Vec::new(),
            budget_status: None,
        };

        // Budget checking disabled - requires radium_core dependency
        // TODO: Re-enable budget checking when radium_core is available as dependency
        if self.budget_manager.is_some() {
            debug!(
                agent_id = %agent_id,
                "Budget manager present but checking disabled (radium_core not available)"
            );
        }

        // Retry loop with provider failover
        loop {
            // Record this attempt in failover context
            let current_model_id = current_model.model_id().to_string();
            failover_context.attempted_combinations.push((current_provider.clone(), current_model_id.clone()));
            
            // Create agent context with current model
            let context = AgentContext {
                model: current_model.as_ref(),
                // Collaboration context removed to avoid circular dependency
            };

            // Execute the agent
            // Note: Telemetry capture requires modifying agents to return ModelResponse
            // For now, telemetry is None - will be captured when agents are updated
            let execution_result = match agent.execute(&effective_input, context).await {
                Ok(output) => {
                    info!(agent_id = %agent_id, output_type = ?output, provider = %current_provider, "Agent execution completed successfully");
                    
                    // Record cost after successful execution
                    if let Some(ref _budget_manager) = self.budget_manager {
                        // Estimate actual cost from input/output (rough estimate)
                        let input_tokens = effective_input.len() as f64 / 4.0;
                        let _output_tokens = if let AgentOutput::Text(ref text) = output {
                            text.len() as f64 / 4.0
                        } else {
                            input_tokens * 0.3 / 0.7 // Default estimate
                        };
                        // Budget cost recording disabled - requires radium_core dependency
                        // TODO: Re-enable when radium_core is available
                        debug!(
                            agent_id = %agent_id,
                            "Budget manager present but cost recording disabled"
                        );
                    }
                    
                    ExecutionResult {
                        output: output,
                        success: true,
                        error: None,
                        telemetry: None, // Will be populated when agents capture ModelResponse
                        routing_decision: None,
                    }
                }
                Err(e) => {
                    // Check if this is a QuotaExceeded error that should trigger failover
                    if let ModelError::QuotaExceeded { provider, message } = &e {
                        let current_model_id = current_model.model_id().to_string();
                        
                        // Determine error type from message
                        let error_type = if let Some(msg) = message {
                            if msg.contains("429") || msg.contains("rate limit") {
                                "rate_limit"
                            } else if msg.contains("402") || msg.contains("quota") {
                                "quota_exhausted"
                            } else {
                                "quota_exhausted"
                            }
                        } else {
                            "quota_exhausted"
                        };
                        failover_context.error_types.push(error_type.to_string());
                        
                        warn!(
                            agent_id = %agent_id,
                            provider = %provider,
                            model = %current_model_id,
                            error_type = %error_type,
                            "Provider/model reported insufficient quota"
                        );
                        
                        // Mark (provider, model) combination as exhausted
                        exhausted_combinations.insert((provider.clone(), current_model_id.clone()));
                        current_provider = provider.clone();

                        // Try cheaper model from same provider first
                        if let Some(cheaper_model_id) = Self::get_cheaper_model(&current_provider, &current_model_id) {
                            if !exhausted_combinations.contains(&(current_provider.clone(), cheaper_model_id.clone())) {
                                info!(
                                    agent_id = %agent_id,
                                    provider = %current_provider,
                                    from_model = %current_model_id,
                                    to_model = %cheaper_model_id,
                                    "Falling back to cheaper model within provider"
                                );

                                // Create cheaper model instance
                                match ModelFactory::create_from_str(
                                    match current_provider.as_str() {
                                        "openai" => "openai",
                                        "claude" | "anthropic" => "claude",
                                        "gemini" => "gemini",
                                        _ => "mock",
                                    },
                                    cheaper_model_id.clone(),
                                ) {
                                    Ok(new_model) => {
                                        current_model = new_model;
                                        // Continue loop to retry with cheaper model
                                        continue;
                                    }
                                    Err(create_err) => {
                                        warn!(
                                            agent_id = %agent_id,
                                            provider = %current_provider,
                                            model = %cheaper_model_id,
                                            error = %create_err,
                                            "Failed to create cheaper model, trying next provider"
                                        );
                                        // Mark cheaper model as exhausted and fall through to provider failover
                                        exhausted_combinations.insert((current_provider.clone(), cheaper_model_id));
                                    }
                                }
                            }
                        }

                        // Find next available provider
                        if let Some((next_provider, next_model_type, next_model_id)) =
                            Self::find_next_available_provider(&exhausted_combinations, &current_provider)
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
                                    ModelType::Claude => "claude",
                                    ModelType::Gemini => "gemini",
                                    ModelType::OpenAI => "openai",
                                    ModelType::Universal => "universal",
                                    ModelType::Ollama => "ollama",
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
                            // All providers exhausted - create checkpoint before stopping
                            // Get budget status if available
                            if let Some(ref budget_manager) = self.budget_manager {
                                if let Some(status_str) = budget_manager.get_budget_status_string() {
                                    failover_context.budget_status = Some(status_str);
                                }
                            }
                            
                            error!(
                                agent_id = %agent_id,
                                attempted = ?failover_context.attempted_combinations,
                                "All configured providers are exhausted"
                            );
                            
                            // Format comprehensive error message
                            let error_message = Self::format_quota_exhaustion_error(&failover_context);
                            
                            // Try to create checkpoint
                            // Discover workspace root by checking current directory and parent directories
                            let workspace_root = std::env::current_dir()
                                .ok()
                                .and_then(|mut path| {
                                    loop {
                                        if path.join(".radium").exists() || path.join(".git").exists() {
                                            return Some(path);
                                        }
                                        if !path.pop() {
                                            return None;
                                        }
                                    }
                                });
                            
                            let checkpoint_id = if let Some(root) = &workspace_root {
                                Self::create_checkpoint_on_exhaustion(
                                    root,
                                    Some(&format!("Provider exhaustion: {}", error_message)),
                                )
                            } else {
                                warn!("Could not discover workspace root, skipping checkpoint creation");
                                None
                            };
                            
                            let final_error_message = if let Some(checkpoint_id) = checkpoint_id {
                                format!(
                                    "{}\n\nWorkspace checkpoint created: {}",
                                    error_message, checkpoint_id
                                )
                            } else {
                                error_message
                            };
                            
                            // Cleanup sandbox before returning
                            if let Some(ref manager) = self.sandbox_manager {
                                manager.cleanup_sandbox(agent_id).await;
                            }
                            return ExecutionResult {
                                output: AgentOutput::Text(final_error_message.clone()),
                                success: false,
                                error: Some(final_error_message),
                                telemetry: None,
                                routing_decision: None,
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
                        routing_decision: None,
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
                                        // Cleanup sandbox before returning
                                        if let Some(ref manager) = self.sandbox_manager {
                                            manager.cleanup_sandbox(agent_id).await;
                                        }
                                        return ExecutionResult {
                                            output: AgentOutput::Text(text.to_string()),
                                            success: execution_result.success,
                                            error: execution_result.error,
                                            telemetry: execution_result.telemetry,
                                            routing_decision: execution_result.routing_decision,
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
            // Cleanup sandbox before returning
            if let Some(ref manager) = self.sandbox_manager {
                manager.cleanup_sandbox(agent_id).await;
            }
            return execution_result;
        }
    }

    /// Executes an agent using the default model configuration or routed model.
    ///
    /// # Arguments
    /// * `agent` - The agent to execute
    /// * `input` - The input for the agent
    /// * `hook_executor` - Optional hook executor for execution interception
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
        // Use model router if available, otherwise use default model
        let (model, routing_decision) = if let Some(ref router) = self.model_router {
            // Route model based on complexity or override
            // Use default strategy (complexity-based) for backward compatibility
            let (model_config, decision) = router.select_model(
                input,
                Some(agent.id()),
                self.tier_override,
            );
            
            // Create model from routed config
            let model = ModelFactory::create(ModelConfig::new(
                model_config.model_type.clone(),
                model_config.model_id.clone(),
            ))?;
            
            (model, Some(decision))
        } else {
            // Fallback to default model
            let model = ModelFactory::create_from_str(
                match &self.default_model_type {
                    ModelType::Mock => "mock",
                    ModelType::Claude => "claude",
                    ModelType::Gemini => "gemini",
                    ModelType::OpenAI => "openai",
                    ModelType::Universal => "universal",
                    ModelType::Ollama => "ollama",
                },
                self.default_model_id.clone(),
            )?;
            (model, None)
        };

        // Execute agent
        let mut result = self.execute_agent(agent.clone(), input, model.clone(), hook_executor).await;

        // Track usage if router is available and execution succeeded
        if let Some(ref router) = self.model_router {
            if let Some(ref routing_decision) = routing_decision {
                if let Some(ref telemetry) = result.telemetry {
                    // Extract model usage from telemetry
                    let usage = radium_abstraction::ModelUsage {
                        prompt_tokens: telemetry.input_tokens as u32,
                        completion_tokens: telemetry.output_tokens as u32,
                        total_tokens: telemetry.total_tokens as u32,
                        cache_usage: None,
                    };
                    
                    // Track usage (non-blocking)
                    router.track_usage(
                        routing_decision.tier,
                        &usage,
                        &model.model_id(),
                    );
                }
                
                // Store routing decision in result for telemetry recording
                result.routing_decision = Some(routing_decision.clone());
            }
        }

        Ok(result)
    }
    
    /// Executes an agent with a specific routing strategy.
    ///
    /// # Arguments
    /// * `agent` - The agent to execute
    /// * `input` - The input for the agent
    /// * `hook_executor` - Optional hook executor for execution interception
    /// * `routing_strategy` - Optional routing strategy (uses router default if not provided)
    ///
    /// # Errors
    /// Returns `ModelError` if model creation fails.
    pub async fn execute_agent_with_routing_strategy(
        &self,
        agent: Arc<dyn Agent + Send + Sync>,
        input: &str,
        hook_executor: Option<&Arc<dyn HookExecutor>>,
        routing_strategy: Option<RoutingStrategy>,
    ) -> Result<ExecutionResult, ModelError> {
        // Use model router if available, otherwise use default model
        let (model, routing_decision) = if let Some(ref router) = self.model_router {
            // Route model based on complexity or override with specified strategy
            let strategy = routing_strategy.unwrap_or(crate::routing::RoutingStrategy::ComplexityBased);
            let (model_config, decision) = router.select_model_with_strategy(
                input,
                Some(agent.id()),
                self.tier_override,
                strategy,
            );
            
            // Create model from routed config
            let model = ModelFactory::create(ModelConfig::new(
                model_config.model_type.clone(),
                model_config.model_id.clone(),
            ))?;
            
            (model, Some(decision))
        } else {
            // Fallback to default model
            let model = ModelFactory::create_from_str(
                match &self.default_model_type {
                    ModelType::Mock => "mock",
                    ModelType::Claude => "claude",
                    ModelType::Gemini => "gemini",
                    ModelType::OpenAI => "openai",
                    ModelType::Universal => "universal",
                    ModelType::Ollama => "ollama",
                },
                self.default_model_id.clone(),
            )?;
            (model, None)
        };

        // Execute agent
        let mut result = self.execute_agent(agent.clone(), input, model.clone(), hook_executor).await;

        // Track usage if router is available and execution succeeded
        if let Some(ref router) = self.model_router {
            if let Some(ref routing_decision) = routing_decision {
                if let Some(ref telemetry) = result.telemetry {
                    // Extract model usage from telemetry
                    let usage = radium_abstraction::ModelUsage {
                        prompt_tokens: telemetry.input_tokens as u32,
                        completion_tokens: telemetry.output_tokens as u32,
                        total_tokens: telemetry.total_tokens as u32,
                        cache_usage: None,
                    };
                    
                    // Track usage (non-blocking)
                    router.track_usage(
                        routing_decision.tier,
                        &usage,
                        &model.model_id(),
                    );
                }
                
                // Store routing decision in result for telemetry recording
                result.routing_decision = Some(routing_decision.clone());
            }
        }

        Ok(result)
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
                ModelType::Claude => "claude",
                ModelType::Gemini => "gemini",
                ModelType::OpenAI => "openai",
                ModelType::Universal => "universal",
                ModelType::Ollama => "ollama",
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
    /// Cancellation tokens for running tasks (task_id -> token).
    cancellation_tokens: Arc<RwLock<std::collections::HashMap<String, CancellationToken>>>,
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
        Self {
            config,
            semaphore,
            registry,
            lifecycle,
            queue,
            executor,
            shutdown_tx: None,
            cancellation_tokens: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
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
        let cancellation_tokens = Arc::clone(&self.cancellation_tokens);

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

                            // Create cancellation token for this task
                            let cancellation_token = CancellationToken::new();
                            {
                                let mut tokens = cancellation_tokens.write().await;
                                tokens.insert(task_id.clone(), cancellation_token.clone());
                            }

                            // Spawn task execution
                            let registry_clone = Arc::clone(&registry);
                            let lifecycle_clone = Arc::clone(&lifecycle);
                            let queue_clone = Arc::clone(&queue);
                            let executor_clone = Arc::clone(&executor);
                            let cancellation_tokens_clone = Arc::clone(&cancellation_tokens);
                            let task_id_clone = task_id.clone();
                            let agent_id_clone = agent_id.clone();
                            let cancellation_token_clone = cancellation_token.clone();
                            let task_timeout = config.task_timeout;

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

                                // Execute with cancellation token support
                                let execution_future = executor_clone.execute_agent_with_default_model(agent, &input, None as Option<&Arc<dyn HookExecutor>>);
                                
                                let execution_result: std::result::Result<ExecutionResult, String> = tokio::select! {
                                    result = tokio::time::timeout(task_timeout, execution_future) => {
                                        match result {
                                            Ok(Ok(result)) => Ok(result),
                                            Ok(Err(e)) => Err(format!("Model execution failed: {}", e)),
                                            Err(_) => Err(format!("Task timed out after {:?}", task_timeout)),
                                        }
                                    }
                                    _ = cancellation_token_clone.cancelled() => {
                                        // Task was cancelled
                                        info!(
                                            task_id = %task_id_clone,
                                            agent_id = %agent_id_clone,
                                            "Task execution cancelled"
                                        );
                                        // Mark as cancelled in lifecycle
                                        let _ = lifecycle_clone.set_state(&agent_id_clone, AgentState::Cancelled).await;
                                        Err("Task cancelled".to_string())
                                    }
                                };

                                match execution_result {
                                    Ok(result) => {
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
                                    Err(msg) if msg == "Task cancelled" => {
                                        // Already handled above, just clean up
                                    }
                                    Err(e) => {
                                        error!(
                                            task_id = %task_id_clone,
                                            agent_id = %agent_id_clone,
                                            error = %e,
                                            "Unexpected error during task execution"
                                        );
                                        let _ = lifecycle_clone.mark_error(&agent_id_clone).await;
                                    }
                                }

                                // Clean up cancellation token
                                {
                                    let mut tokens = cancellation_tokens_clone.write().await;
                                    tokens.remove(&task_id_clone);
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
                // Trigger all cancellation tokens for running tasks
                // We need to do this in an async context, so we spawn a task
                let tokens = Arc::clone(&self.cancellation_tokens);
                tokio::spawn(async move {
                    let tokens_read = tokens.read().await;
                    for (task_id, token) in tokens_read.iter() {
                        info!(task_id = %task_id, "Cancelling task");
                        token.cancel();
                    }
                });
                
                // Send shutdown signal
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

    /// Cancels a specific task by triggering its cancellation token.
    ///
    /// # Arguments
    /// * `task_id` - The task ID to cancel
    ///
    /// # Returns
    /// Returns `true` if the task was found and cancelled, `false` otherwise.
    pub async fn cancel_task(&self, task_id: &str) -> bool {
        let tokens = self.cancellation_tokens.read().await;
        if let Some(token) = tokens.get(task_id) {
            info!(task_id = %task_id, "Triggering cancellation token for task");
            token.cancel();
            true
        } else {
            false
        }
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
        let mut state = lifecycle.get_state("slow-agent").await;
        let start = std::time::Instant::now();
        while state != AgentState::Error && start.elapsed() < std::time::Duration::from_secs(2) {
            time::sleep(Duration::from_millis(50)).await;
            state = lifecycle.get_state("slow-agent").await;
        }

        // Check that agent is in error state (timeout)
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

    #[tokio::test]
    async fn test_executor_with_model_router() {
        use crate::routing::ModelRouter;
        
        let smart_config = ModelConfig::new(ModelType::Mock, "smart-model".to_string());
        let eco_config = ModelConfig::new(ModelType::Mock, "eco-model".to_string());
        let router = Arc::new(ModelRouter::new(smart_config, eco_config, Some(60.0)));
        
        let mut executor = AgentExecutor::with_mock_model();
        executor.set_model_router(router);
        
        let agent = Arc::new(EchoAgent::new("test-agent".to_string(), "Test agent".to_string()));
        
        // Simple task should route to eco
        let result = executor.execute_agent_with_default_model(
            agent.clone(),
            "format this JSON",
            None as Option<&Arc<dyn HookExecutor>>,
        )
        .await
        .unwrap();
        
        assert!(result.success);
        // Routing decision should be present
        assert!(result.routing_decision.is_some());
        
        // Complex task should route to smart
        let result2 = executor.execute_agent_with_default_model(
            agent.clone(),
            "refactor this module with dependency injection and analyze architecture trade-offs",
            None as Option<&Arc<dyn HookExecutor>>,
        )
        .await
        .unwrap();
        
        assert!(result2.success);
        assert!(result2.routing_decision.is_some());
    }

    #[tokio::test]
    async fn test_executor_with_manual_tier_override() {
        use crate::routing::{ModelRouter, RoutingTier};
        
        let smart_config = ModelConfig::new(ModelType::Mock, "smart-model".to_string());
        let eco_config = ModelConfig::new(ModelType::Mock, "eco-model".to_string());
        let router = Arc::new(ModelRouter::new(smart_config, eco_config, Some(60.0)));
        
        let mut executor = AgentExecutor::with_mock_model();
        executor.set_model_router(router);
        executor.set_tier_override(Some(RoutingTier::Smart));
        
        let agent = Arc::new(EchoAgent::new("test-agent".to_string(), "Test agent".to_string()));
        
        // Even simple task should route to smart due to override
        let result = executor.execute_agent_with_default_model(
            agent,
            "simple task",
            None as Option<&Arc<dyn HookExecutor>>,
        )
        .await
        .unwrap();
        
        assert!(result.success);
        assert!(result.routing_decision.is_some());
        if let Some(decision) = result.routing_decision {
            assert_eq!(decision.tier, RoutingTier::Smart);
            assert_eq!(decision.decision_type, crate::routing::DecisionType::Manual);
        }
    }
}
