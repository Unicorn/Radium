//! Hook composition and chaining framework.

use crate::hooks::error::Result;
use crate::hooks::registry::{Hook, HookType};
use crate::hooks::types::{HookContext, HookPriority, HookResult as HookExecutionResult};
use async_trait::async_trait;
use std::sync::Arc;

/// Composite hook that wraps multiple hooks as a single unit.
pub struct CompositeHook {
    name: String,
    priority: HookPriority,
    hook_type: HookType,
    hooks: Vec<Arc<dyn Hook>>,
}

impl CompositeHook {
    /// Create a new composite hook.
    pub fn new(name: String, priority: HookPriority, hook_type: HookType) -> Self {
        Self {
            name,
            priority,
            hook_type,
            hooks: Vec::new(),
        }
    }

    /// Add a hook to the composition.
    pub fn add_hook(mut self, hook: Arc<dyn Hook>) -> Self {
        self.hooks.push(hook);
        self
    }

    /// Add multiple hooks to the composition.
    pub fn add_hooks(mut self, hooks: Vec<Arc<dyn Hook>>) -> Self {
        self.hooks.extend(hooks);
        self
    }

    /// Build the composite hook.
    pub fn build(self) -> Arc<dyn Hook> {
        Arc::new(self)
    }
}

#[async_trait]
impl Hook for CompositeHook {
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
        let mut results = Vec::new();
        let mut last_result: Option<HookExecutionResult> = None;

        // Execute hooks in order
        for hook in &self.hooks {
            // Use modified data from previous hook if available
            let mut hook_context = context.clone();
            if let Some(ref prev_result) = last_result {
                if let Some(ref modified_data) = prev_result.modified_data {
                    // Merge modified data into context
                    if let Some(context_data) = hook_context.data.as_object_mut() {
                        if let Some(modified_obj) = modified_data.as_object() {
                            for (key, value) in modified_obj {
                                context_data.insert(key.clone(), value.clone());
                            }
                        }
                    }
                }
            }

            match hook.execute(&hook_context).await {
                Ok(result) => {
                    results.push(result.clone());
                    last_result = Some(result.clone());

                    // Stop if a hook says to stop
                    if !result.should_continue {
                        break;
                    }
                }
                Err(e) => {
                    // Log error but continue with remaining hooks
                    tracing::warn!(
                        hook_name = %hook.name(),
                        error = %e,
                        "Hook in composition failed"
                    );
                    let error_result = HookExecutionResult::error(format!(
                        "Hook {} failed: {}",
                        hook.name(),
                        e
                    ));
                    results.push(error_result.clone());
                    last_result = Some(error_result);
                }
            }
        }

        // Return aggregated result
        // Use the last result's data if available, otherwise create a summary
        if let Some(result) = last_result {
            Ok(result)
        } else {
            Ok(HookExecutionResult::success())
        }
    }
}

/// Hook chain for sequential execution with data passing.
pub struct HookChain {
    name: String,
    priority: HookPriority,
    hook_type: HookType,
    hooks: Vec<Arc<dyn Hook>>,
}

impl HookChain {
    /// Create a new hook chain.
    pub fn new(name: String, priority: HookPriority, hook_type: HookType) -> Self {
        Self {
            name,
            priority,
            hook_type,
            hooks: Vec::new(),
        }
    }

    /// Add a hook to the chain.
    pub fn link(mut self, hook: Arc<dyn Hook>) -> Self {
        self.hooks.push(hook);
        self
    }

    /// Build the hook chain.
    pub fn build(self) -> Arc<dyn Hook> {
        Arc::new(self)
    }
}

#[async_trait]
impl Hook for HookChain {
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
        let mut current_context = context.clone();
        let mut last_result: Option<HookExecutionResult> = None;

        // Execute hooks sequentially, passing data through
        for hook in &self.hooks {
            // Update context with modified data from previous hook
            if let Some(ref prev_result) = last_result {
                if let Some(ref modified_data) = prev_result.modified_data {
                    current_context.data = modified_data.clone();
                }
            }

            match hook.execute(&current_context).await {
                Ok(result) => {
                    last_result = Some(result.clone());

                    // Stop if a hook says to stop
                    if !result.should_continue {
                        break;
                    }

                    // Update context for next hook
                    if let Some(ref modified_data) = result.modified_data {
                        current_context.data = modified_data.clone();
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        hook_name = %hook.name(),
                        error = %e,
                        "Hook in chain failed"
                    );
                    return Err(e);
                }
            }
        }

        Ok(last_result.unwrap_or_else(|| HookExecutionResult::success()))
    }
}

/// Conditional hook that executes based on a predicate.
pub struct ConditionalHook {
    name: String,
    priority: HookPriority,
    hook_type: HookType,
    predicate: Arc<dyn Fn(&HookContext) -> bool + Send + Sync>,
    true_hook: Arc<dyn Hook>,
    false_hook: Option<Arc<dyn Hook>>,
}

impl ConditionalHook {
    /// Create a new conditional hook.
    pub fn new(
        name: String,
        priority: HookPriority,
        hook_type: HookType,
        predicate: impl Fn(&HookContext) -> bool + Send + Sync + 'static,
        true_hook: Arc<dyn Hook>,
    ) -> Self {
        Self {
            name,
            priority,
            hook_type,
            predicate: Arc::new(predicate),
            true_hook,
            false_hook: None,
        }
    }

    /// Set the hook to execute when condition is false.
    pub fn with_else(mut self, false_hook: Arc<dyn Hook>) -> Self {
        self.false_hook = Some(false_hook);
        self
    }

    /// Build the conditional hook.
    pub fn build(self) -> Arc<dyn Hook> {
        Arc::new(self)
    }
}

#[async_trait]
impl Hook for ConditionalHook {
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
        if (self.predicate)(context) {
            self.true_hook.execute(context).await
        } else if let Some(ref false_hook) = self.false_hook {
            false_hook.execute(context).await
        } else {
            // No else branch, return success without executing anything
            Ok(HookExecutionResult::success())
        }
    }
}

/// Builder for creating hook compositions.
pub struct HookCompositionBuilder {
    name: String,
    priority: HookPriority,
    hook_type: HookType,
}

impl HookCompositionBuilder {
    /// Create a new composition builder.
    pub fn new(name: impl Into<String>, hook_type: HookType) -> Self {
        Self {
            name: name.into(),
            priority: HookPriority::default(),
            hook_type,
        }
    }

    /// Set the priority of the composition.
    pub fn with_priority(mut self, priority: HookPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Build a composite hook.
    pub fn composite(self) -> CompositeHook {
        CompositeHook::new(self.name, self.priority, self.hook_type)
    }

    /// Build a hook chain.
    pub fn chain(self) -> HookChain {
        HookChain::new(self.name, self.priority, self.hook_type)
    }

    /// Build a conditional hook.
    pub fn conditional<F>(self, predicate: F, true_hook: Arc<dyn Hook>) -> ConditionalHook
    where
        F: Fn(&HookContext) -> bool + Send + Sync + 'static,
    {
        ConditionalHook::new(self.name, self.priority, self.hook_type, predicate, true_hook)
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

    #[async_trait::async_trait]
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
    async fn test_composite_hook() {
        let hook1 = Arc::new(TestHook {
            name: "hook1".to_string(),
            priority: HookPriority::default(),
            hook_type: HookType::BeforeModel,
        });

        let hook2 = Arc::new(TestHook {
            name: "hook2".to_string(),
            priority: HookPriority::default(),
            hook_type: HookType::BeforeModel,
        });

        let composite = CompositeHook::new(
            "composite".to_string(),
            HookPriority::default(),
            HookType::BeforeModel,
        )
        .add_hook(hook1)
        .add_hook(hook2)
        .build();

        let context = HookContext::new("before_model", serde_json::json!({}));
        let result = composite.execute(&context).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_hook_chain() {
        let hook = Arc::new(TestHook {
            name: "hook1".to_string(),
            priority: HookPriority::default(),
            hook_type: HookType::BeforeModel,
        });

        let chain = HookChain::new(
            "chain".to_string(),
            HookPriority::default(),
            HookType::BeforeModel,
        )
        .link(hook)
        .build();

        let context = HookContext::new("before_model", serde_json::json!({}));
        let result = chain.execute(&context).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_conditional_hook() {
        let hook = Arc::new(TestHook {
            name: "hook1".to_string(),
            priority: HookPriority::default(),
            hook_type: HookType::BeforeModel,
        });

        let conditional = ConditionalHook::new(
            "conditional".to_string(),
            HookPriority::default(),
            HookType::BeforeModel,
            |_| true,
            hook,
        )
        .build();

        let context = HookContext::new("before_model", serde_json::json!({}));
        let result = conditional.execute(&context).await.unwrap();
        assert!(result.success);
    }
}

