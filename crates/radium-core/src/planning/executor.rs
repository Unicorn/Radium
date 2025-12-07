//! Plan execution with state persistence and progress tracking.
//!
//! This module provides the plan executor that executes plans with intelligent retry logic,
//! error categorization, state persistence, and multiple execution modes.
//!
//! # Overview
//!
//! The plan executor provides:
//!
//! - **Intelligent Retry Logic**: Automatic retries with exponential backoff for recoverable errors
//! - **Error Categorization**: Distinguishes between recoverable and fatal errors
//! - **Execution Modes**: Bounded (limited iterations) and Continuous (run until complete)
//! - **State Persistence**: Saves progress after each task for checkpoint recovery
//! - **Dependency Validation**: Ensures task dependencies are met before execution
//! - **Context File Support**: Injects context files into agent prompts
//!
//! # Execution Lifecycle
//!
//! 1. **Load Manifest**: Load plan manifest from disk (or create new)
//! 2. **Resume Checkpoint**: If resuming, skip completed tasks
//! 3. **Iteration Loop**: Execute iterations based on RunMode
//! 4. **Task Execution**: For each task:
//!    - Check dependencies
//!    - Execute with retry logic
//!    - Save state after completion
//! 5. **Progress Tracking**: Update and display progress
//!
//! # Retry Logic
//!
//! The executor uses exponential backoff for retries:
//!
//! - **Max Retries**: Configurable (default: 3)
//! - **Backoff**: delay = base_delay * 2^attempt
//! - **Error Categorization**: Only retries recoverable errors
//!
//! # Error Categories
//!
//! - **Recoverable**: Network errors, rate limits, timeouts, server errors (5xx)
//! - **Fatal**: Auth failures, missing config, invalid data, dependency errors
//!
//! # Example
//!
//! ```rust,no_run
//! use radium_core::planning::executor::{PlanExecutor, ExecutionConfig, RunMode};
//! use radium_core::models::PlanManifest;
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = ExecutionConfig {
//!     resume: false,
//!     skip_completed: true,
//!     check_dependencies: true,
//!     state_path: PathBuf::from("plan/plan_manifest.json"),
//!     context_files: None,
//!     run_mode: RunMode::Bounded(5), // Limit to 5 iterations
//! };
//!
//! let executor = PlanExecutor::with_config(config);
//! let manifest = executor.load_manifest(&config.state_path)?;
//!
//! // Execute tasks...
//! # Ok(())
//! # }
//! ```
//!
//! # See Also
//!
//! - [User Guide](../../../docs/features/plan-execution.md) - Complete user documentation
//! - [CLI Commands](../../../docs/cli/commands/plan-execution.md) - Command-line usage
//! - [Autonomous Planning](autonomous) - Plan generation
//! - [DAG System](dag) - Dependency management

use crate::models::{Iteration, PlanManifest, PlanStatus, PlanTask};
use crate::{AgentDiscovery, PromptContext, PromptTemplate};
use radium_abstraction::Model;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// Plan execution error.
#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    /// I/O error during state persistence.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization error.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Agent not found.
    #[error("agent not found: {0}")]
    AgentNotFound(String),

    /// Prompt error.
    #[error("prompt error: {0}")]
    Prompt(String),

    /// Model execution error.
    #[error("model execution error: {0}")]
    ModelExecution(String),

    /// Task dependency not met.
    #[error("task dependency not met: {0}")]
    DependencyNotMet(String),
}

/// Error category for retry logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Recoverable error that can be retried (network timeout, rate limit, etc.).
    Recoverable,
    /// Fatal error that should not be retried (auth failure, config missing, etc.).
    Fatal,
}

impl ExecutionError {
    /// Categorizes an error as recoverable or fatal.
    pub fn category(&self) -> ErrorCategory {
        let error_str = self.to_string().to_lowercase();
        
        // Check for recoverable error patterns
        if error_str.contains("429")
            || error_str.contains("rate limit")
            || error_str.contains("timeout")
            || error_str.contains("network")
            || error_str.contains("connection")
            || error_str.contains("5")
            || error_str.contains("server error")
            || error_str.contains("file lock")
            || error_str.contains("temporary")
        {
            return ErrorCategory::Recoverable;
        }
        
        // Check for fatal error patterns
        if error_str.contains("401")
            || error_str.contains("403")
            || error_str.contains("unauthorized")
            || error_str.contains("forbidden")
            || error_str.contains("missing")
            || error_str.contains("invalid")
            || error_str.contains("not found")
            || error_str.contains("dependency not met")
        {
            return ErrorCategory::Fatal;
        }
        
        // Default: treat model execution errors as recoverable (might be transient)
        // Other errors default to fatal
        match self {
            ExecutionError::ModelExecution(_) => ErrorCategory::Recoverable,
            ExecutionError::Io(_) => ErrorCategory::Recoverable, // I/O errors might be transient
            _ => ErrorCategory::Fatal,
        }
    }
}

/// Result type for plan execution operations.
pub type Result<T> = std::result::Result<T, ExecutionError>;

/// Execution mode for plan execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    /// Bounded execution with a maximum iteration limit.
    Bounded(usize),
    /// Continuous execution until all tasks are complete (with sanity limit).
    Continuous,
}

/// Configuration for plan execution.
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    /// Whether to resume from last checkpoint.
    pub resume: bool,

    /// Whether to skip completed tasks.
    pub skip_completed: bool,

    /// Whether to validate task dependencies.
    pub check_dependencies: bool,

    /// Path to save state checkpoints.
    pub state_path: std::path::PathBuf,

    /// Optional context files content to inject into prompts.
    pub context_files: Option<String>,

    /// Execution mode (bounded or continuous).
    pub run_mode: RunMode,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            resume: false,
            skip_completed: true,
            check_dependencies: true,
            state_path: std::path::PathBuf::from("plan/plan_manifest.json"),
            context_files: None,
            run_mode: RunMode::Bounded(5),
        }
    }
}

/// Task execution result.
#[derive(Debug, Clone)]
pub struct TaskResult {
    /// Task ID that was executed.
    pub task_id: String,

    /// Whether execution was successful.
    pub success: bool,

    /// Model response content (if successful).
    pub response: Option<String>,

    /// Error message (if failed).
    pub error: Option<String>,

    /// Token usage information.
    pub tokens_used: Option<(usize, usize)>, // (prompt, completion)
}

/// Plan executor with state persistence.
pub struct PlanExecutor {
    /// Execution configuration.
    config: ExecutionConfig,

    /// Agent discovery service.
    agent_discovery: AgentDiscovery,
}

impl PlanExecutor {
    /// Creates a new plan executor with default configuration.
    pub fn new() -> Self {
        Self { config: ExecutionConfig::default(), agent_discovery: AgentDiscovery::new() }
    }

    /// Creates a new plan executor with custom configuration.
    pub fn with_config(config: ExecutionConfig) -> Self {
        Self { config, agent_discovery: AgentDiscovery::new() }
    }

    /// Executes a task with retry logic for recoverable errors.
    ///
    /// This method wraps `execute_task` with automatic retry logic using exponential backoff.
    ///
    /// # Arguments
    /// * `task` - The task to execute
    /// * `model` - The model instance to use
    /// * `max_retries` - Maximum number of retry attempts (default: 3)
    /// * `base_delay_ms` - Base delay in milliseconds for exponential backoff (default: 1000)
    ///
    /// # Returns
    /// The result of the task execution, or an error if all retries are exhausted
    pub async fn execute_task_with_retry(
        &self,
        task: &PlanTask,
        model: Arc<dyn Model>,
        max_retries: usize,
        base_delay_ms: u64,
    ) -> Result<TaskResult> {
        let mut last_error = None;
        
        for attempt in 0..=max_retries {
            match self.execute_task(task, model.clone()).await {
                Ok(result) => {
                    // If task succeeded, return immediately
                    if result.success {
                        return Ok(result);
                    }
                    
                    // Task execution returned but marked as failed
                    // Check if error is recoverable
                    let error_str = result.error.as_ref().map(|s| s.to_lowercase()).unwrap_or_default();
                    let is_recoverable = error_str.contains("429")
                        || error_str.contains("rate limit")
                        || error_str.contains("timeout")
                        || error_str.contains("network")
                        || error_str.contains("connection")
                        || error_str.contains("5");
                    
                    if !is_recoverable || attempt >= max_retries {
                        // Fatal error or retries exhausted
                        return Ok(result);
                    }
                    
                    // Recoverable error, will retry
                    last_error = Some(result.error.unwrap_or_default());
                }
                Err(e) => {
                    // Check error category
                    let category = e.category();
                    
                    match category {
                        ErrorCategory::Fatal => {
                            // Fatal error, don't retry
                            return Err(e);
                        }
                        ErrorCategory::Recoverable => {
                            if attempt >= max_retries {
                                // Retries exhausted
                                return Err(e);
                            }
                            // Will retry, store error
                            last_error = Some(e.to_string());
                        }
                    }
                }
            }
            
            // Calculate exponential backoff delay: base_delay_ms * 2^attempt
            if attempt < max_retries {
                let delay_ms = base_delay_ms * 2_u64.pow(attempt as u32);
                sleep(Duration::from_millis(delay_ms)).await;
            }
        }
        
        // All retries exhausted
        Err(ExecutionError::ModelExecution(
            last_error.unwrap_or_else(|| "Task execution failed after retries".to_string())
        ))
    }

    /// Executes a single task with the assigned agent.
    ///
    /// # Arguments
    /// * `task` - The task to execute
    /// * `model` - The model instance to use for execution
    ///
    /// # Returns
    /// Task execution result
    ///
    /// # Errors
    /// Returns an error if task execution fails
    pub async fn execute_task(&self, task: &PlanTask, model: Arc<dyn Model>) -> Result<TaskResult> {
        // Check if agent is assigned
        let agent_id = task.agent_id.as_ref().ok_or_else(|| {
            ExecutionError::AgentNotFound("No agent assigned to task".to_string())
        })?;

        // Discover and load agent
        let agents = self.agent_discovery.discover_all().map_err(|e| {
            ExecutionError::AgentNotFound(format!("Failed to discover agents: {}", e))
        })?;

        let agent =
            agents.get(agent_id).ok_or_else(|| ExecutionError::AgentNotFound(agent_id.clone()))?;

        // Load and render prompt
        let prompt_content = std::fs::read_to_string(&agent.prompt_path)?;
        let template = PromptTemplate::from_string(prompt_content);

        // Create context with task information
        let mut context = PromptContext::new();
        context.set("task_id", task.id.clone());
        context.set("task_title", task.title.clone());
        if let Some(desc) = &task.description {
            context.set("task_description", desc.clone());
        }

        // Inject context files if available
        if let Some(ref context_files) = self.config.context_files {
            context.set("context_files", context_files.clone());
        }

        let rendered =
            template.render(&context).map_err(|e| ExecutionError::Prompt(e.to_string()))?;

        // Execute with model
        match model.generate_text(&rendered, None).await {
            Ok(response) => {
                let tokens_used = response
                    .usage
                    .map(|u| (u.prompt_tokens as usize, u.completion_tokens as usize));

                Ok(TaskResult {
                    task_id: task.id.clone(),
                    success: true,
                    response: Some(response.content),
                    error: None,
                    tokens_used,
                })
            }
            Err(e) => Ok(TaskResult {
                task_id: task.id.clone(),
                success: false,
                response: None,
                error: Some(e.to_string()),
                tokens_used: None,
            }),
        }
    }

    /// Marks a task as completed and updates the manifest.
    ///
    /// # Arguments
    /// * `manifest` - The plan manifest to update
    /// * `iteration_id` - The iteration ID
    /// * `task_id` - The task ID to mark complete
    ///
    /// # Errors
    /// Returns an error if task or iteration is not found
    pub fn mark_task_complete(
        &self,
        manifest: &mut PlanManifest,
        iteration_id: &str,
        task_id: &str,
    ) -> Result<()> {
        let iteration = manifest.get_iteration_mut(iteration_id).ok_or_else(|| {
            ExecutionError::AgentNotFound(format!("Iteration not found: {}", iteration_id))
        })?;

        let task = iteration
            .get_task_mut(task_id)
            .ok_or_else(|| ExecutionError::AgentNotFound(format!("Task not found: {}", task_id)))?;

        task.completed = true;

        // Update iteration status
        self.update_iteration_status(iteration);

        Ok(())
    }

    /// Updates iteration status based on task completion.
    fn update_iteration_status(&self, iteration: &mut Iteration) {
        if iteration.is_complete() {
            iteration.status = PlanStatus::Completed;
        } else if iteration.tasks.iter().any(|t| t.completed) {
            iteration.status = PlanStatus::InProgress;
        }
    }

    /// Checks if all task dependencies are met.
    ///
    /// # Arguments
    /// * `manifest` - The plan manifest
    /// * `task` - The task to check dependencies for
    ///
    /// # Returns
    /// `Ok(())` if all dependencies are met, error otherwise
    pub fn check_dependencies(&self, manifest: &PlanManifest, task: &PlanTask) -> Result<()> {
        if !self.config.check_dependencies {
            return Ok(());
        }

        for dep_id in &task.dependencies {
            // Find the dependency task
            let mut found = false;
            let mut completed = false;

            for iteration in &manifest.iterations {
                if let Some(dep_task) = iteration.get_task(dep_id) {
                    found = true;
                    completed = dep_task.completed;
                    break;
                }
            }

            if !found {
                return Err(ExecutionError::DependencyNotMet(format!(
                    "Dependency task not found: {}",
                    dep_id
                )));
            }

            if !completed {
                return Err(ExecutionError::DependencyNotMet(format!(
                    "Dependency task not completed: {}",
                    dep_id
                )));
            }
        }

        Ok(())
    }

    /// Saves the manifest to disk.
    ///
    /// # Arguments
    /// * `manifest` - The manifest to save
    /// * `path` - The path to save to
    ///
    /// # Errors
    /// Returns an error if saving fails
    pub fn save_manifest(&self, manifest: &PlanManifest, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(manifest)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Loads a manifest from disk.
    ///
    /// # Arguments
    /// * `path` - The path to load from
    ///
    /// # Returns
    /// The loaded manifest
    ///
    /// # Errors
    /// Returns an error if loading fails
    pub fn load_manifest(&self, path: &Path) -> Result<PlanManifest> {
        let content = std::fs::read_to_string(path)?;
        let manifest = serde_json::from_str(&content)?;
        Ok(manifest)
    }

    /// Calculates overall plan progress.
    ///
    /// # Arguments
    /// * `manifest` - The plan manifest
    ///
    /// # Returns
    /// Progress as a percentage (0-100)
    pub fn calculate_progress(&self, manifest: &PlanManifest) -> u32 {
        let total_tasks = manifest.total_tasks();
        if total_tasks == 0 {
            return 0;
        }

        let completed_tasks: usize =
            manifest.iterations.iter().flat_map(|i| &i.tasks).filter(|t| t.completed).count();

        #[allow(clippy::cast_precision_loss)]
        let percentage = (completed_tasks as f64 / total_tasks as f64) * 100.0;
        percentage as u32
    }

    /// Checks if there are any incomplete tasks in the manifest.
    ///
    /// # Arguments
    /// * `manifest` - The plan manifest to check
    ///
    /// # Returns
    /// `true` if there are any pending or in-progress tasks, `false` otherwise
    pub fn has_incomplete_tasks(&self, manifest: &PlanManifest) -> bool {
        manifest.iterations.iter().any(|iteration| {
            iteration.tasks.iter().any(|task| !task.completed)
        })
    }

    /// Prints progress information for the current execution state.
    ///
    /// # Arguments
    /// * `manifest` - The plan manifest
    /// * `execution_iteration` - Current execution iteration number
    /// * `elapsed` - Elapsed time since execution started
    /// * `current_task_name` - Name of the currently active task (if any)
    pub fn print_progress(
        &self,
        manifest: &PlanManifest,
        execution_iteration: usize,
        elapsed: Duration,
        current_task_name: Option<&str>,
    ) {
        let total_tasks = manifest.total_tasks();
        let completed_tasks: usize = manifest
            .iterations
            .iter()
            .flat_map(|i| &i.tasks)
            .filter(|t| t.completed)
            .count();

        let elapsed_secs = elapsed.as_secs();
        let elapsed_min = elapsed_secs / 60;
        let elapsed_sec = elapsed_secs % 60;
        let elapsed_str = if elapsed_min > 0 {
            format!("{}m {}s", elapsed_min, elapsed_sec)
        } else {
            format!("{}s", elapsed_sec)
        };

        let current_task = current_task_name.unwrap_or("Evaluating next task");

        println!(
            "[Iteration {}] Progress: {}/{} tasks | Current: {} | Elapsed: {}",
            execution_iteration, completed_tasks, total_tasks, current_task, elapsed_str
        );
    }
}

impl Default for PlanExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::RequirementId;

    #[test]
    fn test_plan_executor_new() {
        let executor = PlanExecutor::new();
        assert!(executor.config.skip_completed);
        assert!(executor.config.check_dependencies);
    }

    #[test]
    fn test_mark_task_complete() {
        let executor = PlanExecutor::new();
        let mut manifest = PlanManifest::new(RequirementId::new(1), "Test Project".to_string());

        let mut iteration = Iteration::new(1, "Iteration 1".to_string());
        let task = PlanTask::new("I1", 1, "Task 1".to_string());
        iteration.add_task(task);
        manifest.add_iteration(iteration);

        let result = executor.mark_task_complete(&mut manifest, "I1", "I1.T1");
        assert!(result.is_ok());

        let iteration = manifest.get_iteration("I1").unwrap();
        let task = iteration.get_task("I1.T1").unwrap();
        assert!(task.completed);
        assert_eq!(iteration.status, PlanStatus::Completed);
    }

    #[test]
    fn test_calculate_progress() {
        let executor = PlanExecutor::new();
        let mut manifest = PlanManifest::new(RequirementId::new(1), "Test Project".to_string());

        let mut iteration = Iteration::new(1, "Iteration 1".to_string());
        let mut task1 = PlanTask::new("I1", 1, "Task 1".to_string());
        task1.completed = true;
        let task2 = PlanTask::new("I1", 2, "Task 2".to_string());
        iteration.add_task(task1);
        iteration.add_task(task2);
        manifest.add_iteration(iteration);

        let progress = executor.calculate_progress(&manifest);
        assert_eq!(progress, 50);
    }

    #[test]
    fn test_check_dependencies() {
        let executor = PlanExecutor::new();
        let mut manifest = PlanManifest::new(RequirementId::new(1), "Test Project".to_string());

        let mut iteration = Iteration::new(1, "Iteration 1".to_string());
        let mut task1 = PlanTask::new("I1", 1, "Task 1".to_string());
        task1.completed = true;

        let mut task2 = PlanTask::new("I1", 2, "Task 2".to_string());
        task2.dependencies = vec!["I1.T1".to_string()];

        iteration.add_task(task1);
        iteration.add_task(task2);
        manifest.add_iteration(iteration);

        let task = manifest.get_iteration("I1").unwrap().get_task("I1.T2").unwrap();
        let result = executor.check_dependencies(&manifest, task);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_dependencies_not_met() {
        let executor = PlanExecutor::new();
        let mut manifest = PlanManifest::new(RequirementId::new(1), "Test Project".to_string());

        let mut iteration = Iteration::new(1, "Iteration 1".to_string());
        let task1 = PlanTask::new("I1", 1, "Task 1".to_string()); // Not completed

        let mut task2 = PlanTask::new("I1", 2, "Task 2".to_string());
        task2.dependencies = vec!["I1.T1".to_string()];

        iteration.add_task(task1);
        iteration.add_task(task2);
        manifest.add_iteration(iteration);

        let task = manifest.get_iteration("I1").unwrap().get_task("I1.T2").unwrap();
        let result = executor.check_dependencies(&manifest, task);
        assert!(result.is_err());
    }
}
