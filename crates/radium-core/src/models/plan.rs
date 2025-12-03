//! Plan data structures for Radium.
//!
//! Defines the structure of plans, iterations, tasks, and manifests.

use crate::workspace::RequirementId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

/// Plan errors.
#[derive(Debug, Error)]
pub enum PlanError {
    /// Invalid plan structure.
    #[error("invalid plan structure: {0}")]
    InvalidStructure(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Plan not found.
    #[error("plan not found: {0}")]
    NotFound(String),
}

/// Result type for plan operations.
pub type Result<T> = std::result::Result<T, PlanError>;

/// Plan status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanStatus {
    /// Plan has not been started.
    NotStarted,

    /// Plan is currently in progress.
    InProgress,

    /// Plan is paused.
    Paused,

    /// Plan is blocked.
    Blocked,

    /// Plan is completed.
    Completed,

    /// Plan has failed.
    Failed,
}

impl std::fmt::Display for PlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotStarted => write!(f, "Not Started"),
            Self::InProgress => write!(f, "In Progress"),
            Self::Paused => write!(f, "Paused"),
            Self::Blocked => write!(f, "Blocked"),
            Self::Completed => write!(f, "Completed"),
            Self::Failed => write!(f, "Failed"),
        }
    }
}

/// Plan metadata stored in plan.json.
///
/// This is the primary metadata file for a plan, stored at
/// `<stage>/<plan-folder>/plan.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    /// Requirement ID (REQ-XXX).
    pub requirement_id: RequirementId,

    /// Project name.
    pub project_name: String,

    /// Folder name (REQ-XXX-slug).
    pub folder_name: String,

    /// Current stage (backlog, development, review, testing, docs).
    pub stage: String,

    /// Plan status.
    pub status: PlanStatus,

    /// Creation timestamp.
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,

    /// Last updated timestamp.
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,

    /// Original specification path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub specification_path: Option<PathBuf>,

    /// Total number of iterations.
    pub total_iterations: u32,

    /// Completed iterations count.
    pub completed_iterations: u32,

    /// Total number of tasks across all iterations.
    pub total_tasks: u32,

    /// Completed tasks count.
    pub completed_tasks: u32,

    /// Additional metadata.
    #[serde(flatten)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Plan {
    /// Create a new plan.
    pub fn new(
        requirement_id: RequirementId,
        project_name: String,
        folder_name: String,
        stage: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            requirement_id,
            project_name,
            folder_name,
            stage,
            status: PlanStatus::NotStarted,
            created_at: now,
            updated_at: now,
            specification_path: None,
            total_iterations: 0,
            completed_iterations: 0,
            total_tasks: 0,
            completed_tasks: 0,
            metadata: HashMap::new(),
        }
    }

    /// Calculate progress percentage (0-100).
    pub fn progress_percentage(&self) -> u32 {
        if self.total_tasks == 0 {
            0
        } else {
            ((f64::from(self.completed_tasks) / f64::from(self.total_tasks)) * 100.0) as u32
        }
    }

    /// Check if plan is complete.
    pub fn is_complete(&self) -> bool {
        self.status == PlanStatus::Completed
            || (self.total_tasks > 0 && self.completed_tasks >= self.total_tasks)
    }

    /// Update status based on task completion.
    pub fn update_status(&mut self) {
        if self.is_complete() {
            self.status = PlanStatus::Completed;
        } else if self.completed_tasks > 0 {
            self.status = PlanStatus::InProgress;
        }
        self.updated_at = Utc::now();
    }
}

/// Plan manifest stored in plan/plan_manifest.json.
///
/// Contains the detailed structure of the plan with iterations and tasks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanManifest {
    /// Requirement ID.
    pub requirement_id: RequirementId,

    /// Project name.
    pub project_name: String,

    /// Iterations in the plan.
    pub iterations: Vec<Iteration>,

    /// Plan-level metadata.
    #[serde(flatten)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl PlanManifest {
    /// Create a new plan manifest.
    pub fn new(requirement_id: RequirementId, project_name: String) -> Self {
        Self { requirement_id, project_name, iterations: Vec::new(), metadata: HashMap::new() }
    }

    /// Add an iteration to the manifest.
    pub fn add_iteration(&mut self, iteration: Iteration) {
        self.iterations.push(iteration);
    }

    /// Get total number of tasks across all iterations.
    pub fn total_tasks(&self) -> usize {
        self.iterations.iter().map(|i| i.tasks.len()).sum()
    }

    /// Get an iteration by ID.
    pub fn get_iteration(&self, id: &str) -> Option<&Iteration> {
        self.iterations.iter().find(|i| i.id == id)
    }

    /// Get a mutable iteration by ID.
    pub fn get_iteration_mut(&mut self, id: &str) -> Option<&mut Iteration> {
        self.iterations.iter_mut().find(|i| i.id == id)
    }
}

/// Iteration within a plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Iteration {
    /// Iteration ID (e.g., "I1", "I2").
    pub id: String,

    /// Iteration number.
    pub number: u32,

    /// Iteration name/title.
    pub name: String,

    /// Iteration description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Iteration goal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,

    /// Tasks in this iteration.
    pub tasks: Vec<PlanTask>,

    /// Iteration status.
    pub status: PlanStatus,

    /// Iteration metadata.
    #[serde(flatten)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Iteration {
    /// Create a new iteration.
    pub fn new(number: u32, name: String) -> Self {
        Self {
            id: format!("I{}", number),
            number,
            name,
            description: None,
            goal: None,
            tasks: Vec::new(),
            status: PlanStatus::NotStarted,
            metadata: HashMap::new(),
        }
    }

    /// Add a task to the iteration.
    pub fn add_task(&mut self, task: PlanTask) {
        self.tasks.push(task);
    }

    /// Get a task by ID.
    pub fn get_task(&self, id: &str) -> Option<&PlanTask> {
        self.tasks.iter().find(|t| t.id == id)
    }

    /// Get a mutable task by ID.
    pub fn get_task_mut(&mut self, id: &str) -> Option<&mut PlanTask> {
        self.tasks.iter_mut().find(|t| t.id == id)
    }

    /// Calculate completion percentage.
    pub fn completion_percentage(&self) -> u32 {
        if self.tasks.is_empty() {
            0
        } else {
            let completed = self.tasks.iter().filter(|t| t.completed).count();
            #[allow(clippy::cast_precision_loss)]
            let percentage = (completed as f64 / self.tasks.len() as f64) * 100.0;
            percentage as u32
        }
    }

    /// Check if iteration is complete.
    pub fn is_complete(&self) -> bool {
        !self.tasks.is_empty() && self.tasks.iter().all(|t| t.completed)
    }
}

/// Task within an iteration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanTask {
    /// Task ID (e.g., "I1.T1", "I2.T3").
    pub id: String,

    /// Task number within iteration.
    pub number: u32,

    /// Task title.
    pub title: String,

    /// Task description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Task completion status.
    #[serde(default)]
    pub completed: bool,

    /// Agent ID to execute this task.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,

    /// Dependencies (other task IDs that must complete first).
    #[serde(default)]
    pub dependencies: Vec<String>,

    /// Acceptance criteria.
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,

    /// Task metadata.
    #[serde(flatten)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl PlanTask {
    /// Create a new task.
    pub fn new(iteration_id: &str, number: u32, title: String) -> Self {
        Self {
            id: format!("{}.T{}", iteration_id, number),
            number,
            title,
            description: None,
            completed: false,
            agent_id: None,
            dependencies: Vec::new(),
            acceptance_criteria: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Mark task as completed.
    pub fn mark_completed(&mut self) {
        self.completed = true;
    }

    /// Check if dependencies are satisfied.
    pub fn dependencies_satisfied(&self, manifest: &PlanManifest) -> bool {
        if self.dependencies.is_empty() {
            return true;
        }

        for dep_id in &self.dependencies {
            let mut found = false;
            for iteration in &manifest.iterations {
                if let Some(task) = iteration.get_task(dep_id) {
                    if !task.completed {
                        return false;
                    }
                    found = true;
                    break;
                }
            }
            // Dependency not found, assume satisfied - no action needed
            let _ = found;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_plan_new() {
        let req_id = RequirementId::from_str("REQ-001").unwrap();
        let plan = Plan::new(
            req_id,
            "Test Project".to_string(),
            "REQ-001-test".to_string(),
            "backlog".to_string(),
        );

        assert_eq!(plan.requirement_id, req_id);
        assert_eq!(plan.project_name, "Test Project");
        assert_eq!(plan.status, PlanStatus::NotStarted);
        assert_eq!(plan.total_tasks, 0);
        assert_eq!(plan.completed_tasks, 0);
    }

    #[test]
    fn test_plan_progress() {
        let req_id = RequirementId::from_str("REQ-001").unwrap();
        let mut plan = Plan::new(
            req_id,
            "Test".to_string(),
            "REQ-001-test".to_string(),
            "backlog".to_string(),
        );

        plan.total_tasks = 10;
        plan.completed_tasks = 5;

        assert_eq!(plan.progress_percentage(), 50);
        assert!(!plan.is_complete());

        plan.completed_tasks = 10;
        assert_eq!(plan.progress_percentage(), 100);
        assert!(plan.is_complete());
    }

    #[test]
    fn test_plan_manifest() {
        let req_id = RequirementId::from_str("REQ-001").unwrap();
        let mut manifest = PlanManifest::new(req_id, "Test Project".to_string());

        let mut iter1 = Iteration::new(1, "Iteration 1".to_string());
        iter1.add_task(PlanTask::new("I1", 1, "Task 1".to_string()));
        iter1.add_task(PlanTask::new("I1", 2, "Task 2".to_string()));

        manifest.add_iteration(iter1);

        assert_eq!(manifest.iterations.len(), 1);
        assert_eq!(manifest.total_tasks(), 2);
    }

    #[test]
    fn test_iteration() {
        let mut iter = Iteration::new(1, "Iteration 1".to_string());
        assert_eq!(iter.id, "I1");
        assert_eq!(iter.number, 1);

        iter.add_task(PlanTask::new("I1", 1, "Task 1".to_string()));
        iter.add_task(PlanTask::new("I1", 2, "Task 2".to_string()));

        assert_eq!(iter.tasks.len(), 2);
        assert_eq!(iter.completion_percentage(), 0);
        assert!(!iter.is_complete());

        iter.tasks[0].mark_completed();
        assert_eq!(iter.completion_percentage(), 50);

        iter.tasks[1].mark_completed();
        assert_eq!(iter.completion_percentage(), 100);
        assert!(iter.is_complete());
    }

    #[test]
    fn test_plan_task() {
        let task = PlanTask::new("I1", 1, "Test Task".to_string());
        assert_eq!(task.id, "I1.T1");
        assert_eq!(task.number, 1);
        assert_eq!(task.title, "Test Task");
        assert!(!task.completed);
    }

    #[test]
    fn test_task_dependencies() {
        let req_id = RequirementId::from_str("REQ-001").unwrap();
        let mut manifest = PlanManifest::new(req_id, "Test".to_string());

        let mut iter = Iteration::new(1, "Iteration 1".to_string());
        let mut task1 = PlanTask::new("I1", 1, "Task 1".to_string());
        let mut task2 = PlanTask::new("I1", 2, "Task 2".to_string());
        task2.dependencies.push("I1.T1".to_string());

        iter.add_task(task1.clone());
        iter.add_task(task2.clone());
        manifest.add_iteration(iter);

        // Task 2 depends on Task 1, which is not complete
        assert!(!task2.dependencies_satisfied(&manifest));

        // Complete Task 1
        task1.mark_completed();
        manifest.iterations[0].tasks[0] = task1;

        // Now Task 2 dependencies are satisfied
        assert!(task2.dependencies_satisfied(&manifest));
    }

    #[test]
    fn test_plan_serialization() {
        let req_id = RequirementId::from_str("REQ-042").unwrap();
        let plan = Plan::new(
            req_id,
            "Test Project".to_string(),
            "REQ-042-test".to_string(),
            "backlog".to_string(),
        );

        let json = serde_json::to_string_pretty(&plan).unwrap();
        let deserialized: Plan = serde_json::from_str(&json).unwrap();

        assert_eq!(plan.requirement_id, deserialized.requirement_id);
        assert_eq!(plan.project_name, deserialized.project_name);
    }
}
