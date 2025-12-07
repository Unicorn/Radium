---
req_id: REQ-012
title: Custom Commands
phase: NEXT
status: Completed
priority: High
estimated_effort: 5-6 hours
dependencies: [REQ-006]
related_docs:
  - docs/features/gemini-cli-enhancements.md#custom-commands-toml-based
  - docs/project/03-implementation-plan.md#step-5-memory--context-system
---

# Custom Commands

## Problem Statement

Users need reusable command shortcuts for common operations. Without custom commands, users must:
- Repeat complex command sequences manually
- Remember long command syntax
- Duplicate command definitions across projects
- Manually inject shell commands and file contents into prompts

The legacy system and modern AI tools (like gemini-cli) provide TOML-based custom commands. Radium needs an equivalent system that supports command discovery, shell/file injection, and argument handling.

## Solution Overview

Implement a TOML-based custom commands system that provides:
- TOML-based command definitions
- Command discovery (user vs project precedence)
- Shell command injection (`!{command}`)
- File content injection (`@{file}`)
- Argument handling (`{{args}}`, `{{arg1}}`)
- Namespaced commands via directory structure

The custom commands system enables reusable command shortcuts, dynamic command execution with shell/file injection, and improved developer productivity.

## Functional Requirements

### FR-1: Custom Command Definitions

**Description**: TOML-based command definitions with shell and file injection.

**Acceptance Criteria**:
- [x] TOML-based command format
- [x] Command name and description
- [x] Shell command injection syntax: `!{command}`
- [x] File content injection syntax: `@{file}`
- [x] Argument handling: `{{args}}`, `{{arg1}}`, etc.
- [x] Command validation

**Implementation**: `crates/radium-core/src/commands/custom.rs`

### FR-2: Command Discovery

**Description**: Discover commands from multiple locations with precedence.

**Acceptance Criteria**:
- [x] Command discovery from user directory (`~/.radium/commands/`)
- [x] Command discovery from project directory (`.radium/commands/`)
- [x] User commands override project commands
- [x] Namespaced commands via directory structure
- [x] Command registry building

**Implementation**: `crates/radium-core/src/commands/custom.rs`

### FR-3: Command Execution

**Description**: Execute custom commands with argument substitution.

**Acceptance Criteria**:
- [x] Command execution with argument substitution
- [x] Shell command execution
- [x] File content injection
- [x] Argument parsing and substitution
- [x] Error handling

**Implementation**: `crates/radium-core/src/commands/custom.rs`

## Technical Requirements

### TR-1: Custom Command Format

**Description**: TOML format for custom command definitions.

**TOML Format**:
```toml
[command]
name = "test-command"
description = "Run tests with coverage"
shell = "!{cargo test --coverage {{args}}}"
file = "@{test-results.md}"
```

**Data Models**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomCommand {
    pub name: String,
    pub description: String,
    pub shell: Option<String>,  // Shell command with !{command} syntax
    pub file: Option<String>,   // File content with @{file} syntax
    pub args: Option<Vec<String>>,  // Argument definitions
}
```

### TR-2: Command Discovery API

**Description**: APIs for discovering and loading custom commands.

**APIs**:
```rust
pub struct CustomCommandDiscovery {
    user_commands_dir: PathBuf,
    project_commands_dir: PathBuf,
}

impl CustomCommandDiscovery {
    pub fn discover_all(&self) -> Result<HashMap<String, CustomCommand>>;
    pub fn get_command(&self, name: &str) -> Result<Option<CustomCommand>>;
}
```

### TR-3: Command Execution API

**Description**: APIs for executing custom commands.

**APIs**:
```rust
pub struct CustomCommandExecutor;

impl CustomCommandExecutor {
    pub fn execute(&self, command: &CustomCommand, args: &[String]) -> Result<String>;
    pub fn process_shell(&self, shell: &str, args: &[String]) -> Result<String>;
    pub fn process_file(&self, file: &str) -> Result<String>;
}
```

## User Experience

### UX-1: Custom Command Definition

**Description**: Users create custom commands in TOML files.

**Example**:
```toml
# .radium/commands/test.toml
[command]
name = "test"
description = "Run tests with coverage"
shell = "!{cargo test --coverage {{args}}}"
```

### UX-2: Command Execution

**Description**: Users execute custom commands with arguments.

**Example**:
```bash
$ rad run test --verbose
# Executes: cargo test --coverage --verbose
```

## Data Requirements

### DR-1: Custom Command Files

**Description**: TOML files containing custom command definitions.

**Location**: 
- User: `~/.radium/commands/*.toml`
- Project: `.radium/commands/*.toml`

**Schema**: See TR-1 Custom Command Format

## Dependencies

- **REQ-006**: Memory & Context System - Required for context management and file injection

## Success Criteria

1. [x] Custom commands can be defined in TOML format
2. [x] Commands can be discovered from user and project directories
3. [x] Shell command injection works correctly
4. [x] File content injection works correctly
5. [x] Argument substitution works correctly
6. [x] Command precedence (user over project) works correctly
7. [x] All custom command operations have comprehensive test coverage (8+ tests)

**Completion Metrics**:
- **Status**: ✅ Complete
- **Test Coverage**: 8+ passing tests
- **Lines of Code**: ~430 lines
- **Implementation**: Full custom commands system
- **Files**: 
  - `crates/radium-core/src/commands/custom.rs`

## Out of Scope

- Advanced command composition (future enhancement)
- Command templates (future enhancement)
- Command marketplace (future enhancement)

## References

- [Gemini CLI Enhancements](../features/gemini-cli-enhancements.md#custom-commands-toml-based)
- [Implementation Plan](../project/03-implementation-plan.md#step-5-memory--context-system)
- [Custom Commands Implementation](../../crates/radium-core/src/commands/custom.rs)

