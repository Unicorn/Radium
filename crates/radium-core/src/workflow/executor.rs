//! Sequential workflow execution.
//!
//! This module provides functionality for executing workflows sequentially,
//! processing steps in order and handling failures.

use std::sync::Arc;
use tracing::{debug, error, info};

use radium_orchestrator::{AgentExecutor, Orchestrator};

use crate::checkpoint::CheckpointManager;
use crate::hooks::integration::OrchestratorHooks;
use crate::hooks::registry::{HookRegistry, HookType};
use crate::hooks::types::{HookContext, HookPriority};
use crate::models::{Workflow, WorkflowState};
use crate::monitoring::{AgentRecord, AgentStatus, MonitoringService};
use crate::policy::{ApprovalMode, ConstitutionManager, PolicyEngine};
use crate::storage::TaskRepository;
use crate::workspace::{Workspace, WorkspaceStructure};

use super::control_flow::{StepCondition, should_execute_step};
use super::engine::{ExecutionContext, StepResult, WorkflowEngine, WorkflowEngineError};
use super::failure::{FailureClassifier, FailureHistory, FailurePolicy};
use chrono::Utc;
use std::collections::HashMap;

/// Executor for running workflows sequentially.
///
/// Executes workflow steps in order, waiting for each step to complete
/// before proceeding to the next.
pub struct WorkflowExecutor {
    /// Core workflow engine.
    engine: WorkflowEngine,
    /// Monitoring service for agent lifecycle tracking (optional).
    monitoring: Option<Arc<std::sync::Mutex<MonitoringService>>>,
    /// Checkpoint manager for creating snapshots (optional).
    checkpoint_manager: Option<Arc<std::sync::Mutex<CheckpointManager>>>,
    /// Hook registry for workflow behavior hooks (optional).
    hook_registry: Option<Arc<HookRegistry>>,
    /// Policy engine for tool execution control (optional).
    policy_engine: Option<Arc<std::sync::Mutex<PolicyEngine>>>,
    /// Constitution manager for session-based rules (optional).
    constitution_manager: Option<Arc<ConstitutionManager>>,
    /// Failure classifier for categorizing errors.
    failure_classifier: FailureClassifier,
    /// Failure histories per task/step.
    failure_histories: std::sync::Mutex<HashMap<String, FailureHistory>>,
    /// Failure policy for retry decisions.
    failure_policy: FailurePolicy,
}

impl WorkflowExecutor {
    /// Creates a new workflow executor.
    ///
    /// # Arguments
    /// * `orchestrator` - The agent orchestrator
    /// * `executor` - The agent executor
    /// * `monitoring` - Optional monitoring service for agent lifecycle tracking
    ///
    /// # Returns
    /// A new `WorkflowExecutor` instance.
    pub fn new(
        orchestrator: Arc<Orchestrator>,
        executor: Arc<AgentExecutor>,
        monitoring: Option<Arc<std::sync::Mutex<MonitoringService>>>,
    ) -> Self {
        // Try to initialize checkpoint manager from workspace
        let checkpoint_manager = Workspace::discover().ok().and_then(|ws| {
            CheckpointManager::new(ws.root()).ok().map(|cm| Arc::new(std::sync::Mutex::new(cm)))
        });

        // Initialize hook registry (optional - hooks can be registered later)
        let hook_registry = Some(Arc::new(HookRegistry::new()));

        // Try to initialize policy engine from workspace
        let (policy_engine, constitution_manager) = if let Ok(ws) = Workspace::discover() {
            let policy_file = ws.root().join(".radium").join("policy.toml");
            let constitution = Some(Arc::new(ConstitutionManager::new()));
            
            // Try to load policy engine from file, fallback to default
            let engine = if policy_file.exists() {
                PolicyEngine::from_file(&policy_file)
                    .map(|mut engine| {
                        // Set hook registry if available
                        if let Some(ref registry) = hook_registry {
                            engine.set_hook_registry(Arc::clone(registry));
                        }
                        Arc::new(std::sync::Mutex::new(engine))
                    })
                    .ok()
            } else {
                // Create default policy engine with Ask mode
                PolicyEngine::new(ApprovalMode::Ask)
                    .map(|mut engine| {
                        if let Some(ref registry) = hook_registry {
                            engine.set_hook_registry(Arc::clone(registry));
                        }
                        Arc::new(std::sync::Mutex::new(engine))
                    })
                    .ok()
            };
            
            (engine, constitution)
        } else {
            (None, None)
        };

        Self {
            engine: WorkflowEngine::new(orchestrator, executor),
            monitoring,
            checkpoint_manager,
            hook_registry,
            policy_engine,
            constitution_manager,
            failure_classifier: FailureClassifier::new(),
            failure_histories: std::sync::Mutex::new(HashMap::new()),
            failure_policy: FailurePolicy::default(),
        }
    }

    /// Get the hook registry for registering workflow behavior hooks.
    pub fn hook_registry(&self) -> Option<&Arc<HookRegistry>> {
        self.hook_registry.as_ref()
    }

    /// Get the policy engine for tool execution control.
    pub fn policy_engine(&self) -> Option<&Arc<std::sync::Mutex<PolicyEngine>>> {
        self.policy_engine.as_ref()
    }

    /// Get the constitution manager for session-based rules.
    pub fn constitution_manager(&self) -> Option<&Arc<ConstitutionManager>> {
        self.constitution_manager.as_ref()
    }

    /// Executes a workflow sequentially.
    ///
    /// Steps are executed in order based on `WorkflowStep.order`. Each step
    /// must complete before the next step begins. If a step fails, execution
    /// stops and the workflow state is set to `Error`.
    ///
    /// # Arguments
    /// * `workflow` - The workflow to execute (mutable reference)
    /// * `db` - Shared database access
    ///
    /// # Returns
    /// `Ok(ExecutionContext)` with execution results if successful, or
    /// `WorkflowEngineError` if execution failed.
    pub async fn execute_workflow(
        &self,
        workflow: &mut Workflow,
        db: Arc<std::sync::Mutex<crate::storage::Database>>,
    ) -> Result<ExecutionContext, WorkflowEngineError> {
        info!(
            workflow_id = %workflow.id,
            step_count = workflow.steps.len(),
            "Starting workflow execution"
        );

        // Validate workflow
        workflow.validate().map_err(|e| {
            error!(
                workflow_id = %workflow.id,
                error = %e,
                "Workflow validation failed"
            );
            WorkflowEngineError::Validation(e.to_string())
        })?;

        // Check if workflow is in a valid state to execute
        if !matches!(workflow.state, WorkflowState::Idle) {
            return Err(WorkflowEngineError::Validation(format!(
                "Workflow is not in Idle state: {:?}",
                workflow.state
            )));
        }

        // Create execution context
        let mut context = ExecutionContext::new(workflow.id.clone());

        // Sort steps by order
        let mut sorted_steps = workflow.steps.clone();
        sorted_steps.sort_by_key(|step| step.order);

        // Update workflow state to Running
        {
            let mut db_guard = db.lock().map_err(|e| {
                WorkflowEngineError::Storage(crate::storage::StorageError::InvalidData(
                    e.to_string(),
                ))
            })?;
            let mut workflow_repo = crate::storage::SqliteWorkflowRepository::new(&mut *db_guard);
            let running_state = WorkflowState::Running;
            self.engine.update_workflow_state(workflow, &running_state, &mut workflow_repo)?;
        }

        // Execute steps sequentially
        for (index, step) in sorted_steps.iter().enumerate() {
            context.current_step_index = index;

            // Check if step should execute based on conditions
            let condition = StepCondition::from_json(step.config_json.as_ref())
                .map_err(|e| WorkflowEngineError::Validation(e.to_string()))?;

            if !should_execute_step(&step.id, condition.as_ref(), &context)
                .map_err(|e| WorkflowEngineError::Validation(e.to_string()))?
            {
                debug!(
                    workflow_id = %workflow.id,
                    step_id = %step.id,
                    "Skipping step due to condition"
                );
                continue;
            }

            debug!(
                workflow_id = %workflow.id,
                step_id = %step.id,
                step_order = step.order,
                step_index = index,
                total_steps = sorted_steps.len(),
                "Executing workflow step"
            );

            // Execute the step
            // We need to load the task from DB, which requires a lock.
            // But execute_step is async (agent call), so we can't hold the lock across it.
            // Solution: Load task inside a block, then execute.

            let started_at = Utc::now();
            let step_result: Result<StepResult, WorkflowEngineError> = async {
                // 1. Load task (Sync DB access)
                let task = {
                    let mut db_guard = db.lock().map_err(|e| {
                        WorkflowEngineError::Storage(crate::storage::StorageError::InvalidData(
                            e.to_string(),
                        ))
                    })?;
                    let task_repo = crate::storage::SqliteTaskRepository::new(&mut *db_guard);
                    task_repo.get_by_id(&step.task_id).map_err(|e| match e {
                        crate::storage::StorageError::NotFound(_) => {
                            WorkflowEngineError::TaskNotFound(step.task_id.clone())
                        }
                        _ => WorkflowEngineError::Storage(e),
                    })?
                };

                // 2. Prepare execution (CPU bound)
                let agent = self
                    .engine
                    .orchestrator
                    .get_agent(&task.agent_id)
                    .await
                    .ok_or_else(|| WorkflowEngineError::AgentNotFound(task.agent_id.clone()))?;

                let input_str = match &task.input {
                    serde_json::Value::String(s) => s.clone(),
                    v => serde_json::to_string(v)
                        .map_err(|e| WorkflowEngineError::InvalidInput(e.to_string()))?,
                };

                // 2.5. Register agent with monitoring service (if available)
                let agent_id = task.agent_id.clone();
                let mut checkpoint_id: Option<String> = None;
                
                if let Some(ref monitoring) = self.monitoring {
                    let record = AgentRecord::new(agent_id.clone(), "workflow".to_string());
                    if let Ok(svc) = monitoring.lock() {
                        if let Err(e) = svc.register_agent(&record) {
                            debug!(
                                agent_id = %agent_id,
                                error = %e,
                                "Failed to register agent with monitoring service"
                            );
                        } else {
                            let _ = svc.update_status(&agent_id, AgentStatus::Running);
                        }
                    }
                }

                // 2.6. Create checkpoint before step execution (if checkpoint manager available)
                // This creates a snapshot before any file modifications
                if let Some(ref checkpoint_mgr) = self.checkpoint_manager {
                    if let Ok(cm) = checkpoint_mgr.lock() {
                        match cm.create_checkpoint(Some(format!("Before workflow step: {}", step.id))) {
                            Ok(checkpoint) => {
                                checkpoint_id = Some(checkpoint.id.clone());
                                debug!(
                                    workflow_id = %context.workflow_id,
                                    step_id = %step.id,
                                    checkpoint_id = %checkpoint.id,
                                    "Created checkpoint before step execution"
                                );
                            }
                            Err(e) => {
                                debug!(
                                    workflow_id = %context.workflow_id,
                                    step_id = %step.id,
                                    error = %e,
                                    "Failed to create checkpoint (workspace may not be a git repo)"
                                );
                            }
                        }
                    }
                }

                // 3. Execute Agent (Async, no DB lock)
                let execution_result = self
                    .engine
                    .executor
                    .execute_agent_with_default_model(agent, &input_str, None)
                    .await
                    .map_err(|e| WorkflowEngineError::Execution(e.to_string()))?;

                let completed_at = Utc::now();
                
                // 4. Record telemetry if available
                if let Some(ref monitoring) = self.monitoring {
                    if let Some(ref telemetry) = execution_result.telemetry {
                        use crate::monitoring::{TelemetryRecord, TelemetryTracking};
                        let mut record = TelemetryRecord::new(agent_id.clone())
                            .with_tokens(telemetry.input_tokens, telemetry.output_tokens);
                        
                        if let Some(ref model_id) = telemetry.model_id {
                            // Try to determine provider from model ID
                            let provider = if model_id.contains("gpt") {
                                "openai"
                            } else if model_id.contains("claude") {
                                "anthropic"
                            } else if model_id.contains("gemini") {
                                "google"
                            } else {
                                "unknown"
                            };
                            record = record.with_model(model_id.clone(), provider.to_string());
                        }
                        
                        record.calculate_cost();
                        
                        // Get hook registry before locking (to avoid holding lock across await)
                        let hook_registry = {
                            if let Ok(svc) = monitoring.lock() {
                                svc.get_hook_registry()
                            } else {
                                None
                            }
                        };
                        
                        // Execute hooks outside the lock if registry exists
                        let mut effective_record = record.clone();
                        if let Some(registry) = hook_registry {
                            use crate::hooks::registry::HookType;
                            use crate::hooks::types::HookContext;
                            let hook_context = HookContext::new(
                                "telemetry_collection",
                                serde_json::json!({
                                    "agent_id": record.agent_id,
                                    "input_tokens": record.input_tokens,
                                    "output_tokens": record.output_tokens,
                                    "total_tokens": record.total_tokens,
                                    "estimated_cost": record.estimated_cost,
                                    "model": record.model,
                                    "provider": record.provider,
                                }),
                            );
                            
                            if let Ok(results) = registry.execute_hooks(HookType::TelemetryCollection, &hook_context).await {
                                for result in results {
                                    if let Some(modified_data) = result.modified_data {
                                        if let Some(custom_fields) = modified_data.as_object() {
                                            if let Some(new_cost) = custom_fields.get("estimated_cost").and_then(|v| v.as_f64()) {
                                                effective_record.estimated_cost = new_cost;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        
                        // Now lock and write to database (synchronous, fast)
                        if let Ok(svc) = monitoring.lock() {
                            // Use a synchronous internal method to write
                            if let Err(e) = svc.record_telemetry_sync(&effective_record) {
                                debug!(
                                    agent_id = %agent_id,
                                    error = %e,
                                    "Failed to record telemetry"
                                );
                            }
                        }
                    }
                }
                
                // 5. Update monitoring status based on execution result
                if let Some(ref monitoring) = self.monitoring {
                    if let Ok(svc) = monitoring.lock() {
                        if execution_result.success {
                            let _ = svc.complete_agent(&agent_id, 0);
                        } else {
                            let error_msg = execution_result.error.as_deref()
                                .unwrap_or("Unknown error");
                            let _ = svc.fail_agent(&agent_id, error_msg);
                        }
                    }
                }
                
                // 6. Check for /restore command in agent output
                let mut restore_requested = false;
                let mut restore_checkpoint_id: Option<String> = None;
                
                if execution_result.success {
                    // Check if output contains /restore command
                    let output_text = match &execution_result.output {
                        radium_orchestrator::AgentOutput::Text(text) => Some(text.as_str()),
                        _ => None,
                    };
                    
                    if let Some(text) = output_text {
                        // Look for /restore command pattern
                        // Format: /restore <checkpoint-id> or /restore
                        if text.contains("/restore") {
                            restore_requested = true;
                            
                            // Try to extract checkpoint ID from text
                            // Pattern: /restore checkpoint-xxxxx or /restore <id>
                            // Simple string parsing instead of regex to avoid dependency
                            if let Some(restore_pos) = text.find("/restore") {
                                let after_restore = &text[restore_pos + "/restore".len()..];
                                let trimmed = after_restore.trim_start();
                                if !trimmed.is_empty() {
                                    // Extract first word after /restore
                                    let id = trimmed.split_whitespace().next().unwrap_or("").to_string();
                                    if !id.is_empty() {
                                        restore_checkpoint_id = Some(id);
                                    }
                                }
                            }
                            
                            // If no checkpoint ID specified, use the one created before this step
                            if restore_checkpoint_id.is_none() {
                                restore_checkpoint_id = checkpoint_id.clone();
                            }
                            
                            // Perform restore if checkpoint ID is available
                            if let Some(ref cp_id) = restore_checkpoint_id {
                                                if let Some(ref checkpoint_mgr) = self.checkpoint_manager {
                                                    if let Ok(cm) = checkpoint_mgr.lock() {                                        match cm.restore_checkpoint(cp_id) {
                                            Ok(()) => {
                                                info!(
                                                    workflow_id = %context.workflow_id,
                                                    step_id = %step.id,
                                                    checkpoint_id = %cp_id,
                                                    "Restored checkpoint per agent request"
                                                );
                                            }
                                            Err(e) => {
                                                error!(
                                                    workflow_id = %context.workflow_id,
                                                    step_id = %step.id,
                                                    checkpoint_id = %cp_id,
                                                    error = %e,
                                                    "Failed to restore checkpoint"
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                // Convert output
                if execution_result.success {
                    let output_value = match execution_result.output {
                        radium_orchestrator::AgentOutput::Text(text) => {
                            // If restore was requested, add note to output
                            if restore_requested {
                                let restore_note = if restore_checkpoint_id.is_some() {
                                    "\n\n[Checkpoint restored. You may need to re-propose tool calls.]".to_string()
                                } else {
                                    "\n\n[Restore requested but no checkpoint ID found.]".to_string()
                                };
                                serde_json::Value::String(format!("{}{}", text, restore_note))
                            } else {
                                serde_json::Value::String(text)
                            }
                        }
                        radium_orchestrator::AgentOutput::StructuredData(data) => data,
                        radium_orchestrator::AgentOutput::ToolCall { name, args } => {
                            serde_json::json!({
                                "type": "tool_call",
                                "name": name,
                                "args": args
                            })
                        }
                        radium_orchestrator::AgentOutput::Terminate => {
                            serde_json::Value::String("terminated".to_string())
                        }
                    };
                    Ok(StepResult::success(step.id.clone(), output_value, started_at, completed_at))
                } else {
                    let error_msg = execution_result
                        .error
                        .unwrap_or_else(|| "Unknown execution error".to_string());
                    Ok(StepResult::failure(step.id.clone(), error_msg, started_at, completed_at))
                }
            }
            .await;

            let step_result = match step_result {
                Ok(res) => res,
                Err(e) => {
                    let error_msg = e.to_string();
                    let completed_at = Utc::now();
                    StepResult::failure(step.id.clone(), error_msg, started_at, completed_at)
                }
            };

            // Record step result
            context.record_step_result(step.id.clone(), step_result.clone());

            // Check if step failed
            if !step_result.success {
                let mut error_msg =
                    step_result.error.unwrap_or_else(|| "Step execution failed".to_string());
                let error_type = "workflow_step_error";
                let error_source = Some("workflow_executor");

                // Execute error hooks if registry is available
                if let Some(ref registry) = self.hook_registry {
                    let hooks = OrchestratorHooks::new(Arc::clone(registry));
                    
                    // Try error interception first
                    if let Ok(Some(handled_message)) = hooks.error_interception(
                        &error_msg,
                        error_type,
                        error_source,
                    ).await {
                        error_msg = handled_message;
                        // Error was handled by hook, but we still need to stop execution
                        // as the step failed
                    } else {
                        // If not handled, try error transformation
                        if let Ok(Some(transformed_message)) = hooks.error_transformation(
                            &error_msg,
                            error_type,
                            error_source,
                        ).await {
                            error_msg = transformed_message;
                        }
                    }

                    // Execute error logging hooks (always execute, even if error was handled)
                    let error_context = crate::hooks::error_hooks::ErrorHookContext::logging(
                        error_msg.clone(),
                        error_type.to_string(),
                        error_source.map(|s| s.to_string()),
                    );
                    let hook_context = error_context.to_hook_context(
                        crate::hooks::error_hooks::ErrorHookType::Logging,
                    );
                    if let Err(e) = registry.execute_hooks(HookType::ErrorLogging, &hook_context).await {
                        tracing::warn!(
                            workflow_id = %workflow.id,
                            step_id = %step.id,
                            error = %e,
                            "Error logging hook execution failed"
                        );
                    }

                    // Try error recovery hooks
                    if let Ok(Some(recovered_message)) = hooks.error_recovery(
                        &error_msg,
                        error_type,
                        error_source,
                    ).await {
                        // If recovery succeeded, we might want to continue or retry
                        // For now, we'll use the recovered message but still stop execution
                        error_msg = recovered_message;
                    }
                }

                // Classify the failure
                let failure_type = self.failure_classifier.classify_from_string(&error_msg);

                // Record failure in history
                {
                    let mut histories = self.failure_histories.lock().unwrap();
                    let history = histories
                        .entry(step.id.clone())
                        .or_insert_with(|| FailureHistory::new(step.id.clone()));
                    history.add_failure(failure_type.clone(), error_msg.clone());
                }

                error!(
                    workflow_id = %workflow.id,
                    step_id = %step.id,
                    error = %error_msg,
                    failure_type = %failure_type.description(),
                    "Workflow step failed, stopping execution"
                );

                // Update workflow state to Error
                {
                    let mut db_guard = db.lock().map_err(|e| {
                        WorkflowEngineError::Storage(crate::storage::StorageError::InvalidData(
                            e.to_string(),
                        ))
                    })?;
                    let mut workflow_repo =
                        crate::storage::SqliteWorkflowRepository::new(&mut *db_guard);
                    let error_state = WorkflowState::Error(error_msg.clone());
                    self.engine.update_workflow_state(
                        workflow,
                        &error_state,
                        &mut workflow_repo,
                    )?;
                }

                return Err(WorkflowEngineError::Execution(error_msg));
            }

            info!(
                workflow_id = %workflow.id,
                step_id = %step.id,
                "Workflow step completed successfully"
            );

            // Check for vibecheck behavior and handle if present
            if let Some(workspace) = Workspace::discover().ok() {
                let ws_structure = WorkspaceStructure::new(workspace.root());
                let behavior_file = ws_structure.memory_dir().join("behavior.json");
                
                if behavior_file.exists() {
                    use crate::workflow::behaviors::vibe_check::{VibeCheckContext, VibeCheckEvaluator, WorkflowPhase};
                    use crate::workflow::behaviors::types::BehaviorAction;
                    
                    // Try to read behavior action
                    if let Ok(Some(action)) = BehaviorAction::read_from_file(&behavior_file) {
                        use crate::workflow::behaviors::types::BehaviorActionType;
                        if action.action == BehaviorActionType::VibeCheck {
                            debug!(
                                workflow_id = %workflow.id,
                                step_id = %step.id,
                                "VibeCheck behavior detected - oversight would be triggered here"
                            );
                            // Note: Full oversight integration requires MetacognitiveService,
                            // ContextManager, and ConstitutionManager which are not currently
                            // available in WorkflowExecutor. This is a placeholder for the
                            // integration point. The actual oversight call should be made via
                            // VibeCheckEvaluator::evaluate_with_oversight() when those services
                            // are available.
                        }
                    }
                }
            }

            // Execute workflow step hooks for behavior evaluation
            if let Some(ref registry) = self.hook_registry {
                if let Some(workspace) = Workspace::discover().ok() {
                    let ws_structure = WorkspaceStructure::new(workspace.root());
                    let behavior_file = ws_structure.memory_dir().join("behavior.json");
                    
                    // Extract output text for behavior evaluation
                    let output_text = match step_result.output.as_ref() {
                        Some(serde_json::Value::String(s)) => s.clone(),
                        Some(v) => serde_json::to_string(v).unwrap_or_default(),
                        None => String::new(),
                    };

                    // Create hook context with behavior file and output
                    let hook_data = serde_json::json!({
                        "behavior_file": behavior_file.to_string_lossy().to_string(),
                        "output": output_text,
                        "step_id": step.id.clone(),
                        "workflow_id": workflow.id.clone(),
                        "step_result": serde_json::to_value(&step_result).unwrap_or_default(),
                    });
                    let hook_context = HookContext::new("workflow_step", hook_data);

                    // Execute hooks for workflow step completion
                    // Note: Behavior hooks can be registered via BehaviorEvaluatorAdapter
                    // For now, we just create the context - hooks will be executed when registered
                    // This maintains backward compatibility while enabling hook-based behavior evaluation
                    debug!(
                        workflow_id = %workflow.id,
                        step_id = %step.id,
                        behavior_file = %behavior_file.display(),
                        "Workflow step hook context prepared"
                    );
                }
            }
        }

        // All steps completed successfully
        context.completed_at = Some(chrono::Utc::now());
        context.current_step_index = sorted_steps.len();

        // Update workflow state to Completed
        {
            let mut db_guard = db.lock().map_err(|e| {
                WorkflowEngineError::Storage(crate::storage::StorageError::InvalidData(
                    e.to_string(),
                ))
            })?;
            let mut workflow_repo = crate::storage::SqliteWorkflowRepository::new(&mut *db_guard);
            let completed_state = WorkflowState::Completed;
            self.engine.update_workflow_state(workflow, &completed_state, &mut workflow_repo)?;
        }

        info!(
            workflow_id = %workflow.id,
            step_count = sorted_steps.len(),
            duration_ms = context.completed_at
                .map_or(0, |completed| completed
                    .signed_duration_since(context.started_at)
                    .num_milliseconds()),
            "Workflow execution completed successfully"
        );

        Ok(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Task, WorkflowStep};
    use crate::storage::repositories::{TaskRepository, WorkflowRepository};
    use crate::storage::{Database, SqliteTaskRepository, SqliteWorkflowRepository};
    use radium_orchestrator::AgentExecutor;
    use radium_orchestrator::{Orchestrator, SimpleAgent};
    use serde_json::json;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn test_execute_workflow_sequential() {
        // Setup - use Arc<Mutex<Database>> to match production API
        let db = Arc::new(Mutex::new(Database::open_in_memory().unwrap()));
        let orchestrator = Arc::new(Orchestrator::new());
        let executor = Arc::new(AgentExecutor::with_mock_model());
        let workflow_executor =
            WorkflowExecutor::new(Arc::clone(&orchestrator), Arc::clone(&executor), None);

        // Register an agent
        let agent = Arc::new(SimpleAgent::new("test-agent".to_string(), "Test agent".to_string()));
        orchestrator.register_agent(agent).await;

        // Create tasks and workflow
        {
            let mut db_lock = db.lock().unwrap();
            let mut task_repo = SqliteTaskRepository::new(&mut db_lock);
            let task1 = Task::new(
                "task-1".to_string(),
                "Task 1".to_string(),
                "First task".to_string(),
                "test-agent".to_string(),
                json!({"input": "test1"}),
            );
            let task2 = Task::new(
                "task-2".to_string(),
                "Task 2".to_string(),
                "Second task".to_string(),
                "test-agent".to_string(),
                json!({"input": "test2"}),
            );
            task_repo.create(&task1).unwrap();
            task_repo.create(&task2).unwrap();

            let mut workflow_repo = SqliteWorkflowRepository::new(&mut db_lock);
            let mut workflow = crate::models::Workflow::new(
                "workflow-1".to_string(),
                "Test Workflow".to_string(),
                "A test workflow".to_string(),
            );
            workflow
                .add_step(WorkflowStep::new(
                    "step-1".to_string(),
                    "Step 1".to_string(),
                    "First step".to_string(),
                    "task-1".to_string(),
                    0,
                ))
                .unwrap();
            workflow
                .add_step(WorkflowStep::new(
                    "step-2".to_string(),
                    "Step 2".to_string(),
                    "Second step".to_string(),
                    "task-2".to_string(),
                    1,
                ))
                .unwrap();
            workflow_repo.create(&workflow).unwrap();
        }

        // Execute workflow with new API
        let mut workflow = {
            let mut db_lock = db.lock().unwrap();
            let workflow_repo = SqliteWorkflowRepository::new(&mut db_lock);
            workflow_repo.get_by_id("workflow-1").unwrap()
        };

        let context =
            workflow_executor.execute_workflow(&mut workflow, Arc::clone(&db)).await.unwrap();

        // Verify results
        assert_eq!(context.workflow_id, "workflow-1");
        assert_eq!(context.step_results.len(), 2);
        assert!(context.step_results.get("step-1").unwrap().success);
        assert!(context.step_results.get("step-2").unwrap().success);

        // Verify workflow state
        {
            let mut db_lock = db.lock().unwrap();
            let workflow_repo = SqliteWorkflowRepository::new(&mut db_lock);
            let workflow = workflow_repo.get_by_id("workflow-1").unwrap();
            assert_eq!(workflow.state, WorkflowState::Completed);
        }
    }

    #[tokio::test]
    async fn test_execute_workflow_single_step() {
        let db = Arc::new(Mutex::new(Database::open_in_memory().unwrap()));
        let orchestrator = Arc::new(Orchestrator::new());
        let executor = Arc::new(AgentExecutor::with_mock_model());
        let workflow_executor =
            WorkflowExecutor::new(Arc::clone(&orchestrator), Arc::clone(&executor), None);

        // Register an agent
        let agent = Arc::new(SimpleAgent::new("test-agent".to_string(), "Test agent".to_string()));
        orchestrator.register_agent(agent).await;

        // Create task and workflow
        {
            let mut db_lock = db.lock().unwrap();
            let mut task_repo = SqliteTaskRepository::new(&mut db_lock);
            let task = Task::new(
                "task-1".to_string(),
                "Task 1".to_string(),
                "Single task".to_string(),
                "test-agent".to_string(),
                json!({"input": "test"}),
            );
            task_repo.create(&task).unwrap();

            let mut workflow_repo = SqliteWorkflowRepository::new(&mut db_lock);
            let mut workflow = crate::models::Workflow::new(
                "workflow-1".to_string(),
                "Single Step Workflow".to_string(),
                "A workflow with one step".to_string(),
            );
            workflow
                .add_step(WorkflowStep::new(
                    "step-1".to_string(),
                    "Step 1".to_string(),
                    "Only step".to_string(),
                    "task-1".to_string(),
                    0,
                ))
                .unwrap();
            workflow_repo.create(&workflow).unwrap();
        }

        // Execute workflow
        let mut workflow = {
            let mut db_lock = db.lock().unwrap();
            let workflow_repo = SqliteWorkflowRepository::new(&mut db_lock);
            workflow_repo.get_by_id("workflow-1").unwrap()
        };

        let context =
            workflow_executor.execute_workflow(&mut workflow, Arc::clone(&db)).await.unwrap();

        assert_eq!(context.step_results.len(), 1);
        assert!(context.step_results.get("step-1").unwrap().success);
    }

    #[tokio::test]
    async fn test_execute_workflow_empty_workflow() {
        let db = Arc::new(Mutex::new(Database::open_in_memory().unwrap()));
        let orchestrator = Arc::new(Orchestrator::new());
        let executor = Arc::new(AgentExecutor::with_mock_model());
        let workflow_executor =
            WorkflowExecutor::new(Arc::clone(&orchestrator), Arc::clone(&executor), None);

        // Create workflow with no steps (valid - will complete immediately)
        {
            let mut db_lock = db.lock().unwrap();
            let mut workflow_repo = SqliteWorkflowRepository::new(&mut db_lock);
            let workflow = crate::models::Workflow::new(
                "workflow-1".to_string(),
                "Empty Workflow".to_string(),
                "A workflow with no steps".to_string(),
            );
            workflow_repo.create(&workflow).unwrap();
        }

        let mut workflow = {
            let mut db_lock = db.lock().unwrap();
            let workflow_repo = SqliteWorkflowRepository::new(&mut db_lock);
            workflow_repo.get_by_id("workflow-1").unwrap()
        };

        let result = workflow_executor.execute_workflow(&mut workflow, Arc::clone(&db)).await;

        // Empty workflow should complete successfully with no steps
        assert!(result.is_ok());
        let context = result.unwrap();
        assert_eq!(context.step_results.len(), 0);
        assert_eq!(workflow.state, WorkflowState::Completed);
    }

    #[tokio::test]
    async fn test_execute_workflow_invalid_state() {
        let db = Arc::new(Mutex::new(Database::open_in_memory().unwrap()));
        let orchestrator = Arc::new(Orchestrator::new());
        let executor = Arc::new(AgentExecutor::with_mock_model());
        let workflow_executor =
            WorkflowExecutor::new(Arc::clone(&orchestrator), Arc::clone(&executor), None);

        // Create workflow and set it to Running state
        {
            let mut db_lock = db.lock().unwrap();
            let mut workflow_repo = SqliteWorkflowRepository::new(&mut db_lock);
            let mut workflow = crate::models::Workflow::new(
                "workflow-1".to_string(),
                "Running Workflow".to_string(),
                "A workflow already running".to_string(),
            );
            workflow.set_state(WorkflowState::Running);
            workflow_repo.create(&workflow).unwrap();
        }

        let mut workflow = {
            let mut db_lock = db.lock().unwrap();
            let workflow_repo = SqliteWorkflowRepository::new(&mut db_lock);
            workflow_repo.get_by_id("workflow-1").unwrap()
        };

        let result = workflow_executor.execute_workflow(&mut workflow, Arc::clone(&db)).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            WorkflowEngineError::Validation(_) => {}
            _ => panic!("Expected Validation error for invalid state"),
        }
    }

    #[tokio::test]
    async fn test_execute_workflow_with_dependencies() {
        // Test workflow with steps that have dependencies
        let db = Arc::new(Mutex::new(Database::open_in_memory().unwrap()));
        let orchestrator = Arc::new(Orchestrator::new());
        let executor = Arc::new(AgentExecutor::with_mock_model());
        let workflow_executor =
            WorkflowExecutor::new(Arc::clone(&orchestrator), Arc::clone(&executor), None);

        // Register an agent
        let agent = Arc::new(SimpleAgent::new("test-agent".to_string(), "Test agent".to_string()));
        orchestrator.register_agent(agent).await;

        // Create tasks and workflow
        {
            let mut db_lock = db.lock().unwrap();
            let mut task_repo = SqliteTaskRepository::new(&mut db_lock);
            for i in 0..3 {
                let task = Task::new(
                    format!("task-{}", i),
                    format!("Task {}", i),
                    format!("Test task {}", i),
                    "test-agent".to_string(),
                    json!({"input": format!("test-{}", i)}),
                );
                task_repo.create(&task).unwrap();
            }

            let mut workflow_repo = SqliteWorkflowRepository::new(&mut db_lock);
            let mut workflow = crate::models::Workflow::new(
                "workflow-1".to_string(),
                "Dependency Workflow".to_string(),
                "A workflow with dependencies".to_string(),
            );

            // Step 1: No dependencies
            workflow
                .add_step(WorkflowStep::new(
                    "step-1".to_string(),
                    "Step 1".to_string(),
                    "First step".to_string(),
                    "task-0".to_string(),
                    0,
                ))
                .unwrap();

            // Step 2: Depends on step-1
            let mut step2 = WorkflowStep::new(
                "step-2".to_string(),
                "Step 2".to_string(),
                "Second step".to_string(),
                "task-1".to_string(),
                1,
            );
            step2.config_json = Some(
                serde_json::to_string(&serde_json::json!({
                    "dependsOn": ["step-1"]
                }))
                .unwrap(),
            );
            workflow.add_step(step2).unwrap();

            // Step 3: Depends on step-2
            let mut step3 = WorkflowStep::new(
                "step-3".to_string(),
                "Step 3".to_string(),
                "Third step".to_string(),
                "task-2".to_string(),
                2,
            );
            step3.config_json = Some(
                serde_json::to_string(&serde_json::json!({
                    "dependsOn": ["step-2"]
                }))
                .unwrap(),
            );
            workflow.add_step(step3).unwrap();

            workflow_repo.create(&workflow).unwrap();
        }

        // Execute workflow
        let mut workflow = {
            let mut db_lock = db.lock().unwrap();
            let workflow_repo = SqliteWorkflowRepository::new(&mut db_lock);
            workflow_repo.get_by_id("workflow-1").unwrap()
        };

        let context =
            workflow_executor.execute_workflow(&mut workflow, Arc::clone(&db)).await.unwrap();

        // Verify all steps executed in order
        assert_eq!(context.step_results.len(), 3);
        assert!(context.step_results.get("step-1").unwrap().success);
        assert!(context.step_results.get("step-2").unwrap().success);
        assert!(context.step_results.get("step-3").unwrap().success);
    }

    #[tokio::test]
    async fn test_execute_workflow_agent_execution_failure() {
        // Test workflow where agent execution fails mid-workflow
        let db = Arc::new(Mutex::new(Database::open_in_memory().unwrap()));
        let orchestrator = Arc::new(Orchestrator::new());
        let executor = Arc::new(AgentExecutor::with_mock_model());
        let workflow_executor =
            WorkflowExecutor::new(Arc::clone(&orchestrator), Arc::clone(&executor), None);

        // Register an agent
        let agent = Arc::new(SimpleAgent::new("test-agent".to_string(), "Test agent".to_string()));
        orchestrator.register_agent(agent).await;

        // Create tasks and workflow
        {
            let mut db_lock = db.lock().unwrap();
            let mut task_repo = SqliteTaskRepository::new(&mut db_lock);
            let task1 = Task::new(
                "task-1".to_string(),
                "Task 1".to_string(),
                "First task".to_string(),
                "test-agent".to_string(),
                json!({"input": "test1"}),
            );
            task_repo.create(&task1).unwrap();

            // Create task with non-existent agent to trigger failure
            let task2 = Task::new(
                "task-2".to_string(),
                "Task 2".to_string(),
                "Second task".to_string(),
                "nonexistent-agent".to_string(),
                json!({"input": "test2"}),
            );
            task_repo.create(&task2).unwrap();

            let mut workflow_repo = SqliteWorkflowRepository::new(&mut db_lock);
            let mut workflow = crate::models::Workflow::new(
                "workflow-1".to_string(),
                "Failure Workflow".to_string(),
                "A workflow with a failing step".to_string(),
            );
            workflow
                .add_step(WorkflowStep::new(
                    "step-1".to_string(),
                    "Step 1".to_string(),
                    "First step".to_string(),
                    "task-1".to_string(),
                    0,
                ))
                .unwrap();
            workflow
                .add_step(WorkflowStep::new(
                    "step-2".to_string(),
                    "Step 2".to_string(),
                    "Second step".to_string(),
                    "task-2".to_string(),
                    1,
                ))
                .unwrap();
            workflow_repo.create(&workflow).unwrap();
        }

        // Execute workflow - should fail on step 2
        let mut workflow = {
            let mut db_lock = db.lock().unwrap();
            let workflow_repo = SqliteWorkflowRepository::new(&mut db_lock);
            workflow_repo.get_by_id("workflow-1").unwrap()
        };

        let result = workflow_executor.execute_workflow(&mut workflow, Arc::clone(&db)).await;

        // Should fail because agent not found for step 2
        assert!(result.is_err());

        // Verify workflow is in error state
        {
            let mut db_lock = db.lock().unwrap();
            let workflow_repo = SqliteWorkflowRepository::new(&mut db_lock);
            let workflow = workflow_repo.get_by_id("workflow-1").unwrap();
            assert!(matches!(workflow.state, WorkflowState::Error(_)));
        }
    }
}
