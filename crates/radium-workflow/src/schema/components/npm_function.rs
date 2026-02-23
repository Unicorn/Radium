//! NPM Function component schema
//!
//! The NPM Function component invokes a specific function exported by an
//! npm package. Supports version pinning and arbitrary JSON arguments.

use serde::{Deserialize, Serialize};
use validator::Validate;

use super::behaviors::{ComponentBehaviors, RateLimitConfig};

/// NPM Function component input.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct NpmFunctionInput {
    /// The npm package name.
    #[validate(length(min = 1, message = "package must not be empty"))]
    pub package: String,

    /// The function to invoke from the package.
    #[validate(length(min = 1, message = "function must not be empty"))]
    pub function: String,

    /// Arguments to pass to the function.
    #[serde(default)]
    pub args: Vec<serde_json::Value>,

    /// Optional package version constraint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Shared component behaviors (retry, rate limit, timeout, etc.).
    #[serde(default = "npm_function_default_behaviors")]
    #[validate(nested)]
    pub behaviors: ComponentBehaviors,
}

fn npm_function_default_behaviors() -> ComponentBehaviors {
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

impl Default for NpmFunctionInput {
    fn default() -> Self {
        Self {
            package: String::new(),
            function: String::new(),
            args: Vec::new(),
            version: None,
            behaviors: npm_function_default_behaviors(),
        }
    }
}

/// NPM Function component output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct NpmFunctionOutput {
    /// The return value of the function call.
    pub result: serde_json::Value,

    /// The resolved package version that was used.
    pub package_version: String,
}

impl Default for NpmFunctionOutput {
    fn default() -> Self {
        Self {
            result: serde_json::Value::Null,
            package_version: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_defaults() {
        let input = NpmFunctionInput::default();
        assert!(input.package.is_empty());
        assert!(input.function.is_empty());
        assert!(input.args.is_empty());
        assert!(input.version.is_none());
        assert_eq!(input.behaviors.timeout_ms, 60_000);
        assert_eq!(input.behaviors.heartbeat_interval_ms, Some(10_000));
    }

    #[test]
    fn test_full_config() {
        let yaml = r#"
package: "lodash"
function: "groupBy"
args:
  - [{"name": "a", "type": 1}, {"name": "b", "type": 2}]
  - "type"
version: "4.17.21"
behaviors:
  timeout_ms: 30000
"#;
        let input: NpmFunctionInput = serde_yaml::from_str(yaml).expect("deserialize");
        assert_eq!(input.package, "lodash");
        assert_eq!(input.function, "groupBy");
        assert_eq!(input.args.len(), 2);
        assert_eq!(input.version, Some("4.17.21".to_string()));
        assert_eq!(input.behaviors.timeout_ms, 30_000);
    }

    #[test]
    fn test_output_round_trip() {
        let output = NpmFunctionOutput {
            result: serde_json::json!({"grouped": true}),
            package_version: "4.17.21".to_string(),
        };
        let yaml = serde_yaml::to_string(&output).expect("serialize");
        let restored: NpmFunctionOutput = serde_yaml::from_str(&yaml).expect("deserialize");
        assert_eq!(restored.result, output.result);
        assert_eq!(restored.package_version, output.package_version);
    }

    #[test]
    fn test_custom_timeout_default() {
        let input = NpmFunctionInput::default();
        assert_eq!(input.behaviors.timeout_ms, 60_000);
        assert_eq!(input.behaviors.rate_limit.requests_per_second, 5);
        assert_eq!(input.behaviors.rate_limit.burst, 10);
    }
}
