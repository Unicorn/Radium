# Migration Guide

Guide for migrating existing systems to use the unified hooks system.

## Overview

The hooks system provides a unified interface for intercepting and customizing behavior. This guide helps you migrate from existing ad-hoc interception mechanisms to the hooks system.

## Migration Steps

### 1. Identify Interception Points

Identify where your code currently intercepts execution:
- Model API calls
- Tool execution
- Error handling
- Workflow steps

### 2. Map to Hook Types

Map your interception points to hook types:
- Model calls → `ModelHook`
- Tool execution → `ToolHook`
- Error handling → `ErrorHook`
- Workflow behaviors → `BehaviorEvaluatorAdapter`

### 3. Implement Hooks

Convert your interception logic to hooks:

**Before**:
```rust
fn before_model_call(input: &str) -> String {
    // Custom logic
    format!("[PREFIX] {}", input)
}
```

**After**:
```rust
#[async_trait]
impl ModelHook for MyHook {
    async fn before_model_call(&self, context: &ModelHookContext) -> Result<HookResult> {
        let modified = format!("[PREFIX] {}", context.input);
        Ok(HookResult::with_data(json!({
            "modified_input": modified
        })))
    }
}
```

### 4. Register Hooks

Replace direct function calls with hook registration:

**Before**:
```rust
let modified = before_model_call(&input);
```

**After**:
```rust
let registry = Arc::new(HookRegistry::new());
registry.register(Arc::new(MyHook)).await?;
let hooks = OrchestratorHooks::new(registry);
let (modified, _) = hooks.before_model_call(&input, &model_id).await?;
```

## Common Migration Patterns

### Pattern 1: Logging

**Before**: Direct logging calls
**After**: Logging hook with configuration

### Pattern 2: Validation

**Before**: Inline validation
**After**: Validation hook with early exit

### Pattern 3: Transformation

**Before**: Transformation functions
**After**: Transformation hooks with modified data

### Pattern 4: Error Handling

**Before**: Try-catch blocks
**After**: Error hooks with recovery strategies

## Workflow Behavior Migration

For workflow behaviors, use `BehaviorEvaluatorAdapter`:

```rust
use radium_core::hooks::adapters::BehaviorHookRegistrar;

let registry = Arc::new(HookRegistry::new());
let evaluator = Arc::new(LoopEvaluator::new());

BehaviorHookRegistrar::register_behavior_hook(
    &registry,
    evaluator,
    "loop-behavior",
    HookPriority::new(100),
).await?;
```

## Backward Compatibility

The hooks system maintains backward compatibility:
- Existing behavior evaluators work via adapters
- Configuration files are optional
- Hooks can be added incrementally

## Testing Migration

1. **Unit Tests**: Test hooks in isolation
2. **Integration Tests**: Test hook execution in context
3. **Performance Tests**: Verify <5% overhead
4. **Regression Tests**: Ensure existing functionality works

## Rollback Plan

If issues arise:
1. Disable hooks via configuration (`enabled = false`)
2. Remove hook registration code
3. Restore original interception logic

## Best Practices

1. **Migrate incrementally**: One hook type at a time
2. **Test thoroughly**: Ensure behavior matches original
3. **Monitor performance**: Verify overhead is acceptable
4. **Document changes**: Update documentation as you migrate

## Support

For migration assistance, see the main project documentation or open an issue.

