# Hooks Examples and Patterns

This document provides practical examples and common patterns for using the Radium hooks system.

## Example Implementations

### Logging Hook

The logging hook demonstrates how to log model calls with timestamps and metadata.

**Location**: `examples/hooks/logging-hook/`

**Key Features**:
- Logs before and after model calls
- Supports JSON and text log formats
- Configurable log levels
- Tracks input/output lengths

**Usage**:
```rust
use logging_hook::{create_before_hook, create_after_hook};
use radium_core::hooks::registry::HookRegistry;
use std::sync::Arc;

let registry = Arc::new(HookRegistry::new());
registry.register(create_before_hook()).await?;
registry.register(create_after_hook()).await?;
```

**Configuration**:
```toml
[[hooks]]
name = "logging-hook-before"
type = "before_model"
priority = 100
enabled = true

[hooks.config]
log_level = "info"
log_format = "json"
```

### Metrics Hook

The metrics hook aggregates telemetry data including token usage and costs.

**Location**: `examples/hooks/metrics-hook/`

**Key Features**:
- Aggregates token usage across all calls
- Tracks costs per model and provider
- Provides summary statistics
- Periodic logging of metrics

**Usage**:
```rust
use metrics_hook::create_metrics_hook;
use radium_core::hooks::registry::HookRegistry;
use std::sync::Arc;

let registry = Arc::new(HookRegistry::new());
let hook = create_metrics_hook();
registry.register(hook.clone()).await?;

// Later, get metrics summary
let summary = hook.get_summary().await;
println!("Total tokens: {}", summary["total_tokens"]);
```

## Common Patterns

### Pattern 1: Input Validation

Validate and sanitize inputs before model calls:

```rust
async fn before_model_call(&self, context: &ModelHookContext) -> Result<HookExecutionResult> {
    // Check for empty input
    if context.input.trim().is_empty() {
        return Ok(HookExecutionResult::stop("Input cannot be empty"));
    }

    // Check input length
    if context.input.len() > MAX_INPUT_LENGTH {
        return Ok(HookExecutionResult::stop("Input too long"));
    }

    // Sanitize input
    let sanitized = context.input.trim().to_string();
    Ok(HookExecutionResult::with_data(json!({
        "modified_input": sanitized
    })))
}
```

### Pattern 2: Response Transformation

Transform model responses for consistency:

```rust
async fn after_model_call(&self, context: &ModelHookContext) -> Result<HookExecutionResult> {
    if let Some(response) = &context.response {
        // Normalize whitespace
        let normalized = response
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join("\n");

        // Add prefix/suffix
        let transformed = format!("[PROCESSED]\n{}\n[/PROCESSED]", normalized);

        Ok(HookExecutionResult::with_data(json!({
            "response": transformed
        })))
    } else {
        Ok(HookExecutionResult::success())
    }
}
```

### Pattern 3: Security Checks

Implement security checks for tool execution:

```rust
async fn before_tool_execution(&self, context: &ToolHookContext) -> Result<HookExecutionResult> {
    // Block dangerous tools
    let dangerous_tools = vec!["delete_file", "rm", "format_disk"];
    if dangerous_tools.contains(&context.tool_name.as_str()) {
        return Ok(HookExecutionResult::stop("Dangerous tool blocked"));
    }

    // Check for path traversal
    if let Some(path) = context.arguments.get("path").and_then(|v| v.as_str()) {
        if path.contains("..") || path.starts_with("/") {
            return Ok(HookExecutionResult::stop("Invalid path detected"));
        }
    }

    Ok(HookExecutionResult::success())
}
```

### Pattern 4: Rate Limiting

Implement rate limiting for model calls:

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct RateLimitHook {
    name: String,
    priority: HookPriority,
    limits: Arc<RwLock<HashMap<String, (u32, Instant)>>>,
    max_calls: u32,
    window: Duration,
}

async fn before_model_call(&self, context: &ModelHookContext) -> Result<HookExecutionResult> {
    let mut limits = self.limits.write().await;
    let key = context.model_id.clone();
    let now = Instant::now();

    // Clean up old entries
    limits.retain(|_, (_, time)| now.duration_since(*time) < self.window);

    // Check rate limit
    let (count, start) = limits.entry(key.clone()).or_insert((0, now));
    if now.duration_since(*start) < self.window {
        if *count >= self.max_calls {
            return Ok(HookExecutionResult::stop("Rate limit exceeded"));
        }
        *count += 1;
    } else {
        *count = 1;
        *start = now;
    }

    Ok(HookExecutionResult::success())
}
```

### Pattern 5: Error Recovery

Implement automatic error recovery:

```rust
async fn error_recovery(&self, context: &ErrorHookContext) -> Result<HookExecutionResult> {
    // Retry network errors
    if context.error_type == "NetworkError" {
        // Attempt recovery
        match attempt_recovery().await {
            Ok(_) => {
                return Ok(HookExecutionResult::with_data(json!({
                    "recovered_error": "Network connection restored"
                })));
            }
            Err(e) => {
                tracing::warn!("Recovery failed: {}", e);
            }
        }
    }

    // Retry timeout errors with backoff
    if context.error_type == "TimeoutError" {
        tokio::time::sleep(Duration::from_secs(1)).await;
        match retry_operation().await {
            Ok(result) => {
                return Ok(HookExecutionResult::with_data(json!({
                    "recovered_error": "Operation retried successfully",
                    "result": result
                })));
            }
            Err(_) => {}
        }
    }

    Ok(HookExecutionResult::success())
}
```

### Pattern 6: Cost Tracking

Track costs across all model calls:

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct CostTrackingHook {
    name: String,
    priority: HookPriority,
    total_cost: Arc<RwLock<f64>>,
    daily_cost: Arc<RwLock<f64>>,
    last_reset: Arc<RwLock<chrono::DateTime<chrono::Utc>>>,
}

async fn execute(&self, context: &HookContext) -> Result<HookExecutionResult> {
    if let Some(cost) = context.data.get("estimated_cost").and_then(|v| v.as_f64()) {
        // Update total cost
        {
            let mut total = self.total_cost.write().await;
            *total += cost;
        }

        // Update daily cost
        {
            let mut daily = self.daily_cost.write().await;
            let mut last_reset = self.last_reset.write().await;
            let now = chrono::Utc::now();

            // Reset daily cost if new day
            if now.date_naive() != last_reset.date_naive() {
                *daily = 0.0;
                *last_reset = now;
            }

            *daily += cost;
        }

        // Alert if daily limit exceeded
        {
            let daily = self.daily_cost.read().await;
            if *daily > DAILY_LIMIT {
                tracing::warn!("Daily cost limit exceeded: ${:.2}", daily);
            }
        }
    }

    Ok(HookExecutionResult::success())
}
```

### Pattern 7: Response Caching

Cache model responses to reduce costs:

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct CacheHook {
    name: String,
    priority: HookPriority,
    cache: Arc<RwLock<HashMap<String, (String, Instant)>>>,
    ttl: Duration,
}

async fn before_model_call(&self, context: &ModelHookContext) -> Result<HookExecutionResult> {
    // Check cache
    let cache_key = format!("{}:{}", context.model_id, context.input);
    let cache = self.cache.read().await;

    if let Some((cached_response, timestamp)) = cache.get(&cache_key) {
        if Instant::now().duration_since(*timestamp) < self.ttl {
            // Return cached response
            return Ok(HookExecutionResult::with_data(json!({
                "cached": true,
                "response": cached_response
            })));
        }
    }

    drop(cache);
    Ok(HookExecutionResult::success())
}

async fn after_model_call(&self, context: &ModelHookContext) -> Result<HookExecutionResult> {
    // Store in cache
    if let Some(response) = &context.response {
        let cache_key = format!("{}:{}", context.model_id, context.input);
        let mut cache = self.cache.write().await;

        // Clean up old entries
        cache.retain(|_, (_, timestamp)| {
            Instant::now().duration_since(*timestamp) < self.ttl
        });

        cache.insert(cache_key, (response.clone(), Instant::now()));
    }

    Ok(HookExecutionResult::success())
}
```

### Pattern 8: Audit Logging

Log all tool executions for audit purposes:

```rust
use std::fs::OpenOptions;
use std::io::Write;

pub struct AuditHook {
    name: String,
    priority: HookPriority,
    log_file: String,
}

async fn after_tool_execution(&self, context: &ToolHookContext) -> Result<HookExecutionResult> {
    let audit_entry = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "tool": context.tool_name,
        "arguments": context.arguments,
        "result": context.result,
    });

    // Write to audit log
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&self.log_file)?;

    writeln!(file, "{}", serde_json::to_string(&audit_entry)?)?;

    Ok(HookExecutionResult::success())
}
```

## Configuration Examples

### Basic Configuration

```toml
[[hooks]]
name = "my-hook"
type = "before_model"
priority = 100
enabled = true
```

### Configuration with Options

```toml
[[hooks]]
name = "validation-hook"
type = "before_model"
priority = 200
enabled = true

[hooks.config]
max_input_length = 10000
validate_format = true
allowed_formats = ["text", "markdown"]
```

### Multiple Hooks

```toml
[[hooks]]
name = "logging-hook"
type = "before_model"
priority = 100
enabled = true

[[hooks]]
name = "metrics-hook"
type = "telemetry_collection"
priority = 50
enabled = true

[[hooks]]
name = "validation-hook"
type = "before_tool"
priority = 200
enabled = true
```

## Integration Examples

### Registering Multiple Hooks

```rust
use radium_core::hooks::registry::HookRegistry;
use std::sync::Arc;

let registry = Arc::new(HookRegistry::new());

// Register logging hooks
let before_log = create_before_hook();
let after_log = create_after_hook();
registry.register(before_log).await?;
registry.register(after_log).await?;

// Register metrics hook
let metrics = create_metrics_hook();
registry.register(metrics).await?;

// Register validation hook
let validation = create_validation_hook();
registry.register(validation).await?;
```

### Using with Orchestrator

```rust
use radium_core::hooks::integration::OrchestratorHooks;

let registry = Arc::new(HookRegistry::new());
// ... register hooks ...

let orchestrator_hooks = OrchestratorHooks::new(registry);

// Before model call
let (modified_input, modifications) = orchestrator_hooks
    .before_model_call(input, model_id)
    .await?;

// After model call
let modified_response = orchestrator_hooks
    .after_model_call(input, model_id, response)
    .await?;
```

## Testing Examples

### Unit Test

```rust
#[tokio::test]
async fn test_validation_hook() {
    let hook = ValidationHook::new("test", 100);
    let context = ModelHookContext::before(
        "test input".to_string(),
        "test-model".to_string(),
    );

    let result = hook.before_model_call(&context).await.unwrap();
    assert!(result.success);
    assert!(result.should_continue);
}
```

### Integration Test

```rust
#[tokio::test]
async fn test_hook_registry() {
    let registry = Arc::new(HookRegistry::new());
    let hook = create_test_hook();
    
    registry.register(hook).await.unwrap();
    
    let hooks = registry.get_hooks(HookType::BeforeModel).await;
    assert_eq!(hooks.len(), 1);
}
```

## Next Steps

- See [Creating Hooks](creating-hooks.md) for step-by-step hook creation
- Review [Hook Types](hook-types.md) for detailed hook type information
- Check [API Reference](api-reference.md) for complete API documentation

