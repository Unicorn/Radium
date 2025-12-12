# Common Workflows

This guide covers common workflows and patterns for using the Radium CLI.

## Getting Started

### 1. Initialize Workspace

```bash
# Initialize in current directory
rad init

# Initialize with context file
rad init --with-context
```

### 2. Create Your First Plan

```bash
# Generate plan from specification
rad plan spec.md

# Or use direct input
rad plan "Build a REST API with authentication"
```

### 3. Execute Plan

```bash
# Execute plan
rad craft REQ-001

# Or use YOLO mode for continuous execution
rad craft REQ-001 --yolo
```

## Complete Workflow

### From Specification to Execution

```bash
# Single command to complete entire workflow
rad complete spec.md
```

This automatically:
1. Detects source type (file, Jira, Braingrid)
2. Fetches content
3. Generates plan
4. Executes plan with YOLO mode

## Agent Execution Patterns

### Single Agent Execution

```bash
# Execute agent with default prompt
rad step code-agent

# Execute with custom prompt
rad step code-agent "Implement user authentication"
```

### Parallel Execution

```bash
# Run multiple agents in parallel
rad run "agent1 'task1' & agent2 'task2' & agent3 'task3'"
```

### Sequential Execution

```bash
# Run agents sequentially (each waits for previous)
rad run "agent1 'setup' && agent2 'build' && agent3 'test'"
```

## Plan Management

### Resuming Execution

```bash
# Resume from last checkpoint
rad craft REQ-001 --resume
```

### Selective Execution

```bash
# Execute specific iteration
rad craft REQ-001 --iteration I1

# Execute specific task
rad craft REQ-001 --task I1.T1
```

### Dry Run

```bash
# See what would be executed
rad craft REQ-001 --dry-run
```

## Monitoring and Analytics

### Track Execution

```bash
# Monitor agent execution
rad monitor list

# Check specific agent
rad monitor status agent-123

# View telemetry
rad monitor telemetry agent-123
```

### View Statistics

```bash
# Session statistics
rad stats session

# Cost tracking
rad stats costs

# Token usage
rad stats usage
```

## Extension Management

### Install Extensions

```bash
# Install from local directory
rad extension install ./my-extension

# Install from URL
rad extension install https://example.com/extension.zip
```

### Manage Extensions

```bash
# List installed extensions
rad extension list

# Get extension info
rad extension info my-extension

# Uninstall extension
rad extension uninstall my-extension
```

## MCP Integration

### Configure MCP Servers

1. Create `.radium/mcp-servers.toml`:

```toml
[[servers]]
name = "my-server"
transport = "stdio"
command = "mcp-server"
args = ["--config", "config.json"]
```

2. List and test:

```bash
rad mcp list
rad mcp test my-server
rad mcp tools my-server
```

## Best Practices

### 1. Use Workspace Structure

Always initialize workspace before starting:

```bash
rad init
```

### 2. Validate Before Execution

```bash
# Validate agents
rad agents validate arch-agent

# Validate templates
rad templates validate basic-workflow
```

### 3. Monitor Execution

Keep an eye on execution:

```bash
# In another terminal
watch -n 1 'rad monitor list'
```

### 4. Use Checkpoints

Checkpoints are automatically created. Restore if needed:

```bash
rad checkpoint list
rad checkpoint restore checkpoint-123
```

### 5. JSON Output for Scripting

Use `--json` flag for programmatic access:

```bash
rad status --json | jq '.workspace'
rad agents list --json | jq '.[] | select(.name | contains("code"))'
```

