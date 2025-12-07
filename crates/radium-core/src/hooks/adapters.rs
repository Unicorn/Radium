//! Adapters for integrating existing systems with the hooks framework.
//!
//! This module provides adapters that allow existing systems (BehaviorEvaluator,
//! PolicyEngine, MonitoringService) to work with the unified Hook trait.

use crate::hooks::error::Result;
use crate::hooks::registry::{Hook, HookRegistry, HookType};
use crate::hooks::types::{HookContext, HookPriority, HookResult as HookExecutionResult};
use crate::workflow::behaviors::{BehaviorAction, BehaviorError, BehaviorEvaluator};
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;

/// Adapter that wraps a BehaviorEvaluator to work as a Hook.
///
/// This adapter allows existing behavior evaluators (loop, trigger, checkpoint, vibecheck)
/// to be registered as hooks while maintaining backward compatibility.
pub struct BehaviorEvaluatorAdapter<E: BehaviorEvaluator> {
    /// The behavior evaluator being adapted.
    evaluator: Arc<E>,
    /// The name of the behavior.
    name: String,
    /// The priority for hook execution.
    priority: HookPriority,
    /// The hook type this adapter represents.
    hook_type: HookType,
}

impl<E: BehaviorEvaluator> BehaviorEvaluatorAdapter<E> {
    /// Create a new adapter for a behavior evaluator.
    pub fn new(
        evaluator: Arc<E>,
        name: impl Into<String>,
        priority: HookPriority,
        hook_type: HookType,
    ) -> Self {
        Self { evaluator, name: name.into(), priority, hook_type }
    }

    /// Create an adapter for workflow step hooks.
    pub fn for_workflow_step(evaluator: Arc<E>, name: impl Into<String>) -> Arc<dyn Hook> {
        Arc::new(Self {
            evaluator,
            name: name.into(),
            priority: HookPriority::new(100),
            hook_type: HookType::PerformanceMonitoring, // Using as placeholder for workflow step
        })
    }
}

#[async_trait]
impl<E: BehaviorEvaluator + Send + Sync + 'static> Hook for BehaviorEvaluatorAdapter<E>
where
    E::Decision: Send + 'static,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> HookPriority {
        self.priority
    }

    fn hook_type(&self) -> HookType {
        self.hook_type
    }

    async fn execute(&self, context: &HookContext) -> Result<HookExecutionResult> {
        // Extract behavior file path and output from context
        let behavior_file =
            context.data.get("behavior_file").and_then(|v| v.as_str()).map(Path::new);

        let output = context.data.get("output").and_then(|v| v.as_str()).unwrap_or("");

        // If no behavior file, return success (no behavior to evaluate)
        let Some(behavior_path) = behavior_file else {
            return Ok(HookExecutionResult::success());
        };

        // Evaluate behavior using the evaluator
        // Note: BehaviorEvaluator::evaluate is synchronous, so we call it directly
        // Since we're already in an async context, we can call it synchronously
        match self.evaluator.evaluate(behavior_path, output, &()) {
            Ok(Some(_decision)) => {
                // Behavior was triggered - return modified result
                Ok(HookExecutionResult::with_data(serde_json::json!({
                    "behavior_triggered": true,
                    "behavior_name": self.name,
                })))
            }
            Ok(None) => {
                // No behavior - continue normally
                Ok(HookExecutionResult::success())
            }
            Err(e) => {
                // Evaluation error - log and continue
                tracing::warn!(
                    behavior_name = %self.name,
                    error = %e,
                    "Behavior evaluation failed"
                );
                Ok(HookExecutionResult::error(format!("Behavior evaluation failed: {}", e)))
            }
        }
    }
}

/// Helper to register behavior evaluators as hooks.
pub struct BehaviorHookRegistrar;

impl BehaviorHookRegistrar {
    /// Register a behavior evaluator as a hook.
    pub async fn register_behavior_hook<E: BehaviorEvaluator + Send + Sync + 'static>(
        registry: &HookRegistry,
        evaluator: Arc<E>,
        name: impl Into<String>,
        priority: HookPriority,
    ) -> Result<()>
    where
        E::Decision: Send + 'static,
    {
        registry: &HookRegistry,
        evaluator: Arc<E>,
        name: impl Into<String>,
        priority: HookPriority,
    ) -> Result<()> {
        let adapter = BehaviorEvaluatorAdapter::new(
            evaluator,
            name,
            priority,
            HookType::PerformanceMonitoring, // Placeholder - workflow step hooks would use a specific type
        );
        registry.register(Arc::new(adapter)).await
    }
}
