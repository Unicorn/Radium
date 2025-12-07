# Creating Extensions

This guide explains how to create and package Radium extensions for distribution.

## Extension Structure

Every extension must follow this directory structure:

```
my-extension/
├── radium-extension.json    # Manifest file (required)
├── agents/                   # Agent configurations (optional)
│   └── my-agent.toml
├── templates/                # Workflow templates (optional)
│   └── my-template.json
├── commands/                 # Custom commands (optional)
│   └── my-command.toml
├── prompts/                  # Prompt templates (optional)
│   └── my-prompt.md
└── mcp/                      # MCP server configs (optional)
    └── my-server.json
```

## Creating the Manifest

Create a `radium-extension.json` file in the root of your extension:

```json
{
  "name": "my-extension",
  "version": "1.0.0",
  "description": "My custom extension with agents and commands",
  "author": "Your Name <your@email.com>",
  "components": {
    "prompts": ["prompts/*.md"],
    "mcp_servers": ["mcp/*.json"],
    "commands": ["commands/*.toml"]
  },
  "dependencies": []
}
```

### Required Fields

- **name**: Extension name (alphanumeric with dashes/underscores, must start with alphanumeric)
- **version**: Version number (semver format: `major.minor.patch`)
- **description**: Extension description
- **author**: Author name

### Optional Fields

- **components**: Component file patterns (see below)
- **dependencies**: Array of extension names this extension depends on

## Adding Components

### Agents

Create agent configuration files in the `agents/` directory:

```toml
# agents/my-agent.toml
[agent]
id = "my-agent"
name = "My Agent"
description = "Custom agent from extension"
prompt_path = "prompts/my-agent.md"
```

Then reference the prompt file in the `prompts/` directory and add it to the manifest:

```json
{
  "components": {
    "prompts": ["prompts/my-agent.md"]
  }
}
```

### Templates

Create workflow template files in the `templates/` directory:

```json
{
  "name": "my-template",
  "description": "Custom workflow template",
  "steps": []
}
```

### Commands

Create command files in the `commands/` directory:

```toml
# commands/my-command.toml
name = "my-command"
description = "Custom command from extension"
template = "echo 'Hello from extension'"
```

Commands from extensions are automatically namespaced with the extension name (e.g., `my-extension:my-command`).

### Prompts

Place prompt markdown files in the `prompts/` directory. These are used by agents defined in the extension.

### MCP Servers

Place MCP server configuration JSON files in the `mcp/` directory.

## Testing Your Extension

Before distributing your extension, test it locally:

```bash
# Install from your extension directory
rad extension install ./my-extension

# Verify it's installed
rad extension list

# Check extension details
rad extension info my-extension

# Test that components are discoverable
rad agents list        # Should show your agents
rad templates list    # Should show your templates
rad commands list     # Should show your commands

# Uninstall when done testing
rad extension uninstall my-extension
```

## Packaging for Distribution

Extensions can be distributed as:

1. **Directory**: Share the extension directory as-is
2. **Archive**: Package as `.tar.gz` (future: automatic archive support)

### Creating a TAR Archive

```bash
tar -czf my-extension.tar.gz my-extension/
```

Users can then extract and install:

```bash
tar -xzf my-extension.tar.gz
rad extension install ./my-extension
```

## Extension Dependencies

If your extension depends on another extension, declare it in the manifest:

```json
{
  "name": "my-extension",
  "version": "1.0.0",
  "description": "Extension that depends on base-extension",
  "author": "Author",
  "dependencies": ["base-extension"]
}
```

When installing, ensure dependencies are installed first, or use `--install-deps`:

```bash
rad extension install ./my-extension --install-deps
```

## Best Practices

1. **Use semantic versioning** for extension versions
2. **Test locally** before distributing
3. **Document your extension** with a README.md
4. **Use descriptive names** for components
5. **Follow naming conventions**: lowercase with dashes for extension names
6. **Validate your manifest** before packaging
7. **Handle dependencies** properly - don't create circular dependencies

## Validation

The extension system validates:

- Manifest schema and required fields
- Component file existence
- Component file syntax (TOML/JSON)
- Path security (no path traversal attacks)
- Dependency availability
- Component ID conflicts

Installation will fail with clear error messages if validation fails.

## See Also

- [Manifest Reference](manifest-reference.md) - Complete schema documentation
- [Extension System Guide](../guides/extension-system.md) - User guide

