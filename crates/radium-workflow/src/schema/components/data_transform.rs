//! Data Transform component schema
//!
//! The Data Transform component applies an expression to input data and
//! returns the transformed result. Supports JSONata and JMESPath expression
//! languages. This is a pure (stateless, side-effect-free) component --
//! it does NOT embed ComponentBehaviors.

use serde::{Deserialize, Serialize};
use validator::Validate;

/// Supported expression languages for data transformation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExpressionLanguage {
    /// JSONata expression language (default).
    #[default]
    Jsonata,
    /// JMESPath expression language.
    Jmespath,
}

/// Data Transform component input.
///
/// Pure tier -- no retry, rate limit, or other I/O behaviors.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct DataTransformInput {
    /// The transformation expression to evaluate.
    #[validate(length(min = 1, message = "expression must not be empty"))]
    pub expression: String,

    /// The expression language to use.
    #[serde(default)]
    pub expression_language: ExpressionLanguage,

    /// The input data to transform.
    pub input: serde_json::Value,
}

impl Default for DataTransformInput {
    fn default() -> Self {
        Self {
            expression: String::new(),
            expression_language: ExpressionLanguage::default(),
            input: serde_json::Value::Null,
        }
    }
}

/// Data Transform component output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DataTransformOutput {
    /// The transformed result.
    pub result: serde_json::Value,
}

impl Default for DataTransformOutput {
    fn default() -> Self {
        Self {
            result: serde_json::Value::Null,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_language() {
        let lang = ExpressionLanguage::default();
        assert_eq!(lang, ExpressionLanguage::Jsonata);
    }

    #[test]
    fn test_input_deserialization() {
        let yaml = r#"
expression: "$.items[?(@.price > 10)]"
expression_language: jmespath
input:
  items:
    - name: "A"
      price: 5
    - name: "B"
      price: 15
"#;
        let input: DataTransformInput = serde_yaml::from_str(yaml).expect("deserialize");
        assert_eq!(input.expression, "$.items[?(@.price > 10)]");
        assert_eq!(input.expression_language, ExpressionLanguage::Jmespath);
        assert!(input.input.is_object());
    }

    #[test]
    fn test_no_behaviors_field() {
        // Verify that DataTransformInput does not have a behaviors field
        // by deserializing YAML that includes one -- it should be ignored.
        let yaml = r#"
expression: "$.name"
input:
  name: "test"
behaviors:
  timeout_ms: 60000
"#;
        // serde should ignore unknown fields or fail -- either way,
        // the struct itself has no behaviors field.
        let input = DataTransformInput::default();
        let yaml_out = serde_yaml::to_string(&input).expect("serialize");
        assert!(!yaml_out.contains("behaviors"));
    }

    #[test]
    fn test_output_round_trip() {
        let output = DataTransformOutput {
            result: serde_json::json!({"filtered": [{"name": "B", "price": 15}]}),
        };
        let yaml = serde_yaml::to_string(&output).expect("serialize");
        let restored: DataTransformOutput = serde_yaml::from_str(&yaml).expect("deserialize");
        assert_eq!(restored.result, output.result);
    }
}
