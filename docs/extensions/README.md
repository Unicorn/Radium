# Extension System

The Radium extension system allows you to package and share reusable agent configurations, MCP servers, custom commands, and hooks. Extensions enable the community to share workflows, tools, and configurations.

## Quick Start

### Installing an Extension

Install an extension from a local directory:

```bash
rad extension install ./my-extension
```

Install from a URL:

```bash
rad extension install https://example.com/extensions/my-extension.tar.gz
```

Install from an archive file:

```bash
rad extension install ./my-extension.tar.gz
```

### Listing Installed Extensions

```bash
rad extension list
```

### Getting Extension Information

```bash
rad extension info my-extension
```

### Searching Extensions

Search locally installed extensions:

```bash
rad extension search "github"
```

Search marketplace extensions:

```bash
rad extension search "github" --marketplace-only
```

Search both local and marketplace:

```bash
rad extension search "github"
```

### Browsing Marketplace

Browse popular extensions from the marketplace:

```bash
rad extension browse
```

### Creating a New Extension

```bash
rad extension create my-extension --author "Your Name" --description "My extension description"
```

### Uninstalling an Extension

```bash
rad extension uninstall my-extension
```

### Installing from Marketplace

Install an extension directly from the marketplace by name:

```bash
rad extension install marketplace-extension-name
```

### Signing and Verifying Extensions

Sign an extension:

```bash
rad extension sign ./my-extension --generate-key
```

Verify an extension signature:

```bash
rad extension verify my-extension
```

### Publishing to Marketplace

Publish your extension to the marketplace:

```bash
rad extension publish ./my-extension --api-key YOUR_API_KEY
```

### Advanced Installation Options

Install with automatic dependency resolution:

```bash
rad extension install ./my-extension --install-deps
```

Overwrite an existing extension:

```bash
rad extension install ./my-extension --overwrite
```

### JSON Output

All commands support JSON output for scripting:

```bash
rad extension list --json
rad extension info my-extension --json
rad extension search "query" --json
```

### Verbose Listing

Get detailed information about all extensions:

```bash
rad extension list --verbose
```

## What are Extensions?

Extensions are packages that can contain:

- **Prompts**: Agent prompt templates that can be used by the agent system
- **MCP Servers**: Model Context Protocol server configurations
- **Commands**: Custom CLI commands that extend Radium functionality
- **Hooks**: Native libraries or WASM modules that customize agent behavior

## Extension Structure

Extensions follow a standard directory structure:

```
my-extension/
├── radium-extension.json    # Extension manifest (required)
├── prompts/                 # Agent prompt templates (optional)
│   └── *.md
├── mcp/                     # MCP server configurations (optional)
│   └── *.json
├── commands/                # Custom commands (optional)
│   └── *.toml
├── hooks/                   # Hook configurations (optional)
│   └── *.toml
└── README.md                # Extension documentation (recommended)
```

## Installation Locations

Extensions can be installed in two locations:

1. **User-level**: `~/.radium/extensions/` - Available to all projects
2. **Project-level**: `.radium/extensions/` - Specific to the current workspace

Project-level extensions take precedence over user-level extensions.

## Documentation

- [User Guide](user-guide.md) - Complete guide to using extensions
- [Quickstart Guide](quickstart.md) - Get started in 10 minutes
- [Creating Extensions](creating-extensions.md) - Guide for extension authors
- [Publishing Guide](publishing-guide.md) - How to publish to the marketplace
- [Marketplace Guide](marketplace.md) - Discover and use marketplace features
- [Architecture](architecture.md) - Technical architecture documentation

## Examples

See the [examples directory](examples/) for sample extensions demonstrating different use cases.

## Troubleshooting

### Extension Not Found

If an extension isn't being discovered:

1. Verify the extension is installed: `rad extension list`
2. Check the manifest file exists: `radium-extension.json`
3. Ensure the extension name matches exactly (case-sensitive)

### Installation Errors

Common installation issues:

- **Invalid manifest**: Check that `radium-extension.json` is valid JSON and contains all required fields
- **Version format**: Version must follow semantic versioning (e.g., `1.0.0`)
- **Name format**: Extension names must be alphanumeric with dashes/underscores only, and start with a letter
- **Path conflicts**: Use `--overwrite` flag if installing over an existing extension

### Component Not Loading

If extension components aren't being discovered:

1. Verify component directories exist and contain files
2. Check glob patterns in manifest match actual file paths
3. Ensure file formats are correct:
   - Prompts: `.md` files
   - MCP servers: `.json` files
   - Commands: `.toml` files
   - Hooks: `.toml` files

For more help, see the [troubleshooting section](creating-extensions.md#troubleshooting) in the creating extensions guide.

