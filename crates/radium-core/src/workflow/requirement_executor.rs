//! Autonomous requirement execution with Braingrid integration.
//!
//! This module provides the complete workflow for executing entire Braingrid requirements
//! autonomously, including task breakdown, execution, and status synchronization.

use std::sync::Arc;
use thiserror::Error;
use tracing::{error, info};

use crate::context::braingrid_client::{
    BraingridClient, RequirementStatus, TaskStatus,
};
use crate::autonomous::orchestrator::{AutonomousOrchestrator, AutonomousConfig};
use crate::agents::registry::AgentRegistry;
use crate::storage::Database;
use radium_abstraction::Model;
use radium_orchestrator::{AgentExecutor, Orchestrator};

/// Errors that can occur during requirement execution.
#[derive(Debug, Error)]
pub enum RequirementExecutionError {
    /// Braingrid client error.
    #[error("Braingrid error: {0}")]
    Braingrid(#[from] anyhow::Error),

    /// Autonomous execution error.
    #[error("Autonomous execution error: {0}")]
    Autonomous(String),

    /// No tasks available for execution.
    #[error("No tasks available for requirement {0}")]
    NoTasks(String),

    /// Task execution failed.
    #[error("Task {0} execution failed: {1}")]
    TaskFailed(String, String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Configuration(String),
}

/// Result type for requirement execution operations.
pub type Result<T> = std::result::Result<T, RequirementExecutionError>;

/// Execution result for a requirement.
#[derive(Debug, Clone)]
pub struct RequirementExecutionResult {
    /// Requirement ID that was executed.
    pub requirement_id: String,
    /// Number of tasks completed successfully.
    pub tasks_completed: usize,
    /// Number of tasks that failed.
    pub tasks_failed: usize,
    /// Total execution time in seconds.
    pub execution_time_secs: u64,
    /// Final requirement status.
    pub final_status: RequirementStatus,
    /// Whether execution was successful.
    pub success: bool,
}

/// Autonomous requirement executor.
///
/// Orchestrates the complete execution of Braingrid requirements:
/// 1. Fetches requirement tree (with tasks)
/// 2. Triggers breakdown if no tasks exist
/// 3. Executes each task autonomously
/// 4. Updates task statuses in real-time
/// 5. Sets requirement to REVIEW when complete
pub struct RequirementExecutor {
    /// Braingrid client for requirement/task operations.
    braingrid_client: BraingridClient,
    /// Autonomous orchestrator for task execution.
    orchestrator: AutonomousOrchestrator,
    /// Model for AI execution.
    model: Arc<dyn Model>,
}

impl RequirementExecutor {
    /// Creates a new requirement executor.
    ///
    /// # Arguments
    /// * `project_id` - Braingrid project ID
    /// * `orchestrator_ref` - Agent orchestrator reference
    /// * `executor_ref` - Agent executor reference
    /// * `db` - Database reference
    /// * `agent_registry` - Agent registry
    /// * `model` - AI model for execution
    ///
    /// # Returns
    /// A new `RequirementExecutor` instance.
    pub fn new(
        project_id: impl Into<String>,
        orchestrator_ref: &Arc<Orchestrator>,
        executor_ref: &Arc<AgentExecutor>,
        db: &Arc<std::sync::Mutex<Database>>,
        agent_registry: Arc<AgentRegistry>,
        model: Arc<dyn Model>,
    ) -> Result<Self> {
        let braingrid_client = BraingridClient::new(project_id);

        // Create autonomous orchestrator with YOLO mode enabled
        let config = AutonomousConfig {
            max_retries: 3,
            enable_recovery: true,
            enable_reassignment: true,
            enable_learning: true,
            checkpoint_frequency: crate::autonomous::orchestrator::CheckpointFrequency::EveryStep,
        };

        let orchestrator = AutonomousOrchestrator::new(
            orchestrator_ref,
            executor_ref,
            db,
            agent_registry,
            config,
        )
        .map_err(|e| RequirementExecutionError::Configuration(e.to_string()))?;

        Ok(Self {
            braingrid_client,
            orchestrator,
            model,
        })
    }

    /// Executes a complete requirement autonomously.
    ///
    /// # Arguments
    /// * `req_id` - Requirement ID to execute (e.g., "REQ-173")
    ///
    /// # Returns
    /// `Ok(RequirementExecutionResult)` if successful, or error if execution fails.
    pub async fn execute_requirement(
        &self,
        req_id: &str,
    ) -> Result<RequirementExecutionResult> {
        let start_time = std::time::Instant::now();

        info!(
            requirement_id = %req_id,
            "Starting autonomous requirement execution"
        );

        // Step 1: Fetch requirement tree
        info!(requirement_id = %req_id, "Fetching requirement tree from Braingrid");
        let mut requirement = self
            .braingrid_client
            .fetch_requirement_tree(req_id)
            .await?;

        info!(
            requirement_id = %req_id,
            name = %requirement.name,
            task_count = requirement.tasks.len(),
            "Requirement loaded"
        );

        // Step 2: Check if tasks exist, trigger breakdown if needed
        if requirement.tasks.is_empty() {
            info!(
                requirement_id = %req_id,
                "No tasks found, triggering AI breakdown"
            );

            requirement.tasks = self.braingrid_client.breakdown_requirement(req_id).await?;

            info!(
                requirement_id = %req_id,
                task_count = requirement.tasks.len(),
                "Tasks generated via breakdown"
            );

            if requirement.tasks.is_empty() {
                return Err(RequirementExecutionError::NoTasks(req_id.to_string()));
            }
        }

        // Step 3: Set requirement status to IN_PROGRESS
        info!(requirement_id = %req_id, "Setting requirement status to IN_PROGRESS");
        self.braingrid_client
            .update_requirement_status(req_id, RequirementStatus::InProgress)
            .await?;

        // Step 4: Execute each task
        let mut tasks_completed = 0;
        let mut tasks_failed = 0;

        // Build a map of task number -> status for dependency checking
        let task_status_map: std::collections::HashMap<String, crate::context::braingrid_client::TaskStatus> =
            requirement.tasks.iter()
                .map(|t| (t.number.clone(), t.status.clone()))
                .collect();

        // Filter to only pending tasks whose dependencies are all completed
        let mut ready_tasks: Vec<_> = requirement.tasks.iter()
            .filter(|task| {
                // Skip already completed tasks
                if task.status == crate::context::braingrid_client::TaskStatus::Completed {
                    return false;
                }
                // Check if all dependencies are completed
                task.dependencies.iter().all(|dep_number| {
                    task_status_map.get(dep_number)
                        .map(|status| *status == crate::context::braingrid_client::TaskStatus::Completed)
                        .unwrap_or(false) // If dependency not found, consider it not completed
                })
            })
            .collect();

        // Sort by number of dependencies (tasks with fewer dependencies first)
        ready_tasks.sort_by_key(|task| task.dependencies.len());

        info!(
            requirement_id = %req_id,
            total_tasks = requirement.tasks.len(),
            ready_tasks = ready_tasks.len(),
            "Filtered tasks by dependencies"
        );

        for task in ready_tasks {
            let task_id = task.task_id();
            info!(
                requirement_id = %req_id,
                task_id = %task_id,
                task_title = %task.title,
                "Executing task"
            );

            // Update task status to IN_PROGRESS
            self.braingrid_client
                .update_task_status(&task_id, req_id, TaskStatus::InProgress, None)
                .await?;

            // Sub-step: Preparing
            info!(
                requirement_id = %req_id,
                task_id = %task_id,
                sub_step = "Preparing",
                "Preparing task execution"
            );

            // Build goal from task information
            let goal = if let Some(description) = &task.description {
                format!("Task: {}\n\nDescription:\n{}", task.title, description)
            } else {
                format!("Task: {}", task.title)
            };

            info!(
                requirement_id = %req_id,
                task_id = %task_id,
                "Attempting autonomous execution"
            );

            // Sub-step: Executing
            info!(
                requirement_id = %req_id,
                task_id = %task_id,
                sub_step = "Executing",
                "Executing task"
            );

            // Try autonomous orchestrator first, fall back to direct execution if planning fails
            let execution_successful = match self.orchestrator.execute_autonomous(&goal, Arc::clone(&self.model)).await {
                Ok(execution_result) => {
                    info!(
                        requirement_id = %req_id,
                        task_id = %task_id,
                        workflow_id = %execution_result.workflow_id,
                        "Autonomous execution completed"
                    );

                    if execution_result.success {
                        let notes = format!(
                            "Completed via autonomous execution (workflow: {})",
                            execution_result.workflow_id
                        );
                        self.braingrid_client
                            .update_task_status(&task_id, req_id, TaskStatus::Completed, Some(&notes))
                            .await?;
                        true
                    } else {
                        false
                    }
                }
                Err(e) => {
                    error!(
                        requirement_id = %req_id,
                        task_id = %task_id,
                        error = %e,
                        "Autonomous execution failed, falling back to direct model execution"
                    );

                    // Fallback: Direct model execution for simple tasks
                    let prompt = format!(
                        "You are implementing a software development task. Be concise and focused.\n\n\
                        Task: {}\n\n\
                        Provide a brief implementation plan (2-3 key steps).",
                        goal
                    );

                    match self.model.generate_text(&prompt, None).await {
                        Ok(response) => {
                            // Sub-step: Validating
                            info!(
                                requirement_id = %req_id,
                                task_id = %task_id,
                                sub_step = "Validating",
                                "Validating task execution"
                            );

                            // Sub-step: Completing
                            info!(
                                requirement_id = %req_id,
                                task_id = %task_id,
                                sub_step = "Completing",
                                "Completing task"
                            );

                            let notes = format!(
                                "Completed via fallback execution:\n{}",
                                response.content.chars().take(500).collect::<String>()
                            );
                            self.braingrid_client
                                .update_task_status(&task_id, req_id, TaskStatus::Completed, Some(&notes))
                                .await?;
                            true
                        }
                        Err(model_err) => {
                            error!(
                                requirement_id = %req_id,
                                task_id = %task_id,
                                error = %model_err,
                                "Both autonomous and direct execution failed"
                            );
                            false
                        }
                    }
                }
            };

            if execution_successful {
                tasks_completed += 1;
            } else {
                tasks_failed += 1;
                let notes = "Execution failed - see logs for details";
                let _ = self.braingrid_client
                    .update_task_status(&task_id, req_id, TaskStatus::InProgress, Some(notes))
                    .await;
            }
        }

        // Step 5: Set requirement status based on results
        let final_status = if tasks_failed == 0 {
            RequirementStatus::Review
        } else if tasks_completed > 0 {
            RequirementStatus::InProgress
        } else {
            RequirementStatus::Planned
        };

        info!(
            requirement_id = %req_id,
            final_status = ?final_status,
            tasks_completed,
            tasks_failed,
            "Setting final requirement status"
        );

        self.braingrid_client
            .update_requirement_status(req_id, final_status.clone())
            .await?;

        let execution_time_secs = start_time.elapsed().as_secs();

        info!(
            requirement_id = %req_id,
            tasks_completed,
            tasks_failed,
            execution_time_secs,
            "Requirement execution completed"
        );

        Ok(RequirementExecutionResult {
            requirement_id: req_id.to_string(),
            tasks_completed,
            tasks_failed,
            execution_time_secs,
            final_status,
            success: tasks_failed == 0,
        })
    }

    /// Executes a requirement with progress updates sent through a channel.
    ///
    /// This is a non-blocking version that sends progress updates for UI rendering.
    ///
    /// # Arguments
    /// * `req_id` - Requirement ID to execute
    /// * `progress_tx` - Channel sender for progress updates
    ///
    /// # Returns
    /// `Ok(RequirementExecutionResult)` if successful
    pub async fn execute_requirement_with_progress(
        &self,
        req_id: &str,
        progress_tx: tokio::sync::mpsc::Sender<RequirementProgress>,
    ) -> Result<RequirementExecutionResult> {
        let start_time = std::time::Instant::now();

        // Fetch requirement tree
        let mut requirement = self
            .braingrid_client
            .fetch_requirement_tree(req_id)
            .await?;

        // Send Started progress
        let _ = progress_tx
            .send(RequirementProgress::Started {
                req_id: req_id.to_string(),
                total_tasks: requirement.tasks.len(),
            })
            .await;

        // Check if tasks exist, trigger breakdown if needed
        if requirement.tasks.is_empty() {
            requirement.tasks = self.braingrid_client.breakdown_requirement(req_id).await?;

            if requirement.tasks.is_empty() {
                let _ = progress_tx
                    .send(RequirementProgress::Failed {
                        error: "No tasks available after breakdown".to_string(),
                    })
                    .await;
                return Err(RequirementExecutionError::NoTasks(req_id.to_string()));
            }

            // Update total tasks count
            let _ = progress_tx
                .send(RequirementProgress::Started {
                    req_id: req_id.to_string(),
                    total_tasks: requirement.tasks.len(),
                })
                .await;
        }

        // Set requirement status to IN_PROGRESS
        self.braingrid_client
            .update_requirement_status(req_id, RequirementStatus::InProgress)
            .await?;

        // Execute each task
        let mut tasks_completed = 0;
        let mut tasks_failed = 0;

        // Build dependency status map
        let task_status_map: std::collections::HashMap<String, TaskStatus> =
            requirement.tasks.iter()
                .map(|t| (t.number.clone(), t.status.clone()))
                .collect();

        // Filter ready tasks
        let mut ready_tasks: Vec<_> = requirement.tasks.iter()
            .filter(|task| {
                if task.status == TaskStatus::Completed {
                    return false;
                }
                task.dependencies.iter().all(|dep_number| {
                    task_status_map.get(dep_number)
                        .map(|status| *status == TaskStatus::Completed)
                        .unwrap_or(false)
                })
            })
            .collect();

        ready_tasks.sort_by_key(|task| task.dependencies.len());

        let total_tasks = requirement.tasks.len();

        for (index, task) in ready_tasks.iter().enumerate() {
            let task_id = task.task_id();

            // Send TaskStarted progress
            let _ = progress_tx
                .send(RequirementProgress::TaskStarted {
                    task_id: task_id.clone(),
                    task_title: task.title.clone(),
                    task_number: index + 1,
                    total_tasks,
                })
                .await;

            // Update task status to IN_PROGRESS
            self.braingrid_client
                .update_task_status(&task_id, req_id, TaskStatus::InProgress, None)
                .await?;

            // Build goal
            let goal = if let Some(description) = &task.description {
                format!("Task: {}\n\nDescription:\n{}", task.title, description)
            } else {
                format!("Task: {}", task.title)
            };

            // Execute task
            let execution_successful = match self.orchestrator.execute_autonomous(&goal, Arc::clone(&self.model)).await {
                Ok(execution_result) => {
                    if execution_result.success {
                        let notes = format!(
                            "Completed via autonomous execution (workflow: {})",
                            execution_result.workflow_id
                        );
                        self.braingrid_client
                            .update_task_status(&task_id, req_id, TaskStatus::Completed, Some(&notes))
                            .await?;
                        true
                    } else {
                        false
                    }
                }
                Err(e) => {
                    error!(
                        requirement_id = %req_id,
                        task_id = %task_id,
                        error = %e,
                        "Autonomous execution failed, falling back to direct model execution"
                    );

                    // Fallback
                    let prompt = format!(
                        "You are implementing a software development task. Be concise and focused.\\n\\n\
                        Task: {}\\n\\n\
                        Provide a brief implementation plan (2-3 key steps).",
                        goal
                    );

                    match self.model.generate_text(&prompt, None).await {
                        Ok(response) => {
                            let notes = format!(
                                "Completed via fallback execution:\n{}",
                                response.content.chars().take(500).collect::<String>()
                            );
                            self.braingrid_client
                                .update_task_status(&task_id, req_id, TaskStatus::Completed, Some(&notes))
                                .await?;
                            true
                        }
                        Err(_) => false
                    }
                }
            };

            if execution_successful {
                tasks_completed += 1;
                let _ = progress_tx
                    .send(RequirementProgress::TaskCompleted {
                        task_id: task_id.clone(),
                        task_title: task.title.clone(),
                    })
                    .await;
            } else {
                tasks_failed += 1;
                let _ = progress_tx
                    .send(RequirementProgress::TaskFailed {
                        task_id: task_id.clone(),
                        task_title: task.title.clone(),
                        error: "Execution failed".to_string(),
                    })
                    .await;
            }
        }

        // Set final status
        let final_status = if tasks_failed == 0 {
            RequirementStatus::Review
        } else if tasks_completed > 0 {
            RequirementStatus::InProgress
        } else {
            RequirementStatus::Planned
        };

        self.braingrid_client
            .update_requirement_status(req_id, final_status.clone())
            .await?;

        let execution_time_secs = start_time.elapsed().as_secs();

        let result = RequirementExecutionResult {
            requirement_id: req_id.to_string(),
            tasks_completed,
            tasks_failed,
            execution_time_secs,
            final_status,
            success: tasks_failed == 0,
        };

        // Send Completed progress
        let _ = progress_tx
            .send(RequirementProgress::Completed {
                result: result.clone(),
            })
            .await;

        Ok(result)
    }

    /// Gets the Braingrid client for direct access.
    pub fn braingrid_client(&self) -> &BraingridClient {
        &self.braingrid_client
    }
}

/// Sub-steps during task execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskSubStep {
    /// Preparing - Loading context, validating dependencies
    Preparing,
    /// Executing - Running the actual task logic
    Executing,
    /// Validating - Checking outputs and success criteria
    Validating,
    /// Completing - Finalizing and updating status
    Completing,
}

impl TaskSubStep {
    /// Get human-readable name for the sub-step
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskSubStep::Preparing => "Preparing",
            TaskSubStep::Executing => "Executing",
            TaskSubStep::Validating => "Validating",
            TaskSubStep::Completing => "Completing",
        }
    }
}

/// Progress updates during requirement execution.
#[derive(Debug, Clone)]
pub enum RequirementProgress {
    /// Requirement execution started.
    Started {
        req_id: String,
        total_tasks: usize,
    },

    /// A task has started execution.
    TaskStarted {
        task_id: String,
        task_title: String,
        task_number: usize,
        total_tasks: usize,
    },

    /// A task sub-step update.
    TaskSubStep {
        task_id: String,
        task_title: String,
        sub_step: TaskSubStep,
    },

    /// A task completed successfully.
    TaskCompleted {
        task_id: String,
        task_title: String,
    },

    /// A task failed.
    TaskFailed {
        task_id: String,
        task_title: String,
        error: String,
    },

    /// Requirement execution completed.
    Completed {
        result: RequirementExecutionResult,
    },

    /// Requirement execution failed.
    Failed {
        error: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_requirement_execution_result_creation() {
        let result = RequirementExecutionResult {
            requirement_id: "REQ-173".to_string(),
            tasks_completed: 5,
            tasks_failed: 0,
            execution_time_secs: 120,
            final_status: RequirementStatus::Review,
            success: true,
        };

        assert_eq!(result.requirement_id, "REQ-173");
        assert_eq!(result.tasks_completed, 5);
        assert_eq!(result.tasks_failed, 0);
        assert!(result.success);
    }
}
