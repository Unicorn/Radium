//! Hook registry for managing and executing hooks.

use crate::hooks::error::Result;
use crate::hooks::profiler::HookProfiler;
use crate::hooks::types::{HookContext, HookPriority, HookResult as HookExecutionResult};
use async_trait::async_trait;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;
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
    /// Set of enabled hook names.
    enabled_hooks: Arc<RwLock<std::collections::HashSet<String>>>,
    /// Optional profiler for performance monitoring.
    profiler: Option<Arc<HookProfiler>>,
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
            enabled_hooks: Arc::new(RwLock::new(HashSet::new())),
            profiler: None,
        }
    }

    /// Create a new hook registry with profiling enabled.
    pub fn with_profiler(profiler: Arc<HookProfiler>) -> Self {
        Self {
            hooks: Arc::new(RwLock::new(Vec::new())),
            enabled_hooks: Arc::new(RwLock::new(HashSet::new())),
            profiler: Some(profiler),
        }
    }

    /// Set the profiler for this registry.
    pub fn set_profiler(&mut self, profiler: Arc<HookProfiler>) {
        self.profiler = Some(profiler);
    }

    /// Get the profiler if available.
    pub fn profiler(&self) -> Option<&Arc<HookProfiler>> {
        self.profiler.as_ref()
    }

    /// Clone the registry (creates a new registry with shared hook storage).
    pub fn clone(&self) -> Self {
        Self {
            hooks: Arc::clone(&self.hooks),
            enabled_hooks: Arc::clone(&self.enabled_hooks),
            profiler: self.profiler.clone(),
        }
    }

    /// Register a hook.
    pub async fn register(&self, hook: Arc<dyn Hook>) -> Result<()> {
        let hook_name = hook.name().to_string();
        let mut hooks = self.hooks.write().await;
        hooks.push(hook);
        // Sort by priority (higher priority first)
        hooks.sort_by(|a, b| b.priority().cmp(&a.priority()));
        
        // Enable by default
        let mut enabled = self.enabled_hooks.write().await;
        enabled.insert(hook_name);
        Ok(())
    }
    
    /// Set the enabled state of a hook.
    pub async fn set_enabled(&self, name: &str, enabled: bool) -> Result<()> {
        // Verify hook exists
        let hooks = self.hooks.read().await;
        if !hooks.iter().any(|h| h.name() == name) {
            return Err(crate::hooks::error::HookError::NotFound(name.to_string()));
        }
        drop(hooks);
        
        let mut enabled_hooks = self.enabled_hooks.write().await;
        if enabled {
            enabled_hooks.insert(name.to_string());
        } else {
            enabled_hooks.remove(name);
        }
        Ok(())
    }
    
    /// Check if a hook is enabled.
    pub async fn is_enabled(&self, name: &str) -> bool {
        let enabled_hooks = self.enabled_hooks.read().await;
        enabled_hooks.contains(name)
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
        hooks.iter().filter(|h| h.hook_type() == hook_type).cloned().collect()
    }

    /// Execute all hooks of a specific type.
    pub async fn execute_hooks(
        &self,
        hook_type: HookType,
        context: &HookContext,
    ) -> Result<Vec<HookExecutionResult>> {
        let hooks = self.get_hooks(hook_type).await;
        let enabled_hooks = self.enabled_hooks.read().await;
        let mut results = Vec::new();

        for hook in hooks {
            // Only execute enabled hooks
            if !enabled_hooks.contains(hook.name()) {
                continue;
            }
            
            // Record execution time if profiling is enabled
            let start_time = Instant::now();
            let hook_result = hook.execute(context).await;
            let duration = start_time.elapsed();
            
            // Record profiling data if profiler is available
            if let Some(profiler) = &self.profiler {
                profiler
                    .record_execution(hook.name(), hook_type, duration)
                    .await;
            }
            
            match hook_result {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hooks::types::HookContext;

    struct TestHook {
        name: String,
        priority: HookPriority,
        hook_type: HookType,
    }

    #[async_trait]
    impl Hook for TestHook {
        fn name(&self) -> &str {
            &self.name
        }

        fn priority(&self) -> HookPriority {
            self.priority
        }

        fn hook_type(&self) -> HookType {
            self.hook_type
        }

        async fn execute(&self, _context: &HookContext) -> Result<HookExecutionResult> {
            Ok(HookExecutionResult::success())
        }
    }

    #[tokio::test]
    async fn test_hook_registry_register() {
        let registry = HookRegistry::new();
        let hook = Arc::new(TestHook {
            name: "test-hook".to_string(),
            priority: HookPriority::default(),
            hook_type: HookType::BeforeModel,
        });

        registry.register(hook).await.unwrap();
        assert_eq!(registry.count().await, 1);
    }

    #[tokio::test]
    async fn test_hook_registry_unregister() {
        let registry = HookRegistry::new();
        let hook = Arc::new(TestHook {
            name: "test-hook".to_string(),
            priority: HookPriority::default(),
            hook_type: HookType::BeforeModel,
        });

        registry.register(hook).await.unwrap();
        assert_eq!(registry.count().await, 1);

        registry.unregister("test-hook").await.unwrap();
        assert_eq!(registry.count().await, 0);
    }

    #[tokio::test]
    async fn test_hook_registry_priority_order() {
        let registry = HookRegistry::new();

        let hook1 = Arc::new(TestHook {
            name: "low-priority".to_string(),
            priority: HookPriority::new(50),
            hook_type: HookType::BeforeModel,
        });

        let hook2 = Arc::new(TestHook {
            name: "high-priority".to_string(),
            priority: HookPriority::new(150),
            hook_type: HookType::BeforeModel,
        });

        registry.register(hook1.clone()).await.unwrap();
        registry.register(hook2.clone()).await.unwrap();

        let hooks = registry.get_hooks(HookType::BeforeModel).await;
        assert_eq!(hooks.len(), 2);
        // Higher priority should be first
        assert_eq!(hooks[0].name(), "high-priority");
        assert_eq!(hooks[1].name(), "low-priority");
    }

    #[tokio::test]
    async fn test_hook_registry_execute_hooks() {
        let registry = HookRegistry::new();
        let hook = Arc::new(TestHook {
            name: "test-hook".to_string(),
            priority: HookPriority::default(),
            hook_type: HookType::BeforeModel,
        });

        registry.register(hook).await.unwrap();

        let context = HookContext::new("before_model", serde_json::json!({}));
        let results = registry.execute_hooks(HookType::BeforeModel, &context).await.unwrap();

        assert_eq!(results.len(), 1);
        assert!(results[0].success);
    }

    #[tokio::test]
    async fn test_hook_registry_filter_by_type() {
        let registry = HookRegistry::new();

        let model_hook = Arc::new(TestHook {
            name: "model-hook".to_string(),
            priority: HookPriority::default(),
            hook_type: HookType::BeforeModel,
        });

        let tool_hook = Arc::new(TestHook {
            name: "tool-hook".to_string(),
            priority: HookPriority::default(),
            hook_type: HookType::BeforeTool,
        });

        registry.register(model_hook).await.unwrap();
        registry.register(tool_hook).await.unwrap();

        let model_hooks = registry.get_hooks(HookType::BeforeModel).await;
        assert_eq!(model_hooks.len(), 1);
        assert_eq!(model_hooks[0].name(), "model-hook");

        let tool_hooks = registry.get_hooks(HookType::BeforeTool).await;
        assert_eq!(tool_hooks.len(), 1);
        assert_eq!(tool_hooks[0].name(), "tool-hook");
    }
}
