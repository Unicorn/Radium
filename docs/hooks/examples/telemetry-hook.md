# Telemetry Hook Example

Example of implementing a telemetry collection hook.

## Implementation

```rust
use radium_core::hooks::telemetry::TelemetryHookContext;
use radium_core::hooks::types::{HookPriority, HookResult};
use radium_core::hooks::error::Result;
use async_trait::async_trait;
use std::sync::Arc;
use serde_json::json;

pub struct TelemetryCollector {
    endpoint: String,
    api_key: Option<String>,
}

impl TelemetryCollector {
    pub fn new(endpoint: impl Into<String>, api_key: Option<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            api_key,
        }
    }
}

#[async_trait]
impl Hook for TelemetryCollector {
    fn name(&self) -> &str {
        "telemetry-collector"
    }

    fn priority(&self) -> HookPriority {
        HookPriority::new(50)
    }

    fn hook_type(&self) -> HookType {
        HookType::TelemetryCollection
    }

    async fn execute(&self, context: &HookContext) -> Result<HookResult> {
        // Extract telemetry data from context
        let event_type = context.data.get("event_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        
        let data = context.data.get("data")
            .cloned()
            .unwrap_or(json!({}));

        // Send to telemetry endpoint (async, fire-and-forget)
        let endpoint = self.endpoint.clone();
        let api_key = self.api_key.clone();
        let payload = json!({
            "event_type": event_type,
            "data": data,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        tokio::spawn(async move {
            let client = reqwest::Client::new();
            let mut request = client.post(&endpoint).json(&payload);
            
            if let Some(key) = api_key {
                request = request.header("Authorization", format!("Bearer {}", key));
            }

            if let Err(e) = request.send().await {
                eprintln!("Failed to send telemetry: {}", e);
            }
        });

        Ok(HookResult::success())
    }
}
```

## Usage

```rust
let registry = Arc::new(HookRegistry::new());
let collector = Arc::new(TelemetryCollector::new(
    "https://api.example.com/telemetry",
    Some("api-key".to_string()),
));
registry.register(collector).await?;
```

## Configuration

```toml
[[hooks]]
name = "telemetry-collector"
type = "telemetry_collection"
priority = 50
enabled = true

[hooks.config]
endpoint = "https://api.example.com/telemetry"
api_key = "${TELEMETRY_API_KEY}"
```

