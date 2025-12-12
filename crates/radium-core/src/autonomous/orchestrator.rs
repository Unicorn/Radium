//! Autonomous execution orchestrator.
//!
//! Coordinates all autonomous capabilities for end-to-end execution from
//! high-level goals to completion with self-healing.

use crate::agents::registry::AgentRegistry;
use crate::checkpoint::CheckpointManager;
use crate::learning::store::LearningStore;
use crate::learning::recovery_learning::RecoveryLearning;
use crate::planning::{AutonomousPlanner, PlanningError};
use crate::workflow::engine::ExecutionContext;
use crate::workflow::executor::WorkflowExecutor;
use crate::workflow::failure::FailurePolicy;
use crate::workflow::recovery::RecoveryManager;
use crate::workflow::reassignment::{AgentReassignment, AgentSelector};
use crate::workflow::service::WorkflowService;
use crate::workflow::templates::WorkflowTemplate;
use crate::workspace::Workspace;
use radium_abstraction::Model;
use radium_orchestrator::{AgentExecutor, Orchestrator, TaskDispatcher, TaskDispatcherConfig};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, warn};

/// Errors that can occur during autonomous execution.
#[derive(Debug, Error)]
pub enum AutonomousError {
    /// Planning error.
    #[error("Planning error: {0}")]
    Planning(#[from] PlanningError),

    /// Workflow execution error.
    #[error("Workflow execution error: {0}")]
    WorkflowExecution(String),

    /// Recovery error.
    #[error("Recovery error: {0}")]
    Recovery(String),

    /// Reassignment error.
    #[error("Reassignment error: {0}")]
    Reassignment(String),

    /// Learning error.
    #[error("Learning error: {0}")]
    Learning(String),

    /// Workspace error.
    #[error("Workspace error: {0}")]
    Workspace(String),
}

/// Result type for autonomous operations.
pub type Result<T> = std::result::Result<T, AutonomousError>;

/// Configuration for autonomous execution.
#[derive(Debug, Clone)]
pub struct AutonomousConfig {
    /// Maximum number of retries.
    pub max_retries: u32,
    /// Enable automatic recovery.
    pub enable_recovery: bool,
    /// Enable agent reassignment.
    pub enable_reassignment: bool,
    /// Enable learning optimization.
    pub enable_learning: bool,
    /// Checkpoint frequency.
    pub checkpoint_frequency: CheckpointFrequency,
    /// Maximum concurrent tasks per agent for dispatcher.
    pub max_concurrent_per_agent: usize,
    /// Poll interval for dispatcher (milliseconds).
    pub dispatcher_poll_interval_ms: u64,
}

impl Default for AutonomousConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            enable_recovery: true,
            enable_reassignment: true,
            enable_learning: true,
            checkpoint_frequency: CheckpointFrequency::EveryStep,
            max_concurrent_per_agent: 10,
            dispatcher_poll_interval_ms: 100,
        }
    }
}

/// Checkpoint frequency for autonomous execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckpointFrequency {
    /// Create checkpoint before every step.
    EveryStep,
    /// Create checkpoint before every iteration.
    EveryIteration,
    /// Create checkpoint only on failure.
    OnFailure,
    /// Create checkpoint at a fixed time interval.
    TimeInterval(Duration),
}

/// Execution result for autonomous execution.
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Whether execution succeeded.
    pub success: bool,
    /// Workflow ID that was executed.
    pub workflow_id: String,
    /// Execution context with results.
    pub context: ExecutionContext,
    /// Number of steps completed.
    pub steps_completed: u32,
    /// Number of steps that failed.
    pub steps_failed: u32,
    /// Number of recoveries performed.
    pub recoveries_performed: u32,
    /// Number of reassignments performed.
    pub reassignments_performed: u32,
    /// Error message if execution failed.
    pub error: Option<String>,
}

/// Monitor for tracking execution progress.
#[derive(Debug, Clone)]
pub struct ExecutionMonitor {
    /// Workflow ID being monitored.
    pub workflow_id: String,
    /// Total number of steps.
    pub total_steps: u32,
    /// Number of completed steps.
    pub completed_steps: u32,
    /// Number of failed steps.
    pub failed_steps: u32,
    /// Number of recovered steps.
    pub recovered_steps: u32,
    /// Current step ID (if executing).
    pub current_step: Option<String>,
}

impl ExecutionMonitor {
    /// Creates a new execution monitor.
    pub fn new(workflow_id: String, total_steps: u32) -> Self {
        Self {
            workflow_id,
            total_steps,
            completed_steps: 0,
            failed_steps: 0,
            recovered_steps: 0,
            current_step: None,
        }
    }

    /// Gets the progress percentage (0.0-100.0).
    pub fn get_progress(&self) -> f32 {
        if self.total_steps == 0 {
            0.0
        } else {
            (self.completed_steps as f32 / self.total_steps as f32) * 100.0
        }
    }

    /// Gets a status summary string.
    pub fn get_status_summary(&self) -> String {
        format!(
            "Progress: {:.1}% ({}/{} steps completed, {} failed, {} recovered)",
            self.get_progress(),
            self.completed_steps,
            self.total_steps,
            self.failed_steps,
            self.recovered_steps
        )
    }
}

/// Autonomous orchestrator coordinating all components.
pub struct AutonomousOrchestrator {
    /// Autonomous planner for goal decomposition.
    planner: AutonomousPlanner,
    /// Workflow executor.
    executor: WorkflowExecutor,
    /// Workflow service.
    workflow_service: WorkflowService,
    /// Database for workflow and task storage.
    db: Arc<std::sync::Mutex<crate::storage::Database>>,
    /// Recovery manager (optional).
    recovery_manager: Option<RecoveryManager>,
    /// Agent reassignment (optional).
    reassignment: Option<AgentReassignment>,
    /// Recovery learning (optional).
    learning: Option<Arc<Mutex<RecoveryLearning>>>,
    /// Configuration.
    config: AutonomousConfig,
    /// Execution monitor.
    monitor: Arc<Mutex<ExecutionMonitor>>,
    /// Task dispatcher for autonomous execution.
    dispatcher: Option<Arc<Mutex<TaskDispatcher>>>,
}

impl AutonomousOrchestrator {
    /// Creates a new autonomous orchestrator.
    ///
    /// # Arguments
    /// * `orchestrator` - The agent orchestrator
    /// * `executor` - The agent executor
    /// * `db` - The database
    /// * `agent_registry` - The agent registry
    /// * `config` - Autonomous configuration
    ///
    /// # Returns
    /// A new orchestrator instance
    pub fn new(
        orchestrator: &Arc<Orchestrator>,
        executor: &Arc<AgentExecutor>,
        db: &Arc<std::sync::Mutex<crate::storage::Database>>,
        agent_registry: Arc<AgentRegistry>,
        config: AutonomousConfig,
    ) -> Result<Self> {
        // Initialize workflow service
        let workflow_service = WorkflowService::new(orchestrator, executor, db);

        // Initialize workflow executor
        let workflow_executor = WorkflowExecutor::new(
            Arc::clone(orchestrator),
            Arc::clone(executor),
            workflow_service.monitoring.clone(),
        );

        // Initialize recovery manager if enabled
        let recovery_manager = if config.enable_recovery {
            Workspace::discover()
                .ok()
                .and_then(|ws| {
                    CheckpointManager::new(ws.root())
                        .ok()
                        .map(|cm| {
                            RecoveryManager::new(
                                Arc::new(Mutex::new(cm)),
                                FailurePolicy::default(),
                            )
                        })
                })
        } else {
            None
        };

        // Initialize agent reassignment if enabled
        let reassignment = if config.enable_reassignment {
            let selector = AgentSelector::new(agent_registry.clone());
            Some(AgentReassignment::new(selector, Some(2)))
        } else {
            None
        };

        // Initialize learning if enabled
        let learning = if config.enable_learning {
            Workspace::discover()
                .ok()
                .and_then(|ws| {
                    let learning_path = ws.radium_dir().join("learning");
                    LearningStore::new(learning_path)
                        .ok()
                        .map(|store| {
                            Arc::new(Mutex::new(RecoveryLearning::new(Arc::new(Mutex::new(store)))))
                        })
                })
        } else {
            None
        };

        // Initialize planner
        let planner = AutonomousPlanner::new(agent_registry.clone());

        // Initialize monitor
        let monitor = Arc::new(Mutex::new(ExecutionMonitor::new(
            "pending".to_string(),
            0,
        )));

        // Initialize task dispatcher
        let dispatcher_config = TaskDispatcherConfig {
            poll_interval: Duration::from_millis(config.dispatcher_poll_interval_ms),
            max_concurrent_per_agent: config.max_concurrent_per_agent,
        };
        let dispatcher = TaskDispatcher::new(
            orchestrator.registry(),
            orchestrator.queue(),
            orchestrator.executor(),
            dispatcher_config,
        );
        let dispatcher = Some(Arc::new(Mutex::new(dispatcher)));

        Ok(Self {
            planner,
            executor: workflow_executor,
            workflow_service,
            db: Arc::clone(db),
            recovery_manager,
            reassignment,
            learning,
            config,
            monitor,
            dispatcher,
        })
    }

    /// Executes autonomously from a high-level goal.
    ///
    /// # Arguments
    /// * `goal` - The high-level goal description
    /// * `model` - The model to use for planning and execution
    ///
    /// # Returns
    /// Execution result with completion status
    ///
    /// # Errors
    /// Returns error if execution fails
    pub async fn execute_autonomous(
        &self,
        goal: &str,
        model: Arc<dyn Model>,
    ) -> Result<ExecutionResult> {
        use tracing::{error, info};
        use crate::storage::{TaskRepository, WorkflowRepository};

        info!(goal = %goal, "Starting autonomous execution");

        // Step 1: Decompose goal into workflow
        let autonomous_plan = self.planner.plan_from_goal(goal, model.clone()).await?;
        let workflow_template = &autonomous_plan.workflow;

        info!(
            project = %autonomous_plan.plan.project_name,
            iterations = autonomous_plan.plan.iterations.len(),
            "Plan generated successfully"
        );

        // Step 2: Create workflow in database
        // Note: This would require WorkflowService to have a create_from_template method
        // For now, we'll use a simplified approach
        let workflow_id = uuid::Uuid::new_v4().to_string();

        // Step 3: Initialize monitor
        {
            let mut monitor = self.monitor.lock().unwrap();
            *monitor = ExecutionMonitor::new(
                workflow_id.clone(),
                workflow_template.steps.len() as u32,
            );
        }

        // Step 4: Convert workflow template to executable workflow
        let db = Arc::clone(&self.db);

        let mut workflow = self.convert_template_to_workflow(
            workflow_template,
            &workflow_id,
        ).await.map_err(|e| {
            error!(
                workflow_id = %workflow_id,
                error = %e,
                "Failed to convert workflow template"
            );
            AutonomousError::WorkflowExecution(format!("Template conversion failed: {}", e))
        })?;

        // Store workflow and tasks in database
        {
            let mut db_guard = db.lock().map_err(|e| {
                AutonomousError::WorkflowExecution(format!("Database lock failed: {}", e))
            })?;

            // Create tasks
            let task_repo = crate::storage::SqliteTaskRepository::new(&mut *db_guard);
            for step in &workflow.steps {
                // Task should already be created by convert_template_to_workflow
                // but verify it exists
                if task_repo.get_by_id(&step.task_id).is_err() {
                    error!(
                        workflow_id = %workflow_id,
                        step_id = %step.id,
                        task_id = %step.task_id,
                        "Task not found for workflow step"
                    );
                    return Err(AutonomousError::WorkflowExecution(
                        format!("Task {} not found", step.task_id)
                    ));
                }
            }

            // Create workflow
            let mut workflow_repo = crate::storage::SqliteWorkflowRepository::new(&mut *db_guard);
            workflow_repo.create(&workflow).map_err(|e| {
                error!(
                    workflow_id = %workflow_id,
                    error = %e,
                    "Failed to create workflow in database"
                );
                AutonomousError::WorkflowExecution(format!("Workflow creation failed: {}", e))
            })?;
        }

        info!(
            workflow_id = %workflow_id,
            step_count = workflow.steps.len(),
            "Workflow created, starting execution"
        );

        // Step 5: Start task dispatcher
        if let Some(ref dispatcher) = self.dispatcher {
            let mut dispatcher_guard = dispatcher.lock().unwrap();
            if let Err(e) = dispatcher_guard.start() {
                error!(
                    workflow_id = %workflow_id,
                    error = %e,
                    "Failed to start task dispatcher"
                );
                return Err(AutonomousError::WorkflowExecution(
                    format!("Failed to start task dispatcher: {}", e)
                ));
            }
            info!("Task dispatcher started");

            // Set up progress reporting bridge
            let progress_reporter = dispatcher_guard.progress_reporter();
            let monitor_clone = Arc::clone(&self.monitor);
            let workflow_id_clone = workflow_id.clone();
            let mut progress_rx = progress_reporter.subscribe();

            // Spawn task to bridge progress events to ExecutionMonitor
            tokio::spawn(async move {
                while let Ok(event) = progress_rx.recv().await {
                    let mut monitor = monitor_clone.lock().unwrap();
                    match event {
                        radium_orchestrator::ProgressEvent::TaskCompleted { .. } => {
                            monitor.completed_steps += 1;
                        }
                        radium_orchestrator::ProgressEvent::TaskFailed { .. } => {
                            monitor.failed_steps += 1;
                        }
                        _ => {}
                    }
                }
            });
        }

        // Step 5.5: Setup time-based checkpointing if configured
        let (checkpoint_cancel_tx, checkpoint_cancel_rx) = tokio::sync::oneshot::channel::<()>();
        
        if let CheckpointFrequency::TimeInterval(interval) = &self.config.checkpoint_frequency {
            let interval = *interval;
            let workflow_id_clone = workflow_id.clone();
            
            // Get or create CheckpointManager
            let checkpoint_manager = Workspace::discover()
                .ok()
                .and_then(|ws| CheckpointManager::new(ws.root()).ok());

            if let Some(manager) = checkpoint_manager {
                let manager = Arc::new(Mutex::new(manager));
                let manager_clone = Arc::clone(&manager);
                let mut cancel_rx = checkpoint_cancel_rx;
                
                // Spawn timer task
                tokio::spawn(async move {
                    let mut interval_timer = tokio::time::interval(interval);
                    interval_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                    
                    // Skip the first immediate tick
                    interval_timer.tick().await;
                    
                    loop {
                        tokio::select! {
                            _ = interval_timer.tick() => {
                                // Create checkpoint
                                if let Ok(mgr) = manager_clone.lock() {
                                    let description = format!(
                                        "Time-based checkpoint for workflow: {}",
                                        workflow_id_clone
                                    );
                                    match mgr.create_checkpoint(Some(description)) {
                                        Ok(checkpoint) => {
                                            info!(
                                                workflow_id = %workflow_id_clone,
                                                checkpoint_id = %checkpoint.id,
                                                interval_secs = interval.as_secs(),
                                                "Created time-based checkpoint"
                                            );
                                        }
                                        Err(e) => {
                                            debug!(
                                                workflow_id = %workflow_id_clone,
                                                error = %e,
                                                "Failed to create time-based checkpoint"
                                            );
                                        }
                                    }
                                }
                            }
                            _ = &mut cancel_rx => {
                                info!(
                                    workflow_id = %workflow_id_clone,
                                    "Time-based checkpoint timer stopped"
                                );
                                break;
                            }
                        }
                    }
                });
            }
        }

        // Step 6: Execute workflow with monitoring
        let mut steps_completed = 0;
        let mut steps_failed = 0;
        let mut recoveries_performed = 0;
        let mut reassignments_performed = 0;
        let mut execution_error: Option<String> = None;

        let context = match self.executor.execute_workflow(&mut workflow, Arc::clone(&db)).await {
            Ok(ctx) => {
                steps_completed = ctx.step_results.values().filter(|r| r.success).count() as u32;
                steps_failed = ctx.step_results.values().filter(|r| !r.success).count() as u32;

                info!(
                    workflow_id = %workflow_id,
                    steps_completed,
                    steps_failed,
                    "Workflow execution completed successfully"
                );
                ctx
            }
            Err(e) => {
                error!(
                    workflow_id = %workflow_id,
                    error = %e,
                    "Workflow execution failed"
                );

                // Try to recover or reassign if enabled
                let mut _recovered = false;

                // Attempt recovery if enabled
                if let Some(ref recovery_manager) = self.recovery_manager {
                    if let Ok(recovery_ctx) = self.attempt_recovery(
                        &workflow_id,
                        recovery_manager,
                        Arc::clone(&db),
                    ).await {
                        recoveries_performed += 1;
                        _recovered = true;
                        info!(
                            workflow_id = %workflow_id,
                            "Recovery successful"
                        );
                        recovery_ctx
                    } else {
                        // Recovery failed, try reassignment if enabled
                        if let Some(ref reassignment) = self.reassignment {
                            if let Ok(reassignment_ctx) = self.attempt_reassignment(
                                &workflow,
                                reassignment,
                                Arc::clone(&db),
                            ).await {
                                reassignments_performed += 1;
                                _recovered = true;
                                info!(
                                    workflow_id = %workflow_id,
                                    "Reassignment successful"
                                );
                                reassignment_ctx
                            } else {
                                execution_error = Some(e.to_string());
                                ExecutionContext::new(workflow_id.clone())
                            }
                        } else {
                            execution_error = Some(e.to_string());
                            ExecutionContext::new(workflow_id.clone())
                        }
                    }
                } else {
                    execution_error = Some(e.to_string());
                    ExecutionContext::new(workflow_id.clone())
                }
            }
        };

        // Stop time-based checkpoint timer
        let _ = checkpoint_cancel_tx.send(());

        // Step 6: Record learning data if enabled
        // TODO: Re-enable learning once method visibility issues are resolved
        // if let Some(ref learning) = self.learning {
        //     if let Ok(mut learning_guard) = learning.lock() {
        //         for (_step_id, result) in &context.step_results {
        //             if !result.success {
        //                 let strategy = if recoveries_performed > 0 {
        //                     "recovery_attempted"
        //                 } else {
        //                     "no_recovery"
        //                 };
        //                 learning_guard.record_failure(strategy);
        //             }
        //         }
        //     }
        // }

        // Update final monitor status
        {
            let mut monitor = self.monitor.lock().unwrap();
            monitor.completed_steps = steps_completed;
            monitor.failed_steps = steps_failed;
            monitor.recovered_steps = recoveries_performed;
        }

        // Step 7: Stop task dispatcher
        if let Some(ref dispatcher) = self.dispatcher {
            let mut dispatcher_guard = dispatcher.lock().unwrap();
            if let Err(e) = dispatcher_guard.stop() {
                warn!(
                    workflow_id = %workflow_id,
                    error = %e,
                    "Failed to stop task dispatcher"
                );
            } else {
                info!("Task dispatcher stopped");
            }

            // Check for critical errors
            if let Some(critical_error) = dispatcher_guard.last_error() {
                error!(
                    workflow_id = %workflow_id,
                    error = %critical_error,
                    "Critical error detected in dispatcher"
                );
                return Err(AutonomousError::WorkflowExecution(
                    format!("Critical error: {}", critical_error)
                ));
            }
        }

        info!(
            workflow_id = %workflow_id,
            steps_completed,
            steps_failed,
            recoveries_performed,
            reassignments_performed,
            "Autonomous execution completed"
        );

        Ok(ExecutionResult {
            success: execution_error.is_none(),
            workflow_id: workflow_id.clone(),
            context,
            steps_completed,
            steps_failed,
            recoveries_performed,
            reassignments_performed,
            error: execution_error,
        })
    }

    /// Gets the current execution monitor.
    pub fn get_monitor(&self) -> ExecutionMonitor {
        self.monitor.lock().unwrap().clone()
    }

    /// Gets the task dispatcher for external access.
    ///
    /// # Returns
    /// Returns `Some(Arc<Mutex<TaskDispatcher>>)` if dispatcher is available, `None` otherwise.
    pub fn get_dispatcher(&self) -> Option<Arc<Mutex<TaskDispatcher>>> {
        self.dispatcher.as_ref().map(Arc::clone)
    }

    /// Converts a WorkflowTemplate to an executable Workflow model.
    ///
    /// Creates Task entries for each step and stores them in the database.
    async fn convert_template_to_workflow(
        &self,
        template: &WorkflowTemplate,
        workflow_id: &str,
    ) -> Result<crate::models::Workflow> {
        use crate::models::{Task, Workflow, WorkflowStep};
        use crate::storage::{SqliteTaskRepository, TaskRepository};

        let db = Arc::clone(&self.db);
        let mut workflow = Workflow::new(
            workflow_id.to_string(),
            template.name.clone(),
            template.description.clone().unwrap_or_default(),
        );

        // Create tasks and steps
        let mut steps = Vec::new();
        for (idx, template_step) in template.steps.iter().enumerate() {
            // Skip UI steps
            if template_step.config.step_type == crate::workflow::templates::WorkflowStepType::Ui {
                continue;
            }

            let agent_id = &template_step.config.agent_id;
            let task_id = format!("{}-task-{}", workflow_id, idx);
            let step_id = format!("{}-step-{}", workflow_id, idx);

            // Create task for this step
            let task = Task::new(
                task_id.clone(),
                template_step.config.agent_name.clone()
                    .unwrap_or_else(|| format!("Step {}", idx)),
                format!("Task for workflow step {}", idx),
                agent_id.clone(),
                serde_json::json!({}), // Empty input for now
            );

            // Store task in database
            {
                let mut db_guard = db.lock().map_err(|e| {
                    AutonomousError::WorkflowExecution(format!("Database lock failed: {}", e))
                })?;
                let mut task_repo = SqliteTaskRepository::new(&mut *db_guard);
                task_repo.create(&task).map_err(|e| {
                    AutonomousError::WorkflowExecution(format!("Task creation failed: {}", e))
                })?;
            }

            // Create workflow step
            let mut step = WorkflowStep::new(
                step_id,
                template_step.config.agent_name.clone()
                    .unwrap_or_else(|| format!("Step {}", idx)),
                format!("Workflow step {}", idx),
                task_id,
                idx as u32,
            );

            // Add config JSON if present
            if let Some(ref module) = template_step.config.module {
                step.config_json = Some(serde_json::to_string(module).unwrap_or_default());
            }

            steps.push(step);
        }

        // Add steps to workflow
        for step in steps {
            workflow.add_step(step).map_err(|e| {
                AutonomousError::WorkflowExecution(format!("Failed to add step: {}", e))
            })?;
        }

        Ok(workflow)
    }

    /// Attempts to recover from a workflow failure using the recovery manager.
    async fn attempt_recovery(
        &self,
        workflow_id: &str,
        recovery_manager: &RecoveryManager,
        _db: Arc<std::sync::Mutex<crate::storage::Database>>,
    ) -> Result<ExecutionContext> {
        use tracing::{info, warn};
        use crate::workflow::recovery::{RecoveryContext, RecoveryStrategy};

        // Try to find a checkpoint for the workflow
        let checkpoint_opt = recovery_manager.find_checkpoint_for_step(workflow_id);

        if let Some(checkpoint) = checkpoint_opt {
            info!(
                workflow_id = %workflow_id,
                checkpoint_id = %checkpoint.id,
                "Attempting recovery from checkpoint"
            );

            // Create recovery context
            use crate::workflow::failure::FailureType;

            let recovery_context = RecoveryContext {
                workflow_id: workflow_id.to_string(),
                failed_step_id: workflow_id.to_string(),
                checkpoint_id: Some(checkpoint.id.clone()),
                execution_context: ExecutionContext::new(workflow_id.to_string()),
                failure_type: FailureType::Transient {
                    reason: "Workflow execution failed".to_string(),
                },
            };

            // Execute recovery
            let strategy = RecoveryStrategy::RestoreCheckpoint {
                checkpoint_id: checkpoint.id.clone(),
            };

            recovery_manager.execute_recovery(strategy, &recovery_context).map_err(|e| {
                AutonomousError::Recovery(format!("Checkpoint restore failed: {}", e))
            })?;

            // Return a minimal context indicating recovery
            let context = ExecutionContext::new(workflow_id.to_string());
            return Ok(context);
        }

        warn!(
            workflow_id = %workflow_id,
            "No checkpoints available for recovery"
        );

        Err(AutonomousError::Recovery("No checkpoints available".to_string()))
    }

    /// Attempts to reassign failed workflow steps to different agents.
    async fn attempt_reassignment(
        &self,
        workflow: &crate::models::Workflow,
        _reassignment: &AgentReassignment,
        _db: Arc<std::sync::Mutex<crate::storage::Database>>,
    ) -> Result<ExecutionContext> {
        
        use tracing::{info, warn};

        info!(
            workflow_id = %workflow.id,
            "Attempting agent reassignment"
        );

        // Find failed steps (would need to track this in real implementation)
        // For now, just return error indicating reassignment not yet fully implemented
        warn!(
            workflow_id = %workflow.id,
            "Reassignment logic needs full implementation"
        );

        Err(AutonomousError::Reassignment(
            "Reassignment not fully implemented".to_string()
        ))
    }
}

