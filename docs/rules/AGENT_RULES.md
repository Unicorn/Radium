# Radium Agent Rules

> **Universal guidelines for all AI agents working on the Radium project.**

## 🚨 Critical Rules - Read First

1. **BRAINGRID IS THE SOURCE OF TRUTH** - All REQs and tasks are in Braingrid (PROJ-14). Local markdown files are deprecated.
2. **ALWAYS check Braingrid first** for related REQs and tasks before starting work
3. **NEVER work on a task that is assigned to another agent/person**
4. **UPDATE Braingrid immediately** when starting or completing tasks (Braingrid is primary, PROGRESS.md is secondary)
5. **COMMIT after each logical unit of work is complete**
6. **RUN all checks before committing** (`cargo check`, `cargo clippy`, `cargo test`)

---

## 📖 Before Starting Work

### Step 1: Check Braingrid for Related Requirements and Tasks
**BRAINGRID IS THE SOURCE OF TRUTH** - Always check Braingrid first to understand the project's structured requirements. Local markdown files (like `BG-REQ-*.md`) are deprecated and should not be used once REQs are in Braingrid.

```bash
# List all requirements for the project
braingrid requirement list -p PROJ-14

# If working on a specific feature, search for related REQs
braingrid requirement list -p PROJ-14 --format json | grep -i "feature-name"

# Check tasks for a specific requirement (replace REQ-XXX with actual REQ ID)
braingrid task list -r REQ-XXX -p PROJ-14

# View full requirement details (replace REQ-XXX with actual REQ ID)
braingrid requirement show REQ-XXX -p PROJ-14
```

**What to look for:**
- Existing REQs that match your work scope
- Related tasks that might be in progress or blocked
- Dependencies between requirements
- Acceptance criteria and success metrics
- Out-of-scope items to avoid

**If you find a related REQ:**
- Review the full requirement content for context
- Check associated tasks and their status
- Note any dependencies or blockers
- Update task status when starting work (see Step 3)

**If no related REQ exists:**
- Consider creating a new REQ if the work is substantial
- Use `braingrid specify -p PROJ-14 --prompt "description"` to create a requirement from a prompt
- Braingrid will automatically analyze and break the REQ into tasks
- Link the REQ to your work in commit messages: `[REQ-XXX]`

### Step 2: Read Progress File
```
Read: roadmap/PROGRESS.md
```
Understand:
- Current sprint goals
- Active tasks and their status
- Blockers that might affect your work
- Recently completed tasks for context

### Step 3: Select or Verify Task Assignment
- If assigned a specific task: Verify it's still unassigned in PROGRESS.md
- If self-selecting: Choose an unassigned task with no unmet dependencies
- **Priority order:** High → Medium → Low
- **Cross-reference with BrainGrid:** Ensure BrainGrid tasks align with local progress

### Step 4: Update Progress Files (Local + BrainGrid)
Before writing any code, update both local progress and BrainGrid:

**Update PROGRESS.md:**
```markdown
- [ ] **RAD-XXX**: Task description
  - **Status:** In Progress  <!-- Changed from "Not Started" -->
  - **Assignee:** [Your Agent Name]  <!-- Added -->
  - **BrainGrid REQ:** REQ-XXX (query Braingrid to find actual REQ ID)
  - **BrainGrid Task:** TASK-X (query Braingrid to find actual task ID)
  - ...
```

**Update BrainGrid Task Status (if applicable):**
```bash
# Update task status to IN_PROGRESS (replace TASK-X with actual task ID from Braingrid)
braingrid task update TASK-X -p PROJ-14 --status IN_PROGRESS

# Or if working on a requirement level (replace REQ-XXX with actual REQ ID)
braingrid requirement update REQ-XXX -p PROJ-14 --status IN_PROGRESS
```

**When to update BrainGrid:**
- Starting work on a task → Set status to `IN_PROGRESS`
- Completing a task → Set status to `COMPLETED` and add completion notes
- Blocking a task → Set status to `BLOCKED` and document the blocker
- Creating new work → Create REQ/task if substantial feature work

### Step 5: Add to Update Log
```markdown
| Date | Agent/Person | Changes |
|------|--------------|---------|
| YYYY-MM-DD | [Your Name] | Started RAD-XXX |
```

---

## 💻 During Development

### Code Quality Requirements

1. **Follow existing patterns** - Review similar code in the codebase first
2. **Add documentation** - All public APIs must have doc comments
3. **Write tests** - New functionality should include unit tests
4. **Keep changes focused** - One task = one logical change set

### Rust-Specific Guidelines

```rust
// ✅ DO: Use proper error handling
fn process() -> Result<Data, Error> {
    let data = fetch_data()?;
    Ok(transform(data))
}

// ❌ DON'T: Use unwrap/expect in library code
fn process() -> Data {
    fetch_data().unwrap()  // Bad!
}
```

```rust
// ✅ DO: Add doc comments for public items
/// Processes the agent's input and returns the result.
///
/// # Arguments
/// * `input` - The input string to process
///
/// # Returns
/// The processed output or an error
pub fn process(input: &str) -> Result<Output, Error>
```

### File Organization

```
radium/
├── radium-core/src/
│   ├── models/       # Data structures (RAD-001, RAD-002, RAD-003)
│   ├── storage/      # Database layer (RAD-004)
│   ├── server/       # gRPC server
│   └── config/       # Configuration
├── model-abstraction/ # Model trait definitions
├── radium-models/     # Model implementations
└── agent-orchestrator/ # Agent framework
```

---

## 📝 Git Workflow

### Branch Naming
```
feat/RAD-XXX-short-description   # New features
fix/RAD-XXX-short-description    # Bug fixes
docs/RAD-XXX-short-description   # Documentation
refactor/RAD-XXX-short-description # Refactoring
```

### Commit Message Format
```
type(scope): description [RAD-XXX]

Optional body with more details.

Optional footer with breaking changes or references.
```

**Types:**
- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation
- `refactor` - Code refactoring
- `test` - Adding tests
- `chore` - Maintenance tasks

**Examples:**
```
feat(models): add Agent and AgentConfig structs [RAD-001]

fix(storage): handle connection timeout gracefully [RAD-004]

docs(api): add gRPC endpoint documentation [RAD-006]
```

### Pre-Commit Checklist

Run these commands before committing:

```bash
cd radium

# 1. Check compilation
cargo check --all-targets

# 2. Run linter
cargo clippy --all-targets -- -D warnings

# 3. Run tests (use cargo-nextest for faster execution)
cargo nextest run
# Or fallback to standard cargo test:
# cargo test

# 4. Format code (auto-fix)
cargo fmt
```

**All checks must pass before committing.**

### Commit Frequency

- **DO commit** after:
  - Completing a logical unit of work
  - Adding a new struct/trait/function that compiles
  - Fixing a bug
  - Adding tests that pass

- **DON'T commit**:
  - Code that doesn't compile
  - Code that fails tests
  - Partial implementations mid-stream

---

## ✅ After Completing Work

### Step 1: Final Checks
```bash
cargo check --all-targets
cargo clippy --all-targets -- -D warnings  
cargo test
cargo fmt --check
```

### Step 2: Commit Changes
```bash
git add -A
git commit -m "type(scope): description [RAD-XXX]"
```

### Step 3: Update Progress Files (Local + BrainGrid)

**Update PROGRESS.md:**
Move task from Active to Completed:

```markdown
## ✅ Completed Tasks

- [x] **RAD-XXX**: Task description
  - **Completed:** YYYY-MM-DD
  - **Commit:** abc1234
  - **Files:** list of files changed
  - **BrainGrid REQ:** REQ-XXX (query Braingrid for actual ID)
  - **BrainGrid Task:** TASK-X (query Braingrid for actual ID)
```

**Update BrainGrid:**
```bash
# Mark task as completed (replace TASK-X with actual task ID from Braingrid)
braingrid task update TASK-X -p PROJ-14 --status COMPLETED \
  --notes "Completed in commit abc1234. Files changed: [list]"

# If completing a requirement, update requirement status (replace REQ-XXX with actual REQ ID)
braingrid requirement update REQ-XXX -p PROJ-14 --status COMPLETED
```

**Automatic Updates:**
- When completing a task, automatically update the corresponding BrainGrid task
- Include commit hash and changed files in BrainGrid notes
- Update requirement status if all tasks are complete

### Step 4: Update Log Entry
```markdown
| YYYY-MM-DD | [Your Name] | Completed RAD-XXX |
```

### Step 5: Check for Unblocked Tasks
Review if your completion unblocks other tasks and note this.

---

## 🚧 Handling Blockers

When you encounter a blocker:

### 1. Document the Blocker
Add to PROGRESS.md:
```markdown
## 🚧 Blockers

- **BLOCKER-XXX**: Description of the issue
  - **Blocking:** RAD-001, RAD-002
  - **Owner:** [Your Name]
  - **Resolution:** Pending
  - **Notes:** Details about what's needed
```

### 2. Update Affected Tasks
```markdown
- [ ] **RAD-001**: Task description
  - **Status:** Blocked
  - **Blocked By:** BLOCKER-XXX
```

### 3. Move to Unblocked Work
Select another task that isn't blocked.

---

## 🤝 Agent Roles

- **Gemini**: Responsible for writing comprehensive unit and integration tests for all completed features. Will not work on feature implementation directly.

## 🤝 Multi-Agent Coordination

### Avoiding Conflicts

1. **Check assignments** - Never start work on an assigned task
2. **Atomic updates** - Update PROGRESS.md immediately when starting
3. **Communicate blockers** - Document immediately when discovered
4. **Respect dependencies** - Don't start tasks with unmet dependencies

### Task Dependencies

```markdown
Dependencies: RAD-001, RAD-002
```
Means: RAD-001 and RAD-002 must be ✅ complete before starting this task.

### Handoff Protocol

When you must stop work mid-task:
1. Commit any working code (even partial)
2. Update PROGRESS.md with detailed status
3. Add notes about what's done and what remains
4. Change status to "Paused" not "Not Started"

```markdown
- [ ] **RAD-XXX**: Task description
  - **Status:** Paused
  - **Assignee:** [Your Name] (paused)
  - **Notes:** Completed X and Y. Remaining: Z. See commit abc123.
```

---

## 📊 Progress Update Examples

### Starting a Task
```markdown
- [ ] **RAD-001**: Design and implement core data structures for agents
  - **Status:** In Progress
  - **Assignee:** Claude
  - **Started:** 2024-12-01
  - **Files:** `crates/radium-core/src/models/`
  - **Notes:** Need Agent, AgentConfig, AgentState structs
```

### Completing a Task
```markdown
- [x] **RAD-001**: Design and implement core data structures for agents
  - **Completed:** 2024-12-01
  - **Assignee:** Claude
  - **Commit:** feat(models): add Agent and AgentConfig structs [RAD-001]
  - **Files:** `crates/radium-core/src/models/agent.rs`
```

### Blocking a Task
```markdown
- [ ] **RAD-004**: Implement SQLite data storage layer
  - **Status:** Blocked
  - **Blocked By:** BLOCKER-001 (rusqlite version conflict)
  - **Assignee:** Unassigned
```

---

## 🔧 Tool-Specific Notes

### For Cursor/Cline
- Use the integrated terminal for running cargo commands
- Leverage file search to find existing patterns
- Use multi-file editing for related changes

### For Claude/Gemini API
- Request file contents before making changes  
- Provide complete file contents when making edits
- Use code blocks with language tags for clarity

---

## 🔄 Process Management

### Background Processes

When running long-running services (servers, watchers, etc.), prefer background execution unless actively monitoring output.

**Starting a background process:**
```bash
# Run in background and save PID
cd radium && cargo run --bin radium-core &
echo $! > .pids/radium-core.pid

# Or use nohup for persistence
nohup cargo run --bin radium-core > logs/radium-core.log 2>&1 &
echo $! > .pids/radium-core.pid
```

### PID File Management

All long-running processes should create PID files in `.pids/` directory:

```bash
# Create PID directory if needed
mkdir -p .pids

# Start process and record PID
cargo run --bin radium-core &
echo $! > .pids/radium-core.pid

# Check if process is running
if [ -f .pids/radium-core.pid ] && kill -0 $(cat .pids/radium-core.pid) 2>/dev/null; then
    echo "radium-core is running"
fi

# Stop process
kill $(cat .pids/radium-core.pid) && rm .pids/radium-core.pid
```

### When to Use Background vs Foreground

| Scenario | Mode | Reason |
|----------|------|--------|
| Running tests | Foreground | Need to see results |
| Starting dev server to test | Background | Don't need to watch output |
| Debugging server issues | Foreground | Need live output |
| Running cargo check/clippy | Foreground | Need to see errors |
| Running database migrations | Foreground | Need to verify success |
| Starting services for integration tests | Background | Just need them running |

### Log Files

When running in background, redirect output to logs:

```bash
mkdir -p logs

# Server logs
cargo run --bin radium-core > logs/radium-core.log 2>&1 &
echo $! > .pids/radium-core.pid

# View logs
tail -f logs/radium-core.log

# Check for errors
grep -i error logs/radium-core.log
```

### Cleanup Protocol

Before ending a session or switching tasks:

1. **Check for running processes:**
   ```bash
   ls .pids/
   ```

2. **Stop non-essential processes:**
   ```bash
   for pid_file in .pids/*.pid; do
       if [ -f "$pid_file" ]; then
           kill $(cat "$pid_file") 2>/dev/null
           rm "$pid_file"
       fi
   done
   ```

3. **Document still-running processes** in PROGRESS.md notes if needed

---

## 🧠 BrainGrid CLI Integration

### Essential Commands

| Action | Command |
|--------|---------|
| List requirements | `braingrid requirement list -p PROJ-14` |
| Show requirement | `braingrid requirement show REQ-XXX -p PROJ-14` (replace REQ-XXX) |
| List tasks | `braingrid task list -r REQ-XXX -p PROJ-14` (replace REQ-XXX) |
| Update task | `braingrid task update TASK-X -p PROJ-14 --status IN_PROGRESS` (replace TASK-X) |
| Create requirement | `braingrid specify -p PROJ-14 --prompt "Feature description"` |
| Project status | `braingrid status` |

### When to Use BrainGrid

**Before Starting Work:**
- Check for existing REQs related to your feature
- Review task dependencies and blockers
- Understand acceptance criteria and success metrics

**During Work:**
- Update task status when starting/completing
- Document blockers in task notes
- Link commits to tasks/REQs in commit messages

**After Completing Work:**
- Mark tasks as COMPLETED
- Add completion notes with commit hash
- Update requirement status if all tasks done

### BrainGrid Workflow

1. **Discovery Phase:**
   ```bash
   braingrid requirement list -p PROJ-14
   # Find relevant REQ, then view details (replace REQ-XXX with actual ID)
   braingrid requirement show REQ-XXX -p PROJ-14
   ```

2. **Task Selection:**
   ```bash
   # List tasks for the REQ you're working on (replace REQ-XXX with actual ID)
   braingrid task list -r REQ-XXX -p PROJ-14
   ```

3. **Status Updates:**
   ```bash
   # Starting work (replace TASK-X with actual task ID from Braingrid)
   braingrid task update TASK-X -p PROJ-14 --status IN_PROGRESS
   
   # Completing work (replace TASK-X with actual task ID)
   braingrid task update TASK-X -p PROJ-14 --status COMPLETED \
     --notes "Completed in commit abc123. Implements feature X."
   ```

4. **Creating New Requirements:**
   ```bash
   braingrid specify -p PROJ-14 --prompt "Add user authentication with OAuth2"
   ```

### Integration with Local Progress

- **Braingrid is Primary:** Braingrid is the source of truth. PROGRESS.md is a secondary summary.
- **Update Braingrid First:** Always update Braingrid task status, then update PROGRESS.md if needed
- **Reference Links:** When needed, include Braingrid REQ links in PROGRESS.md (query Braingrid for current REQ IDs)
- **Commit Messages:** Reference REQ/TASK IDs in commit messages: `[REQ-XXX]` or `[TASK-X]` (use actual IDs from Braingrid)
- **Delete Local Copies:** Once a REQ has full task breakdown in Braingrid, delete local markdown copies

## 📚 Quick Reference

| Action | Command/Location |
|--------|-----------------|
| View progress | `docs/project/PROGRESS.md` (summary) |
| **Check Braingrid (SOURCE OF TRUTH)** | `braingrid requirement list -p PROJ-14` |
| Braingrid workflow | `docs/project/BRAINGRID_WORKFLOW.md` |
| Check code | `cargo check --all-targets` |
| Run linter | `cargo clippy --all-targets -- -D warnings` |
| Run tests | `cargo test` |
| Format code | `cargo fmt` |
| Project root | `radium/` |
| BrainGrid project | `PROJ-14` |

### Status Values
- `Not Started` - Task is in queue
- `In Progress` - Actively being worked on
- `Blocked` - Cannot proceed due to blocker
- `Paused` - Work stopped, can be resumed
- `Completed` - Done and committed

### Priority Levels
- `High` - Critical path items
- `Medium` - Important but not blocking
- `Low` - Nice to have, can be deferred
