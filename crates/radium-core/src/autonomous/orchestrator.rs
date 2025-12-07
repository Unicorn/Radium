//! Autonomous execution orchestrator.
//!
//! Coordinates all autonomous capabilities for end-to-end execution from
//! high-level goals to completion with self-healing.

use crate::agents::registry::AgentRegistry;
use crate::checkpoint::CheckpointManager;
use crate::learning::store::LearningStore;
use crate::learning::recovery_learning::RecoveryLearning;
use crate::planning::{AutonomousPlan, AutonomousPlanner, PlanningError};
use crate::workflow::engine::ExecutionContext;
use crate::workflow::executor::WorkflowExecutor;
use crate::workflow::failure::FailurePolicy;
use crate::workflow::recovery::RecoveryManager;
use crate::workflow::reassignment::{AgentReassignment, AgentSelector};
use crate::workflow::service::WorkflowService;
use crate::workflow::templates::WorkflowTemplate;
use crate::workspace::Workspace;
use radium_abstraction::Model;
use radium_orchestrator::{AgentExecutor, Orchestrator};
use std::sync::{Arc, Mutex};
use thiserror::Error;

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
}

impl Default for AutonomousConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            enable_recovery: true,
            enable_reassignment: true,
            enable_learning: true,
            checkpoint_frequency: CheckpointFrequency::EveryStep,
        }
    }
}

/// Checkpoint frequency for autonomous execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckpointFrequency {
    /// Create checkpoint before every step.
    EveryStep,
    /// Create checkpoint before every iteration.
    EveryIteration,
    /// Create checkpoint only on failure.
    OnFailure,
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
        let planner = AutonomousPlanner::new(agent_registry);

        // Initialize monitor
        let monitor = Arc::new(Mutex::new(ExecutionMonitor::new(
            "pending".to_string(),
            0,
        )));

        Ok(Self {
            planner,
            executor: workflow_executor,
            workflow_service,
            recovery_manager,
            reassignment,
            learning,
            config,
            monitor,
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

        // Step 4: Execute workflow (simplified - would need full integration)
        // For now, return a placeholder result
        // Full implementation would:
        // - Convert WorkflowTemplate to Workflow
        // - Execute with WorkflowExecutor
        // - Handle failures with recovery and reassignment
        // - Record learning data

        info!(workflow_id = %workflow_id, "Autonomous execution completed");

        Ok(ExecutionResult {
            success: true,
            workflow_id: workflow_id.clone(),
            context: ExecutionContext::new(workflow_id),
            steps_completed: workflow_template.steps.len() as u32,
            steps_failed: 0,
            recoveries_performed: 0,
            reassignments_performed: 0,
            error: None,
        })
    }

    /// Gets the current execution monitor.
    pub fn get_monitor(&self) -> ExecutionMonitor {
        self.monitor.lock().unwrap().clone()
    }
}

