# Extension System Test Scenarios

Detailed test scenarios for extension system functionality.

## Installation Scenarios

### Scenario: Install from Local Directory
```bash
# Setup: Create extension directory
mkdir -p my-extension/{agents,templates,commands,prompts,mcp}
# ... create manifest and components ...

# Test: Install extension
rad extension install ./my-extension

# Expected: Success message, extension installed
```

### Scenario: Install with Overwrite
```bash
# Setup: Extension already installed
rad extension install ./my-extension

# Test: Install again with overwrite
rad extension install ./my-extension --overwrite

# Expected: Old version replaced, success message
```

### Scenario: Install with Dependencies
```bash
# Setup: Base extension installed
rad extension install ./base-extension

# Test: Install dependent extension
rad extension install ./dependent-extension --install-deps

# Expected: Dependencies validated, installation succeeds
```

## Discovery Scenarios

### Scenario: List All Extensions
```bash
# Setup: Multiple extensions installed
rad extension install ./ext1
rad extension install ./ext2
rad extension install ./ext3

# Test: List extensions
rad extension list

# Expected: All three extensions listed with metadata
```

### Scenario: Search Extensions
```bash
# Setup: Extensions with different names/descriptions
rad extension install ./web-tools
rad extension install ./data-analysis

# Test: Search
rad extension search "web"

# Expected: Only web-tools extension returned
```

### Scenario: Show Extension Info
```bash
# Setup: Extension installed
rad extension install ./my-extension

# Test: Show info
rad extension info my-extension

# Expected: Full extension details displayed
```

## Component Usage Scenarios

### Scenario: Use Extension Agent
```bash
# Setup: Extension with agent installed
rad extension install ./agent-extension

# Test: List agents
rad agents list

# Expected: Extension agent appears in list

# Test: Use agent
rad agents use extension-agent

# Expected: Agent works correctly
```

### Scenario: Use Extension Template
```bash
# Setup: Extension with template installed
rad extension install ./template-extension

# Test: List templates
rad templates list

# Expected: Extension template appears in list

# Test: Use template
rad workflow create --template extension-template

# Expected: Template works correctly
```

### Scenario: Use Extension Command
```bash
# Setup: Extension with command installed
rad extension install ./command-extension

# Test: Use command
rad extension-name:command-name

# Expected: Command executes correctly
```

## Management Scenarios

### Scenario: Update Extension
```bash
# Setup: Extension v1.0.0 installed
rad extension install ./extension-v1

# Test: Update to v2.0.0
rad extension install ./extension-v2 --overwrite

# Expected: Extension updated, components refreshed
```

### Scenario: Uninstall Extension
```bash
# Setup: Extension installed
rad extension install ./my-extension

# Test: Uninstall
rad extension uninstall my-extension

# Expected: Extension removed, components no longer available
```

### Scenario: Uninstall with Dependencies
```bash
# Setup: Extension A depends on B
rad extension install ./extension-b
rad extension install ./extension-a

# Test: Attempt to uninstall B
rad extension uninstall extension-b

# Expected: Error message about dependency, B not uninstalled
```

## Error Scenarios

### Scenario: Install Invalid Extension
```bash
# Setup: Extension with invalid manifest
# ... create extension with missing required fields ...

# Test: Install
rad extension install ./invalid-extension

# Expected: Clear error message about validation failure
```

### Scenario: Install Conflicting Extension
```bash
# Setup: Extension with agent ID that already exists
rad extension install ./extension-with-agent-x

# Test: Install conflicting extension
rad extension install ./conflicting-extension

# Expected: Conflict error, installation fails
```

### Scenario: Install with Missing Dependencies
```bash
# Setup: Extension depends on non-existent extension
# ... create extension with dependency on "missing-ext" ...

# Test: Install without --install-deps
rad extension install ./dependent-extension

# Expected: Error message about missing dependency
```

## Platform-Specific Scenarios

### Windows
- Install extension with spaces in path
- Install extension with long path names
- Test with Windows Defender enabled

### macOS
- Install extension on case-insensitive filesystem
- Test with Gatekeeper enabled
- Verify executable permissions preserved

### Linux
- Install extension with special characters in name
- Test with SELinux/AppArmor enabled
- Verify symlink handling

## See Also

- [Manual Test Plan](manual-test-plan.md)
- [User Acceptance Tests](user-acceptance-tests.md)
- [Smoke Test Checklist](smoke-test-checklist.md)

