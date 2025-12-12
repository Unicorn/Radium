//! Workflow node types and definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Node types - exhaustive enum prevents invalid types at compile time
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum NodeType {
    Trigger,
    Activity,
    Agent,
    Conditional,
    Loop,
    ChildWorkflow,
    Signal,
    Phase,
    Retry,
    StateVariable,
    ApiEndpoint,
    Condition,
    End,
    DataIn,
    DataOut,
    KongLogging,
    KongCache,
    KongCors,
    GraphqlGateway,
    McpServer,
}

impl NodeType {
    /// Check if this node type is a trigger (start) node
    pub fn is_trigger(&self) -> bool {
        matches!(self, NodeType::Trigger)
    }

    /// Check if this node type is an end node
    pub fn is_end(&self) -> bool {
        matches!(self, NodeType::End)
    }

    /// Check if this node type is an activity
    pub fn is_activity(&self) -> bool {
        matches!(self, NodeType::Activity | NodeType::Agent)
    }

    /// Check if this node type is a Kong component
    pub fn is_kong_component(&self) -> bool {
        matches!(
            self,
            NodeType::KongLogging | NodeType::KongCache | NodeType::KongCors
        )
    }
}

/// Position on canvas
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

/// Retry strategy - exhaustive enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RetryStrategy {
    KeepTrying,
    FailAfterX,
    ExponentialBackoff,
    None,
}

impl Default for RetryStrategy {
    fn default() -> Self {
        RetryStrategy::None
    }
}

/// Retry policy with validation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RetryPolicy {
    pub strategy: RetryStrategy,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_attempts: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_interval: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_interval: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backoff_coefficient: Option<f64>,
}

/// Node data - all possible configuration fields
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NodeData {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_policy: Option<RetryPolicy>,
    /// Input mapping from previous nodes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<HashMap<String, serde_json::Value>>,
    /// Description for documentation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Workflow node with strict typing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: NodeType,
    pub data: NodeData,
    pub position: Position,
}

impl WorkflowNode {
    /// Get the effective activity name for this node
    pub fn activity_name(&self) -> Option<&str> {
        self.data
            .activity_name
            .as_deref()
            .or(self.data.component_name.as_deref())
    }

    /// Check if this node is a trigger
    pub fn is_trigger(&self) -> bool {
        self.node_type.is_trigger()
    }

    /// Check if this node is an end node
    pub fn is_end(&self) -> bool {
        self.node_type.is_end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_type_serialization() {
        let node_type = NodeType::Activity;
        let json = serde_json::to_string(&node_type).unwrap();
        assert_eq!(json, "\"activity\"");

        let node_type = NodeType::KongLogging;
        let json = serde_json::to_string(&node_type).unwrap();
        assert_eq!(json, "\"kong-logging\"");
    }

    #[test]
    fn test_node_type_deserialization() {
        let node_type: NodeType = serde_json::from_str("\"trigger\"").unwrap();
        assert_eq!(node_type, NodeType::Trigger);

        let node_type: NodeType = serde_json::from_str("\"child-workflow\"").unwrap();
        assert_eq!(node_type, NodeType::ChildWorkflow);
    }

    #[test]
    fn test_retry_strategy() {
        let strategy: RetryStrategy = serde_json::from_str("\"exponential-backoff\"").unwrap();
        assert_eq!(strategy, RetryStrategy::ExponentialBackoff);
    }

    #[test]
    fn test_workflow_node_deserialization() {
        let json = r#"{
            "id": "node_1",
            "type": "activity",
            "data": {
                "label": "Test Activity",
                "activityName": "doSomething"
            },
            "position": { "x": 100, "y": 200 }
        }"#;

        let node: WorkflowNode = serde_json::from_str(json).unwrap();
        assert_eq!(node.id, "node_1");
        assert_eq!(node.node_type, NodeType::Activity);
        assert_eq!(node.data.label, "Test Activity");
        assert_eq!(node.data.activity_name, Some("doSomething".to_string()));
    }
}
