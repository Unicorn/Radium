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
    use crate::agents::analytics::AgentAnalyticsService;

    fn create_test_collector(enabled: bool) -> AgentTelemetryCollector {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::monitoring::schema::initialize_schema(&conn).unwrap();
        let analytics = AgentAnalyticsService::new(conn);
        let config = TelemetryConfig {
            enabled,
            retention_days: 90,
        };
        AgentTelemetryCollector::new(analytics, config)
    }

    #[test]
    fn test_telemetry_collector_disabled() {
        let collector = create_test_collector(false);
        let tracker = collector.record_start("test-agent", None).unwrap();
        assert!(!tracker.enabled);
    }

    #[test]
    fn test_telemetry_collector_enabled() {
        let collector = create_test_collector(true);
        let tracker = collector.record_start("test-agent", None).unwrap();
        assert!(tracker.enabled);
        assert_eq!(tracker.agent_id, "test-agent");
        assert_eq!(tracker.tokens, 0);
    }

    #[test]
    fn test_telemetry_collector_with_category() {
        let collector = create_test_collector(true);
        let tracker = collector.record_start("test-agent", Some("test-category")).unwrap();
        assert_eq!(tracker.category, Some("test-category".to_string()));
    }

    #[test]
    fn test_telemetry_collector_record_completion_success() {
        let collector = create_test_collector(true);
        let mut tracker = collector.record_start("test-agent", None).unwrap();
        tracker.record_tokens(100);
        
        // Small delay to ensure duration > 0
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let result = collector.record_completion(tracker, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_telemetry_collector_record_completion_failure() {
        let collector = create_test_collector(true);
        let mut tracker = collector.record_start("test-agent", None).unwrap();
        tracker.record_tokens(50);
        
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let result = collector.record_completion(tracker, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_telemetry_collector_record_completion_disabled_tracker() {
        let collector = create_test_collector(true);
        let tracker = collector.record_start("test-agent", None).unwrap();
        // Manually disable tracker
        let mut disabled_tracker = ExecutionTracker {
            agent_id: tracker.agent_id,
            category: tracker.category,
            start_time: tracker.start_time,
            enabled: false,
            tokens: tracker.tokens,
        };
        
        let result = collector.record_completion(disabled_tracker, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_telemetry_collector_record_completion_when_disabled() {
        let collector = create_test_collector(false);
        let tracker = collector.record_start("test-agent", None).unwrap();
        let result = collector.record_completion(tracker, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_telemetry_collector_cleanup_old_data_enabled() {
        let collector = create_test_collector(true);
        let result = collector.cleanup_old_data();
        assert!(result.is_ok());
    }

    #[test]
    fn test_telemetry_collector_cleanup_old_data_disabled() {
        let collector = create_test_collector(false);
        let result = collector.cleanup_old_data();
        assert!(result.is_ok());
    }

    #[test]
    fn test_execution_tracker_disabled() {
        let tracker = ExecutionTracker::disabled();
        assert!(!tracker.enabled);
        assert!(tracker.agent_id.is_empty());
        assert!(tracker.category.is_none());
        assert_eq!(tracker.tokens, 0);
    }

    #[test]
    fn test_execution_tracker_record_tokens() {
        let mut tracker = ExecutionTracker {
            agent_id: "test".to_string(),
            category: None,
            start_time: Instant::now(),
            enabled: true,
            tokens: 0,
        };
        
        tracker.record_tokens(500);
        assert_eq!(tracker.tokens, 500);
        
        tracker.record_tokens(1000);
        assert_eq!(tracker.tokens, 1000);
    }

    #[test]
    fn test_telemetry_config_default() {
        let config = TelemetryConfig::default();
        assert!(config.enabled);
        assert_eq!(config.retention_days, 90);
    }

    #[test]
    fn test_telemetry_config_custom() {
        let config = TelemetryConfig {
            enabled: false,
            retention_days: 30,
        };
        assert!(!config.enabled);
        assert_eq!(config.retention_days, 30);
    }

    #[test]
    fn test_telemetry_error_display() {
        let error = TelemetryError::Database(rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT),
            None,
        ));
        let msg = format!("{}", error);
        assert!(msg.contains("database error"));
    }

    #[test]
    fn test_telemetry_error_from_analytics_error() {
        let analytics_error = crate::agents::analytics::AnalyticsError::InvalidQuery("test".to_string());
        let telemetry_error: TelemetryError = analytics_error.into();
        assert!(matches!(telemetry_error, TelemetryError::Analytics(_)));
    }

    #[test]
    fn test_telemetry_error_from_database_error() {
        let db_error = rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT),
            None,
        );
        let telemetry_error: TelemetryError = db_error.into();
        assert!(matches!(telemetry_error, TelemetryError::Database(_)));
    }
}

