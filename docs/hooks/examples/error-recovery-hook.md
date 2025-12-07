# Error Recovery Hook Example

Example of implementing an error recovery hook with retry logic.

## Implementation

```rust
use radium_core::hooks::error_hooks::{ErrorHook, ErrorHookContext};
use radium_core::hooks::types::{HookPriority, HookResult};
use radium_core::hooks::error::Result;
use async_trait::async_trait;
use std::sync::Arc;
use serde_json::json;

pub struct ErrorRecoveryHook {
    max_retries: u32,
    retryable_errors: Vec<String>,
}

impl ErrorRecoveryHook {
    pub fn new(max_retries: u32) -> Self {
        Self {
            max_retries,
            retryable_errors: vec![
                "network_error".to_string(),
                "timeout".to_string(),
                "rate_limit".to_string(),
            ],
        }
    }
}

#[async_trait]
impl ErrorHook for ErrorRecoveryHook {
    fn name(&self) -> &str {
        "error-recovery"
    }

    fn priority(&self) -> HookPriority {
        HookPriority::new(200) // High priority for error handling
    }

    async fn intercept_error(
        &self,
        context: &ErrorHookContext,
    ) -> Result<HookResult> {
        // Check if error is retryable
        if !self.retryable_errors.contains(&context.error_type) {
            return Ok(HookResult::success());
        }

        // Check retry count
        let retry_count = context.metadata
            .get("retry_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        if retry_count >= self.max_retries {
            return Ok(HookResult::error(format!(
                "Max retries ({}) exceeded for error: {}",
                self.max_retries, context.error_message
            )));
        }

        // Implement exponential backoff
        let delay_ms = 100 * 2_u64.pow(retry_count);
        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;

        // Return retry instruction
        Ok(HookResult::with_data(json!({
            "retry": true,
            "retry_count": retry_count + 1,
            "delay_ms": delay_ms,
        })))
    }
}
```

## Usage

```rust
let registry = Arc::new(HookRegistry::new());
let recovery = Arc::new(ErrorRecoveryHook::new(3));
registry.register(recovery).await?;
```

## Configuration

```toml
[[hooks]]
name = "error-recovery"
type = "error_interception"
priority = 200
enabled = true

[hooks.config]
max_retries = 3
retryable_errors = ["network_error", "timeout", "rate_limit"]
```

