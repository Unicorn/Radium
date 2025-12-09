//! Cost analytics query module for cost_events table.
//!
//! Provides query and aggregation capabilities for historical cost data
//! stored in the cost_events table.

use crate::monitoring::{MonitoringService, Result as MonitoringResult};
use chrono::{DateTime, Utc};
use rusqlite::params;
use std::collections::HashMap;

/// Represents a single cost event from the database.
#[derive(Debug, Clone)]
pub struct CostEvent {
    /// Timestamp of the cost event (Unix epoch seconds).
    pub timestamp: i64,
    /// Requirement ID (e.g., "REQ-197").
    pub requirement_id: Option<String>,
    /// Model name.
    pub model: Option<String>,
    /// Provider name.
    pub provider: Option<String>,
    /// Input tokens.
    pub tokens_input: u64,
    /// Output tokens.
    pub tokens_output: u64,
    /// Cost in USD.
    pub cost_usd: f64,
    /// Session ID.
    pub session_id: Option<String>,
}

/// Date range for filtering queries.
#[derive(Debug, Clone)]
pub struct DateRange {
    /// Start timestamp (Unix epoch seconds).
    pub start: i64,
    /// End timestamp (Unix epoch seconds).
    pub end: i64,
}

impl DateRange {
    /// Creates a new date range.
    pub fn new(start: i64, end: i64) -> Self {
        Self { start, end }
    }

    /// Creates a date range for the last N days.
    pub fn last_days(days: u32) -> Self {
        let end = Utc::now().timestamp();
        let start = end - (days as i64 * 86400);
        Self { start, end }
    }

    /// Creates a date range for this month.
    pub fn this_month() -> Self {
        let now = Utc::now();
        // Get the date and set to first of month at midnight
        let start = now
            .date_naive()
            .with_day(1)
            .and_then(|d| d.and_hms_opt(0, 0, 0))
            .map(|dt| dt.and_utc().timestamp())
            .unwrap_or_else(|| Utc::now().timestamp() - 2592000); // Fallback to 30 days ago
        let end = Utc::now().timestamp();
        Self { start, end }
    }
}

/// Time period for grouping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimePeriod {
    /// Group by day.
    Day,
    /// Group by week.
    Week,
    /// Group by month.
    Month,
}

/// Cost breakdown for grouped queries.
#[derive(Debug, Clone)]
pub struct CostBreakdown {
    /// Group key (requirement_id, model, provider, or time period).
    pub key: String,
    /// Total cost in USD.
    pub total_cost: f64,
    /// Total input tokens.
    pub total_tokens_input: u64,
    /// Total output tokens.
    pub total_tokens_output: u64,
    /// Total tokens (input + output).
    pub total_tokens: u64,
    /// Number of events.
    pub event_count: u64,
}

/// Cost summary for a date range.
#[derive(Debug, Clone)]
pub struct CostSummary {
    /// Date range.
    pub range: DateRange,
    /// Total cost in USD.
    pub total_cost: f64,
    /// Total input tokens.
    pub total_tokens_input: u64,
    /// Total output tokens.
    pub total_tokens_output: u64,
    /// Total tokens.
    pub total_tokens: u64,
    /// Number of events.
    pub event_count: u64,
    /// Breakdown by requirement.
    pub breakdown_by_requirement: Vec<CostBreakdown>,
    /// Breakdown by provider.
    pub breakdown_by_provider: Vec<CostBreakdown>,
    /// Breakdown by model.
    pub breakdown_by_model: Vec<CostBreakdown>,
}

/// Service for querying cost analytics from cost_events table.
pub struct CostAnalytics<'a> {
    /// Reference to monitoring service for database access.
    monitoring: &'a MonitoringService,
}

impl<'a> CostAnalytics<'a> {
    /// Create a new cost analytics service.
    pub fn new(monitoring: &'a MonitoringService) -> Self {
        Self { monitoring }
    }

    /// Query cost events by date range.
    ///
    /// # Arguments
    /// * `range` - Date range to query
    ///
    /// # Returns
    /// Vector of cost events in the date range
    pub fn query_by_date_range(&self, range: &DateRange) -> MonitoringResult<Vec<CostEvent>> {
        let conn = self.monitoring.conn();

        let mut stmt = conn.prepare(
            "SELECT timestamp, requirement_id, model, provider, tokens_input, tokens_output, cost_usd, session_id
             FROM cost_events
             WHERE timestamp >= ?1 AND timestamp <= ?2
             ORDER BY timestamp DESC",
        )?;

        let events = stmt
            .query_map(params![range.start, range.end], |row| {
                Ok(CostEvent {
                    timestamp: row.get(0)?,
                    requirement_id: row.get(1)?,
                    model: row.get(2)?,
                    provider: row.get(3)?,
                    tokens_input: row.get(4)?,
                    tokens_output: row.get(5)?,
                    cost_usd: row.get(6)?,
                    session_id: row.get(7)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(events)
    }

    /// Group cost events by requirement.
    ///
    /// # Arguments
    /// * `range` - Date range to query
    ///
    /// # Returns
    /// Vector of cost breakdowns grouped by requirement
    pub fn group_by_requirement(&self, range: &DateRange) -> MonitoringResult<Vec<CostBreakdown>> {
        let conn = self.monitoring.conn();

        let mut stmt = conn.prepare(
            "SELECT 
                COALESCE(requirement_id, 'Unknown') as key,
                SUM(cost_usd) as total_cost,
                SUM(tokens_input) as total_tokens_input,
                SUM(tokens_output) as total_tokens_output,
                SUM(tokens_input + tokens_output) as total_tokens,
                COUNT(*) as event_count
             FROM cost_events
             WHERE timestamp >= ?1 AND timestamp <= ?2
             GROUP BY requirement_id
             ORDER BY total_cost DESC",
        )?;

        let breakdowns = stmt
            .query_map(params![range.start, range.end], |row| {
                Ok(CostBreakdown {
                    key: row.get(0)?,
                    total_cost: row.get(1)?,
                    total_tokens_input: row.get(2)?,
                    total_tokens_output: row.get(3)?,
                    total_tokens: row.get(4)?,
                    event_count: row.get(5)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(breakdowns)
    }

    /// Group cost events by model.
    ///
    /// # Arguments
    /// * `range` - Date range to query
    ///
    /// # Returns
    /// Vector of cost breakdowns grouped by model
    pub fn group_by_model(&self, range: &DateRange) -> MonitoringResult<Vec<CostBreakdown>> {
        let conn = self.monitoring.conn();

        let mut stmt = conn.prepare(
            "SELECT 
                COALESCE(model, 'Unknown') as key,
                SUM(cost_usd) as total_cost,
                SUM(tokens_input) as total_tokens_input,
                SUM(tokens_output) as total_tokens_output,
                SUM(tokens_input + tokens_output) as total_tokens,
                COUNT(*) as event_count
             FROM cost_events
             WHERE timestamp >= ?1 AND timestamp <= ?2
             GROUP BY model
             ORDER BY total_cost DESC",
        )?;

        let breakdowns = stmt
            .query_map(params![range.start, range.end], |row| {
                Ok(CostBreakdown {
                    key: row.get(0)?,
                    total_cost: row.get(1)?,
                    total_tokens_input: row.get(2)?,
                    total_tokens_output: row.get(3)?,
                    total_tokens: row.get(4)?,
                    event_count: row.get(5)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(breakdowns)
    }

    /// Group cost events by provider.
    ///
    /// # Arguments
    /// * `range` - Date range to query
    ///
    /// # Returns
    /// Vector of cost breakdowns grouped by provider
    pub fn group_by_provider(&self, range: &DateRange) -> MonitoringResult<Vec<CostBreakdown>> {
        let conn = self.monitoring.conn();

        let mut stmt = conn.prepare(
            "SELECT 
                COALESCE(provider, 'Unknown') as key,
                SUM(cost_usd) as total_cost,
                SUM(tokens_input) as total_tokens_input,
                SUM(tokens_output) as total_tokens_output,
                SUM(tokens_input + tokens_output) as total_tokens,
                COUNT(*) as event_count
             FROM cost_events
             WHERE timestamp >= ?1 AND timestamp <= ?2
             GROUP BY provider
             ORDER BY total_cost DESC",
        )?;

        let breakdowns = stmt
            .query_map(params![range.start, range.end], |row| {
                Ok(CostBreakdown {
                    key: row.get(0)?,
                    total_cost: row.get(1)?,
                    total_tokens_input: row.get(2)?,
                    total_tokens_output: row.get(3)?,
                    total_tokens: row.get(4)?,
                    event_count: row.get(5)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(breakdowns)
    }

    /// Group cost events by time period.
    ///
    /// # Arguments
    /// * `range` - Date range to query
    /// * `period` - Time period to group by (day, week, month)
    ///
    /// # Returns
    /// Vector of cost breakdowns grouped by time period
    pub fn group_by_time_period(
        &self,
        range: &DateRange,
        period: TimePeriod,
    ) -> MonitoringResult<Vec<CostBreakdown>> {
        let conn = self.monitoring.conn();

        let time_expr = match period {
            TimePeriod::Day => "strftime('%Y-%m-%d', datetime(timestamp, 'unixepoch'))",
            TimePeriod::Week => "strftime('%Y-W%W', datetime(timestamp, 'unixepoch'))",
            TimePeriod::Month => "strftime('%Y-%m', datetime(timestamp, 'unixepoch'))",
        };

        let query = format!(
            "SELECT 
                {} as key,
                SUM(cost_usd) as total_cost,
                SUM(tokens_input) as total_tokens_input,
                SUM(tokens_output) as total_tokens_output,
                SUM(tokens_input + tokens_output) as total_tokens,
                COUNT(*) as event_count
             FROM cost_events
             WHERE timestamp >= ?1 AND timestamp <= ?2
             GROUP BY key
             ORDER BY key ASC",
            time_expr
        );

        let mut stmt = conn.prepare(&query)?;

        let breakdowns = stmt
            .query_map(params![range.start, range.end], |row| {
                Ok(CostBreakdown {
                    key: row.get(0)?,
                    total_cost: row.get(1)?,
                    total_tokens_input: row.get(2)?,
                    total_tokens_output: row.get(3)?,
                    total_tokens: row.get(4)?,
                    event_count: row.get(5)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(breakdowns)
    }

    /// Get top N most expensive requirements.
    ///
    /// # Arguments
    /// * `limit` - Maximum number of results
    /// * `range` - Date range to query
    ///
    /// # Returns
    /// Vector of cost breakdowns for top requirements
    pub fn top_expensive_requirements(
        &self,
        limit: usize,
        range: &DateRange,
    ) -> MonitoringResult<Vec<CostBreakdown>> {
        let conn = self.monitoring.conn();

        let mut stmt = conn.prepare(
            "SELECT 
                COALESCE(requirement_id, 'Unknown') as key,
                SUM(cost_usd) as total_cost,
                SUM(tokens_input) as total_tokens_input,
                SUM(tokens_output) as total_tokens_output,
                SUM(tokens_input + tokens_output) as total_tokens,
                COUNT(*) as event_count
             FROM cost_events
             WHERE timestamp >= ?1 AND timestamp <= ?2
             GROUP BY requirement_id
             ORDER BY total_cost DESC
             LIMIT ?3",
        )?;

        let breakdowns = stmt
            .query_map(params![range.start, range.end, limit as i64], |row| {
                Ok(CostBreakdown {
                    key: row.get(0)?,
                    total_cost: row.get(1)?,
                    total_tokens_input: row.get(2)?,
                    total_tokens_output: row.get(3)?,
                    total_tokens: row.get(4)?,
                    event_count: row.get(5)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(breakdowns)
    }

    /// Get total cost summary for a date range.
    ///
    /// # Arguments
    /// * `range` - Date range to query
    ///
    /// # Returns
    /// Cost summary with totals and breakdowns
    pub fn total_cost_summary(&self, range: &DateRange) -> MonitoringResult<CostSummary> {
        let conn = self.monitoring.conn();

        // Get totals
        let mut stmt = conn.prepare(
            "SELECT 
                SUM(cost_usd) as total_cost,
                SUM(tokens_input) as total_tokens_input,
                SUM(tokens_output) as total_tokens_output,
                SUM(tokens_input + tokens_output) as total_tokens,
                COUNT(*) as event_count
             FROM cost_events
             WHERE timestamp >= ?1 AND timestamp <= ?2",
        )?;

        let (total_cost, total_tokens_input, total_tokens_output, total_tokens, event_count) =
            stmt.query_row(params![range.start, range.end], |row| {
                Ok((
                    row.get::<_, Option<f64>>(0)?.unwrap_or(0.0),
                    row.get::<_, Option<u64>>(1)?.unwrap_or(0),
                    row.get::<_, Option<u64>>(2)?.unwrap_or(0),
                    row.get::<_, Option<u64>>(3)?.unwrap_or(0),
                    row.get::<_, Option<u64>>(4)?.unwrap_or(0),
                ))
            })?;

        // Get breakdowns
        let breakdown_by_requirement = self.group_by_requirement(range)?;
        let breakdown_by_provider = self.group_by_provider(range)?;
        let breakdown_by_model = self.group_by_model(range)?;

        Ok(CostSummary {
            range: range.clone(),
            total_cost,
            total_tokens_input,
            total_tokens_output,
            total_tokens,
            event_count,
            breakdown_by_requirement,
            breakdown_by_provider,
            breakdown_by_model,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitoring::{AgentRecord, AgentStatus, MonitoringService, TelemetryRecord};
    use std::path::PathBuf;
    use tempfile::TempDir;

    async fn setup_test_service() -> (MonitoringService, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let monitoring = MonitoringService::open(&db_path).unwrap();
        (monitoring, temp_dir)
    }

    #[tokio::test]
    async fn test_query_by_date_range() {
        let (monitoring, _temp) = setup_test_service().await;
        let analytics = CostAnalytics::new(&monitoring);

        // Create agent with plan_id
        let mut agent = AgentRecord::new("agent-1".to_string(), "test".to_string());
        agent.plan_id = Some("REQ-197".to_string());
        monitoring.register_agent(&agent).unwrap();

        // Create telemetry (which will also create cost_events)
        let telemetry = TelemetryRecord::new("agent-1".to_string())
            .with_tokens(1000, 500)
            .with_model("gpt-4".to_string(), "openai".to_string());
        monitoring.record_telemetry(&telemetry).await.unwrap();

        // Query cost events
        let now = Utc::now().timestamp();
        let range = DateRange::new(now - 86400, now + 86400);
        let events = analytics.query_by_date_range(&range).unwrap();

        assert!(!events.is_empty());
        assert_eq!(events[0].requirement_id, Some("REQ-197".to_string()));
    }

    #[tokio::test]
    async fn test_group_by_requirement() {
        let (monitoring, _temp) = setup_test_service().await;
        let analytics = CostAnalytics::new(&monitoring);

        // Create agents with different plan_ids
        let mut agent1 = AgentRecord::new("agent-1".to_string(), "test".to_string());
        agent1.plan_id = Some("REQ-197".to_string());
        let mut agent2 = AgentRecord::new("agent-2".to_string(), "test".to_string());
        agent2.plan_id = Some("REQ-198".to_string());
        monitoring.register_agent(&agent1).unwrap();
        monitoring.register_agent(&agent2).unwrap();

        // Create telemetry for both
        let telemetry1 = TelemetryRecord::new("agent-1".to_string())
            .with_tokens(1000, 500)
            .with_model("gpt-4".to_string(), "openai".to_string());
        let telemetry2 = TelemetryRecord::new("agent-2".to_string())
            .with_tokens(2000, 1000)
            .with_model("gpt-4".to_string(), "openai".to_string());
        monitoring.record_telemetry(&telemetry1).await.unwrap();
        monitoring.record_telemetry(&telemetry2).await.unwrap();

        let now = Utc::now().timestamp();
        let range = DateRange::new(now - 86400, now + 86400);
        let breakdowns = analytics.group_by_requirement(&range).unwrap();

        assert_eq!(breakdowns.len(), 2);
        assert!(breakdowns.iter().any(|b| b.key == "REQ-197"));
        assert!(breakdowns.iter().any(|b| b.key == "REQ-198"));
    }

    #[tokio::test]
    async fn test_top_expensive_requirements() {
        let (monitoring, _temp) = setup_test_service().await;
        let analytics = CostAnalytics::new(&monitoring);

        // Create agents with different plan_ids
        let mut agent1 = AgentRecord::new("agent-1".to_string(), "test".to_string());
        agent1.plan_id = Some("REQ-197".to_string());
        let mut agent2 = AgentRecord::new("agent-2".to_string(), "test".to_string());
        agent2.plan_id = Some("REQ-198".to_string());
        monitoring.register_agent(&agent1).unwrap();
        monitoring.register_agent(&agent2).unwrap();

        // Create telemetry - REQ-198 should be more expensive
        let telemetry1 = TelemetryRecord::new("agent-1".to_string())
            .with_tokens(1000, 500)
            .with_model("gpt-4".to_string(), "openai".to_string());
        let telemetry2 = TelemetryRecord::new("agent-2".to_string())
            .with_tokens(2000, 1000)
            .with_model("gpt-4".to_string(), "openai".to_string());
        monitoring.record_telemetry(&telemetry1).await.unwrap();
        monitoring.record_telemetry(&telemetry2).await.unwrap();

        let now = Utc::now().timestamp();
        let range = DateRange::new(now - 86400, now + 86400);
        let top = analytics.top_expensive_requirements(1, &range).unwrap();

        assert_eq!(top.len(), 1);
        assert_eq!(top[0].key, "REQ-198");
    }
}

