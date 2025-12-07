# Hooks API Reference

Complete API reference for the Radium Hooks System.

## Core Types

### HookRegistry

Central registry for managing hooks.

```rust
pub struct HookRegistry {
    // ...
}

impl HookRegistry {
    pub fn new() -> Self;
    pub async fn register(&self, hook: Arc<dyn Hook>) -> Result<()>;
    pub async fn unregister(&self, name: &str) -> Option<Arc<dyn Hook>>;
    pub async fn execute_hooks(
        &self,
        hook_type: HookType,
        context: &HookContext,
    ) -> Result<Vec<HookResult>>;
    pub fn list_hooks(&self, hook_type: Option<HookType>) -> Vec<&dyn Hook>;
    pub async fn enable_hook(&self, name: &str) -> Result<()>;
    pub async fn disable_hook(&self, name: &str) -> Result<()>;
}
```

### Hook Trait

Base trait for all hooks.

```rust
#[async_trait]
pub trait Hook: Send + Sync {
    fn name(&self) -> &str;
    fn priority(&self) -> HookPriority;
    fn hook_type(&self) -> HookType;
    async fn execute(&self, context: &HookContext) -> Result<HookResult>;
}
```

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

Result returned by hooks.

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

## Model Hooks

### ModelHook Trait

```rust
#[async_trait]
pub trait ModelHook: Hook {
    async fn before_model_call(
        &self,
        context: &ModelHookContext,
    ) -> Result<HookResult>;
    
    async fn after_model_call(
        &self,
        context: &ModelHookContext,
    ) -> Result<HookResult>;
}
```

### ModelHookContext

```rust
pub struct ModelHookContext {
    pub input: String,
    pub model_id: String,
    pub response: Option<String>,
}
```

## Tool Hooks

### ToolHook Trait

```rust
#[async_trait]
pub trait ToolHook: Hook {
    async fn before(
        &self,
        context: &ToolHookContext,
    ) -> Result<HookResult>;
    
    async fn after(
        &self,
        context: &ToolHookContext,
    ) -> Result<HookResult>;
    
    async fn selection(
        &self,
        context: &ToolHookContext,
    ) -> Result<HookResult>;
}
```

### ToolHookContext

```rust
pub struct ToolHookContext {
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub result: Option<serde_json::Value>,
}
```

## Error Hooks

### ErrorHook Trait

```rust
#[async_trait]
pub trait ErrorHook: Hook {
    async fn intercept_error(
        &self,
        context: &ErrorHookContext,
    ) -> Result<HookResult>;
}
```

### ErrorHookContext

```rust
pub struct ErrorHookContext {
    pub error_message: String,
    pub error_type: String,
    pub error_source: Option<String>,
}
```

## Integration Helpers

### OrchestratorHooks

Helper for integrating hooks with orchestrator.

```rust
pub struct OrchestratorHooks {
    pub registry: Arc<HookRegistry>,
}

impl OrchestratorHooks {
    pub fn new(registry: Arc<HookRegistry>) -> Self;
    pub async fn before_model_call(
        &self,
        input: &str,
        model_id: &str,
    ) -> Result<(String, Option<serde_json::Value>)>;
    pub async fn after_model_call(
        &self,
        input: &str,
        model_id: &str,
        response: &str,
    ) -> Result<String>;
    pub async fn before_tool_execution(
        &self,
        tool_name: &str,
        arguments: &serde_json::Value,
    ) -> Result<serde_json::Value>;
    pub async fn after_tool_execution(
        &self,
        tool_name: &str,
        arguments: &serde_json::Value,
        result: &serde_json::Value,
    ) -> Result<serde_json::Value>;
}
```

## Configuration

### HookConfig

```rust
pub struct HookConfig {
    pub name: String,
    pub hook_type: String,
    pub priority: Option<u32>,
    pub enabled: Option<bool>,
    pub config: Option<HashMap<String, serde_json::Value>>,
}
```

### HookConfigFile

```rust
pub struct HookConfigFile {
    pub hooks: Vec<HookConfig>,
}

impl HookConfigFile {
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self>;
}
```

## Adapters

### BehaviorEvaluatorAdapter

Adapter for integrating behavior evaluators as hooks.

```rust
pub struct BehaviorEvaluatorAdapter<E: BehaviorEvaluator> {
    // ...
}

impl<E: BehaviorEvaluator + Send + Sync + 'static> Hook 
    for BehaviorEvaluatorAdapter<E> 
where
    E::Decision: Send + 'static,
{
    // ...
}
```

