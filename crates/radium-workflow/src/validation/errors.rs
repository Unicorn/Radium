//! Validation error types

use thiserror::Error;

/// Validation errors
#[derive(Debug, Error, Clone)]
pub enum ValidationError {
    #[error("No start node found. Workflow must have exactly one trigger node.")]
    NoStartNode,

    #[error("Multiple start nodes found: {0:?}. Workflow must have exactly one trigger.")]
    MultipleStartNodes(Vec<String>),

    #[error("No end node found. Workflow must have at least one end node.")]
    NoEndNode,

    #[error("Orphan node '{0}' has no connections.")]
    OrphanNode(String),

    #[error("Cycle detected involving nodes: {0:?}")]
    CycleDetected(Vec<String>),

    #[error("Invalid edge: source node '{0}' not found.")]
    InvalidEdgeSource(String),

    #[error("Invalid edge: target node '{0}' not found.")]
    InvalidEdgeTarget(String),

    #[error("Node '{node_id}' references unknown variable '{var_name}'.")]
    UnknownVariable { node_id: String, var_name: String },

    #[error("Activity node '{node_id}' is missing required component_name or activity_name.")]
    MissingActivityName { node_id: String },

    #[error("Node '{node_id}' has invalid configuration: {message}")]
    InvalidConfig { node_id: String, message: String },

    #[error("Workflow is missing required field: {0}")]
    MissingRequiredField(String),

    #[error("Start node '{node_id}' has incoming edges, which is not allowed.")]
    TriggerHasIncomingEdges { node_id: String },

    #[error("Node '{node_id}' is unreachable from the start node.")]
    UnreachableNode { node_id: String },
}

/// Validation warnings (non-fatal issues)
#[derive(Debug, Clone)]
pub enum ValidationWarning {
    /// Activity has no retry policy
    NoRetryPolicy { node_id: String },
    /// Activity has no timeout set
    NoTimeout { node_id: String },
    /// Variable declared but never used
    UnusedVariable { var_name: String },
    /// Deprecated node type
    DeprecatedNodeType { node_id: String, node_type: String },
}

impl std::fmt::Display for ValidationWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationWarning::NoRetryPolicy { node_id } => {
                write!(f, "Activity '{}' has no retry policy configured.", node_id)
            },
            ValidationWarning::NoTimeout { node_id } => {
                write!(f, "Activity '{}' has no timeout configured.", node_id)
            },
            ValidationWarning::UnusedVariable { var_name } => {
                write!(f, "Variable '{}' is declared but never used.", var_name)
            },
            ValidationWarning::DeprecatedNodeType { node_id, node_type } => {
                write!(
                    f,
                    "Node '{}' uses deprecated type '{}'.",
                    node_id, node_type
                )
            },
        }
    }
}

/// Validation result containing errors and warnings
#[derive(Debug, Default)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationResult {
    /// Create a new empty validation result
    pub fn new() -> Self {
        ValidationResult::default()
    }

    /// Check if the validation passed (no errors)
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Add an error
    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    /// Add a warning
    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }

    /// Merge another validation result into this one
    pub fn merge(&mut self, other: ValidationResult) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }

    /// Create a result with a single error
    pub fn with_error(error: ValidationError) -> Self {
        ValidationResult {
            errors: vec![error],
            warnings: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result_is_valid() {
        let result = ValidationResult::new();
        assert!(result.is_valid());

        let result = ValidationResult::with_error(ValidationError::NoStartNode);
        assert!(!result.is_valid());
    }

    #[test]
    fn test_merge_results() {
        let mut result1 = ValidationResult::new();
        result1.add_error(ValidationError::NoStartNode);

        let mut result2 = ValidationResult::new();
        result2.add_error(ValidationError::NoEndNode);
        result2.add_warning(ValidationWarning::NoRetryPolicy {
            node_id: "node1".to_string(),
        });

        result1.merge(result2);
        assert_eq!(result1.errors.len(), 2);
        assert_eq!(result1.warnings.len(), 1);
    }

    #[test]
    fn test_error_display() {
        let error = ValidationError::MissingActivityName {
            node_id: "activity_1".to_string(),
        };
        let message = format!("{}", error);
        assert!(message.contains("activity_1"));
        assert!(message.contains("missing required"));
    }
}
