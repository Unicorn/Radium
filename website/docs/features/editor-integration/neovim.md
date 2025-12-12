---
id: "neovim"
title: "Neovim Integration Guide"
sidebar_label: "Neovim Integration Guide"
---

# Neovim Integration Guide

Complete guide to using Radium with Neovim via the radium-nvim extension.

## Prerequisites

- Neovim 0.5+ (with Lua support)
- Radium CLI installed and in PATH
- Radium workspace initialized (optional, for some features)

## Installation

### Step 1: Install the Extension

```bash
rad extension install radium-nvim
```

Or from a local directory:

```bash
rad extension install ./extensions/radium-nvim
```

### Step 2: Load the Plugin

Add to your Neovim configuration (`~/.config/nvim/init.lua` or `~/.vimrc`):

```lua
-- Load Radium plugin
require("radium")
```

Or if using a plugin manager like packer.nvim:

```lua
use({
    "radium/radium-nvim",
    config = function()
        require("radium")
    end
})
```

## Commands

### :RadiumSendSelection

Sends the current visual selection to Radium for processing.

**Usage:**
1. Select code in visual mode (`v`, `V`, or `Ctrl-v`)
2. Run `:RadiumSendSelection`
3. Code is sent to Radium with full context:
   - File path
   - Language/filetype
   - Selected code
   - Surrounding lines (for context)

**Configuration:**
```lua
-- Set default agent (defaults to "code-agent")
vim.g.radium_default_agent = "code-agent"
```

### :RadiumChat

Opens an interactive chat session with a Radium agent.

**Usage:**
```
:RadiumChat
```

This opens a split window with an integrated terminal running `rad chat`.

### :RadiumApplyBlock

Applies the last agent-generated code block to the current buffer.

**Usage:**
1. After receiving agent output from `:RadiumSendSelection`
2. Run `:RadiumApplyBlock`
3. A diff preview is shown
4. Press `y` to apply or `n` to cancel

**Features:**
- Shows diff preview before applying
- Handles multiple code blocks (prompts for selection)
- Can replace selection or insert at cursor

## Configuration

### Default Agent

```lua
vim.g.radium_default_agent = "code-agent"
```

### Environment Variables

The extension automatically sets these environment variables for hooks:
- `RADIUM_EDITOR_FILE_PATH` - Full path to current file
- `RADIUM_EDITOR_LANGUAGE` - Filetype/language
- `RADIUM_EDITOR_SELECTION` - Selected code
- `RADIUM_EDITOR_SURROUNDING_LINES` - Context around selection

## Hook Integration

The extension includes two hooks that are automatically registered:

### BeforeTool Hook (editor-context)

Injects editor context before tool execution:
- File path
- Language identifier
- Selected code
- Surrounding lines

### AfterTool Hook (code-apply)

Processes agent output:
- Extracts code blocks from markdown
- Structures output for editor application
- Adds metadata (language, line ranges)

## Workflow Example

1. **Select code:**
   ```
   Visual mode: v
   Select function: jjj
   ```

2. **Send to Radium:**
   ```
   :RadiumSendSelection
   ```

3. **Review output:**
   - Agent response appears in split window
   - Review the suggested changes

4. **Apply changes:**
   ```
   :RadiumApplyBlock
   ```
   - Diff preview appears
   - Press `y` to apply or `n` to cancel

## Troubleshooting

### Commands not found

**Issue:** `:RadiumSendSelection` command not available

**Solution:**
- Verify extension is installed: `rad extension list`
- Check plugin is loaded: `:lua print(require("radium"))`
- Reload Neovim configuration

### CLI not found

**Issue:** Error about `rad` command not found

**Solution:**
- Ensure Radium CLI is in PATH
- Test: `which rad` or `rad --version`
- Restart Neovim after installing CLI

### Context not injected

**Issue:** Agent doesn't receive file context

**Solution:**
- Verify hooks are registered: Check extension installation
- Check environment variables are set
- Ensure file has a valid filetype set

## Advanced Usage

### Custom Agent Selection

You can specify a different agent per command by modifying the command:
```lua
vim.api.nvim_create_user_command('RadiumRefactor', function()
    vim.g.radium_default_agent = "refactor-agent"
    require('radium.commands').send_selection()
end, {})
```

### Integration with Other Plugins

The extension stores agent output in buffer-local variables, making it accessible to other plugins or custom Lua code:

```lua
local output = vim.b.radium_last_output
-- Process output as needed
```

## See Also

- [Extension README](../../extensions/radium-nvim/README.md)
- [Architecture Documentation](./architecture.md)
- [Troubleshooting Guide](./troubleshooting.md)

