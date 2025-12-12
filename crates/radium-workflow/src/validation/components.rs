//! Component configuration validation

use crate::schema::{NodeType, WorkflowDefinition};

use super::errors::{ValidationError, ValidationResult, ValidationWarning};

/// Validate component configurations
pub fn validate_components(workflow: &WorkflowDefinition) -> ValidationResult {
    let mut result = ValidationResult::new();

    for node in &workflow.nodes {
        match node.node_type {
            NodeType::Activity | NodeType::Agent => {
                // Activities must have an activity_name or component_name
                if node.activity_name().is_none() {
                    result.add_error(ValidationError::MissingActivityName {
                        node_id: node.id.clone(),
                    });
                }

                // Warn if no timeout
                if node.data.timeout.is_none() {
                    result.add_warning(ValidationWarning::NoTimeout {
                        node_id: node.id.clone(),
                    });
                }

                // Warn if no retry policy
                if node.data.retry_policy.is_none() {
                    result.add_warning(ValidationWarning::NoRetryPolicy {
                        node_id: node.id.clone(),
                    });
                }
            },
            NodeType::Signal => {
                // Signal nodes must have a signal_name
                if node.data.signal_name.is_none() {
                    result.add_error(ValidationError::InvalidConfig {
                        node_id: node.id.clone(),
                        message: "Signal node must have a signal_name".to_string(),
                    });
                }
            },
            NodeType::ChildWorkflow => {
                // Child workflow nodes need configuration
                if node.data.config.is_none() {
                    result.add_warning(ValidationWarning::NoTimeout {
                        node_id: node.id.clone(),
                    });
                }
            },
            NodeType::StateVariable => {
                // State variable nodes need a name in the label or config
                if node.data.label.is_empty() && node.data.config.is_none() {
                    result.add_error(ValidationError::InvalidConfig {
                        node_id: node.id.clone(),
                        message: "State variable node must have a label or config".to_string(),
                    });
                }
            },
            NodeType::KongLogging | NodeType::KongCache | NodeType::KongCors => {
                // Kong components typically need connector_id in config
                // This is a soft warning, not an error
                if node.data.config.is_none() {
                    result.add_warning(ValidationWarning::NoTimeout {
                        node_id: node.id.clone(),
                    });
                }
            },
            // Other node types don't have specific validation
            _ => {},
        }
    }

    // Check for unused variables
    let used_vars = find_used_variables(workflow);
    for var in &workflow.variables {
        if !used_vars.contains(&var.name) {
            result.add_warning(ValidationWarning::UnusedVariable {
                var_name: var.name.clone(),
            });
        }
    }

    result
}

/// Find all variable names used in the workflow
fn find_used_variables(workflow: &WorkflowDefinition) -> std::collections::HashSet<String> {
    let mut used = std::collections::HashSet::new();

    for node in &workflow.nodes {
        // Check config for variable references
        if let Some(config) = &node.data.config {
            for value in config.values() {
                extract_variable_references(value, &mut used);
            }
        }

        // Check input mappings
        if let Some(input) = &node.data.input {
            for value in input.values() {
                extract_variable_references(value, &mut used);
            }
        }
    }

    used
}

/// Extract variable references from a JSON value
fn extract_variable_references(
    value: &serde_json::Value,
    vars: &mut std::collections::HashSet<String>,
) {
    match value {
        serde_json::Value::String(s) => {
            // Check for variable reference patterns like {{varName}} or ${varName}
            let re_curly = regex::Regex::new(r"\{\{(\w+)\}\}").unwrap();
            let re_dollar = regex::Regex::new(r"\$\{(\w+)\}").unwrap();

            for cap in re_curly.captures_iter(s) {
                if let Some(m) = cap.get(1) {
                    vars.insert(m.as_str().to_string());
                }
            }
            for cap in re_dollar.captures_iter(s) {
                if let Some(m) = cap.get(1) {
                    vars.insert(m.as_str().to_string());
                }
            }
        },
        serde_json::Value::Object(map) => {
            for v in map.values() {
                extract_variable_references(v, vars);
            }
        },
        serde_json::Value::Array(arr) => {
            for v in arr {
                extract_variable_references(v, vars);
            }
        },
        _ => {},
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{
        NodeData, NodeType, Position, VariableType, WorkflowEdge, WorkflowNode, WorkflowSettings,
        WorkflowVariable,
    };

    #[test]
    fn test_missing_activity_name() {
        let workflow = WorkflowDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
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
                    id: "activity".to_string(),
                    node_type: NodeType::Activity,
                    data: NodeData {
                        label: "Activity".to_string(),
                        // Missing activity_name!
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
            edges: vec![
                WorkflowEdge::new("e1", "trigger", "activity"),
                WorkflowEdge::new("e2", "activity", "end"),
            ],
            variables: vec![],
            settings: WorkflowSettings::default(),
        };

        let result = validate_components(&workflow);
        assert!(result.errors.iter().any(|e| matches!(e, ValidationError::MissingActivityName { node_id } if node_id == "activity")));
    }

    #[test]
    fn test_unused_variable_warning() {
        let workflow = WorkflowDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            nodes: vec![
                WorkflowNode {
                    id: "trigger".to_string(),
                    node_type: NodeType::Trigger,
                    data: NodeData::default(),
                    position: Position::default(),
                },
                WorkflowNode {
                    id: "end".to_string(),
                    node_type: NodeType::End,
                    data: NodeData::default(),
                    position: Position::default(),
                },
            ],
            edges: vec![WorkflowEdge::new("e1", "trigger", "end")],
            variables: vec![WorkflowVariable::new("unusedVar", VariableType::String)],
            settings: WorkflowSettings::default(),
        };

        let result = validate_components(&workflow);
        assert!(result.warnings.iter().any(|w| matches!(w, ValidationWarning::UnusedVariable { var_name } if var_name == "unusedVar")));
    }

    #[test]
    fn test_valid_activity() {
        let workflow = WorkflowDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            nodes: vec![
                WorkflowNode {
                    id: "trigger".to_string(),
                    node_type: NodeType::Trigger,
                    data: NodeData::default(),
                    position: Position::default(),
                },
                WorkflowNode {
                    id: "activity".to_string(),
                    node_type: NodeType::Activity,
                    data: NodeData {
                        label: "Activity".to_string(),
                        activity_name: Some("doSomething".to_string()),
                        timeout: Some("5m".to_string()),
                        ..Default::default()
                    },
                    position: Position::default(),
                },
                WorkflowNode {
                    id: "end".to_string(),
                    node_type: NodeType::End,
                    data: NodeData::default(),
                    position: Position::default(),
                },
            ],
            edges: vec![
                WorkflowEdge::new("e1", "trigger", "activity"),
                WorkflowEdge::new("e2", "activity", "end"),
            ],
            variables: vec![],
            settings: WorkflowSettings::default(),
        };

        let result = validate_components(&workflow);
        // Should not have the MissingActivityName error
        assert!(!result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::MissingActivityName { .. })));
    }
}
