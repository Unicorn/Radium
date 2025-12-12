//! Policy analytics for tracking enforcement patterns and trends.

use super::storage::{PolicyAnalyticsStorage, PolicyEvent};
use super::types::PolicyDecision;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

/// Policy analytics manager.
pub struct PolicyAnalytics {
    storage: PolicyAnalyticsStorage,
}

impl PolicyAnalytics {
    /// Creates a new policy analytics instance.
    ///
    /// # Arguments
    /// * `conn` - SQLite connection (shared)
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        let storage = PolicyAnalyticsStorage::new(conn);
        Self { storage }
    }

    /// Records a policy evaluation event.
    ///
    /// # Arguments
    /// * `decision` - The policy decision
    /// * `tool_name` - The tool that was evaluated
    /// * `args` - Arguments passed to the tool
    /// * `user` - Optional user/agent identifier
    pub fn record_event(
        &self,
        decision: &PolicyDecision,
        tool_name: &str,
        args: &[&str],
        user: Option<&str>,
    ) {
        let arguments_json = serde_json::to_string(args).unwrap_or_else(|_| "[]".to_string());
        let action = format!("{:?}", decision.action).to_lowercase();

        let event = PolicyEvent {
            id: None,
            timestamp: chrono::Utc::now().timestamp(),
            tool_name: tool_name.to_string(),
            arguments: arguments_json,
            action,
            matched_rule: decision.matched_rule.clone(),
            reason: decision.reason.clone(),
            user: user.map(|s| s.to_string()),
        };

        if let Err(e) = self.storage.store_event(&event) {
            tracing::warn!(error = %e, "Failed to store policy analytics event");
        }
    }

    /// Gets violation trends for the last N days.
    ///
    /// # Arguments
    /// * `days` - Number of days to look back
    ///
    /// # Returns
    /// Vector of (date, violation_count) tuples
    pub fn get_violation_trends(&self, days: i64) -> Result<Vec<(String, i64)>, rusqlite::Error> {
        self.storage.get_violation_trends(days)
    }

    /// Gets rule effectiveness metrics.
    ///
    /// # Returns
    /// Vector of (rule_name, total_evaluations, allow_count, deny_count, ask_count, dry_run_count)
    pub fn get_rule_metrics(&self) -> Result<Vec<(String, i64, i64, i64, i64, i64)>, rusqlite::Error> {
        self.storage.get_rule_metrics()
    }

    /// Exports analytics data as JSON.
    ///
    /// # Arguments
    /// * `start_timestamp` - Optional start timestamp (Unix timestamp)
    /// * `end_timestamp` - Optional end timestamp (Unix timestamp)
    pub fn export_json(&self, start_timestamp: Option<i64>, end_timestamp: Option<i64>) -> Result<String, rusqlite::Error> {
        self.storage.export_json(start_timestamp, end_timestamp)
    }

    /// Exports analytics data as CSV.
    ///
    /// # Arguments
    /// * `start_timestamp` - Optional start timestamp (Unix timestamp)
    /// * `end_timestamp` - Optional end timestamp (Unix timestamp)
    pub fn export_csv(&self, start_timestamp: Option<i64>, end_timestamp: Option<i64>) -> Result<String, rusqlite::Error> {
        self.storage.export_csv(start_timestamp, end_timestamp)
    }
}

