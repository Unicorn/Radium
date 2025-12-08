# VS Code Integration Guide

Complete guide to using Radium with VS Code via the radium-vscode extension.

## Prerequisites

- VS Code 1.74.0+
- Node.js (for extension development/build)
- Radium CLI installed and in PATH
- Radium workspace initialized (optional, for some features)

## Installation

### Step 1: Install the Extension via Radium

```bash
rad extension install radium-vscode
```

### Step 2: Install VS Code Extension

The extension can be installed via:
1. **Command Palette**: `Ctrl+Shift+P` (or `Cmd+Shift+P` on Mac) → "Extensions: Install from VSIX"
2. **Marketplace**: Search for "Radium Code Assistant" (when published)
3. **Manual**: Build and install from source (see Development section)

### Step 3: Activate Extension

The extension activates automatically when you use Radium commands. No additional setup required.

## Commands

All commands are accessible via the Command Palette (`Ctrl+Shift+P` / `Cmd+Shift+P`):

### Radium: Send Selection

Sends the current selection to Radium for processing.

**Usage:**
1. Select code in the editor
2. Open Command Palette (`Ctrl+Shift+P`)
3. Run "Radium: Send Selection"
4. Code is sent to Radium with full context:
   - File path
   - Language identifier
   - Selected code
   - Workspace information

**Output:**
- Results appear in a new document in a side-by-side view
- Output is in Markdown format
- Code blocks are highlighted

### Radium: Apply Code

Applies the last agent-generated code block to the current file.

**Usage:**
1. After receiving agent output from "Radium: Send Selection"
2. Open Command Palette
3. Run "Radium: Apply Code"
4. Select code block if multiple are available
5. Diff preview is shown using VS Code's built-in diff viewer
6. Click "Apply" to confirm or "Cancel" to abort

**Features:**
- Automatic code block detection
- Multiple code block selection
- Side-by-side diff preview
- Inline change application

### Radium: Chat

Opens an interactive chat session with a Radium agent.

**Usage:**
1. Open Command Palette
2. Run "Radium: Chat"
3. An integrated terminal opens with `rad chat` running
4. Type messages and press Enter

**Features:**
- Integrated terminal
- Context-aware (includes current file path and language)
- Full chat history preserved

## Configuration

### VS Code Settings

Add to your VS Code settings.json:

```json
{
  "radium.defaultAgent": "code-agent"
}
```

### Workspace Settings

Workspace-specific configuration in `.vscode/settings.json`:

```json
{
  "radium.defaultAgent": "workspace-specific-agent"
}
```

## Extension Architecture

The VS Code extension consists of:

- **TypeScript Extension**: Main extension code in `src/`
- **Radium Extension**: Hooks and integration via Radium extension system
- **CLI Communication**: Uses `rad step` and `rad chat` commands

## Hook Integration

The extension includes hooks that are automatically registered when the Radium extension is installed:

- **BeforeTool Hook**: Injects editor context
- **AfterTool Hook**: Processes agent output for code extraction

These hooks are shared with the Neovim extension.

## Workflow Example

1. **Select Code:**
   - Highlight code in editor
   - Use `Shift+Arrow` keys or mouse selection

2. **Send to Radium:**
   - `Ctrl+Shift+P` → "Radium: Send Selection"
   - Wait for agent processing
   - Review output in side panel

3. **Apply Changes:**
   - `Ctrl+Shift+P` → "Radium: Apply Code"
   - Select code block if multiple available
   - Review diff preview
   - Click "Apply" to confirm

## Troubleshooting

### Commands Not Appearing

**Issue:** Radium commands not in Command Palette

**Solution:**
- Verify extension is installed: `rad extension list`
- Check extension is activated (should auto-activate)
- Reload VS Code window: `Ctrl+Shift+P` → "Developer: Reload Window"

### CLI Not Found

**Issue:** Error about `rad` command not found

**Solution:**
- Ensure Radium CLI is in PATH
- Test in terminal: `rad --version`
- Restart VS Code after installing CLI
- Check VS Code's integrated terminal can find `rad`

### Output Panel Not Showing

**Issue:** Agent output not visible

**Solution:**
- Check Output panel: View → Output → Select "Radium" channel
- Verify agent executed successfully
- Check for error messages in Output panel

### Diff Preview Not Working

**Issue:** Diff preview doesn't show changes

**Solution:**
- Ensure you've run "Radium: Send Selection" first
- Verify agent output contains code blocks
- Check VS Code version (1.74.0+ required)

## Development

### Building from Source

```bash
cd extensions/radium-vscode
npm install
npm run compile
```

### Installing Development Version

```bash
npm install -g vsce
vsce package
code --install-extension radium-vscode-1.0.0.vsix
```

## See Also

- [Extension README](../../extensions/radium-vscode/README.md)
- [Architecture Documentation](./architecture.md)
- [Troubleshooting Guide](./troubleshooting.md)

