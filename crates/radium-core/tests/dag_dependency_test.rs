//! Integration tests for DAG dependency management system.
//!
//! Tests dependency graph construction, cycle detection, topological sorting,
//! and execution level calculation.

use radium_core::models::{Iteration, PlanManifest, PlanTask};
use radium_core::planning::{DagError, DependencyGraph};
use radium_core::workspace::RequirementId;
use std::str::FromStr;

// Helper to create a simple linear manifest
fn create_linear_manifest() -> PlanManifest {
    let req_id = RequirementId::from_str("REQ-001").unwrap();
    let mut manifest = PlanManifest::new(req_id, "Test Project".to_string());

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

// Helper to create a diamond dependency manifest
fn create_diamond_manifest() -> PlanManifest {
    let req_id = RequirementId::from_str("REQ-001").unwrap();
    let mut manifest = PlanManifest::new(req_id, "Test Project".to_string());

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

    manifest
}

// Helper to create a cyclic manifest
fn create_cyclic_manifest() -> PlanManifest {
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

    manifest
}

#[test]
fn test_dag_construction_simple() {
    let manifest = create_linear_manifest();
    let dag = DependencyGraph::from_manifest(&manifest).unwrap();

    assert_eq!(dag.node_count(), 3);
    assert_eq!(dag.edge_count(), 2);
}

#[test]
fn test_dag_construction_diamond() {
    let manifest = create_diamond_manifest();
    let dag = DependencyGraph::from_manifest(&manifest).unwrap();

    assert_eq!(dag.node_count(), 4);
    assert_eq!(dag.edge_count(), 4); // T1->T2, T1->T3, T2->T4, T3->T4
}

#[test]
fn test_dag_cycle_detection_simple() {
    let manifest = create_cyclic_manifest();
    let result = DependencyGraph::from_manifest(&manifest);

    assert!(result.is_err());
    match result.unwrap_err() {
        DagError::CycleDetected(path) => {
            // Path should contain the cycle
            assert!(path.contains("I1.T1"));
            assert!(path.contains("I1.T2"));
            assert!(path.contains("I1.T3"));
        }
        e => panic!("Expected CycleDetected, got {:?}", e),
    }
}

#[test]
fn test_dag_cycle_detection_complex() {
    let req_id = RequirementId::from_str("REQ-001").unwrap();
    let mut manifest = PlanManifest::new(req_id, "Test Project".to_string());

    let mut iter1 = Iteration::new(1, "Iteration 1".to_string());
    let mut task1 = PlanTask::new("I1", 1, "Task 1".to_string());
    let mut task2 = PlanTask::new("I1", 2, "Task 2".to_string());
    let mut task3 = PlanTask::new("I1", 3, "Task 3".to_string());
    let mut task4 = PlanTask::new("I1", 4, "Task 4".to_string());

    // Complex cycle: T1 -> T2 -> T3 -> T4 -> T2
    task2.dependencies.push("I1.T1".to_string());
    task3.dependencies.push("I1.T2".to_string());
    task4.dependencies.push("I1.T3".to_string());
    task2.dependencies.push("I1.T4".to_string()); // Creates cycle

    iter1.add_task(task1);
    iter1.add_task(task2);
    iter1.add_task(task3);
    iter1.add_task(task4);
    manifest.add_iteration(iter1);

    let result = DependencyGraph::from_manifest(&manifest);
    assert!(result.is_err());
    match result.unwrap_err() {
        DagError::CycleDetected(_) => {}
        e => panic!("Expected CycleDetected, got {:?}", e),
    }
}

#[test]
fn test_dag_topological_sort_linear() {
    let manifest = create_linear_manifest();
    let dag = DependencyGraph::from_manifest(&manifest).unwrap();

    let sorted = dag.topological_sort().unwrap();
    assert_eq!(sorted.len(), 3);
    assert_eq!(sorted[0], "I1.T1");
    assert_eq!(sorted[1], "I1.T2");
    assert_eq!(sorted[2], "I1.T3");
}

#[test]
fn test_dag_topological_sort_diamond() {
    let manifest = create_diamond_manifest();
    let dag = DependencyGraph::from_manifest(&manifest).unwrap();

    let sorted = dag.topological_sort().unwrap();
    assert_eq!(sorted.len(), 4);
    assert_eq!(sorted[0], "I1.T1"); // Must be first
    assert_eq!(sorted[3], "I1.T4"); // Must be last
    // T2 and T3 can be in any order (both depend on T1, both needed by T4)
    assert!(sorted.contains(&"I1.T2".to_string()));
    assert!(sorted.contains(&"I1.T3".to_string()));
}

#[test]
fn test_dag_execution_levels_linear() {
    let manifest = create_linear_manifest();
    let dag = DependencyGraph::from_manifest(&manifest).unwrap();

    let levels = dag.calculate_execution_levels();

    assert_eq!(levels.get("I1.T1"), Some(&0));
    assert_eq!(levels.get("I1.T2"), Some(&1));
    assert_eq!(levels.get("I1.T3"), Some(&2));
}

#[test]
fn test_dag_execution_levels_diamond() {
    let manifest = create_diamond_manifest();
    let dag = DependencyGraph::from_manifest(&manifest).unwrap();

    let levels = dag.calculate_execution_levels();

    assert_eq!(levels.get("I1.T1"), Some(&0));
    assert_eq!(levels.get("I1.T2"), Some(&1));
    assert_eq!(levels.get("I1.T3"), Some(&1));
    assert_eq!(levels.get("I1.T4"), Some(&2));
}

#[test]
fn test_dag_execution_levels_parallel() {
    let req_id = RequirementId::from_str("REQ-001").unwrap();
    let mut manifest = PlanManifest::new(req_id, "Test Project".to_string());

    let mut iter1 = Iteration::new(1, "Iteration 1".to_string());
    let task1 = PlanTask::new("I1", 1, "Task 1".to_string());
    let task2 = PlanTask::new("I1", 2, "Task 2".to_string());
    let task3 = PlanTask::new("I1", 3, "Task 3".to_string());
    let mut task4 = PlanTask::new("I1", 4, "Task 4".to_string());

    // T1, T2, T3 have no dependencies (can run in parallel at level 0)
    // T4 depends on all three
    task4.dependencies.push("I1.T1".to_string());
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
    assert_eq!(levels.get("I1.T2"), Some(&0));
    assert_eq!(levels.get("I1.T3"), Some(&0));
    assert_eq!(levels.get("I1.T4"), Some(&1));

    // Verify parallel execution
    let level0 = dag.get_tasks_at_level(0);
    assert_eq!(level0.len(), 3);
    assert!(level0.contains(&"I1.T1".to_string()));
    assert!(level0.contains(&"I1.T2".to_string()));
    assert!(level0.contains(&"I1.T3".to_string()));
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
        DagError::DependencyNotFound(dep_id) => {
            assert_eq!(dep_id, "I5.T1");
        }
        e => panic!("Expected DependencyNotFound, got {:?}", e),
    }
}

#[test]
fn test_dag_invalid_task_id() {
    let req_id = RequirementId::from_str("REQ-001").unwrap();
    let mut manifest = PlanManifest::new(req_id, "Test Project".to_string());

    let mut iter1 = Iteration::new(1, "Iteration 1".to_string());
    // Create task with invalid ID format
    let mut task = PlanTask::new("I1", 1, "Task 1".to_string());
    task.id = "invalid-format".to_string(); // Invalid format
    iter1.add_task(task);
    manifest.add_iteration(iter1);

    // This should still work (task ID format is validated elsewhere)
    // But if we try to reference it, it might fail
    let result = DependencyGraph::from_manifest(&manifest);
    // Should succeed - invalid format is handled at task creation, not DAG construction
    assert!(result.is_ok());
}

#[test]
fn test_dag_get_dependencies_and_dependents() {
    let manifest = create_diamond_manifest();
    let dag = DependencyGraph::from_manifest(&manifest).unwrap();

    // T1 has no dependencies
    // T2 and T3 depend on T1
    // T4 depends on T2 and T3

    // Note: The current DAG implementation doesn't expose get_dependencies/get_dependents
    // But we can verify the structure through topological sort and execution levels
    let sorted = dag.topological_sort().unwrap();
    assert_eq!(sorted[0], "I1.T1"); // No dependencies
    assert!(sorted.contains(&"I1.T2"));
    assert!(sorted.contains(&"I1.T3"));
    assert_eq!(sorted[3], "I1.T4"); // Depends on T2 and T3
}

#[test]
fn test_dag_empty_manifest() {
    let req_id = RequirementId::from_str("REQ-001").unwrap();
    let manifest = PlanManifest::new(req_id, "Test Project".to_string());

    let dag = DependencyGraph::from_manifest(&manifest).unwrap();
    assert_eq!(dag.node_count(), 0);
    assert_eq!(dag.edge_count(), 0);

    let sorted = dag.topological_sort().unwrap();
    assert!(sorted.is_empty());

    let levels = dag.calculate_execution_levels();
    assert!(levels.is_empty());
}

#[test]
fn test_dag_single_node() {
    let req_id = RequirementId::from_str("REQ-001").unwrap();
    let mut manifest = PlanManifest::new(req_id, "Test Project".to_string());

    let mut iter1 = Iteration::new(1, "Iteration 1".to_string());
    let task1 = PlanTask::new("I1", 1, "Task 1".to_string());
    iter1.add_task(task1);
    manifest.add_iteration(iter1);

    let dag = DependencyGraph::from_manifest(&manifest).unwrap();
    assert_eq!(dag.node_count(), 1);
    assert_eq!(dag.edge_count(), 0);

    let sorted = dag.topological_sort().unwrap();
    assert_eq!(sorted.len(), 1);
    assert_eq!(sorted[0], "I1.T1");

    let levels = dag.calculate_execution_levels();
    assert_eq!(levels.get("I1.T1"), Some(&0));
}

#[test]
fn test_dag_disconnected_components() {
    let req_id = RequirementId::from_str("REQ-001").unwrap();
    let mut manifest = PlanManifest::new(req_id, "Test Project".to_string());

    // Two disconnected task groups
    let mut iter1 = Iteration::new(1, "Iteration 1".to_string());
    let task1 = PlanTask::new("I1", 1, "Task 1".to_string());
    let mut task2 = PlanTask::new("I1", 2, "Task 2".to_string());
    task2.dependencies.push("I1.T1".to_string());

    let task3 = PlanTask::new("I1", 3, "Task 3".to_string());
    let mut task4 = PlanTask::new("I1", 4, "Task 4".to_string());
    task4.dependencies.push("I1.T3".to_string());

    iter1.add_task(task1);
    iter1.add_task(task2);
    iter1.add_task(task3);
    iter1.add_task(task4);
    manifest.add_iteration(iter1);

    let dag = DependencyGraph::from_manifest(&manifest).unwrap();
    assert_eq!(dag.node_count(), 4);
    assert_eq!(dag.edge_count(), 2); // T1->T2, T3->T4

    let sorted = dag.topological_sort().unwrap();
    assert_eq!(sorted.len(), 4);
    // T1 and T3 should come before T2 and T4 respectively
    let t1_idx = sorted.iter().position(|s| s == "I1.T1").unwrap();
    let t2_idx = sorted.iter().position(|s| s == "I1.T2").unwrap();
    let t3_idx = sorted.iter().position(|s| s == "I1.T3").unwrap();
    let t4_idx = sorted.iter().position(|s| s == "I1.T4").unwrap();

    assert!(t1_idx < t2_idx);
    assert!(t3_idx < t4_idx);
}

