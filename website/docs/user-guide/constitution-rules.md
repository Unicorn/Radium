---
id: "constitution-rules"
title: "Constitution Rules"
sidebar_label: "Constitution Rules"
---

# Constitution Rules

## Overview

Constitution Rules provide per-session rules and constraints for workflow execution. They allow you to set guidelines that agents should follow during a specific session, which are then included in oversight requests and agent prompts.

## What are Constitution Rules?

Constitution Rules are session-scoped rules that:
- Apply only to the current session (identified by session ID)
- Are automatically cleaned up after 1 hour of inactivity (TTL-based)
- Are limited to 50 rules per session
- Are included in oversight requests for context-aware feedback
- Can be used to enforce coding standards, preferences, or constraints

## Usage

### Setting Constitution Rules

Constitution rules are managed programmatically through the `ConstitutionManager` API. In practice, they're typically set:

1. **During workflow execution**: Rules can be set at the start of a workflow
2. **Via oversight integration**: Rules are automatically included in oversight requests
3. **Through session management**: Rules are tied to session IDs

### Example Rules

Common use cases for constitution rules:

```rust
// No external API calls
constitution_manager.update_constitution("session-123", "No external API calls".to_string());

// Prefer unit tests
constitution_manager.update_constitution("session-123", "Prefer unit tests over integration tests".to_string());

// Coding standards
constitution_manager.update_constitution("session-123", "Use Result types for error handling".to_string());

// Performance constraints
constitution_manager.update_constitution("session-123", "Optimize for readability over performance".to_string());
```

## Integration with Vibe Check

Constitution rules are automatically included in vibe check oversight requests:

1. When a vibe check is triggered, the current session's constitution rules are gathered
2. Rules are included in the oversight request context
3. The oversight LLM considers these rules when providing feedback
4. Feedback may reference rule violations or suggest rule-compliant approaches

## Session Management

### Session IDs

Constitution rules are scoped to session IDs:
- Each workflow execution typically has a unique session ID
- Rules set for one session don't affect other sessions
- Session IDs are typically UUIDs or workflow identifiers

### TTL and Cleanup

- Rules are automatically cleaned up after 1 hour of inactivity
- This prevents memory leaks from stale sessions
- Active sessions keep their rules until the session ends

### Rule Limits

- Maximum 50 rules per session
- This prevents context bloat in oversight requests
- If you need more rules, consider consolidating them

## Best Practices

1. **Be specific**: Write clear, actionable rules
2. **Keep it focused**: Limit rules to what's truly important for the session
3. **Use for constraints**: Rules work best for constraints and preferences
4. **Review regularly**: Check that rules are being followed via oversight feedback
5. **Session-specific**: Use different rules for different types of workflows

## Examples

### Example 1: Development Session Rules

```rust
// Set rules for a development session
constitution_manager.update_constitution("dev-session", "No external API calls during development".to_string());
constitution_manager.update_constitution("dev-session", "Use mock data for testing".to_string());
constitution_manager.update_constitution("dev-session", "Follow existing code style".to_string());
```

### Example 2: Code Review Session Rules

```rust
// Set rules for a code review session
constitution_manager.update_constitution("review-session", "Focus on security vulnerabilities".to_string());
constitution_manager.update_constitution("review-session", "Check for performance issues".to_string());
constitution_manager.update_constitution("review-session", "Verify error handling".to_string());
```

## Troubleshooting

### Rules not being applied

- Verify the session ID matches the one used in oversight requests
- Check that rules were set before the oversight request
- Ensure the ConstitutionManager is properly initialized

### Rules disappearing

- Rules are cleaned up after 1 hour of inactivity
- Ensure the session is still active
- Re-set rules if the session has been inactive

### Too many rules

- Limit is 50 rules per session
- Consolidate related rules into single entries
- Remove rules that are no longer needed

## References

- [Vibe Check Documentation](./vibe-check.md)
- [Learning System Documentation](./learning-system.md)
- [Constitution Manager Implementation](../../crates/radium-core/src/policy/constitution.rs)

