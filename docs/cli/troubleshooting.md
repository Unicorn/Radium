# Troubleshooting Guide

Common issues and solutions when using the Radium CLI.

## Workspace Issues

### "No Radium workspace found"

**Problem**: Command requires a workspace but none is found.

**Solution**:
```bash
rad init
```

### "Failed to discover workspace"

**Problem**: Workspace structure is corrupted or missing.

**Solution**:
```bash
# Reinitialize workspace
rad init --use-defaults

# Or check workspace structure
rad doctor
```

## Plan Execution Issues

### "Plan not found"

**Problem**: Plan identifier doesn't match any existing plan.

**Solution**:
```bash
# List available plans
rad status

# Use correct REQ ID or folder name
rad craft REQ-001
```

### "Plan manifest not found"

**Problem**: Plan directory exists but manifest is missing.

**Solution**:
```bash
# Regenerate plan
rad plan spec.md --id REQ-001
```

### Execution hangs or loops

**Problem**: Plan execution gets stuck in a loop.

**Solution**:
```bash
# Use bounded execution instead of YOLO
rad craft REQ-001  # Default bounded mode

# Or specify iteration limit
rad craft REQ-001 --iteration I1
```

## Agent Issues

### "Agent not found"

**Problem**: Agent ID doesn't exist in configuration.

**Solution**:
```bash
# List available agents
rad agents list

# Check agent discovery
rad agents validate <agent-id>
```

### "Failed to create model"

**Problem**: Model/engine configuration is invalid.

**Solution**:
```bash
# Check engine status
rad engines status

# Verify authentication
rad auth status

# Use mock engine for testing
RADIUM_ENGINE=mock rad step <agent-id>
```

## Authentication Issues

### "Missing credentials"

**Problem**: Authentication required but not configured.

**Solution**:
```bash
# Login to provider
rad auth login <engine>

# Check status
rad auth status
```

### "Authentication failed"

**Problem**: Credentials are invalid or expired.

**Solution**:
```bash
# Logout and login again
rad auth logout <engine>
rad auth login <engine>
```

## Source Detection Issues

### "Source detection failed"

**Problem**: Source format is not recognized.

**Solution**:
- **File**: Ensure file path is correct and file exists
- **Jira**: Use format `PROJ-123` (uppercase, dash, digits)
- **Braingrid**: Use format `REQ-YYYY-NNN` (year and 3+ digits)

```bash
# Check file exists
ls spec.md

# Use correct format
rad complete RAD-42  # Jira
rad complete REQ-2025-001  # Braingrid
```

### "Source not found"

**Problem**: Source exists but cannot be accessed.

**Solution**:
```bash
# For Jira/Braingrid: authenticate first
rad auth login jira
rad auth login braingrid

# For files: check permissions
ls -l spec.md
```

## Extension Issues

### "Extension not found"

**Problem**: Extension doesn't exist or isn't installed.

**Solution**:
```bash
# List installed extensions
rad extension list

# Install extension
rad extension install <source>
```

### "Extension installation failed"

**Problem**: Extension package is invalid or corrupted.

**Solution**:
```bash
# Validate extension before install
# Check extension manifest structure

# Try with overwrite
rad extension install <source> --overwrite
```

## Monitoring Issues

### "Failed to open monitoring database"

**Problem**: Monitoring database doesn't exist yet.

**Solution**:
- This is normal if no agents have been executed yet
- Execute an agent first to create the database

```bash
rad step <agent-id>
```

### "Agent not found" in monitoring

**Problem**: Agent hasn't been tracked yet.

**Solution**:
- Agents are tracked during execution
- Run an agent first:

```bash
rad step <agent-id>
rad monitor list
```

## Checkpoint Issues

### "Workspace is not a git repository"

**Problem**: Checkpoints require git.

**Solution**:
```bash
# Initialize git repository
git init
git add .
git commit -m "Initial commit"

# Now checkpoints will work
rad checkpoint list
```

### "Checkpoint not found"

**Problem**: Checkpoint ID doesn't exist.

**Solution**:
```bash
# List available checkpoints
rad checkpoint list

# Use correct checkpoint ID
rad checkpoint restore <checkpoint-id>
```

## Performance Issues

### Slow execution

**Problem**: Execution is taking too long.

**Solution**:
```bash
# Use specific iteration/task instead of full plan
rad craft REQ-001 --iteration I1

# Check agent performance
rad monitor telemetry

# Use faster engine/model
rad craft REQ-001 --engine mock
```

### High token usage

**Problem**: Token usage is higher than expected.

**Solution**:
```bash
# Check usage
rad stats usage

# Use more efficient models
rad step <agent-id> --model gpt-3.5-turbo

# Review agent prompts for verbosity
```

## JSON Output Issues

### "Invalid JSON"

**Problem**: JSON output is malformed.

**Solution**:
```bash
# Ensure --json flag is used
rad status --json

# Parse with jq for validation
rad status --json | jq .
```

## General Issues

### Command not found

**Problem**: `rad` command is not in PATH.

**Solution**:
```bash
# Build and install
cargo build --release -p radium-cli

# Add to PATH
export PATH=$PATH:/path/to/target/release
```

### Permission denied

**Problem**: Insufficient permissions for workspace operations.

**Solution**:
```bash
# Check directory permissions
ls -ld .radium

# Fix permissions if needed
chmod -R u+w .radium
```

### Out of memory

**Problem**: Process runs out of memory.

**Solution**:
```bash
# Use bounded execution
rad craft REQ-001  # Not --yolo

# Execute smaller chunks
rad craft REQ-001 --iteration I1
```

## Getting More Help

### Enable verbose logging

```bash
RUST_LOG=debug rad <command>
```

### Check workspace health

```bash
rad doctor
```

### View command help

```bash
rad <command> --help
rad <command> <subcommand> --help
```

