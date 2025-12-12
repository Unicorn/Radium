---
id: "hook-types"
title: "Hook Types Reference"
sidebar_label: "Hook Types Reference"
---

# Hook Types Reference

Complete reference for all 13 hook types in the Radium hooks system.

## Model Hooks

### BeforeModel

**Type**: `HookType::BeforeModel`  
**String**: `"before_model"`  
**Context**: `ModelHookContext` (before state)  
**Use Case**: Intercept and modify model call inputs before execution

**Context Fields**:
- `input: String` - The input prompt/text
- `model_id: String` - The model identifier
- `request_modifications: Option<serde_json::Value>` - Optional request modifications
- `modified_input: Option<String>` - Modified input from previous hooks

**Common Use Cases**:
- Input validation and sanitization
- Adding system prompts or context
- Rate limiting checks
- Request logging
- Input transformation

**Example**:
```rust
async fn before_model_call(&self, context: &ModelHookContext) -> Result<HookExecutionResult> {
    // Validate input
    if context.input.is_empty() {
        return Ok(HookExecutionResult::stop("Input cannot be empty"));
    }
    
    // Modify input
    let modified = format!("System: {}\nUser: {}", system_prompt, context.input);
    Ok(HookExecutionResult::with_data(json!({
        "modified_input": modified
    })))
}
```

### AfterModel

**Type**: `HookType::AfterModel`  
**String**: `"after_model"`  
**Context**: `ModelHookContext` (after state)  
**Use Case**: Process and modify model responses after execution

**Context Fields**:
- `input: String` - The original input
- `model_id: String` - The model identifier
- `response: Option<String>` - The model response
- `modified_input: Option<String>` - Any input modifications

**Common Use Cases**:
- Response validation
- Response transformation
- Logging responses
- Cost tracking
- Response caching

**Example**:
```rust
async fn after_model_call(&self, context: &ModelHookContext) -> Result<HookExecutionResult> {
    let response = context.response.as_ref().unwrap();
    
    // Log response
    tracing::info!("Model response: {}", response);
    
    // Transform response
    let transformed = response.trim().to_uppercase();
    Ok(HookExecutionResult::with_data(json!({
        "response": transformed
    })))
}
```

## Tool Hooks

### BeforeTool

**Type**: `HookType::BeforeTool`  
**String**: `"before_tool"`  
**Context**: `ToolHookContext` (before state)  
**Use Case**: Validate and modify tool arguments before execution

**Context Fields**:
- `tool_name: String` - The tool identifier
- `arguments: serde_json::Value` - Tool arguments
- `modified_arguments: Option<serde_json::Value>` - Modified arguments from previous hooks

**Common Use Cases**:
- Argument validation
- Security checks
- Argument transformation
- Permission checks
- Tool-specific logging

**Example**:
```rust
async fn before_tool_execution(&self, context: &ToolHookContext) -> Result<HookExecutionResult> {
    // Validate arguments
    if context.tool_name == "read_file" {
        let path = context.arguments.get("path").and_then(|v| v.as_str());
        if let Some(p) = path {
            if p.contains("..") {
                return Ok(HookExecutionResult::stop("Invalid path"));
            }
        }
    }
    
    Ok(HookExecutionResult::success())
}
```

### AfterTool

**Type**: `HookType::AfterTool`  
**String**: `"after_tool"`  
**Context**: `ToolHookContext` (after state)  
**Use Case**: Process and modify tool execution results

**Context Fields**:
- `tool_name: String` - The tool identifier
- `arguments: serde_json::Value` - Original arguments
- `result: Option<serde_json::Value>` - Tool execution result
- `modified_result: Option<serde_json::Value>` - Modified result from previous hooks

**Common Use Cases**:
- Result validation
- Result transformation
- Error handling
- Result caching
- Audit logging

**Example**:
```rust
async fn after_tool_execution(&self, context: &ToolHookContext) -> Result<HookExecutionResult> {
    // Log tool execution
    tracing::info!("Tool {} executed with result", context.tool_name);
    
    // Transform result
    if let Some(result) = &context.result {
        let transformed = json!({
            "success": true,
            "data": result
        });
        Ok(HookExecutionResult::with_data(json!({
            "modified_result": transformed
        })))
    } else {
        Ok(HookExecutionResult::success())
    }
}
```

### ToolSelection

**Type**: `HookType::ToolSelection`  
**String**: `"tool_selection"`  
**Context**: `ToolHookContext` (selection state)  
**Use Case**: Control which tools can be executed

**Context Fields**:
- `tool_name: String` - The tool identifier
- `arguments: serde_json::Value` - Tool arguments

**Common Use Cases**:
- Tool allowlisting/denylisting
- Permission checks
- Tool-specific policies
- Usage restrictions

**Example**:
```rust
async fn tool_selection(&self, context: &ToolHookContext) -> Result<HookExecutionResult> {
    // Deny specific tools
    let denied_tools = vec!["delete_file", "rm"];
    if denied_tools.contains(&context.tool_name.as_str()) {
        return Ok(HookExecutionResult::stop("Tool not allowed"));
    }
    
    Ok(HookExecutionResult::success())
}
```

## Error Hooks

### ErrorInterception

**Type**: `HookType::ErrorInterception`  
**String**: `"error_interception"`  
**Context**: `ErrorHookContext` (interception state)  
**Use Case**: Intercept errors before they propagate

**Context Fields**:
- `error_message: String` - The error message
- `error_type: String` - The error type
- `error_source: Option<String>` - Where the error originated
- `recovered: bool` - Whether error was recovered

**Common Use Cases**:
- Error filtering
- Error suppression
- Custom error handling
- Error notification

**Example**:
```rust
async fn error_interception(&self, context: &ErrorHookContext) -> Result<HookExecutionResult> {
    // Suppress specific errors
    if context.error_type == "NetworkError" {
        return Ok(HookExecutionResult::stop("Error handled"));
    }
    
    Ok(HookExecutionResult::success())
}
```

### ErrorTransformation

**Type**: `HookType::ErrorTransformation`  
**String**: `"error_transformation"`  
**Context**: `ErrorHookContext` (transformation state)  
**Use Case**: Transform error messages for better user experience

**Context Fields**: Same as ErrorInterception

**Common Use Cases**:
- User-friendly error messages
- Error message translation
- Error categorization
- Error enrichment

**Example**:
```rust
async fn error_transformation(&self, context: &ErrorHookContext) -> Result<HookExecutionResult> {
    let transformed = match context.error_type.as_str() {
        "NetworkError" => "Connection failed. Please check your internet connection.",
        "TimeoutError" => "Request timed out. Please try again.",
        _ => context.error_message.clone(),
    };
    
    Ok(HookExecutionResult::with_data(json!({
        "transformed_error": transformed
    })))
}
```

### ErrorRecovery

**Type**: `HookType::ErrorRecovery`  
**String**: `"error_recovery"`  
**Context**: `ErrorHookContext` (recovery state)  
**Use Case**: Attempt to recover from errors automatically

**Context Fields**: Same as ErrorInterception

**Common Use Cases**:
- Automatic retry logic
- Fallback strategies
- Error mitigation
- Recovery procedures

**Example**:
```rust
async fn error_recovery(&self, context: &ErrorHookContext) -> Result<HookExecutionResult> {
    // Retry network errors
    if context.error_type == "NetworkError" {
        // Attempt recovery
        let recovered = attempt_recovery().await;
        if recovered {
            return Ok(HookExecutionResult::with_data(json!({
                "recovered_error": "Network connection restored"
            })));
        }
    }
    
    Ok(HookExecutionResult::success())
}
```

### ErrorLogging

**Type**: `HookType::ErrorLogging`  
**String**: `"error_logging"`  
**Context**: `ErrorHookContext` (logging state)  
**Use Case**: Log errors with custom formatting

**Context Fields**: Same as ErrorInterception

**Common Use Cases**:
- Structured error logging
- Error aggregation
- Error reporting
- Error analytics

**Example**:
```rust
async fn error_logging(&self, context: &ErrorHookContext) -> Result<HookExecutionResult> {
    // Log with structured format
    tracing::error!(
        error_type = %context.error_type,
        error_message = %context.error_message,
        source = ?context.error_source,
        "Error occurred"
    );
    
    Ok(HookExecutionResult::success())
}
```

## Telemetry Hooks

### TelemetryCollection

**Type**: `HookType::TelemetryCollection`  
**String**: `"telemetry_collection"`  
**Context**: `TelemetryHookContext`  
**Use Case**: Collect and aggregate telemetry data

**Context Fields**:
- `event_type: String` - The event type
- `data: serde_json::Value` - Telemetry data
- `metadata: Option<serde_json::Value>` - Additional metadata

**Telemetry Data Structure**:
```json
{
    "agent_id": "string",
    "input_tokens": 0,
    "output_tokens": 0,
    "total_tokens": 0,
    "estimated_cost": 0.0,
    "model": "string",
    "provider": "string"
}
```

**Common Use Cases**:
- Cost tracking
- Performance monitoring
- Usage analytics
- Resource monitoring

**Example**:
```rust
async fn execute(&self, context: &HookContext) -> Result<HookExecutionResult> {
    let data = &context.data;
    let tokens = data.get("total_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
    let cost = data.get("estimated_cost").and_then(|v| v.as_f64()).unwrap_or(0.0);
    
    // Aggregate telemetry
    aggregate_telemetry(tokens, cost).await;
    
    Ok(HookExecutionResult::success())
}
```

### CustomLogging

**Type**: `HookType::CustomLogging`  
**String**: `"custom_logging"`  
**Context**: `HookContext`  
**Use Case**: Custom logging for any execution point

**Common Use Cases**:
- Application-specific logging
- Audit trails
- Debug logging
- Custom log formats

### MetricsAggregation

**Type**: `HookType::MetricsAggregation`  
**String**: `"metrics_aggregation"`  
**Context**: `HookContext`  
**Use Case**: Aggregate metrics across executions

**Common Use Cases**:
- Performance metrics
- Success rates
- Error rates
- Usage statistics

### PerformanceMonitoring

**Type**: `HookType::PerformanceMonitoring`  
**String**: `"performance_monitoring"`  
**Context**: `HookContext`  
**Use Case**: Monitor and track performance

**Common Use Cases**:
- Latency tracking
- Throughput monitoring
- Resource usage
- Performance alerts

## Hook Type Selection Guide

Choose the appropriate hook type based on your use case:

- **Input/Output Modification**: Use `BeforeModel`/`AfterModel` or `BeforeTool`/`AfterTool`
- **Validation**: Use `BeforeModel` or `BeforeTool`
- **Error Handling**: Use error hooks (`ErrorInterception`, `ErrorTransformation`, `ErrorRecovery`, `ErrorLogging`)
- **Logging**: Use `CustomLogging` or model/tool hooks
- **Telemetry**: Use `TelemetryCollection` or `MetricsAggregation`
- **Access Control**: Use `ToolSelection`
- **Performance**: Use `PerformanceMonitoring`

## Priority Guidelines

Recommended priorities by hook type:

- **Critical (200+)**: Security checks, access control, critical validation
- **Standard (100-199)**: Logging, transformation, standard validation
- **Low (<100)**: Optional monitoring, non-critical telemetry

