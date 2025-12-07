# Extension System Guide

The Radium Extension System enables users to share and install reusable packages that bundle prompts, MCP servers, and custom commands. This guide explains how to create, install, and manage extensions.

## Overview

Extensions are installable packages that can contain:
- **Prompts**: Agent prompt templates
- **MCP Servers**: Model Context Protocol server configurations
- **Commands**: Custom TOML-based commands

Extensions are installed to `~/.radium/extensions/` and can be discovered and used by Radium automatically.

## Extension Manifest Format

Every extension must include a `radium-extension.json` manifest file at its root. The manifest defines the extension metadata and components.

### Example Manifest

```json
{
  "name": "my-extension",
  "version": "1.0.0",
  "description": "My custom extension with prompts and commands",
  "author": "Your Name",
  "components": {
    "prompts": ["prompts/*.md"],
    "mcp_servers": ["mcp/*.json"],
    "commands": ["commands/*.toml"]
  },
  "dependencies": []
}
```

### Manifest Fields

- **name** (required): Extension name (alphanumeric with dashes/underscores, must start with alphanumeric)
- **version** (required): Version number (semver format: `major.minor.patch`)
- **description** (required): Extension description
- **author** (required): Author name
- **components** (optional): Component file patterns
  - **prompts**: Glob patterns for prompt files (e.g., `["prompts/*.md"]`)
  - **mcp_servers**: Paths to MCP server config files
  - **commands**: Glob patterns for command files (e.g., `["commands/*.toml"]`)
- **dependencies** (optional): Array of extension names this extension depends on

## Extension Directory Structure

```
my-extension/
├── radium-extension.json    # Manifest file (required)
├── prompts/                  # Prompt templates (optional)
│   ├── agent1.md
│   └── agent2.md
├── mcp/                      # MCP server configs (optional)
│   └── server1.json
└── commands/                 # Custom commands (optional)
    └── my-command.toml
```

## Creating an Extension

1. **Create the directory structure**:
   ```bash
   mkdir -p my-extension/{prompts,mcp,commands}
   ```

2. **Create the manifest** (`radium-extension.json`):
   ```json
   {
     "name": "my-extension",
     "version": "1.0.0",
     "description": "My extension",
     "author": "My Name",
     "components": {
       "prompts": ["prompts/*.md"],
       "commands": ["commands/*.toml"]
     }
   }
   ```

3. **Add your components**:
   - Add prompt files to `prompts/`
   - Add MCP configs to `mcp/`
   - Add command definitions to `commands/`

4. **Validate the extension**:
   ```bash
   rad extension install ./my-extension --overwrite
   ```

## Installing Extensions

### From Local Directory

```bash
rad extension install ./path/to/extension
```

Options:
- `--overwrite`: Overwrite existing installation
- `--install-deps`: Install dependencies automatically

### From URL (Future)

URL-based installation will be supported in a future release.

## Managing Extensions

### List Installed Extensions

```bash
rad extension list
```

Show detailed information:
```bash
rad extension list --verbose
```

JSON output:
```bash
rad extension list --json
```

### Show Extension Details

```bash
rad extension info my-extension
```

### Search Extensions

```bash
rad extension search "query"
```

### Uninstall Extension

```bash
rad extension uninstall my-extension
```

**Note**: Extensions that are dependencies of other extensions cannot be uninstalled until dependent extensions are removed first.

## Component Types

### Prompts

Extension prompts are loaded into the agent discovery system. Place prompt files in the `prompts/` directory and reference them in the manifest:

```json
{
  "components": {
    "prompts": ["prompts/*.md", "prompts/custom/*.md"]
  }
}
```

### MCP Servers

MCP server configurations are loaded into the MCP client system. Place JSON configuration files in the `mcp/` directory:

```json
{
  "components": {
    "mcp_servers": ["mcp/database-server.json"]
  }
}
```

### Commands

Custom commands follow the same TOML format as project-level commands. Place command files in the `commands/` directory:

```json
{
  "components": {
    "commands": ["commands/*.toml"]
  }
}
```

## Extension Dependencies

Extensions can declare dependencies on other extensions:

```json
{
  "name": "dependent-extension",
  "dependencies": ["base-extension"]
}
```

When installing, dependencies must either:
- Already be installed, or
- Be installed automatically with `--install-deps` flag

## Best Practices

1. **Naming**: Use kebab-case for extension names (e.g., `my-extension`, not `my_extension` or `MyExtension`)

2. **Versioning**: Follow semantic versioning (major.minor.patch)

3. **Component Organization**: Organize components into subdirectories for clarity

4. **Documentation**: Include a README.md in your extension root explaining usage

5. **Testing**: Test your extension locally before distributing

## Troubleshooting

### Extension Not Found

- Verify the extension is installed: `rad extension list`
- Check the extension name is correct
- Ensure the manifest file is named `radium-extension.json`

### Installation Fails

- Verify the manifest is valid JSON
- Check all required fields are present
- Ensure component directories exist if declared
- Check for dependency conflicts

### Components Not Loading

- Verify component paths in manifest match actual files
- Check file permissions
- Ensure component directories are properly organized

## Future Enhancements

Planned features (not yet implemented):
- Extension marketplace
- Extension versioning system
- Extension signing and verification
- URL-based installation
- Workspace-level extensions

## References

- **Requirements**: See Braingrid for current REQ status: `braingrid requirement list -p PROJ-14 | grep -i "extension"`
- [Gemini CLI Extensions](../features/gemini-cli-enhancements.md#extension-system)

