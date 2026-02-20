// Tool adapter module for converting between radium_abstraction::Tool and orchestration::Tool
//
// This module bridges the gap between the lightweight Tool definition in radium_abstraction
// (used for AI model APIs) and the full-featured Tool in orchestration (with execution capability).

use std::sync::Arc;

use crate::error::Result;
use crate::orchestration::tool::{Tool as OrchestrationTool, ToolHandler, ToolParameters, ToolArguments, ToolResult};

/// Convert orchestration Tool to abstraction Tool (for AI model APIs)
///
/// This strips out the execution capability and converts to a serializable format
/// that can be sent to AI models.
pub fn to_abstraction_tool(tool: &OrchestrationTool) -> radium_abstraction::Tool {
    radium_abstraction::Tool {
        name: tool.name.clone(),
        description: tool.description.clone(),
        parameters: tool_parameters_to_json(&tool.parameters),
    }
}

/// Convert abstraction Tool to orchestration Tool (requires providing a handler)
///
/// Since abstraction Tool doesn't have execution capability, you must provide
/// a ToolHandler implementation.
pub fn from_abstraction_tool(
    tool: radium_abstraction::Tool,
    handler: Arc<dyn ToolHandler>,
) -> OrchestrationTool {
    OrchestrationTool {
        id: tool.name.clone(), // Use name as ID by default
        name: tool.name,
        description: tool.description,
        parameters: json_to_tool_parameters(&tool.parameters),
        handler,
    }
}

/// Convert multiple orchestration Tools to abstraction Tools
pub fn to_abstraction_tools(tools: &[OrchestrationTool]) -> Vec<radium_abstraction::Tool> {
    tools.iter().map(to_abstraction_tool).collect()
}

/// Convert ToolParameters to JSON schema
fn tool_parameters_to_json(params: &ToolParameters) -> serde_json::Value {
    serde_json::to_value(params)
        .unwrap_or_else(|_| serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        }))
}

/// Convert JSON schema to ToolParameters
fn json_to_tool_parameters(json: &serde_json::Value) -> ToolParameters {
    serde_json::from_value(json.clone())
        .unwrap_or_else(|_| ToolParameters::new())
}

/// Adapter that wraps a ToolHandler to work with abstraction ToolCall
pub struct AbstractionToolAdapter {
    tool: OrchestrationTool,
}

impl AbstractionToolAdapter {
    pub fn new(tool: OrchestrationTool) -> Self {
        Self { tool }
    }

    /// Execute a tool using abstraction ToolCall format
    pub async fn execute_abstraction_call(
        &self,
        call: &radium_abstraction::ToolCall,
    ) -> Result<ToolResult> {
        let args = ToolArguments::new(call.arguments.clone());
        self.tool.execute(&args).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct MockHandler;

    #[async_trait]
    impl ToolHandler for MockHandler {
        async fn execute(&self, _args: &ToolArguments) -> Result<ToolResult> {
            Ok(ToolResult::success("mock result"))
        }
    }

    #[test]
    fn test_to_abstraction_tool() {
        let orch_tool = OrchestrationTool {
            id: "test-tool-1".to_string(),
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            parameters: ToolParameters::new()
                .add_property("query", "string", "Search query", true),
            handler: Arc::new(MockHandler),
        };

        let abs_tool = to_abstraction_tool(&orch_tool);

        assert_eq!(abs_tool.name, "test_tool");
        assert_eq!(abs_tool.description, "A test tool");
        assert!(abs_tool.parameters.is_object());
    }

    #[test]
    fn test_from_abstraction_tool() {
        let abs_tool = radium_abstraction::Tool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    }
                },
                "required": ["query"]
            }),
        };

        let orch_tool = from_abstraction_tool(abs_tool.clone(), Arc::new(MockHandler));

        assert_eq!(orch_tool.name, abs_tool.name);
        assert_eq!(orch_tool.description, abs_tool.description);
        assert_eq!(orch_tool.id, "test_tool"); // ID defaults to name
    }

    #[tokio::test]
    async fn test_abstraction_tool_adapter() {
        let orch_tool = OrchestrationTool {
            id: "test-1".to_string(),
            name: "test_tool".to_string(),
            description: "Test".to_string(),
            parameters: ToolParameters::new(),
            handler: Arc::new(MockHandler),
        };

        let adapter = AbstractionToolAdapter::new(orch_tool);

        let call = radium_abstraction::ToolCall {
            id: "call-1".to_string(),
            name: "test_tool".to_string(),
            arguments: serde_json::json!({}),
        };

        let result = adapter.execute_abstraction_call(&call).await.unwrap();
        assert!(result.success);
        assert_eq!(result.output, "mock result");
    }

    #[test]
    fn test_to_abstraction_tools_batch() {
        let tools = vec![
            OrchestrationTool {
                id: "1".to_string(),
                name: "tool1".to_string(),
                description: "Tool 1".to_string(),
                parameters: ToolParameters::new(),
                handler: Arc::new(MockHandler),
            },
            OrchestrationTool {
                id: "2".to_string(),
                name: "tool2".to_string(),
                description: "Tool 2".to_string(),
                parameters: ToolParameters::new(),
                handler: Arc::new(MockHandler),
            },
        ];

        let abs_tools = to_abstraction_tools(&tools);
        assert_eq!(abs_tools.len(), 2);
        assert_eq!(abs_tools[0].name, "tool1");
        assert_eq!(abs_tools[1].name, "tool2");
    }
}
