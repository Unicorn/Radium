//! Agent-specific telemetry collection.
//!
//! Provides privacy-preserving telemetry collection for agent usage.

use crate::agents::analytics::{AgentAnalyticsService, UsageRecord};
use std::time::Instant;
use thiserror::Error;

/// Telemetry errors.
#[derive(Debug, Error)]
pub enum TelemetryError {
    /// Analytics error.
    #[error("analytics error: {0}")]
    Analytics(#[from] crate::agents::analytics::AnalyticsError),

    /// Database error.
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
}

/// Result type for telemetry operations.
pub type Result<T> = std::result::Result<T, TelemetryError>;

/// Telemetry collection configuration.
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    /// Whether telemetry collection is enabled.
    pub enabled: bool,
    /// Data retention period in days.
    pub retention_days: u32,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            retention_days: 90,
        }
    }
}

/// Agent telemetry collector.
pub struct AgentTelemetryCollector {
    analytics: AgentAnalyticsService,
    config: TelemetryConfig,
}

impl AgentTelemetryCollector {
    /// Creates a new telemetry collector.
    pub fn new(
        analytics: AgentAnalyticsService,
        config: TelemetryConfig,
    ) -> Self {
        Self {
            analytics,
            config,
        }
    }

    /// Records agent execution start.
    pub fn record_start(&self, agent_id: &str, category: Option<&str>) -> Result<ExecutionTracker> {
        if !self.config.enabled {
            return Ok(ExecutionTracker::disabled());
        }

        Ok(ExecutionTracker {
            agent_id: agent_id.to_string(),
            category: category.map(|s| s.to_string()),
            start_time: Instant::now(),
            enabled: true,
            tokens: 0,
        })
    }

    /// Records agent execution completion.
    pub fn record_completion(
        &self,
        tracker: ExecutionTracker,
        success: bool,
    ) -> Result<()> {
        if !self.config.enabled || !tracker.enabled {
            return Ok(());
        }

        let duration = tracker.start_time.elapsed();
        let duration_ms = duration.as_millis() as u64;

        let record = UsageRecord {
            agent_id: tracker.agent_id,
            duration_ms,
            tokens: tracker.tokens,
            success,
            category: tracker.category,
        };

        self.analytics.record_usage(&record)?;

        Ok(())
    }

    /// Cleans up old telemetry data based on retention policy.
    pub fn cleanup_old_data(&self) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        // This would delete old records based on retention_days
        // For now, it's a placeholder
        Ok(())
    }
}

/// Execution tracker for recording agent execution metrics.
#[derive(Debug, Clone)]
pub struct ExecutionTracker {
    agent_id: String,
    category: Option<String>,
    start_time: Instant,
    enabled: bool,
    tokens: u64,
}

impl ExecutionTracker {
    /// Creates a disabled tracker (for when telemetry is off).
    fn disabled() -> Self {
        Self {
            agent_id: String::new(),
            category: None,
            start_time: Instant::now(),
            enabled: false,
            tokens: 0,
        }
    }

    /// Records token usage for this execution.
    pub fn record_tokens(&mut self, tokens: u64) {
        self.tokens = tokens;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitoring::MonitoringService;
    use rusqlite::Connection;

    #[test]
    fn test_telemetry_collector_disabled() {
        use rusqlite::Connection;
        let conn = Connection::open_in_memory().unwrap();
        crate::monitoring::initialize_schema(&conn).unwrap();
        let analytics = AgentAnalyticsService::new(conn);
        let config = TelemetryConfig {
            enabled: false,
            retention_days: 90,
        };

        let collector = AgentTelemetryCollector::new(analytics, config);
        let tracker = collector.record_start("test-agent", None).unwrap();
        assert!(!tracker.enabled);
    }
}

