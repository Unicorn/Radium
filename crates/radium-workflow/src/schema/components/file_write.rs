//! File Write component schema
//!
//! The File Write component writes content to a file at a specified path.
//! Supports overwrite and append modes with optional encoding configuration.
//! Uses heartbeat intervals for long-running write operations.

use serde::{Deserialize, Serialize};
use validator::Validate;

use super::behaviors::ComponentBehaviors;

/// Controls how content is written to the file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum FileWriteMode {
    /// Replace the file contents entirely (default).
    #[default]
    Overwrite,
    /// Append to the end of the file.
    Append,
}

/// File Write component input.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct FileWriteInput {
    /// The file path to write to.
    #[validate(length(min = 1, message = "path must not be empty"))]
    pub path: String,

    /// The content to write.
    pub content: String,

    /// The encoding to use (defaults to utf-8 at runtime).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,

    /// Write mode: overwrite or append.
    #[serde(default)]
    pub mode: FileWriteMode,

    /// Shared component behaviors (retry, rate limit, timeout, etc.).
    #[serde(default = "file_write_default_behaviors")]
    #[validate(nested)]
    pub behaviors: ComponentBehaviors,
}

fn file_write_default_behaviors() -> ComponentBehaviors {
    ComponentBehaviors {
        timeout_ms: 60_000,
        heartbeat_interval_ms: Some(10_000),
        ..Default::default()
    }
}

impl Default for FileWriteInput {
    fn default() -> Self {
        Self {
            path: String::new(),
            content: String::new(),
            encoding: None,
            mode: FileWriteMode::default(),
            behaviors: file_write_default_behaviors(),
        }
    }
}

/// File Write component output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FileWriteOutput {
    /// The path that was written to.
    pub path: String,

    /// The number of bytes written.
    pub bytes_written: u64,
}

impl Default for FileWriteOutput {
    fn default() -> Self {
        Self {
            path: String::new(),
            bytes_written: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_mode() {
        let mode = FileWriteMode::default();
        assert_eq!(mode, FileWriteMode::Overwrite);
    }

    #[test]
    fn test_full_config() {
        let yaml = r#"
path: "/tmp/output.txt"
content: "Hello, world!"
encoding: "utf-8"
mode: append
behaviors:
  timeout_ms: 30000
  heartbeat_interval_ms: 5000
"#;
        let input: FileWriteInput = serde_yaml::from_str(yaml).expect("deserialize");
        assert_eq!(input.path, "/tmp/output.txt");
        assert_eq!(input.content, "Hello, world!");
        assert_eq!(input.encoding, Some("utf-8".to_string()));
        assert_eq!(input.mode, FileWriteMode::Append);
        assert_eq!(input.behaviors.timeout_ms, 30_000);
        assert_eq!(input.behaviors.heartbeat_interval_ms, Some(5_000));
    }

    #[test]
    fn test_output_round_trip() {
        let output = FileWriteOutput {
            path: "/tmp/output.txt".to_string(),
            bytes_written: 1024,
        };
        let yaml = serde_yaml::to_string(&output).expect("serialize");
        let restored: FileWriteOutput = serde_yaml::from_str(&yaml).expect("deserialize");
        assert_eq!(restored.path, output.path);
        assert_eq!(restored.bytes_written, output.bytes_written);
    }

    #[test]
    fn test_custom_timeout_verify() {
        let input = FileWriteInput::default();
        assert_eq!(input.behaviors.timeout_ms, 60_000);
        assert_eq!(input.behaviors.heartbeat_interval_ms, Some(10_000));
        assert_eq!(input.mode, FileWriteMode::Overwrite);
        assert!(input.encoding.is_none());
    }
}
