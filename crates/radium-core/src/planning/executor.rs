//! Plan execution with state persistence and progress tracking.

use crate::models::{Iteration, PlanManifest, PlanStatus, PlanTask};
use crate::{AgentDiscovery, PromptContext, PromptTemplate};
use radium_abstraction::Model;
use std::path::Path;
use std::sync::Arc;

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
