//! Telemetry hooks.

use crate::hooks::types::HookContext;
use serde::{Deserialize, Serialize};

/// Context for telemetry hooks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryHookContext {
    /// The telemetry event type.
    pub event_type: String,
    /// The telemetry data.
    pub data: serde_json::Value,
    /// Optional metadata.
    pub metadata: Option<serde_json::Value>,
}

impl TelemetryHookContext {
    /// Create a new telemetry hook context.
    pub fn new(event_type: impl Into<String>, data: serde_json::Value) -> Self {
        Self { event_type: event_type.into(), data, metadata: None }
    }

    /// Create a new telemetry hook context with metadata.
    pub fn with_metadata(
        event_type: impl Into<String>,
        data: serde_json::Value,
        metadata: serde_json::Value,
    ) -> Self {
        Self { event_type: event_type.into(), data, metadata: Some(metadata) }
    }

    /// Convert to hook context.
    pub fn to_hook_context(&self, hook_type: &str) -> HookContext {
        HookContext::new(hook_type, serde_json::to_value(self).unwrap_or(serde_json::Value::Null))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_hook_context_new() {
        let ctx = TelemetryHookContext::new(
            "test_event",
            serde_json::json!({"key": "value"}),
        );
        assert_eq!(ctx.event_type, "test_event");
        assert_eq!(ctx.data, serde_json::json!({"key": "value"}));
        assert!(ctx.metadata.is_none());
    }

    #[test]
    fn test_telemetry_hook_context_with_metadata() {
        let ctx = TelemetryHookContext::with_metadata(
            "test_event",
            serde_json::json!({"key": "value"}),
            serde_json::json!({"meta": "data"}),
        );
        assert_eq!(ctx.event_type, "test_event");
        assert_eq!(ctx.data, serde_json::json!({"key": "value"}));
        assert_eq!(ctx.metadata, Some(serde_json::json!({"meta": "data"})));
    }

    #[test]
    fn test_telemetry_hook_context_to_hook_context() {
        let ctx = TelemetryHookContext::new("test_event", serde_json::json!({}));
        let hook_ctx = ctx.to_hook_context("telemetry_collection");
        assert_eq!(hook_ctx.hook_type, "telemetry_collection");
    }
}
