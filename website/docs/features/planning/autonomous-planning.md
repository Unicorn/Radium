---
id: "autonomous-planning"
title: "Autonomous Planning"
sidebar_label: "Autonomous Planning"
---

# Autonomous Planning

Autonomous planning is Radium's AI-powered system for generating executable plans from natural language specifications. It automatically structures your requirements into iterations and tasks, validates dependencies, and creates execution workflows.

## Overview

The autonomous planning pipeline consists of four main stages:

1. **Plan Generation**: AI generates a structured plan from your specification
2. **Validation**: Multi-stage validation ensures plan correctness
3. **Dependency Analysis**: DAG construction and cycle detection
4. **Workflow Generation**: Creates executable workflow from validated plan

## Plan Generation

Generate a plan from a specification using the `rad plan` command:

```bash
# Generate plan from direct input
rad plan "Build a REST API with authentication"

# Generate plan from file
rad plan spec.md

# Generate plan with custom ID
rad plan --id REQ-123 spec.md
```

The plan generator uses AI to extract:
- Project name and description
- Tech stack requirements
- Iterations with goals
- Tasks with dependencies
- Agent assignments
- Acceptance criteria

## Validation Pipeline

The autonomous planner performs multi-stage validation:

### Stage 1: Dependency Graph Validation

Validates that all task dependencies exist and detects circular dependencies:

```bash
# Plan with valid dependencies
rad plan "Task 1 depends on nothing, Task 2 depends on Task 1"

# Plan with circular dependency (will fail validation)
rad plan "Task 1 depends on Task 2, Task 2 depends on Task 1"
```

### Stage 2: Agent Assignment Validation

Verifies that assigned agents exist in the agent registry:

```bash
# Valid agent assignment
rad plan "Use code-agent to implement feature"

# Unknown agent (warning, not error)
rad plan "Use unknown-agent to implement feature"
```

### Stage 3: Dependency Reference Validation

Ensures all dependency references use valid task ID format (`I[number].T[number]`):

```bash
# Valid dependency reference
rad plan "Task 2 depends on I1.T1"

# Invalid dependency reference (will fail)
rad plan "Task 2 depends on I5.T1"  # I5 doesn't exist
```

## Validation Retry Logic

If validation fails, the planner automatically retries up to 2 times with validation feedback:

1. **Initial Generation**: AI generates plan from specification
2. **Validation Check**: Plan is validated for correctness
3. **Retry with Feedback**: If validation fails, feedback is provided to AI for regeneration
4. **Final Validation**: Regenerated plan is validated again

This retry mechanism helps the AI correct common issues like:
- Missing dependencies
- Invalid task references
- Circular dependencies

## Example: Complete Planning Workflow

```bash
# 1. Create specification file
cat > my-spec.md << EOF
# My Project

Build a web application with user authentication.

## Iteration 1: Setup

1. **Task 1** - Setup project structure
   - Agent: code-agent
   - Dependencies: 
   - Acceptance Criteria:
     - Project structure created
     - Dependencies installed

2. **Task 2** - Implement authentication
   - Agent: code-agent
   - Dependencies: I1.T1
   - Acceptance Criteria:
     - Login endpoint working
     - JWT tokens generated
EOF

# 2. Generate plan
rad plan my-spec.md

# 3. Plan is validated automatically
# 4. If validation fails, retry with feedback
# 5. Validated plan is saved to .radium/plan/REQ-XXX/
```

## Validation Errors

Common validation errors and how to fix them:

### Circular Dependency

**Error**: `Circular dependency detected: I1.T1 -> I1.T2 -> I1.T3 -> I1.T1`

**Fix**: Remove or reorder dependencies to break the cycle.

### Missing Dependency

**Error**: `Task I1.T2 references non-existent dependency: I5.T1`

**Fix**: Ensure the referenced task exists in the plan, or remove the invalid dependency.

### Invalid Task ID Format

**Error**: `Invalid task ID format: INVALID`

**Fix**: Use format `I[number].T[number]` where number is the iteration/task number.

## Best Practices

1. **Start Simple**: Begin with a single iteration and few tasks
2. **Use Clear Dependencies**: Explicitly state task dependencies
3. **Assign Agents**: Specify which agent should handle each task
4. **Define Acceptance Criteria**: Clear criteria help validation and execution
5. **Iterate**: Use validation feedback to refine your plan

## Troubleshooting

### Plan Generation Fails

- Check that your specification is clear and well-formatted
- Ensure you have a valid workspace (`rad init` if needed)
- Verify AI model access and credentials

### Validation Always Fails

- Review validation error messages carefully
- Check for circular dependencies
- Verify all dependency references use correct format
- Ensure assigned agents exist in your workspace

### Retry Logic Not Working

- Validation retries are automatic (up to 2 retries)
- If validation fails after retries, fix the specification manually
- Check that your AI model supports the planning prompt format

## See Also

- [DAG Dependencies](./dag-dependencies.md) - Understanding dependency graphs
- [Execution Modes](./execution-modes.md) - Running plans
- [Error Handling](./error-handling.md) - Handling execution errors

