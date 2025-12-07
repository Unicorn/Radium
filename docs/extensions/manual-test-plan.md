# Extension System Manual Testing Plan

This document outlines manual testing scenarios for the extension system to validate functionality from an end-user perspective.

## Test Environment Setup

### Prerequisites
- Radium CLI installed and working
- Access to create test extensions
- Test extensions available (see `examples/extensions/`)

### Test Platforms
- [ ] macOS (Intel)
- [ ] macOS (Apple Silicon)
- [ ] Linux (Ubuntu)
- [ ] Linux (Fedora)
- [ ] Windows 10/11

## Test Scenarios

### Scenario 1: First-Time Extension Installation

**User Persona**: Extension Consumer (non-technical)

**Steps:**
1. Download or create a test extension
2. Run `rad extension install ./test-extension`
3. Verify installation success message
4. Run `rad extension list`
5. Verify extension appears in list
6. Run `rad extension info test-extension`
7. Verify extension details are displayed
8. Use extension components (if applicable)

**Validation:**
- [ ] Clear progress feedback during installation
- [ ] Success message shows component counts
- [ ] Components immediately available after installation
- [ ] Help text shows new commands (if any)

**Pass Criteria**: User completes workflow without consulting documentation

---

### Scenario 2: Creating and Testing Custom Extension

**User Persona**: Extension Creator (technical)

**Steps:**
1. Create extension directory structure
2. Create `radium-extension.json` manifest
3. Add custom agent configuration to `agents/`
4. Add workflow template to `templates/`
5. Add custom command to `commands/`
6. Run `rad extension install ./my-extension`
7. Test all components work
8. Package as `.tar.gz` (optional)
9. Share with colleague for testing

**Validation:**
- [ ] Manifest validation provides helpful errors for invalid manifests
- [ ] Local installation works from directory
- [ ] All components discovered correctly
- [ ] Components are usable (agents, templates, commands)
- [ ] Archive creation successful (if implemented)
- [ ] Colleague can install from archive

**Pass Criteria**: Creator successfully shares working extension

---

### Scenario 3: Managing Extension Dependencies

**User Persona**: Extension Consumer

**Steps:**
1. Install extension A (has no dependencies)
2. Verify extension A works
3. Install extension B (depends on A)
4. Verify extension B installs successfully
5. Attempt to uninstall A
6. Verify warning about B's dependency
7. Uninstall B first
8. Then uninstall A

**Validation:**
- [ ] Dependency resolution automatic during installation
- [ ] Clear warning when dependency conflict during uninstall
- [ ] Uninstall order enforced (dependents must be removed first)

**Pass Criteria**: User understands dependency relationships

---

### Scenario 4: Handling Installation Errors

**User Persona**: Extension Consumer

**Test Cases:**

#### 4.1: Non-existent Extension
- [ ] Attempt to install `./non-existent-extension`
- [ ] Verify clear error message

#### 4.2: Corrupted Archive
- [ ] Attempt to install corrupted `.tar.gz` file
- [ ] Verify error message about corruption

#### 4.3: Missing Dependencies
- [ ] Attempt to install extension with missing dependencies
- [ ] Verify error message lists missing dependencies

#### 4.4: Conflicting Extension
- [ ] Install extension with conflicting component IDs
- [ ] Verify conflict error with details

**Validation:**
- [ ] Each error has clear, actionable message
- [ ] No partial installations left behind
- [ ] User knows how to fix the issue

**Pass Criteria**: User can resolve errors without support

---

### Scenario 5: Extension Upgrade

**User Persona**: Extension Consumer

**Steps:**
1. Install extension v1.0.0
2. Verify components work
3. Download extension v2.0.0
4. Run `rad extension install ./extension-v2 --overwrite`
5. Verify upgrade success
6. Verify components updated
7. Verify old version removed

**Validation:**
- [ ] Upgrade process is clear
- [ ] Components updated correctly
- [ ] No leftover files from old version

**Pass Criteria**: Upgrade completes successfully

---

### Scenario 6: Platform-Specific Testing

#### Windows
- [ ] Install extension on Windows 10/11
- [ ] Verify path handling (backslashes, drive letters)
- [ ] Test with Windows Defender enabled
- [ ] Verify PowerShell compatibility
- [ ] Test with spaces in paths
- [ ] Verify file permissions preserved

#### macOS
- [ ] Install extension on macOS (Intel and Apple Silicon)
- [ ] Verify case-insensitive filesystem handling
- [ ] Test with Gatekeeper enabled
- [ ] Verify executable permissions preserved
- [ ] Test with quarantine attributes

#### Linux
- [ ] Install extension on Ubuntu, Fedora, Arch
- [ ] Verify file permissions (executable scripts)
- [ ] Test with SELinux enabled (if applicable)
- [ ] Test with AppArmor enabled (if applicable)
- [ ] Verify symlink handling

---

### Scenario 7: CLI Command Discoverability

**User Persona**: New User

**Steps:**
1. User tries to find extension commands without documentation
2. Run `rad --help`
3. Verify extension commands are listed
4. Run `rad extension --help`
5. Verify comprehensive help text
6. Run `rad extension install --help`
7. Verify command-specific help

**Validation:**
- [ ] `rad extension --help` is comprehensive
- [ ] `rad extension <subcommand> --help` is clear
- [ ] Error messages suggest correct commands

**Pass Criteria**: User can discover commands without documentation

---

### Scenario 8: Error Message Clarity

**User Persona**: Extension Consumer

**Test Cases:**
- [ ] Extension not found error
- [ ] Invalid manifest error
- [ ] Missing dependency error
- [ ] Conflict detection error
- [ ] Path traversal security error

**Validation:**
- [ ] Each error message:
  - Explains what went wrong
  - Suggests how to fix it
  - Provides relevant context
  - Uses plain language

**Pass Criteria**: Non-technical users understand error messages

---

## Smoke Test Checklist

For each release, verify:

- [ ] Install extension from directory
- [ ] Install extension from `.tar.gz` (if supported)
- [ ] List installed extensions
- [ ] Show extension info
- [ ] Use extension agent
- [ ] Use extension template
- [ ] Use extension command
- [ ] Update extension
- [ ] Uninstall extension
- [ ] Verify no leftover files
- [ ] Test on Windows
- [ ] Test on macOS
- [ ] Test on Linux

## Test Extensions

Test extensions are available in `examples/extensions/`:

- `example-extension/` - Basic extension with all component types
- `test-extension-simple/` - Minimal extension for quick testing
- `test-extension-complex/` - Comprehensive extension with many components

## Reporting Issues

When issues are found during manual testing:

1. Document the issue with:
   - Steps to reproduce
   - Expected behavior
   - Actual behavior
   - Platform information
   - Error messages (if any)

2. Report via:
   - GitHub Issues
   - Team communication channel

## See Also

- [User Acceptance Tests](user-acceptance-tests.md)
- [Test Scenarios](test-scenarios.md)
- [Smoke Test Checklist](smoke-test-checklist.md)

