//! Tool execution hooks.

use crate::hooks::error::Result;
use crate::hooks::registry::HookType;
use crate::hooks::types::{HookContext, HookPriority, HookResult as HookExecutionResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Type of tool hook.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolHookType {
    /// Before tool execution.
    Before,
    /// After tool execution.
    After,
    /// Tool selection.
    Selection,
}

/// Context for tool execution hooks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolHookContext {
    /// The tool name.
    pub tool_name: String,
    /// The tool arguments.
    pub arguments: serde_json::Value,
    /// Optional tool result (for after hooks).
    pub result: Option<serde_json::Value>,
    /// Optional modified arguments.
    pub modified_arguments: Option<serde_json::Value>,
    /// Optional modified result.
    pub modified_result: Option<serde_json::Value>,
}

impl ToolHookContext {
    /// Create a new tool hook context for before tool execution.
    pub fn before(tool_name: String, arguments: serde_json::Value) -> Self {
        Self { tool_name, arguments, result: None, modified_arguments: None, modified_result: None }
    }

    /// Create a new tool hook context for after tool execution.
    pub fn after(
        tool_name: String,
        arguments: serde_json::Value,
        result: serde_json::Value,
    ) -> Self {
        Self {
            tool_name,
            arguments,
            result: Some(result),
            modified_arguments: None,
            modified_result: None,
        }
    }

    /// Create a new tool hook context for tool selection.
    pub fn selection(tool_name: String, arguments: serde_json::Value) -> Self {
        Self { tool_name, arguments, result: None, modified_arguments: None, modified_result: None }
    }

    /// Convert to hook context.
    pub fn to_hook_context(&self, hook_type: ToolHookType) -> HookContext {
        let hook_type_str = match hook_type {
            ToolHookType::Before => "before_tool",
            ToolHookType::After => "after_tool",
            ToolHookType::Selection => "tool_selection",
        };

        HookContext::new(
            hook_type_str,
            serde_json::to_value(self).unwrap_or(serde_json::Value::Null),
        )
    }
}

/// Trait for tool hooks.
#[async_trait]
pub trait ToolHook: Send + Sync {
    /// Get the name of the hook.
    fn name(&self) -> &str;

    /// Get the priority of the hook.
    fn priority(&self) -> HookPriority;

    /// Execute before tool execution.
    async fn before_tool_execution(&self, context: &ToolHookContext)
    -> Result<HookExecutionResult>;

    /// Execute after tool execution.
    async fn after_tool_execution(&self, context: &ToolHookContext) -> Result<HookExecutionResult>;

    /// Execute for tool selection.
    async fn tool_selection(&self, context: &ToolHookContext) -> Result<HookExecutionResult>;
}

/// Adapter to convert ToolHook to Hook trait.
pub struct ToolHookAdapter {
    hook: Arc<dyn ToolHook>,
    hook_type: ToolHookType,
}

impl ToolHookAdapter {
    /// Create a new adapter for before tool execution.
    pub fn before(hook: Arc<dyn ToolHook>) -> Arc<dyn crate::hooks::registry::Hook> {
        Arc::new(Self { hook, hook_type: ToolHookType::Before })
    }

    /// Create a new adapter for after tool execution.
    pub fn after(hook: Arc<dyn ToolHook>) -> Arc<dyn crate::hooks::registry::Hook> {
        Arc::new(Self { hook, hook_type: ToolHookType::After })
    }

    /// Create a new adapter for tool selection.
    pub fn selection(hook: Arc<dyn ToolHook>) -> Arc<dyn crate::hooks::registry::Hook> {
        Arc::new(Self { hook, hook_type: ToolHookType::Selection })
    }
}

#[async_trait]
impl crate::hooks::registry::Hook for ToolHookAdapter {
    fn name(&self) -> &str {
        self.hook.name()
    }

    fn priority(&self) -> HookPriority {
        self.hook.priority()
    }

    fn hook_type(&self) -> HookType {
        match self.hook_type {
            ToolHookType::Before => HookType::BeforeTool,
            ToolHookType::After => HookType::AfterTool,
            ToolHookType::Selection => HookType::ToolSelection,
        }
    }

    async fn execute(&self, context: &HookContext) -> Result<HookExecutionResult> {
        let tool_context: ToolHookContext = serde_json::from_value(context.data.clone())
            .map_err(|e| crate::hooks::error::HookError::Serialization(e))?;

        match self.hook_type {
            ToolHookType::Before => self.hook.before_tool_execution(&tool_context).await,
            ToolHookType::After => self.hook.after_tool_execution(&tool_context).await,
            ToolHookType::Selection => self.hook.tool_selection(&tool_context).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hooks::types::HookResult as HookExecutionResult;

    struct MockToolHook {
        name: String,
        priority: HookPriority,
        before_called: Arc<tokio::sync::Mutex<bool>>,
        after_called: Arc<tokio::sync::Mutex<bool>>,
        selection_called: Arc<tokio::sync::Mutex<bool>>,
    }

    #[async_trait]
    impl ToolHook for MockToolHook {
        fn name(&self) -> &str {
            &self.name
        }

        fn priority(&self) -> HookPriority {
            self.priority
        }

        async fn before_tool_execution(&self, _context: &ToolHookContext) -> Result<HookExecutionResult> {
            *self.before_called.lock().await = true;
            Ok(HookExecutionResult::success())
        }

        async fn after_tool_execution(&self, _context: &ToolHookContext) -> Result<HookExecutionResult> {
            *self.after_called.lock().await = true;
            Ok(HookExecutionResult::success())
        }

        async fn tool_selection(&self, _context: &ToolHookContext) -> Result<HookExecutionResult> {
            *self.selection_called.lock().await = true;
            Ok(HookExecutionResult::success())
        }
    }

    #[test]
    fn test_tool_hook_type_variants() {
        assert_eq!(ToolHookType::Before, ToolHookType::Before);
        assert_eq!(ToolHookType::After, ToolHookType::After);
        assert_eq!(ToolHookType::Selection, ToolHookType::Selection);
    }

    #[test]
    fn test_tool_hook_context_before() {
        let ctx = ToolHookContext::before(
            "test-tool".to_string(),
            serde_json::json!({"arg": "value"}),
        );
        assert_eq!(ctx.tool_name, "test-tool");
        assert!(ctx.result.is_none());
        assert!(ctx.modified_arguments.is_none());
    }

    #[test]
    fn test_tool_hook_context_after() {
        let ctx = ToolHookContext::after(
            "test-tool".to_string(),
            serde_json::json!({"arg": "value"}),
            serde_json::json!({"result": "success"}),
        );
        assert_eq!(ctx.tool_name, "test-tool");
        assert!(ctx.result.is_some());
    }

    #[test]
    fn test_tool_hook_context_selection() {
        let ctx = ToolHookContext::selection(
            "test-tool".to_string(),
            serde_json::json!({"arg": "value"}),
        );
        assert_eq!(ctx.tool_name, "test-tool");
        assert!(ctx.result.is_none());
    }

    #[test]
    fn test_tool_hook_context_to_hook_context() {
        let ctx = ToolHookContext::before("test".to_string(), serde_json::json!({}));
        let hook_ctx = ctx.to_hook_context(ToolHookType::Before);
        assert_eq!(hook_ctx.hook_type, "before_tool");
    }

    #[tokio::test]
    async fn test_tool_hook_adapter_before() {
        let hook = Arc::new(MockToolHook {
            name: "test".to_string(),
            priority: HookPriority::default(),
            before_called: Arc::new(tokio::sync::Mutex::new(false)),
            after_called: Arc::new(tokio::sync::Mutex::new(false)),
            selection_called: Arc::new(tokio::sync::Mutex::new(false)),
        });
        let called = Arc::clone(&hook.before_called);
        
        let adapter = ToolHookAdapter::before(hook);
        assert_eq!(adapter.name(), "test");
        assert_eq!(adapter.priority().value(), HookPriority::default().value());
        assert_eq!(adapter.hook_type(), HookType::BeforeTool);

        let ctx = ToolHookContext::before("test".to_string(), serde_json::json!({}));
        let hook_ctx = ctx.to_hook_context(ToolHookType::Before);
        let result = adapter.execute(&hook_ctx).await;
        assert!(result.is_ok());
        assert!(*called.lock().await);
    }

    #[tokio::test]
    async fn test_tool_hook_adapter_after() {
        let hook = Arc::new(MockToolHook {
            name: "test".to_string(),
            priority: HookPriority::default(),
            before_called: Arc::new(tokio::sync::Mutex::new(false)),
            after_called: Arc::new(tokio::sync::Mutex::new(false)),
            selection_called: Arc::new(tokio::sync::Mutex::new(false)),
        });
        let called = Arc::clone(&hook.after_called);
        
        let adapter = ToolHookAdapter::after(hook);
        assert_eq!(adapter.hook_type(), HookType::AfterTool);

        let ctx = ToolHookContext::after("test".to_string(), serde_json::json!({}), serde_json::json!({}));
        let hook_ctx = ctx.to_hook_context(ToolHookType::After);
        let result = adapter.execute(&hook_ctx).await;
        assert!(result.is_ok());
        assert!(*called.lock().await);
    }

    #[tokio::test]
    async fn test_tool_hook_adapter_selection() {
        let hook = Arc::new(MockToolHook {
            name: "test".to_string(),
            priority: HookPriority::default(),
            before_called: Arc::new(tokio::sync::Mutex::new(false)),
            after_called: Arc::new(tokio::sync::Mutex::new(false)),
            selection_called: Arc::new(tokio::sync::Mutex::new(false)),
        });
        let called = Arc::clone(&hook.selection_called);
        
        let adapter = ToolHookAdapter::selection(hook);
        assert_eq!(adapter.hook_type(), HookType::ToolSelection);

        let ctx = ToolHookContext::selection("test".to_string(), serde_json::json!({}));
        let hook_ctx = ctx.to_hook_context(ToolHookType::Selection);
        let result = adapter.execute(&hook_ctx).await;
        assert!(result.is_ok());
        assert!(*called.lock().await);
    }

    #[tokio::test]
    async fn test_tool_hook_adapter_serialization_error() {
        let hook = Arc::new(MockToolHook {
            name: "test".to_string(),
            priority: HookPriority::default(),
            before_called: Arc::new(tokio::sync::Mutex::new(false)),
            after_called: Arc::new(tokio::sync::Mutex::new(false)),
            selection_called: Arc::new(tokio::sync::Mutex::new(false)),
        });
        
        let adapter = ToolHookAdapter::before(hook);
        let invalid_ctx = HookContext::new("before_tool", serde_json::Value::String("invalid".to_string()));
        let result = adapter.execute(&invalid_ctx).await;
        assert!(result.is_err());
    }
}
