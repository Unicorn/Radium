//! Code Execute component schema
//!
//! The Code Execute component runs inline JavaScript or TypeScript code
//! within a sandboxed runtime. Supports input bindings for passing data
//! into the code and captures both the return value and console output.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

use super::behaviors::{ComponentBehaviors, RateLimitConfig};

/// Supported languages for inline code execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CodeLanguage {
    /// JavaScript (default).
    #[default]
    JavaScript,
    /// TypeScript (compiled before execution).
    TypeScript,
}

/// Code Execute component input.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct CodeExecuteInput {
    /// The language of the inline code.
    #[serde(default)]
    pub language: CodeLanguage,

    /// The code to execute.
    #[validate(length(min = 1, message = "code must not be empty"))]
    pub code: String,

    /// Named values injected into the code's execution scope.
    #[serde(default)]
    pub input_bindings: HashMap<String, serde_json::Value>,

    /// Shared component behaviors (retry, rate limit, timeout, etc.).
    #[serde(default = "code_execute_default_behaviors")]
    #[validate(nested)]
    pub behaviors: ComponentBehaviors,
}

fn code_execute_default_behaviors() -> ComponentBehaviors {
    ComponentBehaviors {
        timeout_ms: 60_000,
        heartbeat_interval_ms: Some(10_000),
        rate_limit: RateLimitConfig {
            requests_per_second: 5,
            burst: 10,
            ..Default::default()
        },
        ..Default::default()
    }
}

impl Default for CodeExecuteInput {
    fn default() -> Self {
        Self {
            language: CodeLanguage::default(),
            code: String::new(),
            input_bindings: HashMap::new(),
            behaviors: code_execute_default_behaviors(),
        }
    }
}

/// Code Execute component output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CodeExecuteOutput {
    /// The return value of the executed code.
    pub result: serde_json::Value,

    /// Console output lines captured during execution.
    pub console_output: Vec<String>,
}

impl Default for CodeExecuteOutput {
    fn default() -> Self {
        Self {
            result: serde_json::Value::Null,
            console_output: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_language() {
        let lang = CodeLanguage::default();
        assert_eq!(lang, CodeLanguage::JavaScript);
    }

    #[test]
    fn test_input_defaults() {
        let input = CodeExecuteInput::default();
        assert_eq!(input.language, CodeLanguage::JavaScript);
        assert!(input.code.is_empty());
        assert!(input.input_bindings.is_empty());
        assert_eq!(input.behaviors.timeout_ms, 60_000);
        assert_eq!(input.behaviors.heartbeat_interval_ms, Some(10_000));
        assert_eq!(input.behaviors.rate_limit.requests_per_second, 5);
        assert_eq!(input.behaviors.rate_limit.burst, 10);
    }

    #[test]
    fn test_full_config() {
        let yaml = r#"
language: type_script
code: |
  const result = inputs.x + inputs.y;
  return result;
input_bindings:
  x: 10
  y: 20
behaviors:
  timeout_ms: 30000
"#;
        let input: CodeExecuteInput = serde_yaml::from_str(yaml).expect("deserialize");
        assert_eq!(input.language, CodeLanguage::TypeScript);
        assert!(input.code.contains("result"));
        assert_eq!(input.input_bindings.len(), 2);
        assert_eq!(input.behaviors.timeout_ms, 30_000);
    }

    #[test]
    fn test_output_round_trip() {
        let output = CodeExecuteOutput {
            result: serde_json::json!(42),
            console_output: vec!["debug: processing".to_string(), "done".to_string()],
        };
        let yaml = serde_yaml::to_string(&output).expect("serialize");
        let restored: CodeExecuteOutput = serde_yaml::from_str(&yaml).expect("deserialize");
        assert_eq!(restored.result, output.result);
        assert_eq!(restored.console_output, output.console_output);
    }
}
