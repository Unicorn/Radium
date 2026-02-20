//! Workflow variable definitions

use serde::{Deserialize, Serialize};

/// Variable types - exhaustive
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum VariableType {
    String,
    Number,
    Boolean,
    Array,
    Object,
}

impl Default for VariableType {
    fn default() -> Self {
        VariableType::String
    }
}

impl VariableType {
    /// Get the TypeScript type equivalent
    pub fn to_typescript(&self) -> &'static str {
        match self {
            VariableType::String => "string",
            VariableType::Number => "number",
            VariableType::Boolean => "boolean",
            VariableType::Array => "unknown[]",
            VariableType::Object => "Record<string, unknown>",
        }
    }
}

/// Workflow variable
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowVariable {
    pub name: String,
    #[serde(rename = "type")]
    pub var_type: VariableType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_value: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl WorkflowVariable {
    /// Create a new variable
    pub fn new(name: impl Into<String>, var_type: VariableType) -> Self {
        WorkflowVariable {
            name: name.into(),
            var_type,
            initial_value: None,
            description: None,
        }
    }

    /// Get the default value for this variable type
    pub fn default_value(&self) -> serde_json::Value {
        self.initial_value
            .clone()
            .unwrap_or_else(|| match self.var_type {
                VariableType::String => serde_json::Value::String(String::new()),
                VariableType::Number => serde_json::Value::Number(0.into()),
                VariableType::Boolean => serde_json::Value::Bool(false),
                VariableType::Array => serde_json::Value::Array(vec![]),
                VariableType::Object => serde_json::Value::Object(serde_json::Map::new()),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_type_serialization() {
        let var_type = VariableType::String;
        let json = serde_json::to_string(&var_type).unwrap();
        assert_eq!(json, "\"string\"");
    }

    #[test]
    fn test_variable_deserialization() {
        let json = r#"{
            "name": "counter",
            "type": "number",
            "initialValue": 0,
            "description": "A counter variable"
        }"#;

        let var: WorkflowVariable = serde_json::from_str(json).unwrap();
        assert_eq!(var.name, "counter");
        assert_eq!(var.var_type, VariableType::Number);
        assert_eq!(var.initial_value, Some(serde_json::Value::Number(0.into())));
    }

    #[test]
    fn test_typescript_types() {
        assert_eq!(VariableType::String.to_typescript(), "string");
        assert_eq!(VariableType::Number.to_typescript(), "number");
        assert_eq!(VariableType::Boolean.to_typescript(), "boolean");
        assert_eq!(VariableType::Array.to_typescript(), "unknown[]");
        assert_eq!(
            VariableType::Object.to_typescript(),
            "Record<string, unknown>"
        );
    }
}
