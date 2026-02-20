//! Transforms YAML workflow definitions into the compiler's internal WorkflowDefinition.
//!
//! This module bridges the gap between the human-friendly YAML format and the
//! compiler's React Flow-style representation with positions, edge IDs, handles, etc.

use std::collections::{HashMap, HashSet};
use thiserror::Error;

use crate::schema::{
    LegacyVariableType as VariableType, NodeData, NodeType, Position, WorkflowDefinition,
    WorkflowEdge, WorkflowNode, WorkflowSettings, WorkflowVariable,
};

use super::types::{YamlComponent, YamlComponentType, YamlWorkflow};

/// Errors that can occur during YAML-to-WorkflowDefinition transformation.
#[derive(Debug, Error)]
pub enum TransformError {
    #[error("Duplicate component ID: '{0}'")]
    DuplicateComponentId(String),

    #[error("Connection references undefined component: '{0}'")]
    UndefinedComponent(String),

    #[error("Unknown variable type: '{0}'. Valid types: string, number, boolean, array, object")]
    UnknownVariableType(String),
}

/// Transform a YAML workflow into the compiler's internal WorkflowDefinition.
///
/// This function:
/// 1. Validates component IDs are unique
/// 2. Validates all connections reference existing components
/// 3. Maps YAML component types to internal NodeTypes
/// 4. Extracts well-known config fields into typed NodeData fields
/// 5. Auto-generates positions (vertical layout)
/// 6. Auto-generates edge IDs
/// 7. Transforms variables with type parsing
/// 8. Generates a UUID for the workflow ID
pub fn transform(yaml: &YamlWorkflow) -> Result<WorkflowDefinition, TransformError> {
    // 1. Check for duplicate component IDs
    let mut seen_ids = HashSet::new();
    for component in &yaml.components {
        if !seen_ids.insert(&component.id) {
            return Err(TransformError::DuplicateComponentId(component.id.clone()));
        }
    }

    // 2. Check connections reference valid component IDs
    for conn in &yaml.connections {
        if !seen_ids.contains(&conn.from) {
            return Err(TransformError::UndefinedComponent(conn.from.clone()));
        }
        if !seen_ids.contains(&conn.to) {
            return Err(TransformError::UndefinedComponent(conn.to.clone()));
        }
    }

    // 3-5. Transform components into nodes
    let nodes: Vec<WorkflowNode> = yaml
        .components
        .iter()
        .enumerate()
        .map(|(index, component)| transform_component(component, index))
        .collect();

    // 6. Transform connections into edges with auto-generated IDs
    let edges: Vec<WorkflowEdge> = yaml
        .connections
        .iter()
        .enumerate()
        .map(|(index, conn)| {
            let mut edge = WorkflowEdge::new(
                format!("edge_{}", index),
                &conn.from,
                &conn.to,
            );
            edge.label = conn.label.clone();
            edge
        })
        .collect();

    // 7. Transform variables
    let variables: Vec<WorkflowVariable> = yaml
        .variables
        .iter()
        .map(|v| transform_variable(v))
        .collect::<Result<Vec<_>, _>>()?;

    // 8. Build settings
    let settings = match &yaml.settings {
        Some(s) => WorkflowSettings {
            timeout: s.timeout.clone(),
            description: s.description.clone(),
            task_queue: s.task_queue.clone(),
            ..Default::default()
        },
        None => WorkflowSettings::default(),
    };

    // Generate workflow ID and build the definition
    let id = uuid::Uuid::new_v4().to_string();

    Ok(WorkflowDefinition {
        id,
        name: yaml.name.clone(),
        nodes,
        edges,
        variables,
        settings,
    })
}

/// Map a YAML component type to the compiler's NodeType.
fn map_component_type(ct: &YamlComponentType) -> NodeType {
    match ct {
        YamlComponentType::Trigger => NodeType::Trigger,
        YamlComponentType::Stop => NodeType::End,
        YamlComponentType::Activity => NodeType::Activity,
        YamlComponentType::HttpRequest => NodeType::Activity,
        YamlComponentType::DatabaseQuery => NodeType::Activity,
        YamlComponentType::Agent => NodeType::Agent,
        YamlComponentType::Conditional => NodeType::Conditional,
        YamlComponentType::Loop => NodeType::Loop,
        YamlComponentType::Parallel => NodeType::Activity,
        YamlComponentType::Signal => NodeType::Signal,
        YamlComponentType::Timer => NodeType::Activity,
        YamlComponentType::ChildWorkflow => NodeType::ChildWorkflow,
        YamlComponentType::Log => NodeType::Activity,
    }
}

/// Transform a single YAML component into a WorkflowNode.
///
/// Extracts well-known config fields (name, signal_name, timeout, description)
/// into the typed NodeData fields, and leaves the rest in the config map.
fn transform_component(component: &YamlComponent, index: usize) -> WorkflowNode {
    let node_type = map_component_type(&component.component_type);

    // Start with a default label based on the component ID
    let mut data = NodeData {
        label: component.id.clone(),
        ..Default::default()
    };

    // Extract well-known fields from config
    if let Some(config) = &component.config {
        let mut remaining_config: HashMap<String, serde_json::Value> = HashMap::new();

        for (key, value) in config {
            match key.as_str() {
                "name" => {
                    if let Some(name) = value.as_str() {
                        // For activities/agents, set activity_name
                        // For all nodes, update the label
                        data.activity_name = Some(name.to_string());
                        data.label = name.to_string();
                    }
                }
                "signal_name" => {
                    if let Some(signal_name) = value.as_str() {
                        data.signal_name = Some(signal_name.to_string());
                    }
                }
                "timeout" => {
                    if let Some(timeout) = value.as_str() {
                        data.timeout = Some(timeout.to_string());
                    }
                }
                "description" => {
                    if let Some(desc) = value.as_str() {
                        data.description = Some(desc.to_string());
                    }
                }
                _ => {
                    remaining_config.insert(key.clone(), value.clone());
                }
            }
        }

        if !remaining_config.is_empty() {
            data.config = Some(remaining_config);
        }
    }

    // For trigger and end nodes, set sensible default labels
    match component.component_type {
        YamlComponentType::Trigger => {
            if data.activity_name.is_none() {
                data.label = "Start".to_string();
            }
        }
        YamlComponentType::Stop => {
            if data.activity_name.is_none() {
                data.label = "End".to_string();
            }
        }
        _ => {}
    }

    WorkflowNode {
        id: component.id.clone(),
        node_type,
        data,
        position: Position {
            x: 0.0,
            y: index as f64 * 150.0,
        },
    }
}

/// Parse a variable type string into the compiler's VariableType enum.
fn parse_variable_type(type_str: &str) -> Result<VariableType, TransformError> {
    match type_str {
        "string" => Ok(VariableType::String),
        "number" => Ok(VariableType::Number),
        "boolean" => Ok(VariableType::Boolean),
        "array" => Ok(VariableType::Array),
        "object" => Ok(VariableType::Object),
        other => Err(TransformError::UnknownVariableType(other.to_string())),
    }
}

/// Transform a YAML variable into a WorkflowVariable.
fn transform_variable(v: &super::types::YamlVariable) -> Result<WorkflowVariable, TransformError> {
    let var_type = parse_variable_type(&v.var_type)?;

    Ok(WorkflowVariable {
        name: v.name.clone(),
        var_type,
        initial_value: v.default.clone(),
        description: v.description.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen;
    use crate::validation;

    fn minimal_yaml() -> YamlWorkflow {
        serde_yaml::from_str(
            r#"
name: Minimal Workflow
components:
  - id: start
    type: trigger
  - id: end
    type: stop
connections:
  - from: start
    to: end
"#,
        )
        .unwrap()
    }

    fn full_yaml() -> YamlWorkflow {
        serde_yaml::from_str(
            r#"
name: Full Workflow
description: A comprehensive test workflow
components:
  - id: start
    type: trigger
  - id: fetch
    type: http_request
    config:
      name: fetchData
      url: "https://api.example.com"
      method: GET
      timeout: "10s"
  - id: process
    type: activity
    config:
      name: processData
      description: Process the fetched data
  - id: check
    type: conditional
    config:
      name: checkResult
  - id: notify
    type: signal
    config:
      signal_name: data_ready
      name: notifyReady
  - id: end
    type: stop
connections:
  - from: start
    to: fetch
  - from: fetch
    to: process
    label: success
  - from: process
    to: check
  - from: check
    to: notify
    label: "true"
  - from: check
    to: end
    label: "false"
  - from: notify
    to: end
variables:
  - name: counter
    type: number
    default: 0
  - name: result
    type: string
    description: The final result
settings:
  timeout: "5m"
  task_queue: main-queue
"#,
        )
        .unwrap()
    }

    #[test]
    fn test_transform_minimal() {
        let yaml = minimal_yaml();
        let def = transform(&yaml).unwrap();

        assert_eq!(def.name, "Minimal Workflow");
        assert_eq!(def.nodes.len(), 2);
        assert_eq!(def.edges.len(), 1);
        assert!(!def.id.is_empty());

        // Check node types
        let trigger = def.find_node("start").unwrap();
        assert_eq!(trigger.node_type, NodeType::Trigger);
        assert_eq!(trigger.data.label, "Start");

        let end = def.find_node("end").unwrap();
        assert_eq!(end.node_type, NodeType::End);
        assert_eq!(end.data.label, "End");

        // Check positions are auto-generated
        assert_eq!(trigger.position.y, 0.0);
        assert_eq!(end.position.y, 150.0);
    }

    #[test]
    fn test_transform_extracts_activity_name() {
        let yaml: YamlWorkflow = serde_yaml::from_str(
            r#"
name: Activity Test
components:
  - id: start
    type: trigger
  - id: my_activity
    type: activity
    config:
      name: doSomething
      extra_field: value
  - id: end
    type: stop
connections:
  - from: start
    to: my_activity
  - from: my_activity
    to: end
"#,
        )
        .unwrap();

        let def = transform(&yaml).unwrap();
        let activity = def.find_node("my_activity").unwrap();

        assert_eq!(activity.data.activity_name, Some("doSomething".to_string()));
        assert_eq!(activity.data.label, "doSomething");

        // Extra fields should remain in config
        let config = activity.data.config.as_ref().unwrap();
        assert!(config.contains_key("extra_field"));
        assert!(!config.contains_key("name")); // extracted
    }

    #[test]
    fn test_transform_generates_edge_ids() {
        let yaml = full_yaml();
        let def = transform(&yaml).unwrap();

        let edge_ids: Vec<&str> = def.edges.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(edge_ids[0], "edge_0");
        assert_eq!(edge_ids[1], "edge_1");
        assert_eq!(edge_ids[2], "edge_2");
        assert_eq!(edge_ids[3], "edge_3");
        assert_eq!(edge_ids[4], "edge_4");
        assert_eq!(edge_ids[5], "edge_5");
    }

    #[test]
    fn test_transform_connection_labels() {
        let yaml = full_yaml();
        let def = transform(&yaml).unwrap();

        // First edge has no label
        assert!(def.edges[0].label.is_none());
        // Second edge has "success" label
        assert_eq!(def.edges[1].label, Some("success".to_string()));
        // Fourth edge has "true" label
        assert_eq!(def.edges[3].label, Some("true".to_string()));
        // Fifth edge has "false" label
        assert_eq!(def.edges[4].label, Some("false".to_string()));
    }

    #[test]
    fn test_transform_variables() {
        let yaml = full_yaml();
        let def = transform(&yaml).unwrap();

        assert_eq!(def.variables.len(), 2);

        assert_eq!(def.variables[0].name, "counter");
        assert_eq!(def.variables[0].var_type, VariableType::Number);
        assert_eq!(
            def.variables[0].initial_value,
            Some(serde_json::json!(0))
        );

        assert_eq!(def.variables[1].name, "result");
        assert_eq!(def.variables[1].var_type, VariableType::String);
        assert_eq!(
            def.variables[1].description,
            Some("The final result".to_string())
        );
    }

    #[test]
    fn test_transform_rejects_duplicate_ids() {
        let yaml: YamlWorkflow = serde_yaml::from_str(
            r#"
name: Duplicate Test
components:
  - id: start
    type: trigger
  - id: start
    type: stop
connections: []
"#,
        )
        .unwrap();

        let result = transform(&yaml);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, TransformError::DuplicateComponentId(ref id) if id == "start"),
            "Expected DuplicateComponentId, got: {:?}",
            err
        );
    }

    #[test]
    fn test_transform_rejects_undefined_connection() {
        let yaml: YamlWorkflow = serde_yaml::from_str(
            r#"
name: Undefined Connection Test
components:
  - id: start
    type: trigger
  - id: end
    type: stop
connections:
  - from: start
    to: nonexistent
"#,
        )
        .unwrap();

        let result = transform(&yaml);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, TransformError::UndefinedComponent(ref id) if id == "nonexistent"),
            "Expected UndefinedComponent, got: {:?}",
            err
        );
    }

    #[test]
    fn test_transform_unknown_variable_type() {
        let yaml: YamlWorkflow = serde_yaml::from_str(
            r#"
name: Unknown Var Type
components:
  - id: start
    type: trigger
  - id: end
    type: stop
connections:
  - from: start
    to: end
variables:
  - name: bad_var
    type: timestamp
"#,
        )
        .unwrap();

        let result = transform(&yaml);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, TransformError::UnknownVariableType(ref t) if t == "timestamp"),
            "Expected UnknownVariableType, got: {:?}",
            err
        );
    }

    #[test]
    fn test_transform_then_validate() {
        let yaml = minimal_yaml();
        let def = transform(&yaml).unwrap();

        let validation_result = validation::validate(&def);
        assert!(
            validation_result.is_valid(),
            "Transformed workflow should pass validation, but got errors: {:?}",
            validation_result.errors
        );
    }

    #[test]
    fn test_transform_then_compile() {
        let yaml = minimal_yaml();
        let def = transform(&yaml).unwrap();

        let gen_result = codegen::generate(&def);
        assert!(
            gen_result.is_ok(),
            "Transformed workflow should compile, but got error: {:?}",
            gen_result.err()
        );

        let code = gen_result.unwrap();
        assert!(!code.workflow.is_empty());
        assert!(!code.activities.is_empty());
    }
}
