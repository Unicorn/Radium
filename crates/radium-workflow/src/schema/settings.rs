//! Workflow settings definitions

use serde::{Deserialize, Serialize};

use super::node::RetryPolicy;

/// Long-running workflow configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LongRunningConfig {
    pub auto_continue_as_new: bool,
    #[serde(default = "default_max_history_events")]
    pub max_history_events: u32,
    #[serde(default = "default_max_duration_ms")]
    pub max_duration_ms: u64,
    #[serde(default = "default_preserve_state")]
    pub preserve_state: bool,
}

fn default_max_history_events() -> u32 {
    10000
}

fn default_max_duration_ms() -> u64 {
    24 * 60 * 60 * 1000 // 24 hours
}

fn default_preserve_state() -> bool {
    true
}

/// Workflow type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum WorkflowType {
    #[default]
    Task,
    Service,
}

/// Workflow settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_policy: Option<RetryPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_queue: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Long-running workflow configuration
    #[serde(rename = "_longRunning", skip_serializing_if = "Option::is_none")]
    pub long_running: Option<LongRunningConfig>,
    /// Workflow type (task or service)
    #[serde(rename = "_workflowType", skip_serializing_if = "Option::is_none")]
    pub workflow_type: Option<WorkflowType>,
}

impl WorkflowSettings {
    /// Check if this is a long-running workflow
    pub fn is_long_running(&self) -> bool {
        self.long_running
            .as_ref()
            .map(|lr| lr.auto_continue_as_new)
            .unwrap_or(false)
    }

    /// Get the default timeout
    pub fn default_timeout(&self) -> &str {
        self.timeout.as_deref().unwrap_or("1m")
    }

    /// Get the task queue
    pub fn task_queue(&self) -> &str {
        self.task_queue.as_deref().unwrap_or("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_defaults() {
        let settings = WorkflowSettings::default();
        assert_eq!(settings.default_timeout(), "1m");
        assert_eq!(settings.task_queue(), "default");
        assert!(!settings.is_long_running());
    }

    #[test]
    fn test_long_running_settings() {
        let json = r#"{
            "timeout": "5m",
            "taskQueue": "my-queue",
            "_longRunning": {
                "autoContinueAsNew": true,
                "maxHistoryEvents": 5000,
                "maxDurationMs": 3600000,
                "preserveState": true
            }
        }"#;

        let settings: WorkflowSettings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.timeout, Some("5m".to_string()));
        assert!(settings.is_long_running());
        assert_eq!(
            settings.long_running.as_ref().unwrap().max_history_events,
            5000
        );
    }

    #[test]
    fn test_workflow_type() {
        let json = r#"{"_workflowType": "service"}"#;
        let settings: WorkflowSettings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.workflow_type, Some(WorkflowType::Service));
    }
}
