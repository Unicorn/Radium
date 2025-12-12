---
id: "api-reference"
title: "Hooks System - API Reference"
sidebar_label: "Hooks System - API Ref"
---

# Hooks System - API Reference

Complete API documentation for the Radium hooks system.

## Core Types

### HookPriority

Priority for hook execution order. Higher priority hooks execute first.

```rust
pub struct HookPriority(pub u32);

impl HookPriority {
    pub fn new(priority: u32) -> Self;
    pub fn value(&self) -> u32;
}
```

**Default**: 100

### HookContext

Context passed to hooks during execution.

```rust
pub struct HookContext {
    pub hook_type: String,
    pub data: serde_json::Value,
    pub metadata: serde_json::Value,
}

impl HookContext {
    pub fn new(hook_type: impl Into<String>, data: serde_json::Value) -> Self;
    pub fn with_metadata(
        hook_type: impl Into<String>,
        data: serde_json::Value,
        metadata: serde_json::Value,
    ) -> Self;
}
```

### HookResult

Result of hook execution.

```rust
pub struct HookResult {
    pub success: bool,
    pub message: Option<String>,
    pub modified_data: Option<serde_json::Value>,
    pub should_continue: bool,
}

impl HookResult {
    pub fn success() -> Self;
    pub fn with_data(data: serde_json::Value) -> Self;
    pub fn stop(message: impl Into<String>) -> Self;
    pub fn error(message: impl Into<String>) -> Self;
}
```

## Hook Trait

Core trait for all hook implementations.

```rust
#[async_trait]
pub trait Hook: Send + Sync {
    fn name(&self) -> &str;
    fn priority(&self) -> HookPriority;
    fn hook_type(&self) -> HookType;
    async fn execute(&self, context: &HookContext) -> Result<HookExecutionResult>;
}
```

## Hook Types

Enumeration of all supported hook types.

```rust
pub enum HookType {
    BeforeModel,
    AfterModel,
    BeforeTool,
    AfterTool,
    ToolSelection,
    ErrorInterception,
    ErrorTransformation,
    ErrorRecovery,
    ErrorLogging,
    TelemetryCollection,
    CustomLogging,
    MetricsAggregation,
    PerformanceMonitoring,
}

impl HookType {
    pub fn as_str(&self) -> &'static str;
}
```

## HookRegistry

Registry for managing and executing hooks.

```rust
pub struct HookRegistry {
    hooks: Arc<RwLock<Vec<Arc<dyn Hook>>>>,
}

impl HookRegistry {
    pub fn new() -> Self;
    pub fn clone(&self) -> Self;
    
    pub async fn register(&self, hook: Arc<dyn Hook>) -> Result<()>;
    pub async fn unregister(&self, name: &str) -> Result<()>;
    pub async fn get_hooks(&self, hook_type: HookType) -> Vec<Arc<dyn Hook>>;
    pub async fn execute_hooks(
        &self,
        hook_type: HookType,
        context: &HookContext,
    ) -> Result<Vec<HookExecutionResult>>;
    pub async fn clear(&self);
    pub async fn count(&self) -> usize;
}
```

## Model Hooks

### ModelHook Trait

Trait for model call hooks.

```rust
#[async_trait]
pub trait ModelHook: Send + Sync {
    fn name(&self) -> &str;
    fn priority(&self) -> HookPriority;
    async fn before_model_call(&self, context: &ModelHookContext) -> Result<HookExecutionResult>;
    async fn after_model_call(&self, context: &ModelHookContext) -> Result<HookExecutionResult>;
}
```

### ModelHookContext

Context for model call hooks.

```rust
pub struct ModelHookContext {
    pub input: String,
    pub model_id: String,
    pub request_modifications: Option<serde_json::Value>,
    pub response: Option<String>,
    pub modified_input: Option<String>,
}

impl ModelHookContext {
    pub fn before(input: String, model_id: String) -> Self;
    pub fn after(input: String, model_id: String, response: String) -> Self;
    pub fn to_hook_context(&self, hook_type: ModelHookType) -> HookContext;
}
```

### ModelHookAdapter

Adapter to convert `ModelHook` to `Hook` trait.

```rust
pub struct ModelHookAdapter {
    hook: Arc<dyn ModelHook>,
    hook_type: ModelHookType,
}

impl ModelHookAdapter {
    pub fn before(hook: Arc<dyn ModelHook>) -> Arc<dyn Hook>;
    pub fn after(hook: Arc<dyn ModelHook>) -> Arc<dyn Hook>;
}
```

## Tool Hooks

### ToolHook Trait

Trait for tool execution hooks.

```rust
#[async_trait]
pub trait ToolHook: Send + Sync {
    fn name(&self) -> &str;
    fn priority(&self) -> HookPriority;
    async fn before_tool_execution(&self, context: &ToolHookContext) -> Result<HookExecutionResult>;
    async fn after_tool_execution(&self, context: &ToolHookContext) -> Result<HookExecutionResult>;
    async fn tool_selection(&self, context: &ToolHookContext) -> Result<HookExecutionResult>;
}
```

### ToolHookContext

Context for tool execution hooks.

```rust
pub struct ToolHookContext {
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub modified_arguments: Option<serde_json::Value>,
    pub modified_result: Option<serde_json::Value>,
}

impl ToolHookContext {
    pub fn before(tool_name: String, arguments: serde_json::Value) -> Self;
    pub fn after(tool_name: String, arguments: serde_json::Value, result: serde_json::Value) -> Self;
    pub fn selection(tool_name: String, arguments: serde_json::Value) -> Self;
    pub fn to_hook_context(&self, hook_type: ToolHookType) -> HookContext;
}
```

### ToolHookAdapter

Adapter to convert `ToolHook` to `Hook` trait.

```rust
pub struct ToolHookAdapter {
    hook: Arc<dyn ToolHook>,
    hook_type: ToolHookType,
}

impl ToolHookAdapter {
    pub fn before(hook: Arc<dyn ToolHook>) -> Arc<dyn Hook>;
    pub fn after(hook: Arc<dyn ToolHook>) -> Arc<dyn Hook>;
    pub fn selection(hook: Arc<dyn ToolHook>) -> Arc<dyn Hook>;
}
```

## Error Hooks

### ErrorHook Trait

Trait for error handling hooks.

```rust
#[async_trait]
pub trait ErrorHook: Send + Sync {
    fn name(&self) -> &str;
    fn priority(&self) -> HookPriority;
    async fn error_interception(&self, context: &ErrorHookContext) -> HookErrorResult<HookExecutionResult>;
    async fn error_transformation(&self, context: &ErrorHookContext) -> HookErrorResult<HookExecutionResult>;
    async fn error_recovery(&self, context: &ErrorHookContext) -> HookErrorResult<HookExecutionResult>;
    async fn error_logging(&self, context: &ErrorHookContext) -> HookErrorResult<HookExecutionResult>;
}
```

### ErrorHookContext

Context for error handling hooks.

```rust
pub struct ErrorHookContext {
    pub error_message: String,
    pub error_type: String,
    pub error_source: Option<String>,
    pub recovered: bool,
}

impl ErrorHookContext {
    pub fn interception(error_message: String, error_type: String, error_source: Option<String>) -> Self;
    pub fn transformation(error_message: String, error_type: String, error_source: Option<String>) -> Self;
    pub fn recovery(error_message: String, error_type: String, error_source: Option<String>) -> Self;
    pub fn logging(error_message: String, error_type: String, error_source: Option<String>) -> Self;
    pub fn to_hook_context(&self, hook_type: ErrorHookType) -> HookContext;
}
```

## Telemetry Hooks

### TelemetryHookContext

Context for telemetry hooks.

```rust
pub struct TelemetryHookContext {
    pub event_type: String,
    pub data: serde_json::Value,
    pub metadata: Option<serde_json::Value>,
}

impl TelemetryHookContext {
    pub fn new(event_type: impl Into<String>, data: serde_json::Value) -> Self;
    pub fn with_metadata(
        event_type: impl Into<String>,
        data: serde_json::Value,
        metadata: serde_json::Value,
    ) -> Self;
    pub fn to_hook_context(&self, hook_type: &str) -> HookContext;
}
```

Telemetry data structure:

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

## Hook Configuration

### HookConfig

Configuration for hooks loaded from TOML files.

```rust
pub struct HookConfig {
    pub hooks: Vec<HookDefinition>,
}

pub struct HookDefinition {
    pub name: String,
    #[serde(rename = "type")]
    pub hook_type: String,
    pub priority: Option<u32>,
    pub script: Option<String>,
    pub config: Option<toml::Value>,
}

impl HookConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self>;
    pub fn from_str(content: &str) -> Result<Self>;
    pub fn validate(&self) -> Result<()>;
}
```

Example configuration:

```toml
[[hooks]]
name = "my-hook"
type = "before_model"
priority = 100
enabled = true

[hooks.config]
log_level = "info"
```

## Hook Loader

### HookLoader

Loader for discovering and loading hooks from extensions and workspace.

```rust
pub struct HookLoader {
    factories: HashMap<String, HookFactory>,
}

pub type HookFactory = fn(&HookDefinition) -> Result<Option<`Arc<dyn Hook>`>>;

impl HookLoader {
    pub fn new() -> Self;
    pub fn register_factory(&mut self, pattern: impl Into<String>, factory: HookFactory);
    pub async fn load_hooks_from_config<P: AsRef<Path>>(
        &self,
        config: &HookConfig,
        registry: &Arc<HookRegistry>,
        workspace_root: Option<P>,
    ) -> Result<usize>;
    pub async fn load_from_extensions(registry: &Arc<HookRegistry>) -> Result<usize>;
    pub async fn load_from_directory<P: AsRef<Path>>(
        dir: P,
        registry: &Arc<HookRegistry>,
    ) -> Result<usize>;
    pub async fn load_from_workspace<P: AsRef<Path>>(
        workspace_root: P,
        registry: &Arc<HookRegistry>,
    ) -> Result<usize>;
    pub fn discover_config_files() -> Result<Vec<PathBuf>>;
}
```

**Note**: For v1.0, hooks must be registered programmatically. The loader discovers configurations and sets enable/disable state. Hook factories can be registered to automatically instantiate hooks from configurations. Dynamic library loading is deferred to v2.0.

## Error Types

### HookError

Errors that can occur in the hooks system.

```rust
#[derive(Error, Debug)]
pub enum HookError {
    #[error("Failed to register hook: {0}")]
    RegistrationFailed(String),
    
    #[error("Hook execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Hook not found: {0}")]
    NotFound(String),
    
    #[error("Invalid hook configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Hook validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Configuration parsing error: {0}")]
    ConfigParse(#[from] toml::de::Error),
    
    #[error("Hook discovery error: {0}")]
    Discovery(String),
}

pub type Result<T> = std::result::Result<T, HookError>;
```

## Integration Points

### OrchestratorHooks

Helper for executing hooks in the orchestrator.

```rust
pub struct OrchestratorHooks {
    registry: Arc<HookRegistry>,
}

impl OrchestratorHooks {
    pub fn new(registry: Arc<HookRegistry>) -> Self;
    pub async fn before_model_call(&self, input: &str, model_id: &str) -> Result<(String, Option<serde_json::Value>)>;
    pub async fn after_model_call(&self, input: &str, model_id: &str, response: &str) -> Result<String>;
    pub async fn before_tool_execution(&self, tool_name: &str, arguments: &serde_json::Value) -> Result<serde_json::Value>;
    pub async fn after_tool_execution(&self, tool_name: &str, arguments: &serde_json::Value, result: &serde_json::Value) -> Result<serde_json::Value>;
    pub async fn tool_selection(&self, tool_name: &str, arguments: &serde_json::Value) -> Result<bool>;
    pub async fn error_interception(&self, error_message: &str, error_type: &str, error_source: Option<&str>) -> Result<Option<String>>;
    pub async fn error_transformation(&self, error_message: &str, error_type: &str, error_source: Option<&str>) -> Result<Option<String>>;
    pub async fn error_recovery(&self, error_message: &str, error_type: &str, error_source: Option<&str>) -> Result<Option<String>>;
    pub async fn telemetry_collection(&self, event_type: &str, data: &serde_json::Value) -> Result<()>;
}
```

## Examples

See example implementations in:
- `examples/hooks/logging-hook/` - Model call logging
- `examples/hooks/metrics-hook/` - Telemetry aggregation

## Related Documentation

- [Getting Started Guide](getting-started.md)
- [Hook Development Guide](hook-development.md)

