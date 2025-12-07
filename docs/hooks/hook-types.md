# Hook Types Reference

Complete reference for all hook types in the Radium Hooks System.

## Model Hooks

Intercept and modify model API calls.

### BeforeModelCall

Executed before a model API call is made.

**Context**: `ModelHookContext` with:
- `input: String` - The input to the model
- `model_id: String` - The model identifier

**Use Cases**:
- Input validation
- Input transformation
- Rate limiting
- Logging

**Example**:
```rust
async fn before_model_call(&self, context: &ModelHookContext) -> Result<HookResult> {
    // Modify input
    let modified = format!("[PREFIX] {}", context.input);
    Ok(HookResult::with_data(json!({
        "modified_input": modified
    })))
}
```

### AfterModelCall

Executed after a model API call completes.

**Context**: `ModelHookContext` with:
- `input: String` - The original input
- `model_id: String` - The model identifier
- `response: String` - The model response

**Use Cases**:
- Response validation
- Response transformation
- Logging
- Caching

## Tool Hooks

Intercept and modify tool execution.

### BeforeToolExecution

Executed before a tool is executed.

**Context**: `ToolHookContext` with:
- `tool_name: String` - The tool name
- `arguments: Value` - The tool arguments

**Use Cases**:
- Argument validation
- Argument transformation
- Permission checking
- Logging

### AfterToolExecution

Executed after a tool completes.

**Context**: `ToolHookContext` with:
- `tool_name: String` - The tool name
- `arguments: Value` - The tool arguments
- `result: Value` - The tool result

**Use Cases**:
- Result validation
- Result transformation
- Logging
- Error handling

### ToolSelection

Executed when a tool is selected for execution.

**Context**: `ToolHookContext` with:
- `tool_name: String` - The tool name
- `arguments: Value` - The tool arguments

**Use Cases**:
- Tool filtering
- Permission checking
- Tool routing

## Error Hooks

Handle errors and implement recovery strategies.

### ErrorInterception

Executed when an error occurs.

**Context**: `ErrorHookContext` with:
- `error_message: String` - The error message
- `error_type: String` - The error type
- `error_source: Option<String>` - Where the error originated

**Use Cases**:
- Error logging
- Error transformation
- Error recovery
- Notification

**Example**:
```rust
async fn intercept_error(&self, context: &ErrorHookContext) -> Result<HookResult> {
    // Log error
    eprintln!("Error: {} ({})", context.error_message, context.error_type);
    
    // Attempt recovery
    if context.error_type == "network_error" {
        return Ok(HookResult::with_data(json!({
            "recovered": true,
            "retry": true
        })));
    }
    
    Ok(HookResult::success())
}
```

## Telemetry Hooks

Collect metrics and performance data.

### TelemetryCollection

Executed for telemetry events.

**Context**: `TelemetryHookContext` with:
- `event_type: String` - The event type
- `data: Value` - Event data

**Use Cases**:
- Metrics collection
- Performance monitoring
- Usage analytics
- Cost tracking

## Workflow Hooks

Integrate with workflow behaviors.

### WorkflowStep

Executed after a workflow step completes.

**Context**: `HookContext` with:
- `behavior_file: String` - Path to behavior.json
- `output: String` - Step output
- `step_id: String` - Step identifier
- `workflow_id: String` - Workflow identifier

**Use Cases**:
- Behavior evaluation
- Workflow control
- Step logging
- State management

## Hook Result

All hooks return a `HookResult` that controls execution:

- **`success()`**: Continue execution normally
- **`with_data(data)`**: Continue with modified data
- **`stop(message)`**: Stop execution immediately
- **`error(message)`**: Log error but continue execution

## Priority

Hooks execute in priority order (higher priority first). Use priorities to control execution order:

- **100-200**: Critical hooks (validation, security)
- **50-99**: Important hooks (logging, monitoring)
- **1-49**: Optional hooks (telemetry, analytics)

