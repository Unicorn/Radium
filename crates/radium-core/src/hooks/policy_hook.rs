//! Policy hook for tool execution approval.
//!
//! This hook integrates the policy engine with the hooks system to gate
//! file-mutating operations through approval checks.

use crate::hooks::error::Result;
use crate::hooks::registry::{Hook, HookRegistry, HookType};
use crate::hooks::types::{HookContext, HookPriority, HookResult as HookExecutionResult};
use crate::policy::{PolicyAction, PolicyEngine, PolicyError};
use async_trait::async_trait;
use std::sync::Arc;

/// Hook that checks policy before tool execution.
pub struct PolicyHook {
    /// Policy engine for making decisions.
    policy_engine: Arc<PolicyEngine>,
    /// Name of the hook.
    name: String,
}

impl PolicyHook {
    /// Create a new policy hook.
    pub fn new(policy_engine: Arc<PolicyEngine>) -> Self {
        Self {
            policy_engine,
            name: "policy_check".to_string(),
        }
    }

    /// Check if a tool is a file-mutating operation.
    fn is_file_mutating_operation(tool_name: &str) -> bool {
        matches!(
            tool_name,
            "write_file"
                | "delete_file"
                | "create_file"
                | "rename_path"
                | "apply_patch"
                | "search_replace"
                | "create_dir"
        )
    }

    /// Extract file path from tool arguments.
    fn extract_file_path(args: &serde_json::Value) -> Option<String> {
        // Try common argument names for file paths
        if let Some(path) = args.get("file_path").and_then(|v| v.as_str()) {
            return Some(path.to_string());
        }
        if let Some(path) = args.get("path").and_then(|v| v.as_str()) {
            return Some(path.to_string());
        }
        if let Some(path) = args.get("old_path").and_then(|v| v.as_str()) {
            return Some(path.to_string());
        }
        // For apply_patch, check patch content
        if let Some(patch) = args.get("patch") {
            if let Some(content) = patch.get("content").and_then(|v| v.as_str()) {
                // Try to extract file path from unified diff
                for line in content.lines() {
                    if line.starts_with("--- a/") {
                        return Some(line.strip_prefix("--- a/")?.to_string());
                    }
                    if line.starts_with("+++ b/") {
                        return Some(line.strip_prefix("+++ b/")?.to_string());
                    }
                }
            }
        }
        None
    }
}

#[async_trait]
impl Hook for PolicyHook {
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> HookPriority {
        HookPriority::new(200) // High priority to run early
    }

    fn hook_type(&self) -> HookType {
        HookType::BeforeTool
    }

    async fn execute(&self, context: &HookContext) -> Result<HookExecutionResult> {
        // HookContext from ToolHookContext is serialized as JSON
        // Try to deserialize as ToolHookContext first
        let tool_name = if let Ok(tool_ctx) = serde_json::from_value::<crate::hooks::tool::ToolHookContext>(context.data.clone()) {
            tool_ctx.tool_name
        } else {
            // Fallback: try to extract directly from data
            context.data
                .get("tool_name")
                .and_then(|v| v.as_str())
                .map(str::to_string)
                .unwrap_or_default()
        };

        // Only check file-mutating operations
        if tool_name.is_empty() || !Self::is_file_mutating_operation(&tool_name) {
            return Ok(HookExecutionResult::success());
        }

        // Extract arguments
        let arguments = if let Ok(tool_ctx) = serde_json::from_value::<crate::hooks::tool::ToolHookContext>(context.data.clone()) {
            tool_ctx.arguments
        } else {
            context.data
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}))
        };

        // Extract file path for better error messages
        let file_path = Self::extract_file_path(&arguments);
        let args_vec: Vec<&str> = arguments
            .as_object()
            .map(|obj| {
                obj.values()
                    .filter_map(|v| v.as_str())
                    .collect()
            })
            .unwrap_or_default();

        // Evaluate policy
        let decision = match self.policy_engine.evaluate_tool(&tool_name, &args_vec).await {
            Ok(d) => d,
            Err(PolicyError::LoadError { .. } | PolicyError::ParseError { .. }) => {
                // Policy file errors - allow execution but log warning
                tracing::warn!("Policy evaluation failed, allowing execution: {}", tool_name);
                return Ok(HookExecutionResult::success());
            }
            Err(e) => {
                // Other policy errors - deny for safety
                tracing::error!("Policy evaluation error: {}", e);
                return Ok(HookExecutionResult::error(format!(
                    "Policy check failed: {}. Execution denied for safety.",
                    e
                )));
            }
        };

        match decision.action {
            PolicyAction::Allow => {
                // Policy allows execution
                Ok(HookExecutionResult::success())
            }
            PolicyAction::Deny => {
                // Policy denies execution
                let reason = decision.reason.unwrap_or_else(|| {
                    format!("Policy rule denied execution of {}", tool_name)
                });
                Ok(HookExecutionResult::error(reason))
            }
            PolicyAction::AskUser => {
                // Policy requires user approval
                let reason = decision.reason.unwrap_or_else(|| {
                    format!(
                        "User approval required for {} operation{}",
                        tool_name,
                        file_path
                            .as_ref()
                            .map(|p| format!(" on {}", p))
                            .unwrap_or_default()
                    )
                });

                // Return error to abort execution - the orchestrator should handle
                // this by emitting ApprovalRequired event and waiting for user response
                Ok(HookExecutionResult::error(format!(
                    "APPROVAL_REQUIRED: {}",
                    reason
                )))
            }
            PolicyAction::DryRunFirst => {
                // Policy requires dry-run preview first
                let preview_info = decision.preview.map(|p| {
                    format!(
                        "Dry-run preview: {} would affect: {}",
                        p.tool_name,
                        p.affected_resources.join(", ")
                    )
                }).unwrap_or_else(|| "Dry-run required".to_string());

                Ok(HookExecutionResult::error(format!(
                    "DRY_RUN_REQUIRED: {}",
                    preview_info
                )))
            }
        }
    }
}

/// Helper to register policy hook in a hook registry.
pub struct PolicyHookRegistrar;

impl PolicyHookRegistrar {
    /// Register a policy hook in the given registry.
    pub async fn register_policy_hook(
        registry: &HookRegistry,
        policy_engine: Arc<PolicyEngine>,
    ) -> Result<()> {
        let hook = Arc::new(PolicyHook::new(policy_engine));
        registry.register(hook).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::types::ApprovalMode;

    #[tokio::test]
    async fn test_policy_hook_allows_non_mutating_operations() {
        let engine = Arc::new(PolicyEngine::new(ApprovalMode::Ask).unwrap());
        let hook = PolicyHook::new(engine);

        let context = HookContext::new(serde_json::json!({
            "tool_name": "read_file",
            "arguments": {"file_path": "test.txt"}
        }));

        let result = hook.execute(&context).await.unwrap();
        assert!(result.should_continue);
    }

    #[tokio::test]
    async fn test_policy_hook_checks_file_mutating_operations() {
        let engine = Arc::new(PolicyEngine::new(ApprovalMode::Ask).unwrap());
        let hook = PolicyHook::new(engine);

        let context = HookContext::new(serde_json::json!({
            "tool_name": "write_file",
            "arguments": {"file_path": "test.txt", "content": "test"}
        }));

        let result = hook.execute(&context).await.unwrap();
        // In Ask mode, should require approval (returns error with APPROVAL_REQUIRED)
        assert!(!result.should_continue || result.message.as_ref().map(|m| m.contains("APPROVAL_REQUIRED")).unwrap_or(false));
    }
}
