---
id: "creating-extensions"
title: "Creating Extensions"
sidebar_label: "Creating Extensions"
---

# Creating Extensions

This guide walks you through creating your own Radium extension from scratch.

## Extension Manifest

Every extension must have a `radium-extension.json` manifest file at its root. This file defines the extension's metadata and components.

### Basic Manifest Structure

```json
{
  "name": "my-extension",
  "version": "1.0.0",
  "description": "A brief description of what this extension does",
  "author": "Your Name",
  "components": {
    "prompts": [],
    "mcp_servers": [],
    "commands": [],
    "hooks": []
  },
  "dependencies": []
}
```

### Required Fields

- **name**: Extension name (alphanumeric, dashes, underscores only; must start with a letter)
- **version**: Semantic version (e.g., `1.0.0`, `2.1.3`)
- **description**: Brief description of the extension
- **author**: Author name or contact information

### Component Paths

Component paths support glob patterns:

```json
{
  "components": {
    "prompts": ["prompts/*.md", "prompts/frameworks/*.md"],
    "mcp_servers": ["mcp/*.json"],
    "commands": ["commands/*.toml", "commands/deploy/*.toml"],
    "hooks": ["hooks/*.toml"]
  }
}
```

### Dependencies

Declare other extensions your extension depends on:

```json
{
  "dependencies": ["required-extension-1", "required-extension-2"]
}
```

Dependencies are automatically installed when you install an extension with the `--install-deps` flag.

## Directory Structure

Create the following directory structure:

```
my-extension/
├── radium-extension.json
├── prompts/          # Optional: Agent prompts
├── mcp/              # Optional: MCP server configs
├── commands/         # Optional: Custom commands
├── hooks/            # Optional: Hook configs
└── README.md         # Recommended: Documentation
```

## Component Types

### Prompts

Prompts are markdown files containing agent prompt templates. Place them in the `prompts/` directory:

```
prompts/
├── code-review-agent.md
└── documentation-agent.md
```

Example prompt file:

```markdown
# Code Review Agent

You are an expert code reviewer. Analyze the provided code and provide constructive feedback.

## Guidelines
- Focus on code quality and best practices
- Suggest improvements with examples
- Be respectful and constructive
```

### MCP Servers

MCP server configurations are JSON files that define how to connect to Model Context Protocol servers. Place them in the `mcp/` directory:

```
mcp/
└── my-mcp-server.json
```

Example MCP server config (note: this is a simplified example; actual MCP configs use TOML format in the workspace):

```json
{
  "name": "my-mcp-server",
  "transport": "stdio",
  "command": "mcp-server",
  "args": ["--config", "config.json"]
}
```

**Note**: MCP server configurations in extensions are loaded and merged with the workspace MCP configuration. See the [MCP documentation](../mcp/configuration.md) for full configuration format.

### Commands

Custom commands are TOML files defining executable commands. Place them in the `commands/` directory:

```
commands/
└── deploy.toml
```

Example command file:

```toml
name = "deploy"
description = "Deploy application to production"
command = "deploy.sh"
args = ["--env", "production"]
```

Commands from extensions are namespaced with the extension name (e.g., `my-extension:deploy`).

### Hooks

Hooks are TOML files that configure native libraries or WASM modules. Place them in the `hooks/` directory:

```
hooks/
└── logging-hook.toml
```

Example hook file:

```toml
name = "logging-hook"
type = "native"
library = "liblogging.so"
```

## Validation Rules

### Extension Name

- Must start with a letter
- Can contain letters, numbers, dashes, and underscores
- Must be unique (case-sensitive)

Examples:
- ✅ `my-extension`
- ✅ `extension_123`
- ❌ `123-extension` (starts with number)
- ❌ `my extension` (contains space)

### Version Format

Must follow semantic versioning (semver):

- Format: `MAJOR.MINOR.PATCH`
- Examples: `1.0.0`, `2.1.3`, `0.1.0`

### Component Paths

- Paths cannot be empty
- Glob patterns are supported
- Paths are relative to the extension root

## Creating Your First Extension

1. **Create the directory structure**:

```bash
mkdir my-extension
cd my-extension
mkdir -p prompts mcp commands hooks
```

2. **Create the manifest** (`radium-extension.json`):

```json
{
  "name": "my-extension",
  "version": "1.0.0",
  "description": "My first Radium extension",
  "author": "Your Name",
  "components": {
    "prompts": ["prompts/*.md"],
    "commands": ["commands/*.toml"]
  },
  "dependencies": []
}
```

3. **Add components**:

Create a prompt file (`prompts/my-agent.md`):

```markdown
# My Custom Agent

This is a custom agent prompt.
```

4. **Test installation**:

```bash
rad extension install ./my-extension
rad extension list
rad extension info my-extension
```

5. **Verify components are loaded**:

```bash
# Check if prompts are discoverable
rad agents list

# Check if commands are available
rad commands list
```

## Best Practices

### Naming

- Use descriptive, lowercase names with dashes
- Avoid generic names that might conflict
- Consider prefixing with your organization/username

### Versioning

- Start with `1.0.0` for initial release
- Follow semantic versioning for updates
- Increment version when making changes

### Documentation

- Include a `README.md` explaining what the extension does
- Document all components and their purpose
- Provide usage examples
- List any dependencies or requirements

### Testing

- Test installation from local directory
- Verify all components are discoverable
- Test with different Radium versions if possible
- Check for conflicts with other extensions

### Distribution

- Package as `.tar.gz` or `.zip` for distribution
- Include all necessary files
- Ensure manifest is valid JSON
- Test installation from archive

## Troubleshooting

### Manifest Validation Errors

**Error**: `invalid extension name`

- Check name starts with a letter
- Ensure no spaces or special characters (except dashes/underscores)

**Error**: `invalid version format`

- Use semantic versioning format: `MAJOR.MINOR.PATCH`
- Examples: `1.0.0`, `2.1.3`

**Error**: `missing required field`

- Ensure all required fields are present: `name`, `version`, `description`, `author`

### Component Discovery Issues

**Components not found**:

1. Verify component directories exist
2. Check glob patterns match actual file paths
3. Ensure file extensions are correct:
   - Prompts: `.md`
   - MCP: `.json`
   - Commands: `.toml`
   - Hooks: `.toml`

**Components not loading**:

1. Check file formats are valid
2. Verify paths in manifest match directory structure
3. Ensure files are not empty

### Installation Issues

**Installation fails**:

- Check manifest is valid JSON
- Verify all required fields are present
- Ensure extension name is unique
- Use `--overwrite` if reinstalling

**Dependencies not installing**:

- Ensure dependency names match exactly
- Check dependencies are installed first
- Use `--install-deps` flag

## Next Steps

- See [Architecture](architecture.md) for technical details
- Check [examples](examples/) for sample extensions
- Review [CLI commands](../user-guide/agent-configuration.md) for usage

