---
id: "plan-execution"
title: "Plan Execution System"
sidebar_label: "Plan Execution System"
---

# Plan Execution System

The plan execution system executes generated plans with intelligent retry logic, error categorization, state persistence, and multiple execution modes. It ensures reliable execution even in the face of transient errors.

## Overview

The plan executor provides robust execution of plans with:

- **Intelligent Retry Logic**: Automatic retries with exponential backoff for recoverable errors
- **Error Categorization**: Distinguishes between recoverable and fatal errors
- **Execution Modes**: Bounded (limited iterations) and Continuous (run until complete)
- **State Persistence**: Saves progress after each task for checkpoint recovery
- **Dependency Validation**: Ensures task dependencies are met before execution
- **Context File Support**: Injects context files into agent prompts

### Key Features

- **Automatic Retries**: Recoverable errors are retried automatically
- **Exponential Backoff**: Delays increase exponentially between retries
- **Checkpoint Recovery**: Resume execution from last checkpoint
- **Progress Tracking**: Real-time progress reporting
- **Multiple Execution Modes**: Bounded or continuous execution

## Execution Lifecycle

The executor follows a structured lifecycle:

1. **Load Manifest**: Load plan manifest from disk (or create new)
2. **Resume Checkpoint**: If resuming, skip completed tasks
3. **Iteration Loop**: Execute iterations based on RunMode
4. **Task Execution**: For each task:
   - Check dependencies
   - Execute with retry logic
   - Save state after completion
5. **Progress Tracking**: Update and display progress

## Retry Logic

The executor uses intelligent retry logic with exponential backoff.

### Retry Behavior

- **Max Retries**: Configurable (default: 3 attempts)
- **Backoff Formula**: `delay = base_delay_ms * 2^attempt`
- **Error Categorization**: Only retries recoverable errors

### Retry Flow

```
Task Execution
  ↓
Error Occurs
  ↓
Categorize Error
  ↓
Recoverable? → Yes → Retry with backoff
  ↓                    ↓
  No              Max retries?
  ↓                    ↓
Fail Immediately      Yes → Fail
                      ↓
                      No → Retry
```

### Example: Retry Sequence

```
Attempt 1: Error (rate limit)
  → Wait 1 second (1000ms * 2^0)
Attempt 2: Error (rate limit)
  → Wait 2 seconds (1000ms * 2^1)
Attempt 3: Error (rate limit)
  → Wait 4 seconds (1000ms * 2^2)
Attempt 4: Success!
```

## Error Categorization

Errors are automatically categorized to determine retry behavior.

### Recoverable Errors

These errors are retried with exponential backoff:

- **HTTP 429**: Rate limit exceeded
- **Network Timeouts**: Connection timeouts, read timeouts
- **Connection Errors**: Network unreachable, connection refused
- **HTTP 5xx**: Server errors (500, 502, 503, 504)
- **File Lock Errors**: Temporary file locking issues
- **Temporary I/O Errors**: Transient I/O failures
- **Model Execution Errors**: May be transient (rate limits, timeouts)

### Fatal Errors

These errors fail immediately without retry:

- **HTTP 401/403**: Authentication/authorization failures
- **Missing Configuration**: Required config not found
- **Invalid Data**: Malformed data, invalid format
- **Dependency Not Met**: Task dependencies not completed
- **Agent Not Found**: Referenced agent doesn't exist

### Error Categorization Logic

```rust
use radium_core::planning::executor::{ExecutionError, ErrorCategory};

let error = ExecutionError::ModelExecution("Rate limit exceeded".to_string());
match error.category() {
    ErrorCategory::Recoverable => {
        // Will retry with exponential backoff
    }
    ErrorCategory::Fatal => {
        // Will fail immediately
    }
}
```

## Execution Modes

The executor supports two execution modes:

### Bounded Mode

Execute up to N iterations, then stop:

```rust
use radium_core::planning::executor::RunMode;

let mode = RunMode::Bounded(5); // Execute up to 5 iterations
```

**Use Cases:**
- Incremental execution
- Testing specific iterations
- Limited resource scenarios

### Continuous Mode

Execute all iterations until plan is complete:

```rust
let mode = RunMode::Continuous; // Execute until complete (YOLO mode)
```

**Use Cases:**
- Full plan execution
- Automated workflows
- Complete feature implementation

**Safety**: Includes a sanity limit to prevent infinite loops.

## State Persistence

The executor saves state after each task completion for checkpoint recovery.

### Checkpoint Structure

State is saved to `plan/plan_manifest.json` with:
- Task completion status
- Iteration status
- Plan progress
- Timestamps

### Resuming from Checkpoint

```rust
use radium_core::planning::executor::{ExecutionConfig, PlanExecutor};
use std::path::PathBuf;

let config = ExecutionConfig {
    resume: true,  // Enable resume mode
    skip_completed: true,  // Skip completed tasks
    // ... other config
};

let executor = PlanExecutor::with_config(config);
// Executor will automatically skip completed tasks
```

## API Usage

### Basic Execution

```rust
use radium_core::planning::executor::{PlanExecutor, ExecutionConfig, RunMode};
use radium_core::models::PlanManifest;
use std::path::PathBuf;

async fn execute_plan() -> Result<(), Box<dyn std::error::Error>> {
    let config = ExecutionConfig {
        resume: false,
        skip_completed: true,
        check_dependencies: true,
        state_path: PathBuf::from("plan/plan_manifest.json"),
        context_files: None,
        run_mode: RunMode::Bounded(5),
    };

    let executor = PlanExecutor::with_config(config);
    let manifest = executor.load_manifest(&config.state_path)?;

    // Execute plan...
    Ok(())
}
```

### Task Execution with Retry

```rust
use radium_core::planning::executor::PlanExecutor;
use radium_core::models::PlanTask;
use radium_abstraction::Model;
use std::sync::Arc;

async fn execute_with_retry(
    executor: &PlanExecutor,
    task: &PlanTask,
    model: Arc<dyn Model>,
) -> Result<TaskResult, ExecutionError> {
    // Execute with 3 retries, 1 second base delay
    executor.execute_task_with_retry(task, model, 3, 1000).await
}
```

### Dependency Validation

```rust
use radium_core::planning::executor::PlanExecutor;

fn validate_dependencies(
    executor: &PlanExecutor,
    manifest: &PlanManifest,
    task: &PlanTask,
) -> Result<(), ExecutionError> {
    executor.check_dependencies(manifest, task)
}
```

### Progress Tracking

```rust
use radium_core::planning::executor::PlanExecutor;
use std::time::Duration;

fn track_progress(executor: &PlanExecutor, manifest: &PlanManifest) {
    let progress = executor.calculate_progress(manifest);
    println!("Progress: {}%", progress);

    executor.print_progress(
        manifest,
        1,  // Current iteration
        Duration::from_secs(120),  // Elapsed time
        Some("Task 1.1"),  // Current task
    );
}
```

## Configuration Options

### ExecutionConfig

```rust
pub struct ExecutionConfig {
    /// Resume from last checkpoint
    pub resume: bool,

    /// Skip already completed tasks
    pub skip_completed: bool,

    /// Validate task dependencies before execution
    pub check_dependencies: bool,

    /// Path to save state checkpoints
    pub state_path: PathBuf,

    /// Optional context files content to inject into prompts
    pub context_files: Option<String>,

    /// Execution mode (bounded or continuous)
    pub run_mode: RunMode,
}
```

### Default Configuration

```rust
ExecutionConfig {
    resume: false,
    skip_completed: true,
    check_dependencies: true,
    state_path: PathBuf::from("plan/plan_manifest.json"),
    context_files: None,
    run_mode: RunMode::Bounded(5),
}
```

## Common Use Cases

### 1. Basic Plan Execution

Execute a plan with default settings:

```rust
let executor = PlanExecutor::new();
let manifest = executor.load_manifest(&PathBuf::from("plan/plan_manifest.json"))?;
// Execute plan...
```

### 2. Resume from Checkpoint

Resume execution from last checkpoint:

```rust
let config = ExecutionConfig {
    resume: true,
    skip_completed: true,
    // ... other config
};
let executor = PlanExecutor::with_config(config);
// Completed tasks are automatically skipped
```

### 3. Bounded Execution

Execute limited iterations:

```rust
let config = ExecutionConfig {
    run_mode: RunMode::Bounded(3),  // Only 3 iterations
    // ... other config
};
```

### 4. Continuous Execution (YOLO Mode)

Execute until complete:

```rust
let config = ExecutionConfig {
    run_mode: RunMode::Continuous,  // Run until done
    // ... other config
};
```

### 5. Context File Injection

Inject context files into prompts:

```rust
let config = ExecutionConfig {
    context_files: Some(std::fs::read_to_string("context.md")?),
    // ... other config
};
```

## Error Handling

### Handling Execution Errors

```rust
use radium_core::planning::executor::{ExecutionError, ErrorCategory};

match executor.execute_task(&task, model).await {
    Ok(result) => {
        if result.success {
            // Task completed successfully
        } else {
            // Task failed, check if retryable
            let error = result.error.unwrap();
            // Handle error...
        }
    }
    Err(e) => {
        match e.category() {
            ErrorCategory::Recoverable => {
                // Will be retried automatically
            }
            ErrorCategory::Fatal => {
                // Must be fixed manually
            }
        }
    }
}
```

### Common Error Scenarios

**Rate Limit (Recoverable)**:
```
Error: Rate limit exceeded (429)
→ Retried with exponential backoff
→ Eventually succeeds
```

**Authentication Failure (Fatal)**:
```
Error: Unauthorized (401)
→ Fails immediately
→ Must fix API key
```

**Dependency Not Met (Fatal)**:
```
Error: Dependency task not completed: I1.T1
→ Fails immediately
→ Must complete dependency first
```

## Best Practices

1. **Use Checkpoints**: Enable resume mode for long-running plans
2. **Monitor Progress**: Track progress for visibility
3. **Handle Errors**: Check error categories for appropriate handling
4. **Validate Dependencies**: Ensure dependencies are met before execution
5. **Use Bounded Mode**: For testing or incremental execution
6. **Use Continuous Mode**: For full automated execution

## Integration Points

- **Plan Generator**: Generates plans for execution
- **DAG System**: Provides dependency ordering
- **Agent System**: Discovers and executes agents
- **Model System**: Executes tasks with AI models
- **Workflow System**: Can be used within workflows

## Related Features

- [Autonomous Planning](./autonomous-planning.md) - Plan generation
- [DAG Dependencies](./dag-dependencies.md) - Dependency management
- [CLI Commands](../cli/commands/plan-execution.md) - Command-line usage
- [Checkpointing](./checkpointing.md) - Checkpoint system

## See Also

- [API Reference](../../crates/radium-core/src/planning/executor.rs) - Complete API documentation
- [Examples](../examples/) - Usage examples

