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
        Self {
            event_type: event_type.into(),
            data,
            metadata: None,
        }
    }

    /// Create a new telemetry hook context with metadata.
    pub fn with_metadata(
        event_type: impl Into<String>,
        data: serde_json::Value,
        metadata: serde_json::Value,
    ) -> Self {
        Self {
            event_type: event_type.into(),
            data,
            metadata: Some(metadata),
        }
    }

    /// Convert to hook context.
    pub fn to_hook_context(&self, hook_type: &str) -> HookContext {
        HookContext::new(
            hook_type,
            serde_json::to_value(self).unwrap_or(serde_json::Value::Null),
        )
    }
}

