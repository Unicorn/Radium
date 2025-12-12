---
id: "extension-versioning"
title: "Extension Versioning Guide"
sidebar_label: "Extension Versioning Guide"
---

# Extension Versioning Guide

This guide explains semantic versioning for Radium extensions and how to manage compatibility.

## Semantic Versioning

Radium extensions use semantic versioning (semver): `MAJOR.MINOR.PATCH`

### Version Components

- **MAJOR**: Increment for breaking changes
- **MINOR**: Increment for new features (backward compatible)
- **PATCH**: Increment for bug fixes (backward compatible)

### Examples

- `1.0.0` - Initial release
- `1.0.1` - Bug fix (patch)
- `1.1.0` - New feature (minor)
- `2.0.0` - Breaking change (major)

## Version Format

### Valid Versions

- Must follow `MAJOR.MINOR.PATCH` format
- Each component is a non-negative integer
- No leading zeros (except for `0.x.x`)

**Valid**:
- `1.0.0`
- `2.1.3`
- `0.1.0`
- `10.20.30`

**Invalid**:
- `1.0` (missing patch)
- `1.0.0.0` (too many components)
- `v1.0.0` (no 'v' prefix)
- `1.0.0-beta` (no pre-release versions yet)

## When to Increment Versions

### MAJOR Version (Breaking Changes)

Increment MAJOR when:
- Removing components
- Changing component structure
- Changing required manifest fields
- Breaking backward compatibility
- Removing or changing dependencies

**Examples**:
- Removing a prompt file
- Changing command TOML structure
- Requiring new manifest fields
- Changing component directory structure

### MINOR Version (New Features)

Increment MINOR when:
- Adding new components
- Adding optional manifest fields
- Adding new functionality
- Enhancing existing components (backward compatible)

**Examples**:
- Adding a new prompt
- Adding a new MCP server config
- Adding optional metadata fields
- Improving component functionality

### PATCH Version (Bug Fixes)

Increment PATCH when:
- Fixing bugs
- Correcting documentation
- Fixing typos in manifests
- Improving error messages

**Examples**:
- Fixing incorrect glob patterns
- Correcting manifest field values
- Fixing component file formats
- Improving validation

## Radium Version Compatibility

### Documenting Compatibility

In your extension's README, document:

```markdown
## Requirements

- Radium version: 0.1.0 or later
- Dependencies: None
```

### Version Constraints (Future)

Future versions may support version constraints in manifests:

```json
{
  "radium_version": ">=0.1.0,<0.2.0"
}
```

For now, document compatibility in README.

## Breaking Changes Management

### Deprecation Strategy

When planning breaking changes:

1. **Deprecate first**: Mark components as deprecated in a MINOR release
2. **Document migration**: Provide migration guide
3. **Remove later**: Remove in next MAJOR release

### Migration Guides

For MAJOR version updates, provide migration guides:

```markdown
## Migration from 1.x to 2.x

### Breaking Changes

- Component structure changed
- New required manifest fields

### Migration Steps

1. Update manifest structure
2. Add new required fields
3. Reorganize components
```

## Versioning Best Practices

### Start with 1.0.0

- Don't start with `0.x.x` unless truly experimental
- `1.0.0` indicates a stable, usable extension
- `0.x.x` suggests the extension is still in development

### Consistent Versioning

- Use semantic versioning consistently
- Don't skip version numbers unnecessarily
- Document version changes in CHANGELOG

### Release Notes

Include release notes for each version:

```markdown
## Changelog

### 1.1.0 (2025-01-15)

- Added new code review agent for Python
- Improved error handling in commands
- Updated documentation

### 1.0.1 (2025-01-10)

- Fixed manifest validation issue
- Corrected component paths

### 1.0.0 (2025-01-01)

- Initial release
```

## Dependency Versioning

### Declaring Dependencies

Currently, dependencies are declared by name only:

```json
{
  "dependencies": ["required-extension"]
}
```

Future versions may support version constraints:

```json
{
  "dependencies": [
    {
      "name": "required-extension",
      "version": ">=1.0.0,<2.0.0"
    }
  ]
}
```

### Dependency Compatibility

- **Test with dependencies**: Ensure your extension works with dependency versions
- **Document requirements**: Specify which dependency versions are required
- **Handle missing dependencies**: Provide clear error messages

## Version Tagging

### Git Tags

Tag your releases:

```bash
# Tag a release
git tag v1.0.0
git push origin v1.0.0
```

### Release Archives

Create archives with version in filename:

```
my-extension-1.0.0.tar.gz
my-extension-1.1.0.tar.gz
```

## Backward Compatibility

### Maintaining Compatibility

- **Additive changes**: Prefer adding over modifying
- **Deprecation**: Mark old components as deprecated before removing
- **Documentation**: Document compatibility requirements
- **Testing**: Test with previous versions

### Breaking Changes

When making breaking changes:

1. **Plan ahead**: Announce breaking changes in advance
2. **Provide migration**: Give users a migration path
3. **Version appropriately**: Use MAJOR version increment
4. **Document clearly**: Explain what changed and why

## Version Comparison

### Comparing Versions

Radium uses semantic versioning comparison:

- `1.0.0` < `1.0.1` < `1.1.0` < `2.0.0`
- Pre-release versions (future): `1.0.0-alpha` < `1.0.0` < `1.0.1-beta`

### Version Requirements

When specifying version requirements (future):

- `>=1.0.0`: Version 1.0.0 or later
- `&lt;2.0.0`: Before version 2.0.0
- `~1.0.0`: Compatible with 1.0.x (>=1.0.0, &lt;1.1.0)
- `^1.0.0`: Compatible with 1.x.x (>=1.0.0, &lt;2.0.0)

## Troubleshooting Version Issues

### Invalid Version Format

**Problem**: Version validation fails

**Solution**: Ensure version follows `MAJOR.MINOR.PATCH` format

### Version Conflicts

**Problem**: Extension version conflicts with existing installation

**Solution**: Use `--overwrite` flag or uninstall first

### Compatibility Issues

**Problem**: Extension doesn't work with current Radium version

**Solution**: 
- Check Radium version requirements
- Update Radium if needed
- Check extension compatibility documentation

## See Also

- [Extension Distribution Guide](extension-distribution.md)
- [Extension Best Practices](extension-best-practices.md)
- [Extension Testing Guide](extension-testing.md)
- [Creating Extensions](../extensions/creating-extensions.md)

