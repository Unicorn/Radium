---
id: "testing"
title: "CLI Testing Patterns"
sidebar_label: "CLI Testing Patterns"
---

# CLI Testing Patterns

This document describes testing patterns and conventions for the Radium CLI.

## Test Structure

All CLI tests are located in `apps/cli/tests/` and follow a consistent structure:

```rust
//! Tests for the `rad <command>` command.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// Helper to initialize a workspace for testing
fn init_workspace(temp_dir: &TempDir) {
    let temp_path = temp_dir.path().to_str().unwrap();
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.arg("init")
       .arg("--use-defaults")
       .arg(temp_path)
       .assert()
       .success();
}

#[test]
fn test_command_basic() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("command-name")
        .assert()
        .success();
}
```

## Test Utilities

### Workspace Initialization

Most tests need a workspace. Use the helper pattern:

```rust
fn init_workspace(temp_dir: &TempDir) {
    let temp_path = temp_dir.path().to_str().unwrap();
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.arg("init")
       .arg("--use-defaults")
       .arg(temp_path)
       .assert()
       .success();
}
```

### Command Execution

Use `assert_cmd::Command` to execute CLI commands:

```rust
use assert_cmd::Command;

let mut cmd = Command::cargo_bin("radium-cli").unwrap();
cmd.current_dir(temp_dir.path())
    .arg("command")
    .arg("arg")
    .assert()
    .success();
```

## Test Categories

### Basic Functionality Tests

Test that commands execute successfully with valid inputs:

```rust
#[test]
fn test_command_success() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Workspace:"));
}
```

### Error Handling Tests

Test that commands fail gracefully with invalid inputs:

```rust
#[test]
fn test_command_no_workspace() {
    let temp_dir = TempDir::new().unwrap();
    // Don't initialize workspace
    
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("status")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to discover workspace"));
}
```

### JSON Output Tests

Test that `--json` flag produces valid JSON:

```rust
#[test]
fn test_command_json_output() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let output = cmd.current_dir(temp_dir.path())
        .arg("status")
        .arg("--json")
        .assert()
        .success()
        .get_output();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Verify it's valid JSON
    let _json: serde_json::Value = serde_json::from_str(&stdout)
        .expect("Output should be valid JSON");
}
```

### Input Validation Tests

Test that commands validate inputs correctly:

```rust
#[test]
fn test_command_invalid_input() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("command")
        .arg("invalid-input")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid"));
}
```

### Edge Case Tests

Test edge cases like empty results, missing files, etc.:

```rust
#[test]
fn test_command_empty_results() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("agents")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No agents found"));
}
```

## Test Patterns by Command Type

### Workspace Commands

Test workspace discovery and validation:

```rust
#[test]
fn test_init_creates_workspace() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap();
    
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.arg("init")
       .arg("--use-defaults")
       .arg(temp_path)
       .assert()
       .success();
    
    // Verify workspace structure
    assert!(temp_dir.path().join(".radium").exists());
}
```

### Plan Commands

Test plan generation and execution:

```rust
#[test]
fn test_plan_generation() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    
    // Create spec file
    let spec_path = temp_dir.path().join("spec.md");
    fs::write(&spec_path, "# Test Spec\n\nDescription").unwrap();
    
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("plan")
        .arg("spec.md")
        .assert()
        .success();
}
```

### Agent Commands

Test agent discovery and execution:

```rust
#[test]
fn test_agents_list() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("agents")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("agents"));
}
```

## Running Tests

### Run All Tests

```bash
cargo test -p radium-cli
```

### Run Specific Test File

```bash
cargo test -p radium-cli --test cli_status_test
```

### Run Specific Test

```bash
cargo test -p radium-cli --test cli_status_test test_status_basic
```

### Run with Output

```bash
cargo test -p radium-cli -- --nocapture
```

## Test Coverage Goals

- **>90% code coverage** for all command modules
- **Both success and error paths** tested
- **JSON output format** validated
- **Workspace discovery** tested
- **Model selection** tested (where applicable)
- **Input validation** tested

## Common Test Helpers

### Create Test Files

```rust
fn create_test_file(temp_dir: &TempDir, path: &str, content: &str) {
    let file_path = temp_dir.path().join(path);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&file_path, content).unwrap();
}
```

### Assert File Exists

```rust
use std::path::Path;

fn assert_file_exists(temp_dir: &TempDir, path: &str) {
    assert!(temp_dir.path().join(path).exists(), 
            "File should exist: {}", path);
}
```

### Assert Output Contains

```rust
cmd.assert()
    .success()
    .stdout(predicate::str::contains("expected text"));
```

## Mocking and Isolation

Tests should be isolated and not depend on:
- External network access (for AI models, use mocks)
- Specific file system state
- Environment variables (set them in tests if needed)
- System configuration

## Performance Testing

For performance-sensitive commands, add benchmarks:

```rust
#[test]
#[ignore] // Don't run in normal test suite
fn test_command_performance() {
    // Measure execution time
    let start = std::time::Instant::now();
    // ... execute command
    let duration = start.elapsed();
    assert!(duration.as_millis() < 1000, "Command too slow");
}
```

## Integration Tests

For end-to-end workflows, use integration tests:

```rust
#[test]
fn test_end_to_end_workflow() {
    let temp_dir = TempDir::new().unwrap();
    
    // Initialize
    init_workspace(&temp_dir);
    
    // Generate plan
    // ... create spec and run plan
    
    // Execute plan
    // ... run craft
    
    // Verify results
    // ... check outputs
}
```

## Best Practices

1. **Use temporary directories** for all tests
2. **Clean up after tests** (TempDir handles this automatically)
3. **Test both success and failure cases**
4. **Test JSON output separately** from human-readable output
5. **Use descriptive test names** that explain what is being tested
6. **Group related tests** in the same file
7. **Use helper functions** to reduce duplication
8. **Assert on both stdout and stderr** when relevant

## Common Issues

### Test Fails Due to Workspace

**Problem**: Test fails because workspace doesn't exist

**Solution**: Always initialize workspace in tests that need it:

```rust
let temp_dir = TempDir::new().unwrap();
init_workspace(&temp_dir);
```

### Test Fails Due to Async

**Problem**: Test doesn't wait for async operations

**Solution**: Use `#[tokio::test]` for async tests (if needed), but most CLI tests use `assert_cmd` which handles async automatically.

### Test Output Differs

**Problem**: Test output doesn't match expected format

**Solution**: Use `predicate::str::contains()` for partial matches instead of exact matches, or normalize output (trim whitespace, etc.).

