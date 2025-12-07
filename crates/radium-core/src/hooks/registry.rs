//! Hook registry for managing and executing hooks.

use crate::hooks::error::Result;
use crate::hooks::types::{HookContext, HookPriority, HookResult as HookExecutionResult};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Type of hook.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum HookType {
    /// Before model call hook.
    BeforeModel,
    /// After model call hook.
    AfterModel,
    /// Before tool execution hook.
    BeforeTool,
    /// After tool execution hook.
    AfterTool,
    /// Tool selection hook.
    ToolSelection,
    /// Error interception hook.
    ErrorInterception,
    /// Error transformation hook.
    ErrorTransformation,
    /// Error recovery hook.
    ErrorRecovery,
    /// Error logging hook.
    ErrorLogging,
    /// Telemetry collection hook.
    TelemetryCollection,
    /// Custom logging hook.
    CustomLogging,
    /// Metrics aggregation hook.
    MetricsAggregation,
    /// Performance monitoring hook.
    PerformanceMonitoring,
}

impl HookType {
    /// Get the string representation of the hook type.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BeforeModel => "before_model",
            Self::AfterModel => "after_model",
            Self::BeforeTool => "before_tool",
            Self::AfterTool => "after_tool",
            Self::ToolSelection => "tool_selection",
            Self::ErrorInterception => "error_interception",
            Self::ErrorTransformation => "error_transformation",
            Self::ErrorRecovery => "error_recovery",
            Self::ErrorLogging => "error_logging",
            Self::TelemetryCollection => "telemetry_collection",
            Self::CustomLogging => "custom_logging",
            Self::MetricsAggregation => "metrics_aggregation",
            Self::PerformanceMonitoring => "performance_monitoring",
        }
    }
}

/// Trait for hook implementations.
#[async_trait]
pub trait Hook: Send + Sync {
    /// Get the name of the hook.
    fn name(&self) -> &str;

    /// Get the priority of the hook.
    fn priority(&self) -> HookPriority;

    /// Get the hook type.
    fn hook_type(&self) -> HookType;

    /// Execute the hook with the given context.
    async fn execute(&self, context: &HookContext) -> Result<HookExecutionResult>;
}

/// Registry for managing hooks.
pub struct HookRegistry {
    /// Registered hooks, organized by type.
    hooks: Arc<RwLock<Vec<Arc<dyn Hook>>>>,
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for HookRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HookRegistry").finish_non_exhaustive()
    }
}

impl HookRegistry {
    /// Create a new hook registry.
    pub fn new() -> Self {
        Self {
            hooks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Clone the registry (creates a new registry with shared hook storage).
    pub fn clone(&self) -> Self {
        Self {
            hooks: Arc::clone(&self.hooks),
        }
    }

    /// Register a hook.
    pub async fn register(&self, hook: Arc<dyn Hook>) -> Result<()> {
        let mut hooks = self.hooks.write().await;
        hooks.push(hook);
        // Sort by priority (higher priority first)
        hooks.sort_by(|a, b| b.priority().cmp(&a.priority()));
        Ok(())
    }

    /// Unregister a hook by name.
    pub async fn unregister(&self, name: &str) -> Result<()> {
        let mut hooks = self.hooks.write().await;
        hooks.retain(|h| h.name() != name);
        Ok(())
    }

    /// Get all hooks of a specific type.
    pub async fn get_hooks(&self, hook_type: HookType) -> Vec<Arc<dyn Hook>> {
        let hooks = self.hooks.read().await;
        hooks
            .iter()
            .filter(|h| h.hook_type() == hook_type)
            .cloned()
            .collect()
    }

    /// Execute all hooks of a specific type.
    pub async fn execute_hooks(
        &self,
        hook_type: HookType,
        context: &HookContext,
    ) -> Result<Vec<HookExecutionResult>> {
        let hooks = self.get_hooks(hook_type).await;
        let mut results = Vec::new();

        for hook in hooks {
            match hook.execute(context).await {
                Ok(result) => {
                    results.push(result.clone());
                    // If a hook says to stop, we stop executing remaining hooks
                    if !result.should_continue {
                        break;
                    }
                }
                Err(e) => {
                    // Log error but continue with other hooks
                    tracing::warn!(
                        hook_name = %hook.name(),
                        error = %e,
                        "Hook execution failed"
                    );
                    results.push(HookExecutionResult::error(format!(
                        "Hook {} failed: {}",
                        hook.name(),
                        e
                    )));
                }
            }
        }

        Ok(results)
    }

    /// Clear all hooks.
    pub async fn clear(&self) {
        let mut hooks = self.hooks.write().await;
        hooks.clear();
    }

    /// Get the number of registered hooks.
    pub async fn count(&self) -> usize {
        let hooks = self.hooks.read().await;
        hooks.len()
    }
}

