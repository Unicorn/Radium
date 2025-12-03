//! Workflow data structures for Radium Core.
//!
//! This module defines the core data structures for workflows, including
//! the Workflow struct itself, workflow steps, runtime state, and
//! conversion utilities for working with gRPC protocol definitions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::models::proto_convert;
use crate::proto;

/// Runtime state of a workflow.
///
/// Tracks the current execution state of a workflow, allowing the system
/// to monitor and manage workflow lifecycle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum WorkflowState {
    /// Workflow is ready but not currently executing.
    #[default]
    Idle,
    /// Workflow is currently executing.
    Running,
    /// Workflow execution has been paused.
    Paused,
    /// Workflow encountered an error during execution.
    Error(String),
    /// Workflow execution completed successfully.
    Completed,
}

/// A step in a workflow.
///
/// Represents a single step within a workflow, which references a task
/// to be executed and may include step-specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// Unique identifier for the step within the workflow.
    pub id: String,
    /// Human-readable name for the step.
    pub name: String,
    /// Description of what this step does.
    pub description: String,
    /// ID of the task to execute for this step.
    pub task_id: String,
    /// Optional step-specific configuration (JSON string).
    pub config_json: Option<String>,
    /// Order/sequence number of this step in the workflow.
    pub order: u32,
}

impl WorkflowStep {
    /// Creates a new workflow step.
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the step
    /// * `name` - Human-readable name
    /// * `description` - Description of the step
    /// * `task_id` - ID of the task to execute
    /// * `order` - Sequence order in the workflow
    ///
    /// # Returns
    /// A new `WorkflowStep` with no configuration.
    pub fn new(id: String, name: String, description: String, task_id: String, order: u32) -> Self {
        Self { id, name, description, task_id, config_json: None, order }
    }

    /// Validates the workflow step.
    ///
    /// # Returns
    /// `Ok(())` if the step is valid, or a `WorkflowError` if invalid.
    ///
    /// # Errors
    /// * `WorkflowError::InvalidStep` - If the step data is invalid
    pub fn validate(&self) -> Result<(), WorkflowError> {
        if self.id.is_empty() {
            return Err(WorkflowError::InvalidStep("step id cannot be empty".to_string()));
        }

        if self.name.is_empty() {
            return Err(WorkflowError::InvalidStep("step name cannot be empty".to_string()));
        }

        if self.task_id.is_empty() {
            return Err(WorkflowError::InvalidStep("task_id cannot be empty".to_string()));
        }

        Ok(())
    }
}

// Conversion from proto::WorkflowStep to WorkflowStep
impl TryFrom<proto::WorkflowStep> for WorkflowStep {
    type Error = WorkflowError;

    fn try_from(proto_step: proto::WorkflowStep) -> Result<Self, Self::Error> {
        Ok(WorkflowStep {
            id: proto_step.id,
            name: proto_step.name,
            description: proto_step.description,
            task_id: proto_step.task_id,
            config_json: if proto_step.config_json.is_empty() {
                None
            } else {
                Some(proto_step.config_json)
            },
            order: proto_step.order as u32,
        })
    }
}

// Conversion from WorkflowStep to proto::WorkflowStep
impl From<WorkflowStep> for proto::WorkflowStep {
    #[allow(clippy::cast_possible_wrap)] // order is always small positive
    fn from(step: WorkflowStep) -> Self {
        proto::WorkflowStep {
            id: step.id,
            name: step.name,
            description: step.description,
            task_id: step.task_id,
            config_json: step.config_json.unwrap_or_default(),
            order: step.order as i32,
        }
    }
}

/// Core workflow data structure.
///
/// Represents a workflow in the Radium system, which is a sequence of
/// steps that define a process for agents to follow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    /// Unique identifier for the workflow.
    pub id: String,
    /// Human-readable name for the workflow.
    pub name: String,
    /// Description of the workflow's purpose.
    pub description: String,
    /// Steps that make up this workflow.
    pub steps: Vec<WorkflowStep>,
    /// Current runtime state of the workflow.
    pub state: WorkflowState,
    /// Timestamp when the workflow was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the workflow was last updated.
    pub updated_at: DateTime<Utc>,
}

impl Workflow {
    /// Creates a new workflow with the specified properties.
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the workflow
    /// * `name` - Human-readable name
    /// * `description` - Description of the workflow
    ///
    /// # Returns
    /// A new `Workflow` with `Idle` state, no steps, and current timestamps.
    pub fn new(id: String, name: String, description: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            description,
            steps: Vec::new(),
            state: WorkflowState::default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Adds a step to the workflow.
    ///
    /// # Arguments
    /// * `step` - The workflow step to add
    ///
    /// # Errors
    /// * `WorkflowError::InvalidStep` - If the step is invalid
    pub fn add_step(&mut self, step: WorkflowStep) -> Result<(), WorkflowError> {
        step.validate()?;
        self.steps.push(step);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Validates the workflow data.
    ///
    /// # Returns
    /// `Ok(())` if the workflow is valid, or a `WorkflowError` if invalid.
    ///
    /// # Errors
    /// * `WorkflowError::InvalidWorkflow` - If the workflow data is invalid
    /// * `WorkflowError::InvalidStep` - If any step is invalid
    pub fn validate(&self) -> Result<(), WorkflowError> {
        if self.id.is_empty() {
            return Err(WorkflowError::InvalidWorkflow("id cannot be empty".to_string()));
        }

        if self.name.is_empty() {
            return Err(WorkflowError::InvalidWorkflow("name cannot be empty".to_string()));
        }

        // Validate all steps
        for step in &self.steps {
            step.validate()?;
        }

        // Check for duplicate step IDs
        let mut step_ids = std::collections::HashSet::new();
        for step in &self.steps {
            if !step_ids.insert(&step.id) {
                return Err(WorkflowError::InvalidWorkflow(format!(
                    "duplicate step id: {}",
                    step.id
                )));
            }
        }

        Ok(())
    }

    /// Updates the workflow's state and sets the updated_at timestamp.
    ///
    /// # Arguments
    /// * `state` - The new state for the workflow
    pub fn set_state(&mut self, state: WorkflowState) {
        self.state = state;
        self.updated_at = Utc::now();
    }
}

/// Builder for creating `Workflow` instances.
///
/// Provides a fluent interface for constructing workflows with optional fields.
#[derive(Debug, Default)]
pub struct WorkflowBuilder {
    id: Option<String>,
    name: Option<String>,
    description: Option<String>,
    steps: Vec<WorkflowStep>,
    state: Option<WorkflowState>,
}

impl WorkflowBuilder {
    /// Creates a new `WorkflowBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the workflow ID.
    #[must_use]
    pub fn id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the workflow name.
    #[must_use]
    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets the workflow description.
    #[must_use]
    pub fn description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Adds a step to the workflow.
    #[must_use]
    pub fn add_step(mut self, step: WorkflowStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Sets the steps for the workflow.
    #[must_use]
    pub fn steps(mut self, steps: Vec<WorkflowStep>) -> Self {
        self.steps = steps;
        self
    }

    /// Sets the initial workflow state.
    #[must_use]
    pub fn state(mut self, state: WorkflowState) -> Self {
        self.state = Some(state);
        self
    }

    /// Builds the `Workflow` from the builder.
    ///
    /// # Returns
    /// `Ok(Workflow)` if all required fields are set, or a `WorkflowError` if validation fails.
    ///
    /// # Errors
    /// * `WorkflowError::InvalidWorkflow` - If required fields are missing or invalid
    /// * `WorkflowError::InvalidStep` - If any step is invalid
    pub fn build(self) -> Result<Workflow, WorkflowError> {
        let id =
            self.id.ok_or_else(|| WorkflowError::InvalidWorkflow("id is required".to_string()))?;
        let name = self
            .name
            .ok_or_else(|| WorkflowError::InvalidWorkflow("name is required".to_string()))?;
        let description = self
            .description
            .ok_or_else(|| WorkflowError::InvalidWorkflow("description is required".to_string()))?;

        let now = Utc::now();
        let workflow = Workflow {
            id,
            name,
            description,
            steps: self.steps,
            state: self.state.unwrap_or_default(),
            created_at: now,
            updated_at: now,
        };

        workflow.validate()?;
        Ok(workflow)
    }
}

/// Errors that can occur when working with workflows.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum WorkflowError {
    /// Invalid workflow data.
    #[error("Invalid workflow: {0}")]
    InvalidWorkflow(String),

    /// Invalid workflow step.
    #[error("Invalid step: {0}")]
    InvalidStep(String),

    /// Error during proto conversion.
    #[error("Proto conversion error: {0}")]
    ProtoConversion(String),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(String),
}

impl From<serde_json::Error> for WorkflowError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err.to_string())
    }
}

// Conversion from proto::Workflow to Workflow
impl TryFrom<proto::Workflow> for Workflow {
    type Error = WorkflowError;

    fn try_from(proto_workflow: proto::Workflow) -> Result<Self, Self::Error> {
        let state = proto_convert::json_from_str(&proto_workflow.state)?;
        let created_at =
            proto_convert::parse_rfc3339_timestamp(&proto_workflow.created_at, "created_at")
                .map_err(|e| WorkflowError::ProtoConversion(e))?;
        let updated_at =
            proto_convert::parse_rfc3339_timestamp(&proto_workflow.updated_at, "updated_at")
                .map_err(|e| WorkflowError::ProtoConversion(e))?;

        let steps = proto_workflow
            .steps
            .into_iter()
            .map(WorkflowStep::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Workflow {
            id: proto_workflow.id,
            name: proto_workflow.name,
            description: proto_workflow.description,
            steps,
            state,
            created_at,
            updated_at,
        })
    }
}

// Conversion from Workflow to proto::Workflow
impl From<Workflow> for proto::Workflow {
    fn from(workflow: Workflow) -> Self {
        let steps = workflow.steps.into_iter().map(proto::WorkflowStep::from).collect();
        let state = proto_convert::json_to_string(&workflow.state, "");

        proto::Workflow {
            id: workflow.id,
            name: workflow.name,
            description: workflow.description,
            steps,
            state,
            created_at: proto_convert::format_rfc3339_timestamp(&workflow.created_at),
            updated_at: proto_convert::format_rfc3339_timestamp(&workflow.updated_at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_state_default() {
        let state = WorkflowState::default();
        assert_eq!(state, WorkflowState::Idle);
    }

    #[test]
    fn test_workflow_step_new() {
        let step = WorkflowStep::new(
            "step-1".to_string(),
            "Step 1".to_string(),
            "First step".to_string(),
            "task-1".to_string(),
            0,
        );
        assert_eq!(step.id, "step-1");
        assert_eq!(step.name, "Step 1");
        assert_eq!(step.task_id, "task-1");
        assert_eq!(step.order, 0);
        assert!(step.config_json.is_none());
    }

    #[test]
    fn test_workflow_step_validate_success() {
        let step = WorkflowStep::new(
            "step-1".to_string(),
            "Step 1".to_string(),
            "First step".to_string(),
            "task-1".to_string(),
            0,
        );
        assert!(step.validate().is_ok());
    }

    #[test]
    fn test_workflow_step_validate_empty_id() {
        let step = WorkflowStep::new(
            "".to_string(),
            "Step 1".to_string(),
            "First step".to_string(),
            "task-1".to_string(),
            0,
        );
        assert!(step.validate().is_err());
    }

    #[test]
    fn test_workflow_step_validate_empty_task_id() {
        let step = WorkflowStep::new(
            "step-1".to_string(),
            "Step 1".to_string(),
            "First step".to_string(),
            "".to_string(),
            0,
        );
        assert!(step.validate().is_err());
    }

    #[test]
    fn test_workflow_new() {
        let workflow = Workflow::new(
            "workflow-1".to_string(),
            "Test Workflow".to_string(),
            "A test workflow".to_string(),
        );

        assert_eq!(workflow.id, "workflow-1");
        assert_eq!(workflow.name, "Test Workflow");
        assert_eq!(workflow.description, "A test workflow");
        assert!(workflow.steps.is_empty());
        assert_eq!(workflow.state, WorkflowState::Idle);
    }

    #[test]
    fn test_workflow_add_step() {
        let mut workflow = Workflow::new(
            "workflow-1".to_string(),
            "Test Workflow".to_string(),
            "A test workflow".to_string(),
        );

        let step = WorkflowStep::new(
            "step-1".to_string(),
            "Step 1".to_string(),
            "First step".to_string(),
            "task-1".to_string(),
            0,
        );

        assert!(workflow.add_step(step).is_ok());
        assert_eq!(workflow.steps.len(), 1);
    }

    #[test]
    fn test_workflow_validate_success() {
        let mut workflow = Workflow::new(
            "workflow-1".to_string(),
            "Test Workflow".to_string(),
            "A test workflow".to_string(),
        );

        let step = WorkflowStep::new(
            "step-1".to_string(),
            "Step 1".to_string(),
            "First step".to_string(),
            "task-1".to_string(),
            0,
        );
        workflow.add_step(step).unwrap();

        assert!(workflow.validate().is_ok());
    }

    #[test]
    fn test_workflow_validate_empty_id() {
        let workflow = Workflow::new(
            "".to_string(),
            "Test Workflow".to_string(),
            "A test workflow".to_string(),
        );
        assert!(workflow.validate().is_err());
    }

    #[test]
    fn test_workflow_validate_duplicate_step_ids() {
        let mut workflow = Workflow::new(
            "workflow-1".to_string(),
            "Test Workflow".to_string(),
            "A test workflow".to_string(),
        );

        let step1 = WorkflowStep::new(
            "step-1".to_string(),
            "Step 1".to_string(),
            "First step".to_string(),
            "task-1".to_string(),
            0,
        );
        workflow.add_step(step1).unwrap();

        let step2 = WorkflowStep::new(
            "step-1".to_string(), // Duplicate ID
            "Step 2".to_string(),
            "Second step".to_string(),
            "task-2".to_string(),
            1,
        );
        workflow.add_step(step2).unwrap();

        assert!(workflow.validate().is_err());
    }

    #[test]
    fn test_workflow_set_state() {
        let mut workflow = Workflow::new(
            "workflow-1".to_string(),
            "Test Workflow".to_string(),
            "A test workflow".to_string(),
        );

        let initial_updated_at = workflow.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(10));
        workflow.set_state(WorkflowState::Running);

        assert_eq!(workflow.state, WorkflowState::Running);
        assert!(workflow.updated_at > initial_updated_at);
    }

    #[test]
    fn test_proto_workflow_to_workflow() {
        let state = serde_json::to_string(&WorkflowState::Idle).unwrap();

        let proto_workflow = proto::Workflow {
            id: "workflow-1".to_string(),
            name: "Test Workflow".to_string(),
            description: "A test workflow".to_string(),
            steps: vec![proto::WorkflowStep {
                id: "step-1".to_string(),
                name: "Step 1".to_string(),
                description: "First step".to_string(),
                task_id: "task-1".to_string(),
                config_json: "".to_string(),
                order: 0,
            }],
            state,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
        };

        let workflow = Workflow::try_from(proto_workflow).unwrap();
        assert_eq!(workflow.id, "workflow-1");
        assert_eq!(workflow.name, "Test Workflow");
        assert_eq!(workflow.description, "A test workflow");
        assert_eq!(workflow.steps.len(), 1);
        assert_eq!(workflow.steps[0].task_id, "task-1");
    }

    #[test]
    fn test_proto_workflow_to_workflow_missing_id() {
        let state = serde_json::to_string(&WorkflowState::Idle).unwrap();

        let proto_workflow = proto::Workflow {
            id: "".to_string(),
            name: "Test Workflow".to_string(),
            description: "A test workflow".to_string(),
            steps: vec![],
            state,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
        };

        // Workflow with empty ID should still parse from proto, but validation would fail
        let workflow = Workflow::try_from(proto_workflow).unwrap();
        assert!(workflow.validate().is_err());
    }

    #[test]
    fn test_workflow_to_proto_workflow() {
        let mut workflow = Workflow::new(
            "workflow-1".to_string(),
            "Test Workflow".to_string(),
            "A test workflow".to_string(),
        );

        let step = WorkflowStep::new(
            "step-1".to_string(),
            "Step 1".to_string(),
            "First step".to_string(),
            "task-1".to_string(),
            0,
        );
        workflow.add_step(step).unwrap();

        let proto_workflow = proto::Workflow::from(workflow);
        assert_eq!(proto_workflow.id, "workflow-1");
        assert_eq!(proto_workflow.name, "Test Workflow");
        assert_eq!(proto_workflow.description, "A test workflow");
        assert_eq!(proto_workflow.steps.len(), 1);
        assert_eq!(proto_workflow.steps[0].id, "step-1");
    }

    #[test]
    fn test_workflow_proto_round_trip() {
        let mut original_workflow = Workflow::new(
            "workflow-1".to_string(),
            "Test Workflow".to_string(),
            "A test workflow".to_string(),
        );

        let step = WorkflowStep::new(
            "step-1".to_string(),
            "Step 1".to_string(),
            "First step".to_string(),
            "task-1".to_string(),
            0,
        );
        original_workflow.add_step(step).unwrap();

        let proto_workflow = proto::Workflow::from(original_workflow.clone());
        let converted_workflow = Workflow::try_from(proto_workflow).unwrap();

        assert_eq!(original_workflow.id, converted_workflow.id);
        assert_eq!(original_workflow.name, converted_workflow.name);
        assert_eq!(original_workflow.description, converted_workflow.description);
        assert_eq!(original_workflow.steps.len(), converted_workflow.steps.len());
    }

    #[test]
    fn test_workflow_builder_minimal() {
        let workflow = WorkflowBuilder::new()
            .id("test-workflow".to_string())
            .name("Test Workflow".to_string())
            .description("A test workflow".to_string())
            .build()
            .expect("Should build workflow successfully");

        assert_eq!(workflow.id, "test-workflow");
        assert_eq!(workflow.name, "Test Workflow");
        assert_eq!(workflow.description, "A test workflow");
        assert_eq!(workflow.state, WorkflowState::Idle);
        assert!(workflow.steps.is_empty());
    }

    #[test]
    fn test_workflow_builder_with_state() {
        let workflow = WorkflowBuilder::new()
            .id("test-workflow".to_string())
            .name("Test Workflow".to_string())
            .description("A test workflow".to_string())
            .state(WorkflowState::Running)
            .build()
            .expect("Should build workflow successfully");

        assert_eq!(workflow.state, WorkflowState::Running);
    }

    #[test]
    fn test_workflow_builder_with_step() {
        let step = WorkflowStep::new(
            "step-1".to_string(),
            "Step 1".to_string(),
            "First step".to_string(),
            "task-1".to_string(),
            0,
        );
        let workflow = WorkflowBuilder::new()
            .id("test-workflow".to_string())
            .name("Test Workflow".to_string())
            .description("A test workflow".to_string())
            .add_step(step)
            .build()
            .expect("Should build workflow successfully");

        assert_eq!(workflow.steps.len(), 1);
        assert_eq!(workflow.steps[0].id, "step-1");
        assert_eq!(workflow.steps[0].task_id, "task-1");
    }

    #[test]
    fn test_workflow_builder_missing_id() {
        let result = WorkflowBuilder::new()
            .name("Test Workflow".to_string())
            .description("A test workflow".to_string())
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("id is required"));
    }

    #[test]
    fn test_workflow_builder_missing_name() {
        let result = WorkflowBuilder::new()
            .id("test-workflow".to_string())
            .description("A test workflow".to_string())
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("name is required"));
    }

    #[test]
    fn test_workflow_builder_validation() {
        let result = WorkflowBuilder::new()
            .id("".to_string()) // Empty ID should fail validation
            .name("Test Workflow".to_string())
            .description("A test workflow".to_string())
            .build();

        assert!(result.is_err());
    }
}
