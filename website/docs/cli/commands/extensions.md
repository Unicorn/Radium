---
id: "extensions"
title: "Extension Management Commands"
sidebar_label: "Extension Management Commands"
---

# Extension Management Commands

Commands for installing and managing extension packages.

## `rad extension`

Manage extension packages.

### Subcommands

#### `list`

List installed extensions.

```bash
rad extension list [--json] [--verbose]
```

#### `install <source>`

Install an extension from source.

```bash
rad extension install <source> [--overwrite] [--no-deps]
```

Options:
- `--overwrite` - Overwrite existing extension
- `--no-deps` - Skip dependency installation

#### `uninstall <name>`

Uninstall an extension.

```bash
rad extension uninstall <name>
```

#### `info <name>`

Show extension information.

```bash
rad extension info <name> [--json]
```

#### `search <query>`

Search for extensions.

```bash
rad extension search <query> [--json]
```

#### `create <name>`

Create a new extension template.

```bash
rad extension create <name> [--author <author>] [--description <desc>]
```

### Examples

```bash
# List extensions
rad extension list

# Install extension
rad extension install ./my-extension

# Install from URL
rad extension install https://example.com/extension.zip

# Uninstall extension
rad extension uninstall my-extension

# Get extension info
rad extension info my-extension

# Search extensions
rad extension search "mcp"

# Create new extension
rad extension create my-extension --author "John Doe" --description "My extension"
```

