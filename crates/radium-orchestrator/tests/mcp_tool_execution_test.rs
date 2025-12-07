//! Tests for MCP tool execution in orchestration

use async_trait::async_trait;
use radium_orchestrator::orchestration::{
    context::OrchestrationContext,
    engine::{EngineConfig, OrchestrationEngine},
    mcp_tools::{create_mcp_tool, McpContent, McpIntegrationTrait, McpToolDefinition, McpToolResult},
    tool::{Tool, ToolCall},
    FinishReason, OrchestrationResult,
};
use radium_orchestrator::OrchestrationProvider;
use serde_json::json;
use std::sync::Arc;

/// Mock MCP integration for testing
struct MockMcpIntegration {
    tools: Vec<(String, String, String)>, // (server, tool_name, description)
}

impl MockMcpIntegration {
    fn new() -> Self {
        Self { tools: Vec::new() }
    }

    fn add_tool(&mut self, server: String, tool_name: String, description: String) {
        self.tools.push((server, tool_name, description));
    }
}

#[async_trait]
impl McpIntegrationTrait for MockMcpIntegration {
    async fn execute_tool(
        &self,
        server_name: &str,
        tool_name: &str,
        arguments: &serde_json::Value,
    ) -> Result<McpToolResult, radium_orchestrator::error::OrchestrationError> {
        // Verify tool exists
        let found = self
            .tools
            .iter()
            .any(|(s, t, _)| s == server_name && t == tool_name);

        if !found {
            return Err(radium_orchestrator::error::OrchestrationError::Other(format!(
                "Tool '{}' not found on server '{}'",
                tool_name, server_name
            )));
        }

        // Return mock result based on tool name
        let result = match tool_name {
            "echo" => McpToolResult {
                content: vec![McpContent::Text {
                    text: format!("Echo: {}", arguments.get("text").and_then(|v| v.as_str()).unwrap_or("")),
                }],
                is_error: false,
            },
            "error_tool" => McpToolResult {
                content: vec![McpContent::Text {
                    text: "Tool execution failed".to_string(),
                }],
                is_error: true,
            },
            _ => McpToolResult {
                content: vec![McpContent::Text {
                    text: format!("Tool '{}' executed with args: {}", tool_name, arguments),
                }],
                is_error: false,
            },
        };

        Ok(result)
    }
}

/// Mock provider that requests tool calls
struct MockToolCallProvider {
    tool_calls: Vec<ToolCall>,
}

impl MockToolCallProvider {
    fn new(tool_calls: Vec<ToolCall>) -> Self {
        Self { tool_calls }
    }
}

#[async_trait]
impl OrchestrationProvider for MockToolCallProvider {
    async fn execute_with_tools(
        &self,
        _input: &str,
        _tools: &[Tool],
        _context: &OrchestrationContext,
    ) -> radium_orchestrator::error::Result<OrchestrationResult> {
        Ok(OrchestrationResult {
            response: "".to_string(),
            tool_calls: self.tool_calls.clone(),
            finish_reason: FinishReason::Stop,
        })
    }

    fn provider_name(&self) -> &'static str {
        "mock"
    }

    fn supports_function_calling(&self) -> bool {
        true
    }
}

#[tokio::test]
async fn test_mcp_tool_discovery() {
    let mut mock_mcp = MockMcpIntegration::new();
    mock_mcp.add_tool("server1".to_string(), "echo".to_string(), "Echo tool".to_string());
    mock_mcp.add_tool("server2".to_string(), "search".to_string(), "Search tool".to_string());

    let mcp_integration: Arc<dyn McpIntegrationTrait> = Arc::new(mock_mcp);

    // Create MCP tool definitions
    let tool1_def = McpToolDefinition {
        server_name: "server1".to_string(),
        tool_name: "echo".to_string(),
        original_tool_name: "echo".to_string(),
        description: "Echo tool".to_string(),
        input_schema: Some(json!({
            "type": "object",
            "properties": {
                "text": {"type": "string"}
            }
        })),
    };

    let tool2_def = McpToolDefinition {
        server_name: "server2".to_string(),
        tool_name: "search".to_string(),
        original_tool_name: "search".to_string(),
        description: "Search tool".to_string(),
        input_schema: None,
    };

    let tool1 = create_mcp_tool(tool1_def, Arc::clone(&mcp_integration));
    let tool2 = create_mcp_tool(tool2_def, Arc::clone(&mcp_integration));

    assert_eq!(tool1.name, "echo");
    assert_eq!(tool2.name, "search");
}

#[tokio::test]
async fn test_mcp_tool_execution() {
    let mut mock_mcp = MockMcpIntegration::new();
    mock_mcp.add_tool("server1".to_string(), "echo".to_string(), "Echo tool".to_string());

    let mcp_integration: Arc<dyn McpIntegrationTrait> = Arc::new(mock_mcp);

    let tool_def = McpToolDefinition {
        server_name: "server1".to_string(),
        tool_name: "echo".to_string(),
        original_tool_name: "echo".to_string(),
        description: "Echo tool".to_string(),
        input_schema: Some(json!({
            "type": "object",
            "properties": {
                "text": {"type": "string"}
            }
        })),
    };

    let tool = create_mcp_tool(tool_def, Arc::clone(&mcp_integration));

    // Execute tool
    let args = radium_orchestrator::orchestration::tool::ToolArguments::new(json!({
        "text": "Hello, world!"
    }));

    let result = tool.execute(&args).await.unwrap();
    assert!(result.success);
    assert!(result.output.contains("Echo: Hello, world!"));
}

#[tokio::test]
async fn test_mcp_tool_execution_error() {
    let mut mock_mcp = MockMcpIntegration::new();
    mock_mcp.add_tool("server1".to_string(), "error_tool".to_string(), "Error tool".to_string());

    let mcp_integration: Arc<dyn McpIntegrationTrait> = Arc::new(mock_mcp);

    let tool_def = McpToolDefinition {
        server_name: "server1".to_string(),
        tool_name: "error_tool".to_string(),
        original_tool_name: "error_tool".to_string(),
        description: "Error tool".to_string(),
        input_schema: None,
    };

    let tool = create_mcp_tool(tool_def, Arc::clone(&mcp_integration));

    let args = radium_orchestrator::orchestration::tool::ToolArguments::new(json!({}));
    let result = tool.execute(&args).await.unwrap();

    // Error tool should return error result
    assert!(!result.success);
    assert!(result.output.contains("Tool execution failed"));
}

#[tokio::test]
async fn test_mcp_tool_conflict_resolution() {
    let mut mock_mcp = MockMcpIntegration::new();
    mock_mcp.add_tool("server1".to_string(), "query".to_string(), "Query tool 1".to_string());
    mock_mcp.add_tool("server2".to_string(), "query".to_string(), "Query tool 2".to_string());

    let mcp_integration: Arc<dyn McpIntegrationTrait> = Arc::new(mock_mcp);

    // Create tools with same name from different servers
    let tool1_def = McpToolDefinition {
        server_name: "server1".to_string(),
        tool_name: "server1:query".to_string(), // Prefixed for conflict
        original_tool_name: "query".to_string(),
        description: "Query tool 1".to_string(),
        input_schema: None,
    };

    let tool2_def = McpToolDefinition {
        server_name: "server2".to_string(),
        tool_name: "server2:query".to_string(), // Prefixed for conflict
        original_tool_name: "query".to_string(),
        description: "Query tool 2".to_string(),
        input_schema: None,
    };

    let tool1 = create_mcp_tool(tool1_def, Arc::clone(&mcp_integration));
    let tool2 = create_mcp_tool(tool2_def, Arc::clone(&mcp_integration));

    // Both tools should exist with prefixed names
    assert_eq!(tool1.name, "server1:query");
    assert_eq!(tool2.name, "server2:query");

    // Both should execute correctly
    let args = radium_orchestrator::orchestration::tool::ToolArguments::new(json!({}));
    let result1 = tool1.execute(&args).await.unwrap();
    let result2 = tool2.execute(&args).await.unwrap();

    assert!(result1.success);
    assert!(result2.success);
}

#[tokio::test]
async fn test_mcp_tool_in_orchestration_engine() {
    let mut mock_mcp = MockMcpIntegration::new();
    mock_mcp.add_tool("server1".to_string(), "echo".to_string(), "Echo tool".to_string());

    let mcp_integration: Arc<dyn McpIntegrationTrait> = Arc::new(mock_mcp);

    let tool_def = McpToolDefinition {
        server_name: "server1".to_string(),
        tool_name: "echo".to_string(),
        original_tool_name: "echo".to_string(),
        description: "Echo tool".to_string(),
        input_schema: Some(json!({
            "type": "object",
            "properties": {
                "text": {"type": "string"}
            }
        })),
    };

    let mcp_tool = create_mcp_tool(tool_def, Arc::clone(&mcp_integration));

    // Create orchestration engine with MCP tool
    let provider = Arc::new(MockToolCallProvider::new(vec![ToolCall {
        id: "call-1".to_string(),
        name: "echo".to_string(),
        arguments: json!({"text": "test"}),
    }]));

    let engine = OrchestrationEngine::new(
        provider,
        vec![mcp_tool],
        EngineConfig::default(),
    );

    let mut context = OrchestrationContext::new("test-session");
    let result = engine.execute("test input", &mut context).await.unwrap();

    // Should have executed the tool (may finish with MaxIterations if tool calls continue)
    assert!(result.finish_reason == FinishReason::Stop || result.finish_reason == FinishReason::MaxIterations);
}

