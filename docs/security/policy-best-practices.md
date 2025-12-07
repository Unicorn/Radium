# Policy Engine Best Practices

This guide provides security best practices for configuring and using the Radium Policy Engine effectively.

## Principle of Least Privilege

**Always start restrictive and relax as needed.**

1. **Default to Ask mode** - Use `approval_mode = "ask"` for maximum safety
2. **Explicit allow rules** - Only allow operations you explicitly trust
3. **Deny by default** - Use deny rules for operations you never want

```toml
# Good: Explicit allow, deny by default
approval_mode = "ask"

[[rules]]
name = "Allow only specific safe operations"
priority = "user"
action = "allow"
tool_pattern = "read_file"
arg_pattern = "*.md"
```

## Defense in Depth

**Use multiple layers of protection.**

1. **Combine priority levels** - Use Admin rules for critical security, User rules for convenience
2. **Multiple rule types** - Combine tool patterns with argument patterns
3. **Session constitutions** - Add temporary restrictions for specific tasks

```toml
# Layer 1: Admin rules (highest priority)
[[rules]]
name = "Block dangerous operations"
priority = "admin"
action = "deny"
tool_pattern = "run_terminal_cmd"
arg_pattern = "rm -rf *"

# Layer 2: User rules (medium priority)
[[rules]]
name = "Require approval for writes"
priority = "user"
action = "ask_user"
tool_pattern = "write_*"

# Layer 3: Default approval mode
approval_mode = "ask"
```

## Priority System Usage

### Admin Priority

**Use for:**
- Security-critical rules that must never be overridden
- Enterprise policies that apply to all users
- Rules that block dangerous operations

**Examples:**
- Blocking external network access
- Preventing file operations outside workspace
- Denying sudo/root access

```toml
[[rules]]
name = "Enterprise security policy"
priority = "admin"
action = "deny"
tool_pattern = "http_*"
reason = "Enterprise policy: no external network access"
```

### User Priority

**Use for:**
- Workspace-specific policies
- Developer preferences
- Convenience rules

**Examples:**
- Allowing safe read operations
- Requiring approval for writes
- Custom tool access patterns

```toml
[[rules]]
name = "Workspace-specific rule"
priority = "user"
action = "allow"
tool_pattern = "read_*"
```

### Default Priority

**Use for:**
- Fallback rules
- System defaults
- Low-priority policies

**Note:** Default priority rules are rarely needed since approval mode provides the fallback.

## Pattern Matching Best Practices

### Be Specific

**Bad:**
```toml
tool_pattern = "*"  # Too broad
```

**Good:**
```toml
tool_pattern = "read_*"  # Specific pattern
```

### Use Argument Patterns

Combine tool and argument patterns for fine-grained control:

```toml
[[rules]]
name = "Allow reading only markdown files"
priority = "user"
action = "allow"
tool_pattern = "read_file"
arg_pattern = "*.md"
```

### Test Patterns

Always test patterns before deploying:

```bash
# Test if a pattern matches
rad policy check read_file config.toml

# Validate policy file
rad policy validate
```

## Session Constitution Guidelines

### When to Use

- **Temporary restrictions** - Add constraints for a specific task
- **Per-session security** - Different security levels for different sessions
- **Dynamic policies** - Adjust policies based on runtime conditions

### Best Practices

1. **Clear expiration** - Constitutions auto-expire after 1 hour
2. **Limit rules** - Maximum 50 rules per session (enforced)
3. **Combine with static rules** - Use constitutions to supplement, not replace, static rules

## Audit Logging

### Enable Logging

Policy decisions should be logged for audit trails:

1. **Log all decisions** - Record allow/deny/ask decisions
2. **Include context** - Log tool name, arguments, matched rule
3. **Track changes** - Log when policies are modified

### Log Format

```json
{
  "timestamp": "2025-01-15T10:30:00Z",
  "tool": "write_file",
  "args": ["config.toml"],
  "decision": "ask_user",
  "matched_rule": "Require approval for file writes",
  "session_id": "session-123"
}
```

## Common Security Pitfalls

### 1. Overly Permissive Patterns

**Problem:**
```toml
tool_pattern = "*"  # Allows everything
```

**Solution:**
```toml
tool_pattern = "read_*"  # Specific pattern
```

### 2. Missing Admin Rules

**Problem:** Critical security rules at User priority can be overridden.

**Solution:** Use Admin priority for security-critical rules.

### 3. Ignoring Argument Patterns

**Problem:** Only matching tool names, not arguments.

**Solution:** Use argument patterns for fine-grained control:

```toml
[[rules]]
name = "Block dangerous arguments"
priority = "admin"
action = "deny"
tool_pattern = "run_terminal_cmd"
arg_pattern = "rm -rf *"
```

### 4. Not Testing Policies

**Problem:** Deploying untested policies.

**Solution:** Always test with `rad policy check` before deploying.

### 5. Forgetting Approval Mode

**Problem:** Relying only on rules, ignoring approval mode fallback.

**Solution:** Set appropriate approval mode as safety net:

```toml
approval_mode = "ask"  # Safe default
```

## Enterprise Deployment

### Policy Distribution

1. **Version control** - Commit policy.toml to version control
2. **Team sharing** - Share policies via git
3. **Centralized management** - Use admin rules for org-wide policies

### Policy Review

1. **Regular audits** - Review policies quarterly
2. **Change management** - Require approval for policy changes
3. **Documentation** - Document all rules with reasons

### Compliance

1. **Regulatory requirements** - Ensure policies meet compliance needs
2. **Audit trails** - Maintain logs of all policy decisions
3. **Access control** - Restrict who can modify policies

## Performance Considerations

### Rule Count

- **Optimal:** 10-50 rules
- **Acceptable:** 50-100 rules
- **Problematic:** 100+ rules (consider consolidation)

### Pattern Complexity

- **Simple patterns** - Fast evaluation
- **Complex patterns** - May slow evaluation
- **Argument patterns** - Additional overhead

### Optimization Tips

1. **Order rules by frequency** - Most common rules first (within priority)
2. **Consolidate similar rules** - Combine rules with same action
3. **Use specific patterns** - More specific = faster evaluation

## Migration Guide

### From No Policy to Policy Engine

1. **Start with Ask mode** - `approval_mode = "ask"`
2. **Add allow rules gradually** - Start with most common safe operations
3. **Add deny rules for dangerous ops** - Block known dangerous operations
4. **Test thoroughly** - Use `rad policy check` extensively
5. **Monitor and adjust** - Review logs and adjust as needed

### From Other Policy Systems

1. **Map approval modes** - Match your current mode to Radium modes
2. **Convert rules** - Translate rules to Radium format
3. **Test equivalence** - Verify same behavior
4. **Gradual migration** - Migrate one workspace at a time

## See Also

- [Policy Engine Documentation](../features/policy-engine.md)
- [Example Configurations](../../examples/policy-examples.toml)
- [CLI Reference](../cli/commands/policy.md)

