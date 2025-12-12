---
id: "architecture"
title: "Hooks System Architecture"
sidebar_label: "Hooks System Architecture"
---

# Hooks System Architecture

This document describes the architecture and design patterns of the Radium hooks system.

## Overview

The hooks system provides a framework for intercepting and customizing execution flow at various points. It uses an adapter pattern to integrate different hook types with a unified `Hook` trait, enabling type-safe and extensible behavior customization.

## Core Components

### Hook Registry

The `HookRegistry` is the central component that manages all hooks. It provides:

- **Thread-Safe Storage**: Uses `Arc<RwLock<Vec<Arc<dyn Hook>>>>` for concurrent access
- **Priority-Based Execution**: Hooks execute in priority order (higher priority first)
- **Enable/Disable State**: Tracks which hooks are enabled via `HashSet<String>`
- **Type Filtering**: Retrieves hooks by type for efficient execution

**Key Design Decisions**:
- Hooks are stored in a single vector and filtered by type during execution
- Enabled state is tracked separately to allow dynamic enable/disable
- Early termination: if a hook returns `should_continue = false`, remaining hooks are skipped

### Hook Trait

The core `Hook` trait provides a unified interface:

```rust
#[async_trait]
pub trait Hook: Send + Sync {
    fn name(&self) -> &str;
    fn priority(&self) -> HookPriority;
    fn hook_type(&self) -> HookType;
    async fn execute(&self, context: &HookContext) -> Result<HookExecutionResult>;
}
```

All hooks must implement this trait, enabling polymorphic execution through the registry.

### Adapter Pattern

The system uses adapters to convert specialized hook traits (`ModelHook`, `ToolHook`, `ErrorHook`) into the unified `Hook` trait:

- **ModelHookAdapter**: Converts `ModelHook` to `Hook` for before/after model calls
- **ToolHookAdapter**: Converts `ToolHook` to `Hook` for tool execution hooks
- **ErrorHookAdapter**: Converts `ErrorHook` to `Hook` for error handling hooks

This pattern provides:
- **Type Safety**: Each hook type has its own context structure
- **Flexibility**: Hooks can implement multiple hook types
- **Unified Execution**: All hooks execute through the same registry interface

## Hook Types

The system supports 13 hook types organized into four categories:

### Model Hooks (2 types)
- `BeforeModel`: Execute before model calls
- `AfterModel`: Execute after model calls

### Tool Hooks (3 types)
- `BeforeTool`: Execute before tool execution
- `AfterTool`: Execute after tool execution
- `ToolSelection`: Execute during tool selection

### Error Hooks (4 types)
- `ErrorInterception`: Intercept errors before propagation
- `ErrorTransformation`: Transform error messages
- `ErrorRecovery`: Attempt error recovery
- `ErrorLogging`: Log errors with custom formatting

### Telemetry Hooks (4 types)
- `TelemetryCollection`: Collect and aggregate telemetry
- `CustomLogging`: Custom logging hooks
- `MetricsAggregation`: Aggregate metrics
- `PerformanceMonitoring`: Monitor performance

## Execution Flow

### Hook Execution Process

1. **Registration**: Hooks are registered with the registry via `register()`
2. **Enablement**: Hooks are enabled by default, can be disabled via `set_enabled()`
3. **Execution**: When a hook point is triggered:
   - Registry retrieves all hooks of the specified type
   - Hooks are filtered to only enabled ones
   - Hooks are sorted by priority (descending)
   - Each hook is executed sequentially
   - If a hook returns `should_continue = false`, execution stops
   - Results are collected and returned

### Priority System

Hooks execute in priority order:
- **Higher priority (200+)**: Critical hooks that must run first
- **Medium priority (100-199)**: Standard hooks
- **Low priority (<100)**: Optional hooks

Priority is set when creating a hook and cannot be changed after registration.

### Error Handling

The registry handles hook execution errors gracefully:
- If a hook execution fails, the error is logged
- An error result is added to the results
- Execution continues with remaining hooks
- This ensures one failing hook doesn't break the entire system

## Context System

### HookContext

The base context structure passed to all hooks:

```rust
pub struct HookContext {
    pub hook_type: String,
    pub data: serde_json::Value,
    pub metadata: serde_json::Value,
}
```

### Specialized Contexts

Each hook type has a specialized context:
- **ModelHookContext**: Contains input, model_id, response
- **ToolHookContext**: Contains tool_name, arguments, result
- **ErrorHookContext**: Contains error_message, error_type, error_source
- **TelemetryHookContext**: Contains event_type, data

These contexts convert to `HookContext` via `to_hook_context()` methods.

## Result System

### HookExecutionResult

Hooks return results that can:
- **Continue execution**: `HookResult::success()`
- **Modify data**: `HookResult::with_data(modified_data)`
- **Stop execution**: `HookResult::stop("reason")`
- **Report error**: `HookResult::error("message")`

### Data Modification

Hooks can modify execution data by returning modified data in the result:
- Model hooks can modify input/response
- Tool hooks can modify arguments/results
- Error hooks can transform error messages

The registry collects modifications from all hooks and applies them.

## Configuration System

### HookConfig

Hooks can be configured via TOML files (`.radium/hooks.toml`):

```toml
[[hooks]]
name = "my-hook"
type = "before_model"
priority = 100
enabled = true

[hooks.config]
log_level = "info"
```

### HookLoader

The `HookLoader` discovers and loads hook configurations:
- Discovers hooks from extensions
- Loads workspace configurations
- Sets enable/disable state based on configuration

**Note**: For v1.0, hooks must be registered programmatically. Configuration controls enable/disable state. Dynamic library loading is deferred to v2.0.

## Integration Points

### OrchestratorHooks

Helper for executing hooks in the orchestrator (requires `orchestrator-integration` feature):

```rust
pub struct OrchestratorHooks {
    registry: Arc<HookRegistry>,
}
```

Provides convenience methods for common hook execution patterns.

### HookRegistryAdapter

Adapter that implements `HookExecutor` trait for use with `AgentExecutor`:

```rust
pub struct HookRegistryAdapter {
    registry: Arc<HookRegistry>,
}
```

This adapter bridges the hooks system with the orchestrator without creating circular dependencies.

## Thread Safety

The entire hooks system is designed for concurrent access:

- **Arc<RwLock<>>**: Registry uses `Arc<RwLock>` for thread-safe access
- **Send + Sync**: All hooks must be `Send + Sync`
- **Async Execution**: All hook execution is async, enabling non-blocking operations

## Performance Considerations

- **Lazy Filtering**: Hooks are filtered by type only when needed
- **Priority Sorting**: Hooks are sorted once per execution
- **Early Termination**: Execution stops if a hook requests it
- **Error Isolation**: Hook failures don't affect other hooks

## Extension Points

The system is designed for extensibility:

1. **New Hook Types**: Add new `HookType` variants
2. **New Adapters**: Create adapters for new hook categories
3. **Custom Loaders**: Implement custom hook loading strategies
4. **Integration Helpers**: Create helpers for specific integration points

## Future Enhancements

Planned enhancements (out of scope for v1.0):

- **Hook Marketplace**: Distribution platform for hooks
- **Advanced Composition**: Hook composition patterns
- **Performance Optimization**: Caching and optimization strategies
- **Dynamic Loading**: Runtime loading of hook libraries

