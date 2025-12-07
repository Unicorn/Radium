# Extension System

The Radium Extension System allows users to install and share reusable packages that bundle agents, templates, commands, prompts, and MCP server configurations.

## Quick Start

### Installing an Extension

```bash
# Install from a local directory
rad extension install ./my-extension

# Install with overwrite if already installed
rad extension install ./my-extension --overwrite

# Install and automatically install dependencies
rad extension install ./my-extension --install-deps
```

### Listing Extensions

```bash
# List all installed extensions
rad extension list

# List with detailed information
rad extension list --verbose

# List in JSON format
rad extension list --json
```

### Viewing Extension Details

```bash
# Show information about a specific extension
rad extension info my-extension

# Show in JSON format
rad extension info my-extension --json
```

### Searching Extensions

```bash
# Search for extensions by name or description
rad extension search "query"
```

### Uninstalling Extensions

```bash
# Uninstall an extension
rad extension uninstall my-extension
```

**Note**: Extensions that are dependencies of other extensions cannot be uninstalled until dependent extensions are removed first.

## Extension Components

Extensions can contain the following component types:

- **Agents**: Agent configuration files (`.toml`) in the `agents/` directory
- **Templates**: Workflow template files (`.json`) in the `templates/` directory
- **Commands**: Custom command files (`.toml`) in the `commands/` directory
- **Prompts**: Prompt template files (`.md`) in the `prompts/` directory
- **MCP Servers**: MCP server configuration files (`.json`) in the `mcp/` directory

## Extension Discovery

Extension components are automatically discovered and integrated into Radium's discovery systems:

- Extension agents are discovered by `AgentDiscovery`
- Extension templates are discovered by `TemplateDiscovery`
- Extension commands are discovered by `CommandRegistry` (with namespace support)

Search path priority (highest to lowest):
1. Project-local components
2. User-level components
3. Extension components (project-level)
4. Extension components (user-level)

## Extension Locations

Extensions are installed to:
- User-level: `~/.radium/extensions/<extension-name>/`
- Project-level: `./.radium/extensions/<extension-name>/`

Project-level extensions take precedence over user-level extensions.

## See Also

- [Creating Extensions](creating-extensions.md) - Guide for extension developers
- [Manifest Reference](manifest-reference.md) - Complete manifest schema documentation
- [Extension System Guide](../guides/extension-system.md) - Detailed user guide

