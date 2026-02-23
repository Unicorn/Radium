//! Secret Read component schema
//!
//! The Secret Read component retrieves a secret value from the platform's
//! secret store. Supports versioned secrets. The returned value is masked
//! in logs to prevent accidental exposure.

use serde::{Deserialize, Serialize};
use validator::Validate;

use super::behaviors::{ComponentBehaviors, RateLimitConfig, RetryPolicy};

/// Secret Read component input.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct SecretReadInput {
    /// The name of the secret to retrieve.
    #[validate(length(min = 1, message = "name must not be empty"))]
    pub name: String,

    /// Specific version of the secret to retrieve. When None, the latest
    /// version is returned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Shared component behaviors (retry, rate limit, timeout, etc.).
    #[serde(default = "secret_read_default_behaviors")]
    #[validate(nested)]
    pub behaviors: ComponentBehaviors,
}

fn secret_read_default_behaviors() -> ComponentBehaviors {
    ComponentBehaviors {
        timeout_ms: 5_000,
        retry_policy: RetryPolicy {
            max_attempts: 1,
            ..Default::default()
        },
        rate_limit: RateLimitConfig {
            requests_per_second: 20,
            burst: 30,
            ..Default::default()
        },
        ..Default::default()
    }
}

impl Default for SecretReadInput {
    fn default() -> Self {
        Self {
            name: String::new(),
            version: None,
            behaviors: secret_read_default_behaviors(),
        }
    }
}

/// Secret Read component output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SecretReadOutput {
    /// The secret value (masked in logs).
    pub value: String,

    /// The version of the secret that was retrieved.
    pub version: String,
}

impl Default for SecretReadOutput {
    fn default() -> Self {
        Self {
            value: String::new(),
            version: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_defaults() {
        let input = SecretReadInput::default();
        assert!(input.name.is_empty());
        assert!(input.version.is_none());
        assert_eq!(input.behaviors.timeout_ms, 5_000);
        assert_eq!(input.behaviors.retry_policy.max_attempts, 1);
        assert_eq!(input.behaviors.rate_limit.requests_per_second, 20);
        assert_eq!(input.behaviors.rate_limit.burst, 30);
    }

    #[test]
    fn test_custom_timeout_verify() {
        let yaml = r#"
name: "api_key"
behaviors:
  timeout_ms: 10000
"#;
        let input: SecretReadInput = serde_yaml::from_str(yaml).expect("deserialize");
        assert_eq!(input.name, "api_key");
        assert_eq!(input.behaviors.timeout_ms, 10_000);
    }

    #[test]
    fn test_full_config() {
        let yaml = r#"
name: "database_password"
version: "v3"
behaviors:
  timeout_ms: 3000
  retry_policy:
    max_attempts: 2
  rate_limit:
    requests_per_second: 10
    burst: 15
"#;
        let input: SecretReadInput = serde_yaml::from_str(yaml).expect("deserialize");
        assert_eq!(input.name, "database_password");
        assert_eq!(input.version, Some("v3".to_string()));
        assert_eq!(input.behaviors.timeout_ms, 3_000);
        assert_eq!(input.behaviors.retry_policy.max_attempts, 2);
        assert_eq!(input.behaviors.rate_limit.requests_per_second, 10);
        assert_eq!(input.behaviors.rate_limit.burst, 15);
    }

    #[test]
    fn test_output_round_trip() {
        let output = SecretReadOutput {
            value: "s3cr3t-value".to_string(),
            version: "v3".to_string(),
        };
        let yaml = serde_yaml::to_string(&output).expect("serialize");
        let restored: SecretReadOutput = serde_yaml::from_str(&yaml).expect("deserialize");
        assert_eq!(restored.value, output.value);
        assert_eq!(restored.version, output.version);
    }
}
