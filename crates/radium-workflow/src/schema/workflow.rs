//! Main workflow definition

use serde::{Deserialize, Serialize};

use super::{WorkflowEdge, WorkflowNode, WorkflowSettings, WorkflowVariable};

/// Complete workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub id: String,
    pub name: String,
    pub nodes: Vec<WorkflowNode>,
    pub edges: Vec<WorkflowEdge>,
    #[serde(default)]
    pub variables: Vec<WorkflowVariable>,
    #[serde(default)]
    pub settings: WorkflowSettings,
}

impl WorkflowDefinition {
    /// Create a new empty workflow
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        WorkflowDefinition {
            id: id.into(),
            name: name.into(),
            nodes: vec![],
            edges: vec![],
            variables: vec![],
            settings: WorkflowSettings::default(),
        }
    }

    /// Find a node by ID
    pub fn find_node(&self, id: &str) -> Option<&WorkflowNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Find all edges with a given source node
    pub fn edges_from(&self, node_id: &str) -> Vec<&WorkflowEdge> {
        self.edges.iter().filter(|e| e.source == node_id).collect()
    }

    /// Find all edges with a given target node
    pub fn edges_to(&self, node_id: &str) -> Vec<&WorkflowEdge> {
        self.edges.iter().filter(|e| e.target == node_id).collect()
    }

    /// Find the trigger (start) node
    pub fn find_trigger(&self) -> Option<&WorkflowNode> {
        self.nodes.iter().find(|n| n.is_trigger())
    }

    /// Find all end nodes
    pub fn find_end_nodes(&self) -> Vec<&WorkflowNode> {
        self.nodes.iter().filter(|n| n.is_end()).collect()
    }

    /// Check if this workflow has variables
    pub fn has_variables(&self) -> bool {
        !self.variables.is_empty()
    }

    /// Get a safe function name for code generation
    pub fn function_name(&self) -> String {
        // Convert to PascalCase and ensure valid identifier
        let name = self
            .name
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '_')
            .collect::<String>();

        let parts: Vec<&str> = name.split(|c| c == ' ' || c == '_').collect();
        parts
            .iter()
            .filter(|s| !s.is_empty())
            .map(|s| {
                let mut chars = s.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{NodeData, NodeType, Position};

    fn create_test_workflow() -> WorkflowDefinition {
        WorkflowDefinition {
            id: "wf_test".to_string(),
            name: "Test Workflow".to_string(),
            nodes: vec![
                WorkflowNode {
                    id: "trigger_1".to_string(),
                    node_type: NodeType::Trigger,
                    data: NodeData {
                        label: "Start".to_string(),
                        ..Default::default()
                    },
                    position: Position::default(),
                },
                WorkflowNode {
                    id: "activity_1".to_string(),
                    node_type: NodeType::Activity,
                    data: NodeData {
                        label: "Do Something".to_string(),
                        activity_name: Some("doSomething".to_string()),
                        ..Default::default()
                    },
                    position: Position::default(),
                },
                WorkflowNode {
                    id: "end_1".to_string(),
                    node_type: NodeType::End,
                    data: NodeData {
                        label: "End".to_string(),
                        ..Default::default()
                    },
                    position: Position::default(),
                },
            ],
            edges: vec![
                WorkflowEdge::new("edge_1", "trigger_1", "activity_1"),
                WorkflowEdge::new("edge_2", "activity_1", "end_1"),
            ],
            variables: vec![],
            settings: WorkflowSettings::default(),
        }
    }

    #[test]
    fn test_find_trigger() {
        let workflow = create_test_workflow();
        let trigger = workflow.find_trigger();
        assert!(trigger.is_some());
        assert_eq!(trigger.unwrap().id, "trigger_1");
    }

    #[test]
    fn test_find_end_nodes() {
        let workflow = create_test_workflow();
        let end_nodes = workflow.find_end_nodes();
        assert_eq!(end_nodes.len(), 1);
        assert_eq!(end_nodes[0].id, "end_1");
    }

    #[test]
    fn test_edges_from() {
        let workflow = create_test_workflow();
        let edges = workflow.edges_from("trigger_1");
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].target, "activity_1");
    }

    #[test]
    fn test_function_name() {
        let workflow = WorkflowDefinition::new("id", "My Test Workflow");
        assert_eq!(workflow.function_name(), "MyTestWorkflow");

        let workflow = WorkflowDefinition::new("id", "simple_workflow");
        assert_eq!(workflow.function_name(), "SimpleWorkflow");
    }

    #[test]
    fn test_full_workflow_deserialization() {
        let json = r#"{
            "id": "wf_123",
            "name": "Example Workflow",
            "nodes": [
                {
                    "id": "trigger",
                    "type": "trigger",
                    "data": { "label": "Start" },
                    "position": { "x": 0, "y": 0 }
                },
                {
                    "id": "end",
                    "type": "end",
                    "data": { "label": "End" },
                    "position": { "x": 200, "y": 0 }
                }
            ],
            "edges": [
                { "id": "e1", "source": "trigger", "target": "end" }
            ],
            "variables": [],
            "settings": {
                "timeout": "5m"
            }
        }"#;

        let workflow: WorkflowDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(workflow.id, "wf_123");
        assert_eq!(workflow.nodes.len(), 2);
        assert_eq!(workflow.edges.len(), 1);
        assert!(workflow.find_trigger().is_some());
    }
}
