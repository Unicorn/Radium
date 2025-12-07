//! Dependency graph construction and validation for plan execution.
//!
//! Provides DAG (Directed Acyclic Graph) functionality for task dependencies,
//! including cycle detection, topological sorting, and execution level calculation.

use crate::models::{PlanManifest, PlanTask};
use petgraph::algo::{is_cyclic_directed, toposort};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::Direction;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Errors that can occur during DAG operations.
#[derive(Debug, Error)]
pub enum DagError {
    /// Circular dependency detected.
    #[error("circular dependency detected: {0}")]
    CycleDetected(String),

    /// Dependency reference not found.
    #[error("dependency task not found: {0}")]
    DependencyNotFound(String),

    /// Invalid task ID format.
    #[error("invalid task ID format: {0}")]
    InvalidTaskId(String),

    /// Topological sort failed (should not happen if cycle detection works).
    #[error("topological sort failed: {0}")]
    TopologicalSortFailed(String),
}

/// Result type for DAG operations.
pub type Result<T> = std::result::Result<T, DagError>;

/// Dependency graph for plan tasks.
///
/// Builds a directed graph from plan manifest task dependencies and provides
/// algorithms for cycle detection, topological sorting, and execution level calculation.
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// The underlying graph structure.
    graph: DiGraph<String, ()>,

    /// Mapping from task ID to node index.
    node_map: HashMap<String, NodeIndex>,

    /// Reverse mapping from node index to task ID.
    task_map: HashMap<NodeIndex, String>,
}

impl DependencyGraph {
    /// Creates a new dependency graph from a plan manifest.
    ///
    /// # Arguments
    /// * `manifest` - The plan manifest containing tasks and dependencies
    ///
    /// # Errors
    /// Returns error if:
    /// - A dependency reference doesn't exist
    /// - A circular dependency is detected
    pub fn from_manifest(manifest: &PlanManifest) -> Result<Self> {
        let mut graph = DiGraph::new();
        let mut node_map = HashMap::new();
        let mut task_map = HashMap::new();

        // First pass: create nodes for all tasks
        for iteration in &manifest.iterations {
            for task in &iteration.tasks {
                let node = graph.add_node(task.id.clone());
                node_map.insert(task.id.clone(), node);
                task_map.insert(node, task.id.clone());
            }
        }

        // Second pass: create edges for dependencies and validate references
        for iteration in &manifest.iterations {
            for task in &iteration.tasks {
                let from_node = node_map.get(&task.id).ok_or_else(|| {
                    DagError::InvalidTaskId(format!("Task not found in node map: {}", task.id))
                })?;

                for dep_id in &task.dependencies {
                    // Validate dependency exists
                    let mut found = false;
                    for iter in &manifest.iterations {
                        if iter.get_task(dep_id).is_some() {
                            found = true;
                            break;
                        }
                    }

                    if !found {
                        return Err(DagError::DependencyNotFound(dep_id.clone()));
                    }

                    let to_node = node_map.get(dep_id).ok_or_else(|| {
                        DagError::DependencyNotFound(format!(
                            "Dependency task not found: {} (referenced by {})",
                            dep_id, task.id
                        ))
                    })?;

                    // Add edge: task depends on dep_id, so edge goes from dep_id to task
                    // (dependencies must complete before the task)
                    graph.add_edge(*to_node, *from_node, ());
                }
            }
        }

        // Check for cycles
        if is_cyclic_directed(&graph) {
            let cycle_path = Self::find_cycle_path(&graph, &task_map)?;
            return Err(DagError::CycleDetected(cycle_path));
        }

        Ok(Self { graph, node_map, task_map })
    }

    /// Finds a cycle path in the graph for error reporting.
    fn find_cycle_path(
        graph: &DiGraph<String, ()>,
        task_map: &HashMap<NodeIndex, String>,
    ) -> Result<String> {
        // Use DFS to find a cycle
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for node in graph.node_indices() {
            if Self::dfs_cycle(graph, node, &mut visited, &mut rec_stack, &mut path, task_map) {
                if !path.is_empty() {
                    let cycle_str = path
                        .iter()
                        .rev()
                        .map(|idx| task_map.get(idx).cloned().unwrap_or_else(|| "?".to_string()))
                        .collect::<Vec<_>>()
                        .join(" -> ");
                    return Ok(cycle_str);
                }
            }
        }

        Ok("unknown cycle".to_string())
    }

    /// DFS helper to detect cycles.
    fn dfs_cycle(
        graph: &DiGraph<String, ()>,
        node: NodeIndex,
        visited: &mut HashSet<NodeIndex>,
        rec_stack: &mut HashSet<NodeIndex>,
        path: &mut Vec<NodeIndex>,
        task_map: &HashMap<NodeIndex, String>,
    ) -> bool {
        visited.insert(node);
        rec_stack.insert(node);
        path.push(node);

        for neighbor in graph.neighbors_directed(node, Direction::Outgoing) {
            if !visited.contains(&neighbor) {
                if Self::dfs_cycle(graph, neighbor, visited, rec_stack, path, task_map) {
                    return true;
                }
            } else if rec_stack.contains(&neighbor) {
                // Found a cycle
                path.push(neighbor);
                return true;
            }
        }

        rec_stack.remove(&node);
        path.pop();
        false
    }

    /// Performs topological sort to get execution order.
    ///
    /// Returns tasks in an order where all dependencies come before dependents.
    ///
    /// # Errors
    /// Returns error if topological sort fails (should not happen if cycle detection passed).
    pub fn topological_sort(&self) -> Result<Vec<String>> {
        match toposort(&self.graph, None) {
            Ok(sorted_indices) => {
                let sorted_tasks: Vec<String> = sorted_indices
                    .iter()
                    .filter_map(|idx| self.task_map.get(idx).cloned())
                    .collect();
                Ok(sorted_tasks)
            }
            Err(cycle) => {
                // This should not happen if cycle detection worked, but handle it anyway
                let cycle_task = self
                    .task_map
                    .get(&cycle.node_id())
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string());
                Err(DagError::TopologicalSortFailed(format!(
                    "Cycle detected during topological sort involving: {}",
                    cycle_task
                )))
            }
        }
    }

    /// Detects cycles in the dependency graph.
    ///
    /// # Errors
    /// Returns error with cycle path if a cycle is detected.
    pub fn detect_cycles(&self) -> Result<()> {
        if is_cyclic_directed(&self.graph) {
            let cycle_path = Self::find_cycle_path(&self.graph, &self.task_map)?;
            return Err(DagError::CycleDetected(cycle_path));
        }
        Ok(())
    }

    /// Calculates execution levels for parallel scheduling.
    ///
    /// Tasks with no dependencies are at level 0.
    /// Tasks depending on level N tasks are at level N+1.
    ///
    /// # Returns
    /// A map from task ID to execution level.
    pub fn calculate_execution_levels(&self) -> HashMap<String, u32> {
        let mut levels = HashMap::new();
        let mut visited = HashSet::new();

        // Initialize all tasks
        for task_id in self.node_map.keys() {
            levels.insert(task_id.clone(), 0);
        }

        // Calculate levels using BFS-like approach
        let mut queue: Vec<(NodeIndex, u32)> = Vec::new();

        // Start with tasks that have no incoming edges (no dependencies)
        for node in self.graph.node_indices() {
            if self.graph.neighbors_directed(node, Direction::Incoming).count() == 0 {
                queue.push((node, 0));
                visited.insert(node);
            }
        }

        while let Some((node, level)) = queue.pop() {
            let task_id = self
                .task_map
                .get(&node)
                .cloned()
                .unwrap_or_else(|| "".to_string());
            levels.insert(task_id.clone(), level);

            // Update dependents
            for dependent in self.graph.neighbors_directed(node, Direction::Outgoing) {
                if !visited.contains(&dependent) {
                    let current_level = levels
                        .get(self.task_map.get(&dependent).unwrap())
                        .copied()
                        .unwrap_or(0);
                    let new_level = level + 1;
                    if new_level > current_level {
                        levels.insert(
                            self.task_map.get(&dependent).unwrap().clone(),
                            new_level,
                        );
                    }
                    visited.insert(dependent);
                    queue.push((dependent, new_level));
                } else {
                    // Update level if this path gives a higher level
                    let current_level = levels
                        .get(self.task_map.get(&dependent).unwrap())
                        .copied()
                        .unwrap_or(0);
                    let new_level = level + 1;
                    if new_level > current_level {
                        levels.insert(
                            self.task_map.get(&dependent).unwrap().clone(),
                            new_level,
                        );
                    }
                }
            }
        }

        levels
    }

    /// Gets all tasks that can be executed in parallel at a given level.
    ///
    /// # Arguments
    /// * `level` - The execution level
    ///
    /// # Returns
    /// Vector of task IDs at the specified level
    pub fn get_tasks_at_level(&self, level: u32) -> Vec<String> {
        let levels = self.calculate_execution_levels();
        levels
            .into_iter()
            .filter(|(_, l)| *l == level)
            .map(|(task_id, _)| task_id)
            .collect()
    }

    /// Gets the number of nodes (tasks) in the graph.
    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Gets the number of edges (dependencies) in the graph.
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Iteration, PlanManifest, PlanTask};
    use crate::workspace::RequirementId;
    use std::str::FromStr;

    fn create_test_manifest() -> PlanManifest {
        let req_id = RequirementId::from_str("REQ-001").unwrap();
        let mut manifest = PlanManifest::new(req_id, "Test Project".to_string());

        // Iteration 1: T1 -> T2 -> T3 (linear)
        let mut iter1 = Iteration::new(1, "Iteration 1".to_string());
        let task1 = PlanTask::new("I1", 1, "Task 1".to_string());
        let mut task2 = PlanTask::new("I1", 2, "Task 2".to_string());
        let mut task3 = PlanTask::new("I1", 3, "Task 3".to_string());

        task2.dependencies.push("I1.T1".to_string());
        task3.dependencies.push("I1.T2".to_string());

        iter1.add_task(task1);
        iter1.add_task(task2);
        iter1.add_task(task3);
        manifest.add_iteration(iter1);

        manifest
    }

    #[test]
    fn test_dag_construction() {
        let manifest = create_test_manifest();
        let dag = DependencyGraph::from_manifest(&manifest).unwrap();

        assert_eq!(dag.node_count(), 3);
        assert_eq!(dag.edge_count(), 2);
    }

    #[test]
    fn test_dag_cycle_detection() {
        let req_id = RequirementId::from_str("REQ-001").unwrap();
        let mut manifest = PlanManifest::new(req_id, "Test Project".to_string());

        let mut iter1 = Iteration::new(1, "Iteration 1".to_string());
        let mut task1 = PlanTask::new("I1", 1, "Task 1".to_string());
        let mut task2 = PlanTask::new("I1", 2, "Task 2".to_string());
        let mut task3 = PlanTask::new("I1", 3, "Task 3".to_string());

        // Create cycle: T1 -> T2 -> T3 -> T1
        task2.dependencies.push("I1.T1".to_string());
        task3.dependencies.push("I1.T2".to_string());
        task1.dependencies.push("I1.T3".to_string());

        iter1.add_task(task1);
        iter1.add_task(task2);
        iter1.add_task(task3);
        manifest.add_iteration(iter1);

        let result = DependencyGraph::from_manifest(&manifest);
        assert!(result.is_err());
        match result.unwrap_err() {
            DagError::CycleDetected(_) => {}
            _ => panic!("Expected CycleDetected error"),
        }
    }

    #[test]
    fn test_dag_topological_sort() {
        let manifest = create_test_manifest();
        let dag = DependencyGraph::from_manifest(&manifest).unwrap();

        let sorted = dag.topological_sort().unwrap();
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0], "I1.T1");
        assert_eq!(sorted[1], "I1.T2");
        assert_eq!(sorted[2], "I1.T3");
    }

    #[test]
    fn test_dag_execution_levels() {
        let req_id = RequirementId::from_str("REQ-001").unwrap();
        let mut manifest = PlanManifest::new(req_id, "Test Project".to_string());

        // Diamond dependency: T1 -> T2, T3 -> T4
        let mut iter1 = Iteration::new(1, "Iteration 1".to_string());
        let task1 = PlanTask::new("I1", 1, "Task 1".to_string());
        let mut task2 = PlanTask::new("I1", 2, "Task 2".to_string());
        let mut task3 = PlanTask::new("I1", 3, "Task 3".to_string());
        let mut task4 = PlanTask::new("I1", 4, "Task 4".to_string());

        task2.dependencies.push("I1.T1".to_string());
        task3.dependencies.push("I1.T1".to_string());
        task4.dependencies.push("I1.T2".to_string());
        task4.dependencies.push("I1.T3".to_string());

        iter1.add_task(task1);
        iter1.add_task(task2);
        iter1.add_task(task3);
        iter1.add_task(task4);
        manifest.add_iteration(iter1);

        let dag = DependencyGraph::from_manifest(&manifest).unwrap();
        let levels = dag.calculate_execution_levels();

        assert_eq!(levels.get("I1.T1"), Some(&0));
        assert_eq!(levels.get("I1.T2"), Some(&1));
        assert_eq!(levels.get("I1.T3"), Some(&1));
        assert_eq!(levels.get("I1.T4"), Some(&2));
    }

    #[test]
    fn test_dag_dependency_not_found() {
        let req_id = RequirementId::from_str("REQ-001").unwrap();
        let mut manifest = PlanManifest::new(req_id, "Test Project".to_string());

        let mut iter1 = Iteration::new(1, "Iteration 1".to_string());
        let mut task1 = PlanTask::new("I1", 1, "Task 1".to_string());
        task1.dependencies.push("I5.T1".to_string()); // Non-existent dependency

        iter1.add_task(task1);
        manifest.add_iteration(iter1);

        let result = DependencyGraph::from_manifest(&manifest);
        assert!(result.is_err());
        match result.unwrap_err() {
            DagError::DependencyNotFound(_) => {}
            _ => panic!("Expected DependencyNotFound error"),
        }
    }

    #[test]
    fn test_dag_get_tasks_at_level() {
        let req_id = RequirementId::from_str("REQ-001").unwrap();
        let mut manifest = PlanManifest::new(req_id, "Test Project".to_string());

        // Diamond dependency
        let mut iter1 = Iteration::new(1, "Iteration 1".to_string());
        let task1 = PlanTask::new("I1", 1, "Task 1".to_string());
        let mut task2 = PlanTask::new("I1", 2, "Task 2".to_string());
        let mut task3 = PlanTask::new("I1", 3, "Task 3".to_string());
        let mut task4 = PlanTask::new("I1", 4, "Task 4".to_string());

        task2.dependencies.push("I1.T1".to_string());
        task3.dependencies.push("I1.T1".to_string());
        task4.dependencies.push("I1.T2".to_string());
        task4.dependencies.push("I1.T3".to_string());

        iter1.add_task(task1);
        iter1.add_task(task2);
        iter1.add_task(task3);
        iter1.add_task(task4);
        manifest.add_iteration(iter1);

        let dag = DependencyGraph::from_manifest(&manifest).unwrap();

        let level0 = dag.get_tasks_at_level(0);
        assert_eq!(level0.len(), 1);
        assert!(level0.contains(&"I1.T1".to_string()));

        let level1 = dag.get_tasks_at_level(1);
        assert_eq!(level1.len(), 2);
        assert!(level1.contains(&"I1.T2".to_string()));
        assert!(level1.contains(&"I1.T3".to_string()));

        let level2 = dag.get_tasks_at_level(2);
        assert_eq!(level2.len(), 1);
        assert!(level2.contains(&"I1.T4".to_string()));
    }
}

