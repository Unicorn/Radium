# Context Files Examples

This directory contains example context files demonstrating different patterns and use cases for GEMINI.md files in Radium.

## Examples

### Basic Project Context

**File**: `basic-project.md`

A simple, self-contained context file with project guidelines and standards. This is a good starting point for new projects.

**Use Case**: Small projects or teams that want a single, straightforward context file.

**CLI Commands**:
```bash
# View context files
rad context list

# Validate context file
rad context validate
```

### Context with Imports

**Files**: `with-imports.md`, `coding-standards.md`, `architecture-notes.md`

Demonstrates how to organize context across multiple files using the `@file.md` import syntax. The main context file imports supporting files for better organization.

**Use Case**: Large projects that want to organize context into focused, reusable modules.

**CLI Commands**:
```bash
# Show which files would be loaded
rad context show .

# Validate imports
rad context validate
```

### Subdirectory Context

**File**: `subdirectory-example.md`

Shows how to create directory-specific context files that override or extend project-level context. This is useful for modules or features that need specialized instructions.

**Use Case**: Projects with modules that have different requirements than the project standard.

**CLI Commands**:
```bash
# See hierarchical loading
rad context show src/api

# List all context files
rad context list
```

### Hierarchical Context

**File**: `hierarchical-example.md`

Demonstrates how context files work together across multiple levels: global (`~/.radium/GEMINI.md`), project root, and subdirectory. Shows precedence and merging behavior.

**Use Case**: Understanding how context files interact at different levels.

**CLI Commands**:
```bash
# See loading order and precedence
rad context show src/api

# Discover all context files
rad context list
```

### Language-Specific Context

**Files**: `rust-project.md`, `typescript-project.md`

Examples of language-specific context files that provide guidelines tailored to a particular programming language and ecosystem.

**Use Case**: Projects using specific languages that want language-specific guidelines.

**CLI Commands**:
```bash
# Initialize with template
rad context init --template coding-standards

# Validate context
rad context validate
```

### Team Collaboration

**File**: `team-collaboration.md`

Demonstrates best practices for using context files in a team environment, including what to commit to version control and how to handle personal preferences.

**Use Case**: Teams that want to share context files via version control while allowing personal customization.

**CLI Commands**:
```bash
# List all context files (global, project, subdirectory)
rad context list

# Validate team context files
rad context validate
```

## Usage

To use these examples:

1. Copy the relevant example file(s) to your project
2. Rename to `GEMINI.md` if using as a project root context file
3. Customize the content to match your project's needs
4. If using imports, ensure imported files are in the correct relative locations
5. Use `rad context validate` to check for issues

## Example Scenarios

### Scenario 1: Starting a New Project

1. Copy `basic-project.md` to your project root as `GEMINI.md`
2. Customize with your project's specific guidelines
3. Run `rad context validate` to check for issues

### Scenario 2: Organizing Large Project Context

1. Create separate files for different concerns (e.g., `coding-standards.md`, `architecture.md`)
2. Create main `GEMINI.md` that imports these files using `@file.md` syntax
3. Use `rad context show .` to verify imports work correctly

### Scenario 3: Module-Specific Context

1. Create `GEMINI.md` in a subdirectory (e.g., `src/api/GEMINI.md`)
2. Add module-specific guidelines
3. Use `rad context show src/api` to see how it merges with project context

### Scenario 4: Team Setup

1. Create project root `GEMINI.md` with team standards (commit to git)
2. Each team member can create `~/.radium/GEMINI.md` for personal preferences
3. Use `rad context list` to see all context files

## Templates

See the `templates/` directory for starter templates that can be used when initializing new workspaces with `rad init --with-context`.

You can also use `rad context init` to create context files from templates:

```bash
# Create basic context file
rad context init

# Create with specific template
rad context init --template coding-standards

# Create global context file
rad context init --global
```

