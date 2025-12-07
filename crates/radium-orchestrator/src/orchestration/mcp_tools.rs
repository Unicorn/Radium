//! MCP tool integration for orchestration
//!
//! This module provides integration between MCP (Model Context Protocol) tools
//! and the orchestration system, allowing agents to discover and execute MCP tools.
//!
//! Note: MCP integration initialization happens at the application level (TUI/CLI)
//! to avoid circular dependencies. This module provides the tool handler and
//! conversion utilities.

use async_trait::async_trait;
use serde_json;
use std::sync::Arc;
use std::path::PathBuf;
use base64::{Engine as _, engine::general_purpose};

use super::tool::{Tool, ToolArguments, ToolHandler, ToolParameters, ToolResult};
use crate::error::Result;

/// Save content data to a temporary file.
fn save_content_to_temp_file(data: &str, extension: &str) -> std::io::Result<PathBuf> {
    use std::io::Write;
    
    // Try to decode as base64 first, otherwise use as-is
    let bytes = if data.len() > 100 && data.chars().all(|c| c.is_alphanumeric() || c == '+' || c == '/' || c == '=') {
        // Looks like base64, try to decode
        general_purpose::STANDARD.decode(data).unwrap_or_else(|_| data.as_bytes().to_vec())
    } else {
        data.as_bytes().to_vec()
    };

    let mut temp_file = std::env::temp_dir();
    temp_file.push(format!("radium_mcp_{}.{}", uuid::Uuid::new_v4(), extension));
    
    let mut file = std::fs::File::create(&temp_file)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    
    Ok(temp_file)
}

/// Trait for MCP integration to avoid direct dependency on radium-core
#[async_trait]
pub trait McpIntegrationTrait: Send + Sync {
    /// Execute an MCP tool
    async fn execute_tool(
        &self,
        server_name: &str,
        tool_name: &str,
        arguments: &serde_json::Value,
    ) -> Result<McpToolResult>;
}

/// MCP tool execution result
#[derive(Debug, Clone)]
pub struct McpToolResult {
    /// Content from tool execution
    pub content: Vec<McpContent>,
    /// Whether the execution was an error
    pub is_error: bool,
}

/// MCP content types
#[derive(Debug, Clone)]
pub enum McpContent {
    /// Text content
    Text { text: String },
    /// Image content
    Image { data: String, mime_type: String },
    /// Audio content
    Audio { data: String, mime_type: String },
}

/// MCP tool handler for executing MCP tools
pub struct McpToolHandler {
    /// MCP integration instance
    mcp_integration: Arc<dyn McpIntegrationTrait>,
    /// Server name
    server_name: String,
    /// Tool name (original, not prefixed)
    tool_name: String,
}

#[async_trait]
impl ToolHandler for McpToolHandler {
    async fn execute(&self, args: &ToolArguments) -> Result<ToolResult> {
        // Convert ToolArguments to JSON Value for MCP
        let arguments = args.args.clone();

        // Execute the tool via MCP integration
        match self
            .mcp_integration
            .execute_tool(&self.server_name, &self.tool_name, &arguments)
            .await
        {
            Ok(mcp_result) => {
                // Convert MCP result to ToolResult
                // Extract text content from MCP result and handle rich content
                let mut output_parts = Vec::new();
                let mut temp_files = Vec::new();

                for content in &mcp_result.content {
                    match content {
                        McpContent::Text { text } => {
                            output_parts.push(text.clone());
                        }
                        McpContent::Image { data, mime_type } => {
                            // Save image to temp file
                            let extension = match mime_type.as_str() {
                                "image/png" => "png",
                                "image/jpeg" | "image/jpg" => "jpg",
                                "image/gif" => "gif",
                                "image/webp" => "webp",
                                _ => "bin",
                            };

                            if data.starts_with("http://") || data.starts_with("https://") {
                                // URL-based image
                                output_parts.push(format!("[Image: {}] URL: {}", mime_type, data));
                            } else {
                                // Base64 or raw data - save to temp file
                                match save_content_to_temp_file(data, extension) {
                                    Ok(path) => {
                                        output_parts.push(format!(
                                            "[Image: {}] Saved to: {}",
                                            mime_type,
                                            path.display()
                                        ));
                                        temp_files.push(path);
                                    }
                                    Err(e) => {
                                        output_parts.push(format!(
                                            "[Image: {}] Failed to save: {}",
                                            mime_type, e
                                        ));
                                    }
                                }
                            }
                        }
                        McpContent::Audio { data, mime_type } => {
                            // Save audio to temp file
                            let extension = match mime_type.as_str() {
                                "audio/mpeg" | "audio/mp3" => "mp3",
                                "audio/wav" => "wav",
                                "audio/ogg" => "ogg",
                                "audio/flac" => "flac",
                                _ => "bin",
                            };

                            if data.starts_with("http://") || data.starts_with("https://") {
                                // URL-based audio
                                output_parts.push(format!("[Audio: {}] URL: {}", mime_type, data));
                            } else {
                                // Base64 or raw data - save to temp file
                                match save_content_to_temp_file(data, extension) {
                                    Ok(path) => {
                                        output_parts.push(format!(
                                            "[Audio: {}] Saved to: {}",
                                            mime_type,
                                            path.display()
                                        ));
                                        temp_files.push(path);
                                    }
                                    Err(e) => {
                                        output_parts.push(format!(
                                            "[Audio: {}] Failed to save: {}",
                                            mime_type, e
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }

                let output = if output_parts.is_empty() {
                    "Tool executed successfully".to_string()
                } else {
                    output_parts.join("\n")
                };

                if mcp_result.is_error {
                    Ok(ToolResult::error(output))
                } else {
                    Ok(ToolResult::success(output))
                }
            }
            Err(e) => Ok(ToolResult::error(format!("MCP tool execution failed: {}", e))),
        }
    }
}

/// Convert MCP tool schema to ToolParameters
fn mcp_schema_to_tool_parameters(
    schema: Option<&serde_json::Value>,
) -> ToolParameters {
    let mut params = ToolParameters::new();

    if let Some(schema) = schema {
        if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
            let required = schema
                .get("required")
                .and_then(|r| r.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(std::string::ToString::to_string))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            for (name, prop) in properties {
                if let Some(prop_obj) = prop.as_object() {
                    let param_type = prop_obj
                        .get("type")
                        .and_then(|t| t.as_str())
                        .unwrap_or("string")
                        .to_string();
                    let description = prop_obj
                        .get("description")
                        .and_then(|d| d.as_str())
                        .unwrap_or("")
                        .to_string();
                    let is_required = required.contains(name);

                    params = params.add_property(name, param_type, description, is_required);
                }
            }
        }
    }

    params
}

/// MCP tool definition for creating orchestration tools
pub struct McpToolDefinition {
    /// Server name
    pub server_name: String,
    /// Tool name (may be prefixed like "server:tool")
    pub tool_name: String,
    /// Original tool name (not prefixed)
    pub original_tool_name: String,
    /// Tool description
    pub description: String,
    /// Tool input schema
    pub input_schema: Option<serde_json::Value>,
}

/// Create an orchestration Tool from an MCP tool definition
///
/// # Arguments
/// * `def` - MCP tool definition
/// * `mcp_integration` - MCP integration trait implementation
///
/// # Returns
/// Tool object for orchestration
pub fn create_mcp_tool(
    def: McpToolDefinition,
    mcp_integration: Arc<dyn McpIntegrationTrait>,
) -> Tool {
    // Parse server:tool format if present to get original tool name
    let (actual_server, actual_tool) = if def.tool_name.contains(':') {
        let parts: Vec<&str> = def.tool_name.splitn(2, ':').collect();
        if parts.len() == 2 {
            (parts[0].to_string(), parts[1].to_string())
        } else {
            (def.server_name.clone(), def.original_tool_name.clone())
        }
    } else {
        (def.server_name.clone(), def.original_tool_name.clone())
    };

    // Create handler
    let handler = Arc::new(McpToolHandler {
        mcp_integration,
        server_name: actual_server.clone(),
        tool_name: actual_tool.clone(),
    });

    // Convert MCP tool schema to ToolParameters
    let parameters = mcp_schema_to_tool_parameters(def.input_schema.as_ref());

    

    Tool::new(
        format!("mcp_{}_{}", actual_server, actual_tool),
        def.tool_name.clone(), // Use the registered name (may be prefixed)
        def.description,
        parameters,
        handler,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_schema_to_tool_parameters_empty() {
        let params = mcp_schema_to_tool_parameters(None);
        assert_eq!(params.properties.len(), 0);
        assert_eq!(params.required.len(), 0);
    }

    #[test]
    fn test_mcp_schema_to_tool_parameters_simple() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query"
                },
                "limit": {
                    "type": "number",
                    "description": "Result limit"
                }
            },
            "required": ["query"]
        });

        let params = mcp_schema_to_tool_parameters(Some(&schema));
        assert_eq!(params.properties.len(), 2);
        assert_eq!(params.required.len(), 1);
        assert_eq!(params.required[0], "query");
    }
}

