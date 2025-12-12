# Agent Development Instructions

This document contains core instructions for all agents working on the Radium project. These guidelines ensure consistent, high-quality development practices and proper integration with our project management tools.

## Braingrid Integration

Braingrid is our optional source of truth for feature requirements (REQs) and tasks. All agents should integrate with Braingrid when working on feature development.

### Core Principles

- **Braingrid is accessed via CLI** - Always use the `braingrid` command-line tool, never assume direct API access
- **REQ-XXX references** - When users reference `REQ-XXX`, they are referring to a Braingrid feature requirement
- **Task-driven development** - All work should be broken down into actionable tasks within Braingrid

### Working with Requirements (REQs)

#### Before Starting Work

1. **Check for existing REQs** - When a user references an REQ-XXX:
   ```bash
   braingrid requirement show REQ-XXX -p PROJ-14
   braingrid task list -r REQ-XXX -p PROJ-14
   ```

2. **Validate task breakdown** - Ensure the REQ has robust sub-tasks describing the implementation:
   - If tasks are missing or insufficient, run:
     ```bash
     braingrid requirement update REQ-XXX -p PROJ-14 --action "Update requirement with latest commits and break requirement into tasks"
     ```
   - All REQs must include either:
     - Unit test creation tasks, OR
     - Manual testing tasks
   - Verify tasks cover all aspects of the requirement

3. **Set REQ status to In Progress** - When starting work on an REQ:
   ```bash
   braingrid requirement update REQ-XXX -p PROJ-14 --status IN_PROGRESS
   ```

#### During Development

1. **Update task status in real-time** - As you work on tasks:
   - When starting a task: `braingrid task update TASK-X -p PROJ-14 --status IN_PROGRESS`
   - When completing a task: `braingrid task update TASK-X -p PROJ-14 --status COMPLETED --notes "Completed in commit [hash]. Implements [feature]."`
   - Update progress regularly, not just at the end

2. **Reference REQ/TASK in commits** - Include identifiers in commit messages:
   ```
   [REQ-XXX] [TASK-X] Brief description of changes
   ```

3. **Link code to requirements** - Reference REQ-XXX in code comments when relevant:
   ```rust
   // Implements REQ-XXX: Feature description
   ```

#### Completing Work

1. **Verify all tasks are complete** - Before marking REQ as complete:
   ```bash
   braingrid task list -r REQ-XXX -p PROJ-14
   ```
   - Ensure all tasks show `COMPLETED` status
   - Verify test tasks (unit or manual) are included and completed

2. **Run relevant tests** - Execute all tests related to the REQ:
   - Unit tests: `cargo test` or appropriate test command
   - Integration tests: Run full test suite
   - Manual testing: Follow test plan if applicable

3. **Mark REQ as Review** - Once all tasks are complete and tests pass:
   ```bash
   braingrid requirement update REQ-XXX -p PROJ-14 --status REVIEW
   ```

4. **Final commit** - Create a final commit summarizing the work:
   ```
   [REQ-XXX] Complete implementation

   - All tasks completed
   - Tests passing
   - Ready for review
   ```

### Creating New Requirements

When substantial new features are needed:

1. **Use Braingrid to specify** - Create a new requirement:
   ```bash
   braingrid specify -p PROJ-14 "Feature description"
   ```

2. **Break into tasks** - Ensure the requirement is properly decomposed:
   ```bash
   braingrid requirement update REQ-XXX -p PROJ-14 --action "Break requirement into tasks"
   ```

3. **Include testing** - Always add test tasks (unit or manual) to the requirement

## Git Workflow Best Practices

### Commit Messages

- **Format**: `[REQ-XXX] [TASK-X] Brief description`
- **Include context**: Reference what changed and why
- **Link to Braingrid**: Always include REQ/TASK identifiers when applicable

### Branch Naming

- **Feature branches**: `feature/REQ-XXX-short-description`
- **Bug fixes**: `fix/REQ-XXX-short-description`
- **Refactoring**: `refactor/REQ-XXX-short-description`

### Commit Frequency

- **Commit often**: Make small, logical commits as you complete sub-tasks
- **Update Braingrid**: After each commit that completes a task, update the task status
- **Sync status**: Keep Braingrid task status in sync with actual progress

## Development Workflow

### Standard Development Cycle

1. **Discovery**
   - Check Braingrid for related REQs
   - Review existing tasks and dependencies
   - Understand acceptance criteria

2. **Planning**
   - Ensure tasks are well-defined
   - Break down work if needed
   - Identify test requirements

3. **Implementation**
   - Update task status to IN_PROGRESS
   - Write code following project standards
   - Commit frequently with proper messages
   - Update task status as work progresses

4. **Testing**
   - Write/run unit tests
   - Perform manual testing if required
   - Verify all acceptance criteria

5. **Completion**
   - Mark all tasks as COMPLETED
   - Update REQ status to REVIEW
   - Create summary commit
   - Ensure all tests pass

### Code Quality Standards

- **Follow language conventions**: Rust, TypeScript, etc. style guides
- **Write tests**: Aim for >80% code coverage
- **Document code**: Add docstrings and comments for complex logic
- **Handle errors**: Explicit error handling, never silently ignore failures
- **Keep it simple**: YAGNI principle - don't over-engineer

## Common Commands Reference

### Braingrid CLI

```bash
# List requirements
braingrid requirement list -p PROJ-14

# Show requirement details
braingrid requirement show REQ-XXX -p PROJ-14

# Update requirement status
braingrid requirement update REQ-XXX -p PROJ-14 --status IN_PROGRESS
braingrid requirement update REQ-XXX -p PROJ-14 --status REVIEW

# Break requirement into tasks
braingrid requirement update REQ-XXX -p PROJ-14 --action "Update requirement with latest commits and break requirement into tasks"

# List tasks for a requirement
braingrid task list -r REQ-XXX -p PROJ-14

# Update task status
braingrid task update TASK-X -p PROJ-14 --status IN_PROGRESS
braingrid task update TASK-X -p PROJ-14 --status COMPLETED --notes "Description of completion"

# Create new requirement
braingrid specify -p PROJ-14 "Feature description"
```

### Testing

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p radium-core

# Run with coverage
cargo test --all-features
```

## Notes

- **Project ID**: The project ID `PROJ-14` is used throughout - adjust if your project uses a different ID
- **Real-time updates**: Update Braingrid status as you work, not just at the end
- **Test coverage**: Every REQ must have associated test tasks (unit or manual)
- **Status flow**: PENDING → IN_PROGRESS → COMPLETED (tasks) / REVIEW (REQ) → DONE

