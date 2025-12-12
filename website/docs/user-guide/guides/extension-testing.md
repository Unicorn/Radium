---
id: "extension-testing"
title: "Extension Testing Guide"
sidebar_label: "Extension Testing Guide"
---

# Extension Testing Guide

This guide explains how to test your Radium extension before distribution.

## Local Testing Workflow

### 1. Validate Manifest

First, ensure your manifest is valid:

```bash
# Check JSON syntax
cat radium-extension.json | jq .

# Or use a JSON validator online
```

### 2. Test Installation

Install your extension locally:

```bash
# From extension directory
rad extension install ./my-extension

# Verify installation
rad extension list
rad extension info my-extension
```

### 3. Verify Components

Check that all components are discoverable:

```bash
# Check prompts (if applicable)
rad agents list

# Check commands (if applicable)
rad commands list

# Check MCP servers (if applicable)
rad mcp list

# Check hooks (if applicable)
rad hooks list
```

### 4. Test Functionality

Test that components actually work:

- **Prompts**: Create an agent using the prompt
- **Commands**: Execute the command
- **MCP Servers**: Connect and use the MCP server
- **Hooks**: Verify hook is loaded and functional

### 5. Test Uninstallation

Verify clean removal:

```bash
rad extension uninstall my-extension

# Verify components are removed
rad extension list
rad agents list  # Should not show extension agents
```

## Validation Checklist

Use this checklist before distribution:

### Manifest Validation

- [ ] Manifest is valid JSON
- [ ] All required fields present (name, version, description, author)
- [ ] Version follows semantic versioning
- [ ] Name follows naming conventions
- [ ] Component paths are valid glob patterns
- [ ] Dependencies are correctly declared

### Structure Validation

- [ ] All directories referenced in manifest exist
- [ ] All files referenced in manifest exist
- [ ] File formats are correct (.md for prompts, .json for MCP, .toml for commands/hooks)
- [ ] No extra files that aren't declared

### Component Validation

- [ ] Prompt files are valid markdown
- [ ] MCP server configs are valid JSON
- [ ] Command files are valid TOML
- [ ] Hook files are valid TOML
- [ ] All components can be loaded without errors

### Integration Validation

- [ ] Extension installs without errors
- [ ] All components are discoverable after installation
- [ ] Components work as expected
- [ ] Extension uninstalls cleanly
- [ ] No conflicts with other extensions

## Testing with Different Radium Versions

If possible, test your extension with:

- **Latest stable version**: Ensure compatibility
- **Previous version**: Check backward compatibility
- **Development version**: Test with upcoming features

### Version Compatibility

- Document minimum Radium version required
- Test with version constraints in manifest (if supported)
- Provide migration notes for breaking changes

## Testing Dependencies

### Install with Dependencies

Test dependency resolution:

```bash
# Install extension with dependencies
rad extension install ./my-extension --install-deps

# Verify dependencies are installed
rad extension list
```

### Test Dependency Scenarios

- [ ] Extension installs when dependencies are already installed
- [ ] Extension installs when dependencies need to be installed
- [ ] Extension fails gracefully when dependencies are missing
- [ ] Dependency conflicts are detected

## Testing Component Integration

### Agent/Prompt Integration

If your extension provides prompts:

```bash
# Install extension
rad extension install ./my-extension

# Verify prompts are discoverable
rad agents list

# Create an agent using the prompt
rad agents create test-agent "Test Agent" --prompt-path prompts/my-agent.md

# Verify agent works
rad agents info test-agent
```

### MCP Server Integration

If your extension provides MCP servers:

```bash
# Install extension
rad extension install ./my-extension

# Verify MCP configs are loaded
rad mcp list

# Test MCP server connection
# (Follow MCP-specific testing procedures)
```

### Command Integration

If your extension provides commands:

```bash
# Install extension
rad extension install ./my-extension

# Verify commands are discoverable
rad commands list

# Execute a command
rad commands execute my-extension:my-command
```

### Workflow Template Integration

If your extension provides workflow templates:

```bash
# Install extension
rad extension install ./my-extension

# Verify templates are discoverable
rad workflows list

# Execute a workflow template
rad workflows execute my-workflow-template
```

## Testing Edge Cases

### Installation Edge Cases

Test these scenarios:

- [ ] Installing over existing extension (with and without --overwrite)
- [ ] Installing with missing dependencies
- [ ] Installing with invalid manifest
- [ ] Installing with missing component files
- [ ] Installing from URL
- [ ] Installing from archive

### Conflict Testing

- [ ] Test with extensions that have similar names
- [ ] Test with extensions that provide similar components
- [ ] Verify conflict detection works
- [ ] Test overwrite behavior

### Error Handling

- [ ] Invalid manifest format
- [ ] Missing required files
- [ ] Invalid component formats
- [ ] Network errors (for URL installation)
- [ ] Permission errors

## Automated Testing

### Validation Script

Create a simple validation script:

```bash
#!/bin/bash
# validate-extension.sh

EXTENSION_DIR=$1

echo "Validating extension: $EXTENSION_DIR"

# Check manifest exists
if [ ! -f "$EXTENSION_DIR/radium-extension.json" ]; then
    echo "ERROR: Manifest not found"
    exit 1
fi

# Validate JSON
if ! jq . "$EXTENSION_DIR/radium-extension.json" > /dev/null 2>&1; then
    echo "ERROR: Invalid JSON in manifest"
    exit 1
fi

# Try installation
if ! rad extension install "$EXTENSION_DIR"; then
    echo "ERROR: Installation failed"
    exit 1
fi

# Verify installation
if ! rad extension list | grep -q "$(jq -r .name "$EXTENSION_DIR/radium-extension.json")"; then
    echo "ERROR: Extension not found after installation"
    exit 1
fi

echo "Validation passed!"
```

### Test Extension

Create a test extension that exercises all features:

```bash
# Create test extension
rad extension create test-extension --author "Test" --description "Test extension"

# Add components
# ... add test components ...

# Test installation
rad extension install ./test-extension

# Test components
# ... test each component type ...

# Cleanup
rad extension uninstall test-extension
```

## Testing Checklist

Before distributing your extension:

### Pre-Distribution

- [ ] All validation checks pass
- [ ] Extension installs successfully
- [ ] All components are discoverable
- [ ] Components work as expected
- [ ] Extension uninstalls cleanly
- [ ] No conflicts with common extensions
- [ ] README is complete and accurate
- [ ] Examples work as documented

### Post-Distribution

- [ ] Test installation from URL
- [ ] Verify archive structure
- [ ] Test on different platforms (if applicable)
- [ ] Get feedback from beta testers
- [ ] Address reported issues

## Troubleshooting Test Failures

### Installation Fails

**Check**:
- Manifest validity
- File structure
- Component paths
- Required fields

### Components Not Discoverable

**Check**:
- Glob patterns in manifest
- File locations
- File formats
- Extension installation location

### Components Don't Work

**Check**:
- File content validity
- Integration points
- Dependencies
- Configuration

## See Also

- [Extension Distribution Guide](extension-distribution.md)
- [Extension Best Practices](extension-best-practices.md)
- [Extension Versioning Guide](extension-versioning.md)
- [Creating Extensions](../extensions/creating-extensions.md)

