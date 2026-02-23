//! Cache component schema
//!
//! The Cache component provides key-value cache operations: Get, Set, Delete,
//! and Exists. Supports optional TTL for Set operations and configurable
//! rate limiting for high-throughput cache access patterns.

use serde::{Deserialize, Serialize};
use validator::Validate;

use super::behaviors::{ComponentBehaviors, RateLimitConfig};

/// Cache operation to perform.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CacheAction {
    /// Retrieve a value by key (default).
    #[default]
    Get,
    /// Store a value with an optional TTL.
    Set,
    /// Remove a value by key.
    Delete,
    /// Check whether a key exists.
    Exists,
}

/// Cache component input.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct CacheInput {
    /// The cache operation to perform.
    #[serde(default)]
    pub action: CacheAction,

    /// The cache key.
    #[validate(length(min = 1, message = "key must not be empty"))]
    pub key: String,

    /// The value to store (only used for Set action).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,

    /// Time-to-live in milliseconds for Set operations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttl_ms: Option<u64>,

    /// Shared component behaviors (retry, rate limit, timeout, etc.).
    #[serde(default = "cache_default_behaviors")]
    #[validate(nested)]
    pub behaviors: ComponentBehaviors,
}

fn cache_default_behaviors() -> ComponentBehaviors {
    ComponentBehaviors {
        timeout_ms: 5_000,
        rate_limit: RateLimitConfig {
            requests_per_second: 100,
            burst: 150,
            ..Default::default()
        },
        ..Default::default()
    }
}

impl Default for CacheInput {
    fn default() -> Self {
        Self {
            action: CacheAction::default(),
            key: String::new(),
            value: None,
            ttl_ms: None,
            behaviors: cache_default_behaviors(),
        }
    }
}

/// Cache component output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CacheOutput {
    /// The retrieved value (for Get action).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,

    /// Whether the key exists (for Exists/Get actions).
    #[serde(default)]
    pub exists: bool,

    /// Whether the key was deleted (for Delete action).
    #[serde(default)]
    pub deleted: bool,
}

impl Default for CacheOutput {
    fn default() -> Self {
        Self {
            value: None,
            exists: false,
            deleted: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_action() {
        let action = CacheAction::default();
        assert_eq!(action, CacheAction::Get);
    }

    #[test]
    fn test_set_action_config() {
        let yaml = r#"
action: set
key: "user:123"
value:
  name: "Alice"
  role: "admin"
ttl_ms: 300000
"#;
        let input: CacheInput = serde_yaml::from_str(yaml).expect("deserialize");
        assert_eq!(input.action, CacheAction::Set);
        assert_eq!(input.key, "user:123");
        assert!(input.value.is_some());
        assert_eq!(input.value.unwrap()["name"], "Alice");
        assert_eq!(input.ttl_ms, Some(300_000));
    }

    #[test]
    fn test_get_action_config() {
        let yaml = r#"
key: "user:123"
"#;
        let input: CacheInput = serde_yaml::from_str(yaml).expect("deserialize");
        assert_eq!(input.action, CacheAction::Get);
        assert_eq!(input.key, "user:123");
        assert!(input.value.is_none());
        assert!(input.ttl_ms.is_none());
    }

    #[test]
    fn test_output_for_get() {
        let output = CacheOutput {
            value: Some(serde_json::json!({"name": "Alice"})),
            exists: true,
            deleted: false,
        };
        let yaml = serde_yaml::to_string(&output).expect("serialize");
        let restored: CacheOutput = serde_yaml::from_str(&yaml).expect("deserialize");
        assert!(restored.exists);
        assert!(!restored.deleted);
        assert_eq!(restored.value.unwrap()["name"], "Alice");
    }

    #[test]
    fn test_output_for_exists() {
        let output = CacheOutput {
            value: None,
            exists: true,
            deleted: false,
        };
        let yaml = serde_yaml::to_string(&output).expect("serialize");
        let restored: CacheOutput = serde_yaml::from_str(&yaml).expect("deserialize");
        assert!(restored.exists);
        assert!(restored.value.is_none());
    }

    #[test]
    fn test_output_for_delete() {
        let output = CacheOutput {
            value: None,
            exists: false,
            deleted: true,
        };
        let yaml = serde_yaml::to_string(&output).expect("serialize");
        let restored: CacheOutput = serde_yaml::from_str(&yaml).expect("deserialize");
        assert!(restored.deleted);
        assert!(!restored.exists);
    }

    #[test]
    fn test_custom_rate_limit_default() {
        let input = CacheInput::default();
        assert_eq!(input.behaviors.rate_limit.requests_per_second, 100);
        assert_eq!(input.behaviors.rate_limit.burst, 150);
        assert_eq!(input.behaviors.timeout_ms, 5_000);
    }
}
