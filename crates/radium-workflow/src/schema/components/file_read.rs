//! File Read component schema
//!
//! The File Read component reads files from the filesystem.
//! Supports optional byte-range reads (offset + length) and a configurable
//! safety limit on file size. Uses heartbeat intervals for large-file reads.

use serde::{Deserialize, Serialize};
use validator::Validate;

use super::behaviors::{ComponentBehaviors, RateLimitConfig};

/// File Read component input.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct FileReadInput {
    /// The file path to read.
    #[validate(length(min = 1, message = "path must not be empty"))]
    pub path: String,

    /// The encoding to use when returning file content (defaults to "utf-8").
    #[serde(default = "default_encoding")]
    pub encoding: String,

    /// Maximum file size in bytes that this component is permitted to read.
    /// If the file exceeds this limit, the component will fail safely rather
    /// than attempting to load an unexpectedly large file into memory.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_size_bytes: Option<u64>,

    /// Byte offset to start reading from. When absent, reading begins at the
    /// start of the file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<u64>,

    /// Maximum number of bytes to read. When absent, the entire file (from
    /// `offset`) is returned, subject to `max_size_bytes`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub length: Option<u64>,

    /// Shared component behaviors (retry, rate limit, timeout, etc.).
    #[serde(default = "file_read_default_behaviors")]
    #[validate(nested)]
    pub behaviors: ComponentBehaviors,
}

fn default_encoding() -> String {
    "utf-8".to_string()
}

fn file_read_default_behaviors() -> ComponentBehaviors {
    ComponentBehaviors {
        timeout_ms: 60_000,
        heartbeat_interval_ms: Some(10_000),
        rate_limit: RateLimitConfig {
            requests_per_second: 20,
            burst: 40,
            ..Default::default()
        },
        ..Default::default()
    }
}

impl Default for FileReadInput {
    fn default() -> Self {
        Self {
            path: String::new(),
            encoding: default_encoding(),
            max_size_bytes: None,
            offset: None,
            length: None,
            behaviors: file_read_default_behaviors(),
        }
    }
}

/// File Read component output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FileReadOutput {
    /// The file content as a string.
    pub content: String,

    /// Actual number of bytes read.
    pub size_bytes: u64,

    /// The resolved (canonical) path that was read.
    pub path: String,

    /// The encoding that was used to decode the file.
    pub encoding: String,
}

impl Default for FileReadOutput {
    fn default() -> Self {
        Self {
            content: String::new(),
            size_bytes: 0,
            path: String::new(),
            encoding: default_encoding(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_with_defaults() {
        let input = FileReadInput::default();
        assert_eq!(input.path, "");
        assert_eq!(input.encoding, "utf-8");
        assert!(input.max_size_bytes.is_none());
        assert!(input.offset.is_none());
        assert!(input.length.is_none());
        assert_eq!(input.behaviors.timeout_ms, 60_000);
        assert_eq!(input.behaviors.heartbeat_interval_ms, Some(10_000));
        assert_eq!(input.behaviors.rate_limit.requests_per_second, 20);
        assert_eq!(input.behaviors.rate_limit.burst, 40);
    }

    #[test]
    fn test_full_config_deserialization() {
        let yaml = r#"
path: "/var/log/app.log"
encoding: "ascii"
max_size_bytes: 10485760
offset: 1024
length: 4096
behaviors:
  timeout_ms: 30000
  heartbeat_interval_ms: 5000
  rate_limit:
    requests_per_second: 10
    burst: 20
"#;
        let input: FileReadInput = serde_yaml::from_str(yaml).expect("deserialize");
        assert_eq!(input.path, "/var/log/app.log");
        assert_eq!(input.encoding, "ascii");
        assert_eq!(input.max_size_bytes, Some(10_485_760));
        assert_eq!(input.offset, Some(1024));
        assert_eq!(input.length, Some(4096));
        assert_eq!(input.behaviors.timeout_ms, 30_000);
        assert_eq!(input.behaviors.heartbeat_interval_ms, Some(5_000));
        assert_eq!(input.behaviors.rate_limit.requests_per_second, 10);
        assert_eq!(input.behaviors.rate_limit.burst, 20);
    }

    #[test]
    fn test_output_serialize_deserialize() {
        let output = FileReadOutput {
            content: "Hello, world!".to_string(),
            size_bytes: 13,
            path: "/tmp/hello.txt".to_string(),
            encoding: "utf-8".to_string(),
        };
        let yaml = serde_yaml::to_string(&output).expect("serialize");
        let restored: FileReadOutput = serde_yaml::from_str(&yaml).expect("deserialize");
        assert_eq!(restored.content, output.content);
        assert_eq!(restored.size_bytes, output.size_bytes);
        assert_eq!(restored.path, output.path);
        assert_eq!(restored.encoding, output.encoding);
    }

    #[test]
    fn test_default_encoding() {
        let input = FileReadInput::default();
        assert_eq!(input.encoding, "utf-8");

        let output = FileReadOutput::default();
        assert_eq!(output.encoding, "utf-8");
    }
}
