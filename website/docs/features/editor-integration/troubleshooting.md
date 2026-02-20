---
id: "troubleshooting"
title: "Editor Integration Troubleshooting"
sidebar_label: "Editor Integration Troubleshooting"
---

# Editor Integration Troubleshooting

Common issues and solutions for Radium editor integrations.

## General Issues

### CLI Not Found

**Symptoms:**
- Error: "rad: command not found"
- Commands fail to execute
- Extension reports CLI missing

**Solutions:**
1. Verify Radium CLI installation:
   ```bash
   which rad
   rad --version
   ```

2. Add to PATH:
   - **Linux/macOS**: Add to `~/.bashrc` or `~/.zshrc`
   - **Windows**: Add to system PATH

3. Restart editor after installation

4. Verify in editor terminal:
   - VS Code: Open integrated terminal, run `rad --version`
   - Neovim: Run `:!rad --version`

### Extension Not Loading

**Symptoms:**
- Commands not available
- No error messages
- Extension appears installed but non-functional

**Solutions:**

1. **Check Extension Installation:**
   ```bash
   rad extension list
   ```
   Verify extension appears in list

2. **Verify Extension Structure:**
   - Check `radium-extension.json` exists
   - Verify manifest is valid JSON
   - Ensure required directories exist

3. **Check Extension Location:**
   - User-level: `~/.radium/extensions/<extension-name>/`
   - Workspace-level: `.radium/extensions/<extension-name>/`

4. **Reload Editor:**
   - Neovim: `:source ~/.config/nvim/init.lua` or restart
   - VS Code: `Ctrl+Shift+P` → "Developer: Reload Window"

### Hooks Not Executing

**Symptoms:**
- Context not injected
- Code blocks not extracted
- Agent doesn't receive file information

**Solutions:**

1. **Verify Hook Files Exist:**
   ```bash
   ls ~/.radium/extensions/radium-nvim/hooks/
   # Should show: editor-context.toml, code-apply.toml
   ```

2. **Check Hook Configuration:**
   - Verify TOML syntax is valid
   - Check hook types are correct: `before_tool`, `after_tool`
   - Ensure `enabled = true`

3. **Test Hook Registration:**
   ```bash
   rad hooks list  # If available
   ```

4. **Check Hook Scripts:**
   - Verify script files are executable: `chmod +x hooks/*.sh`
   - Test scripts manually if possible

## Neovim Specific

### Commands Not Registered

**Symptoms:**
- `:RadiumSendSelection` not found
- `E492: Not an editor command`

**Solutions:**

1. **Check Plugin Loading:**
   ```lua
   :lua print(require("radium"))
   ```
   Should not error

2. **Verify Plugin Path:**
   ```lua
   :lua print(vim.fn.stdpath("data") .. "/site/pack/*/start/radium-nvim")
   ```

3. **Check Lua Module:**
   ```lua
   :lua print(package.path)
   ```
   Verify plugin directory in path

4. **Manually Load:**
   ```lua
   :lua require("radium")
   ```

### Visual Selection Issues

**Symptoms:**
- Selection empty
- Wrong code selected
- Context extraction fails

**Solutions:**

1. **Ensure Visual Mode:**
   - Use `v` (character), `V` (line), or `Ctrl-v` (block)
   - Verify selection highlighted

2. **Check Selection Functions:**
   ```lua
   :lua print(vim.fn.mode())  -- Should show 'v', 'V', or ''
   ```

3. **Test Selection Retrieval:**
   ```lua
   :lua local utils = require("radium.utils"); print(utils.get_visual_selection())
   ```

### Diff Preview Not Working

**Symptoms:**
- No diff shown
- Error when applying
- Preview window not opening

**Solutions:**

1. **Check Neovim Version:**
   - Requires Neovim 0.5+ for floating windows
   - Verify: `:version`

2. **Test Diff Function:**
   ```lua
   :lua local diff = require("radium.diff"); print(diff.generate_diff("old", "new"))
   ```

3. **Check Window API:**
   ```lua
   :lua print(vim.api.nvim_open_win)
   ```
   Should not be nil

## VS Code Specific

### Commands Not in Palette

**Symptoms:**
- "Radium: Send Selection" not appearing
- Extension installed but commands missing

**Solutions:**

1. **Check Extension Activation:**
   - Open Output panel
   - Select "Radium" channel
   - Look for activation messages

2. **Verify package.json:**
   - Check `contributes.commands` section
   - Ensure command IDs match

3. **Reload Window:**
   - `Ctrl+Shift+P` → "Developer: Reload Window"

4. **Check Extension Logs:**
   - View → Output → Select "Radium"

### Terminal Not Opening

**Symptoms:**
- Chat command doesn't open terminal
- Terminal opens but empty
- Error creating terminal

**Solutions:**

1. **Check Terminal API:**
   - Verify VS Code version 1.74.0+
   - Test: `Ctrl+` ` (backtick) to open terminal manually

2. **Check Terminal Permissions:**
   - Verify terminal creation not blocked
   - Check workspace settings

3. **Test Terminal Creation:**
   ```typescript
   // In extension developer console
   vscode.window.createTerminal({ name: "Test" }).show();
   ```

### Diff Viewer Issues

**Symptoms:**
- Diff not showing
- "Apply" button missing
- Preview incorrect

**Solutions:**

1. **Check VS Code Version:**
   - Requires 1.74.0+ for diff API
   - Update VS Code if needed

2. **Verify Diff API:**
   - Test with simple diff manually
   - Check for API changes in VS Code updates

3. **Check Output Format:**
   - Ensure code blocks properly extracted
   - Verify JSON structure

## Clipboard Mode

### Clipboard Access Denied

**Symptoms:**
- "Failed to read from clipboard"
- Permission errors
- Empty clipboard when content exists

**Solutions:**

1. **Check Permissions:**
   - macOS: Grant terminal app clipboard access in System Preferences
   - Linux: May need `xclip` or `xsel` installed
   - Windows: Usually works by default

2. **Test Clipboard Access:**
   ```bash
   # macOS
   pbpaste
   
   # Linux
   xclip -selection clipboard -o
   
   # Windows
   clip
   ```

3. **Install Dependencies:**
   ```bash
   # Linux (Ubuntu/Debian)
   sudo apt-get install xclip
   
   # Or xsel
   sudo apt-get install xsel
   ```

### File Path Not Parsed

**Symptoms:**
- Annotation not recognized
- Language detection fails
- File path missing in context

**Solutions:**

1. **Check Annotation Format:**
   - Must be on first line
   - Format: `// File: path` or `# File: path`
   - No extra spaces or characters

2. **Test Parsing:**
   ```bash
   echo "// File: test.rs\nfn main() {}" | rad clipboard send
   ```

3. **Verify Parser:**
   - Check regex patterns in `parser.rs`
   - Test with different comment styles

### Language Detection Fails

**Symptoms:**
- Language shows as "unknown"
- Wrong language detected
- No language information

**Solutions:**

1. **Add File Extension:**
   - Include file extension in path annotation
   - Example: `// File: src/main.rs`

2. **Use Explicit Annotation:**
   - Add language hint in annotation
   - Provide filetype in context manually

3. **Check Detection Patterns:**
   - Verify code patterns match known languages
   - Check shebang detection for scripts

## Performance Issues

### Slow Command Execution

**Symptoms:**
- Commands take long time
- Editor freezes during execution
- Timeout errors

**Solutions:**

1. **Check Agent Response Time:**
   - Test agent directly: `rad step code-agent`
   - Verify network/model API response times

2. **Optimize Context:**
   - Reduce surrounding lines count
   - Limit selection size for large files

3. **Check Hook Performance:**
   - Verify hooks aren't doing expensive operations
   - Test hook execution time

### Memory Usage

**Symptoms:**
- High memory consumption
- Editor becomes slow
- Out of memory errors

**Solutions:**

1. **Limit Output Size:**
   - Truncate large agent responses
   - Process output in chunks

2. **Clear Stored Data:**
   - Clear buffer variables periodically
   - Reset agent output storage

## Getting Help

### Debug Mode

Enable debug logging:

**Neovim:**
```lua
vim.g.radium_debug = true
```

**VS Code:**
```json
{
  "radium.debug": true
}
```

### Log Files

Check log files for errors:
- Neovim: `:messages`
- VS Code: View → Output → "Radium"
- CLI: Check stderr output

### Reporting Issues

When reporting issues, include:

1. Editor version
2. Extension version
3. Radium CLI version
4. Error messages/logs
5. Steps to reproduce
6. System information

## See Also

- [Neovim Integration Guide](./neovim.md)
- [VS Code Integration Guide](./vscode.md)
- [Clipboard Mode Guide](./clipboard.md)
- [Architecture Documentation](./architecture.md)

