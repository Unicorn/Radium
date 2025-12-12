# Plan Generation and Execution Commands

Commands for generating plans from specifications and executing them.

## `rad plan`

Generate a structured plan from a specification file.

### Usage

```bash
rad plan <input> [options]
```

### Arguments

- `input` - Path to specification file or direct content

### Options

- `--id <REQ-ID>` - Override auto-generated requirement ID
- `--name <name>` - Customize folder name suffix

### Examples

```bash
# Generate plan from file
rad plan spec.md

# Generate plan with custom ID
rad plan spec.md --id REQ-001

# Generate plan with custom name
rad plan spec.md --name my-feature

# Generate plan from direct input
rad plan "Build a REST API with authentication"
```

## `rad craft`

Execute a generated plan through its iterations and tasks.

### Usage

```bash
rad craft <plan-identifier> [options]
```

### Arguments

- `plan-identifier` - Plan ID (REQ-XXX) or folder name

### Options

- `--iteration <I1>` - Execute specific iteration only
- `--task <I1.T1>` - Execute specific task only
- `--resume` - Resume from last checkpoint
- `--dry-run` - Show what would be executed without running
- `--json` - Output results as JSON
- `--yolo` - Enable continuous execution mode (runs until all tasks complete)
- `--engine <engine>` - Engine to use for execution

### Examples

```bash
# Execute plan by REQ ID
rad craft REQ-001

# Execute specific iteration
rad craft REQ-001 --iteration I1

# Execute specific task
rad craft REQ-001 --task I1.T1

# Resume from checkpoint
rad craft REQ-001 --resume

# Dry run
rad craft REQ-001 --dry-run

# YOLO mode (continuous execution)
rad craft REQ-001 --yolo

# Use specific engine
rad craft REQ-001 --engine claude
```

## `rad complete`

Complete workflow from source to execution (automatically generates plan and executes).

### Usage

```bash
rad complete <source>
```

### Arguments

- `source` - File path, Jira ticket ID (RAD-42), or Braingrid REQ ID (REQ-2025-001)

### Examples

```bash
# Complete from file
rad complete spec.md

# Complete from Jira ticket
rad complete RAD-42

# Complete from Braingrid REQ
rad complete REQ-2025-001
```

### Source Types

- **Local File**: Path to a markdown specification file
- **Jira Ticket**: Format `PROJ-123` (uppercase letters, dash, digits)
- **Braingrid REQ**: Format `REQ-YYYY-NNN` (year and 3+ digit number)

