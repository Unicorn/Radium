//! MCP tool discovery and execution.

use crate::mcp::client::McpClient;
use crate::mcp::{McpError, McpTool, McpToolResult, Result};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Tool registry for managing discovered MCP tools.
pub struct McpToolRegistry {
    /// Map of tool names to tools (with server prefix for conflicts).
    tools: HashMap<String, McpTool>,
    /// Map of original tool names to prefixed names (for conflict resolution).
    tool_name_map: HashMap<String, String>,
    /// Server name for prefixing.
    server_name: String,
}

impl McpToolRegistry {
    /// Create a new tool registry.
    pub fn new(server_name: String) -> Self {
        Self {
            tools: HashMap::new(),
            tool_name_map: HashMap::new(),
            server_name,
        }
    }

    /// Register a tool, handling name conflicts with automatic prefixing.
    pub fn register_tool(&mut self, tool: McpTool) {
        let original_name = tool.name.clone();
        let prefixed_name = if self.tools.contains_key(&original_name) {
            format!("{}:{}", self.server_name, original_name)
        } else {
            original_name.clone()
        };

        self.tool_name_map.insert(original_name.clone(), prefixed_name.clone());
        self.tools.insert(prefixed_name, tool);
    }

    /// Get a tool by name (supports both original and prefixed names).
    pub fn get_tool(&self, name: &str) -> Option<&McpTool> {
        self.tools.get(name)
            .or_else(|| {
                // Try to find by original name
                self.tool_name_map.get(name).and_then(|prefixed| {
                    self.tools.get(prefixed)
                })
            })
    }

    /// Get all registered tools.
    pub fn get_all_tools(&self) -> Vec<&McpTool> {
        self.tools.values().collect()
    }

    /// Check if a tool exists.
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name) || self.tool_name_map.contains_key(name)
    }
}

impl McpClient {
    /// Discover tools from the MCP server.
    ///
    /// # Errors
    ///
    /// Returns an error if tool discovery fails.
    pub async fn discover_tools(&self) -> Result<Vec<McpTool>> {
        let result = self
            .send_request("tools/list", None)
            .await?;

        let tools_value = result
            .get("tools")
            .ok_or_else(|| {
                McpError::Protocol("tools/list response missing 'tools' field".to_string())
            })?;

        let tools: Vec<McpTool> = serde_json::from_value(tools_value.clone())
            .map_err(|e| {
                McpError::Protocol(format!("Failed to parse tools: {}", e))
            })?;

        Ok(tools)
    }

    /// Execute a tool on the MCP server.
    ///
    /// # Errors
    ///
    /// Returns an error if tool execution fails.
    pub async fn execute_tool(
        &self,
        tool_name: &str,
        arguments: &Value,
    ) -> Result<McpToolResult> {
        let params = json!({
            "name": tool_name,
            "arguments": arguments
        });

        let result = self
            .send_request("tools/call", Some(params))
            .await?;

        // Parse the result
        let content = result
            .get("content")
            .and_then(|c| c.as_array())
            .ok_or_else(|| {
                McpError::Protocol("tools/call response missing 'content' field".to_string())
            })?;

        let mcp_content: Vec<crate::mcp::McpContent> = serde_json::from_value(
            serde_json::Value::Array(content.clone())
        )
        .map_err(|e| {
            McpError::Protocol(format!("Failed to parse tool result content: {}", e))
        })?;

        let is_error = result
            .get("isError")
            .and_then(|e| e.as_bool())
            .unwrap_or(false);

        Ok(McpToolResult {
            content: mcp_content,
            is_error,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_registry_creation() {
        let registry = McpToolRegistry::new("test-server".to_string());
        assert_eq!(registry.get_all_tools().len(), 0);
    }

    #[test]
    fn test_tool_registry_register_tool() {
        let mut registry = McpToolRegistry::new("test-server".to_string());
        let tool = McpTool {
            name: "test_tool".to_string(),
            description: Some("A test tool".to_string()),
            input_schema: None,
        };

        registry.register_tool(tool);
        assert_eq!(registry.get_all_tools().len(), 1);
        assert!(registry.has_tool("test_tool"));
    }

    #[test]
    fn test_tool_registry_conflict_resolution() {
        let mut registry = McpToolRegistry::new("server1".to_string());
        
        let tool1 = McpTool {
            name: "query".to_string(),
            description: Some("Query tool".to_string()),
            input_schema: None,
        };
        registry.register_tool(tool1);

        let tool2 = McpTool {
            name: "query".to_string(),
            description: Some("Another query tool".to_string()),
            input_schema: None,
        };
        registry.register_tool(tool2);

        // First tool should have original name, second should be prefixed
        assert!(registry.has_tool("query"));
        assert!(registry.has_tool("server1:query"));
        assert_eq!(registry.get_all_tools().len(), 2);
    }

    #[test]
    fn test_tool_registry_get_tool() {
        let mut registry = McpToolRegistry::new("test-server".to_string());
        let tool = McpTool {
            name: "test_tool".to_string(),
            description: Some("A test tool".to_string()),
            input_schema: None,
        };

        registry.register_tool(tool);
        
        let retrieved = registry.get_tool("test_tool");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test_tool");
    }

    #[test]
    fn test_tool_registry_get_tool_by_prefixed_name() {
        let mut registry = McpToolRegistry::new("server1".to_string());
        
        let tool1 = McpTool {
            name: "query".to_string(),
            description: Some("Query tool".to_string()),
            input_schema: None,
        };
        registry.register_tool(tool1);

        let tool2 = McpTool {
            name: "query".to_string(),
            description: Some("Another query tool".to_string()),
            input_schema: None,
        };
        registry.register_tool(tool2);

        // Should be able to get by prefixed name
        let retrieved = registry.get_tool("server1:query");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "query");
    }

    #[test]
    fn test_tool_registry_get_tool_nonexistent() {
        let registry = McpToolRegistry::new("test-server".to_string());
        let retrieved = registry.get_tool("nonexistent");
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_tool_registry_has_tool_nonexistent() {
        let registry = McpToolRegistry::new("test-server".to_string());
        assert!(!registry.has_tool("nonexistent"));
    }

    #[test]
    fn test_tool_registry_multiple_tools() {
        let mut registry = McpToolRegistry::new("test-server".to_string());
        
        let tool1 = McpTool {
            name: "tool1".to_string(),
            description: Some("First tool".to_string()),
            input_schema: None,
        };
        registry.register_tool(tool1);

        let tool2 = McpTool {
            name: "tool2".to_string(),
            description: Some("Second tool".to_string()),
            input_schema: None,
        };
        registry.register_tool(tool2);

        let tool3 = McpTool {
            name: "tool3".to_string(),
            description: Some("Third tool".to_string()),
            input_schema: None,
        };
        registry.register_tool(tool3);

        assert_eq!(registry.get_all_tools().len(), 3);
        assert!(registry.has_tool("tool1"));
        assert!(registry.has_tool("tool2"));
        assert!(registry.has_tool("tool3"));
    }

    #[test]
    fn test_tool_registry_tool_with_schema() {
        let mut registry = McpToolRegistry::new("test-server".to_string());
        let tool = McpTool {
            name: "schema_tool".to_string(),
            description: Some("Tool with schema".to_string()),
            input_schema: Some(json!({
                "type": "object",
                "properties": {
                    "param1": {"type": "string"}
                }
            })),
        };

        registry.register_tool(tool);
        
        let retrieved = registry.get_tool("schema_tool");
        assert!(retrieved.is_some());
        assert!(retrieved.unwrap().input_schema.is_some());
    }

    #[test]
    fn test_tool_registry_tool_without_description() {
        let mut registry = McpToolRegistry::new("test-server".to_string());
        let tool = McpTool {
            name: "no_desc_tool".to_string(),
            description: None,
            input_schema: None,
        };

        registry.register_tool(tool);
        
        let retrieved = registry.get_tool("no_desc_tool");
        assert!(retrieved.is_some());
        assert!(retrieved.unwrap().description.is_none());
    }

    #[test]
    fn test_tool_registry_conflict_resolution_multiple() {
        let mut registry = McpToolRegistry::new("server1".to_string());
        
        // Register three tools with the same name
        for i in 0..3 {
            let tool = McpTool {
                name: "duplicate".to_string(),
                description: Some(format!("Tool {}", i)),
                input_schema: None,
            };
            registry.register_tool(tool);
        }

        // First should have original name, others should be prefixed
        assert!(registry.has_tool("duplicate"));
        assert!(registry.has_tool("server1:duplicate"));
        assert_eq!(registry.get_all_tools().len(), 3);
    }

    #[test]
    fn test_tool_registry_empty_registry() {
        let registry = McpToolRegistry::new("test-server".to_string());
        assert_eq!(registry.get_all_tools().len(), 0);
        assert!(!registry.has_tool("any_tool"));
    }

    #[test]
    fn test_tool_registry_get_all_tools_order() {
        let mut registry = McpToolRegistry::new("test-server".to_string());
        
        // Register tools in a specific order
        let tool_names = vec!["tool_a", "tool_b", "tool_c"];
        for name in &tool_names {
            let tool = McpTool {
                name: name.to_string(),
                description: None,
                input_schema: None,
            };
            registry.register_tool(tool);
        }

        let all_tools = registry.get_all_tools();
        assert_eq!(all_tools.len(), 3);
        
        // Verify all tools are present (order may vary due to HashMap)
        let retrieved_names: Vec<String> = all_tools.iter().map(|t| t.name.clone()).collect();
        for name in &tool_names {
            assert!(retrieved_names.contains(&name.to_string()));
        }
    }

    #[test]
    fn test_tool_registry_conflict_with_different_servers() {
        // Simulate tools from different servers
        let mut registry1 = McpToolRegistry::new("server1".to_string());
        let mut registry2 = McpToolRegistry::new("server2".to_string());
        
        let tool1 = McpTool {
            name: "common_tool".to_string(),
            description: Some("From server1".to_string()),
            input_schema: None,
        };
        registry1.register_tool(tool1);

        let tool2 = McpTool {
            name: "common_tool".to_string(),
            description: Some("From server2".to_string()),
            input_schema: None,
        };
        registry2.register_tool(tool2);

        // Each registry should handle its own conflicts
        assert!(registry1.has_tool("common_tool"));
        assert!(registry2.has_tool("common_tool"));
    }
}

