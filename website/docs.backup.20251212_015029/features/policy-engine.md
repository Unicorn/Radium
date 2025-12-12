# Policy Engine

The Radium Policy Engine provides fine-grained control over tool execution to ensure security and prevent unwanted operations. It enables workspace-specific and enterprise-ready security policies with rule-based enforcement.

## Overview

The policy engine allows you to:

- Control which tools agents can execute
- Set approval requirements for sensitive operations
- Configure workspace-specific security policies
- Enforce enterprise security requirements
- Prevent accidental destructive operations

## Features

- **TOML-based configuration** - Simple, declarative policy rules
- **Priority-based matching** - Admin > User > Default priority tiers
- **Pattern matching** - Glob patterns for tool names and arguments
- **Approval modes** - Yolo, AutoEdit, and Ask modes for different security levels
- **Session constitutions** - Per-session rules for temporary constraints
- **Hook integration** - Works with Radium's hook system for extensibility

## Configuration

Policy rules are configured in `.radium/policy.toml` in your workspace root.

### Basic Structure

```toml
# Approval mode: yolo, autoEdit, or ask
approval_mode = "ask"

# Policy rules
[[rules]]
name = "Rule name"
priority = "user"  # admin, user, or default
action = "allow"   # allow, deny, or ask_user
tool_pattern = "read_*"
reason = "Optional reason for this rule"
```

### Approval Modes

- **`yolo`** - Auto-approve all tool executions (use with caution)
- **`autoEdit`** - Auto-approve file edit operations, ask for others
- **`ask`** - Ask for approval on all tool executions (safest, default)

### Priority Levels

Rules are evaluated in priority order (highest first):

1. **`admin`** - Highest priority, typically for security-critical rules
2. **`user`** - Medium priority, for user-defined policies
3. **`default`** - Lowest priority, for default system policies

The first matching rule wins. If no rules match, the approval mode default is applied.

### Actions

- **`allow`** - Allow tool execution without prompting
- **`deny`** - Block tool execution
- **`ask_user`** - Require user approval before execution

## Pattern Matching

### Tool Name Patterns

Use glob patterns to match tool names:

```toml
# Match all tools starting with "read_"
tool_pattern = "read_*"

# Match specific tool
tool_pattern = "write_file"

# Match MCP tools
tool_pattern = "mcp_*"

# Match tools from specific MCP server
tool_pattern = "mcp_server1_*"
```

### Argument Patterns

Optionally match tool arguments:

```toml
[[rules]]
name = "Block dangerous commands"
priority = "admin"
action = "deny"
tool_pattern = "run_terminal_cmd"
arg_pattern = "rm -rf *"
reason = "Prevent accidental deletion"
```

Argument patterns can match:
- Individual arguments
- The full argument string (space-joined)

## Example Configurations

### Safe Default Configuration

```toml
approval_mode = "ask"

[[rules]]
name = "Allow safe file operations"
priority = "user"
action = "allow"
tool_pattern = "read_*"
reason = "Safe read operations are always allowed"

[[rules]]
name = "Require approval for file writes"
priority = "user"
action = "ask_user"
tool_pattern = "write_*"
reason = "File writes require user approval"

[[rules]]
name = "Deny dangerous shell commands"
priority = "admin"
action = "deny"
tool_pattern = "run_terminal_cmd"
arg_pattern = "rm -rf *"
reason = "Prevent accidental deletion"
```

### Enterprise Security Configuration

```toml
approval_mode = "ask"

# Admin rules (highest priority)
[[rules]]
name = "Block all network operations"
priority = "admin"
action = "deny"
tool_pattern = "http_*"
reason = "Enterprise policy: no external network access"

[[rules]]
name = "Block file system operations outside workspace"
priority = "admin"
action = "deny"
tool_pattern = "write_file"
arg_pattern = "../*"
reason = "Enterprise policy: workspace isolation"

# User rules (medium priority)
[[rules]]
name = "Allow safe operations"
priority = "user"
action = "allow"
tool_pattern = "read_*"

[[rules]]
name = "Require approval for edits"
priority = "user"
action = "ask_user"
tool_pattern = "write_*"
```

## CLI Commands

### List Policies

```bash
# List all policy rules
rad policy list

# Verbose output with table format
rad policy list --verbose

# JSON output
rad policy list --json
```

### Check Policy Evaluation

```bash
# Check if a tool would be allowed
rad policy check read_file config.toml

# Check with multiple arguments
rad policy check run_terminal_cmd "rm -rf /tmp/test"

# JSON output
rad policy check write_file test.txt --json
```

### Validate Policy File

```bash
# Validate default policy file
rad policy validate

# Validate specific file
rad policy validate --file /path/to/policy.toml
```

### Initialize Policy File

```bash
# Create default policy.toml template
rad policy init

# Overwrite existing file
rad policy init --force
```

## Session Constitutions

Session constitutions allow you to add temporary rules for a specific execution session. These rules are automatically cleaned up after 1 hour of inactivity.

### Use Cases

- Temporary restrictions for a specific task
- Per-session security constraints
- Dynamic policy adjustments

### Integration

Session constitutions are managed through the `ConstitutionManager` and integrated with workflow execution. Rules are combined with static policy rules, with session rules taking precedence when there are conflicts.

## Workflow Integration

The policy engine is automatically integrated with workflow execution:

1. Policy engine is initialized from `.radium/policy.toml` if present
2. Constitution manager is available for session-based rules
3. Policy evaluation happens during tool execution
4. Decisions are logged for audit trails

## Architecture

### Components

- **PolicyEngine** - Core evaluation engine
- **PolicyRule** - Individual rule definition
- **ConstitutionManager** - Session-based rule management
- **PolicyDecision** - Evaluation result

### Evaluation Flow

1. Tool execution request received
2. BeforeTool hooks executed (if registered)
3. Rules evaluated in priority order (Admin > User > Default)
4. First matching rule's action returned
5. If no match, approval mode default applied
6. AfterTool hooks executed (if registered)

## Best Practices

1. **Start with Ask mode** - Use `ask` approval mode for maximum safety
2. **Use Admin priority sparingly** - Reserve for critical security rules
3. **Test policies** - Use `rad policy check` to test rules before deployment
4. **Document rules** - Always include `reason` fields for clarity
5. **Validate patterns** - Use `rad policy validate` to check syntax
6. **Version control** - Commit policy.toml to version control for team sharing

## Troubleshooting

### Pattern Not Matching

- Check glob pattern syntax
- Verify tool name format (use `rad policy check` to test)
- Ensure pattern doesn't have extra spaces

### Rule Not Applied

- Check rule priority (higher priority rules win)
- Verify rule order (first match wins)
- Ensure approval mode default isn't overriding

### Performance with Many Rules

- Rules are sorted by priority on load
- Evaluation stops at first match
- Consider consolidating similar rules

## See Also

- [Policy Best Practices](../security/policy-best-practices.md)
- [Example Configurations](../../examples/policy-examples.toml)
- [CLI Reference](../cli/commands/policy.md)

