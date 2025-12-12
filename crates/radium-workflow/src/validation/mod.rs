//! Workflow validation module
//!
//! Provides comprehensive validation of workflow definitions including:
//! - Graph structure validation (connectivity, cycles)
//! - Component configuration validation
//! - Variable reference validation

mod components;
mod errors;
mod graph;

pub use errors::{ValidationError, ValidationResult, ValidationWarning};
pub use graph::validate_graph;

use crate::schema::WorkflowDefinition;

/// Validate a workflow definition
///
/// Returns a ValidationResult containing any errors and warnings found.
pub fn validate(workflow: &WorkflowDefinition) -> ValidationResult {
    let mut result = ValidationResult::new();

    // Graph structure validation
    let graph_result = graph::validate_graph(workflow);
    result.merge(graph_result);

    // Component configuration validation
    let component_result = components::validate_components(workflow);
    result.merge(component_result);

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{
        NodeData, NodeType, Position, WorkflowEdge, WorkflowNode, WorkflowSettings,
    };

    fn create_valid_workflow() -> WorkflowDefinition {
        WorkflowDefinition {
            id: "wf_test".to_string(),
            name: "Test Workflow".to_string(),
            nodes: vec![
                WorkflowNode {
                    id: "trigger".to_string(),
                    node_type: NodeType::Trigger,
                    data: NodeData {
                        label: "Start".to_string(),
                        ..Default::default()
                    },
                    position: Position::default(),
                },
                WorkflowNode {
                    id: "end".to_string(),
                    node_type: NodeType::End,
                    data: NodeData {
                        label: "End".to_string(),
                        ..Default::default()
                    },
                    position: Position::default(),
                },
            ],
            edges: vec![WorkflowEdge::new("e1", "trigger", "end")],
            variables: vec![],
            settings: WorkflowSettings::default(),
        }
    }

    #[test]
    fn test_valid_workflow() {
        let workflow = create_valid_workflow();
        let result = validate(&workflow);
        assert!(
            result.is_valid(),
            "Expected valid workflow, got errors: {:?}",
            result.errors
        );
    }

    #[test]
    fn test_missing_trigger() {
        let workflow = WorkflowDefinition {
            id: "wf_test".to_string(),
            name: "Test".to_string(),
            nodes: vec![WorkflowNode {
                id: "end".to_string(),
                node_type: NodeType::End,
                data: NodeData {
                    label: "End".to_string(),
                    ..Default::default()
                },
                position: Position::default(),
            }],
            edges: vec![],
            variables: vec![],
            settings: WorkflowSettings::default(),
        };

        let result = validate(&workflow);
        assert!(!result.is_valid());
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::NoStartNode)));
    }
}
