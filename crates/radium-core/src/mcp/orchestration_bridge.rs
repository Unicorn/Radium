//! Bridge between MCP integration and orchestration system
//!
//! This module provides a bridge to convert MCP tools to orchestration tools
//! without creating circular dependencies.

use crate::mcp::integration::McpIntegration;
use radium_orchestrator::orchestration::mcp_tools::{
    create_mcp_tool, McpContent, McpIntegrationTrait, McpToolDefinition, McpToolResult,
};
use radium_orchestrator::orchestration::tool::Tool;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Wrapper that implements McpIntegrationTrait for McpIntegration
pub struct McpIntegrationWrapper {
    integration: Arc<Mutex<McpIntegration>>,
}

impl McpIntegrationWrapper {
    /// Create a new wrapper
    pub fn new(integration: Arc<Mutex<McpIntegration>>) -> Self {
        Self { integration }
    }
}

#[async_trait::async_trait]
impl McpIntegrationTrait for McpIntegrationWrapper {
    async fn execute_tool(
        &self,
        server_name: &str,
        tool_name: &str,
        arguments: &serde_json::Value,
    ) -> Result<McpToolResult, radium_orchestrator::error::OrchestrationError> {
        let integration = self.integration.lock().await;
        match integration.execute_tool(server_name, tool_name, arguments).await {
            Ok(mcp_result) => {
                // Convert radium_core::mcp::McpContent to orchestration McpContent
                let content: Vec<McpContent> = mcp_result
                    .content
                    .into_iter()
                    .map(|c| match c {
                        crate::mcp::McpContent::Text { text } => McpContent::Text { text },
                        crate::mcp::McpContent::Image { data, mime_type } => {
                            McpContent::Image { data, mime_type }
                        }
                        crate::mcp::McpContent::Audio { data, mime_type } => {
                            McpContent::Audio { data, mime_type }
                        }
                    })
                    .collect();

                Ok(McpToolResult { content, is_error: mcp_result.is_error })
            }
            Err(e) => Err(radium_orchestrator::error::OrchestrationError::Other(format!(
                "MCP tool execution failed: {}",
                e
            ))),
        }
    }
}

/// Discover MCP tools and convert them to orchestration Tool objects
///
/// # Arguments
/// * `integration` - Initialized MCP integration instance
///
/// # Returns
/// Vector of Tool objects representing MCP tools
pub async fn discover_mcp_tools_for_orchestration(
    integration: Arc<Mutex<McpIntegration>>,
) -> anyhow::Result<Vec<Tool>> {
    let tool_defs = {
        let int = integration.lock().await;
        int.get_all_tool_definitions().await
    };

    let wrapper: Arc<dyn McpIntegrationTrait> = Arc::new(McpIntegrationWrapper::new(integration));

    let mut tools = Vec::new();

    for (server_name, tool_name, mcp_tool) in tool_defs {
        let def = McpToolDefinition {
            server_name: server_name.clone(),
            tool_name: tool_name.clone(),
            original_tool_name: mcp_tool.name.clone(),
            description: mcp_tool
                .description
                .clone()
                .unwrap_or_else(|| format!("MCP tool from server '{}'", server_name)),
            input_schema: mcp_tool.input_schema.clone(),
        };

        let tool = create_mcp_tool(def, Arc::clone(&wrapper));
        tools.push(tool);
    }

    Ok(tools)
}

