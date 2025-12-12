//! Graph structure validation

use std::collections::{HashMap, HashSet, VecDeque};

use crate::schema::WorkflowDefinition;

use super::errors::{ValidationError, ValidationResult};

/// Validate the graph structure of a workflow
pub fn validate_graph(workflow: &WorkflowDefinition) -> ValidationResult {
    let mut result = ValidationResult::new();

    // Build node ID set for quick lookup
    let node_ids: HashSet<&str> = workflow.nodes.iter().map(|n| n.id.as_str()).collect();

    // Validate edges reference existing nodes
    for edge in &workflow.edges {
        if !node_ids.contains(edge.source.as_str()) {
            result.add_error(ValidationError::InvalidEdgeSource(edge.source.clone()));
        }
        if !node_ids.contains(edge.target.as_str()) {
            result.add_error(ValidationError::InvalidEdgeTarget(edge.target.clone()));
        }
    }

    // Find start nodes (triggers)
    let start_nodes: Vec<&str> = workflow
        .nodes
        .iter()
        .filter(|n| n.is_trigger())
        .map(|n| n.id.as_str())
        .collect();

    if start_nodes.is_empty() {
        result.add_error(ValidationError::NoStartNode);
    } else if start_nodes.len() > 1 {
        result.add_error(ValidationError::MultipleStartNodes(
            start_nodes.iter().map(|s| s.to_string()).collect(),
        ));
    }

    // Find end nodes
    let end_nodes: Vec<&str> = workflow
        .nodes
        .iter()
        .filter(|n| n.is_end())
        .map(|n| n.id.as_str())
        .collect();

    if end_nodes.is_empty() {
        result.add_error(ValidationError::NoEndNode);
    }

    // Check trigger has no incoming edges
    for trigger_id in &start_nodes {
        let incoming = workflow.edges.iter().any(|e| e.target == *trigger_id);
        if incoming {
            result.add_error(ValidationError::TriggerHasIncomingEdges {
                node_id: trigger_id.to_string(),
            });
        }
    }

    // Check for orphan nodes (no edges connected)
    let connected_nodes = get_connected_nodes(workflow);
    for node in &workflow.nodes {
        if !connected_nodes.contains(node.id.as_str()) && workflow.nodes.len() > 1 {
            result.add_error(ValidationError::OrphanNode(node.id.clone()));
        }
    }

    // Check for unreachable nodes (BFS from start)
    if let Some(start_id) = start_nodes.first() {
        let reachable = get_reachable_nodes(workflow, start_id);
        for node in &workflow.nodes {
            if !reachable.contains(node.id.as_str()) && !node.is_trigger() {
                result.add_error(ValidationError::UnreachableNode {
                    node_id: node.id.clone(),
                });
            }
        }
    }

    // Check for cycles
    if let Some(cycle) = detect_cycle(workflow) {
        result.add_error(ValidationError::CycleDetected(cycle));
    }

    result
}

/// Get all nodes that have at least one edge connected
fn get_connected_nodes(workflow: &WorkflowDefinition) -> HashSet<&str> {
    let mut connected = HashSet::new();
    for edge in &workflow.edges {
        connected.insert(edge.source.as_str());
        connected.insert(edge.target.as_str());
    }
    connected
}

/// Get all nodes reachable from a start node using BFS
fn get_reachable_nodes<'a>(
    workflow: &'a WorkflowDefinition,
    start_id: &'a str,
) -> HashSet<&'a str> {
    let mut reachable = HashSet::new();
    let mut queue = VecDeque::new();

    // Build adjacency map
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for edge in &workflow.edges {
        adj.entry(edge.source.as_str())
            .or_default()
            .push(edge.target.as_str());
    }

    queue.push_back(start_id);
    reachable.insert(start_id);

    while let Some(current) = queue.pop_front() {
        if let Some(neighbors) = adj.get(current) {
            for neighbor in neighbors {
                if !reachable.contains(neighbor) {
                    reachable.insert(neighbor);
                    queue.push_back(neighbor);
                }
            }
        }
    }

    reachable
}

/// Detect cycles in the workflow graph using DFS
fn detect_cycle(workflow: &WorkflowDefinition) -> Option<Vec<String>> {
    // Build adjacency map
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for edge in &workflow.edges {
        adj.entry(edge.source.as_str())
            .or_default()
            .push(edge.target.as_str());
    }

    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();
    let mut path = Vec::new();

    for node in &workflow.nodes {
        if !visited.contains(node.id.as_str()) {
            if let Some(cycle) = dfs_cycle_detect(
                node.id.as_str(),
                &adj,
                &mut visited,
                &mut rec_stack,
                &mut path,
            ) {
                return Some(cycle);
            }
        }
    }

    None
}

/// DFS helper for cycle detection
fn dfs_cycle_detect<'a>(
    node: &'a str,
    adj: &HashMap<&str, Vec<&'a str>>,
    visited: &mut HashSet<&'a str>,
    rec_stack: &mut HashSet<&'a str>,
    path: &mut Vec<&'a str>,
) -> Option<Vec<String>> {
    visited.insert(node);
    rec_stack.insert(node);
    path.push(node);

    if let Some(neighbors) = adj.get(node) {
        for neighbor in neighbors {
            if !visited.contains(neighbor) {
                if let Some(cycle) = dfs_cycle_detect(neighbor, adj, visited, rec_stack, path) {
                    return Some(cycle);
                }
            } else if rec_stack.contains(neighbor) {
                // Found a cycle - extract the cycle path
                let cycle_start = path.iter().position(|&n| n == *neighbor).unwrap();
                let cycle: Vec<String> =
                    path[cycle_start..].iter().map(|s| s.to_string()).collect();
                return Some(cycle);
            }
        }
    }

    path.pop();
    rec_stack.remove(node);
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{
        NodeData, NodeType, Position, WorkflowEdge, WorkflowNode, WorkflowSettings,
    };

    fn create_node(id: &str, node_type: NodeType) -> WorkflowNode {
        WorkflowNode {
            id: id.to_string(),
            node_type,
            data: NodeData {
                label: id.to_string(),
                ..Default::default()
            },
            position: Position::default(),
        }
    }

    #[test]
    fn test_valid_linear_workflow() {
        let workflow = WorkflowDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            nodes: vec![
                create_node("trigger", NodeType::Trigger),
                create_node("activity", NodeType::Activity),
                create_node("end", NodeType::End),
            ],
            edges: vec![
                WorkflowEdge::new("e1", "trigger", "activity"),
                WorkflowEdge::new("e2", "activity", "end"),
            ],
            variables: vec![],
            settings: WorkflowSettings::default(),
        };

        let result = validate_graph(&workflow);
        assert!(result.is_valid(), "Errors: {:?}", result.errors);
    }

    #[test]
    fn test_cycle_detection() {
        let workflow = WorkflowDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            nodes: vec![
                create_node("trigger", NodeType::Trigger),
                create_node("a", NodeType::Activity),
                create_node("b", NodeType::Activity),
                create_node("end", NodeType::End),
            ],
            edges: vec![
                WorkflowEdge::new("e1", "trigger", "a"),
                WorkflowEdge::new("e2", "a", "b"),
                WorkflowEdge::new("e3", "b", "a"), // Creates cycle
                WorkflowEdge::new("e4", "b", "end"),
            ],
            variables: vec![],
            settings: WorkflowSettings::default(),
        };

        let result = validate_graph(&workflow);
        assert!(!result.is_valid());
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::CycleDetected(_))));
    }

    #[test]
    fn test_orphan_node_detection() {
        let workflow = WorkflowDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            nodes: vec![
                create_node("trigger", NodeType::Trigger),
                create_node("orphan", NodeType::Activity), // Not connected
                create_node("end", NodeType::End),
            ],
            edges: vec![WorkflowEdge::new("e1", "trigger", "end")],
            variables: vec![],
            settings: WorkflowSettings::default(),
        };

        let result = validate_graph(&workflow);
        assert!(!result.is_valid());
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::OrphanNode(id) if id == "orphan")));
    }

    #[test]
    fn test_invalid_edge_source() {
        let workflow = WorkflowDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            nodes: vec![
                create_node("trigger", NodeType::Trigger),
                create_node("end", NodeType::End),
            ],
            edges: vec![
                WorkflowEdge::new("e1", "nonexistent", "end"), // Invalid source
            ],
            variables: vec![],
            settings: WorkflowSettings::default(),
        };

        let result = validate_graph(&workflow);
        assert!(!result.is_valid());
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidEdgeSource(_))));
    }

    #[test]
    fn test_trigger_with_incoming_edge() {
        let workflow = WorkflowDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            nodes: vec![
                create_node("trigger", NodeType::Trigger),
                create_node("activity", NodeType::Activity),
                create_node("end", NodeType::End),
            ],
            edges: vec![
                WorkflowEdge::new("e1", "trigger", "activity"),
                WorkflowEdge::new("e2", "activity", "trigger"), // Invalid - edge to trigger
                WorkflowEdge::new("e3", "activity", "end"),
            ],
            variables: vec![],
            settings: WorkflowSettings::default(),
        };

        let result = validate_graph(&workflow);
        assert!(!result.is_valid());
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::TriggerHasIncomingEdges { .. })));
    }
}
