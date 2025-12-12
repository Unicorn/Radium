//! Workflow edge definitions

use serde::{Deserialize, Serialize};

/// Workflow edge connecting two nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_handle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_handle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub edge_type: Option<String>,
}

impl WorkflowEdge {
    /// Create a new edge between two nodes
    pub fn new(
        id: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        WorkflowEdge {
            id: id.into(),
            source: source.into(),
            target: target.into(),
            source_handle: None,
            target_handle: None,
            label: None,
            edge_type: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_serialization() {
        let edge = WorkflowEdge::new("edge_1", "node_1", "node_2");
        let json = serde_json::to_string(&edge).unwrap();
        assert!(json.contains("\"source\":\"node_1\""));
        assert!(json.contains("\"target\":\"node_2\""));
    }

    #[test]
    fn test_edge_deserialization() {
        let json = r#"{
            "id": "edge_1",
            "source": "node_1",
            "target": "node_2",
            "sourceHandle": "output",
            "label": "success"
        }"#;

        let edge: WorkflowEdge = serde_json::from_str(json).unwrap();
        assert_eq!(edge.id, "edge_1");
        assert_eq!(edge.source, "node_1");
        assert_eq!(edge.target, "node_2");
        assert_eq!(edge.source_handle, Some("output".to_string()));
        assert_eq!(edge.label, Some("success".to_string()));
    }
}
