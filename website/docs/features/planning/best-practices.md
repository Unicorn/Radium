---
id: "best-practices"
title: "Planning Best Practices"
sidebar_label: "Planning Best Practices"
---

# Planning Best Practices

This guide covers best practices for creating, validating, and executing plans with Radium's autonomous planning system.

## Plan Creation

### Start Simple

Begin with a minimal plan and iterate:

```markdown
# My Project

Build a simple feature.

## Iteration 1: Core Feature

1. **Task 1** - Implement core logic
   - Agent: code-agent
   - Dependencies: 
   - Acceptance Criteria:
     - Feature works as specified
```

### Use Clear Dependencies

Explicitly state task dependencies:

```markdown
1. **Task 1** - Setup project
   - Dependencies: 

2. **Task 2** - Implement feature
   - Dependencies: I1.T1  # Explicitly depends on Task 1
```

### Assign Appropriate Agents

Choose agents that match the task:

```markdown
1. **Task 1** - Write code
   - Agent: code-agent  # For implementation

2. **Task 2** - Review code
   - Agent: review-agent  # For code review
```

### Define Clear Acceptance Criteria

Specific criteria help validation and execution:

```markdown
1. **Task 1** - Implement API endpoint
   - Acceptance Criteria:
     - Endpoint responds to GET /api/users
     - Returns JSON with user data
     - Status code 200 on success
```

## Plan Validation

### Fix Validation Errors Early

Address validation errors before execution:

```bash
# Generate plan
rad plan spec.md

# If validation fails:
# → Fix errors in specification
# → Regenerate plan
rad plan spec.md
```

### Avoid Circular Dependencies

Circular dependencies prevent execution:

```markdown
# ❌ Bad: Circular dependency
Task 1 depends on Task 2
Task 2 depends on Task 1

# ✅ Good: Linear dependency
Task 1 (no dependencies)
Task 2 depends on Task 1
Task 3 depends on Task 2
```

### Use Valid Task References

Always use correct task ID format:

```markdown
# ✅ Good: Valid reference
Dependencies: I1.T1

# ❌ Bad: Invalid reference
Dependencies: Task1
Dependencies: I5.T1  # I5 doesn't exist
```

## Plan Execution

### Start with Bounded Mode

Use bounded mode for initial testing:

```bash
# Test first 5 iterations
rad craft

# Review results
# If good, continue with more iterations
rad craft --resume
```

### Monitor Progress

Watch execution progress for issues:

```bash
rad craft

# Watch for:
# - Error messages
# - Token usage
# - Progress percentage
```

### Use Graceful Shutdown

Stop execution cleanly when needed:

```bash
# Press Ctrl+C to abort
# Progress is saved automatically
rad craft --resume  # Continue later
```

## Error Handling

### Let Retries Handle Transient Errors

Automatic retries handle most recoverable errors:

```bash
# Rate limit error → automatic retry
# Network error → automatic retry
# No action needed
```

### Fix Fatal Errors Immediately

Address fatal errors before continuing:

```bash
# Authentication error → fix credentials
rad auth login

# Configuration error → fix config
# Then resume execution
rad craft --resume
```

### Review Error Messages

Error messages provide actionable guidance:

```bash
# Error: 401 unauthorized
# Suggestion: Run 'rad auth login' to authenticate
```

## Cost Management

### Monitor Token Usage

Track token consumption:

```bash
# Check token usage
rad stats

# Output:
# Total Tokens: 45,000
# Total Cost: $0.45
```

### Use Appropriate Models

Choose models based on task complexity:

```bash
# Simple tasks → cheaper models
rad craft --engine claude --model claude-3-haiku

# Complex tasks → more capable models
rad craft --engine claude --model claude-3-opus
```

### Set Budget Limits

Control spending with budgets:

```bash
# Set daily budget
rad budget set --daily 10.00

# Monitor budget
rad budget status
```

## Context and Memory

### Leverage Context Files

Use context files for project guidelines:

```bash
# Create GEMINI.md with project context
echo "# Project Guidelines" > GEMINI.md
echo "Always write tests." >> GEMINI.md

# Context is automatically included in agent prompts
rad craft
```

### Use Memory Store

Agent outputs are automatically stored for future context:

```bash
# First agent execution
rad craft

# Later agent can access previous outputs
# Memory is automatically included in context
```

## Troubleshooting

### Plan Generation Issues

**Problem**: Plan generation fails

**Solutions**:
- Check specification format
- Verify workspace is initialized
- Ensure AI model access

### Validation Failures

**Problem**: Validation always fails

**Solutions**:
- Review validation error messages
- Check for circular dependencies
- Verify task ID formats
- Ensure agents exist

### Execution Errors

**Problem**: Execution fails repeatedly

**Solutions**:
- Check error categories (recoverable vs fatal)
- Verify authentication credentials
- Review configuration files
- Check network connectivity

### High Costs

**Problem**: Token usage is high

**Solutions**:
- Use cheaper models for simple tasks
- Optimize prompts to reduce token usage
- Set budget limits
- Review session analytics

## Common Patterns

### Incremental Development

```bash
# 1. Create minimal plan
rad plan "Build feature X"

# 2. Execute first iteration
rad craft

# 3. Review and refine
# 4. Add more iterations
# 5. Execute again
rad craft --resume
```

### Full Automation

```bash
# 1. Create complete plan
rad plan spec.md

# 2. Execute all iterations
rad craft --yolo

# 3. Review results
```

### Error Recovery

```bash
# 1. Execution fails
rad craft
# → Error: network connection failed

# 2. Fix issue (e.g., network connectivity)

# 3. Resume execution
rad craft --resume
```

## See Also

- [Autonomous Planning](./autonomous-planning.md) - Plan generation
- [Execution Modes](./execution-modes.md) - Execution configuration
- [Error Handling](./error-handling.md) - Error management
- [Monitoring Integration](./monitoring-integration.md) - Cost tracking

