# Radium Project Rules for Claude

> **Instructions for Claude AI when working on the Radium project.**
> 
> Include this file in your system prompt or provide it as context at the start of a session.

## Project Overview

You are assisting with **Radium**, a Rust-based agentic orchestration tool. The codebase is a Cargo workspace containing multiple crates for the backend, model abstraction, and agent orchestration.

## 🚨 MANDATORY: Before ANY Code Changes

Every session must start with these steps:

### Step 1: Request Progress File
Ask the human to provide `roadmap/PROGRESS.md`:
> "I need to check the current project status. Please share the contents of `roadmap/PROGRESS.md`."

### Step 2: Review Task Status
From PROGRESS.md, identify:
- Current sprint and goals
- Active tasks (especially unassigned ones)
- Task dependencies
- Any blockers

### Step 3: Confirm Before Proceeding
Before writing code, verify:
- [ ] Task is unassigned or assigned to you
- [ ] All dependencies are complete (check ✅ tasks)
- [ ] No blockers affect the task

## Project Structure

```
radium/                    # Main Rust workspace
├── Cargo.toml            # Workspace manifest
├── radium-core/          # Core server and infrastructure
│   ├── Cargo.toml
│   ├── build.rs          # Protobuf compilation
│   ├── proto/radium.proto
│   └── src/
│       ├── lib.rs        # Library exports
│       ├── main.rs       # Server binary
│       ├── error.rs      # Error types
│       ├── config/       # Configuration
│       ├── server/       # gRPC implementation
│       ├── models/       # Data structures
│       └── storage/      # Database layer
├── model-abstraction/    # Model trait definitions
│   └── src/lib.rs        # Model, ChatMessage, etc.
├── radium-models/        # Model implementations
│   └── src/lib.rs        # MockModel, future: real models
├── agent-orchestrator/   # Agent framework
│   └── src/lib.rs        # Agent trait, Orchestrator
└── apps/radium-client/   # CLI client
    └── src/main.rs

roadmap/                   # Documentation & tracking
├── PROGRESS.md           # 📌 TASK TRACKER
└── 0X-*.md               # Architecture docs

/
├── AGENT_RULES.md        # Development guidelines
├── GEMINI_RULES.md       # Gemini-specific rules
├── CLAUDE_RULES.md       # This file
```

## Task Workflow

### Starting a Task

1. **Confirm task selection** with the human
2. **Provide PROGRESS.md update** for them to apply:

```markdown
## Update for PROGRESS.md

In the Active Tasks section, update:

- [ ] **RAD-XXX**: [Task description]
  - **Status:** In Progress
  - **Assignee:** Claude
  - **Started:** YYYY-MM-DD
  - [other fields unchanged]

In the Update Log, add:
| YYYY-MM-DD | Claude | Started RAD-XXX |
```

### Writing Code

**Always provide complete files.** Structure responses as:

```
### File: `crates/radium-core/src/models/agent.rs`

```rust
// Complete file contents here
// Include all imports
// Include all documentation
// Include all code
```

### Next Steps:
1. Run `cd radium && cargo check`
2. [Additional instructions]
```

**Rust Code Requirements:**

```rust
// ✅ REQUIRED: Result for fallible operations
pub fn load_agent(id: &str) -> Result<Agent, AgentError> {
    let data = storage.get(id)?;
    Agent::try_from(data)
}

// ❌ FORBIDDEN: unwrap/expect in library code
let agent = load_agent(id).unwrap();  // NEVER DO THIS

// ✅ REQUIRED: Documentation for public items
/// Loads an agent by its unique identifier.
///
/// # Arguments
/// * `id` - The unique agent identifier
///
/// # Returns
/// The loaded agent on success
///
/// # Errors
/// * `AgentError::NotFound` - Agent doesn't exist
/// * `AgentError::StorageError` - Database access failed
pub fn load_agent(id: &str) -> Result<Agent, AgentError>
```

### Completing a Task

1. **Ensure all code compiles and passes tests**
2. **Provide PROGRESS.md update:**

```markdown
## Update for PROGRESS.md

Move from Active to Completed:

- [x] **RAD-XXX**: [Task description]
  - **Completed:** YYYY-MM-DD
  - **Commit:** feat(scope): description [RAD-XXX]
  - **Files:** `path/to/file1.rs`, `path/to/file2.rs`

In the Update Log, add:
| YYYY-MM-DD | Claude | Completed RAD-XXX |
```

3. **Provide commit command:**

```bash
git add -A
git commit -m "feat(scope): description [RAD-XXX]"
```

## Pre-Commit Requirements

Before any commit, the human must run:

```bash
cd radium

# All must pass:
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo nextest run  # Faster than cargo test
cargo fmt
```

**Remind the human to run these checks.**

## Commit Message Format

```
type(scope): description [RAD-XXX]
```

| Type | Use For |
|------|---------|
| `feat` | New features |
| `fix` | Bug fixes |
| `docs` | Documentation |
| `refactor` | Code restructuring |
| `test` | Adding tests |
| `chore` | Maintenance |

**Scope examples:** `models`, `storage`, `server`, `api`, `config`, `agent`

## Handling Blockers

When blocked:

1. **Stop and inform the human**
2. **Provide blocker documentation:**

```markdown
## Blocker for PROGRESS.md

Add to Blockers section:
- **BLOCKER-XXX**: [Clear description]
  - **Blocking:** RAD-XXX
  - **Owner:** Claude
  - **Resolution:** Pending
  - **Notes:** [What's needed]

Update the blocked task:
- [ ] **RAD-XXX**: [Description]
  - **Status:** Blocked
  - **Blocked By:** BLOCKER-XXX
```

3. **Suggest alternative tasks** that aren't blocked

## Multi-File Changes

When changes span multiple files:

1. List all files being changed upfront
2. Provide complete contents for each file
3. Specify the order to create/modify them
4. Group related changes together

Example structure:
```
This task requires changes to 3 files:
1. `crates/radium-core/src/models/mod.rs` - Add module export
2. `crates/radium-core/src/models/agent.rs` - New file
3. `crates/radium-core/src/lib.rs` - Re-export types

### File 1: `crates/radium-core/src/models/mod.rs`
[contents]

### File 2: `crates/radium-core/src/models/agent.rs`
[contents]

### File 3: `crates/radium-core/src/lib.rs`
[contents]
```

## Response Guidelines

- Be direct and technical
- Provide complete, compilable code
- Include all imports at the top of files
- Add doc comments to all public items
- Specify exact file paths
- Always remind about pre-commit checks
- Don't apologize; fix issues directly
- When unsure, ask for clarification

## Quick Reference

| Resource | Path |
|----------|------|
| Task Tracker | `roadmap/PROGRESS.md` |
| Full Rules | `roadmap/AGENT_RULES.md` |
| Core Library | `crates/radium-core/src/lib.rs` |
| Proto File | `crates/radium-core/proto/radium.proto` |
| Model Trait | `crates/radium-model-abstraction/src/lib.rs` |
| Agent Trait | `crates/radium-agent-orchestrator/src/lib.rs` |
| Error Types | `crates/radium-core/src/error.rs` |

## Process Management

When running long-running services (dev servers, watchers), **prefer background execution** unless actively debugging.

### Starting Background Processes

Instruct the human to run:
```bash
# Create directories
mkdir -p .pids logs

# Start server in background with PID tracking
cd radium && cargo run --bin radium-core > ../logs/radium-core.log 2>&1 &
echo $! > ../.pids/radium-core.pid
```

### Process Commands Reference

```bash
# Check if process is running
kill -0 $(cat .pids/radium-core.pid) 2>/dev/null && echo "Running" || echo "Stopped"

# View logs
tail -f logs/radium-core.log

# Stop process
kill $(cat .pids/radium-core.pid) && rm .pids/radium-core.pid

# Cleanup all processes
for f in .pids/*.pid; do [ -f "$f" ] && kill $(cat "$f") 2>/dev/null && rm "$f"; done
```

### Mode Selection Guide

| Mode | Use For |
|------|---------|
| **Foreground** | `cargo check`, `cargo test`, `cargo clippy`, debugging issues |
| **Background** | Dev servers, file watchers, integration test services |

### Session Cleanup

Remind humans to cleanup before ending a session:
> "Before we finish, let's stop any running processes: `for f in .pids/*.pid; do [ -f \"$f\" ] && kill $(cat \"$f\") 2>/dev/null && rm \"$f\"; done`"

## Cargo Commands

```bash
# Check compilation
cd radium && cargo check --all-targets

# Run linter
cd radium && cargo clippy --all-targets -- -D warnings

# Run tests
cd radium && cargo test

# Format code
cd radium && cargo fmt

# Build release
cd radium && cargo build --release

# Run server (foreground - for debugging)
cd radium && cargo run --bin radium-core

# Run server (background - preferred)
mkdir -p .pids logs
cd radium && cargo run --bin radium-core > ../logs/radium-core.log 2>&1 &
echo $! > ../.pids/radium-core.pid
```
