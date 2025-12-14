//! Trigger component schema
//!
//! The Trigger component defines how a workflow is initiated.
//! It supports multiple trigger types: manual, schedule, webhook, event, and signal.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Types of workflow triggers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum TriggerType {
    /// Manually triggered workflow
    #[default]
    Manual,
    /// Scheduled execution (cron or interval)
    Schedule,
    /// Webhook-triggered workflow
    Webhook,
    /// Event-driven trigger
    Event,
    /// Signal from another workflow
    Signal,
}

impl TriggerType {
    /// Convert to TypeScript string representation
    pub fn to_typescript(&self) -> &'static str {
        match self {
            TriggerType::Manual => "'manual'",
            TriggerType::Schedule => "'schedule'",
            TriggerType::Webhook => "'webhook'",
            TriggerType::Event => "'event'",
            TriggerType::Signal => "'signal'",
        }
    }
}

/// Schedule configuration for scheduled triggers
#[derive(Debug, Clone, Serialize, Deserialize, Default, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleConfig {
    /// Cron expression (e.g., "0 0 * * *" for daily at midnight)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cron: Option<String>,

    /// Interval in seconds between executions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval_seconds: Option<u64>,

    /// Timezone for schedule interpretation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
}

impl ScheduleConfig {
    /// Create a cron-based schedule
    pub fn cron(expression: impl Into<String>) -> Self {
        Self {
            cron: Some(expression.into()),
            interval_seconds: None,
            timezone: None,
        }
    }

    /// Create an interval-based schedule
    pub fn interval(seconds: u64) -> Self {
        Self {
            cron: None,
            interval_seconds: Some(seconds),
            timezone: None,
        }
    }

    /// Set timezone
    pub fn with_timezone(mut self, timezone: impl Into<String>) -> Self {
        self.timezone = Some(timezone.into());
        self
    }

    /// Validate schedule configuration
    pub fn validate_config(&self) -> Result<(), String> {
        if self.cron.is_none() && self.interval_seconds.is_none() {
            return Err("Schedule requires either cron or interval_seconds".to_string());
        }
        if self.cron.is_some() && self.interval_seconds.is_some() {
            return Err("Schedule cannot have both cron and interval_seconds".to_string());
        }
        Ok(())
    }
}

/// Webhook configuration for webhook triggers
#[derive(Debug, Clone, Serialize, Deserialize, Default, Validate)]
#[serde(rename_all = "camelCase")]
pub struct WebhookConfig {
    /// Custom path for the webhook (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Allowed HTTP methods
    #[serde(default)]
    pub methods: Vec<String>,

    /// Authentication type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication: Option<String>,

    /// Secret for signature validation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
}

impl WebhookConfig {
    /// Create a new webhook config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set path
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Add allowed method
    pub fn with_method(mut self, method: impl Into<String>) -> Self {
        self.methods.push(method.into());
        self
    }

    /// Set authentication
    pub fn with_authentication(mut self, auth: impl Into<String>) -> Self {
        self.authentication = Some(auth.into());
        self
    }
}

/// Trigger component input
#[derive(Debug, Clone, Serialize, Deserialize, Default, Validate)]
#[serde(rename_all = "camelCase")]
pub struct TriggerInput {
    /// Type of trigger
    #[serde(default)]
    pub trigger_type: TriggerType,

    /// Schedule configuration (for schedule type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule: Option<ScheduleConfig>,

    /// Webhook configuration (for webhook type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<WebhookConfig>,

    /// Event type to listen for (for event type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_type: Option<String>,

    /// Signal name to listen for (for signal type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal_name: Option<String>,

    /// Initial payload passed to workflow
    #[serde(default)]
    pub payload: serde_json::Value,
}

impl TriggerInput {
    /// Create a manual trigger
    pub fn manual() -> Self {
        Self {
            trigger_type: TriggerType::Manual,
            ..Default::default()
        }
    }

    /// Create a scheduled trigger with cron
    pub fn scheduled_cron(cron: impl Into<String>) -> Self {
        Self {
            trigger_type: TriggerType::Schedule,
            schedule: Some(ScheduleConfig::cron(cron)),
            ..Default::default()
        }
    }

    /// Create a scheduled trigger with interval
    pub fn scheduled_interval(seconds: u64) -> Self {
        Self {
            trigger_type: TriggerType::Schedule,
            schedule: Some(ScheduleConfig::interval(seconds)),
            ..Default::default()
        }
    }

    /// Create a webhook trigger
    pub fn webhook() -> Self {
        Self {
            trigger_type: TriggerType::Webhook,
            webhook: Some(WebhookConfig::new()),
            ..Default::default()
        }
    }

    /// Create an event trigger
    pub fn event(event_type: impl Into<String>) -> Self {
        Self {
            trigger_type: TriggerType::Event,
            event_type: Some(event_type.into()),
            ..Default::default()
        }
    }

    /// Create a signal trigger
    pub fn signal(signal_name: impl Into<String>) -> Self {
        Self {
            trigger_type: TriggerType::Signal,
            signal_name: Some(signal_name.into()),
            ..Default::default()
        }
    }

    /// Set payload
    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = payload;
        self
    }

    /// Validate trigger configuration
    pub fn validate_config(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        match self.trigger_type {
            TriggerType::Schedule => {
                if self.schedule.is_none() {
                    errors.push("Schedule trigger requires schedule configuration".to_string());
                } else if let Some(ref schedule) = self.schedule {
                    if let Err(e) = schedule.validate_config() {
                        errors.push(e);
                    }
                }
            }
            TriggerType::Webhook => {
                if self.webhook.is_none() {
                    errors.push("Webhook trigger requires webhook configuration".to_string());
                }
            }
            TriggerType::Event => {
                if self.event_type.is_none() {
                    errors.push("Event trigger requires event_type".to_string());
                }
            }
            TriggerType::Signal => {
                if self.signal_name.is_none() {
                    errors.push("Signal trigger requires signal_name".to_string());
                }
            }
            TriggerType::Manual => {
                // No additional configuration required
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Trigger component output
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerOutput {
    /// Whether the trigger fired
    pub triggered: bool,

    /// Unique trigger ID
    pub trigger_id: String,

    /// When the trigger fired
    pub triggered_at: DateTime<Utc>,

    /// The payload received/generated
    pub payload: serde_json::Value,
}

impl TriggerOutput {
    /// Create a new trigger output
    pub fn new(trigger_id: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            triggered: true,
            trigger_id: trigger_id.into(),
            triggered_at: Utc::now(),
            payload,
        }
    }
}

impl Default for TriggerOutput {
    fn default() -> Self {
        Self {
            triggered: true,
            trigger_id: uuid::Uuid::new_v4().to_string(),
            triggered_at: Utc::now(),
            payload: serde_json::Value::Null,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigger_type_serialization() {
        assert_eq!(
            serde_json::to_string(&TriggerType::Manual).unwrap(),
            "\"manual\""
        );
        assert_eq!(
            serde_json::to_string(&TriggerType::Schedule).unwrap(),
            "\"schedule\""
        );
        assert_eq!(
            serde_json::to_string(&TriggerType::Webhook).unwrap(),
            "\"webhook\""
        );
    }

    #[test]
    fn test_trigger_type_typescript() {
        assert_eq!(TriggerType::Manual.to_typescript(), "'manual'");
        assert_eq!(TriggerType::Schedule.to_typescript(), "'schedule'");
    }

    #[test]
    fn test_schedule_config_cron() {
        let schedule = ScheduleConfig::cron("0 0 * * *").with_timezone("UTC");
        assert!(schedule.cron.is_some());
        assert!(schedule.interval_seconds.is_none());
        assert_eq!(schedule.timezone, Some("UTC".to_string()));
        assert!(schedule.validate_config().is_ok());
    }

    #[test]
    fn test_schedule_config_interval() {
        let schedule = ScheduleConfig::interval(3600);
        assert!(schedule.interval_seconds.is_some());
        assert!(schedule.cron.is_none());
        assert!(schedule.validate_config().is_ok());
    }

    #[test]
    fn test_schedule_config_invalid() {
        let schedule = ScheduleConfig::default();
        assert!(schedule.validate_config().is_err());

        let schedule = ScheduleConfig {
            cron: Some("0 0 * * *".to_string()),
            interval_seconds: Some(3600),
            timezone: None,
        };
        assert!(schedule.validate_config().is_err());
    }

    #[test]
    fn test_trigger_input_manual() {
        let trigger = TriggerInput::manual();
        assert_eq!(trigger.trigger_type, TriggerType::Manual);
        assert!(trigger.validate_config().is_ok());
    }

    #[test]
    fn test_trigger_input_scheduled() {
        let trigger = TriggerInput::scheduled_cron("0 0 * * *");
        assert_eq!(trigger.trigger_type, TriggerType::Schedule);
        assert!(trigger.schedule.is_some());
        assert!(trigger.validate_config().is_ok());
    }

    #[test]
    fn test_trigger_input_webhook() {
        let trigger = TriggerInput::webhook();
        assert_eq!(trigger.trigger_type, TriggerType::Webhook);
        assert!(trigger.webhook.is_some());
        assert!(trigger.validate_config().is_ok());
    }

    #[test]
    fn test_trigger_input_event() {
        let trigger = TriggerInput::event("user.created");
        assert_eq!(trigger.trigger_type, TriggerType::Event);
        assert_eq!(trigger.event_type, Some("user.created".to_string()));
        assert!(trigger.validate_config().is_ok());
    }

    #[test]
    fn test_trigger_input_signal() {
        let trigger = TriggerInput::signal("approval_received");
        assert_eq!(trigger.trigger_type, TriggerType::Signal);
        assert_eq!(trigger.signal_name, Some("approval_received".to_string()));
        assert!(trigger.validate_config().is_ok());
    }

    #[test]
    fn test_trigger_input_validation_errors() {
        let trigger = TriggerInput {
            trigger_type: TriggerType::Schedule,
            schedule: None,
            ..Default::default()
        };
        assert!(trigger.validate_config().is_err());

        let trigger = TriggerInput {
            trigger_type: TriggerType::Event,
            event_type: None,
            ..Default::default()
        };
        assert!(trigger.validate_config().is_err());
    }

    #[test]
    fn test_trigger_input_serialization() {
        let trigger = TriggerInput::scheduled_cron("0 0 * * *")
            .with_payload(serde_json::json!({"key": "value"}));

        let json = serde_json::to_string(&trigger).unwrap();
        assert!(json.contains("triggerType"));
        assert!(json.contains("schedule"));
        assert!(json.contains("cron"));
    }

    #[test]
    fn test_trigger_output() {
        let output = TriggerOutput::new("trigger-123", serde_json::json!({"data": "test"}));
        assert!(output.triggered);
        assert_eq!(output.trigger_id, "trigger-123");
    }

    #[test]
    fn test_trigger_output_serialization() {
        let output = TriggerOutput {
            triggered: true,
            trigger_id: "trigger-123".to_string(),
            triggered_at: DateTime::parse_from_rfc3339("2025-01-15T10:30:00Z")
                .unwrap()
                .with_timezone(&Utc),
            payload: serde_json::json!({}),
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("triggered"));
        assert!(json.contains("triggerId"));
        assert!(json.contains("triggeredAt"));
    }
}
