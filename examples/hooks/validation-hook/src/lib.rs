//! Example validation hook implementation.
//!
//! This hook validates tool arguments before execution, demonstrating security checks
//! and input validation patterns.

use async_trait::async_trait;
use radium_core::hooks::tool::{ToolHook, ToolHookContext};
use radium_core::hooks::types::{HookPriority, HookResult as HookExecutionResult};
use radium_core::hooks::error::Result;
use serde_json::json;
use tracing::{info, warn};

/// Validation hook that validates tool arguments before execution.
pub struct ValidationHook {
    name: String,
    priority: HookPriority,
}

impl ValidationHook {
    /// Create a new validation hook.
    pub fn new(name: impl Into<String>, priority: u32) -> Self {
        Self {
            name: name.into(),
            priority: HookPriority::new(priority),
        }
    }
}

#[async_trait]
impl ToolHook for ValidationHook {
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> HookPriority {
        self.priority
    }

    async fn before_tool_execution(&self, context: &ToolHookContext) -> Result<HookExecutionResult> {
        // Block dangerous tools
        let dangerous_tools = vec!["delete_file", "rm", "format_disk", "execute_shell"];
        if dangerous_tools.contains(&context.tool_name.as_str()) {
            warn!(
                hook = %self.name,
                tool = %context.tool_name,
                "Dangerous tool blocked"
            );
            return Ok(HookExecutionResult::stop("Dangerous tool blocked by validation hook"));
        }

        // Validate file operations
        if context.tool_name == "read_file" || context.tool_name == "write_file" {
            if let Some(path) = context.arguments.get("path").and_then(|v| v.as_str()) {
                // Check for path traversal
                if path.contains("..") {
                    warn!(
                        hook = %self.name,
                        tool = %context.tool_name,
                        path = %path,
                        "Path traversal detected"
                    );
                    return Ok(HookExecutionResult::stop("Invalid path: path traversal detected"));
                }

                // Check for absolute paths (if not allowed)
                if path.starts_with("/") && !path.starts_with("/tmp/") {
                    warn!(
                        hook = %self.name,
                        tool = %context.tool_name,
                        path = %path,
                        "Absolute path not allowed"
                    );
                    return Ok(HookExecutionResult::stop("Absolute paths not allowed"));
                }
            } else {
                warn!(
                    hook = %self.name,
                    tool = %context.tool_name,
                    "Missing required 'path' argument"
                );
                return Ok(HookExecutionResult::stop("Missing required 'path' argument"));
            }
        }

        // Validate shell command arguments
        if context.tool_name == "execute_command" || context.tool_name == "run_shell" {
            if let Some(command) = context.arguments.get("command").and_then(|v| v.as_str()) {
                // Block dangerous commands
                let dangerous_commands = vec!["rm -rf", "format", "dd if=", "mkfs"];
                for dangerous in dangerous_commands {
                    if command.contains(dangerous) {
                        warn!(
                            hook = %self.name,
                            tool = %context.tool_name,
                            command = %command,
                            "Dangerous command blocked"
                        );
                        return Ok(HookExecutionResult::stop(format!(
                            "Dangerous command blocked: {}",
                            dangerous
                        )));
                    }
                }
            }
        }

        info!(
            hook = %self.name,
            tool = %context.tool_name,
            "Tool validation passed"
        );

        Ok(HookExecutionResult::success())
    }

    async fn after_tool_execution(&self, context: &ToolHookContext) -> Result<HookExecutionResult> {
        // Log successful execution
        info!(
            hook = %self.name,
            tool = %context.tool_name,
            "Tool execution completed"
        );
        Ok(HookExecutionResult::success())
    }

    async fn tool_selection(&self, context: &ToolHookContext) -> Result<HookExecutionResult> {
        // Allow all tools (validation happens in before_tool_execution)
        Ok(HookExecutionResult::success())
    }
}

/// Create a validation hook.
pub fn create_validation_hook() -> std::sync::Arc<dyn radium_core::hooks::registry::Hook> {
    let hook = std::sync::Arc::new(ValidationHook::new("validation-hook", 200));
    radium_core::hooks::tool::ToolHookAdapter::before(hook)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validation_hook_dangerous_tool() {
        let hook = ValidationHook::new("test-validation", 200);
        let context = ToolHookContext::before(
            "delete_file".to_string(),
            json!({"path": "test.txt"}),
        );

        let result = hook.before_tool_execution(&context).await.unwrap();
        assert!(!result.should_continue);
    }

    #[tokio::test]
    async fn test_validation_hook_path_traversal() {
        let hook = ValidationHook::new("test-validation", 200);
        let context = ToolHookContext::before(
            "read_file".to_string(),
            json!({"path": "../../etc/passwd"}),
        );

        let result = hook.before_tool_execution(&context).await.unwrap();
        assert!(!result.should_continue);
    }

    #[tokio::test]
    async fn test_validation_hook_valid_path() {
        let hook = ValidationHook::new("test-validation", 200);
        let context = ToolHookContext::before(
            "read_file".to_string(),
            json!({"path": "src/main.rs"}),
        );

        let result = hook.before_tool_execution(&context).await.unwrap();
        assert!(result.should_continue);
    }
}

