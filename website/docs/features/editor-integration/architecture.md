---
id: "architecture"
title: "Editor Integration Architecture"
sidebar_label: "Editor Integration Architecture"
---

# Editor Integration Architecture

Technical architecture and implementation details for Radium's editor integration system.

## Overview

The editor integration system leverages Radium's extension infrastructure and hook system to provide seamless bidirectional code exchange between editors and Radium agents.

## Architecture Components

### 1. Extension System

All editor integrations are implemented as Radium extensions:

- **radium-nvim**: Neovim Lua plugin
- **radium-vscode**: VS Code TypeScript extension
- **Clipboard mode**: CLI commands (no extension needed)

Extensions are installed via:
```bash
rad extension install <extension-name>
```

### 2. Hook System

Editor integrations use two key hook types:

#### BeforeTool Hook

**Purpose:** Inject editor context before tool execution

**Hook:** `editor-context` (BeforeTool, priority 150)

**Context Data:**
```json
{
  "file_path": "/path/to/file.rs",
  "language": "rust",
  "selection": "selected code",
  "surrounding_lines": "context above\n---\ncontext below"
}
```

**Implementation:**
- Reads environment variables set by editor
- Merges context into tool execution context
- Passes enriched context to agent

#### AfterTool Hook

**Purpose:** Process agent output for editor application

**Hook:** `code-apply` (AfterTool, priority 100)

**Processing:**
- Extracts markdown code blocks
- Structures output with metadata
- Formats for editor consumption

**Output Format:**
```json
{
  "code_blocks": [
    {
      "language": "rust",
      "content": "fn improved() {}",
      "index": 0
    }
  ],
  "block_count": 1
}
```

### 3. CLI Communication

Editor extensions communicate with Radium via CLI commands:

- **`rad step <agent-id>`**: Execute single agent step
- **`rad chat <agent-id>`**: Interactive chat session

**Communication Protocol:**
- Input: Context JSON via stdin
- Output: Agent response via stdout
- Environment: Context variables for hooks

### 4. Data Flow

```
Editor → Extension → CLI Command → Hook (BeforeTool) → Agent → Hook (AfterTool) → CLI → Extension → Editor
```

**Detailed Flow:**

1. **User Action** (select code, run command)
2. **Extension** extracts context:
   - File path
   - Language
   - Selection
   - Surrounding lines
3. **Extension** sets environment variables for hooks
4. **Extension** executes `rad step` with context JSON
5. **BeforeTool Hook** enriches context
6. **Agent** processes with enriched context
7. **AfterTool Hook** extracts code blocks
8. **CLI** returns structured output
9. **Extension** displays results
10. **User** applies changes via diff preview

## Extension Structure

### Neovim Extension (radium-nvim)

```
radium-nvim/
├── radium-extension.json    # Extension manifest
├── hooks/
│   ├── editor-context.toml  # BeforeTool hook config
│   └── code-apply.toml      # AfterTool hook config
├── plugin/
│   ├── radium.lua           # Main plugin entry
│   └── radium/
│       ├── commands.lua     # Command implementations
│       ├── utils.lua        # Utilities
│       └── diff.lua         # Diff functionality
└── tests/                   # Integration tests
```

**Key Components:**
- Lua plugin for Neovim API
- Commands registered via `vim.api.nvim_create_user_command`
- Context extraction using Neovim API
- CLI execution via `vim.fn.jobstart`

### VS Code Extension (radium-vscode)

```
radium-vscode/
├── radium-extension.json    # Radium extension manifest
├── package.json             # VS Code extension manifest
├── hooks/                   # Shared hooks (same as Neovim)
├── src/
│   ├── extension.ts         # Main entry point
│   ├── commands/
│   │   ├── sendSelection.ts
│   │   ├── applyCode.ts
│   │   └── chat.ts
│   └── utils/
│       ├── context.ts       # Context extraction
│       └── cli.ts           # CLI communication
└── out/                     # Compiled JavaScript
```

**Key Components:**
- TypeScript extension using VS Code API
- Commands registered via `vscode.commands.registerCommand`
- Context extraction using VS Code API
- CLI execution via `child_process.spawn`

### Clipboard Mode

```
radium-core/src/clipboard/
├── mod.rs                   # Clipboard operations
└── parser.rs                # Format parsing

apps/cli/src/commands/
└── clipboard.rs             # CLI commands
```

**Key Components:**
- Cross-platform clipboard access (`arboard` crate)
- File path annotation parsing
- Language detection
- CLI commands: `send` and `receive`

## Hook Implementation

### Hook Configuration

Hooks are configured in TOML files:

```toml
[[hooks]]
name = "editor-context"
type = "before_tool"
priority = 150
enabled = true
script = "hooks/editor-context.sh"
```

### Hook Execution

1. **Discovery**: Hooks are discovered from extension directories
2. **Registration**: Loaded into HookRegistry on extension installation
3. **Execution**: Triggered during tool execution lifecycle
4. **Context**: Receive HookContext with tool data
5. **Modification**: Return HookResult with modified data

### Environment Variables

Editor extensions set these for hook access:

- `RADIUM_EDITOR_FILE_PATH`
- `RADIUM_EDITOR_LANGUAGE`
- `RADIUM_EDITOR_SELECTION`
- `RADIUM_EDITOR_SURROUNDING_LINES`

## Context Format

### Input Context (Editor → Radium)

```json
{
  "file_path": "/absolute/path/to/file.rs",
  "language": "rust",
  "selection": "fn main() {\n    println!(\"Hello\");\n}",
  "surrounding_lines": "use std::io;\n---\nfn helper() {}",
  "workspace": "/workspace/root"  // VS Code only
}
```

### Output Format (Radium → Editor)

```json
{
  "original_output": "Full agent response...",
  "code_blocks": [
    {
      "language": "rust",
      "content": "fn improved() {}",
      "index": 0
    }
  ],
  "block_count": 1
}
```

## Extension Discovery

Extensions are discovered from:

1. **User-level**: `~/.radium/extensions/`
2. **Workspace-level**: `.radium/extensions/` (takes precedence)

Hooks are automatically loaded from extension `hooks/` directories.

## Security Considerations

- **Path Validation**: File paths are validated before use
- **Sandboxing**: Tool execution can use sandboxes (Docker/Seatbelt)
- **Context Sanitization**: Editor context is sanitized before injection
- **CLI Isolation**: Commands run in isolated processes

## Performance

- **Hook Priority**: Editor context hook has high priority (150) for early execution
- **Caching**: Hook results can be cached for repeated operations
- **Async Execution**: Hooks execute asynchronously to avoid blocking
- **Streaming**: Large outputs streamed for better performance

## Extension Points

The system is extensible:

1. **Custom Hooks**: Add hooks to extensions for custom processing
2. **Custom Commands**: Extend CLI with editor-specific commands
3. **Format Support**: Add parsers for additional annotation formats
4. **Editor Support**: Create new extensions for other editors

## Future Enhancements

- Real-time sync (bidirectional updates)
- Multi-file operations
- Language server protocol integration
- Advanced diff/merge capabilities
- Workspace-wide context awareness

## See Also

- [Extension System Documentation](../extensions/README.md)
- [Hook System Documentation](../hooks/README.md)
- [Neovim Integration](./neovim.md)
- [VS Code Integration](./vscode.md)

