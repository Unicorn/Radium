//! Workflow state container
//!
//! Defines the state structure for workflow execution, including
//! input/output variables, workflow-scoped state, and execution metadata.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::schema::variables::{VariableDefinition, VariableType, VariableValue};

/// Current execution state of a workflow
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionState {
    /// Workflow is pending start
    #[default]
    Pending,
    /// Workflow is actively running
    Running,
    /// Workflow is paused (waiting for signal/timer)
    Paused,
    /// Workflow completed successfully
    Completed,
    /// Workflow failed with error
    Failed,
    /// Workflow was cancelled
    Cancelled,
    /// Workflow timed out
    TimedOut,
}

impl ExecutionState {
    /// Check if this is a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Cancelled | Self::TimedOut
        )
    }

    /// Check if the workflow is actively running
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running | Self::Paused)
    }

    /// Get the TypeScript string representation
    pub fn to_typescript(&self) -> &'static str {
        match self {
            Self::Pending => "'pending'",
            Self::Running => "'running'",
            Self::Paused => "'paused'",
            Self::Completed => "'completed'",
            Self::Failed => "'failed'",
            Self::Cancelled => "'cancelled'",
            Self::TimedOut => "'timedOut'",
        }
    }
}

impl std::fmt::Display for ExecutionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Running => write!(f, "running"),
            Self::Paused => write!(f, "paused"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Cancelled => write!(f, "cancelled"),
            Self::TimedOut => write!(f, "timedOut"),
        }
    }
}

/// Error information captured during workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateError {
    /// Error code for categorization
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Additional error details (JSON)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    /// When the error occurred
    pub occurred_at: DateTime<Utc>,
    /// Stack trace or activity/node context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<Vec<String>>,
}

impl StateError {
    /// Create a new state error
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
            occurred_at: Utc::now(),
            stack: None,
        }
    }

    /// Add details to the error
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    /// Add stack context to the error
    pub fn with_stack(mut self, stack: Vec<String>) -> Self {
        self.stack = Some(stack);
        self
    }

    /// Create a type mismatch error
    pub fn type_mismatch(
        variable_name: &str,
        expected: &VariableType,
        actual: &VariableType,
    ) -> Self {
        Self::new(
            "TYPE_MISMATCH",
            format!(
                "Variable '{}' expects type {:?}, got {:?}",
                variable_name, expected, actual
            ),
        )
    }
}

/// Complete workflow state container
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowState {
    /// Workflow instance ID
    pub workflow_id: String,

    /// State version for optimistic concurrency
    pub version: u64,

    /// Input variables (immutable after start)
    pub input: HashMap<String, VariableValue>,

    /// Output variables (set at completion)
    pub output: HashMap<String, VariableValue>,

    /// Workflow-scoped variables (mutable during execution)
    pub variables: HashMap<String, VariableValue>,

    /// Variable definitions for type checking
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub definitions: Vec<VariableDefinition>,

    /// Current execution state
    pub execution_state: ExecutionState,

    /// Error information if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<StateError>,

    /// When the workflow was created
    pub created_at: DateTime<Utc>,

    /// When the state was last updated
    pub updated_at: DateTime<Utc>,

    /// Arbitrary metadata
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
}

impl WorkflowState {
    /// Create a new workflow state
    pub fn new(workflow_id: impl Into<String>, definitions: Vec<VariableDefinition>) -> Self {
        let now = Utc::now();
        Self {
            workflow_id: workflow_id.into(),
            version: 1,
            input: HashMap::new(),
            output: HashMap::new(),
            variables: HashMap::new(),
            definitions,
            execution_state: ExecutionState::Pending,
            error: None,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    /// Create state with initial input
    pub fn with_input(mut self, input: HashMap<String, VariableValue>) -> Self {
        self.input = input;
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Get a variable value (checks variables first, then input)
    pub fn get(&self, name: &str) -> Option<&VariableValue> {
        self.variables.get(name).or_else(|| self.input.get(name))
    }

    /// Get a variable value with type coercion
    pub fn get_as<T>(&self, name: &str) -> Option<T>
    where
        T: FromVariableValue,
    {
        self.get(name).and_then(|v| T::from_variable_value(v))
    }

    /// Set a variable value with type validation
    pub fn set(
        &mut self,
        name: impl Into<String>,
        value: VariableValue,
    ) -> Result<(), StateError> {
        let name = name.into();

        // Validate type if definition exists
        if let Some(def) = self.definitions.iter().find(|d| d.name == name) {
            if let Err(e) = def.validate_value(&value) {
                return Err(StateError::new("VALIDATION_ERROR", e.to_string()));
            }
        }

        self.variables.insert(name, value);
        self.updated_at = Utc::now();
        self.version += 1;
        Ok(())
    }

    /// Set output variable
    pub fn set_output(
        &mut self,
        name: impl Into<String>,
        value: VariableValue,
    ) -> Result<(), StateError> {
        let name = name.into();
        self.output.insert(name, value);
        self.updated_at = Utc::now();
        self.version += 1;
        Ok(())
    }

    /// Transition to a new execution state
    pub fn transition_to(&mut self, state: ExecutionState) -> Result<(), StateError> {
        // Validate state transitions
        let valid_transition = match (&self.execution_state, &state) {
            (ExecutionState::Pending, ExecutionState::Running) => true,
            (ExecutionState::Running, ExecutionState::Paused) => true,
            (ExecutionState::Running, ExecutionState::Completed) => true,
            (ExecutionState::Running, ExecutionState::Failed) => true,
            (ExecutionState::Running, ExecutionState::Cancelled) => true,
            (ExecutionState::Running, ExecutionState::TimedOut) => true,
            (ExecutionState::Paused, ExecutionState::Running) => true,
            (ExecutionState::Paused, ExecutionState::Cancelled) => true,
            _ => false,
        };

        if !valid_transition {
            return Err(StateError::new(
                "INVALID_TRANSITION",
                format!(
                    "Cannot transition from {} to {}",
                    self.execution_state, state
                ),
            ));
        }

        self.execution_state = state;
        self.updated_at = Utc::now();
        self.version += 1;
        Ok(())
    }

    /// Mark workflow as failed with error
    pub fn fail(&mut self, error: StateError) {
        self.error = Some(error);
        self.execution_state = ExecutionState::Failed;
        self.updated_at = Utc::now();
        self.version += 1;
    }

    /// Check if workflow is complete (terminal state)
    pub fn is_complete(&self) -> bool {
        self.execution_state.is_terminal()
    }

    /// Check if workflow is actively running
    pub fn is_running(&self) -> bool {
        self.execution_state.is_running()
    }

    /// Get all variable names that are defined
    pub fn defined_variables(&self) -> Vec<&str> {
        self.definitions.iter().map(|d| d.name.as_str()).collect()
    }

    /// Initialize all variables to their defaults
    pub fn initialize_defaults(&mut self) {
        for def in &self.definitions {
            if !self.variables.contains_key(&def.name) && !self.input.contains_key(&def.name) {
                self.variables.insert(def.name.clone(), def.effective_default());
            }
        }
    }
}

/// Trait for converting from VariableValue
pub trait FromVariableValue: Sized {
    fn from_variable_value(value: &VariableValue) -> Option<Self>;
}

impl FromVariableValue for String {
    fn from_variable_value(value: &VariableValue) -> Option<Self> {
        value.as_string()
    }
}

impl FromVariableValue for i64 {
    fn from_variable_value(value: &VariableValue) -> Option<Self> {
        match value {
            VariableValue::Integer(i) => Some(*i),
            VariableValue::Number(n) => Some(*n as i64),
            _ => None,
        }
    }
}

impl FromVariableValue for f64 {
    fn from_variable_value(value: &VariableValue) -> Option<Self> {
        value.as_number()
    }
}

impl FromVariableValue for bool {
    fn from_variable_value(value: &VariableValue) -> Option<Self> {
        value.as_boolean()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::variables::VariableConstraints;

    #[test]
    fn test_execution_state_terminal() {
        assert!(!ExecutionState::Pending.is_terminal());
        assert!(!ExecutionState::Running.is_terminal());
        assert!(!ExecutionState::Paused.is_terminal());
        assert!(ExecutionState::Completed.is_terminal());
        assert!(ExecutionState::Failed.is_terminal());
        assert!(ExecutionState::Cancelled.is_terminal());
        assert!(ExecutionState::TimedOut.is_terminal());
    }

    #[test]
    fn test_workflow_state_creation() {
        let state = WorkflowState::new("wf-123", vec![]);

        assert_eq!(state.workflow_id, "wf-123");
        assert_eq!(state.version, 1);
        assert_eq!(state.execution_state, ExecutionState::Pending);
        assert!(!state.is_complete());
    }

    #[test]
    fn test_workflow_state_get_set() {
        let mut state = WorkflowState::new("wf-123", vec![]);

        state.set("counter", VariableValue::Integer(42)).unwrap();
        assert_eq!(state.get("counter"), Some(&VariableValue::Integer(42)));
        assert_eq!(state.version, 2);

        // Test get_as
        let counter: Option<i64> = state.get_as("counter");
        assert_eq!(counter, Some(42));
    }

    #[test]
    fn test_workflow_state_with_definitions() {
        let definitions = vec![
            VariableDefinition::new("count", VariableType::Integer)
                .with_constraints(VariableConstraints::new().range(Some(0.0), Some(100.0))),
        ];

        let mut state = WorkflowState::new("wf-123", definitions);

        // Valid value
        assert!(state.set("count", VariableValue::Integer(50)).is_ok());

        // Invalid value (out of range)
        let result = state.set("count", VariableValue::Integer(200));
        assert!(result.is_err());
    }

    #[test]
    fn test_state_transitions() {
        let mut state = WorkflowState::new("wf-123", vec![]);

        // Valid transitions
        assert!(state.transition_to(ExecutionState::Running).is_ok());
        assert_eq!(state.execution_state, ExecutionState::Running);

        assert!(state.transition_to(ExecutionState::Paused).is_ok());
        assert_eq!(state.execution_state, ExecutionState::Paused);

        assert!(state.transition_to(ExecutionState::Running).is_ok());
        assert_eq!(state.execution_state, ExecutionState::Running);

        assert!(state.transition_to(ExecutionState::Completed).is_ok());
        assert_eq!(state.execution_state, ExecutionState::Completed);
        assert!(state.is_complete());
    }

    #[test]
    fn test_invalid_transition() {
        let mut state = WorkflowState::new("wf-123", vec![]);

        // Invalid: Pending -> Completed (must go through Running)
        let result = state.transition_to(ExecutionState::Completed);
        assert!(result.is_err());
    }

    #[test]
    fn test_workflow_fail() {
        let mut state = WorkflowState::new("wf-123", vec![]);
        state.transition_to(ExecutionState::Running).unwrap();

        let error = StateError::new("TEST_ERROR", "Something went wrong");
        state.fail(error);

        assert_eq!(state.execution_state, ExecutionState::Failed);
        assert!(state.error.is_some());
        assert!(state.is_complete());
    }

    #[test]
    fn test_initialize_defaults() {
        let definitions = vec![
            VariableDefinition::new("count", VariableType::Integer),
            VariableDefinition::new("name", VariableType::String),
        ];

        let mut state = WorkflowState::new("wf-123", definitions);
        state.initialize_defaults();

        assert_eq!(state.get("count"), Some(&VariableValue::Integer(0)));
        assert_eq!(state.get("name"), Some(&VariableValue::String(String::new())));
    }

    #[test]
    fn test_input_fallback() {
        let mut state = WorkflowState::new("wf-123", vec![]);

        let mut input = HashMap::new();
        input.insert("userId".to_string(), VariableValue::String("user-1".to_string()));
        state.input = input;

        // get should find input values
        assert_eq!(state.get("userId"), Some(&VariableValue::String("user-1".to_string())));

        // setting a variable should shadow input
        state.set("userId", VariableValue::String("user-2".to_string())).unwrap();
        assert_eq!(state.get("userId"), Some(&VariableValue::String("user-2".to_string())));
    }

    #[test]
    fn test_state_error() {
        let error = StateError::new("ERROR_CODE", "Error message")
            .with_details(serde_json::json!({"key": "value"}))
            .with_stack(vec!["activity-1".to_string(), "activity-2".to_string()]);

        assert_eq!(error.code, "ERROR_CODE");
        assert_eq!(error.message, "Error message");
        assert!(error.details.is_some());
        assert!(error.stack.is_some());
    }

    #[test]
    fn test_serialization() {
        let state = WorkflowState::new("wf-123", vec![])
            .with_metadata("env", "test");

        let json = serde_json::to_string_pretty(&state).unwrap();
        assert!(json.contains("\"workflowId\": \"wf-123\""));

        let parsed: WorkflowState = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.workflow_id, "wf-123");
        assert_eq!(parsed.metadata.get("env"), Some(&"test".to_string()));
    }
}
