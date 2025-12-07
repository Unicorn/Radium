//! Core types for the hooks system.

use serde::{Deserialize, Serialize};

/// Priority for hook execution order.
///
/// Hooks with higher priority values execute first.
/// Default priority is 100.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct HookPriority(pub u32);

impl Default for HookPriority {
    fn default() -> Self {
        Self(100)
    }
}

impl HookPriority {
    /// Create a new hook priority.
    pub fn new(priority: u32) -> Self {
        Self(priority)
    }

    /// Get the priority value.
    pub fn value(&self) -> u32 {
        self.0
    }
}

/// Context passed to hooks during execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookContext {
    /// The type of hook being executed.
    pub hook_type: String,
    /// Additional context data (hook-specific).
    pub data: serde_json::Value,
    /// Metadata about the execution context.
    pub metadata: serde_json::Value,
}

impl HookContext {
    /// Create a new hook context.
    pub fn new(hook_type: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            hook_type: hook_type.into(),
            data,
            metadata: serde_json::json!({}),
        }
    }

    /// Create a new hook context with metadata.
    pub fn with_metadata(
        hook_type: impl Into<String>,
        data: serde_json::Value,
        metadata: serde_json::Value,
    ) -> Self {
        Self {
            hook_type: hook_type.into(),
            data,
            metadata,
        }
    }
}

/// Result of hook execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResult {
    /// Whether the hook execution was successful.
    pub success: bool,
    /// Optional message from the hook.
    pub message: Option<String>,
    /// Optional modified data from the hook.
    pub modified_data: Option<serde_json::Value>,
    /// Whether execution should continue.
    pub should_continue: bool,
}

impl Default for HookResult {
    fn default() -> Self {
        Self {
            success: true,
            message: None,
            modified_data: None,
            should_continue: true,
        }
    }
}

impl HookResult {
    /// Create a successful hook result.
    pub fn success() -> Self {
        Self::default()
    }

    /// Create a successful hook result with modified data.
    pub fn with_data(data: serde_json::Value) -> Self {
        Self {
            success: true,
            message: None,
            modified_data: Some(data),
            should_continue: true,
        }
    }

    /// Create a hook result that stops execution.
    pub fn stop(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: Some(message.into()),
            modified_data: None,
            should_continue: false,
        }
    }

    /// Create a hook result with an error message but continue execution.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: Some(message.into()),
            modified_data: None,
            should_continue: true,
        }
    }
}

