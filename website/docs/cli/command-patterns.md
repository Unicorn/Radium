---
id: "command-patterns"
title: "CLI Command Implementation Patterns"
sidebar_label: "CLI Command Implementation Patterns"
---

# CLI Command Implementation Patterns

This guide provides patterns and best practices for implementing new CLI commands in Radium.

## Command Structure Template

Here's a template for implementing a new command:

```rust
//! Command name implementation.
//!
//! Brief description of what the command does.

use anyhow::{Context, Result};
use colored::Colorize;
use radium_core::Workspace;
// ... other imports

/// Execute the command.
///
/// Detailed description of command behavior.
pub async fn execute(
    // Required arguments
    arg1: Type1,
    arg2: Option<Type2>,
    // Flags
    json: bool,
    verbose: bool,
) -> anyhow::Result<()> {
    // Early return for JSON output
    if json {
        return execute_json(arg1, arg2).await;
    }

    // Human-readable output
    println!("{}", "Command Name".bold().cyan());
    println!();

    // Workspace discovery (if needed)
    let workspace = Workspace::discover()
        .context("Failed to discover workspace")?;

    // Command logic here
    // ...

    Ok(())
}

/// Execute command with JSON output.
async fn execute_json(
    arg1: Type1,
    arg2: Option<Type2>,
) -> anyhow::Result<()> {
    use serde_json::json;

    // Build JSON response
    let output = json!({
        "status": "success",
        "data": { /* ... */ }
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
```

## Adding a Command to main.rs

1. **Add command variant** to the `Command` enum:

```rust
#[derive(Subcommand, Debug)]
enum Command {
    // ... existing commands
    MyCommand {
        /// Argument description
        arg: String,
        /// Flag description
        #[arg(long)]
        json: bool,
    },
}
```

2. **Add routing** in the match statement:

```rust
match command {
    // ... existing matches
    Command::MyCommand { arg, json } => {
        my_command::execute(arg, json).await?;
    }
}
```

3. **Import the module** at the top:

```rust
use commands::{
    // ... existing imports
    my_command,
};
```

4. **Register module** in `apps/cli/src/commands/mod.rs`:

```rust
pub mod my_command;
```

## Common Patterns

### Workspace Discovery

Always use `Workspace::discover()` with context:

```rust
let workspace = Workspace::discover()
    .context("Failed to discover workspace")?;
```

If the command can work without a workspace, handle the error gracefully:

```rust
let workspace = match Workspace::discover() {
    Ok(w) => Some(w),
    Err(_) => {
        if !allow_no_workspace {
            anyhow::bail!("Workspace required. Run 'rad init' first.");
        }
        None
    }
};
```

### Model Selection

For commands that need AI models:

```rust
use radium_models::ModelSelector;

let selector = ModelSelector::new()
    .with_override(engine_override)
    .with_model_override(model_override);
    
let model = selector.select().await
    .context("Failed to select model")?;
```

### Output Formatting

**Human-readable output** should be:
- Colorful and informative
- Use consistent symbols (✓ for success, ✗ for errors)
- Group related information
- Provide actionable feedback

```rust
println!("{}", "Section Header".bold().cyan());
println!("  {} Item 1", "✓".green());
println!("  {} Item 2: {}", "•".dimmed(), value.cyan());
```

**JSON output** should be:
- Structured and consistent
- Include all relevant data
- Use proper types (strings, numbers, booleans, arrays, objects)

```rust
let output = json!({
    "status": "success",
    "count": items.len(),
    "items": items.iter().map(|i| json!({
        "id": i.id,
        "name": i.name,
    })).collect::<Vec<_>>(),
});
```

### Error Handling

Use `anyhow::Context` for error propagation:

```rust
let file = fs::read_to_string(&path)
    .context(format!("Failed to read file: {}", path.display()))?;
```

Provide actionable error messages:

```rust
if !workspace.exists() {
    anyhow::bail!(
        "Workspace not found. Run 'rad init' to create one."
    );
}
```

### Input Validation

Validate user input early:

```rust
if arg.is_empty() {
    anyhow::bail!("Argument cannot be empty");
}

if !path.exists() {
    anyhow::bail!("Path does not exist: {}", path.display());
}
```

For file paths, prevent path traversal:

```rust
use std::path::Path;

fn validate_path(path: &Path, base: &Path) -> anyhow::Result<()> {
    let canonical = path.canonicalize()
        .context("Failed to canonicalize path")?;
    let base_canonical = base.canonicalize()
        .context("Failed to canonicalize base path")?;
    
    if !canonical.starts_with(&base_canonical) {
        anyhow::bail!("Path traversal detected");
    }
    
    Ok(())
}
```

### Progress Indication

For long-running operations, show progress:

```rust
println!("{}", "Processing...".bold());
for (i, item) in items.iter().enumerate() {
    print!("\r  [{}/{}] {}", i + 1, items.len(), item.name);
    // ... process item
}
println!(); // New line after progress
```

### Interactive Prompts

Use `inquire` for interactive prompts:

```rust
use inquire::{Confirm, Text, Select};

let confirm = Confirm::new("Proceed?")
    .with_default(true)
    .prompt()?;

let input = Text::new("Enter value:")
    .with_default("default")
    .prompt()?;

let choice = Select::new("Select option:", options)
    .prompt()?;
```

## Command Categories

### Simple Commands

Commands that just read and display information:

```rust
pub async fn execute(json: bool) -> anyhow::Result<()> {
    let data = fetch_data().await?;
    if json {
        output_json(&data)?;
    } else {
        output_human(&data);
    }
    Ok(())
}
```

### Commands with Side Effects

Commands that modify state should:
- Validate inputs
- Show what will happen
- Confirm destructive operations
- Provide rollback if possible

```rust
pub async fn execute(path: PathBuf, force: bool) -> anyhow::Result<()> {
    // Validate
    if !path.exists() {
        anyhow::bail!("Path does not exist");
    }
    
    // Warn about destructive operation
    if !force {
        let confirm = Confirm::new("This will delete data. Continue?")
            .with_default(false)
            .prompt()?;
        if !confirm {
            return Ok(());
        }
    }
    
    // Execute
    perform_operation(&path).await?;
    
    println!("{} Operation completed", "✓".green());
    Ok(())
}
```

### Commands with AI Integration

Commands that use AI models should:
- Show model selection
- Display thinking/processing status
- Handle rate limits gracefully
- Show token usage

```rust
pub async fn execute(prompt: String, model: Option<String>) -> anyhow::Result<()> {
    let selector = ModelSelector::new()
        .with_model_override(model);
    let model = selector.select().await?;
    
    println!("{} Using model: {}", "•".dimmed(), model.name().cyan());
    println!("{} Processing...", "🤔".yellow());
    
    let response = model.generate(&prompt).await
        .context("AI generation failed")?;
    
    if let Some(usage) = &response.usage {
        println!("{} Tokens: {}", "•".dimmed(), usage.total_tokens);
    }
    
    println!("{}", response.content);
    Ok(())
}
```

## Testing Your Command

See [Testing Patterns](testing.md) for detailed testing guidelines.

Basic test structure:

```rust
#[test]
fn test_command_basic() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("my-command")
        .arg("arg-value")
        .assert()
        .success();
}
```

## Best Practices

1. **Always support `--json` flag** for scripting and CI/CD
2. **Provide helpful error messages** with context and suggestions
3. **Use consistent formatting** across all commands
4. **Validate inputs early** before performing operations
5. **Show progress** for long-running operations
6. **Handle edge cases** gracefully (missing workspace, empty results, etc.)
7. **Document command behavior** in the doc comment
8. **Follow existing patterns** for consistency

## Common Pitfalls

1. **Forgetting workspace discovery** - Most commands need a workspace
2. **Not handling errors gracefully** - Use `context()` for better error messages
3. **Inconsistent output formatting** - Follow the established patterns
4. **Missing JSON support** - All commands should support `--json`
5. **Not validating inputs** - Always validate before processing
6. **Blocking operations** - Use async I/O for file and network operations

