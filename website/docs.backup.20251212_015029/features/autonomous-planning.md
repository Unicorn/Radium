# Autonomous Planning System

The autonomous planning system converts high-level goals into structured, executable workflows with automatic validation and dependency analysis. It uses AI to decompose goals into iterations and tasks, validates the structure, and generates workflow templates ready for execution.

## Overview

Autonomous planning provides an intelligent way to break down complex goals into actionable plans:

- **AI-Powered Decomposition**: Uses language models to convert goals into structured plans
- **Automatic Validation**: Multi-stage validation with intelligent retry logic
- **Dependency Analysis**: Builds dependency graphs to detect cycles and ensure correct ordering
- **Workflow Generation**: Creates executable workflow templates from validated plans

### Key Features

- **Goal-to-Workflow Pipeline**: Single API call converts goals to executable workflows
- **Validation Retry Logic**: Automatically retries plan generation on validation failures (max 2 retries)
- **Cycle Detection**: Identifies circular dependencies before execution
- **Agent Assignment**: Validates agent assignments and handles unknown agents gracefully
- **Dependency Validation**: Ensures all task dependencies exist and are valid

## How It Works

### Pipeline Flow

The autonomous planning system follows a structured pipeline:

```
Goal → PlanGenerator → ParsedPlan
                          ↓
                    PlanValidator (with retry)
                          ↓
                    DependencyGraph (DAG)
                          ↓
                    WorkflowGenerator
                          ↓
                    AutonomousPlan (complete)
```

### Step-by-Step Process

1. **Plan Generation**: The `PlanGenerator` uses an AI model to decompose the goal into a structured plan with:
   - Project name and description
   - Tech stack detection
   - Iterations (typically 3-5)
   - Tasks within each iteration
   - Dependencies between tasks
   - Agent assignments

2. **Plan Validation**: The `PlanValidator` performs multi-stage validation:
   - **Dependency Graph Validation**: Builds a DAG and checks for cycles
   - **Agent Validation**: Verifies agent IDs exist in the registry
   - **Dependency References**: Ensures all task dependencies exist

3. **Retry Logic**: If validation fails:
   - Validation errors are fed back to the generator
   - The plan is regenerated with error context
   - The process repeats up to 2 times
   - If validation still fails after retries, an error is returned

4. **Dependency Analysis**: A `DependencyGraph` is built to:
   - Detect circular dependencies
   - Calculate execution order (topological sort)
   - Determine parallel execution levels

5. **Workflow Generation**: The `WorkflowGenerator` creates an executable workflow:
   - Uses topological sort to determine step order
   - Creates workflow steps for each task
   - Preserves agent assignments and metadata

## API Usage

### Basic Example

```rust
use radium_core::planning::autonomous::AutonomousPlanner;
use radium_core::agents::registry::AgentRegistry;
use radium_abstraction::Model;
use std::sync::Arc;

async fn create_plan() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize components
    let agent_registry = Arc::new(AgentRegistry::new());
    let planner = AutonomousPlanner::new(agent_registry);
    let model: Arc<dyn Model> = /* ... */;

    // Generate plan from goal
    let goal = "Build a REST API with authentication and user management";
    let autonomous_plan = planner.plan_from_goal(goal, model).await?;

    // Access generated components
    println!("Project: {}", autonomous_plan.plan.project_name);
    println!("Iterations: {}", autonomous_plan.plan.iterations.len());
    println!("Workflow steps: {}", autonomous_plan.workflow.steps().len());

    // Use the workflow for execution
    // Use the DAG for dependency analysis
    // Use the manifest for progress tracking

    Ok(())
}
```

### Validation Report

The validation system provides detailed feedback:

```rust
use radium_core::planning::autonomous::{PlanValidator, ValidationReport};
use radium_core::agents::registry::AgentRegistry;
use std::sync::Arc;

fn validate_plan(plan: &ParsedPlan) -> ValidationReport {
    let registry = Arc::new(AgentRegistry::new());
    let validator = PlanValidator::new(registry);
    validator.validate_plan(plan)
}

// Check validation results
let report = validate_plan(&plan);
if !report.is_valid {
    println!("Errors: {:?}", report.errors);
    // Errors must be fixed (e.g., circular dependencies, missing references)
}
if !report.warnings.is_empty() {
    println!("Warnings: {:?}", report.warnings);
    // Warnings don't block execution (e.g., unknown agents)
}
```

## Validation Retry Logic

The system includes intelligent retry logic to handle validation failures:

### Retry Process

1. **Initial Generation**: Plan is generated from goal
2. **First Validation**: Plan is validated
3. **On Failure**: If validation fails:
   - Errors are collected into feedback
   - Goal + errors are sent back to generator
   - New plan is generated with error context
4. **Re-validation**: New plan is validated
5. **Max Retries**: Process repeats up to 2 times
6. **Final Failure**: If still invalid after retries, error is returned

### Error Categories

- **Errors**: Must be fixed (circular dependencies, missing references, invalid formats)
- **Warnings**: Don't block execution (unknown agents, missing optional fields)

### Example: Retry Flow

```
Goal: "Build API"
  ↓
Plan Generated (attempt 1)
  ↓
Validation: FAILED (circular dependency detected)
  ↓
Feedback: "Goal + Validation errors"
  ↓
Plan Regenerated (attempt 2)
  ↓
Validation: SUCCESS
  ↓
AutonomousPlan returned
```

## Dependency Analysis

The system builds a dependency graph (DAG) to:

### Cycle Detection

Detects circular dependencies before execution:

```rust
use radium_core::planning::dag::DependencyGraph;

let dag = DependencyGraph::from_manifest(&manifest)?;

// Check for cycles
if let Err(e) = dag.detect_cycles() {
    println!("Cycle detected: {}", e);
    // Cycle path is included in error message
}
```

### Execution Ordering

Uses topological sort to determine correct execution order:

```rust
let execution_order = dag.topological_sort()?;
// Returns tasks in dependency order
// Tasks with no dependencies come first
// Dependent tasks come after their dependencies
```

### Parallel Execution Levels

Calculates execution levels for parallel scheduling:

```rust
let levels = dag.calculate_execution_levels();
// Tasks at level 0 can run in parallel
// Tasks at level N+1 depend on tasks at level N
```

## Workflow Generation

The `WorkflowGenerator` converts validated plans into executable workflows:

### Process

1. **Topological Sort**: Gets tasks in dependency order from DAG
2. **Step Creation**: Creates workflow steps for each task
3. **Agent Assignment**: Preserves agent assignments from plan
4. **Metadata Preservation**: Includes task descriptions and acceptance criteria

### Example

```rust
use radium_core::planning::autonomous::WorkflowGenerator;
use radium_core::planning::dag::DependencyGraph;

let generator = WorkflowGenerator::new();
let dag = DependencyGraph::from_manifest(&manifest)?;
let workflow = generator.generate_workflow(&plan, &dag)?;

// Workflow is ready for execution
// Steps are in correct dependency order
// Agent assignments are preserved
```

## Common Use Cases

### 1. Goal-to-Workflow Conversion

Convert a high-level goal directly to an executable workflow:

```rust
let goal = "Build a microservices architecture with API gateway";
let autonomous_plan = planner.plan_from_goal(goal, model).await?;
let workflow = autonomous_plan.workflow;
// Execute workflow immediately
```

### 2. Plan Validation Only

Validate an existing plan without generating a workflow:

```rust
let validator = PlanValidator::new(agent_registry);
let report = validator.validate_plan(&plan);
if report.is_valid {
    // Plan is ready for execution
} else {
    // Fix errors before proceeding
}
```

### 3. Dependency Analysis

Analyze task dependencies without full planning:

```rust
let dag = DependencyGraph::from_manifest(&manifest)?;
let order = dag.topological_sort()?;
let levels = dag.calculate_execution_levels();
// Use for execution scheduling
```

## Error Handling

### Planning Errors

- `GenerationFailed`: AI model failed to generate plan
- `ValidationFailed`: Plan failed validation after max retries
- `Dag`: Dependency graph errors (cycles, missing dependencies)
- `WorkflowGenerationFailed`: Failed to generate workflow template
- `AgentNotFound`: Referenced agent doesn't exist

### Handling Errors

```rust
match planner.plan_from_goal(goal, model).await {
    Ok(plan) => {
        // Use plan
    }
    Err(PlanningError::ValidationFailed(msg)) => {
        // Plan had validation errors after retries
        // Consider manual intervention
    }
    Err(PlanningError::GenerationFailed(msg)) => {
        // Model execution failed
        // Check model configuration
    }
    Err(e) => {
        // Other errors
    }
}
```

## Best Practices

1. **Clear Goals**: Provide specific, actionable goals for better plan generation
2. **Agent Registry**: Ensure all referenced agents exist in the registry
3. **Dependency Design**: Design task dependencies carefully to avoid cycles
4. **Validation Monitoring**: Check validation reports to understand plan quality
5. **Retry Handling**: Be aware that retries may change the plan structure

## Integration Points

- **Plan Generator**: Uses AI models for plan generation
- **Agent Registry**: Validates agent assignments
- **DAG System**: Provides dependency analysis
- **Workflow System**: Generates executable workflow templates
- **Plan Executor**: Executes generated workflows

## Related Features

- [DAG Dependency Management](./dag-dependencies.md) - Dependency graph system
- [Plan Execution](./plan-execution.md) - Executing generated plans
- [Agent System](../developer-guide/agent-system-architecture.md) - Agent configuration
- [Workflow Templates](../user-guide/orchestration.md) - Workflow execution

## See Also

- [API Reference](../../crates/radium-core/src/planning/autonomous.rs) - Complete API documentation
- [CLI Commands](../cli/commands/plan-execution.md) - Command-line usage
- [Examples](../examples/) - Usage examples

