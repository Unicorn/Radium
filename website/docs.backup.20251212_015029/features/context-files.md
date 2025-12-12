# Context Files

Context files (GEMINI.md) provide a way to give persistent instructions to agents without repeating them in every prompt. They enable project-specific agent behavior customization and team-shared context files via version control.

## Overview

Context files allow you to:

- Provide persistent instructions to agents across all interactions
- Maintain consistency across multiple agent executions
- Share project-specific guidelines and constraints with your team
- Organize context hierarchically (global, project, subdirectory)
- Import and reuse context from other files

## File Locations and Precedence

Context files are automatically discovered and loaded hierarchically. The loading order determines precedence:

1. **Global context** (`~/.radium/GEMINI.md`) - Lowest precedence
   - Shared across all projects
   - Useful for personal preferences and common guidelines

2. **Project root context** (`GEMINI.md` in workspace root) - Medium precedence
   - Project-specific instructions
   - Shared with team via version control

3. **Subdirectory context** (`<subdirectory>/GEMINI.md`) - Highest precedence
   - Directory-specific instructions
   - Overrides project and global context

### Precedence Rules

- Higher precedence files override lower precedence files
- All applicable files are merged, with lower precedence content prepended to higher precedence content
- Subdirectory context takes precedence over project context, which takes precedence over global context

## File Format

Context files are plain Markdown files. You can include:

- Project guidelines and standards
- Coding conventions
- Architecture notes
- Common patterns and practices
- Any other instructions for agents

### Example: Basic Context File

```markdown
# Project Context

This project uses Rust and follows these guidelines:
- Use `cargo fmt` for formatting
- Write comprehensive tests for all public APIs
- Document all public types and functions

## Code Style

- Prefer explicit error handling over panics
- Use `anyhow::Result` for application code
- Keep functions focused and single-purpose
```

## Import Syntax

You can import other files using the `@file.md` syntax. This is useful for organizing context across multiple files.

### Basic Import

```markdown
# Project Context

@coding-standards.md
@architecture-notes.md

## Project-Specific Instructions

Additional project-specific context here...
```

### Import Resolution

- Imports are resolved relative to the file containing the import
- Supports both relative paths (`@subdir/file.md`) and absolute paths
- Circular imports are detected and reported as errors
- Duplicate imports are automatically skipped

### Example: Organized Context Files

**GEMINI.md** (project root):
```markdown
# Project Context

@docs/coding-standards.md
@docs/architecture.md

## Project Overview

This is a high-performance agent orchestration platform.
```

**docs/coding-standards.md**:
```markdown
# Coding Standards

- Use Rust's standard formatting
- Write tests for all public APIs
- Document complex algorithms
```

**docs/architecture.md**:
```markdown
# Architecture

The system uses a modular monorepo structure with:
- Core crate for business logic
- CLI crate for user interface
- TUI crate for terminal interface
```

## Integration with Agents

Context files are automatically loaded and injected into agent prompts. When you run any agent command (`rad step`, `rad run`, `rad chat`, or `rad craft`), context files are:

1. Discovered based on the current working directory (or plan directory for `rad craft`)
2. Loaded hierarchically (global → project → subdirectory)
3. Merged together with proper precedence
4. Imported files are processed and merged
5. Injected into the agent's prompt context

The context appears in the prompt under a `# Context Files` section, allowing agents to access your persistent instructions.

### Supported Commands

All of the following commands automatically load and use context files:

- **`rad step <agent-id> <prompt>`** - Single agent execution with context
- **`rad run <script>`** - Script-based agent execution with context
- **`rad chat <agent-id>`** - Interactive chat session with persistent context
- **`rad craft <plan-id>`** - Plan execution with context from plan directory

## CLI Commands

Radium provides several commands for managing context files:

### List Context Files

```bash
rad context list
```

Lists all context files found in the workspace, categorized by type (global, project, subdirectory).

**Example output:**
```
Context Files

  ✓ Found 3 context file(s)

  1. ~/.radium/GEMINI.md (global) - 2.3 KB
  2. /project/GEMINI.md (project) - 1.5 KB
  3. /project/src/GEMINI.md (subdirectory) - 0.8 KB
```

### Show Context for Path

```bash
rad context show <path>
```

Shows which context files would be loaded for a specific path, along with their loading order and a preview of the merged content.

**Example:**
```bash
rad context show src/
```

**Example output:**
```
Context Files for Path

  ✓ Context files for: src/

  Loading order (precedence: lowest to highest):

  1. Global (lowest) ~/.radium/GEMINI.md (2.3 KB)
  2. Project /project/GEMINI.md (1.5 KB)
  3. Subdirectory (highest) /project/src/GEMINI.md (0.8 KB)

  Merged content preview:
    [First 10 lines of merged content]
```

### Validate Context Files

```bash
rad context validate
```

Validates all context files in the workspace, checking for:
- Readability
- Valid import syntax
- Circular import detection
- Missing import files
- Empty files (reported as warnings)

**Example output:**
```
Validating Context Files

  • Validating 3 context file(s)...

  ✓ All context files are valid!
```

**Error output:**
```
Validating Context Files

  • Validating 2 context file(s)...

  ✗ Found 1 error(s):

    /project/GEMINI.md Import error: Circular import detected: /project/file2.md

  ! Found 1 warning(s):

    /project/src/GEMINI.md File is empty
```

### Initialize Context File

```bash
rad context init [OPTIONS]
```

Creates a new context file from a template. This is the easiest way to get started with context files.

**Options:**
- `--template <TYPE>` - Template type to use (default: `basic`)
  - Available templates: `basic`, `coding-standards`, `architecture`, `team-conventions`
- `--global` - Create global context file (`~/.radium/GEMINI.md`) instead of project file
- `--path <PATH>` - Custom path for the context file (defaults to project root)

**Examples:**

Create a basic project context file:
```bash
rad context init
```

Create a coding standards template:
```bash
rad context init --template coding-standards
```

Create a global context file:
```bash
rad context init --global
```

Create a context file at a custom path:
```bash
rad context init --path docs/CONTEXT.md
```

**Template Types:**

1. **basic** - Simple project context with common sections (default)
   - Project overview
   - Guidelines
   - Code style
   - Testing
   - Documentation

2. **coding-standards** - Code style and conventions template
   - Language-specific standards
   - Code formatting
   - Naming conventions
   - Code organization
   - Best practices
   - Anti-patterns

3. **architecture** - Architecture documentation template
   - System overview
   - Components
   - Data flow
   - Design decisions
   - Dependencies
   - Future considerations

4. **team-conventions** - Team-specific guidelines template
   - Communication standards
   - Development workflow
   - Code review standards
   - Documentation standards
   - Tooling preferences
   - Onboarding information

**Example output:**
```
Initialize Context File

  ✓ Created context file: /project/GEMINI.md
  • Template: basic
  • Description: Simple project context with common sections

  Next steps:
    • Edit the file to customize it for your project
    • Use `rad context validate` to check for issues
    • See `docs/features/context-files.md` for more information
```

**Overwrite Protection:**

If a context file already exists at the target location, the command will prompt before overwriting:
```
  ! File already exists: /project/GEMINI.md
  Overwrite? (y/N):
```

## Best Practices

### Organization

1. **Keep project root context focused**: Include only project-specific guidelines in the root `GEMINI.md`
2. **Use imports for organization**: Break large context files into smaller, focused files
3. **Use subdirectories sparingly**: Only create subdirectory context files when truly needed

### Content Guidelines

1. **Be specific**: Provide clear, actionable instructions
2. **Keep it relevant**: Focus on information agents need to perform tasks
3. **Update regularly**: Keep context files in sync with project changes
4. **Use markdown structure**: Organize content with headers and lists for clarity

### Import Management

1. **Avoid circular imports**: Structure imports in a tree or DAG pattern
2. **Use descriptive names**: Name imported files clearly (e.g., `coding-standards.md`)
3. **Keep imports relative**: Use relative paths when possible for portability

## Troubleshooting

### Context Files Not Loading

**Problem**: Agents don't seem to receive context from your files.

**Solutions**:
- Verify file is named `GEMINI.md` (case-sensitive)
- Check file location (project root, subdirectory, or `~/.radium/`)
- Use `rad context list` to see if files are discovered
- Check current working directory matches expected location

### Import Errors

**Problem**: `rad context validate` reports import errors.

**Solutions**:
- Verify imported files exist at the specified path
- Check for circular imports using the validate command
- Ensure import paths are relative to the file containing the import
- Use absolute paths if relative resolution is unclear

### Precedence Not Working

**Problem**: Expected context file isn't overriding another.

**Solutions**:
- Remember: subdirectory > project > global precedence
- Check that the file exists at the expected location
- Use `rad context show <path>` to see which files are loaded
- Verify file names are exactly `GEMINI.md` (no typos)

### Context Too Long

**Problem**: Context files are making prompts too long.

**Solutions**:
- Break large files into smaller, focused files using imports
- Use subdirectory context files only where needed
- Consider if all context is necessary for every agent execution
- Review and remove outdated or redundant instructions

## Examples

See the [examples directory](../../examples/context-files/) for complete, working examples of:

- Basic project context files
- Context files with imports
- Subdirectory-specific context
- Real-world use cases

## References

- [Context Files Implementation](../../crates/radium-core/src/context/files.rs) - Technical implementation details
- [Context Manager Integration](../../crates/radium-core/src/context/manager.rs) - How context is integrated into prompts
- [Agent Configuration Guide](../user-guide/agent-configuration.md) - Configuring agents that use context

