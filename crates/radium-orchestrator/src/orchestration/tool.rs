// Tool abstractions for orchestration
//
// Tools represent actions the orchestrator can take (e.g., invoking agents).
// This module defines the tool interface and parameter structures.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::error::Result;

/// Tool call from orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique ID for this tool call
    pub id: String,
    /// Name of the tool to invoke
    pub name: String,
    /// Arguments for the tool
    pub arguments: Value,
}

/// Tool parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    /// Parameter name
    pub name: String,
    /// Parameter type (e.g., "string", "number", "boolean", "object")
    #[serde(rename = "type")]
    pub param_type: String,
    /// Parameter description
    pub description: String,
    /// Whether parameter is required
    pub required: bool,
}

/// Tool parameters schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameters {
    /// Type (always "object" for function parameters)
    #[serde(rename = "type")]
    pub param_type: String,
    /// Property definitions
    pub properties: HashMap<String, ToolPropertySchema>,
    /// Required property names
    pub required: Vec<String>,
}

impl ToolParameters {
    /// Create a new tool parameters schema
    pub fn new() -> Self {
        Self { param_type: "object".to_string(), properties: HashMap::new(), required: Vec::new() }
    }

    /// Add a property to the schema
    #[must_use]
    pub fn add_property(
        mut self,
        name: impl Into<String>,
        property_type: impl Into<String>,
        description: impl Into<String>,
        required: bool,
    ) -> Self {
        let name = name.into();
        self.properties.insert(
            name.clone(),
            ToolPropertySchema {
                property_type: property_type.into(),
                description: description.into(),
            },
        );
        if required {
            self.required.push(name);
        }
        self
    }
}

impl Default for ToolParameters {
    fn default() -> Self {
        Self::new()
    }
}

/// Tool property schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPropertySchema {
    /// Property type
    #[serde(rename = "type")]
    pub property_type: String,
    /// Property description
    pub description: String,
}

/// Arguments passed to tool handler
#[derive(Debug, Clone)]
pub struct ToolArguments {
    /// Parsed arguments as JSON value
    pub args: Value,
}

impl ToolArguments {
    /// Create new tool arguments
    pub fn new(args: Value) -> Self {
        Self { args }
    }

    /// Get argument as string
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.args.get(key)?.as_str().map(str::to_string)
    }

    /// Get argument as i64
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.args.get(key)?.as_i64()
    }

    /// Get argument as bool
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.args.get(key)?.as_bool()
    }

    /// Get argument as object
    pub fn get_object(&self, key: &str) -> Option<&serde_json::Map<String, Value>> {
        self.args.get(key)?.as_object()
    }
}

/// Result from tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Whether execution succeeded
    pub success: bool,
    /// Output from the tool
    pub output: String,
    /// Whether this result represents an error (for ReturnToModel error handling)
    ///
    /// When `is_error` is true, the result represents a tool execution error
    /// that should be sent back to the model for handling.
    #[serde(default)]
    pub is_error: bool,
    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl ToolResult {
    /// Create a successful result
    pub fn success(output: impl Into<String>) -> Self {
        Self {
            success: true,
            output: output.into(),
            is_error: false,
            metadata: HashMap::new(),
        }
    }

    /// Create an error result
    pub fn error(output: impl Into<String>) -> Self {
        Self {
            success: false,
            output: output.into(),
            is_error: false, // Normal error, not ReturnToModel error
            metadata: HashMap::new(),
        }
    }

    /// Create an error result for ReturnToModel strategy (with is_error flag)
    pub fn error_for_model(output: impl Into<String>) -> Self {
        Self {
            success: false,
            output: output.into(),
            is_error: true,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the result
    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Handler for tool execution
#[async_trait]
pub trait ToolHandler: Send + Sync {
    /// Execute the tool with given arguments
    ///
    /// # Arguments
    /// * `args` - Tool arguments
    ///
    /// # Returns
    /// Result of tool execution
    async fn execute(&self, args: &ToolArguments) -> Result<ToolResult>;
}

/// Tool definition for orchestration
#[derive(Clone)]
pub struct Tool {
    /// Unique tool identifier
    pub id: String,
    /// Tool name (used in function calls)
    pub name: String,
    /// Tool description
    pub description: String,
    /// Parameter schema
    pub parameters: ToolParameters,
    /// Handler for executing the tool
    pub handler: Arc<dyn ToolHandler>,
}

impl Tool {
    /// Create a new tool
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: ToolParameters,
        handler: Arc<dyn ToolHandler>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            parameters,
            handler,
        }
    }

    /// Execute this tool with given arguments
    pub async fn execute(&self, args: &ToolArguments) -> Result<ToolResult> {
        self.handler.execute(args).await
    }
}

// Implement Debug manually since Arc<dyn ToolHandler> doesn't implement Debug
impl std::fmt::Debug for Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tool")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("description", &self.description)
            .field("parameters", &self.parameters)
            .field("handler", &"<handler>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_parameters_builder() {
        let params = ToolParameters::new()
            .add_property("task", "string", "The task to perform", true)
            .add_property("priority", "number", "Task priority", false);

        assert_eq!(params.properties.len(), 2);
        assert_eq!(params.required.len(), 1);
        assert_eq!(params.required[0], "task");
    }

    #[test]
    fn test_tool_arguments_get_string() {
        let args = ToolArguments::new(serde_json::json!({
            "task": "test task",
            "priority": 5
        }));

        assert_eq!(args.get_string("task"), Some("test task".to_string()));
        assert_eq!(args.get_i64("priority"), Some(5));
        assert_eq!(args.get_string("missing"), None);
    }

    #[test]
    fn test_tool_result_success() {
        let result = ToolResult::success("Task completed").with_metadata("duration", "1.5s");

        assert!(result.success);
        assert_eq!(result.output, "Task completed");
        assert_eq!(result.metadata.get("duration"), Some(&"1.5s".to_string()));
    }

    #[test]
    fn test_tool_result_error() {
        let result = ToolResult::error("Task failed");

        assert!(!result.success);
        assert_eq!(result.output, "Task failed");
    }
}
