# Radium VS Code Extension

Seamless bidirectional integration between VS Code and Radium agents for code analysis, refactoring, and AI-assisted development.

## Overview

The Radium VS Code extension enables direct integration between VS Code and Radium agents, allowing you to:
- Send code selections to Radium for analysis and refactoring
- Apply agent-generated code blocks directly to your files
- Chat with Radium agents directly from VS Code
- Automatically inject editor context (file path, language, workspace) into agent requests

## Installation

Install via Radium extension system:

```bash
rad extension install ./extensions/radium-vscode
```

Or install from a workspace:

```bash
rad extension install radium-vscode
```

Then install the VS Code extension:
1. Open VS Code
2. Press `Ctrl+Shift+X` (or `Cmd+Shift+X` on Mac) to open Extensions
3. Search for "Radium Code Assistant"
4. Click Install

Or install from VSIX:
```bash
code --install-extension radium-vscode-1.0.0.vsix
```

## Commands

### Radium: Send Selection

Sends the current selection to Radium for processing.

**Usage:**
1. Select code in the editor
2. Open command palette (`Ctrl+Shift+P` / `Cmd+Shift+P`)
3. Run "Radium: Send Selection"
4. Code is sent to Radium with full context (file path, language, workspace)

### Radium: Apply Code

Applies the last agent-generated code block to the current file.

**Usage:**
1. After receiving agent output, open command palette
2. Run "Radium: Apply Code"
3. A diff preview is shown
4. Confirm to apply changes

### Radium: Chat

Opens an interactive chat session with a Radium agent.

**Usage:**
1. Open command palette
2. Run "Radium: Chat"
3. An integrated terminal opens with interactive chat session

## Configuration

Default agent can be configured in VS Code settings:

```json
{
  "radium.defaultAgent": "code-agent"
}
```

## Architecture

This extension leverages:
- Radium's extension system for hooks
- VS Code's extension API for editor integration
- CLI commands (`rad step`, `rad chat`) for communication

## Requirements

- VS Code 1.74.0+
- Radium CLI installed and available in PATH
- Radium workspace initialized (for some features)

## License

Same as Radium project license.

