---
id: "dag-dependencies"
title: "DAG Dependency Management System"
sidebar_label: "DAG Dependency Management S..."
---

# DAG Dependency Management System

The DAG (Directed Acyclic Graph) dependency management system ensures task dependencies are valid and executable. It provides cycle detection, topological sorting for execution order, and execution level calculation for parallel scheduling.

## Overview

The DAG system is essential for plan execution because it:

- **Prevents Cycles**: Detects circular dependencies that would block execution
- **Determines Order**: Uses topological sort to find the correct execution order
- **Enables Parallelism**: Calculates execution levels for parallel task execution
- **Validates Dependencies**: Ensures all referenced dependencies exist

### Key Features

- **Cycle Detection**: Identifies circular dependencies with path reporting
- **Topological Sort**: Determines execution order respecting dependencies
- **Execution Levels**: Calculates which tasks can run in parallel
- **Dependency Validation**: Verifies all dependencies exist before execution
- **Two-Pass Construction**: Efficient graph building with validation

## Why DAGs Matter

Task dependencies form a directed graph. If this graph contains cycles, tasks cannot be executed because they depend on each other in a circular way.

### Example: Circular Dependency

```
Task A depends on Task B
Task B depends on Task C
Task C depends on Task A  ← Cycle!
```

This creates an impossible situation: A needs B, B needs C, C needs A. No task can start.

A DAG (Directed Acyclic Graph) ensures there are no cycles, making execution possible.

## How It Works

### Two-Pass Construction

The graph is built in two passes for efficiency and correctness:

1. **First Pass - Nodes**: Create a node for each task in the plan
2. **Second Pass - Edges**: Create edges for dependencies and validate references

This approach ensures:
- All nodes exist before creating edges
- Invalid references are caught early
- Better error messages with context

### Graph Structure

- **Nodes**: Represent tasks (task IDs like "I1.T1")
- **Edges**: Represent dependencies (edge from dependency to dependent)
- **Direction**: Edges point from dependencies to dependents

**Important**: If Task B depends on Task A, there's an edge A → B (A must complete before B).

## API Usage

### Building a Dependency Graph

```rust
use radium_core::planning::dag::DependencyGraph;
use radium_core::models::PlanManifest;

fn build_graph(manifest: &PlanManifest) -> Result<DependencyGraph, DagError> {
    let dag = DependencyGraph::from_manifest(manifest)?;
    Ok(dag)
}
```

### Cycle Detection

Detect circular dependencies before execution:

```rust
use radium_core::planning::dag::{DependencyGraph, DagError};

fn check_cycles(dag: &DependencyGraph) -> Result<(), DagError> {
    dag.detect_cycles()?;
    println!("No cycles detected - graph is valid");
    Ok(())
}

// Handle cycle errors
match dag.detect_cycles() {
    Ok(()) => println!("Graph is valid"),
    Err(DagError::CycleDetected(path)) => {
        println!("Cycle detected: {}", path);
        // Example: "I1.T1 -> I1.T2 -> I1.T3 -> I1.T1"
        // Fix by removing or reordering dependencies
    }
    Err(e) => return Err(e),
}
```

### Topological Sort

Get tasks in execution order:

```rust
fn get_execution_order(dag: &DependencyGraph) -> Result<Vec<String>, DagError> {
    let order = dag.topological_sort()?;
    // Returns tasks in dependency order
    // Tasks with no dependencies come first
    // Dependent tasks come after their dependencies
    Ok(order)
}

// Example output for linear dependencies:
// ["I1.T1", "I1.T2", "I1.T3"]
// T1 has no dependencies, T2 depends on T1, T3 depends on T2
```

### Execution Levels

Calculate which tasks can run in parallel:

```rust
fn calculate_parallel_levels(dag: &DependencyGraph) -> HashMap<String, u32> {
    let levels = dag.calculate_execution_levels();
    
    // Tasks at level 0: no dependencies, can run immediately in parallel
    // Tasks at level 1: depend on level 0 tasks
    // Tasks at level 2: depend on level 1 tasks, etc.
    
    levels
}

// Get tasks at a specific level
let level_0_tasks = dag.get_tasks_at_level(0);
// All tasks in level_0_tasks can run in parallel
```

## Common Patterns

### Linear Dependencies

Simple sequential execution:

```
T1 → T2 → T3
```

- **Topological Sort**: `["T1", "T2", "T3"]`
- **Execution Levels**: T1=0, T2=1, T3=2
- **Parallelism**: None (sequential execution)

### Diamond Dependencies

Parallel branches that converge:

```
    T1
   / \
  T2  T3
   \ /
    T4
```

- **Topological Sort**: `["T1", "T2", "T3", "T4"]` (T2 and T3 can be in any order)
- **Execution Levels**: T1=0, T2=1, T3=1, T4=2
- **Parallelism**: T2 and T3 can run in parallel after T1 completes

### Complex Dependencies

Multiple levels with mixed dependencies:

```
T1 → T2 → T4
T1 → T3 → T4
T1 → T5
```

- **Topological Sort**: `["T1", "T2", "T3", "T5", "T4"]` (T2, T3, T5 can be in any order)
- **Execution Levels**: T1=0, T2=1, T3=1, T5=1, T4=2
- **Parallelism**: T2, T3, and T5 can run in parallel after T1

## Error Handling

### Error Types

- **CycleDetected**: Circular dependency found (includes cycle path)
- **DependencyNotFound**: Referenced dependency doesn't exist
- **InvalidTaskId**: Task ID has invalid format (should be "I[number].T[number]")
- **TopologicalSortFailed**: Sort failed (should not happen if cycle detection passed)

### Handling Errors

```rust
use radium_core::planning::dag::{DependencyGraph, DagError};

match DependencyGraph::from_manifest(&manifest) {
    Ok(dag) => {
        // Graph is valid, proceed with execution
    }
    Err(DagError::CycleDetected(path)) => {
        println!("Fix cycle: {}", path);
        // Remove or reorder dependencies to break the cycle
    }
    Err(DagError::DependencyNotFound(dep_id)) => {
        println!("Missing dependency: {}", dep_id);
        // Add the missing dependency or fix the reference
    }
    Err(DagError::InvalidTaskId(id)) => {
        println!("Invalid task ID format: {}", id);
        // Fix task ID to match "I[number].T[number]" format
    }
    Err(e) => {
        println!("Unexpected error: {}", e);
    }
}
```

## Cycle Detection Algorithm

The system uses DFS (Depth-First Search) to detect cycles:

1. **Visit each node**: Start DFS from each unvisited node
2. **Track recursion stack**: Maintain a stack of nodes in current path
3. **Detect back edges**: If we encounter a node in the recursion stack, we found a cycle
4. **Report path**: Build the cycle path for error reporting

### Example: Cycle Detection

```
Graph: T1 → T2 → T3 → T1

DFS from T1:
  T1 (add to stack)
    → T2 (add to stack)
      → T3 (add to stack)
        → T1 (already in stack - CYCLE!)
        
Cycle path: T1 → T2 → T3 → T1
```

## Topological Sort Algorithm

Uses Kahn's algorithm (via petgraph):

1. **Find nodes with no incoming edges**: These can start immediately
2. **Remove nodes and edges**: As nodes are processed, remove their outgoing edges
3. **Repeat**: Continue until all nodes are processed
4. **Result**: Nodes in dependency order

### Example: Topological Sort

```
Graph: T1 → T2 → T4
       T1 → T3 → T4

Step 1: T1 has no dependencies → add T1
Step 2: Remove T1, T2 and T3 have no dependencies → add T2, T3
Step 3: Remove T2, T3, T4 has no dependencies → add T4
Result: [T1, T2, T3, T4]
```

## Execution Level Calculation

Uses BFS-like approach to assign levels:

1. **Level 0**: Tasks with no incoming edges (no dependencies)
2. **Level N+1**: Tasks that depend on level N tasks
3. **Parallel Execution**: All tasks at the same level can run concurrently

### Example: Execution Levels

```
Graph: T1 → T2 → T4
       T1 → T3 → T4

Level 0: [T1]        (no dependencies)
Level 1: [T2, T3]    (depend on T1, can run in parallel)
Level 2: [T4]        (depends on T2 and T3)
```

## Performance Characteristics

- **Construction**: O(V + E) where V = vertices (tasks), E = edges (dependencies)
- **Cycle Detection**: O(V + E) using DFS
- **Topological Sort**: O(V + E) using Kahn's algorithm
- **Execution Levels**: O(V + E) using BFS-like traversal

For typical plans (10-50 tasks, 20-100 dependencies), all operations are very fast (&lt;1ms).

## Best Practices

1. **Design Dependencies Carefully**: Avoid cycles by planning task order
2. **Validate Early**: Check for cycles before execution starts
3. **Use Execution Levels**: Leverage parallel execution for independent tasks
4. **Handle Errors Gracefully**: Provide clear error messages for cycle resolution
5. **Test Edge Cases**: Test with empty graphs, single nodes, disconnected components

## Integration Points

- **Autonomous Planning**: Uses DAG for plan validation
- **Plan Executor**: Uses DAG for execution ordering
- **Workflow Generator**: Uses topological sort for step ordering
- **Parallel Execution**: Uses execution levels for scheduling

## Related Features

- [Autonomous Planning](./autonomous-planning.md) - Uses DAG for validation
- [Plan Execution](./plan-execution.md) - Uses DAG for execution order
- [Workflow Templates](../user-guide/orchestration.md) - Uses topological sort

## See Also

- [API Reference](../../crates/radium-core/src/planning/dag.rs) - Complete API documentation
- [Petgraph Documentation](https://docs.rs/petgraph/) - Underlying graph library
- [Examples](../examples/) - Usage examples

