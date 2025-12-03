# Radium Project Rules for Gemini

> **Instructions for Gemini AI when working on the Radium project.**
> 
> Copy/paste this file as context when starting a session, or reference it in your system prompt.

## Project Overview

You are assisting with **Radium**, a Rust-based agentic orchestration tool. The project is structured as a Cargo workspace with multiple crates.

## 🚨 CRITICAL: Always Do This First

Before writing ANY code, you MUST:

1. **Request the contents of `roadmap/PROGRESS.md`** - This is the task tracker
2. **Request the contents of `AGENT_RULES.md`** - This has full development guidelines
3. **Verify task assignment** - Do not work on tasks assigned to others
4. **Confirm task dependencies are met** - Check that prerequisite tasks are complete

**Say this to the user:**
> "Before I begin, I need to check the project status. Can you share the contents of `roadmap/PROGRESS.md`?"

## Project Structure

```
radium/                    # Main Rust workspace
├── radium-core/          # Core gRPC server and infrastructure  
│   ├── src/
│   │   ├── lib.rs        # Library entry point
│   │   ├── main.rs       # Server binary
│   │   ├── config/       # Configuration management
│   │   ├── server/       # gRPC service implementation
│   │   ├── models/       # Data structures (agents, workflows, tasks)
│   │   └── storage/      # Database layer
│   └── proto/            # Protocol buffer definitions
├── model-abstraction/    # AI model trait definitions
├── radium-models/        # Model implementations (MockModel, etc.)
├── agent-orchestrator/   # Agent trait and orchestration
└── apps/radium-client/   # CLI client application

roadmap/                   # Documentation
├── PROGRESS.md           # 📌 TASK TRACKER - Always read first
└── *.md                  # Architecture documentation

/
├── AGENT_RULES.md        # Development guidelines
├── GEMINI_RULES.md       # Gemini-specific rules
├── CLAUDE_RULES.md       # This file
```

## Workflow Instructions

### When Starting a Task

1. **Read PROGRESS.md** - Understand current state
2. **Select an unassigned task** - Check the "Active Tasks" section
3. **Provide an update block** for PROGRESS.md:

```markdown
Update PROGRESS.md - Task Start:

- [ ] **RAD-XXX**: [Task description]
  - **Status:** In Progress
  - **Assignee:** Gemini
  - **Started:** [Today's date]

Add to Update Log:
| [Date] | Gemini | Started RAD-XXX |
```

### When Writing Code

Follow these Rust conventions:

```rust
// ✅ Use Result for fallible operations
pub fn process_data(input: &str) -> Result<ProcessedData, ProcessError> {
    let validated = validate_input(input)?;
    let result = transform(validated)?;
    Ok(result)
}

// ❌ NEVER use unwrap() or expect() in library code
let data = get_data().unwrap();  // DO NOT DO THIS

// ✅ Add documentation to all public items
/// Processes input data and returns the result.
///
/// # Arguments
/// * `input` - The raw input string to process
///
/// # Returns
/// The processed data on success
///
/// # Errors
/// Returns `ProcessError::InvalidInput` if the input fails validation
pub fn process_data(input: &str) -> Result<ProcessedData, ProcessError>
```

### When Providing Code

Always provide complete, working code. Structure your responses as:

1. **File path** - Where the code goes
2. **Complete file contents** - Not partial snippets
3. **Instructions** - Any commands to run

Example format:

```
## File: crates/radium-core/src/models/agent.rs

[Complete file content here]

## Commands to run:
cd radium && cargo check
```

### When Completing a Task

Provide an update block for PROGRESS.md:

```markdown
Update PROGRESS.md - Task Complete:

Move to Completed section:
- [x] **RAD-XXX**: [Task description]
  - **Completed:** [Today's date]
  - **Commit:** type(scope): description [RAD-XXX]
  - **Files:** [List of files created/modified]

Add to Update Log:
| [Date] | Gemini | Completed RAD-XXX |
```

Provide the commit command:
```bash
git add -A
git commit -m "type(scope): description [RAD-XXX]"
```

## Pre-Commit Checklist

Before suggesting a commit, ensure the user runs:

```bash
cd radium
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo nextest run  # Faster than cargo test
cargo fmt
```

All commands must pass before committing.

## Commit Message Format

```
type(scope): description [RAD-XXX]
```

**Types:**
- `feat` - New feature
- `fix` - Bug fix  
- `docs` - Documentation only
- `refactor` - Code refactoring
- `test` - Adding tests
- `chore` - Maintenance

**Examples:**
- `feat(models): add Agent and AgentConfig structs [RAD-001]`
- `fix(storage): handle database connection timeout [RAD-004]`
- `docs(api): add gRPC endpoint documentation [RAD-006]`

## Handling Blockers

If you encounter a blocker:

1. Inform the user immediately
2. Provide a blocker update for PROGRESS.md:

```markdown
Add to Blockers section:
- **BLOCKER-XXX**: [Description of the issue]
  - **Blocking:** RAD-XXX, RAD-YYY
  - **Owner:** Gemini
  - **Resolution:** Pending
  - **Notes:** [What's needed to resolve]

Update affected task:
- [ ] **RAD-XXX**: [Task description]
  - **Status:** Blocked
  - **Blocked By:** BLOCKER-XXX
```

3. Suggest moving to an unblocked task

## Response Style

- Be direct and technical
- Provide complete, working code
- Include all necessary imports
- Add proper error handling
- Include documentation comments
- Always specify file paths
- Remind user to run checks before committing

## Process Management

When running long-running services (dev servers, watchers), **prefer background execution** unless actively debugging.

### Starting Background Processes

Instruct the user to run:
```bash
# Create directories
mkdir -p .pids logs

# Start server in background with PID tracking
cd radium && cargo run --bin radium-core > ../logs/radium-core.log 2>&1 &
echo $! > ../.pids/radium-core.pid
```

### Process Commands

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

### When to Recommend Each Mode

| Mode | Use For |
|------|---------|
| **Foreground** | `cargo check`, `cargo test`, `cargo clippy`, debugging issues |
| **Background** | Dev servers, file watchers, integration test services |

### Session Cleanup

Remind users to cleanup before ending:
> "Before we finish, let's stop any running processes: `for f in .pids/*.pid; do [ -f \"$f\" ] && kill $(cat \"$f\") 2>/dev/null && rm \"$f\"; done`"

## Key Files Reference

| Purpose | File Path |
|---------|-----------|
| Tasks | `roadmap/PROGRESS.md` |
| Rules | `roadmap/AGENT_RULES.md` |
| Core lib | `crates/radium-core/src/lib.rs` |
| Protos | `crates/radium-core/proto/radium.proto` |
| Model trait | `crates/radium-model-abstraction/src/lib.rs` |
| Agent trait | `crates/radium-agent-orchestrator/src/lib.rs` |
| Error types | `crates/radium-core/src/error.rs` |
| Config | `crates/radium-core/src/config/mod.rs` |
| PIDs | `.pids/*.pid` |
| Logs | `logs/*.log` |
