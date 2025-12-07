---
req_id: REQ-003
title: Core CLI Commands
phase: NOW
status: Completed
priority: Critical
estimated_effort: 11-14 hours
dependencies: [REQ-001, REQ-002]
related_docs:
  - docs/project/02-now-next-later.md#step-2-core-cli-commands
  - docs/project/03-implementation-plan.md#step-2-core-cli-commands
  - docs/legacy/legacy-system-feature-backlog.md#11-core-commands
---

# Core CLI Commands

## Problem Statement

Radium requires a comprehensive command-line interface that matches the legacy system's `rad` command structure. Without a complete CLI, users cannot:
- Initialize and manage workspaces
- Generate and execute plans
- Manage agents and templates
- Execute individual agents or workflows
- View workspace status and diagnostics
- Authenticate with AI providers

The legacy system provided a rich CLI with commands for all major operations. Radium needs an equivalent CLI that provides the same functionality with improved error handling and JSON output support.

## Solution Overview

Implement a complete CLI system that provides:
- Workspace management commands (`rad init`, `rad status`, `rad clean`, `rad doctor`)
- Plan generation and execution (`rad plan`, `rad craft`)
- Agent and template management (`rad agents`, `rad templates`)
- Agent execution (`rad step`, `rad run`)
- Authentication management (`rad auth`)
- Comprehensive test coverage (216 tests, 95% coverage)

The CLI serves as the primary user interface for Radium, enabling all workspace, plan, and agent operations.

## Functional Requirements

### FR-1: Workspace Management Commands

**Description**: Commands for initializing and managing Radium workspaces.

**Acceptance Criteria**:
- [x] `rad init` - Intelligent workspace initialization with interactive wizard
- [x] `rad status` - Show workspace and engine status (human and JSON output)
- [x] `rad clean` - Clean workspace artifacts (verbose and non-verbose modes)
- [x] `rad doctor` - Environment validation and diagnostics
- [x] Git/VCS root detection and warnings
- [x] Workspace structure validation
- [x] JSON output support for scripting

**Implementation**: 
- `apps/cli/src/commands/init.rs`
- `apps/cli/src/commands/status.rs`
- `apps/cli/src/commands/clean.rs`
- `apps/cli/src/commands/doctor.rs`

### FR-2: Plan Generation and Execution Commands

**Description**: Commands for generating plans from specifications and executing them.

**Acceptance Criteria**:
- [x] `rad plan <spec-path>` - Generate plans from specification markdown files
- [x] `rad plan` - Interactive mode with direct input
- [x] `rad craft [plan-identifier]` - Execute plans with iteration/task selection
- [x] Plan discovery by REQ-ID or folder name
- [x] Progress tracking and display
- [x] Dry-run mode support
- [x] JSON output for CI/CD integration
- [x] Resume from checkpoint support

**Implementation**:
- `apps/cli/src/commands/plan.rs`
- `apps/cli/src/commands/craft.rs`

### FR-3: Agent and Template Management Commands

**Description**: Commands for managing agents and workflow templates.

**Acceptance Criteria**:
- [x] `rad agents list` - List all discovered agents
- [x] `rad agents search <query>` - Search agents by name or description
- [x] `rad agents info <id>` - Show agent details
- [x] `rad agents validate <id>` - Validate agent configuration
- [x] `rad agents create <id>` - Create agent template
- [x] `rad templates list` - List all workflow templates
- [x] `rad templates info <id>` - Show template details
- [x] `rad templates validate <id>` - Validate template structure

**Implementation**:
- `apps/cli/src/commands/agents.rs`
- `apps/cli/src/commands/templates.rs`

### FR-4: Agent Execution Commands

**Description**: Commands for executing individual agents or agent scripts.

**Acceptance Criteria**:
- [x] `rad step <agent-id> [input]` - Execute single agent with optional input
- [x] `rad run <script-path>` - Execute agent script/workflow
- [x] Agent discovery and validation
- [x] Input handling (file, stdin, direct)
- [x] Output formatting (human-readable, JSON)
- [x] Error handling and reporting

**Implementation**:
- `apps/cli/src/commands/step.rs`
- `apps/cli/src/commands/run.rs`

### FR-5: Authentication Management

**Description**: Commands for managing authentication with AI providers.

**Acceptance Criteria**:
- [x] `rad auth login <engine>` - Login to AI provider
- [x] `rad auth logout <engine>` - Logout from AI provider
- [x] `rad auth status` - Show authentication status
- [x] API key management
- [x] Multiple provider support

**Implementation**:
- `apps/cli/src/commands/auth.rs`

### FR-6: CLI Structure and Help

**Description**: Consistent CLI structure with comprehensive help text.

**Acceptance Criteria**:
- [x] `rad` as main command with subcommands
- [x] Proper help text for all commands
- [x] Command argument parsing and validation
- [x] Error messages and usage hints
- [x] Non-interactive mode detection (CI environment)

**Implementation**:
- `apps/cli/src/main.rs`
- `apps/cli/src/commands/mod.rs`

## Technical Requirements

### TR-1: CLI Command Structure

**Description**: Command-line interface structure matching legacy system.

**Command Hierarchy**:
```
rad
├── init                    # Initialize workspace
├── status                  # Show workspace status
├── clean                   # Clean workspace artifacts
├── doctor                  # Environment diagnostics
├── plan <spec>             # Generate plan from specification
├── craft [plan-id]         # Execute plan
├── agents
│   ├── list                # List all agents
│   ├── search <query>      # Search agents
│   ├── info <id>           # Show agent info
│   ├── validate <id>       # Validate agent
│   └── create <id>         # Create agent template
├── templates
│   ├── list                # List templates
│   ├── info <id>           # Show template info
│   └── validate <id>       # Validate template
├── step <agent-id> [input] # Execute single agent
├── run <script>            # Execute agent script
└── auth
    ├── login <engine>      # Login to provider
    ├── logout <engine>     # Logout from provider
    └── status              # Show auth status
```

### TR-2: Command Output Formats

**Description**: Support for human-readable and JSON output formats.

**APIs**:
```rust
pub trait CommandOutput {
    fn human(&self) -> String;
    fn json(&self) -> Result<String>;
}

// Example: Status command output
pub struct StatusOutput {
    pub workspace: WorkspaceStatus,
    pub engines: Vec<EngineStatus>,
    pub auth: AuthStatus,
}
```

### TR-3: Error Handling

**Description**: Comprehensive error handling with user-friendly messages.

**Error Types**:
- Workspace errors (not found, invalid structure)
- Plan errors (not found, invalid format)
- Agent errors (not found, invalid config)
- Execution errors (agent failure, timeout)
- Authentication errors (invalid credentials)

## User Experience

### UX-1: Workspace Initialization

**Description**: Users initialize workspace with interactive wizard.

**Example**:
```bash
$ rad init
Creating workspace structure...
✓ Created .radium directory
✓ Created stage directories
✓ Initialized requirement counter
Workspace initialized at /path/to/project
```

### UX-2: Plan Generation

**Description**: Users generate plans from specification files.

**Example**:
```bash
$ rad plan spec.md
Generating plan...
  Requirement ID: REQ-001
  Folder name: REQ-001-feature-name
  ✓ Generated 3 iterations
  ✓ Generated 12 tasks
Plan created: radium/backlog/REQ-001-feature-name
```

### UX-3: Plan Execution

**Description**: Users execute plans with progress tracking.

**Example**:
```bash
$ rad craft REQ-001
Executing plan: REQ-001-feature-name
Iteration 1 (3/12 tasks complete, 25%)
  ✓ Task 1.1: Setup workspace
  ✓ Task 1.2: Create data models
  → Task 1.3: Implement API endpoints
```

### UX-4: Agent Management

**Description**: Users manage and validate agents.

**Example**:
```bash
$ rad agents list
Found 5 agents:
  arch-agent (core) - Architecture Agent
  plan-agent (core) - Planning Agent
  code-agent (core) - Code Generation Agent
  review-agent (core) - Code Review Agent
  doc-agent (core) - Documentation Agent

$ rad agents validate arch-agent
✓ Agent configuration valid
✓ Prompt file found
✓ All required fields present
```

## Data Requirements

### DR-1: Command Configuration

**Description**: CLI command definitions and argument parsing.

**Implementation**: Uses `clap` crate for argument parsing

### DR-2: Output Formats

**Description**: Structured output for JSON and human-readable formats.

**Format**: JSON for scripting, human-readable for interactive use

## Dependencies

- **REQ-001**: Workspace System - Required for workspace operations
- **REQ-002**: Agent Configuration - Required for agent management commands

## Success Criteria

1. [x] All CLI commands implemented and functional
2. [x] CLI structure matches legacy system
3. [x] Comprehensive test coverage (216 tests, 95% coverage)
4. [x] JSON output support for all commands
5. [x] Error handling with user-friendly messages
6. [x] Help text and usage information
7. [x] Non-interactive mode support (CI environment)

**Completion Metrics**:
- **Status**: ✅ Complete
- **Test Coverage**: ✅ 216 tests across 15 test files
- **Coverage Breakdown**:
  - `rad init` - 15 tests
  - `rad status` - 14 tests
  - `rad clean` - 12 tests
  - `rad plan` - 11 tests
  - `rad craft` - 11 tests
  - `rad agents` - 18 tests
  - `rad templates` - 13 tests
  - `rad auth` - 8 tests
  - `rad step` - 10 tests
  - `rad run` - 10 tests
  - `rad doctor` - 11 tests
  - End-to-end integration - 66 tests
- **Implementation**: All CLI commands operational
- **Files**: 
  - `apps/cli/src/main.rs`
  - `apps/cli/src/commands/*.rs` (21 command files)

## Out of Scope

- TUI interface (covered in REQ-016)
- Desktop application (separate project)
- Advanced workflow features (covered in REQ-004)

## References

- [Now/Next/Later Roadmap](../project/02-now-next-later.md#step-2-core-cli-commands)
- [Implementation Plan](../project/03-implementation-plan.md#step-2-core-cli-commands)
- [Feature Backlog](../legacy/legacy-system-feature-backlog.md#11-core-commands)
- [CLI Commands Implementation](../../apps/cli/src/commands/)

