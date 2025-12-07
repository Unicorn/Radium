# Logging Hook Example

A complete example of implementing a logging hook for model calls.

## Implementation

```rust
use radium_core::hooks::model::{ModelHook, ModelHookContext};
use radium_core::hooks::types::{HookPriority, HookResult};
use radium_core::hooks::error::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{info, debug};

pub struct ModelCallLogger {
    log_level: String,
}

impl ModelCallLogger {
    pub fn new(log_level: impl Into<String>) -> Self {
        Self {
            log_level: log_level.into(),
        }
    }
}

#[async_trait]
impl ModelHook for ModelCallLogger {
    fn name(&self) -> &str {
        "model-call-logger"
    }

    fn priority(&self) -> HookPriority {
        HookPriority::new(100)
    }

    async fn before_model_call(
        &self,
        context: &ModelHookContext,
    ) -> Result<HookResult> {
        match self.log_level.as_str() {
            "debug" => {
                debug!(
                    model_id = %context.model_id,
                    input_len = context.input.len(),
                    "Model call starting"
                );
            }
            _ => {
                info!(
                    model_id = %context.model_id,
                    "Calling model: {}",
                    context.model_id
                );
            }
        }
        Ok(HookResult::success())
    }

    async fn after_model_call(
        &self,
        context: &ModelHookContext,
    ) -> Result<HookResult> {
        match self.log_level.as_str() {
            "debug" => {
                debug!(
                    model_id = %context.model_id,
                    response_len = context.response.as_ref().map(|r| r.len()).unwrap_or(0),
                    "Model call completed"
                );
            }
            _ => {
                info!(
                    model_id = %context.model_id,
                    "Model call completed"
                );
            }
        }
        Ok(HookResult::success())
    }
}
```

## Usage

```rust
use radium_core::hooks::registry::HookRegistry;

let registry = Arc::new(HookRegistry::new());
let logger = Arc::new(ModelCallLogger::new("info"));
registry.register(logger).await?;
```

## Configuration

```toml
[[hooks]]
name = "model-call-logger"
type = "before_model_call"
priority = 100
enabled = true

[hooks.config]
log_level = "info"
```

