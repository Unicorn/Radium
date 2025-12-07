# Extension System Smoke Test Checklist

Quick smoke test checklist for validating extension system functionality before releases.

## Pre-Release Checklist

### Installation
- [ ] Install extension from local directory
- [ ] Install extension from `.tar.gz` archive (if supported)
- [ ] Install extension with `--overwrite` flag
- [ ] Install extension with `--install-deps` flag
- [ ] Install extension with dependencies (auto-resolve)

### Discovery
- [ ] List installed extensions (`rad extension list`)
- [ ] List with `--verbose` flag
- [ ] List with `--json` flag
- [ ] Search extensions (`rad extension search`)
- [ ] Show extension info (`rad extension info`)

### Component Integration
- [ ] Extension agents appear in `rad agents list`
- [ ] Extension templates appear in `rad templates list`
- [ ] Extension commands are executable
- [ ] Components work correctly when used

### Management
- [ ] Update extension to new version
- [ ] Uninstall extension
- [ ] Verify no leftover files after uninstall
- [ ] Uninstall with dependency check (should fail if dependents exist)

### Error Handling
- [ ] Invalid manifest rejected with clear error
- [ ] Missing dependencies detected
- [ ] Component conflicts detected
- [ ] Path traversal attacks blocked
- [ ] Absolute paths rejected

### Platform Testing
- [ ] Test on Windows 10/11
- [ ] Test on macOS (Intel)
- [ ] Test on macOS (Apple Silicon)
- [ ] Test on Linux (Ubuntu)
- [ ] Test on Linux (Fedora)

## Quick Test Script

```bash
#!/bin/bash
# Quick smoke test for extension system

# Create test extension
mkdir -p test-ext/{agents,templates,commands}
echo '{"name":"test-ext","version":"1.0.0","description":"Test","author":"Test"}' > test-ext/radium-extension.json

# Install
rad extension install ./test-ext || exit 1

# List
rad extension list | grep test-ext || exit 1

# Info
rad extension info test-ext || exit 1

# Uninstall
rad extension uninstall test-ext || exit 1

# Verify removed
rad extension list | grep test-ext && exit 1

echo "Smoke test passed!"
```

## Critical Path Tests

These tests must pass for release:

1. **Installation**: Extension can be installed from directory
2. **Discovery**: Installed extensions are discoverable
3. **Components**: Extension components are integrated and usable
4. **Uninstall**: Extension can be cleanly removed
5. **Security**: Path traversal attacks are blocked

## Regression Tests

Verify these haven't regressed:

- [ ] Existing extensions still work after update
- [ ] Extension discovery performance is acceptable
- [ ] No memory leaks with many extensions
- [ ] CLI commands are backward compatible

## See Also

- [Manual Test Plan](manual-test-plan.md)
- [Test Scenarios](test-scenarios.md)

