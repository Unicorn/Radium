//! Example workflows for testing and documentation.
//!
//! This module provides example workflow definitions that demonstrate
//! different workflow patterns and can be used for testing.

use radium_core::models::{Workflow, WorkflowStep};

/// Creates a simple sequential workflow with 3 steps.
///
/// This workflow demonstrates basic sequential execution where each
/// step executes in order.
pub fn create_sequential_workflow() -> Workflow {
    let mut workflow = Workflow::new(
        "example-sequential".to_string(),
        "Sequential Workflow Example".to_string(),
        "A simple workflow that executes steps sequentially".to_string(),
    );

    workflow
        .add_step(WorkflowStep::new(
            "step-1".to_string(),
            "Step 1".to_string(),
            "First step in sequence".to_string(),
            "task-1".to_string(),
            0,
        ))
        .unwrap();

    workflow
        .add_step(WorkflowStep::new(
            "step-2".to_string(),
            "Step 2".to_string(),
            "Second step in sequence".to_string(),
            "task-2".to_string(),
            1,
        ))
        .unwrap();

    workflow
        .add_step(WorkflowStep::new(
            "step-3".to_string(),
            "Step 3".to_string(),
            "Third step in sequence".to_string(),
            "task-3".to_string(),
            2,
        ))
        .unwrap();

    workflow
}

/// Creates a workflow with conditional branching.
///
/// This workflow demonstrates conditional step execution based on
/// previous step results.
pub fn create_conditional_workflow() -> Workflow {
    let mut workflow = Workflow::new(
        "example-conditional".to_string(),
        "Conditional Workflow Example".to_string(),
        "A workflow with conditional step execution".to_string(),
    );

    // First step always executes
    workflow
        .add_step(WorkflowStep::new(
            "step-1".to_string(),
            "Step 1".to_string(),
            "Initial step".to_string(),
            "task-1".to_string(),
            0,
        ))
        .unwrap();

    // Second step executes only if step-1 succeeds
    let mut step2 = WorkflowStep::new(
        "step-2".to_string(),
        "Step 2".to_string(),
        "Conditional step".to_string(),
        "task-2".to_string(),
        1,
    );
    step2.config_json = Some(
        r#"{"condition": "step-1.result.success == true", "depends_on": ["step-1"]}"#.to_string(),
    );
    workflow.add_step(step2).unwrap();

    workflow
}

/// Creates a workflow with parallel step execution.
///
/// This workflow demonstrates parallel execution where steps with
/// the same order value execute concurrently.
pub fn create_parallel_workflow() -> Workflow {
    let mut workflow = Workflow::new(
        "example-parallel".to_string(),
        "Parallel Workflow Example".to_string(),
        "A workflow with parallel step execution".to_string(),
    );

    // First step
    workflow
        .add_step(WorkflowStep::new(
            "step-1".to_string(),
            "Step 1".to_string(),
            "Initial step".to_string(),
            "task-1".to_string(),
            0,
        ))
        .unwrap();

    // Steps 2 and 3 execute in parallel (same order)
    workflow
        .add_step(WorkflowStep::new(
            "step-2".to_string(),
            "Step 2".to_string(),
            "Parallel step A".to_string(),
            "task-2".to_string(),
            1,
        ))
        .unwrap();

    workflow
        .add_step(WorkflowStep::new(
            "step-3".to_string(),
            "Step 3".to_string(),
            "Parallel step B".to_string(),
            "task-3".to_string(),
            1, // Same order as step-2 for parallel execution
        ))
        .unwrap();

    // Final step after parallel steps complete
    workflow
        .add_step(WorkflowStep::new(
            "step-4".to_string(),
            "Step 4".to_string(),
            "Final step".to_string(),
            "task-4".to_string(),
            2,
        ))
        .unwrap();

    workflow
}

/// Creates a complex workflow combining sequential, conditional, and parallel execution.
///
/// This workflow demonstrates a realistic scenario with multiple execution patterns.
pub fn create_complex_workflow() -> Workflow {
    let mut workflow = Workflow::new(
        "example-complex".to_string(),
        "Complex Workflow Example".to_string(),
        "A complex workflow combining multiple execution patterns".to_string(),
    );

    // Initial sequential steps
    workflow
        .add_step(WorkflowStep::new(
            "init".to_string(),
            "Initialize".to_string(),
            "Initial setup step".to_string(),
            "task-init".to_string(),
            0,
        ))
        .unwrap();

    // Parallel processing
    workflow
        .add_step(WorkflowStep::new(
            "process-a".to_string(),
            "Process A".to_string(),
            "Parallel process A".to_string(),
            "task-a".to_string(),
            1,
        ))
        .unwrap();

    workflow
        .add_step(WorkflowStep::new(
            "process-b".to_string(),
            "Process B".to_string(),
            "Parallel process B".to_string(),
            "task-b".to_string(),
            1, // Same order for parallel
        ))
        .unwrap();

    // Conditional step based on parallel results
    let mut conditional = WorkflowStep::new(
        "validate".to_string(),
        "Validate".to_string(),
        "Validation step".to_string(),
        "task-validate".to_string(),
        2,
    );
    conditional.config_json = Some(
        r#"{"condition": "process-a.result.success == true && process-b.result.success == true", "depends_on": ["process-a", "process-b"]}"#
            .to_string(),
    );
    workflow.add_step(conditional).unwrap();

    // Final step
    workflow
        .add_step(WorkflowStep::new(
            "finalize".to_string(),
            "Finalize".to_string(),
            "Final cleanup step".to_string(),
            "task-finalize".to_string(),
            3,
        ))
        .unwrap();

    workflow
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_sequential_workflow() {
        let workflow = create_sequential_workflow();
        assert_eq!(workflow.steps.len(), 3);
        assert_eq!(workflow.steps[0].order, 0);
        assert_eq!(workflow.steps[1].order, 1);
        assert_eq!(workflow.steps[2].order, 2);
    }

    #[test]
    fn test_create_conditional_workflow() {
        let workflow = create_conditional_workflow();
        assert_eq!(workflow.steps.len(), 2);
        assert!(workflow.steps[1].config_json.is_some());
    }

    #[test]
    fn test_create_parallel_workflow() {
        let workflow = create_parallel_workflow();
        assert_eq!(workflow.steps.len(), 4);
        // Steps 2 and 3 should have same order for parallel execution
        assert_eq!(workflow.steps[1].order, 1);
        assert_eq!(workflow.steps[2].order, 1);
    }

    #[test]
    fn test_create_complex_workflow() {
        let workflow = create_complex_workflow();
        assert_eq!(workflow.steps.len(), 5);
        // Verify parallel steps
        assert_eq!(workflow.steps[1].order, 1);
        assert_eq!(workflow.steps[2].order, 1);
    }
}
