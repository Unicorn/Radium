---
req_id: REQ-011
title: Context Files
phase: NEXT
status: Review
priority: High
estimated_effort: 3-4 hours
dependencies: [REQ-002, REQ-006]
related_docs:
  - docs/features/gemini-cli-enhancements.md#context-files-geminimd
  - docs/project/03-implementation-plan.md#step-1-agent-configuration-system
---

# Context Files

## Problem Statement

Users need a way to provide persistent instructions to agents without repeating them in every prompt. Without context files, users must:
- Include the same instructions in every agent prompt
- Maintain consistency across multiple agent interactions
- Duplicate project-specific guidelines and constraints
- Manually inject context for each agent execution

The legacy system and modern AI tools (like gemini-cli) use hierarchical context files (GEMINI.md) to provide persistent instructions. Radium needs an equivalent system that supports hierarchical loading and context imports.

## Solution Overview

Implement a hierarchical context file system that provides:
- Hierarchical loading (global → project root → subdirectory)
- Automatic context file discovery
- Context imports with `@file.md` syntax
- Customizable context file names
- Memory management commands
- Integration with prompt system

The context files system enables project-specific agent behavior customization, persistent instructions without repetition, and team-shared context files via version control.

## Functional Requirements

### FR-1: Hierarchical Context Loading

**Description**: Load context files from multiple locations with precedence.

**Acceptance Criteria**:
- [x] Hierarchical loading order: global → project root → subdirectory
- [x] Context file discovery and scanning
- [x] Precedence resolution (subdirectory overrides project, project overrides global)
- [x] Context file merging
- [x] Custom context file name configuration

**Implementation**: `crates/radium-core/src/context/files.rs`

### FR-2: Context File Discovery

**Description**: Automatically discover context files in workspace.

**Acceptance Criteria**:
- [x] Automatic context file discovery
- [x] Default file name: `GEMINI.md`
- [x] Custom file name configuration
- [x] Recursive directory scanning
- [x] Context file validation

**Implementation**: `crates/radium-core/src/context/files.rs`

### FR-3: Context Imports

**Description**: Import other context files using `@file.md` syntax.

**Acceptance Criteria**:
- [x] Context import syntax: `@file.md`
- [x] Import resolution and processing
- [x] Circular import detection
- [x] Import path resolution (relative and absolute)
- [x] Import content merging

**Implementation**: `crates/radium-core/src/context/files.rs`

### FR-4: Integration with Prompt System

**Description**: Integrate context files into prompt processing.

**Acceptance Criteria**:
- [x] Context file content injection into prompts
- [x] Integration with ContextManager
- [x] Context file precedence in context building
- [x] Context file caching
- [x] Context file change detection

**Implementation**: 
- `crates/radium-core/src/context/files.rs`
- `crates/radium-core/src/context/manager.rs` (extended)

## Technical Requirements

### TR-1: Context File Format

**Description**: Markdown format for context files.

**Format**: Markdown files with optional frontmatter

**Example**:
```markdown
# Project Context

This project uses Rust and follows these guidelines:
- Use `cargo fmt` for formatting
- Write comprehensive tests
- Document all public APIs

## Architecture

The system uses a modular architecture with:
- Core crate for business logic
- CLI crate for user interface
- TUI crate for terminal interface
```

### TR-2: Context File Loading API

**Description**: APIs for loading and processing context files.

**APIs**:
```rust
pub struct ContextFileLoader {
    workspace_root: PathBuf,
    custom_file_name: Option<String>,
}

impl ContextFileLoader {
    pub fn load_hierarchical(&self, path: &Path) -> Result<String>;
    pub fn discover_context_files(&self) -> Result<Vec<PathBuf>>;
    pub fn process_imports(&self, content: &str) -> Result<String>;
}
```

### TR-3: Context File Precedence

**Description**: Precedence order for context file loading.

**Precedence** (highest to lowest):
1. Subdirectory context file (e.g., `src/api/GEMINI.md`)
2. Project root context file (e.g., `GEMINI.md`)
3. Global context file (e.g., `~/.radium/GEMINI.md`)

**Merging**: Lower precedence files are prepended to higher precedence files

## User Experience

### UX-1: Context File Creation

**Description**: Users create context files in their project.

**Example**:
```markdown
# GEMINI.md (project root)
This project uses TypeScript and React.
Follow the coding standards in CONTRIBUTING.md.

@CONTRIBUTING.md
```

### UX-2: Automatic Context Loading

**Description**: Context files are automatically loaded for agents.

**Example**:
```bash
$ rad step code-agent
# Agent automatically receives context from GEMINI.md files
```

## Data Requirements

### DR-1: Context Files

**Description**: Markdown files containing persistent instructions.

**Location**: 
- Global: `~/.radium/GEMINI.md`
- Project: `GEMINI.md` (project root)
- Subdirectory: `<subdir>/GEMINI.md`

**Format**: Markdown with optional imports

## Dependencies

- **REQ-002**: Agent Configuration - Required for agent system
- **REQ-006**: Memory & Context System - Required for context management

## Success Criteria

1. [x] Context files can be loaded hierarchically
2. [x] Context file discovery works automatically
3. [x] Context imports are processed correctly
4. [x] Context files are integrated into prompt processing
5. [x] Precedence resolution works correctly
6. [x] All context file operations have comprehensive test coverage

**Completion Metrics**:
- **Status**: Review
- **Estimated Effort**: 3-4 hours
- **Priority**: High
- **Completed**: 2025-01-XX

## Out of Scope

- Advanced context merging strategies (future enhancement)
- Context file versioning (future enhancement)
- Context file templates (future enhancement)

## References

- [Gemini CLI Enhancements](../features/gemini-cli-enhancements.md#context-files-geminimd)
- [Implementation Plan](../project/03-implementation-plan.md#step-1-agent-configuration-system)
- [Context Files Documentation](https://geminicli.com/docs/cli/gemini-md)

