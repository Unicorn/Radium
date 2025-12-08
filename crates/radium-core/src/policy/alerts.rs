//! Real-time policy violation alerts with webhook support.

use super::types::{PolicyAction, PolicyDecision, PolicyError, PolicyResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{error, warn};

/// Alert severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    /// Informational alerts.
    Info,
    /// Warning-level alerts.
    Warning,
    /// Critical alerts requiring immediate attention.
    Critical,
}

/// Configuration for a webhook alert destination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    /// Webhook URL.
    pub url: String,
    /// Optional authentication token (Bearer token).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    /// Severity threshold - only send alerts at or above this level.
    #[serde(default = "default_severity")]
    pub min_severity: AlertSeverity,
}

fn default_severity() -> AlertSeverity {
    AlertSeverity::Warning
}

/// Alert configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// Webhook destinations.
    #[serde(default)]
    pub webhooks: Vec<WebhookConfig>,
    /// Maximum number of alerts per minute (rate limiting).
    #[serde(default = "default_rate_limit")]
    pub rate_limit_per_minute: u32,
    /// Enable/disable alerts.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_rate_limit() -> u32 {
    10
}

fn default_enabled() -> bool {
    true
}

/// Rate limiter using token bucket algorithm.
struct RateLimiter {
    /// Maximum tokens (alerts) allowed.
    capacity: u32,
    /// Current number of tokens available.
    tokens: u32,
    /// Last time tokens were replenished.
    last_refill: Instant,
    /// Refill interval (1 minute).
    refill_interval: Duration,
    /// Tokens to add per refill.
    refill_amount: u32,
}

impl RateLimiter {
    fn new(capacity: u32) -> Self {
        Self {
            capacity,
            tokens: capacity,
            last_refill: Instant::now(),
            refill_interval: Duration::from_secs(60),
            refill_amount: capacity,
        }
    }

    /// Try to consume a token. Returns true if allowed, false if rate limited.
    fn try_consume(&mut self) -> bool {
        self.refill();
        if self.tokens > 0 {
            self.tokens -= 1;
            true
        } else {
            false
        }
    }

    /// Refill tokens based on elapsed time.
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        
        if elapsed >= self.refill_interval {
            // Calculate how many full intervals have passed
            let intervals = elapsed.as_secs() / 60;
            if intervals > 0 {
                self.tokens = self.capacity.min(self.tokens + (intervals as u32 * self.refill_amount));
                self.last_refill = now;
            }
        }
    }
}

/// Alert payload sent to webhooks.
#[derive(Debug, Clone, Serialize)]
pub struct AlertPayload {
    /// Alert severity.
    pub severity: AlertSeverity,
    /// Timestamp of the violation.
    pub timestamp: String,
    /// Tool name that triggered the violation.
    pub tool_name: String,
    /// Arguments passed to the tool.
    pub arguments: Vec<String>,
    /// Policy action taken (deny, ask_user, etc.).
    pub action: String,
    /// Matched rule name (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_rule: Option<String>,
    /// Reason for the violation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Optional user/agent identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

/// Manager for policy violation alerts.
pub struct AlertManager {
    /// Alert configuration.
    config: AlertConfig,
    /// Rate limiter.
    rate_limiter: Arc<RwLock<RateLimiter>>,
    /// HTTP client for webhook calls.
    client: reqwest::Client,
}

impl AlertManager {
    /// Creates a new alert manager with the given configuration.
    pub fn new(config: AlertConfig) -> Self {
        let rate_limiter = RateLimiter::new(config.rate_limit_per_minute);
        Self {
            config,
            rate_limiter: Arc::new(RwLock::new(rate_limiter)),
            client: reqwest::Client::new(),
        }
    }

    /// Creates a default alert manager (disabled).
    pub fn disabled() -> Self {
        Self::new(AlertConfig {
            enabled: false,
            webhooks: Vec::new(),
            rate_limit_per_minute: 10,
        })
    }

    /// Sends an alert for a policy violation.
    ///
    /// # Arguments
    /// * `decision` - The policy decision that triggered the alert
    /// * `tool_name` - The tool that was evaluated
    /// * `args` - Arguments passed to the tool
    /// * `user` - Optional user/agent identifier
    pub async fn send_alert(
        &self,
        decision: &PolicyDecision,
        tool_name: &str,
        args: &[&str],
        user: Option<&str>,
    ) {
        if !self.config.enabled || self.config.webhooks.is_empty() {
            return;
        }

        // Check rate limiting
        {
            let mut limiter = self.rate_limiter.write().await;
            if !limiter.try_consume() {
                warn!("Alert rate limited - too many alerts in the last minute");
                return;
            }
        }

        // Determine severity based on action
        let severity = match decision.action {
            PolicyAction::Deny => AlertSeverity::Critical,
            PolicyAction::AskUser => AlertSeverity::Warning,
            PolicyAction::DryRunFirst => AlertSeverity::Info,
            PolicyAction::Allow => return, // Don't alert on allowed actions
        };

        // Create alert payload
        let payload = AlertPayload {
            severity,
            timestamp: chrono::Utc::now().to_rfc3339(),
            tool_name: tool_name.to_string(),
            arguments: args.iter().map(|s| s.to_string()).collect(),
            action: format!("{:?}", decision.action).to_lowercase(),
            matched_rule: decision.matched_rule.clone(),
            reason: decision.reason.clone(),
            user: user.map(|s| s.to_string()),
        };

        // Send to all configured webhooks
        for webhook in &self.config.webhooks {
            // Check severity threshold
            if !self.meets_severity_threshold(severity, webhook.min_severity) {
                continue;
            }

            if let Err(e) = self.send_webhook(webhook, &payload).await {
                error!(
                    webhook_url = %webhook.url,
                    error = %e,
                    "Failed to send alert webhook"
                );
            }
        }
    }

    /// Checks if severity meets the webhook's minimum threshold.
    fn meets_severity_threshold(&self, severity: AlertSeverity, min_severity: AlertSeverity) -> bool {
        match (severity, min_severity) {
            (AlertSeverity::Critical, _) => true,
            (AlertSeverity::Warning, AlertSeverity::Warning | AlertSeverity::Info) => true,
            (AlertSeverity::Info, AlertSeverity::Info) => true,
            _ => false,
        }
    }

    /// Sends a webhook notification.
    async fn send_webhook(&self, webhook: &WebhookConfig, payload: &AlertPayload) -> Result<(), reqwest::Error> {
        let mut request = self.client.post(&webhook.url).json(payload);

        // Add authentication token if provided
        if let Some(token) = &webhook.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;
        
        if !response.status().is_success() {
            return Err(reqwest::Error::from(response.error_for_status().unwrap_err()));
        }

        Ok(())
    }

    /// Gets the current alert configuration.
    pub fn config(&self) -> &AlertConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::types::PolicyAction;

    #[test]
    fn test_rate_limiter_consumes_tokens() {
        let mut limiter = RateLimiter::new(5);
        assert!(limiter.try_consume());
        assert!(limiter.try_consume());
        assert!(limiter.try_consume());
        assert!(limiter.try_consume());
        assert!(limiter.try_consume());
        assert!(!limiter.try_consume()); // Should be rate limited
    }

    #[test]
    fn test_alert_severity_threshold() {
        let manager = AlertManager::disabled();
        
        // Critical meets all thresholds
        assert!(manager.meets_severity_threshold(AlertSeverity::Critical, AlertSeverity::Info));
        assert!(manager.meets_severity_threshold(AlertSeverity::Critical, AlertSeverity::Warning));
        assert!(manager.meets_severity_threshold(AlertSeverity::Critical, AlertSeverity::Critical));
        
        // Warning meets Warning and Info
        assert!(manager.meets_severity_threshold(AlertSeverity::Warning, AlertSeverity::Info));
        assert!(manager.meets_severity_threshold(AlertSeverity::Warning, AlertSeverity::Warning));
        assert!(!manager.meets_severity_threshold(AlertSeverity::Warning, AlertSeverity::Critical));
        
        // Info only meets Info
        assert!(manager.meets_severity_threshold(AlertSeverity::Info, AlertSeverity::Info));
        assert!(!manager.meets_severity_threshold(AlertSeverity::Info, AlertSeverity::Warning));
        assert!(!manager.meets_severity_threshold(AlertSeverity::Info, AlertSeverity::Critical));
    }

    #[test]
    fn test_alert_payload_serialization() {
        let payload = AlertPayload {
            severity: AlertSeverity::Critical,
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            tool_name: "run_terminal_cmd".to_string(),
            arguments: vec!["rm".to_string(), "-rf".to_string(), "/".to_string()],
            action: "deny".to_string(),
            matched_rule: Some("deny-dangerous-commands".to_string()),
            reason: Some("Safety: prevent accidental deletion".to_string()),
            user: Some("agent-123".to_string()),
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("critical"));
        assert!(json.contains("run_terminal_cmd"));
        assert!(json.contains("deny-dangerous-commands"));
    }
}

