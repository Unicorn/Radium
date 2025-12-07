//! Error handling hooks.

use crate::hooks::error::Result as HookErrorResult;
use crate::hooks::registry::HookType;
use crate::hooks::types::{HookContext, HookPriority, HookResult as HookExecutionResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Type of error hook.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorHookType {
    /// Error interception.
    Interception,
    /// Error transformation.
    Transformation,
    /// Error recovery.
    Recovery,
    /// Error logging.
    Logging,
}

/// Context for error handling hooks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorHookContext {
    /// The error message.
    pub error_message: String,
    /// The error type/kind.
    pub error_type: String,
    /// The error source/location.
    pub error_source: Option<String>,
    /// Optional error data.
    pub error_data: Option<serde_json::Value>,
    /// Optional transformed error message.
    pub transformed_error: Option<String>,
    /// Whether the error was recovered.
    pub recovered: bool,
}

impl ErrorHookContext {
    /// Create a new error hook context for interception.
    pub fn interception(
        error_message: String,
        error_type: String,
        error_source: Option<String>,
    ) -> Self {
        Self {
            error_message,
            error_type,
            error_source,
            error_data: None,
            transformed_error: None,
            recovered: false,
        }
    }

    /// Create a new error hook context for transformation.
    pub fn transformation(
        error_message: String,
        error_type: String,
        error_source: Option<String>,
    ) -> Self {
        Self {
            error_message,
            error_type,
            error_source,
            error_data: None,
            transformed_error: None,
            recovered: false,
        }
    }

    /// Create a new error hook context for recovery.
    pub fn recovery(
        error_message: String,
        error_type: String,
        error_source: Option<String>,
    ) -> Self {
        Self {
            error_message,
            error_type,
            error_source,
            error_data: None,
            transformed_error: None,
            recovered: false,
        }
    }

    /// Create a new error hook context for logging.
    pub fn logging(
        error_message: String,
        error_type: String,
        error_source: Option<String>,
    ) -> Self {
        Self {
            error_message,
            error_type,
            error_source,
            error_data: None,
            transformed_error: None,
            recovered: false,
        }
    }

    /// Convert to hook context.
    pub fn to_hook_context(&self, hook_type: ErrorHookType) -> HookContext {
        let hook_type_str = match hook_type {
            ErrorHookType::Interception => "error_interception",
            ErrorHookType::Transformation => "error_transformation",
            ErrorHookType::Recovery => "error_recovery",
            ErrorHookType::Logging => "error_logging",
        };

        HookContext::new(
            hook_type_str,
            serde_json::to_value(self).unwrap_or(serde_json::Value::Null),
        )
    }
}

/// Trait for error handling hooks.
#[async_trait]
pub trait ErrorHook: Send + Sync {
    /// Get the name of the hook.
    fn name(&self) -> &str;

    /// Get the priority of the hook.
    fn priority(&self) -> HookPriority;

    /// Execute error interception hook.
    async fn error_interception(
        &self,
        context: &ErrorHookContext,
    ) -> HookErrorResult<HookExecutionResult>;

    /// Execute error transformation hook.
    async fn error_transformation(
        &self,
        context: &ErrorHookContext,
    ) -> HookErrorResult<HookExecutionResult>;

    /// Execute error recovery hook.
    async fn error_recovery(&self, context: &ErrorHookContext) -> HookErrorResult<HookExecutionResult>;

    /// Execute error logging hook.
    async fn error_logging(&self, context: &ErrorHookContext) -> HookErrorResult<HookExecutionResult>;
}

/// Adapter to convert ErrorHook to Hook trait.
pub struct ErrorHookAdapter {
    hook: Arc<dyn ErrorHook>,
    hook_type: ErrorHookType,
}

impl ErrorHookAdapter {
    /// Create a new adapter for error interception.
    pub fn interception(hook: Arc<dyn ErrorHook>) -> Arc<dyn crate::hooks::registry::Hook> {
        Arc::new(Self {
            hook,
            hook_type: ErrorHookType::Interception,
        })
    }

    /// Create a new adapter for error transformation.
    pub fn transformation(hook: Arc<dyn ErrorHook>) -> Arc<dyn crate::hooks::registry::Hook> {
        Arc::new(Self {
            hook,
            hook_type: ErrorHookType::Transformation,
        })
    }

    /// Create a new adapter for error recovery.
    pub fn recovery(hook: Arc<dyn ErrorHook>) -> Arc<dyn crate::hooks::registry::Hook> {
        Arc::new(Self {
            hook,
            hook_type: ErrorHookType::Recovery,
        })
    }

    /// Create a new adapter for error logging.
    pub fn logging(hook: Arc<dyn ErrorHook>) -> Arc<dyn crate::hooks::registry::Hook> {
        Arc::new(Self {
            hook,
            hook_type: ErrorHookType::Logging,
        })
    }
}

#[async_trait]
impl crate::hooks::registry::Hook for ErrorHookAdapter {
    fn name(&self) -> &str {
        self.hook.name()
    }

    fn priority(&self) -> HookPriority {
        self.hook.priority()
    }

    fn hook_type(&self) -> HookType {
        match self.hook_type {
            ErrorHookType::Interception => HookType::ErrorInterception,
            ErrorHookType::Transformation => HookType::ErrorTransformation,
            ErrorHookType::Recovery => HookType::ErrorRecovery,
            ErrorHookType::Logging => HookType::ErrorLogging,
        }
    }

    async fn execute(&self, context: &HookContext) -> HookErrorResult<HookExecutionResult> {
        let error_context: ErrorHookContext = serde_json::from_value(context.data.clone())
            .map_err(|e| crate::hooks::error::HookError::Serialization(e))?;

        match self.hook_type {
            ErrorHookType::Interception => {
                self.hook.error_interception(&error_context).await
            }
            ErrorHookType::Transformation => {
                self.hook.error_transformation(&error_context).await
            }
            ErrorHookType::Recovery => self.hook.error_recovery(&error_context).await,
            ErrorHookType::Logging => self.hook.error_logging(&error_context).await,
        }
    }
}
