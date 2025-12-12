---
id: "workflow-behaviors"
title: "Workflow Behaviors"
sidebar_label: "Workflow Behaviors"
---

# Workflow Behaviors

Workflow behaviors provide dynamic execution control for Radium workflows, allowing agents to adapt to changing conditions, handle errors gracefully, and coordinate with other agents dynamically.

## Overview

Workflow behaviors enable agents to control workflow execution through a `behavior.json` file. This file can be written by any agent during workflow execution to request specific behaviors like looping back to previous steps, triggering other agents, pausing for manual review, or requesting oversight.

## How It Works

1. **Agent Execution**: An agent executes a workflow step and determines that a behavior is needed
2. **Behavior File**: The agent writes a `behavior.json` file to `.radium/memory/behavior.json`
3. **File Detection**: The workflow engine checks for the behavior file after step completion
4. **Behavior Parsing**: The JSON file is parsed into a `BehaviorAction` enum
5. **Evaluator Selection**: The appropriate evaluator is selected based on action type
6. **Decision Evaluation**: The evaluator processes the action and returns a decision
7. **Workflow Control**: The engine applies the decision (loop, trigger, checkpoint, etc.)

## Behavior Types

### Loop Behavior

Allows agents to request repeating previous steps with configurable limits.

**Use Cases:**
- Retry failed steps with modifications
- Iterative refinement of implementation
- Re-run tests after fixes

**Configuration:**
- `max_iterations`: Maximum number of loop iterations (optional)
- `steps`: Number of steps to go back
- `skip`: List of step IDs to exclude from the loop

**Example:**
```json
{
  "action": "loop",
  "reason": "Tests are failing, need to fix implementation"
}
```

**Configuration Example:**
```json
{
  "steps": 2,
  "maxIterations": 3,
  "skip": ["step-1"]
}
```

### Trigger Behavior

Allows agents to dynamically trigger other agents during workflow execution.

**Use Cases:**
- Trigger review agent after code changes
- Conditional security scan trigger
- Dynamic workflow orchestration

**Configuration:**
- `triggerAgentId`: Agent ID to trigger (required in behavior.json or module config)

**Example:**
```json
{
  "action": "trigger",
  "triggerAgentId": "review-agent",
  "reason": "Need code review before proceeding"
}
```

### Checkpoint Behavior

Allows agents to pause workflow execution for manual intervention.

**Use Cases:**
- Approval gates before dangerous operations
- Manual review checkpoints
- State snapshots before risky operations

**Example:**
```json
{
  "action": "checkpoint",
  "reason": "Need user approval for database migration"
}
```

### VibeCheck Behavior

Allows agents to request metacognitive oversight to prevent reasoning lock-in.

**Use Cases:**
- Request oversight when uncertain about approach
- Phase-aware interrupts (planning/implementation/review)
- Risk assessment before proceeding

**Example:**
```json
{
  "action": "vibecheck",
  "reason": "Uncertain about approach, need oversight"
}
```

## Behavior Action Format

The `behavior.json` file must be placed at `.radium/memory/behavior.json` and follow this format:

```json
{
  "action": "loop" | "trigger" | "checkpoint" | "continue" | "stop" | "vibecheck",
  "reason": "Why this action was chosen",
  "triggerAgentId": "agent-to-trigger"
}
```

### Action Types

- **`loop`**: Repeat previous steps (requires loop configuration)
- **`trigger`**: Trigger another agent dynamically (requires triggerAgentId)
- **`checkpoint`**: Pause workflow execution for manual intervention
- **`continue`**: Continue normal execution (default if no behavior file)
- **`stop`**: Stop the current loop (used within loop behavior)
- **`vibecheck`**: Request metacognitive oversight

## Behavior Evaluation Flow

```
┌─────────────────┐
│ Agent Executes  │
│  Workflow Step  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Agent Writes    │
│ behavior.json   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Workflow Engine │
│ Detects File    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Parse JSON      │
│ BehaviorAction  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Select          │
│ Evaluator       │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Evaluate        │
│ Decision        │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Apply Behavior  │
│ (loop/trigger/  │
│ checkpoint/etc) │
└─────────────────┘
```

## Integration with WorkflowExecutor

Behaviors are automatically integrated with the `WorkflowExecutor`. The executor checks for `behavior.json` files after each step execution and evaluates them using the appropriate behavior evaluators.

## Error Handling

- **Invalid JSON**: Errors are logged, workflow continues with default behavior
- **Missing behavior file**: No special behavior, normal execution continues
- **Missing configuration**: Behavior evaluator returns `None`, no behavior triggered
- **Invalid configuration**: BehaviorError is returned, logged, and workflow continues

## Best Practices

1. **Use clear reasons**: Always include a `reason` field explaining why the behavior was chosen
2. **Set max iterations**: For loop behavior, always set `max_iterations` to prevent infinite loops
3. **Use checkpoints sparingly**: Checkpoints pause workflow execution - use only when necessary
4. **Trigger agents carefully**: Ensure the target agent exists and is available before triggering
5. **Test behaviors**: Verify behavior.json format before committing to workflow

## Troubleshooting

### Behavior Not Detected

- Verify `behavior.json` is at `.radium/memory/behavior.json`
- Check JSON syntax is valid
- Ensure action type matches expected values (lowercase)

### Loop Not Executing

- Verify loop configuration exists in module/workflow config
- Check `max_iterations` hasn't been reached
- Ensure behavior.json contains `"action": "loop"`

### Trigger Not Working

- Verify `triggerAgentId` is provided (in behavior.json or config)
- Check that the target agent exists and is registered
- Ensure agent ID matches exactly (case-sensitive)

### Checkpoint Not Pausing

- Verify behavior.json contains `"action": "checkpoint"`
- Check that checkpoint evaluator is enabled
- Review workflow executor logs for errors

## See Also

- [Policy Engine](./policy-engine.md) - Tool execution control
- [Constitution System](./constitution-system.md) - Session-based rules
- [Workflow Templates](../user-guide/workflow-templates.md) - Reusable workflows
- [Behavior Examples](../../examples/behaviors/) - Example behavior.json files

