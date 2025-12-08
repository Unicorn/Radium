//! Dependency graph construction and validation for plan execution.
//!
//! This module provides a DAG (Directed Acyclic Graph) system for managing task dependencies
//! in plans. It enables cycle detection, topological sorting for execution order, and
//! execution level calculation for parallel scheduling.
//!
//! # Overview
//!
//! The DAG system ensures that task dependencies are valid and executable:
//!
//! - **Cycle Detection**: Prevents circular dependencies that would block execution
//! - **Topological Sort**: Determines the correct execution order of tasks
//! - **Execution Levels**: Calculates which tasks can run in parallel
//! - **Dependency Validation**: Ensures all referenced dependencies exist
//!
//! # Why DAGs Matter
//!
//! Task dependencies form a directed graph. If this graph contains cycles, tasks cannot
//! be executed because they depend on each other in a circular way. A DAG (Directed
//! Acyclic Graph) ensures there are no cycles, making execution possible.
//!
//! # Two-Pass Construction
//!
//! The graph is built in two passes for efficiency and correctness:
//!
//! 1. **First Pass**: Create nodes for all tasks
//! 2. **Second Pass**: Create edges for dependencies and validate references
//!
//! This approach ensures all nodes exist before creating edges, preventing invalid
//! references and enabling better error messages.
//!
//! # Example
//!
//! ```rust,no_run
//! use radium_core::planning::dag::DependencyGraph;
//! use radium_core::models::PlanManifest;
//!
//! # fn example(manifest: &PlanManifest) -> Result<(), Box<dyn std::error::Error>> {
//! // Build dependency graph
//! let dag = DependencyGraph::from_manifest(manifest)?;
//!
//! // Check for cycles
//! dag.detect_cycles()?;
//!
//! // Get execution order
//! let execution_order = dag.topological_sort()?;
//!
//! // Calculate parallel execution levels
//! let levels = dag.calculate_execution_levels();
//! // Tasks at level 0 can run in parallel
//! // Tasks at level N+1 depend on tasks at level N
//! # Ok(())
//! # }
//! ```
//!
//! # See Also
//!
//! - [User Guide](../../../docs/features/dag-dependencies.md) - Complete user documentation
//! - [Autonomous Planning](autonomous) - Uses DAG for plan validation
//! - [Plan Executor](executor) - Uses DAG for execution ordering

use crate::models::PlanManifest;
use petgraph::algo::{is_cyclic_directed, toposort};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::Direction;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Errors that can occur during DAG operations.
///
/// # Error Types
///
/// - **CycleDetected**: A circular dependency was found. The error message includes
///   the cycle path (e.g., "I1.T1 -> I1.T2 -> I1.T3 -> I1.T1")
/// - **DependencyNotFound**: A task references a dependency that doesn't exist
/// - **InvalidTaskId**: A task ID has an invalid format (should be "I{N}.T{N}")
/// - **TopologicalSortFailed**: Topological sort failed (should not happen if cycle
///   detection passed, but handled for safety)
///
/// # Example
///
/// ```rust,no_run
/// use radium_core::planning::dag::{DependencyGraph, DagError};
///
/// # fn example() -> Result<(), DagError> {
/// let dag = DependencyGraph::from_manifest(&manifest)?;
///
/// match dag.detect_cycles() {
///     Ok(()) => println!("No cycles detected"),
///     Err(DagError::CycleDetected(path)) => {
///         println!("Cycle found: {}", path);
///         // Fix the cycle by removing or reordering dependencies
///     }
///     Err(e) => return Err(e),
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Error)]
pub enum DagError {
    /// Circular dependency detected.
    ///
    /// The error message contains the cycle path showing which tasks form the cycle.
    /// Example: "I1.T1 -> I1.T2 -> I1.T3 -> I1.T1"
    #[error("circular dependency detected: {0}")]
    CycleDetected(String),

    /// Dependency reference not found.
    ///
    /// A task references a dependency that doesn't exist in the plan manifest.
    /// The error message includes the missing dependency ID.
    #[error("dependency task not found: {0}")]
    DependencyNotFound(String),

    /// Invalid task ID format.
    ///
    /// Task IDs must follow the format "I{N}.T{N}" where N is a number.
    /// Example: "I1.T1", "I2.T3"
    #[error("invalid task ID format: {0}")]
    InvalidTaskId(String),

    /// Topological sort failed (should not happen if cycle detection works).
    ///
    /// This error should not occur if cycle detection passed, but is handled
    /// for safety. Indicates an internal graph structure issue.
    #[error("topological sort failed: {0}")]
    TopologicalSortFailed(String),
}

/// Result type for DAG operations.
pub type Result<T> = std::result::Result<T, DagError>;

/// Dependency graph for plan tasks.
///
/// Builds a directed graph from plan manifest task dependencies and provides
/// algorithms for cycle detection, topological sorting, and execution level calculation.
///
/// # Construction
///
/// The graph is built in two passes:
/// 1. **Nodes**: Create a node for each task
/// 2. **Edges**: Create edges for dependencies (from dependency to dependent)
///
/// # Graph Structure
///
/// - **Nodes**: Represent tasks (task IDs)
/// - **Edges**: Represent dependencies (edge from dependency to dependent)
/// - **Direction**: Edges point from dependencies to dependents
///
/// This means: if Task B depends on Task A, there's an edge A → B
///
/// # Example
///
/// ```rust,no_run
/// use radium_core::planning::dag::DependencyGraph;
/// use radium_core::models::PlanManifest;
///
/// # fn example(manifest: &PlanManifest) -> Result<(), Box<dyn std::error::Error>> {
/// // Build graph from manifest
/// let dag = DependencyGraph::from_manifest(manifest)?;
///
/// // Get execution order
/// let order = dag.topological_sort()?;
///
/// // Get tasks that can run in parallel at level 0
/// let parallel_tasks = dag.get_tasks_at_level(0);
/// # Ok(())
/// # }
/// ```
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
    /// This ensures tasks are executed in the correct order, with dependencies
    /// completing before their dependents.
    ///
    /// # Returns
    ///
    /// A vector of task IDs in execution order. Tasks with no dependencies come first,
    /// followed by tasks whose dependencies have been completed.
    ///
    /// # Errors
    ///
    /// Returns error if topological sort fails (should not happen if cycle detection passed).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use radium_core::planning::dag::DependencyGraph;
    /// # fn example(dag: &DependencyGraph) -> Result<(), Box<dyn std::error::Error>> {
    /// let order = dag.topological_sort()?;
    /// // If T2 depends on T1, order will be [T1, T2, ...]
    /// # Ok(())
    /// # }
    /// ```
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
    /// This enables parallel execution: all tasks at the same level can run
    /// concurrently since they have no dependencies on each other.
    ///
    /// # Returns
    ///
    /// A map from task ID to execution level. Tasks at level 0 can start immediately.
    /// Tasks at level N+1 can start after all tasks at level N complete.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use radium_core::planning::dag::DependencyGraph;
    /// # fn example(dag: &DependencyGraph) {
    /// let levels = dag.calculate_execution_levels();
    /// // Tasks at level 0: no dependencies, can run in parallel
    /// // Tasks at level 1: depend on level 0 tasks
    /// // Tasks at level 2: depend on level 1 tasks, etc.
    /// # }
    /// ```
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

    #[test]
    fn test_dag_complex_dependency_patterns() {
        let req_id = RequirementId::from_str("REQ-001").unwrap();
        let mut manifest = PlanManifest::new(req_id, "Complex Project".to_string());

        // Create a complex dependency pattern: multiple chains converging
        let mut iter1 = Iteration::new(1, "Iteration 1".to_string());
        
        // Chain 1: T1 -> T2 -> T5
        let task1 = PlanTask::new("I1", 1, "Task 1".to_string());
        let mut task2 = PlanTask::new("I1", 2, "Task 2".to_string());
        task2.dependencies.push("I1.T1".to_string());
        let mut task5 = PlanTask::new("I1", 5, "Task 5".to_string());
        task5.dependencies.push("I1.T2".to_string());
        
        // Chain 2: T3 -> T4 -> T5 (converges with Chain 1)
        let task3 = PlanTask::new("I1", 3, "Task 3".to_string());
        let mut task4 = PlanTask::new("I1", 4, "Task 4".to_string());
        task4.dependencies.push("I1.T3".to_string());
        task5.dependencies.push("I1.T4".to_string());
        
        iter1.add_task(task1);
        iter1.add_task(task2);
        iter1.add_task(task3);
        iter1.add_task(task4);
        iter1.add_task(task5);
        manifest.add_iteration(iter1);

        let dag = DependencyGraph::from_manifest(&manifest).unwrap();
        
        // Should detect no cycles
        assert!(dag.detect_cycles().is_ok());
        
        // Topological sort should respect dependencies
        let sorted = dag.topological_sort().unwrap();
        assert_eq!(sorted.len(), 5);
        
        // T1 and T3 should come first (no dependencies)
        assert!(sorted.iter().position(|t| t == "I1.T1").unwrap() < sorted.iter().position(|t| t == "I1.T2").unwrap());
        assert!(sorted.iter().position(|t| t == "I1.T3").unwrap() < sorted.iter().position(|t| t == "I1.T4").unwrap());
        
        // T2 and T4 should come before T5
        assert!(sorted.iter().position(|t| t == "I1.T2").unwrap() < sorted.iter().position(|t| t == "I1.T5").unwrap());
        assert!(sorted.iter().position(|t| t == "I1.T4").unwrap() < sorted.iter().position(|t| t == "I1.T5").unwrap());
        
        // Execution levels
        let levels = dag.calculate_execution_levels();
        assert_eq!(levels.get("I1.T1"), Some(&0));
        assert_eq!(levels.get("I1.T3"), Some(&0));
        assert_eq!(levels.get("I1.T2"), Some(&1));
        assert_eq!(levels.get("I1.T4"), Some(&1));
        assert_eq!(levels.get("I1.T5"), Some(&2));
    }

    #[test]
    fn test_dag_multiple_cycles() {
        let req_id = RequirementId::from_str("REQ-001").unwrap();
        let mut manifest = PlanManifest::new(req_id, "Multi Cycle Project".to_string());

        // Create two separate cycles in different iterations
        let mut iter1 = Iteration::new(1, "Iteration 1".to_string());
        let mut task1 = PlanTask::new("I1", 1, "Task 1".to_string());
        let mut task2 = PlanTask::new("I1", 2, "Task 2".to_string());
        task1.dependencies.push("I1.T2".to_string());
        task2.dependencies.push("I1.T1".to_string());
        iter1.add_task(task1);
        iter1.add_task(task2);
        manifest.add_iteration(iter1);

        let result = DependencyGraph::from_manifest(&manifest);
        assert!(result.is_err());
        match result.unwrap_err() {
            DagError::CycleDetected(path) => {
                // Should detect the cycle
                assert!(path.contains("I1.T1") || path.contains("I1.T2"));
            }
            _ => panic!("Expected CycleDetected error"),
        }
    }

    #[test]
    fn test_dag_self_reference() {
        let req_id = RequirementId::from_str("REQ-001").unwrap();
        let mut manifest = PlanManifest::new(req_id, "Self Ref Project".to_string());

        let mut iter1 = Iteration::new(1, "Iteration 1".to_string());
        let mut task1 = PlanTask::new("I1", 1, "Task 1".to_string());
        task1.dependencies.push("I1.T1".to_string()); // Self-reference
        iter1.add_task(task1);
        manifest.add_iteration(iter1);

        let result = DependencyGraph::from_manifest(&manifest);
        assert!(result.is_err());
        match result.unwrap_err() {
            DagError::CycleDetected(_) => {} // Self-reference creates a cycle
            _ => panic!("Expected CycleDetected error for self-reference"),
        }
    }

    #[test]
    fn test_dag_empty_manifest() {
        let req_id = RequirementId::from_str("REQ-001").unwrap();
        let manifest = PlanManifest::new(req_id, "Empty Project".to_string());

        let dag = DependencyGraph::from_manifest(&manifest).unwrap();
        assert_eq!(dag.node_count(), 0);
        assert_eq!(dag.edge_count(), 0);
        
        let sorted = dag.topological_sort().unwrap();
        assert!(sorted.is_empty());
        
        let levels = dag.calculate_execution_levels();
        assert!(levels.is_empty());
    }

    #[test]
    fn test_dag_single_task_no_dependencies() {
        let req_id = RequirementId::from_str("REQ-001").unwrap();
        let mut manifest = PlanManifest::new(req_id, "Single Task Project".to_string());

        let mut iter1 = Iteration::new(1, "Iteration 1".to_string());
        let task1 = PlanTask::new("I1", 1, "Task 1".to_string());
        iter1.add_task(task1);
        manifest.add_iteration(iter1);

        let dag = DependencyGraph::from_manifest(&manifest).unwrap();
        assert_eq!(dag.node_count(), 1);
        assert_eq!(dag.edge_count(), 0);
        
        assert!(dag.detect_cycles().is_ok());
        
        let sorted = dag.topological_sort().unwrap();
        assert_eq!(sorted.len(), 1);
        assert_eq!(sorted[0], "I1.T1");
        
        let levels = dag.calculate_execution_levels();
        assert_eq!(levels.get("I1.T1"), Some(&0));
    }

    #[test]
    fn test_dag_topological_sort_respects_dependencies() {
        let manifest = create_test_manifest();
        let dag = DependencyGraph::from_manifest(&manifest).unwrap();

        // Topological sort should ensure T1 comes before T2, and T2 before T3
        let sorted = dag.topological_sort().unwrap();
        let t1_pos = sorted.iter().position(|t| t == "I1.T1").unwrap();
        let t2_pos = sorted.iter().position(|t| t == "I1.T2").unwrap();
        let t3_pos = sorted.iter().position(|t| t == "I1.T3").unwrap();

        assert!(t1_pos < t2_pos);
        assert!(t2_pos < t3_pos);
    }

    #[test]
    fn test_dag_invalid_task_id_format() {
        let req_id = RequirementId::from_str("REQ-001").unwrap();
        let mut manifest = PlanManifest::new(req_id, "Invalid ID Project".to_string());

        let mut iter1 = Iteration::new(1, "Iteration 1".to_string());
        let mut task1 = PlanTask::new("I1", 1, "Task 1".to_string());
        task1.dependencies.push("INVALID".to_string()); // Non-existent dependency
        iter1.add_task(task1);
        manifest.add_iteration(iter1);

        let result = DependencyGraph::from_manifest(&manifest);
        assert!(result.is_err());
        match result.unwrap_err() {
            DagError::DependencyNotFound(_) => {} // Invalid format is treated as missing dependency
            _ => panic!("Expected DependencyNotFound error"),
        }
    }
}

