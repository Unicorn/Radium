//! Human-friendly YAML workflow types
//!
//! These types define the simplified YAML format that agents write.
//! They are transformed into the compiler's internal WorkflowDefinition
//! by the transformer module.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A workflow definition in the human-friendly YAML format.
///
/// This is what agents write -- a simplified representation that hides
/// React Flow positions, edge types, handles, and other UI concerns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlWorkflow {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub components: Vec<YamlComponent>,
    #[serde(default)]
    pub connections: Vec<YamlConnection>,
    #[serde(default)]
    pub variables: Vec<YamlVariable>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<YamlSettings>,
}

/// A single component in the YAML workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlComponent {
    pub id: String,
    #[serde(rename = "type")]
    pub component_type: YamlComponentType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<HashMap<String, serde_json::Value>>,
}

/// Component types available in the YAML format.
///
/// Maps to the compiler's `NodeType` via the transformer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum YamlComponentType {
    Trigger,
    Stop,
    #[serde(alias = "activity")]
    Action,
    HttpRequest,
    DatabaseQuery,
    Agent,
    Conditional,
    Loop,
    Parallel,
    #[serde(alias = "signal")]
    Message,
    Timer,
    #[serde(alias = "child_workflow")]
    ChildService,
    Log,
    ShellExecute,
    NpmFunction,
    CodeExecute,
    DataTransform,
}

/// A connection between two components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlConnection {
    pub from: String,
    pub to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// A workflow variable declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlVariable {
    pub name: String,
    #[serde(rename = "type")]
    pub var_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Workflow-level settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_queue: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_minimal_yaml() {
        let yaml = r#"
name: Simple Workflow
components:
  - id: start
    type: trigger
  - id: end
    type: stop
connections:
  - from: start
    to: end
"#;
        let workflow: YamlWorkflow = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(workflow.name, "Simple Workflow");
        assert_eq!(workflow.components.len(), 2);
        assert_eq!(workflow.connections.len(), 1);
        assert!(workflow.description.is_none());
        assert!(workflow.variables.is_empty());
        assert!(workflow.settings.is_none());
    }

    #[test]
    fn test_deserialize_full_yaml() {
        let yaml = r#"
name: Full Workflow
description: A comprehensive workflow
components:
  - id: start
    type: trigger
  - id: fetch_data
    type: http_request
    config:
      name: fetchData
      url: "https://api.example.com/data"
      method: GET
  - id: process
    type: action
    config:
      name: processData
      timeout: "30s"
  - id: end
    type: stop
connections:
  - from: start
    to: fetch_data
  - from: fetch_data
    to: process
    label: success
  - from: process
    to: end
variables:
  - name: counter
    type: number
    default: 0
    description: A counter variable
  - name: result
    type: string
settings:
  timeout: "5m"
  task_queue: my-queue
  description: Production workflow
"#;
        let workflow: YamlWorkflow = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(workflow.name, "Full Workflow");
        assert_eq!(workflow.description, Some("A comprehensive workflow".to_string()));
        assert_eq!(workflow.components.len(), 4);
        assert_eq!(workflow.connections.len(), 3);
        assert_eq!(workflow.variables.len(), 2);
        assert!(workflow.settings.is_some());

        let settings = workflow.settings.unwrap();
        assert_eq!(settings.timeout, Some("5m".to_string()));
        assert_eq!(settings.task_queue, Some("my-queue".to_string()));

        // Verify config parsing
        let fetch = &workflow.components[1];
        let config = fetch.config.as_ref().unwrap();
        assert_eq!(config.get("name").unwrap(), "fetchData");
        assert_eq!(config.get("method").unwrap(), "GET");

        // Verify variable parsing
        assert_eq!(workflow.variables[0].name, "counter");
        assert_eq!(workflow.variables[0].var_type, "number");
        assert_eq!(workflow.variables[0].default, Some(serde_json::json!(0)));
    }

    #[test]
    fn test_component_type_deserialization() {
        let cases = vec![
            ("trigger", YamlComponentType::Trigger),
            ("stop", YamlComponentType::Stop),
            ("action", YamlComponentType::Action),
            ("http_request", YamlComponentType::HttpRequest),
            ("database_query", YamlComponentType::DatabaseQuery),
            ("agent", YamlComponentType::Agent),
            ("conditional", YamlComponentType::Conditional),
            ("loop", YamlComponentType::Loop),
            ("parallel", YamlComponentType::Parallel),
            ("message", YamlComponentType::Message),
            ("timer", YamlComponentType::Timer),
            ("child_service", YamlComponentType::ChildService),
            ("log", YamlComponentType::Log),
            ("shell_execute", YamlComponentType::ShellExecute),
            ("npm_function", YamlComponentType::NpmFunction),
            ("code_execute", YamlComponentType::CodeExecute),
            ("data_transform", YamlComponentType::DataTransform),
        ];

        for (yaml_str, expected) in cases {
            let yaml = format!("\"{}\"", yaml_str);
            let parsed: YamlComponentType = serde_yaml::from_str(&yaml).unwrap();
            assert_eq!(parsed, expected, "Failed to parse component type: {}", yaml_str);
        }
    }

    #[test]
    fn test_activity_alias_deserializes_to_action() {
        // "activity" is the deprecated name; it must still deserialize to Action
        let parsed: YamlComponentType = serde_yaml::from_str("\"activity\"").unwrap();
        assert_eq!(parsed, YamlComponentType::Action);

        // "action" is the canonical new name
        let parsed: YamlComponentType = serde_yaml::from_str("\"action\"").unwrap();
        assert_eq!(parsed, YamlComponentType::Action);
    }

    #[test]
    fn test_child_workflow_alias_deserializes_to_child_service() {
        // "child_workflow" is the deprecated name; it must still deserialize to ChildService
        let parsed: YamlComponentType = serde_yaml::from_str("\"child_workflow\"").unwrap();
        assert_eq!(parsed, YamlComponentType::ChildService);

        // "child_service" is the canonical new name
        let parsed: YamlComponentType = serde_yaml::from_str("\"child_service\"").unwrap();
        assert_eq!(parsed, YamlComponentType::ChildService);
    }

    #[test]
    fn test_signal_alias_deserializes_to_message() {
        // "signal" is the deprecated name; it must still deserialize to Message
        let parsed: YamlComponentType = serde_yaml::from_str("\"signal\"").unwrap();
        assert_eq!(parsed, YamlComponentType::Message);

        // "message" is the canonical new name
        let parsed: YamlComponentType = serde_yaml::from_str("\"message\"").unwrap();
        assert_eq!(parsed, YamlComponentType::Message);
    }
}
