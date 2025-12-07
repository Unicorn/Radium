//! Model call hooks.

use crate::hooks::error::Result;
use crate::hooks::registry::HookType;
use crate::hooks::types::{HookContext, HookPriority, HookResult as HookExecutionResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Type of model hook.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelHookType {
    /// Before model call.
    Before,
    /// After model call.
    After,
}

/// Context for model call hooks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelHookContext {
    /// The input/prompt sent to the model.
    pub input: String,
    /// The model ID being used.
    pub model_id: String,
    /// Optional request modifications.
    pub request_modifications: Option<serde_json::Value>,
    /// Optional response from the model (for after hooks).
    pub response: Option<String>,
    /// Optional modified input.
    pub modified_input: Option<String>,
}

impl ModelHookContext {
    /// Create a new model hook context for before model call.
    pub fn before(input: String, model_id: String) -> Self {
        Self {
            input,
            model_id,
            request_modifications: None,
            response: None,
            modified_input: None,
        }
    }

    /// Create a new model hook context for after model call.
    pub fn after(input: String, model_id: String, response: String) -> Self {
        Self {
            input,
            model_id,
            request_modifications: None,
            response: Some(response),
            modified_input: None,
        }
    }

    /// Convert to hook context.
    pub fn to_hook_context(&self, hook_type: ModelHookType) -> HookContext {
        let hook_type_str = match hook_type {
            ModelHookType::Before => "before_model",
            ModelHookType::After => "after_model",
        };

        HookContext::new(
            hook_type_str,
            serde_json::to_value(self).unwrap_or(serde_json::Value::Null),
        )
    }
}

/// Trait for model hooks.
#[async_trait]
pub trait ModelHook: Send + Sync {
    /// Get the name of the hook.
    fn name(&self) -> &str;

    /// Get the priority of the hook.
    fn priority(&self) -> HookPriority;

    /// Execute before model call.
    async fn before_model_call(
        &self,
        context: &ModelHookContext,
    ) -> Result<HookExecutionResult>;

    /// Execute after model call.
    async fn after_model_call(
        &self,
        context: &ModelHookContext,
    ) -> Result<HookExecutionResult>;
}

/// Adapter to convert ModelHook to Hook trait.
pub struct ModelHookAdapter {
    hook: Arc<dyn ModelHook>,
    hook_type: ModelHookType,
}

impl ModelHookAdapter {
    /// Create a new adapter for before model call.
    pub fn before(hook: Arc<dyn ModelHook>) -> Arc<dyn crate::hooks::registry::Hook> {
        Arc::new(Self {
            hook,
            hook_type: ModelHookType::Before,
        })
    }

    /// Create a new adapter for after model call.
    pub fn after(hook: Arc<dyn ModelHook>) -> Arc<dyn crate::hooks::registry::Hook> {
        Arc::new(Self {
            hook,
            hook_type: ModelHookType::After,
        })
    }
}

#[async_trait]
impl crate::hooks::registry::Hook for ModelHookAdapter {
    fn name(&self) -> &str {
        self.hook.name()
    }

    fn priority(&self) -> HookPriority {
        self.hook.priority()
    }

    fn hook_type(&self) -> HookType {
        match self.hook_type {
            ModelHookType::Before => HookType::BeforeModel,
            ModelHookType::After => HookType::AfterModel,
        }
    }

    async fn execute(&self, context: &HookContext) -> Result<HookExecutionResult> {
        let model_context: ModelHookContext = serde_json::from_value(context.data.clone())
            .map_err(|e| crate::hooks::error::HookError::Serialization(e))?;

        match self.hook_type {
            ModelHookType::Before => self.hook.before_model_call(&model_context).await,
            ModelHookType::After => self.hook.after_model_call(&model_context).await,
        }
    }
}

