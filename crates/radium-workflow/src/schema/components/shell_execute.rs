//! Shell Execute component schema
//!
//! The Shell Execute component runs shell commands in a sandboxed environment.
//! Supports argument passing, environment variables, working directory, and
//! configurable output capture (stdout, stderr, or both).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

use super::behaviors::{ComponentBehaviors, RateLimitConfig};

/// Controls which output streams are captured from the shell process.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CaptureMode {
    /// Capture only standard output.
    #[default]
    Stdout,
    /// Capture only standard error.
    Stderr,
    /// Capture both stdout and stderr.
    Both,
}

/// Shell Execute component input.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct ShellExecuteInput {
    /// The command to execute.
    #[validate(length(min = 1, message = "command must not be empty"))]
    pub command: String,

    /// Arguments to pass to the command.
    #[serde(default)]
    pub args: Vec<String>,

    /// Working directory for the command.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,

    /// Environment variables to set for the command.
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Which output streams to capture.
    #[serde(default)]
    pub capture: CaptureMode,

    /// Shared component behaviors (retry, rate limit, timeout, etc.).
    #[serde(default = "shell_execute_default_behaviors")]
    #[validate(nested)]
    pub behaviors: ComponentBehaviors,
}

fn shell_execute_default_behaviors() -> ComponentBehaviors {
    ComponentBehaviors {
        timeout_ms: 120_000,
        heartbeat_interval_ms: Some(10_000),
        rate_limit: RateLimitConfig {
            requests_per_second: 1,
            burst: 3,
            ..Default::default()
        },
        ..Default::default()
    }
}

impl Default for ShellExecuteInput {
    fn default() -> Self {
        Self {
            command: String::new(),
            args: Vec::new(),
            working_dir: None,
            env: HashMap::new(),
            capture: CaptureMode::default(),
            behaviors: shell_execute_default_behaviors(),
        }
    }
}

/// Shell Execute component output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ShellExecuteOutput {
    /// Standard output from the command.
    pub stdout: String,

    /// Standard error from the command.
    pub stderr: String,

    /// Exit code of the process.
    pub exit_code: i32,
}

impl Default for ShellExecuteOutput {
    fn default() -> Self {
        Self {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_capture_mode() {
        let mode = CaptureMode::default();
        assert_eq!(mode, CaptureMode::Stdout);
    }

    #[test]
    fn test_input_with_defaults() {
        let input = ShellExecuteInput::default();
        assert!(input.command.is_empty());
        assert!(input.args.is_empty());
        assert!(input.working_dir.is_none());
        assert!(input.env.is_empty());
        assert_eq!(input.capture, CaptureMode::Stdout);
        assert_eq!(input.behaviors.timeout_ms, 120_000);
        assert_eq!(input.behaviors.heartbeat_interval_ms, Some(10_000));
    }

    #[test]
    fn test_full_config_deserialization() {
        let yaml = r#"
command: "ls"
args:
  - "-la"
  - "/tmp"
working_dir: "/home/user"
env:
  PATH: "/usr/bin"
  HOME: "/home/user"
capture: both
behaviors:
  timeout_ms: 60000
"#;
        let input: ShellExecuteInput = serde_yaml::from_str(yaml).expect("deserialize");
        assert_eq!(input.command, "ls");
        assert_eq!(input.args.len(), 2);
        assert_eq!(input.working_dir, Some("/home/user".to_string()));
        assert_eq!(input.env.len(), 2);
        assert_eq!(input.capture, CaptureMode::Both);
        assert_eq!(input.behaviors.timeout_ms, 60_000);
    }

    #[test]
    fn test_output_serialize_deserialize() {
        let output = ShellExecuteOutput {
            stdout: "hello world\n".to_string(),
            stderr: String::new(),
            exit_code: 0,
        };
        let yaml = serde_yaml::to_string(&output).expect("serialize");
        let restored: ShellExecuteOutput = serde_yaml::from_str(&yaml).expect("deserialize");
        assert_eq!(restored.stdout, output.stdout);
        assert_eq!(restored.stderr, output.stderr);
        assert_eq!(restored.exit_code, output.exit_code);
    }

    #[test]
    fn test_custom_rate_limit_default() {
        let input = ShellExecuteInput::default();
        assert_eq!(input.behaviors.rate_limit.requests_per_second, 1);
        assert_eq!(input.behaviors.rate_limit.burst, 3);
    }
}
