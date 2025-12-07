# Extension Manifest Reference

Complete reference for the `radium-extension.json` manifest file format.

## Schema

```json
{
  "name": "string (required)",
  "version": "string (required)",
  "description": "string (required)",
  "author": "string (required)",
  "components": {
    "prompts": ["string (glob patterns)"],
    "mcp_servers": ["string (file paths)"],
    "commands": ["string (glob patterns)"]
  },
  "dependencies": ["string (extension names)"]
}
```

## Field Descriptions

### name (required)

Extension name identifier.

**Rules:**
- Must start with an alphanumeric character
- Can contain alphanumeric characters, dashes (`-`), and underscores (`_`)
- Must be unique among installed extensions
- Examples: `my-extension`, `test_extension`, `extension123`

**Invalid examples:**
- `123extension` (starts with number)
- `my extension` (contains space)
- `my@extension` (contains special character)

### version (required)

Extension version number.

**Format:** Semantic versioning (semver)
- `major.minor.patch` (e.g., `1.0.0`)
- `major.minor` (e.g., `1.0`)
- `major` (e.g., `1`)

**Examples:**
- `1.0.0`
- `2.1.3`
- `0.1.0`

### description (required)

Human-readable description of the extension.

**Rules:**
- Non-empty string
- Can contain any Unicode characters
- Used in extension listings and help text

**Example:**
```json
"description": "Provides custom agents and templates for web development"
```

### author (required)

Extension author information.

**Format:** Author name, optionally with email
- `Author Name`
- `Author Name <email@example.com>`

**Example:**
```json
"author": "John Doe <john@example.com>"
```

### components (optional)

Component file declarations.

**Structure:**
```json
{
  "prompts": ["string (glob patterns)"],
  "mcp_servers": ["string (file paths)"],
  "commands": ["string (glob patterns)"]
}
```

All component arrays are optional. If a component type is not declared, that directory is not required.

#### prompts

Array of glob patterns for prompt files.

**Examples:**
```json
"prompts": ["prompts/*.md", "prompts/custom/*.md"]
```

**Rules:**
- Patterns are relative to extension root
- Supports glob wildcards (`*`, `?`)
- No path traversal (`..`) allowed
- No absolute paths allowed

#### mcp_servers

Array of file paths to MCP server configuration files.

**Examples:**
```json
"mcp_servers": ["mcp/server1.json", "mcp/server2.json"]
```

**Rules:**
- Exact file paths (no glob patterns)
- Must be JSON files
- Relative to extension root
- No path traversal allowed

#### commands

Array of glob patterns for command files.

**Examples:**
```json
"commands": ["commands/*.toml", "commands/deploy/*.toml"]
```

**Rules:**
- Patterns are relative to extension root
- Supports glob wildcards
- Must be TOML files
- No path traversal allowed

### dependencies (optional)

Array of extension names that this extension depends on.

**Example:**
```json
"dependencies": ["base-extension", "common-tools"]
```

**Rules:**
- Extension names must match installed extensions
- Dependencies are checked during installation
- Circular dependencies are detected and rejected
- Extensions with dependencies cannot be uninstalled if dependent extensions exist

## Complete Example

```json
{
  "name": "web-dev-tools",
  "version": "1.2.0",
  "description": "Web development agents, templates, and commands",
  "author": "Web Dev Team <team@example.com>",
  "components": {
    "prompts": [
      "prompts/*.md",
      "prompts/frameworks/*.md"
    ],
    "mcp_servers": [
      "mcp/npm-server.json",
      "mcp/docker-server.json"
    ],
    "commands": [
      "commands/*.toml",
      "commands/deploy/*.toml"
    ]
  },
  "dependencies": ["base-tools"]
}
```

## Validation Rules

The manifest is validated during:

1. **Load**: When reading the manifest file
2. **Installation**: Before installing the extension
3. **Update**: Before updating an extension

### Validation Checks

- All required fields are present and non-empty
- Extension name follows naming rules
- Version follows semver format
- Component paths are valid (no path traversal, no absolute paths)
- Component files exist (if not using glob patterns)
- Component file syntax is valid (TOML/JSON parsing)
- Dependencies are installed (if specified)

## Error Messages

Common validation errors and their meanings:

- `missing required field: name` - Extension name is missing or empty
- `invalid extension name: 'xxx'` - Name doesn't follow naming rules
- `invalid version format: 'xxx'` - Version doesn't follow semver
- `component file not found: xxx` - Referenced component file doesn't exist
- `invalid component path (security): xxx` - Path contains unsafe elements
- `component file syntax error in xxx: ...` - Component file has syntax errors
- `missing dependency: 'xxx'` - Required dependency is not installed

## See Also

- [Creating Extensions](creating-extensions.md) - Step-by-step guide
- [Extension System Guide](../guides/extension-system.md) - User guide

