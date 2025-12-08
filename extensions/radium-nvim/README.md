# Radium Neovim Extension

Seamless bidirectional integration between Radium and Neovim for code analysis, refactoring, and AI-assisted development.

## Overview

The Radium Neovim extension enables direct integration between Neovim and Radium agents, allowing you to:
- Send code selections to Radium for analysis and refactoring
- Apply agent-generated code blocks directly to your buffers
- Chat with Radium agents directly from Neovim
- Automatically inject editor context (file path, language, surrounding code) into agent requests

## Architecture

### Components

1. **Neovim Plugin** (`plugin/radium.lua`)
   - Lua plugin that provides Neovim commands and UI integration
   - Handles communication with Radium CLI via `rad step` and `rad chat`

2. **Hooks** (`hooks/`)
   - `editor-context.toml`: BeforeTool hook that injects editor context
   - `code-apply.toml`: AfterTool hook that processes agent output for editor application

3. **Commands** (`commands/`)
   - Custom Radium commands integrated with the extension system

### Data Flow

```
Neovim → :RadiumSendSelection → rad step → BeforeTool Hook (context injection) 
→ Agent Processing → AfterTool Hook (code extraction) → Neovim → :RadiumApplyBlock
```

## Installation

Install via Radium extension system:

```bash
rad extension install ./extensions/radium-nvim
```

Or install from a workspace:

```bash
rad extension install radium-nvim
```

Then load the plugin in your Neovim configuration:

```lua
-- In your init.lua or config
require("radium")
```

## Commands

### :RadiumSendSelection

Sends the current visual selection to Radium for processing.

**Usage:**
1. Select code in visual mode
2. Run `:RadiumSendSelection`
3. Code is sent to Radium with full context (file path, language, surrounding lines)

### :RadiumChat

Opens an interactive chat session with a Radium agent.

**Usage:**
```
:RadiumChat
```

Opens a split window with an interactive terminal running `rad chat`.

### :RadiumApplyBlock

Applies the last agent-generated code block to the current buffer.

**Usage:**
1. After receiving agent output, run `:RadiumApplyBlock`
2. A diff preview is shown
3. Confirm to apply changes

## Context Injection

The extension automatically injects the following context into Radium requests:

- **file_path**: Full path to the current file
- **language**: Filetype/language identifier
- **selection**: Selected code text
- **surrounding_lines**: Code context around the selection

This context is injected via the BeforeTool hook before agent processing.

## Extension System Integration

This extension leverages Radium's extension system:
- Hooks are automatically registered when the extension is installed
- Commands are integrated with `rad` CLI
- All components follow Radium extension conventions

## Requirements

- Neovim 0.5+ (with Lua support)
- Radium CLI installed and available in PATH
- Radium workspace initialized (for some features)

## License

Same as Radium project license.

