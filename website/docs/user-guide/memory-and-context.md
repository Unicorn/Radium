---
id: "memory-and-context"
title: "Memory & Context System"
sidebar_label: "Memory & Context System"
---

# Memory & Context System

The Memory & Context System enables agents to maintain continuity across executions and access relevant information for their tasks. This system provides plan-scoped memory storage, context gathering from multiple sources, and conversation history tracking.

## Overview

The Memory & Context System consists of several key components:

- **Memory Store**: Persists agent outputs per plan for later retrieval
- **Context Manager**: Gathers and builds comprehensive context from multiple sources
- **Context Sources**: Support for file, HTTP, Jira, and Braingrid sources
- **Conversation History**: Tracks and summarizes interaction history
- **Custom Commands**: Reusable command definitions with template substitution

## Memory Store

The memory store automatically persists agent outputs during plan execution. Each agent's output is stored and can be retrieved by subsequent agents for context.

### Memory Storage Location

Memory entries are stored in plan-scoped directories:
```
.radium/plan/REQ-XXX/memory/<agent-id>.json
```

### Automatic Storage

When executing plans with `rad craft`, agent outputs are automatically stored:

1. Agent executes and produces output
2. Output is truncated to last 2000 characters
3. Entry is stored in memory store for the plan's requirement ID
4. Subsequent agents can access previous outputs via ContextManager

### Memory Entry Structure

Each memory entry contains:
- **agent_id**: Identifier of the agent that produced the output
- **output**: Last 2000 characters of agent output
- **timestamp**: When the entry was created
- **metadata**: Optional key-value metadata

### Example: Using Memory in Plan Execution

```bash
# Execute a plan - memory is automatically stored
rad craft REQ-69

# Agent outputs are stored in:
# .radium/plan/REQ-69/memory/plan-agent.json
# .radium/plan/REQ-69/memory/code-agent.json
# .radium/plan/REQ-69/memory/review-agent.json
```

## Context Manager

The Context Manager gathers context from multiple sources and builds comprehensive context strings for agent prompts.

### Context Sources

The Context Manager automatically gathers context from:

1. **Context Files**: Hierarchical GEMINI.md files (see [Context Files](../features/context-files.md))
2. **Plan Context**: Information about the current plan (requirement ID, status, path)
3. **Memory Context**: Previous agent outputs from the memory store
4. **Architecture Context**: `.radium/architecture.md` if present
5. **Learning Context**: Past mistakes and strategies from the learning system
6. **External Sources**: HTTP, Jira, Braingrid sources (see [Context Sources](context-sources.md))

### Building Context

Context is automatically built when using ContextManager:

```rust
let manager = ContextManager::for_plan(&workspace, req_id)?;
let context = manager.build_context("agent-name[input:file.md]", Some(req_id))?;
```

The context string includes all available sources in order of precedence:
1. Context files (highest precedence)
2. Plan context
3. Architecture context
4. Memory context
5. Learning context
6. File injection content

### Memory Context Retrieval

To retrieve memory context for a specific agent:

```rust
let memory_context = manager.gather_memory_context("plan-agent")?;
```

This returns the last 2000 characters of the agent's output if available.

## Context Injection Syntax

Agents can specify context injection using special syntax in their invocation.

### File Injection

Inject file contents into the prompt:

```bash
rad step code-agent[input:spec.md,requirements.md]
```

Multiple files can be specified, separated by commas. Files are resolved relative to the workspace root.

### Tail Context

Inject the last N lines from a previous agent's output:

```bash
rad step review-agent[tail:50]
```

This retrieves the last 50 lines from the agent's previous output stored in memory.

### Combined Injection

Multiple injection types can be combined:

```bash
rad step agent[input:file1.md,file2.md][tail:100]
```

## Context Files

Context files (GEMINI.md) provide persistent instructions to agents. They are loaded hierarchically with three levels of precedence:

1. **Global**: `~/.radium/GEMINI.md` (lowest precedence)
2. **Project**: `./GEMINI.md` in workspace root
3. **Subdirectory**: `./subdirectory/GEMINI.md` (highest precedence)

See [Context Files](../features/context-files.md) for detailed documentation.

## Conversation History

Conversation history tracks interactions per session and prevents context window bloat through summarization.

### Session-Based Tracking

Each chat session has a unique session ID. History is tracked per session:

```
.radium/_internals/sessions/<session-id>/history.json
```

### History Summarization

To prevent context window bloat, only the last 5 interactions are included in summaries. The history manager automatically:

- Tracks all interactions (up to 10 per session)
- Provides summaries containing last 5 interactions
- Maintains session isolation

### Using History in Chat

When using `rad chat`, history is automatically tracked:

```bash
rad chat code-agent
# Session history is automatically maintained and included in prompts
```

### Retrieving History

```rust
let history = HistoryManager::new(&history_dir)?;
let summary = history.get_summary(Some("session-id"));
```

## Custom Commands

Custom commands allow you to define reusable operations using TOML configuration files.

### Command Definition

Commands are defined in `.radium/commands/*.toml`:

```toml
[command]
name = "build"
description = "Build the project"
template = "!{cargo build} {{args}}"
```

### Template Substitution

Commands support three types of template substitution:

1. **Arguments**: `{{args}}` or `{{arg1}}` - Replaced with command arguments
2. **Shell Commands**: `!{command}` - Executed and replaced with output
3. **File Contents**: `@{file}` - Replaced with file contents

### Command Execution

Commands can be executed via the `rad custom` command or programmatically:

```rust
let command = CustomCommand::load("build")?;
let output = command.execute(&["--release"], &workspace_root)?;
```

### Sandbox Execution

When configured, commands execute within a sandbox for security:

```rust
let mut sandbox = Sandbox::new(&config)?;
let output = command.execute_with_sandbox(&args, &workspace_root, Some(&mut sandbox))?;
```

### Hook Integration

Commands can be approved or denied by hooks before execution:

```rust
let output = command.execute_with_hooks(&args, &workspace_root, Some(hook_registry)).await?;
```

See [Custom Commands](custom-commands.md) for detailed documentation.

## Best Practices

### Memory Management

- Memory entries are automatically truncated to 2000 characters
- Memory is scoped per requirement ID for isolation
- Memory persists across plan execution restarts

### Context Organization

- Use hierarchical context files for organization
- Place project-wide context in `./GEMINI.md`
- Use subdirectory context files for specific areas
- Import common patterns using `@file.md` syntax

### Performance

- Context files are cached for performance
- Large context files may impact token usage
- Use history summarization to manage context window size

## Troubleshooting

### Memory Not Persisting

If agent outputs aren't being stored:
1. Verify the workspace is initialized (`rad init`)
2. Check that the plan has a valid requirement ID
3. Ensure `.radium/plan/REQ-XXX/memory/` directory exists

### Context Not Loading

If context isn't being gathered:
1. Verify context files exist in expected locations
2. Check file permissions and accessibility
3. Verify external sources (HTTP, Jira) are reachable
4. Check ContextManager is initialized with `for_plan()`

### History Not Tracking

If conversation history isn't working:
1. Verify `.radium/_internals/sessions/` directory exists
2. Check session ID is being passed correctly
3. Ensure HistoryManager is initialized with correct path

## API Reference

### MemoryStore

```rust
// Create memory store for a plan
let mut store = MemoryStore::new(&workspace_root, req_id)?;

// Store agent output
let entry = MemoryEntry::new("agent-id".to_string(), "output".to_string());
store.store(entry)?;

// Retrieve agent output
let entry = store.get("agent-id")?;

// List all agents with stored memory
let agents = store.list_agents();
```

### ContextManager

```rust
// Create context manager for a plan
let mut manager = ContextManager::for_plan(&workspace, req_id)?;

// Build comprehensive context
let context = manager.build_context("agent[input:file.md]", Some(req_id))?;

// Gather specific context types
let plan_ctx = manager.gather_plan_context(req_id)?;
let mem_ctx = manager.gather_memory_context("agent-id")?;
let arch_ctx = manager.gather_architecture_context();
```

### HistoryManager

```rust
// Create history manager
let mut history = HistoryManager::new(&history_dir)?;

// Add interaction
history.add_interaction(
    Some("session-id"),
    "goal".to_string(),
    "plan".to_string(),
    "output".to_string(),
)?;

// Get summary (last 5 interactions)
let summary = history.get_summary(Some("session-id"));

// Get all interactions
let interactions = history.get_interactions(Some("session-id"));
```

