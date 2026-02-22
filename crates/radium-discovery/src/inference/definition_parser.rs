//! Parse workflow definitions to extract component and workflow references

use serde_json::Value;

/// Relationships extracted from a workflow definition
#[derive(Debug, Default, PartialEq)]
pub struct ExtractedRelationships {
    /// Component IDs used in the workflow (from nodes)
    pub component_ids: Vec<String>,
    /// Child workflow IDs referenced (from child_workflow nodes)
    pub child_workflow_ids: Vec<String>,
}

/// Extract component and workflow references from a definition JSON
pub fn extract_relationships(definition: &Value) -> ExtractedRelationships {
    let mut result = ExtractedRelationships::default();

    // The definition should have a "nodes" array
    let Some(nodes) = definition.get("nodes").and_then(Value::as_array) else {
        return result;
    };

    for node in nodes {
        // Extract component_id from node
        if let Some(component_id) = node.get("component_id").and_then(Value::as_str) {
            if !component_id.is_empty() {
                result.component_ids.push(component_id.to_string());
            }
        }

        // Extract node_type as a component reference
        if let Some(node_type) = node.get("node_type").and_then(Value::as_str) {
            // Skip special types
            if !matches!(node_type, "start" | "stop" | "trigger" | "child_workflow" | "activity")
                && !result.component_ids.contains(&node_type.to_string())
            {
                result.component_ids.push(node_type.to_string());
            }
        }

        // Extract child workflow references
        if let Some(node_type) = node.get("type").and_then(Value::as_str) {
            if node_type == "child_workflow" {
                if let Some(workflow_id) = node
                    .get("config")
                    .and_then(|c| c.get("workflow_id"))
                    .and_then(Value::as_str)
                {
                    if !workflow_id.is_empty() {
                        result.child_workflow_ids.push(workflow_id.to_string());
                    }
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_empty_definition() {
        let def = json!({});
        let result = extract_relationships(&def);
        assert!(result.component_ids.is_empty());
        assert!(result.child_workflow_ids.is_empty());
    }

    #[test]
    fn test_extract_component_ids() {
        let def = json!({
            "nodes": [
                {"component_id": "send-email", "node_type": "activity"},
                {"component_id": "http-request", "node_type": "activity"},
            ]
        });
        let result = extract_relationships(&def);
        assert_eq!(result.component_ids, vec!["send-email", "http-request"]);
    }

    #[test]
    fn test_extract_node_types() {
        let def = json!({
            "nodes": [
                {"node_type": "http_request"},
                {"node_type": "database_query"},
                {"node_type": "start"},
                {"node_type": "stop"},
            ]
        });
        let result = extract_relationships(&def);
        assert_eq!(result.component_ids, vec!["http_request", "database_query"]);
    }

    #[test]
    fn test_extract_child_workflows() {
        let def = json!({
            "nodes": [
                {
                    "type": "child_workflow",
                    "config": {"workflow_id": "workflow-123"}
                },
                {
                    "type": "child_workflow",
                    "config": {"workflow_id": "workflow-456"}
                }
            ]
        });
        let result = extract_relationships(&def);
        assert_eq!(
            result.child_workflow_ids,
            vec!["workflow-123", "workflow-456"]
        );
    }

    #[test]
    fn test_extract_mixed() {
        let def = json!({
            "nodes": [
                {"component_id": "send-email", "node_type": "activity"},
                {"type": "child_workflow", "config": {"workflow_id": "sub-1"}},
                {"node_type": "trigger"},
            ]
        });
        let result = extract_relationships(&def);
        assert_eq!(result.component_ids, vec!["send-email"]);
        assert_eq!(result.child_workflow_ids, vec!["sub-1"]);
    }

    #[test]
    fn test_no_duplicates() {
        let def = json!({
            "nodes": [
                {"component_id": "send-email"},
                {"node_type": "send-email"},
            ]
        });
        let result = extract_relationships(&def);
        assert_eq!(result.component_ids.len(), 1);
    }
}
