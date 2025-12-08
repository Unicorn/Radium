//! Parallel task executor with concurrency control and state tracking.
//!
//! This module provides parallel execution of Braingrid tasks while respecting
//! dependencies, managing concurrency limits, and tracking execution state.

use crate::context::braingrid_client::{BraingridClient, BraingridTask, TaskStatus};
use crate::planning::dag::DependencyGraph;
use crate::workflow::agent_selector::AgentSelector;
use crate::workflow::execution_state::{ExecutionState, TaskExecutionStatus, TaskResult};
use chrono::Utc;
use radium_orchestrator::{AgentExecutor, AgentOutput};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{error, info, warn};

/// Execution report summarizing task execution results.
#[derive(Debug, Clone)]
pub struct ExecutionReport {
    /// Total number of tasks.
    pub total_tasks: usize,
    /// Number of tasks completed successfully.
    pub completed_tasks: usize,
    /// Number of tasks that failed.
    pub failed_tasks: usize,
    /// Number of tasks that were blocked.
    pub blocked_tasks: usize,
    /// Total execution time in seconds.
    pub total_execution_time_secs: u64,
    /// Whether execution was successful (all tasks completed).
    pub success: bool,
}

/// Parallel executor for Braingrid tasks.
///
/// Executes tasks concurrently while respecting dependencies and concurrency limits.
pub struct ParallelExecutor {
    /// Maximum number of concurrent task executions.
    max_concurrent: usize,
    /// Semaphore for concurrency control.
    semaphore: Arc<Semaphore>,
    /// Braingrid client for status updates.
    braingrid_client: Arc<BraingridClient>,
    /// Agent executor for task execution.
    agent_executor: Arc<AgentExecutor>,
    /// Agent selector for intelligent agent selection.
    agent_selector: Arc<AgentSelector>,
}

impl ParallelExecutor {
    /// Creates a new parallel executor.
    ///
    /// # Arguments
    /// * `max_concurrent` - Maximum number of concurrent task executions
    /// * `braingrid_client` - Braingrid client for status updates
    /// * `agent_executor` - Agent executor for task execution
    /// * `agent_selector` - Agent selector for intelligent agent selection
    pub fn new(
        max_concurrent: usize,
        braingrid_client: Arc<BraingridClient>,
        agent_executor: Arc<AgentExecutor>,
        agent_selector: Arc<AgentSelector>,
    ) -> Self {
        Self {
            max_concurrent,
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            braingrid_client,
            agent_executor,
            agent_selector,
        }
    }

    /// Executes tasks in parallel while respecting dependencies.
    ///
    /// # Arguments
    /// * `tasks` - Vector of Braingrid tasks to execute
    /// * `dep_graph` - Dependency graph for task dependencies
    /// * `requirement_id` - The requirement ID for status updates
    ///
    /// # Returns
    /// Execution report with summary statistics
    pub async fn execute_tasks(
        &self,
        tasks: Vec<BraingridTask>,
        dep_graph: &DependencyGraph,
        requirement_id: &str,
    ) -> Result<ExecutionReport, String> {
        let start_time = std::time::Instant::now();

        // Get all task IDs
        let task_ids: Vec<String> = tasks.iter().map(|t| t.number.clone()).collect();
        let task_map: std::collections::HashMap<String, &BraingridTask> = tasks
            .iter()
            .map(|t| (t.number.clone(), t))
            .collect();

        // Initialize execution state
        let execution_state = Arc::new(ExecutionState::new(task_ids.clone()));

        // Validate dependency graph
        dep_graph
            .detect_cycles()
            .map_err(|e| format!("Circular dependency detected: {}", e))?;

        info!(
            requirement_id = %requirement_id,
            total_tasks = tasks.len(),
            max_concurrent = self.max_concurrent,
            "Starting parallel task execution"
        );

        // Track completed tasks for dependency checking
        let mut completed_task_ids = HashSet::new();

        // Continuously execute ready tasks in batches until all are done
        loop {
            // Get tasks that are ready to execute (dependencies satisfied)
            let ready_task_ids = dep_graph.ready_tasks(&completed_task_ids);

            if ready_task_ids.is_empty() {
                // No more ready tasks - check if we're done
                if execution_state.completed_count() + execution_state.failed_count()
                    >= task_ids.len()
                {
                    break;
                }
                // Wait a bit for tasks to complete
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                continue;
            }

            // Filter to tasks that aren't already completed/failed and aren't blocked
            let mut batch_task_ids = Vec::new();
            for task_id in ready_task_ids {
                // Check if task is already completed or failed
                if execution_state.is_completed(&task_id) || execution_state.is_failed(&task_id) {
                    continue;
                }

                // Check if task is blocked by failed dependencies
                if self.is_blocked_by_failures(&task_id, &tasks, &execution_state) {
                    execution_state.mark_blocked(&task_id);
                    warn!(
                        requirement_id = %requirement_id,
                        task_id = %task_id,
                        "Task blocked by failed dependencies"
                    );
                    continue;
                }

                batch_task_ids.push(task_id);
                
                // Limit batch size to max_concurrent
                if batch_task_ids.len() >= self.max_concurrent {
                    break;
                }
            }

            if batch_task_ids.is_empty() {
                // No tasks to execute in this batch, wait a bit
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                continue;
            }

            // Execute batch of tasks in parallel
            let mut batch_handles = Vec::new();
            for task_id in batch_task_ids {
                let task = task_map
                    .get(&task_id)
                    .ok_or_else(|| format!("Task not found: {}", task_id))?;

                // Clone necessary data for async task
                let task_id_clone = task_id.clone();
                let task_clone = task.clone();
                let requirement_id_clone = requirement_id.to_string();
                let execution_state_clone = Arc::clone(&execution_state);
                let braingrid_client_clone = Arc::clone(&self.braingrid_client);
                let agent_executor_clone = Arc::clone(&self.agent_executor);
                let agent_selector_clone = Arc::clone(&self.agent_selector);
                let semaphore_clone = Arc::clone(&self.semaphore);

                // Spawn task execution
                let handle = tokio::spawn(async move {
                    // Acquire semaphore permit for concurrency control
                    let _permit = semaphore_clone.acquire().await.map_err(|e| {
                        format!("Failed to acquire semaphore permit: {}", e)
                    })?;

                    // Update task status to IN_PROGRESS in Braingrid
                    let _ = braingrid_client_clone
                        .update_task_status(
                            &task_clone.task_id(),
                            &requirement_id_clone,
                            TaskStatus::InProgress,
                            None,
                        )
                        .await;

                    // Mark as running in execution state
                    execution_state_clone.mark_running(&task_id_clone);

                    let started_at = Utc::now();

                    // Select agent
                    let agent_id = match agent_selector_clone.select_agent(&task_clone).await {
                        Ok(id) => id,
                        Err(e) => {
                            error!(
                                requirement_id = %requirement_id_clone,
                                task_id = %task_id_clone,
                                error = %e,
                                "Failed to select agent, using code-agent as fallback"
                            );
                            "code-agent".to_string()
                        }
                    };

                    info!(
                        requirement_id = %requirement_id_clone,
                        task_id = %task_id_clone,
                        agent_id = %agent_id,
                        "Executing task with agent"
                    );

                    // Build goal from task information
                    let goal = if let Some(description) = &task_clone.description {
                        format!("Task: {}\n\nDescription:\n{}", task_clone.title, description)
                    } else {
                        format!("Task: {}", task_clone.title)
                    };

                    // Execute task
                    let execution_result = agent_executor_clone
                        .execute_agent_with_default_model(Some(&agent_id), &goal, None)
                        .await;

                    let completed_at = Utc::now();

                    // Process execution result
                    match execution_result {
                        Ok(result) if result.success => {
                            // Extract output
                            let output = match result.output {
                                AgentOutput::Text(text) => text,
                                AgentOutput::StructuredData(data) => {
                                    serde_json::to_string(&data).unwrap_or_default()
                                }
                                AgentOutput::ToolCall { name, args } => {
                                    format!("Tool call: {} with args: {:?}", name, args)
                                }
                                AgentOutput::Terminate => "Terminated".to_string(),
                            };

                            // TODO: Extract git commits from execution (requires git integration)
                            let commits = vec![];

                            // TODO: Extract test results from execution (requires test integration)
                            let test_results = None;

                            let task_result = TaskResult::success(
                                output,
                                commits,
                                test_results,
                                started_at,
                                completed_at,
                                agent_id,
                            );

                            execution_state_clone.mark_completed(&task_id_clone, task_result);

                            // Update Braingrid status
                            let notes = format!(
                                "Completed via agent {} in {}s",
                                agent_id,
                                (completed_at - started_at).num_seconds()
                            );
                            let _ = braingrid_client_clone
                                .update_task_status(
                                    &task_clone.task_id(),
                                    &requirement_id_clone,
                                    TaskStatus::Completed,
                                    Some(&notes),
                                )
                                .await;

                            info!(
                                requirement_id = %requirement_id_clone,
                                task_id = %task_id_clone,
                                "Task completed successfully"
                            );

                            Ok(task_id_clone)
                        }
                        Ok(result) => {
                            // Execution failed
                            let error_msg = result.error.unwrap_or_else(|| {
                                "Unknown execution error".to_string()
                            });

                            let output = match result.output {
                                AgentOutput::Text(text) => text,
                                AgentOutput::StructuredData(data) => {
                                    serde_json::to_string(&data).unwrap_or_default()
                                }
                                AgentOutput::ToolCall { .. } => "Tool call failed".to_string(),
                                AgentOutput::Terminate => "Terminated".to_string(),
                            };

                            let task_result = TaskResult::failure(
                                output,
                                started_at,
                                completed_at,
                                agent_id,
                                error_msg.clone(),
                            );

                            execution_state_clone.mark_failed(&task_id_clone, task_result);

                            // Update Braingrid status
                            let _ = braingrid_client_clone
                                .update_task_status(
                                    &task_clone.task_id(),
                                    &requirement_id_clone,
                                    TaskStatus::InProgress, // Keep as IN_PROGRESS on failure
                                    Some(&error_msg),
                                )
                                .await;

                            error!(
                                requirement_id = %requirement_id_clone,
                                task_id = %task_id_clone,
                                error = %error_msg,
                                "Task execution failed"
                            );

                            Err(error_msg)
                        }
                        Err(e) => {
                            // Execution error
                            let error_msg = format!("Execution error: {}", e);
                            let task_result = TaskResult::failure(
                                String::new(),
                                started_at,
                                completed_at,
                                agent_id,
                                error_msg.clone(),
                            );

                            execution_state_clone.mark_failed(&task_id_clone, task_result);

                            // Update Braingrid status
                            let _ = braingrid_client_clone
                                .update_task_status(
                                    &task_clone.task_id(),
                                    &requirement_id_clone,
                                    TaskStatus::InProgress,
                                    Some(&error_msg),
                                )
                                .await;

                            error!(
                                requirement_id = %requirement_id_clone,
                                task_id = %task_id_clone,
                                error = %error_msg,
                                "Task execution error"
                            );

                            Err(error_msg)
                        }
                    }
                });

                batch_handles.push(handle);
            }

            // Wait for all tasks in this batch to complete
            for handle in batch_handles {
                match handle.await {
                    Ok(Ok(task_id)) => {
                        if execution_state.is_completed(&task_id) {
                            completed_task_ids.insert(task_id);
                        }
                    }
                    Ok(Err(e)) => {
                        error!(
                            requirement_id = %requirement_id,
                            error = %e,
                            "Task execution failed"
                        );
                    }
                    Err(e) => {
                        error!(
                            requirement_id = %requirement_id,
                            error = %e,
                            "Task join error"
                        );
                    }
                }
            }
        }

        let total_execution_time_secs = start_time.elapsed().as_secs();

        // Build execution report
        let completed_count = execution_state.completed_count();
        let failed_count = execution_state.failed_count();
        let blocked_count = task_ids
            .iter()
            .filter(|id| execution_state.get_status(id) == TaskExecutionStatus::Blocked)
            .count();

        let report = ExecutionReport {
            total_tasks: tasks.len(),
            completed_tasks: completed_count,
            failed_tasks: failed_count,
            blocked_tasks: blocked_count,
            total_execution_time_secs,
            success: failed_count == 0 && blocked_count == 0,
        };

        info!(
            requirement_id = %requirement_id,
            completed = completed_count,
            failed = failed_count,
            blocked = blocked_count,
            duration_secs = total_execution_time_secs,
            "Parallel execution completed"
        );

        Ok(report)
    }

    /// Checks if a task is blocked by failed dependencies.
    ///
    /// A task is blocked if any of its direct dependencies have failed.
    fn is_blocked_by_failures(
        &self,
        task_id: &str,
        tasks: &[BraingridTask],
        execution_state: &ExecutionState,
    ) -> bool {
        // Find the task
        if let Some(task) = tasks.iter().find(|t| t.number == task_id) {
            // Check if any dependency has failed
            for dep_number in &task.dependencies {
                if execution_state.is_failed(dep_number) {
                    return true;
                }
            }
        }
        false
    }
}

