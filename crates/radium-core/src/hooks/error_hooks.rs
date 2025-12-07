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
    async fn error_recovery(
        &self,
        context: &ErrorHookContext,
    ) -> HookErrorResult<HookExecutionResult>;

    /// Execute error logging hook.
    async fn error_logging(
        &self,
        context: &ErrorHookContext,
    ) -> HookErrorResult<HookExecutionResult>;
}

/// Adapter to convert ErrorHook to Hook trait.
pub struct ErrorHookAdapter {
    hook: Arc<dyn ErrorHook>,
    hook_type: ErrorHookType,
}

impl ErrorHookAdapter {
    /// Create a new adapter for error interception.
    pub fn interception(hook: Arc<dyn ErrorHook>) -> Arc<dyn crate::hooks::registry::Hook> {
        Arc::new(Self { hook, hook_type: ErrorHookType::Interception })
    }

    /// Create a new adapter for error transformation.
    pub fn transformation(hook: Arc<dyn ErrorHook>) -> Arc<dyn crate::hooks::registry::Hook> {
        Arc::new(Self { hook, hook_type: ErrorHookType::Transformation })
    }

    /// Create a new adapter for error recovery.
    pub fn recovery(hook: Arc<dyn ErrorHook>) -> Arc<dyn crate::hooks::registry::Hook> {
        Arc::new(Self { hook, hook_type: ErrorHookType::Recovery })
    }

    /// Create a new adapter for error logging.
    pub fn logging(hook: Arc<dyn ErrorHook>) -> Arc<dyn crate::hooks::registry::Hook> {
        Arc::new(Self { hook, hook_type: ErrorHookType::Logging })
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
            ErrorHookType::Interception => self.hook.error_interception(&error_context).await,
            ErrorHookType::Transformation => self.hook.error_transformation(&error_context).await,
            ErrorHookType::Recovery => self.hook.error_recovery(&error_context).await,
            ErrorHookType::Logging => self.hook.error_logging(&error_context).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hooks::types::HookResult as HookExecutionResult;

    struct MockErrorHook {
        name: String,
        priority: HookPriority,
        interception_called: Arc<tokio::sync::Mutex<bool>>,
        transformation_called: Arc<tokio::sync::Mutex<bool>>,
        recovery_called: Arc<tokio::sync::Mutex<bool>>,
        logging_called: Arc<tokio::sync::Mutex<bool>>,
    }

    #[async_trait]
    impl ErrorHook for MockErrorHook {
        fn name(&self) -> &str {
            &self.name
        }

        fn priority(&self) -> HookPriority {
            self.priority
        }

        async fn error_interception(&self, _context: &ErrorHookContext) -> HookErrorResult<HookExecutionResult> {
            *self.interception_called.lock().await = true;
            Ok(HookExecutionResult::success())
        }

        async fn error_transformation(&self, _context: &ErrorHookContext) -> HookErrorResult<HookExecutionResult> {
            *self.transformation_called.lock().await = true;
            Ok(HookExecutionResult::success())
        }

        async fn error_recovery(&self, _context: &ErrorHookContext) -> HookErrorResult<HookExecutionResult> {
            *self.recovery_called.lock().await = true;
            Ok(HookExecutionResult::success())
        }

        async fn error_logging(&self, _context: &ErrorHookContext) -> HookErrorResult<HookExecutionResult> {
            *self.logging_called.lock().await = true;
            Ok(HookExecutionResult::success())
        }
    }

    #[test]
    fn test_error_hook_type_variants() {
        assert_eq!(ErrorHookType::Interception, ErrorHookType::Interception);
        assert_eq!(ErrorHookType::Transformation, ErrorHookType::Transformation);
        assert_eq!(ErrorHookType::Recovery, ErrorHookType::Recovery);
        assert_eq!(ErrorHookType::Logging, ErrorHookType::Logging);
    }

    #[test]
    fn test_error_hook_context_interception() {
        let ctx = ErrorHookContext::interception(
            "test error".to_string(),
            "TestError".to_string(),
            Some("test.rs:10".to_string()),
        );
        assert_eq!(ctx.error_message, "test error");
        assert_eq!(ctx.error_type, "TestError");
        assert_eq!(ctx.error_source, Some("test.rs:10".to_string()));
        assert!(!ctx.recovered);
        assert!(ctx.transformed_error.is_none());
    }

    #[test]
    fn test_error_hook_context_transformation() {
        let ctx = ErrorHookContext::transformation(
            "test error".to_string(),
            "TestError".to_string(),
            None,
        );
        assert_eq!(ctx.error_message, "test error");
        assert_eq!(ctx.error_type, "TestError");
        assert!(ctx.error_source.is_none());
    }

    #[test]
    fn test_error_hook_context_recovery() {
        let ctx = ErrorHookContext::recovery(
            "test error".to_string(),
            "TestError".to_string(),
            Some("test.rs:10".to_string()),
        );
        assert_eq!(ctx.error_message, "test error");
        assert!(!ctx.recovered);
    }

    #[test]
    fn test_error_hook_context_logging() {
        let ctx = ErrorHookContext::logging(
            "test error".to_string(),
            "TestError".to_string(),
            Some("test.rs:10".to_string()),
        );
        assert_eq!(ctx.error_message, "test error");
    }

    #[test]
    fn test_error_hook_context_to_hook_context() {
        let ctx = ErrorHookContext::interception(
            "test".to_string(),
            "Test".to_string(),
            None,
        );
        let hook_ctx = ctx.to_hook_context(ErrorHookType::Interception);
        assert_eq!(hook_ctx.hook_type, "error_interception");
    }

    #[tokio::test]
    async fn test_error_hook_adapter_interception() {
        let hook = Arc::new(MockErrorHook {
            name: "test".to_string(),
            priority: HookPriority::default(),
            interception_called: Arc::new(tokio::sync::Mutex::new(false)),
            transformation_called: Arc::new(tokio::sync::Mutex::new(false)),
            recovery_called: Arc::new(tokio::sync::Mutex::new(false)),
            logging_called: Arc::new(tokio::sync::Mutex::new(false)),
        });
        let called = Arc::clone(&hook.interception_called);
        
        let adapter = ErrorHookAdapter::interception(hook);
        assert_eq!(adapter.name(), "test");
        assert_eq!(adapter.priority().value(), HookPriority::default().value());
        assert_eq!(adapter.hook_type(), HookType::ErrorInterception);

        let ctx = ErrorHookContext::interception("test".to_string(), "Test".to_string(), None);
        let hook_ctx = ctx.to_hook_context(ErrorHookType::Interception);
        let result = adapter.execute(&hook_ctx).await;
        assert!(result.is_ok());
        assert!(*called.lock().await);
    }

    #[tokio::test]
    async fn test_error_hook_adapter_transformation() {
        let hook = Arc::new(MockErrorHook {
            name: "test".to_string(),
            priority: HookPriority::default(),
            interception_called: Arc::new(tokio::sync::Mutex::new(false)),
            transformation_called: Arc::new(tokio::sync::Mutex::new(false)),
            recovery_called: Arc::new(tokio::sync::Mutex::new(false)),
            logging_called: Arc::new(tokio::sync::Mutex::new(false)),
        });
        let called = Arc::clone(&hook.transformation_called);
        
        let adapter = ErrorHookAdapter::transformation(hook);
        assert_eq!(adapter.hook_type(), HookType::ErrorTransformation);

        let ctx = ErrorHookContext::transformation("test".to_string(), "Test".to_string(), None);
        let hook_ctx = ctx.to_hook_context(ErrorHookType::Transformation);
        let result = adapter.execute(&hook_ctx).await;
        assert!(result.is_ok());
        assert!(*called.lock().await);
    }

    #[tokio::test]
    async fn test_error_hook_adapter_recovery() {
        let hook = Arc::new(MockErrorHook {
            name: "test".to_string(),
            priority: HookPriority::default(),
            interception_called: Arc::new(tokio::sync::Mutex::new(false)),
            transformation_called: Arc::new(tokio::sync::Mutex::new(false)),
            recovery_called: Arc::new(tokio::sync::Mutex::new(false)),
            logging_called: Arc::new(tokio::sync::Mutex::new(false)),
        });
        let called = Arc::clone(&hook.recovery_called);
        
        let adapter = ErrorHookAdapter::recovery(hook);
        assert_eq!(adapter.hook_type(), HookType::ErrorRecovery);

        let ctx = ErrorHookContext::recovery("test".to_string(), "Test".to_string(), None);
        let hook_ctx = ctx.to_hook_context(ErrorHookType::Recovery);
        let result = adapter.execute(&hook_ctx).await;
        assert!(result.is_ok());
        assert!(*called.lock().await);
    }

    #[tokio::test]
    async fn test_error_hook_adapter_logging() {
        let hook = Arc::new(MockErrorHook {
            name: "test".to_string(),
            priority: HookPriority::default(),
            interception_called: Arc::new(tokio::sync::Mutex::new(false)),
            transformation_called: Arc::new(tokio::sync::Mutex::new(false)),
            recovery_called: Arc::new(tokio::sync::Mutex::new(false)),
            logging_called: Arc::new(tokio::sync::Mutex::new(false)),
        });
        let called = Arc::clone(&hook.logging_called);
        
        let adapter = ErrorHookAdapter::logging(hook);
        assert_eq!(adapter.hook_type(), HookType::ErrorLogging);

        let ctx = ErrorHookContext::logging("test".to_string(), "Test".to_string(), None);
        let hook_ctx = ctx.to_hook_context(ErrorHookType::Logging);
        let result = adapter.execute(&hook_ctx).await;
        assert!(result.is_ok());
        assert!(*called.lock().await);
    }

    #[tokio::test]
    async fn test_error_hook_adapter_serialization_error() {
        let hook = Arc::new(MockErrorHook {
            name: "test".to_string(),
            priority: HookPriority::default(),
            interception_called: Arc::new(tokio::sync::Mutex::new(false)),
            transformation_called: Arc::new(tokio::sync::Mutex::new(false)),
            recovery_called: Arc::new(tokio::sync::Mutex::new(false)),
            logging_called: Arc::new(tokio::sync::Mutex::new(false)),
        });
        
        let adapter = ErrorHookAdapter::interception(hook);
        let invalid_ctx = HookContext::new("error_interception", serde_json::Value::String("invalid".to_string()));
        let result = adapter.execute(&invalid_ctx).await;
        assert!(result.is_err());
    }
}
