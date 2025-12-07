---
req_id: REQ-018
title: Extension System
phase: LATER
status: Completed
priority: Low
estimated_effort: 8-10 hours
dependencies: [REQ-002, REQ-009]
related_docs:
  - docs/features/future-enhancements.md#extension-system
  - docs/features/gemini-cli-enhancements.md#extension-system
  - docs/guides/extension-system.md
---

# Extension System

## Problem Statement

Users need a way to share and install reusable agent configurations, MCP servers, and custom commands. Without an extension system, users cannot:
- Share agent configurations with others
- Install community-contributed extensions
- Package related components (prompts, MCP servers, commands) together
- Easily distribute custom workflows and tools

Modern AI tools (like gemini-cli) provide extension systems for community sharing. Radium needs an equivalent system that enables installable extension packages.

## Solution Overview

Implement an extension system that provides:
- Installable extension packages with manifest files
- Extension structure (prompts, MCP servers, custom commands)
- Extension discovery and installation
- Community sharing and discovery
- Extension validation and dependency management

The extension system enables community-contributed extensions, easy sharing of agent configurations, and a foundation for an extensible ecosystem.

## Functional Requirements

### FR-1: Extension Manifest

**Description**: Manifest file format for extension packages.

**Acceptance Criteria**:
- [x] Extension manifest format (radium-extension.json)
- [x] Extension metadata (name, version, description, author)
- [x] Extension components (prompts, MCP servers, commands)
- [x] Dependency declarations
- [x] Extension validation

**Implementation**: `crates/radium-core/src/extensions/manifest.rs`

### FR-2: Extension Structure

**Description**: Directory structure for extension packages.

**Acceptance Criteria**:
- [x] Extension package format
- [x] Component organization (prompts/, mcp/, commands/)
- [x] Extension installation location
- [x] Extension discovery mechanism

**Implementation**: `crates/radium-core/src/extensions/structure.rs`

### FR-3: Extension Installation

**Description**: Install and manage extension packages.

**Acceptance Criteria**:
- [x] Extension installation from local files
- [ ] Extension installation from URLs (skeleton implemented)
- [x] Extension uninstallation
- [x] Extension update mechanism
- [x] Dependency resolution

**Implementation**: `crates/radium-core/src/extensions/installer.rs`

### FR-4: Extension Discovery

**Description**: Discover and list installed extensions.

**Acceptance Criteria**:
- [x] List installed extensions
- [x] Extension search functionality
- [x] Extension metadata display
- [x] Extension validation

**Implementation**: `crates/radium-core/src/extensions/discovery.rs`

## Technical Requirements

### TR-1: Extension Manifest Format

**Description**: JSON format for extension manifests.

**Format**:
```json
{
  "name": "extension-name",
  "version": "1.0.0",
  "description": "Extension description",
  "author": "Author Name",
  "components": {
    "prompts": ["prompts/*.md"],
    "mcp_servers": ["mcp/*.json"],
    "commands": ["commands/*.toml"]
  },
  "dependencies": []
}
```

### TR-2: Extension API

**Description**: APIs for extension management.

**APIs**:
```rust
pub struct ExtensionManager {
    extensions_dir: PathBuf,
}

impl ExtensionManager {
    pub fn install(&self, extension_path: &Path) -> Result<()>;
    pub fn uninstall(&self, extension_name: &str) -> Result<()>;
    pub fn list(&self) -> Result<Vec<Extension>>;
    pub fn get(&self, name: &str) -> Result<Option<Extension>>;
}
```

## User Experience

### UX-1: Extension Installation

**Description**: Users install extensions via CLI.

**Example**:
```bash
$ rad extension install ./my-extension
Installing extension: my-extension
✓ Extension installed successfully
```

### UX-2: Extension Discovery

**Description**: Users discover installed extensions.

**Example**:
```bash
$ rad extension list
Installed Extensions:
  my-extension (1.0.0) - Custom agent configurations
  community-tools (2.1.0) - Community MCP servers
```

## Data Requirements

### DR-1: Extension Packages

**Description**: Extension package files.

**Location**: `~/.radium/extensions/` or project-specific location

**Format**: Directory with manifest and components

## Dependencies

- **REQ-002**: Agent Configuration - Required for agent system
- **REQ-009**: MCP Integration - Required for MCP server support

## Success Criteria

1. [x] Extension manifest format is defined and validated
2. [x] Extensions can be installed and uninstalled
3. [x] Extension components are properly integrated (integration helpers provided)
4. [x] Extension discovery works correctly
5. [x] All extension operations have comprehensive test coverage

**Completion Metrics**:
- **Status**: Completed
- **Priority**: Low
- **Estimated Effort**: 8-10 hours
- **Implementation Files**:
  - `crates/radium-core/src/extensions/manifest.rs` - Manifest format and validation
  - `crates/radium-core/src/extensions/structure.rs` - Extension structure and paths
  - `crates/radium-core/src/extensions/discovery.rs` - Extension discovery
  - `crates/radium-core/src/extensions/installer.rs` - Installation and management
  - `crates/radium-core/src/extensions/integration.rs` - Integration helpers
  - `apps/cli/src/commands/extension.rs` - CLI commands
  - `docs/guides/extension-system.md` - User guide
  - `examples/extensions/example-extension/` - Example extension package

## Out of Scope

- Extension marketplace (future enhancement)
- Extension versioning system (future enhancement)
- Extension signing and verification (future enhancement)

## References

- [Future Enhancements](../features/future-enhancements.md#extension-system)
- [Gemini CLI Enhancements](../features/gemini-cli-enhancements.md#extension-system)

