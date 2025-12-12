---
id: "error-handling"
title: "CLI Error Handling Guidelines"
sidebar_label: "CLI Error Handling Guidelines"
---

# CLI Error Handling Guidelines

This document describes the standardized error handling patterns for the Radium CLI.

## Error Message Format

### Human-Readable Format

Error messages should follow this format:

```
Error: <what failed>: <why it failed>. <how to fix>
```

**Example:**
```
Error: Workspace not found: No .radium directory found in current or parent directories. Run 'rad init' to create one.
```

### JSON Format

JSON error output should use this structure:

```json
{
  "error": true,
  "message": "<what failed>: <why it failed>",
  "context": {
    "command": "<command name>",
    "operation": "<specific operation>",
    "details": "<additional context>"
  },
  "suggestion": "<how to fix>"
}
```

**Example:**
```json
{
  "error": true,
  "message": "Workspace not found: No .radium directory found",
  "context": {
    "command": "status",
    "operation": "workspace discovery",
    "details": "Searched from /path/to/current/dir"
  },
  "suggestion": "Run 'rad init' to create a workspace"
}
```

## Error Categories

### Workspace Errors

**Pattern:**
```
Error: Workspace not found: <reason>. Run 'rad init' to create one.
```

**Implementation:**
```rust
let workspace = Workspace::discover()
    .context("No Radium workspace found. Run 'rad init' to create one.")?;
```

### Authentication Errors

**Pattern:**
```
Error: Not authenticated with <engine>: <reason>. Run 'rad auth login <engine>' to authenticate.
```

**Implementation:**
```rust
if !is_authenticated(&engine) {
    anyhow::bail!(
        "Not authenticated with {}. Run 'rad auth login {}' to authenticate.",
        engine, engine
    );
}
```

### Plan Errors

**Pattern:**
```
Error: Plan `<id>` not found: <reason>. Use 'rad plan' to create one or 'rad craft' to list available plans.
```

**Implementation:**
```rust
let plan = workspace.find_plan(&plan_id)
    .with_context(|| format!(
        "Plan {} not found. Use 'rad plan' to create one or 'rad craft' to list available plans.",
        plan_id
    ))?;
```

### Agent Errors

**Pattern:**
```
Error: Agent `<id>` not found: <reason>. Use 'rad agents list' to see available agents.
```

**Implementation:**
```rust
let agent = discovery.find_agent(&agent_id)
    .with_context(|| format!(
        "Agent {} not found. Use 'rad agents list' to see available agents.",
        agent_id
    ))?;
```

### File/Path Errors

**Pattern:**
```
Error: File not found: `<path>`. <suggestion>
```

**Implementation:**
```rust
let file = fs::read_to_string(&path)
    .with_context(|| format!(
        "File not found: {}. Check the path and try again.",
        path.display()
    ))?;
```

### Validation Errors

**Pattern:**
```
Error: Validation failed: <what>. <how to fix>
```

**Implementation:**
```rust
if !is_valid(&input) {
    anyhow::bail!(
        "Validation failed: {}. {}",
        issue,
        suggestion
    );
}
```

## Color Coding

Use the `colored` crate for consistent color coding:

- **Errors**: Red (`"Error message".red()`)
- **Warnings**: Yellow (`"Warning message".yellow()`)
- **Success**: Green (`"Success message".green()`)
- **Info**: Cyan (`"Info message".cyan()`)
- **Dimmed**: Dimmed (`"Secondary info".dimmed()`)

**Example:**
```rust
use colored::Colorize;

println!("{}", format!("Error: {}", message).red());
println!("{}", format!("Warning: {}", warning).yellow());
println!("{}", "Success!".green());
```

## Error Propagation

Always use `anyhow::Context` for error propagation:

```rust
use anyhow::Context;

let result = operation()
    .context("Operation failed")?;
```

For more specific context:

```rust
let result = operation()
    .with_context(|| format!("Failed to {}: {}", operation_name, details))?;
```

## JSON Error Output

When `--json` flag is used, output structured errors:

```rust
if json_output {
    let error_json = json!({
        "error": true,
        "message": format!("{}: {}", what, why),
        "context": {
            "command": "command-name",
            "operation": operation_name,
            "details": additional_details,
        },
        "suggestion": how_to_fix,
    });
    println!("{}", serde_json::to_string_pretty(&error_json)?);
    std::process::exit(1);
} else {
    println!("{}", format!("Error: {}: {}. {}", what, why, how_to_fix).red());
    return Err(anyhow::anyhow!("{}: {}", what, why));
}
```

## Common Error Scenarios

### Workspace Not Found

```rust
let workspace = Workspace::discover()
    .context("No Radium workspace found. Run 'rad init' to create one.")?;
```

### Invalid Input

```rust
if input.is_empty() {
    anyhow::bail!("Input cannot be empty. Provide a valid value.");
}
```

### File Not Found

```rust
let content = fs::read_to_string(&path)
    .with_context(|| format!(
        "File not found: {}. Check the path and try again.",
        path.display()
    ))?;
```

### Permission Denied

```rust
fs::write(&path, content)
    .with_context(|| format!(
        "Permission denied: Cannot write to {}. Check file permissions.",
        path.display()
    ))?;
```

### Network/API Errors

```rust
let response = client.request()
    .await
    .context("Failed to connect to API. Check your network connection and try again.")?;
```

### Model Selection Errors

```rust
let model = selector.select()
    .await
    .context("Failed to select model. Check your engine configuration with 'rad engines list'.")?;
```

## Best Practices

1. **Always provide context** - Use `context()` or `with_context()` to add helpful information
2. **Include actionable suggestions** - Tell users how to fix the problem
3. **Use consistent formatting** - Follow the standard format across all commands
4. **Color code appropriately** - Red for errors, yellow for warnings
5. **Support JSON output** - Always provide structured JSON errors when `--json` is used
6. **Don't expose internals** - Error messages should be user-friendly, not technical
7. **Chain context** - Build up context as errors propagate through the call stack

## Examples

### Good Error Message

```rust
let workspace = Workspace::discover()
    .context("No Radium workspace found. Run 'rad init' to create one.")?;
```

**Output:**
```
Error: No Radium workspace found. Run 'rad init' to create one.
```

### Better Error Message

```rust
let workspace = Workspace::discover()
    .with_context(|| format!(
        "No Radium workspace found in {} or parent directories. Run 'rad init' to create one.",
        current_dir.display()
    ))?;
```

**Output:**
```
Error: No Radium workspace found in /path/to/dir or parent directories. Run 'rad init' to create one.
```

### JSON Error Example

```rust
if json_output {
    let error_json = json!({
        "error": true,
        "message": "Workspace not found",
        "context": {
            "command": "status",
            "operation": "workspace discovery",
            "searched_path": current_dir.display().to_string(),
        },
        "suggestion": "Run 'rad init' to create a workspace"
    });
    println!("{}", serde_json::to_string_pretty(&error_json)?);
    std::process::exit(1);
}
```

## Migration Guide

When updating existing commands:

1. **Identify error messages** - Find all `anyhow::bail!()` and `?` operators
2. **Add context** - Wrap with `context()` or `with_context()`
3. **Format consistently** - Use the standard format
4. **Add JSON support** - Check for `--json` flag and output structured errors
5. **Test error paths** - Verify error messages are helpful and consistent

## Testing Error Messages

Test error messages in your test suite:

```rust
#[test]
fn test_error_message_format() {
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.arg("command")
        .arg("invalid-input")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error:"))
        .stderr(predicate::str::contains("Run 'rad"));
}

#[test]
fn test_json_error_output() {
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let output = cmd
        .arg("command")
        .arg("--json")
        .arg("invalid-input")
        .assert()
        .failure()
        .get_output();
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    let json: serde_json::Value = serde_json::from_str(&stderr).unwrap();
    assert_eq!(json["error"], true);
    assert!(json["suggestion"].is_string());
}
```

