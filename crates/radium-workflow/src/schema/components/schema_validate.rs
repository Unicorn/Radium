//! Schema Validate component schema
//!
//! The Schema Validate component validates a JSON value against a JSON Schema
//! and returns whether the data is valid along with detailed error information.
//! This is a pure (stateless, side-effect-free) component -- it does NOT embed
//! ComponentBehaviors.

use serde::{Deserialize, Serialize};

/// Schema Validate component input.
///
/// Pure tier -- no retry, rate limit, or other I/O behaviors.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SchemaValidateInput {
    /// The JSON Schema to validate against.
    pub schema: serde_json::Value,

    /// The data to validate.
    pub data: serde_json::Value,

    /// When true, disallow additional properties not defined in the schema.
    #[serde(default)]
    pub strict: bool,
}

impl Default for SchemaValidateInput {
    fn default() -> Self {
        Self {
            schema: serde_json::Value::Object(serde_json::Map::new()),
            data: serde_json::Value::Null,
            strict: false,
        }
    }
}

/// A single validation error from schema validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SchemaValidationError {
    /// JSON pointer to the invalid field (e.g. "/address/zip").
    pub path: String,

    /// Human-readable description of the error.
    pub message: String,

    /// The JSON Schema keyword that failed (e.g. "required", "type", "minLength").
    pub keyword: String,
}

/// Schema Validate component output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SchemaValidateOutput {
    /// Whether the data passed validation.
    pub valid: bool,

    /// Validation errors (empty when valid is true).
    #[serde(default)]
    pub errors: Vec<SchemaValidationError>,
}

impl Default for SchemaValidateOutput {
    fn default() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_defaults() {
        let input = SchemaValidateInput::default();
        assert!(input.schema.is_object());
        assert!(input.data.is_null());
        assert!(!input.strict);
    }

    #[test]
    fn test_full_config_with_schema() {
        let yaml = r#"
schema:
  type: object
  properties:
    name:
      type: string
    age:
      type: integer
  required:
    - name
data:
  name: "Alice"
  age: 30
strict: true
"#;
        let input: SchemaValidateInput = serde_yaml::from_str(yaml).expect("deserialize");
        assert!(input.schema.is_object());
        assert_eq!(input.data["name"], "Alice");
        assert_eq!(input.data["age"], 30);
        assert!(input.strict);
    }

    #[test]
    fn test_output_with_errors() {
        let output = SchemaValidateOutput {
            valid: false,
            errors: vec![
                SchemaValidationError {
                    path: "/name".to_string(),
                    message: "is required".to_string(),
                    keyword: "required".to_string(),
                },
                SchemaValidationError {
                    path: "/age".to_string(),
                    message: "must be an integer".to_string(),
                    keyword: "type".to_string(),
                },
            ],
        };
        let yaml = serde_yaml::to_string(&output).expect("serialize");
        let restored: SchemaValidateOutput = serde_yaml::from_str(&yaml).expect("deserialize");
        assert!(!restored.valid);
        assert_eq!(restored.errors.len(), 2);
        assert_eq!(restored.errors[0].path, "/name");
        assert_eq!(restored.errors[0].keyword, "required");
        assert_eq!(restored.errors[1].path, "/age");
        assert_eq!(restored.errors[1].keyword, "type");
    }

    #[test]
    fn test_output_valid() {
        let output = SchemaValidateOutput::default();
        assert!(output.valid);
        assert!(output.errors.is_empty());

        let yaml = serde_yaml::to_string(&output).expect("serialize");
        let restored: SchemaValidateOutput = serde_yaml::from_str(&yaml).expect("deserialize");
        assert!(restored.valid);
        assert!(restored.errors.is_empty());
    }
}
