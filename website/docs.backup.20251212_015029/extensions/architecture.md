# Extension System Architecture

This document describes the technical architecture of the Radium extension system.

## Overview

The extension system is implemented in `crates/radium-core/src/extensions/` and consists of seven core modules:

1. **manifest.rs** - Manifest parsing and validation
2. **structure.rs** - Extension directory structure and validation
3. **installer.rs** - Installation from multiple sources with dependency resolution
4. **discovery.rs** - Extension discovery and search
5. **integration.rs** - Integration helpers for system components
6. **conflict.rs** - Conflict detection for file and component overlaps
7. **validator.rs** - Extension validation logic

## Module Breakdown

### Manifest Module (`manifest.rs`)

The manifest module handles parsing and validation of `radium-extension.json` files.

**Key Types:**
- `ExtensionManifest` - Main manifest structure
- `ExtensionComponents` - Component path definitions
- `ExtensionManifestError` - Manifest-specific errors

**Validation Rules:**
- Name must be alphanumeric with dashes/underscores, starting with a letter
- Version must follow semantic versioning (semver)
- Component paths cannot be empty
- All required fields must be present

**Example:**
```rust
let manifest = ExtensionManifest::load(&manifest_path)?;
manifest.validate()?;
```

### Structure Module (`structure.rs`)

The structure module defines extension directory organization and provides path resolution.

**Key Types:**
- `Extension` - Represents an installed extension
- `ExtensionStructureError` - Structure-specific errors

**Directory Constants:**
- `COMPONENT_PROMPTS` - "prompts"
- `COMPONENT_MCP` - "mcp"
- `COMPONENT_COMMANDS` - "commands"
- `COMPONENT_HOOKS` - "hooks"
- `MANIFEST_FILE` - "radium-extension.json"

**Path Resolution:**
- `prompts_dir()` - Returns path to prompts directory
- `mcp_dir()` - Returns path to MCP directory
- `commands_dir()` - Returns path to commands directory
- `hooks_dir()` - Returns path to hooks directory

**Installation Locations:**
- User-level: `~/.radium/extensions/`
- Workspace-level: `.radium/extensions/` (project root)

### Installer Module (`installer.rs`)

The installer module handles installation from multiple sources with dependency resolution.

**Key Types:**
- `ExtensionManager` - Main installation manager
- `InstallOptions` - Installation configuration
- `ExtensionInstallerError` - Installer-specific errors

**Installation Sources:**
- Local directories
- Archive files (`.tar.gz`, `.zip`)
- URLs (HTTP/HTTPS)

**Installation Flow:**
1. Validate source and manifest
2. Check for conflicts
3. Resolve dependencies (if `install_dependencies` is true)
4. Copy files to extensions directory
5. Validate structure (if `validate_after_install` is true)

**Dependency Resolution:**
- Recursively installs declared dependencies
- Detects dependency cycles
- Validates dependency versions (future enhancement)

### Discovery Module (`discovery.rs`)

The discovery module finds and loads installed extensions.

**Key Types:**
- `ExtensionDiscovery` - Discovery service
- `DiscoveryOptions` - Discovery configuration
- `ExtensionDiscoveryError` - Discovery-specific errors

**Discovery Methods:**
- `discover_all()` - Discovers all extensions in search paths
- `discover_in_directory()` - Discovers extensions in specific directory
- `get()` - Gets extension by name
- `search()` - Searches extensions by query

**Search Paths:**
- Default: `~/.radium/extensions/`
- Custom: Can be configured via `DiscoveryOptions`

### Integration Module (`integration.rs`)

The integration module provides helper functions for integrating extension components into system components.

**Integration Functions:**
- `get_extension_prompt_dirs()` - Returns prompt directories from all extensions
- `get_extension_command_dirs()` - Returns command directories from all extensions
- `get_extension_mcp_configs()` - Returns MCP config paths from all extensions
- `get_extension_hook_paths()` - Returns hook file paths from all extensions
- `get_all_extensions()` - Returns all installed extensions

**Usage Pattern:**
```rust
let prompt_dirs = get_extension_prompt_dirs()?;
for dir in prompt_dirs {
    // Load prompts from directory
}
```

### Conflict Module (`conflict.rs`)

The conflict module detects conflicts between extension components and existing components.

**Key Types:**
- `ConflictDetector` - Conflict detection service
- `ConflictError` - Conflict-specific errors

**Conflict Checks:**
- Agent ID conflicts
- Template name conflicts
- Command name conflicts
- Dependency cycles

**Conflict Detection Flow:**
1. Discover existing components (agents, templates, commands)
2. Scan extension package for components
3. Compare names/IDs
4. Report conflicts before installation

### Validator Module (`validator.rs`)

The validator module provides comprehensive extension validation.

**Validation Checks:**
- Manifest validity
- Structure correctness
- Component file existence
- Path resolution
- Dependency availability

## Integration Points

### Agent Discovery Integration

**Location:** `crates/radium-core/src/agents/discovery.rs`

Agent discovery should call `get_extension_prompt_dirs()` to include extension prompts in the search paths. Extension prompts are discovered with lower precedence than built-in prompts.

**Integration Pattern:**
```rust
let mut search_paths = default_search_paths();
let extension_dirs = get_extension_prompt_dirs()?;
search_paths.extend(extension_dirs);
```

### MCP System Integration

**Location:** `crates/radium-core/src/mcp/config.rs`

The MCP config manager should call `get_extension_mcp_configs()` during initialization to load extension-provided MCP server configurations. Extension configs are merged with workspace configs.

**Integration Pattern:**
```rust
let mut servers = load_workspace_configs()?;
let extension_configs = get_extension_mcp_configs()?;
for config_path in extension_configs {
    let config = load_mcp_config(&config_path)?;
    servers.push(config);
}
```

### Command Registry Integration

**Location:** `crates/radium-core/src/commands/custom.rs`

The command registry already integrates with extensions via `get_extension_command_dirs()`. Extension commands are discovered with lower precedence than user/project commands.

**Current Implementation:**
- Extension commands are loaded first (lowest precedence)
- User commands override extension commands
- Project commands override both (highest precedence)
- Extension commands are namespaced with extension name

## Conflict Detection Mechanism

The conflict detection system prevents installation of extensions that would conflict with existing components.

### Conflict Types

1. **Agent Conflicts**: Extension agent IDs match existing agent IDs
2. **Template Conflicts**: Extension template names match existing templates
3. **Command Conflicts**: Extension command names match existing commands
4. **Dependency Cycles**: Circular dependencies between extensions

### Detection Flow

1. Before installation, `ConflictDetector::check_conflicts()` is called
2. System discovers existing components
3. Extension package is scanned for components
4. Names/IDs are compared
5. Conflicts are reported as errors

### Resolution

- Conflicts prevent installation by default
- Use `--overwrite` flag to allow overwriting (future: may allow selective overwrite)
- Dependency cycles are always errors

## Dependency Resolution Flow

When installing an extension with dependencies:

1. Parse manifest and extract dependencies
2. Check if dependencies are already installed
3. For each missing dependency:
   - Resolve dependency source (if provided)
   - Install dependency recursively
   - Validate dependency installation
4. Check for dependency cycles
5. Install main extension

### Dependency Cycle Detection

The system detects cycles by:
1. Building dependency graph
2. Performing depth-first search
3. Detecting back edges (cycles)

## Extension Discovery Order

Extensions are discovered in the following order (precedence from highest to lowest):

1. **Project-level extensions** (`.radium/extensions/`)
2. **User-level extensions** (`~/.radium/extensions/`)

Within each level, extensions are processed in alphabetical order by name.

## Component Discovery Order

### Prompts

1. Built-in prompts (from `prompts/` directory)
2. Project-level extension prompts
3. User-level extension prompts

### MCP Servers

1. Workspace MCP config (`.radium/mcp-servers.toml`)
2. Extension MCP configs (merged)

### Commands

1. Extension commands (lowest precedence)
2. User commands
3. Project commands (highest precedence)

Extension commands are namespaced: `extension-name:command-name`

## Error Handling

All extension operations use structured error types:

- `ExtensionManifestError` - Manifest parsing/validation errors
- `ExtensionStructureError` - Structure validation errors
- `ExtensionInstallerError` - Installation errors
- `ExtensionDiscoveryError` - Discovery errors
- `ConflictError` - Conflict detection errors
- `ExtensionValidationError` - Validation errors

These are unified into `ExtensionError` for public API.

## Testing

Comprehensive test coverage includes:

- Unit tests for each module
- Integration tests for installation workflows
- Edge case tests for validation
- Security tests for path traversal prevention
- Conflict detection tests
- Archive handling tests
- Benchmark tests for performance

Test files are located in `crates/radium-core/tests/extension_*_test.rs`.

## Future Enhancements

Potential future enhancements (out of scope for REQ-102):

- Extension marketplace
- Extension versioning system
- Extension signing and verification
- Extension update mechanism
- Extension metadata search
- Extension ratings/reviews

