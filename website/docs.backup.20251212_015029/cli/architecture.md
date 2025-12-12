# CLI Architecture

This document describes the architecture of the Radium CLI (`rad`), including its command structure, execution model, and core patterns.

## Overview

The Radium CLI is a comprehensive command-line interface built in Rust using:
- **Clap** for argument parsing and command structure
- **Tokio** for async execution
- **anyhow** for error handling
- **colored** for terminal output formatting

The CLI provides 30+ command modules organized into logical categories for workspace management, plan execution, agent orchestration, and advanced features.

## Command Structure

### Entry Point

The CLI entry point is `apps/cli/src/main.rs`, which:

1. **Parses arguments** using Clap's derive macros
2. **Loads configuration** from multiple sources (CLI args, env vars, config files)
3. **Routes commands** to their respective implementations
4. **Handles errors** and provides user feedback

### Command Routing

Commands are routed through a large `match` statement in `main.rs` (lines 459-556). Each command variant maps to an async execution function in its respective module:

```rust
match command {
    Command::Init { path, use_defaults, ... } => {
        init::execute(path, use_defaults, ...).await?;
    }
    Command::Plan { input, id, name } => {
        plan::execute(input, id, name).await?;
    }
    // ... 30+ more commands
}
```

### Command Module Organization

All command implementations live in `apps/cli/src/commands/`:

- **Core commands**: `init.rs`, `status.rs`, `clean.rs`, `doctor.rs`
- **Plan commands**: `plan.rs`, `craft.rs`, `complete.rs`
- **Agent commands**: `agents.rs`, `step.rs`, `run.rs`, `chat.rs`
- **Management**: `templates.rs`, `engines.rs`, `auth.rs`
- **Advanced**: `autonomous.rs`, `vibecheck.rs`, `mcp.rs`, `learning.rs`
- **Infrastructure**: `checkpoint.rs`, `policy.rs`, `constitution.rs`, `context.rs`
- **Monitoring**: `monitor.rs`, `stats.rs`, `budget.rs`
- **Development**: `sandbox.rs`, `custom.rs`, `extension.rs`, `hooks.rs`, `validate.rs`

Commands are registered in `apps/cli/src/commands/mod.rs` and exported for use in `main.rs`.

## Execution Model

### Async Architecture

All commands are async functions that return `anyhow::Result<()>`. The CLI uses Tokio's async runtime for I/O-bound operations:

- File system operations
- Network requests (AI model APIs)
- Process execution
- Workspace discovery

### Command Signature Pattern

Commands follow a consistent signature pattern:

```rust
pub async fn execute(
    // Required arguments
    arg1: Type1,
    arg2: Type2,
    // Optional flags
    json: bool,
    verbose: bool,
    // ... other options
) -> anyhow::Result<()>
```

### Error Handling

Commands use `anyhow::Context` for error propagation:

```rust
let workspace = Workspace::discover()
    .context("Failed to discover workspace")?;
```

Errors are automatically formatted with context chains, providing helpful debugging information.

## Common Patterns

### Workspace Discovery

Most commands need to discover the current workspace:

```rust
let workspace = Workspace::discover()
    .context("Failed to discover workspace")?;
```

The `Workspace::discover()` function searches upward from the current directory for a `.radium` folder, following the same pattern as Git.

### Model Selection

Commands that interact with AI models use `ModelSelector`:

```rust
use radium_models::ModelSelector;

let selector = ModelSelector::new()
    .with_override(engine_override)
    .with_model_override(model_override);
    
let model = selector.select().await?;
```

This provides flexible model selection with CLI argument overrides, configuration file defaults, and environment variable fallbacks.

### Output Formatting

Commands support both human-readable and JSON output:

```rust
pub async fn execute(json_output: bool) -> anyhow::Result<()> {
    if json_output {
        execute_json().await
    } else {
        execute_human().await
    }
}
```

**Human-readable output** uses the `colored` crate for terminal formatting:
- Success: green checkmarks (✓)
- Errors: red text
- Info: cyan/bold headers
- Warnings: yellow text

**JSON output** uses `serde_json` for structured data:
```rust
let output = json!({
    "status": "success",
    "data": { /* ... */ }
});
println!("{}", serde_json::to_string_pretty(&output)?);
```

### Configuration Loading

The CLI loads configuration from multiple sources with precedence:

1. CLI arguments (highest priority)
2. Environment variables
3. Local config file (`./.radiumrc`)
4. Global config file (`~/.radium/config.toml`)
5. Defaults (lowest priority)

Configuration is loaded in `main.rs` before command execution:

```rust
let cli_config = config::load_config();
unsafe {
    config::apply_config_to_env(&cli_config);
}
```

**Note**: The `unsafe` block is used because environment variables are set before async spawning. This is safe as long as it only happens in single-threaded code before any async operations.

## Command Categories

### Core Workflow Commands

- `init` - Initialize workspace
- `status` - Show workspace status
- `clean` - Clean workspace artifacts
- `doctor` - Environment diagnostics

### Plan Generation and Execution

- `plan` - Generate plans from specifications
- `craft` - Execute plans
- `complete` - End-to-end workflow (fetch, plan, execute)

### Agent Management

- `agents` - List, search, validate, create agents
- `templates` - Manage workflow templates
- `step` - Execute single agent
- `run` - Execute agent script
- `chat` - Interactive chat session

### Advanced Features

- `autonomous` - Autonomous goal execution
- `vibecheck` - Metacognitive oversight
- `mcp` - Model Context Protocol management
- `learning` - Learning system operations
- `checkpoint` - Workspace state snapshots
- `policy` - Policy management
- `constitution` - Constitutional AI rules
- `context` - Context management
- `sandbox` - Isolated command execution
- `custom` - User-defined commands
- `extension` - Extension management
- `hooks` - Lifecycle hooks
- `engines` - AI model configuration
- `budget` - Cost and token usage tracking
- `validate` - Configuration validation

### Monitoring and Analytics

- `monitor` - Real-time monitoring
- `stats` - Statistics and analytics

## Shell Completion

The CLI supports shell completion generation via the `RADIUM_GENERATE_COMPLETIONS` environment variable:

```bash
RADIUM_GENERATE_COMPLETIONS=bash rad > radium.bash
```

Supported shells:
- Bash
- Zsh
- Fish
- PowerShell
- Elvish

Completion generation uses `clap_complete` to generate completions from the Clap command structure.

## Testing

See [Testing Patterns](testing.md) for detailed information about CLI testing.

## Extension Points

The CLI is designed for extensibility:

1. **Custom Commands** - Users can define custom commands via `rad custom`
2. **Extensions** - Plugin system for adding commands, hooks, and MCP servers
3. **Hooks** - Lifecycle hooks for workspace events
4. **MCP Integration** - Model Context Protocol for tool integration

## Performance Considerations

- Commands use async I/O for non-blocking operations
- Workspace discovery is cached where possible
- Model selection is lazy (only when needed)
- JSON output is streamed for large datasets

## Security Considerations

- Credentials stored in `~/.radium/auth/credentials.json` with 0600 permissions
- Environment variable access is limited to single-threaded initialization
- Input validation prevents path traversal
- Command injection prevention for shell execution

See [Security Documentation](security.md) for more details.

