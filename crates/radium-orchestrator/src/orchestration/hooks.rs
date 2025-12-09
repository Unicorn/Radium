//! Hook executor for orchestration tool execution
//!
//! This module provides a trait-based interface for executing hooks
//! (BeforeTool/AfterTool) to avoid circular dependencies with radium-core.

use serde_json::Value;

/// Result of hook execution
#[derive(Debug, Clone)]
pub struct ToolHookResult {
    /// Whether execution should continue
    pub should_continue: bool,
    /// Optional message from the hook
    pub message: Option<String>,
    /// Modified data from the hook (e.g., modified tool arguments or results)
    pub modified_data: Option<Value>,
}

/// Trait for executing tool hooks to avoid direct dependency on radium-core
#[async_trait::async_trait]
pub trait ToolHookExecutor: Send + Sync {
    /// Execute before tool execution hooks
    ///
    /// # Arguments
    /// * `tool_name` - Name of the tool being executed
    /// * `arguments` - Tool arguments as JSON
    ///
    /// # Returns
    /// Modified arguments (if hooks modified them) or original arguments
    /// Returns error if hooks request to abort execution
    async fn before_tool_execution(
        &self,
        tool_name: &str,
        arguments: &Value,
    ) -> std::result::Result<Value, String>;

    /// Execute after tool execution hooks
    ///
    /// # Arguments
    /// * `tool_name` - Name of the tool that was executed
    /// * `arguments` - Tool arguments that were used
    /// * `result` - Tool execution result as JSON
    ///
    /// # Returns
    /// Modified result (if hooks modified it) or original result
    async fn after_tool_execution(
        &self,
        tool_name: &str,
        arguments: &Value,
        result: &Value,
    ) -> std::result::Result<Value, String>;
}

/// Simple implementation that does nothing (no-op)
///
/// This can be used when hooks are not available or not needed.
pub struct NoOpToolHookExecutor;

#[async_trait::async_trait]
impl ToolHookExecutor for NoOpToolHookExecutor {
    async fn before_tool_execution(
        &self,
        _tool_name: &str,
        arguments: &Value,
    ) -> std::result::Result<Value, String> {
        Ok(arguments.clone())
    }

    async fn after_tool_execution(
        &self,
        _tool_name: &str,
        _arguments: &Value,
        result: &Value,
    ) -> std::result::Result<Value, String> {
        Ok(result.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_no_op_hook_executor() {
        let executor = NoOpToolHookExecutor;
        let args = serde_json::json!({"key": "value"});
        let result = serde_json::json!({"output": "test"});

        let before_result = executor.before_tool_execution("test_tool", &args).await.unwrap();
        assert_eq!(before_result, args);

        let after_result = executor.after_tool_execution("test_tool", &args, &result).await.unwrap();
        assert_eq!(after_result, result);
    }
}

