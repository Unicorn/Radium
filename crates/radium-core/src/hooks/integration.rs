//! Hooks integration helpers for orchestrator providers.
//!
//! This module provides utilities for integrating hooks with orchestrator providers.
//! Since radium-orchestrator doesn't depend on radium-core, hooks integration
//! happens at the application level where both crates are available.

use crate::hooks::model::ModelHookContext;
use crate::hooks::registry::{HookRegistry, HookType};
use crate::hooks::telemetry::TelemetryHookContext;
use crate::hooks::tool::ToolHookContext;
use crate::hooks::types::HookContext;
use radium_orchestrator::{AgentOutput, HookExecutor, HookResult};
use std::sync::Arc;

/// Helper for executing hooks around orchestrator operations.
pub struct OrchestratorHooks {
    /// Hook registry.
    pub registry: Arc<HookRegistry>,
}

impl OrchestratorHooks {
    /// Create a new orchestrator hooks helper.
    pub fn new(registry: Arc<HookRegistry>) -> Self {
        Self { registry }
    }

    /// Execute before model call hooks.
    pub async fn before_model_call(
        &self,
        input: &str,
        model_id: &str,
    ) -> crate::hooks::error::Result<(String, Option<serde_json::Value>)> {
        let context = ModelHookContext::before(input.to_string(), model_id.to_string());
        let hook_context = context.to_hook_context(crate::hooks::model::ModelHookType::Before);

        let results = self.registry.execute_hooks(HookType::BeforeModel, &hook_context).await?;

        // Collect modifications from hooks
        let mut modified_input = input.to_string();
        let mut request_modifications = None;

        for result in results {
            if let Some(data) = result.modified_data {
                // Try to extract modified input or request modifications
                if let Some(input_val) = data.get("modified_input").and_then(|v| v.as_str()) {
                    modified_input = input_val.to_string();
                }
                if let Some(mods) = data.get("request_modifications") {
                    request_modifications = Some(mods.clone());
                }
            }
            // If any hook says to stop, we stop
            if !result.should_continue {
                return Err(crate::hooks::error::HookError::ExecutionFailed(
                    result.message.unwrap_or_else(|| "Hook requested stop".to_string()),
                ));
            }
        }

        Ok((modified_input, request_modifications))
    }

    /// Execute after model call hooks.
    pub async fn after_model_call(
        &self,
        input: &str,
        model_id: &str,
        response: &str,
    ) -> crate::hooks::error::Result<String> {
        let context =
            ModelHookContext::after(input.to_string(), model_id.to_string(), response.to_string());
        let hook_context = context.to_hook_context(crate::hooks::model::ModelHookType::After);

        let results = self.registry.execute_hooks(HookType::AfterModel, &hook_context).await?;

        // Collect modifications from hooks
        let mut modified_response = response.to_string();

        for result in results {
            if let Some(data) = result.modified_data {
                if let Some(response_val) = data.get("response").and_then(|v| v.as_str()) {
                    modified_response = response_val.to_string();
                }
            }
        }

        Ok(modified_response)
    }

    /// Execute before tool execution hooks.
    pub async fn before_tool_execution(
        &self,
        tool_name: &str,
        arguments: &serde_json::Value,
    ) -> crate::hooks::error::Result<serde_json::Value> {
        let context = ToolHookContext::before(tool_name.to_string(), arguments.clone());
        let hook_context = context.to_hook_context(crate::hooks::tool::ToolHookType::Before);

        let results = self.registry.execute_hooks(HookType::BeforeTool, &hook_context).await?;

        // Collect modifications from hooks
        let mut modified_arguments = arguments.clone();

        for result in results {
            if let Some(data) = result.modified_data {
                if let Some(args) = data.get("modified_arguments") {
                    modified_arguments = args.clone();
                }
            }
            if !result.should_continue {
                return Err(crate::hooks::error::HookError::ExecutionFailed(
                    result.message.unwrap_or_else(|| "Hook requested stop".to_string()),
                ));
            }
        }

        Ok(modified_arguments)
    }

    /// Execute after tool execution hooks.
    pub async fn after_tool_execution(
        &self,
        tool_name: &str,
        arguments: &serde_json::Value,
        result: &serde_json::Value,
    ) -> crate::hooks::error::Result<serde_json::Value> {
        let context =
            ToolHookContext::after(tool_name.to_string(), arguments.clone(), result.clone());
        let hook_context = context.to_hook_context(crate::hooks::tool::ToolHookType::After);

        let hook_results = self.registry.execute_hooks(HookType::AfterTool, &hook_context).await?;

        // Collect modifications from hooks
        let mut modified_result = result.clone();

        for hook_result in hook_results {
            if let Some(data) = hook_result.modified_data {
                if let Some(res) = data.get("modified_result") {
                    modified_result = res.clone();
                }
            }
        }

        Ok(modified_result)
    }

    /// Execute tool selection hooks.
    pub async fn tool_selection(
        &self,
        tool_name: &str,
        arguments: &serde_json::Value,
    ) -> crate::hooks::error::Result<bool> {
        let context = ToolHookContext::selection(tool_name.to_string(), arguments.clone());
        let hook_context = context.to_hook_context(crate::hooks::tool::ToolHookType::Selection);

        let results = self.registry.execute_hooks(HookType::ToolSelection, &hook_context).await?;

        // If any hook says to stop, don't execute the tool
        for result in results {
            if !result.should_continue {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Execute error interception hooks.
    pub async fn error_interception(
        &self,
        error_message: &str,
        error_type: &str,
        error_source: Option<&str>,
    ) -> crate::hooks::error::Result<Option<String>> {
        let context = crate::hooks::error_hooks::ErrorHookContext::interception(
            error_message.to_string(),
            error_type.to_string(),
            error_source.map(|s| s.to_string()),
        );
        let hook_context =
            context.to_hook_context(crate::hooks::error_hooks::ErrorHookType::Interception);

        let results =
            self.registry.execute_hooks(HookType::ErrorInterception, &hook_context).await?;

        // Check if any hook handled the error
        for result in results {
            if !result.should_continue {
                // Error was handled, return the message
                return Ok(result.message);
            }
        }

        Ok(None)
    }

    /// Execute telemetry hooks.
    pub async fn telemetry_collection(
        &self,
        event_type: &str,
        data: &serde_json::Value,
    ) -> crate::hooks::error::Result<()> {
        let context = TelemetryHookContext::new(event_type, data.clone());
        let hook_context = context.to_hook_context("telemetry_collection");

        let _results =
            self.registry.execute_hooks(HookType::TelemetryCollection, &hook_context).await?;

        // Telemetry hooks are fire-and-forget, we don't need to process results
        Ok(())
    }
}

/// Adapter that implements HookExecutor trait for HookRegistry.
/// This allows HookRegistry to be used with AgentExecutor without creating
/// a circular dependency between radium-core and radium-orchestrator.
pub struct HookRegistryAdapter {
    registry: Arc<HookRegistry>,
}

impl HookRegistryAdapter {
    /// Create a new adapter from a HookRegistry.
    pub fn new(registry: Arc<HookRegistry>) -> Self {
        Self { registry }
    }
}

#[async_trait::async_trait]
impl HookExecutor for HookRegistryAdapter {
    async fn execute_before_model(&self, agent_id: &str, input: &str) -> Result<Vec<HookResult>, String> {
        let hook_context = HookContext::new(
            "before_model",
            serde_json::json!({
                "agent_id": agent_id,
                "input": input,
            }),
        );

        match self.registry.execute_hooks(HookType::BeforeModel, &hook_context).await {
            Ok(results) => Ok(results
                .into_iter()
                .map(|r| HookResult {
                    should_continue: r.should_continue,
                    message: r.message,
                    modified_data: r.modified_data,
                })
                .collect()),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn execute_after_model(
        &self,
        agent_id: &str,
        output: &AgentOutput,
        success: bool,
    ) -> Result<Vec<HookResult>, String> {
        let output_json = match output {
            AgentOutput::Text(t) => serde_json::json!({ "text": t }),
            AgentOutput::StructuredData(s) => s.clone(),
            AgentOutput::ToolCall { name, args } => {
                serde_json::json!({ "tool_call": { "name": name, "args": args } })
            }
            AgentOutput::Terminate => serde_json::json!({ "terminate": true }),
        };

        let hook_context = HookContext::new(
            "after_model",
            serde_json::json!({
                "agent_id": agent_id,
                "output": output_json,
                "success": success,
            }),
        );

        match self.registry.execute_hooks(HookType::AfterModel, &hook_context).await {
            Ok(results) => Ok(results
                .into_iter()
                .map(|r| HookResult {
                    should_continue: r.should_continue,
                    message: r.message,
                    modified_data: r.modified_data,
                })
                .collect()),
            Err(e) => Err(e.to_string()),
        }
    }
}
