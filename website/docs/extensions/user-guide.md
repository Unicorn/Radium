---
id: "user-guide"
title: "Extension System User Guide"
sidebar_label: "Extension System User Guide"
---

# Extension System User Guide

Complete guide to using the Radium extension system for installing, managing, and working with extensions.

## Table of Contents

- [Overview](#overview)
- [Installation Commands](#installation-commands)
- [Discovery Commands](#discovery-commands)
- [Management Commands](#management-commands)
- [Signing and Verification](#signing-and-verification)
- [Marketplace Integration](#marketplace-integration)
- [Troubleshooting](#troubleshooting)
- [Migration Guide](#migration-guide)

## Overview

The Radium extension system allows you to install and manage reusable packages containing:
- **Prompts**: Agent prompt templates
- **MCP Servers**: Model Context Protocol server configurations
- **Commands**: Custom CLI commands
- **Hooks**: Native libraries or WASM modules

Extensions can be installed from:
- Local directories
- Archive files (`.tar.gz`, `.zip`)
- URLs (HTTP/HTTPS)
- Marketplace by name

## Installation Commands

### Install Extension

Install an extension from various sources:

```bash
# Install from local directory
rad extension install ./my-extension

# Install from archive file
rad extension install ./my-extension.tar.gz

# Install from URL
rad extension install https://example.com/extensions/my-extension.tar.gz

# Install from marketplace by name
rad extension install marketplace-extension-name

# Install and overwrite existing extension
rad extension install ./my-extension --overwrite

# Install with automatic dependency resolution
rad extension install ./my-extension --install-deps
```

**Options:**
- `--overwrite`: Overwrite an existing extension with the same name
- `--install-deps`: Automatically install all declared dependencies

**Examples:**

```bash
# Install a local extension
$ rad extension install ./code-review-tools
Installing extension from: ./code-review-tools
Validating extension package...
✓ Extension 'code-review-tools' installed successfully
  Version: 1.0.0
  Description: Code review agent prompts and tools

# Install from marketplace
$ rad extension install github-integration
Found extension 'github-integration' in marketplace
Downloading from: https://marketplace.radium.ai/extensions/github-integration.tar.gz
✓ Extension 'github-integration' installed successfully
```

### Create Extension Template

Create a new extension with the proper structure:

```bash
# Basic extension
rad extension create my-extension

# With author and description
rad extension create my-extension --author "Your Name" --description "My custom extension"
```

This creates a directory structure:
```
my-extension/
├── radium-extension.json
├── prompts/
├── mcp/
├── commands/
├── hooks/
└── README.md
```

## Discovery Commands

### List Extensions

List all installed extensions:

```bash
# Basic listing
rad extension list

# Detailed information
rad extension list --verbose

# JSON output
rad extension list --json
```

**Example Output:**

```bash
$ rad extension list
┌─────────────────────┬─────────┬──────────────────────────────┐
│ Name                │ Version │ Description                  │
├─────────────────────┼─────────┼──────────────────────────────┤
│ code-review-tools   │ 1.0.0   │ Code review agent prompts    │
│ github-integration  │ 2.1.0   │ GitHub API integration      │
│ mcp-database-tools  │ 1.5.0   │ Database MCP server configs  │
└─────────────────────┴─────────┴──────────────────────────────┘
```

### Search Extensions

Search for extensions by name or description:

```bash
# Search local and marketplace
rad extension search "github"

# Search only marketplace
rad extension search "github" --marketplace-only

# Search only local extensions
rad extension search "github" --local-only

# JSON output
rad extension search "github" --json
```

**Example:**

```bash
$ rad extension search "code review"
Found 3 extensions:

Local:
  code-review-tools (1.0.0) - Code review agent prompts

Marketplace:
  advanced-code-review (2.0.0) - Advanced code review tools
  code-review-assistant (1.5.0) - AI-powered code review assistant
```

### Browse Marketplace

Browse popular extensions from the marketplace:

```bash
# Browse popular extensions
rad extension browse

# JSON output
rad extension browse --json
```

### Get Extension Information

Show detailed information about a specific extension:

```bash
# Show extension info
rad extension info my-extension

# JSON output
rad extension info my-extension --json
```

**Example:**

```bash
$ rad extension info code-review-tools
Extension: code-review-tools
Version: 1.0.0
Author: Radium Team
Description: Code review agent prompts and tools

Components:
  - Prompts: 5 files
  - Commands: 2 files

Dependencies: None

Installation Path: ~/.radium/extensions/code-review-tools
```

## Management Commands

### Uninstall Extension

Remove an installed extension:

```bash
rad extension uninstall my-extension
```

**Example:**

```bash
$ rad extension uninstall code-review-tools
Uninstalling extension 'code-review-tools'...
✓ Extension 'code-review-tools' uninstalled successfully
```

## Signing and Verification

### Sign Extension

Sign an extension with a private key for authenticity:

```bash
# Sign with existing key
rad extension sign ./my-extension --key-file ./private.key

# Generate new keypair and sign
rad extension sign ./my-extension --generate-key
```

**Example:**

```bash
$ rad extension sign ./my-extension --generate-key
Generated new keypair:
  Private key: ./private.key (keep secure!)
  Public key: ./public.key (share with users)
✓ Extension signed successfully
  Signature: radium-extension.json.sig
```

### Verify Extension

Verify an extension's signature:

```bash
# Verify installed extension
rad extension verify my-extension

# Verify with specific public key
rad extension verify my-extension --key-file ./public.key
```

**Example:**

```bash
$ rad extension verify my-extension
✓ Extension signature verified
  Signer: Your Name
  Signed: 2025-12-07T10:30:00Z
```

### Manage Trusted Keys

Manage trusted signing keys:

```bash
# Add trusted key
rad extension trust-key add --name "Publisher Name" --key-file ./public.key

# List trusted keys
rad extension trust-key list

# Remove trusted key
rad extension trust-key remove --name "Publisher Name"
```

## Marketplace Integration

### Publishing Extensions

Publish your extension to the marketplace:

```bash
# Publish with API key from environment
rad extension publish ./my-extension

# Publish with API key provided
rad extension publish ./my-extension --api-key YOUR_API_KEY

# Publish with automatic signing
rad extension publish ./my-extension --api-key YOUR_API_KEY --sign-with-key ./private.key
```

See the [Publishing Guide](publishing-guide.md) for detailed instructions.

### Installing from Marketplace

Install extensions directly from the marketplace:

```bash
# Install by name (automatically detects marketplace)
rad extension install extension-name

# Search first, then install
rad extension search "query"
rad extension install found-extension-name
```

## Extension Manifest Format

Extensions use a `radium-extension.json` manifest file. Here's the complete schema:

```json
{
  "name": "extension-name",
  "version": "1.0.0",
  "description": "Extension description",
  "author": "Author Name",
  "components": {
    "prompts": ["prompts/*.md"],
    "mcp_servers": ["mcp/*.json"],
    "commands": ["commands/*.toml"],
    "hooks": ["hooks/*.toml"]
  },
  "dependencies": ["other-extension-1", "other-extension-2"],
  "metadata": {
    "tags": ["tag1", "tag2"],
    "category": "development",
    "license": "MIT"
  },
  "signature": "base64-encoded-signature"
}
```

### Field Descriptions

- **name** (required): Extension name (alphanumeric, dashes, underscores; must start with letter)
- **version** (required): Semantic version (e.g., `1.0.0`)
- **description** (required): Brief description of the extension
- **author** (required): Author name or contact information
- **components** (optional): Component path definitions using glob patterns
  - `prompts`: Array of prompt file paths (`.md` files)
  - `mcp_servers`: Array of MCP server config paths (`.json` files)
  - `commands`: Array of command file paths (`.toml` files)
  - `hooks`: Array of hook file paths (`.toml` files)
- **dependencies** (optional): Array of extension names this extension depends on
- **metadata** (optional): Additional metadata (tags, category, license, etc.)
- **signature** (optional): Cryptographic signature for verification

### Component Path Patterns

Component paths support glob patterns:

```json
{
  "components": {
    "prompts": [
      "prompts/*.md",              // All .md files in prompts/
      "prompts/frameworks/*.md",    // All .md files in prompts/frameworks/
      "prompts/agents/code-review.md"  // Specific file
    ],
    "mcp_servers": ["mcp/*.json"],
    "commands": ["commands/*.toml", "commands/deploy/*.toml"],
    "hooks": ["hooks/*.toml"]
  }
}
```

## Installation Locations

Extensions can be installed in two locations:

1. **User-level**: `~/.radium/extensions/` - Available to all projects
2. **Project-level**: `.radium/extensions/` - Specific to the current workspace

Project-level extensions take precedence over user-level extensions when both exist.

## Troubleshooting

### Extension Not Found

If an extension isn't being discovered:

1. **Verify installation:**
   ```bash
   rad extension list
   ```

2. **Check manifest exists:**
   ```bash
   ls ~/.radium/extensions/my-extension/radium-extension.json
   ```

3. **Verify extension name** (case-sensitive):
   ```bash
   rad extension info my-extension
   ```

4. **Check installation location:**
   - User-level: `~/.radium/extensions/`
   - Project-level: `.radium/extensions/`

### Installation Errors

**Invalid manifest:**
- Check that `radium-extension.json` is valid JSON
- Ensure all required fields are present: `name`, `version`, `description`, `author`
- Verify JSON syntax with a JSON validator

**Version format error:**
- Version must follow semantic versioning: `MAJOR.MINOR.PATCH`
- Examples: `1.0.0`, `2.1.3`, `0.1.0`
- Invalid: `1.0`, `v1.0.0`, `1.0.0-beta`

**Name format error:**
- Must start with a letter
- Can contain letters, numbers, dashes, underscores
- No spaces or special characters
- Examples: ✅ `my-extension`, ✅ `extension_123`, ❌ `123-extension`, ❌ `my extension`

**Path conflicts:**
- Use `--overwrite` flag if installing over an existing extension
- Or uninstall the existing extension first

### Component Not Loading

If extension components aren't being discovered:

1. **Verify component directories exist:**
   ```bash
   ls ~/.radium/extensions/my-extension/prompts/
   ```

2. **Check glob patterns match files:**
   - Manifest: `"prompts": ["prompts/*.md"]`
   - Actual files: `prompts/agent.md` ✅
   - Actual files: `prompts/agent.txt` ❌ (wrong extension)

3. **Ensure correct file formats:**
   - Prompts: `.md` files
   - MCP servers: `.json` files
   - Commands: `.toml` files
   - Hooks: `.toml` files

4. **Check file permissions:**
   ```bash
   ls -la ~/.radium/extensions/my-extension/
   ```

### Dependency Issues

**Dependencies not installing:**
- Ensure dependency names match exactly (case-sensitive)
- Check dependencies are available (installed or in marketplace)
- Use `--install-deps` flag during installation

**Dependency conflicts:**
- Check for version conflicts between extensions
- Review dependency declarations in manifests
- Consider using different extension versions

### Marketplace Issues

**Cannot connect to marketplace:**
- Check internet connection
- Verify marketplace URL is accessible
- Check for firewall/proxy issues

**Authentication errors:**
- Verify API key is correct
- Check API key hasn't expired
- Ensure you have publishing permissions

**Extension not found in marketplace:**
- Verify extension name is correct
- Check if extension is published
- Try searching instead of installing by name

### Signature Verification Errors

**Signature verification fails:**
- Ensure extension was signed
- Verify public key is correct
- Check signature file exists: `radium-extension.json.sig`
- Try adding the publisher's public key as trusted

## Migration Guide

### From Manual Installation

If you've been manually copying extension files, migrate to the extension system:

1. **Identify your extensions:**
   - List files in `~/.radium/prompts/`, `~/.radium/mcp/`, etc.
   - Group related files by extension

2. **Create extension structure:**
   ```bash
   rad extension create my-extension --author "Your Name" --description "Description"
   ```

3. **Move files to extension:**
   ```bash
   # Move prompts
   mv ~/.radium/prompts/my-prompts/* my-extension/prompts/
   
   # Move MCP configs
   mv ~/.radium/mcp/my-mcp.json my-extension/mcp/
   
   # Move commands
   mv ~/.radium/commands/my-command.toml my-extension/commands/
   ```

4. **Update manifest:**
   Edit `my-extension/radium-extension.json` to include component paths:
   ```json
   {
     "components": {
       "prompts": ["prompts/*.md"],
       "mcp_servers": ["mcp/*.json"],
       "commands": ["commands/*.toml"]
     }
   }
   ```

5. **Install extension:**
   ```bash
   rad extension install ./my-extension
   ```

6. **Verify installation:**
   ```bash
   rad extension list
   rad extension info my-extension
   ```

7. **Clean up old files** (after verifying extension works):
   ```bash
   # Backup first!
   rm -rf ~/.radium/prompts/my-prompts/
   rm ~/.radium/mcp/my-mcp.json
   ```

### Benefits of Migration

- **Version management**: Track extension versions
- **Dependency resolution**: Automatic dependency installation
- **Easy updates**: Update extensions with single command
- **Marketplace integration**: Share extensions easily
- **Signing support**: Verify extension authenticity
- **Better organization**: Clear extension boundaries

## Next Steps

- [Creating Extensions](creating-extensions.md) - Learn how to create your own extensions
- [Publishing Guide](publishing-guide.md) - Publish extensions to the marketplace
- [Marketplace Guide](marketplace.md) - Discover and use marketplace features
- [Architecture](architecture.md) - Understand the technical architecture
- [Examples](../examples/extensions/) - See example extensions

