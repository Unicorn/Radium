# Policy Profiles Guide

This guide explains the different policy profiles available in `examples/policies/` and when to use each one. Each profile demonstrates different security tiers and is optimized for specific agent types and use cases.

## Overview

Policy profiles define security rules that control which tools agents can execute and under what conditions. They use a combination of:
- **Approval modes**: `yolo`, `autoEdit`, or `ask`
- **Priority levels**: `admin` (highest), `user`, `default` (lowest)
- **Glob patterns**: Match tool names and arguments
- **Actions**: `allow`, `deny`, or `ask_user`

## Available Profiles

### 1. Permissive Policy (`permissive-policy.toml`)

**Security Level**: Low (Maximum Automation)

**Approval Mode**: `yolo` - Auto-approves all operations

**Use Case**: 
- Local development with trusted agents
- Rapid prototyping and experimentation
- When you want maximum automation with minimal friction

**Characteristics**:
- Allows all file operations (read and write)
- Allows safe shell commands (git, cargo, npm)
- Still blocks dangerous operations (rm -rf, sudo)
- Allows MCP tools

**Trade-offs**:
- ✅ Maximum speed and automation
- ✅ Minimal user interaction required
- ⚠️ Lower security - agents can make changes without approval
- ⚠️ Risk of unintended modifications

**Recommended For**:
- Personal development projects
- Trusted development environments
- Rapid iteration workflows

---

### 2. Balanced Policy (`balanced-policy.toml`)

**Security Level**: Medium (Balanced Security and Usability)

**Approval Mode**: `autoEdit` - Auto-approves edit operations, asks for others

**Use Case**:
- General development with safety guards
- When you want automation for common operations but oversight for unusual ones
- Team development environments

**Characteristics**:
- Allows all read operations
- Auto-approves file edit operations (write_file, edit_file)
- Requires approval for other write operations
- Allows safe shell commands (git, cargo, test)
- Blocks dangerous operations (rm -rf, sudo)
- Requires approval for MCP tools

**Trade-offs**:
- ✅ Good balance of automation and safety
- ✅ Common operations are streamlined
- ✅ Unusual operations require approval
- ⚠️ Some operations still require manual approval

**Recommended For**:
- General development workflows
- Team environments
- Most common use cases

---

### 3. Strict Policy (`strict-policy.toml`)

**Security Level**: High (Maximum Security)

**Approval Mode**: `ask` - Requires approval for all operations

**Use Case**:
- Production-like environments
- When maximum security is required
- Review-heavy workflows

**Characteristics**:
- Allows only read operations
- Denies all file writes
- Denies all shell commands
- Denies all MCP tools
- Requires approval for everything else

**Trade-offs**:
- ✅ Maximum security and control
- ✅ All operations are reviewed
- ⚠️ High friction - many approvals required
- ⚠️ Slower workflow due to approval requirements

**Recommended For**:
- Production environments
- Security-critical projects
- When you need complete oversight

---

### 4. Research Profile (`research-profile.toml`)

**Security Level**: High (Read-Only)

**Approval Mode**: `ask` - Requires approval for non-read operations

**Use Case**:
- Research agents (e.g., `research-agent`)
- Code exploration and documentation search
- When agents should only read, never modify

**Characteristics**:
- Allows all read operations (read_file, codebase_search, grep, list_dir)
- Denies all write operations (write_file, search_replace, delete_file)
- Denies all shell commands
- Denies network operations
- Denies MCP tools with side effects

**Trade-offs**:
- ✅ Maximum safety for exploration
- ✅ Prevents accidental modifications
- ✅ Ideal for research and analysis agents
- ⚠️ Cannot make any changes (by design)

**Recommended For**:
- `research-agent` - Read-only code exploration
- `analyzer-agent` - Static analysis without modifications
- `reviewer-agent` - Code review without changes

**Agent Mapping**:
- `research-agent` - Perfect match
- `analyzer-agent` - Good match (analysis only)
- `reviewer-agent` - Good match (review only)

---

### 5. Execution Profile (`execution-profile.toml`)

**Security Level**: Medium-High (Write-Enabled with Guards)

**Approval Mode**: `ask` - Requires approval for write operations

**Use Case**:
- Execution agents (e.g., `executor-agent`)
- Code generation and file modifications
- When agents need to write but with oversight

**Characteristics**:
- Allows read operations for code understanding
- Requires approval for file writes and edits
- Allows safe build and test commands
- Blocks dangerous operations (rm -rf, sudo, production access)
- Blocks external API calls
- Requires approval for MCP tools

**Trade-offs**:
- ✅ Allows necessary write operations
- ✅ Blocks dangerous operations automatically
- ✅ Requires approval for writes (safety guard)
- ⚠️ Write operations require manual approval

**Recommended For**:
- `executor-agent` - Code generation and modifications
- Development workflows requiring code changes
- When you need write access with safety

**Agent Mapping**:
- `executor-agent` - Perfect match

---

## Profile Comparison

| Profile | Approval Mode | Read | Write | Shell | Security | Automation |
|---------|--------------|------|-------|-------|----------|------------|
| Permissive | `yolo` | ✅ Allow | ✅ Allow | ✅ Allow (safe) | Low | High |
| Balanced | `autoEdit` | ✅ Allow | ⚠️ Auto-approve edits | ✅ Allow (safe) | Medium | Medium |
| Strict | `ask` | ✅ Allow | ❌ Deny | ❌ Deny | High | Low |
| Research | `ask` | ✅ Allow | ❌ Deny | ❌ Deny | High | Low |
| Execution | `ask` | ✅ Allow | ⚠️ Ask | ✅ Allow (safe) | Medium-High | Medium |

## Choosing a Profile

### For Agent Types

- **Research Agent** → Use `research-profile.toml`
- **Analyzer Agent** → Use `research-profile.toml` (analysis only)
- **Executor Agent** → Use `execution-profile.toml`
- **Reviewer Agent** → Use `research-profile.toml` (review only)

### For Workflows

- **Rapid Development** → Use `permissive-policy.toml`
- **General Development** → Use `balanced-policy.toml`
- **Production/Review** → Use `strict-policy.toml`
- **Code Exploration** → Use `research-profile.toml`
- **Code Generation** → Use `execution-profile.toml`

## Applying a Profile

To apply a policy profile to your workspace:

```bash
# Copy the profile to your workspace policy file
cp examples/policies/balanced-policy.toml .radium/policy.toml

# Or use the policy template system (if available)
rad policy templates apply balanced-policy
```

## Customizing Profiles

You can customize any profile by:

1. **Copying the profile** to `.radium/policy.toml`
2. **Modifying rules** to match your specific needs
3. **Adding custom rules** for your tools and workflows
4. **Adjusting priorities** to change rule evaluation order

## Glob Pattern Examples

Profiles use glob patterns to match tools and arguments:

### Tool Patterns
- `read_*` - Matches all tools starting with "read_"
- `write_*` - Matches all tools starting with "write_"
- `mcp_*` - Matches all MCP tools
- `*` - Matches all tools

### Argument Patterns
- `git *` - Matches commands starting with "git"
- `cargo *` - Matches commands starting with "cargo"
- `rm -rf *` - Matches dangerous deletion commands
- `* production *` - Matches any command containing "production"

## Security Best Practices

1. **Start Strict**: Begin with `strict-policy.toml` and relax rules as needed
2. **Use Agent-Specific Profiles**: Match profiles to agent capabilities
3. **Review Rules Regularly**: Periodically review and update policy rules
4. **Test Policies**: Use `rad policy check` to test tool execution
5. **Document Custom Rules**: Add comments explaining custom rules

## See Also

- [Policy Engine Documentation](../features/policy-engine.md)
- [Policy Best Practices](../security/policy-best-practices.md)
- [Agent Configuration Guide](../user-guide/agent-configuration.md)

