# Editor Integration Overview

Radium provides seamless bidirectional integration with popular editors through extensions and clipboard mode, enabling AI-assisted development without leaving your editor.

## Integration Methods

Radium offers three integration methods, each with different levels of functionality:

### 1. Neovim Extension (radium-nvim)

**Best for:** Neovim users who want full-featured integration

- Direct commands in Neovim (`:RadiumSendSelection`, `:RadiumChat`, `:RadiumApplyBlock`)
- Automatic context injection (file path, language, surrounding code)
- Diff preview before applying changes
- Integrated chat sessions

**Installation:**
```bash
rad extension install radium-nvim
```

**Documentation:** [Neovim Integration Guide](./neovim.md)

### 2. VS Code Extension (radium-vscode)

**Best for:** VS Code users who want native IDE integration

- Command palette integration
- Built-in diff viewer
- Integrated terminal for chat
- Workspace awareness

**Installation:**
```bash
rad extension install radium-vscode
```

**Documentation:** [VS Code Integration Guide](./vscode.md)

### 3. Clipboard Mode

**Best for:** Users of any editor or when extensions aren't available

- Works with any editor via copy/paste
- File path annotation support
- Automatic language detection
- No extension installation required

**Usage:**
```bash
rad clipboard send    # Send code from clipboard to Radium
rad clipboard receive # Get processed code to clipboard
```

**Documentation:** [Clipboard Mode Guide](./clipboard.md)

## Quick Start

1. **Install Radium CLI** (if not already installed)
2. **Choose your integration method** based on your editor
3. **Install the extension** (for Neovim or VS Code) or use clipboard mode
4. **Start using** - select code and send to Radium!

## Architecture

All integration methods leverage:

- **Extension System**: Radium's extension infrastructure for hooks and commands
- **Hook System**: BeforeTool and AfterTool hooks for context injection and result processing
- **CLI Commands**: `rad step` and `rad chat` for agent communication

See [Architecture Documentation](./architecture.md) for technical details.

## Comparison

| Feature | Neovim | VS Code | Clipboard |
|---------|--------|---------|-----------|
| Editor Commands | ✅ | ✅ | ❌ |
| Context Injection | ✅ | ✅ | ⚠️ Manual |
| Diff Preview | ✅ | ✅ | ❌ |
| Chat Integration | ✅ | ✅ | ❌ |
| Works with Any Editor | ❌ | ❌ | ✅ |
| Setup Complexity | Medium | Medium | Low |

## Requirements

- Radium CLI installed and available in PATH
- Radium workspace initialized (for some features)
- Editor-specific requirements (see individual guides)

## Next Steps

- [Neovim Integration](./neovim.md) - Complete Neovim setup guide
- [VS Code Integration](./vscode.md) - Complete VS Code setup guide
- [Clipboard Mode](./clipboard.md) - Universal editor support
- [Architecture](./architecture.md) - Technical implementation details
- [Troubleshooting](./troubleshooting.md) - Common issues and solutions

