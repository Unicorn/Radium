//! Example audit hook implementation.
//!
//! This hook logs all model calls and tool executions to an audit log file for compliance.

use async_trait::async_trait;
use radium_core::hooks::model::{ModelHook, ModelHookContext};
use radium_core::hooks::tool::{ToolHook, ToolHookContext};
use radium_core::hooks::types::{HookPriority, HookResult as HookExecutionResult};
use radium_core::hooks::error::Result;
use serde_json::json;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Audit hook that logs all model calls and tool executions.
pub struct AuditHook {
    name: String,
    priority: HookPriority,
    log_file: Arc<RwLock<PathBuf>>,
}

impl AuditHook {
    /// Create a new audit hook.
    pub fn new(name: impl Into<String>, priority: u32, log_file: impl Into<PathBuf>) -> Self {
        Self {
            name: name.into(),
            priority: HookPriority::new(priority),
            log_file: Arc::new(RwLock::new(log_file.into())),
        }
    }

    /// Write audit entry to log file.
    async fn write_audit_entry(&self, entry: &serde_json::Value) -> Result<()> {
        let log_file = self.log_file.read().await;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&*log_file)
            .map_err(|e| radium_core::hooks::error::HookError::Io(e))?;

        let entry_str = serde_json::to_string(entry)
            .map_err(|e| radium_core::hooks::error::HookError::Serialization(e))?;
        writeln!(file, "{}", entry_str)
            .map_err(|e| radium_core::hooks::error::HookError::Io(e))?;

        Ok(())
    }
}

#[async_trait]
impl ModelHook for AuditHook {
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> HookPriority {
        self.priority
    }

    async fn before_model_call(&self, context: &ModelHookContext) -> Result<HookExecutionResult> {
        let entry = json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "event": "before_model_call",
            "hook": self.name,
            "model_id": context.model_id,
            "input_length": context.input.len(),
            "input_preview": context.input.chars().take(100).collect::<String>(),
        });

        self.write_audit_entry(&entry).await?;
        info!(hook = %self.name, model = %context.model_id, "Audit log: before model call");

        Ok(HookExecutionResult::success())
    }

    async fn after_model_call(&self, context: &ModelHookContext) -> Result<HookExecutionResult> {
        let entry = json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "event": "after_model_call",
            "hook": self.name,
            "model_id": context.model_id,
            "input_length": context.input.len(),
            "response_length": context.response.as_ref().map(|r| r.len()).unwrap_or(0),
            "response_preview": context.response.as_ref()
                .map(|r| r.chars().take(100).collect::<String>())
                .unwrap_or_default(),
        });

        self.write_audit_entry(&entry).await?;
        info!(hook = %self.name, model = %context.model_id, "Audit log: after model call");

        Ok(HookExecutionResult::success())
    }
}

#[async_trait]
impl ToolHook for AuditHook {
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> HookPriority {
        self.priority
    }

    async fn before_tool_execution(&self, context: &ToolHookContext) -> Result<HookExecutionResult> {
        let entry = json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "event": "before_tool_execution",
            "hook": self.name,
            "tool_name": context.tool_name,
            "arguments": context.arguments,
        });

        self.write_audit_entry(&entry).await?;
        info!(hook = %self.name, tool = %context.tool_name, "Audit log: before tool execution");

        Ok(HookExecutionResult::success())
    }

    async fn after_tool_execution(&self, context: &ToolHookContext) -> Result<HookExecutionResult> {
        let entry = json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "event": "after_tool_execution",
            "hook": self.name,
            "tool_name": context.tool_name,
            "arguments": context.arguments,
            "result": context.result,
            "success": context.result.is_some(),
        });

        self.write_audit_entry(&entry).await?;
        info!(hook = %self.name, tool = %context.tool_name, "Audit log: after tool execution");

        Ok(HookExecutionResult::success())
    }

    async fn tool_selection(&self, _context: &ToolHookContext) -> Result<HookExecutionResult> {
        Ok(HookExecutionResult::success())
    }
}

/// Create audit hooks (model and tool).
pub fn create_audit_hooks(log_file: impl Into<PathBuf>) -> (
    std::sync::Arc<dyn radium_core::hooks::registry::Hook>,
    std::sync::Arc<dyn radium_core::hooks::registry::Hook>,
    std::sync::Arc<dyn radium_core::hooks::registry::Hook>,
    std::sync::Arc<dyn radium_core::hooks::registry::Hook>,
) {
    let hook = std::sync::Arc::new(AuditHook::new("audit-hook", 100, log_file));
    
    (
        radium_core::hooks::model::ModelHookAdapter::before(hook.clone()),
        radium_core::hooks::model::ModelHookAdapter::after(hook.clone()),
        radium_core::hooks::tool::ToolHookAdapter::before(hook.clone()),
        radium_core::hooks::tool::ToolHookAdapter::after(hook),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_audit_hook_model_call() {
        let temp_file = NamedTempFile::new().unwrap();
        let hook = AuditHook::new("test-audit", 100, temp_file.path());
        let context = ModelHookContext::before(
            "test input".to_string(),
            "test-model".to_string(),
        );

        let result = hook.before_model_call(&context).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_audit_hook_tool_execution() {
        let temp_file = NamedTempFile::new().unwrap();
        let hook = AuditHook::new("test-audit", 100, temp_file.path());
        let context = ToolHookContext::before(
            "read_file".to_string(),
            json!({"path": "test.txt"}),
        );

        let result = hook.before_tool_execution(&context).await.unwrap();
        assert!(result.success);
    }
}

