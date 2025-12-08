//! Cost data query service for filtering and aggregating telemetry data.

use crate::analytics::export::{CostRecord, CostSummary, ExportOptions, TierBreakdown, TierMetrics};
use crate::monitoring::{MonitoringService, Result as MonitoringResult};
use chrono::{DateTime, Utc};
use rusqlite::params;
use std::collections::HashMap;

/// Service for querying cost data from telemetry.
pub struct CostQueryService<'a> {
    /// Reference to monitoring service for database access.
    monitoring: &'a MonitoringService,
}

impl<'a> CostQueryService<'a> {
    /// Create a new cost query service.
    pub fn new(monitoring: &'a MonitoringService) -> Self {
        Self { monitoring }
    }

    /// Query cost records with filters applied.
    ///
    /// # Arguments
    /// * `options` - Export options containing filters
    ///
    /// # Returns
    /// Vector of cost records matching the filters
    pub fn query_records(&self, options: &ExportOptions) -> MonitoringResult<Vec<CostRecord>> {
        let conn = self.monitoring.conn();

        // Build query with JOIN to agents table for plan_id
        // Use a simpler approach: build query string and use params! macro with conditional values
        let mut query = String::from(
            "SELECT t.agent_id, t.timestamp, t.input_tokens, t.output_tokens, t.cached_tokens,
                    t.total_tokens, t.estimated_cost, t.model, t.provider, a.plan_id, t.model_tier
             FROM telemetry t
             LEFT JOIN agents a ON t.agent_id = a.id
             WHERE 1=1",
        );

        // Build parameters based on filters
        let start_ts = options.start_date.map(|d| d.timestamp() as i64);
        let end_ts = options.end_date.map(|d| d.timestamp() as i64);

        // Add filters to query
        if start_ts.is_some() {
            query.push_str(" AND t.timestamp >= ?1");
        }
        if end_ts.is_some() {
            let param_num = if start_ts.is_some() { "?2" } else { "?1" };
            query.push_str(&format!(" AND t.timestamp <= {}", param_num));
        }
        if options.plan_id.is_some() {
            let param_num = match (start_ts.is_some(), end_ts.is_some()) {
                (true, true) => "?3",
                (true, false) | (false, true) => "?2",
                _ => "?1",
            };
            query.push_str(&format!(" AND a.plan_id = {}", param_num));
        }
        if options.provider.is_some() {
            let param_num = match (
                start_ts.is_some(),
                end_ts.is_some(),
                options.plan_id.is_some(),
            ) {
                (true, true, true) => "?4",
                (true, true, false) | (true, false, true) | (false, true, true) => "?3",
                (true, false, false) | (false, true, false) | (false, false, true) => "?2",
                _ => "?1",
            };
            query.push_str(&format!(" AND t.provider = {}", param_num));
        }

        query.push_str(" ORDER BY t.timestamp DESC");

        // Execute query with appropriate params
        let records = match (
            start_ts,
            end_ts,
            &options.plan_id,
            &options.provider,
        ) {
            (Some(s), Some(e), Some(p), Some(pr)) => {
                let mut stmt = conn.prepare(&query)?;
                stmt.query_map(params![s, e, p, pr], |row| self.row_to_record(row))?
                    .collect::<std::result::Result<Vec<_>, _>>()?
            }
            (Some(s), Some(e), Some(p), None) => {
                let mut stmt = conn.prepare(&query)?;
                stmt.query_map(params![s, e, p], |row| self.row_to_record(row))?
                    .collect::<std::result::Result<Vec<_>, _>>()?
            }
            (Some(s), Some(e), None, Some(pr)) => {
                let mut stmt = conn.prepare(&query)?;
                stmt.query_map(params![s, e, pr], |row| self.row_to_record(row))?
                    .collect::<std::result::Result<Vec<_>, _>>()?
            }
            (Some(s), Some(e), None, None) => {
                let mut stmt = conn.prepare(&query)?;
                stmt.query_map(params![s, e], |row| self.row_to_record(row))?
                    .collect::<std::result::Result<Vec<_>, _>>()?
            }
            (Some(s), None, Some(p), Some(pr)) => {
                let mut stmt = conn.prepare(&query)?;
                stmt.query_map(params![s, p, pr], |row| self.row_to_record(row))?
                    .collect::<std::result::Result<Vec<_>, _>>()?
            }
            (Some(s), None, Some(p), None) => {
                let mut stmt = conn.prepare(&query)?;
                stmt.query_map(params![s, p], |row| self.row_to_record(row))?
                    .collect::<std::result::Result<Vec<_>, _>>()?
            }
            (Some(s), None, None, Some(pr)) => {
                let mut stmt = conn.prepare(&query)?;
                stmt.query_map(params![s, pr], |row| self.row_to_record(row))?
                    .collect::<std::result::Result<Vec<_>, _>>()?
            }
            (Some(s), None, None, None) => {
                let mut stmt = conn.prepare(&query)?;
                stmt.query_map(params![s], |row| self.row_to_record(row))?
                    .collect::<std::result::Result<Vec<_>, _>>()?
            }
            (None, Some(e), Some(p), Some(pr)) => {
                let mut stmt = conn.prepare(&query)?;
                stmt.query_map(params![e, p, pr], |row| self.row_to_record(row))?
                    .collect::<std::result::Result<Vec<_>, _>>()?
            }
            (None, Some(e), Some(p), None) => {
                let mut stmt = conn.prepare(&query)?;
                stmt.query_map(params![e, p], |row| self.row_to_record(row))?
                    .collect::<std::result::Result<Vec<_>, _>>()?
            }
            (None, Some(e), None, Some(pr)) => {
                let mut stmt = conn.prepare(&query)?;
                stmt.query_map(params![e, pr], |row| self.row_to_record(row))?
                    .collect::<std::result::Result<Vec<_>, _>>()?
            }
            (None, Some(e), None, None) => {
                let mut stmt = conn.prepare(&query)?;
                stmt.query_map(params![e], |row| self.row_to_record(row))?
                    .collect::<std::result::Result<Vec<_>, _>>()?
            }
            (None, None, Some(p), Some(pr)) => {
                let mut stmt = conn.prepare(&query)?;
                stmt.query_map(params![p, pr], |row| self.row_to_record(row))?
                    .collect::<std::result::Result<Vec<_>, _>>()?
            }
            (None, None, Some(p), None) => {
                let mut stmt = conn.prepare(&query)?;
                stmt.query_map(params![p], |row| self.row_to_record(row))?
                    .collect::<std::result::Result<Vec<_>, _>>()?
            }
            (None, None, None, Some(pr)) => {
                let mut stmt = conn.prepare(&query)?;
                stmt.query_map(params![pr], |row| self.row_to_record(row))?
                    .collect::<std::result::Result<Vec<_>, _>>()?
            }
            (None, None, None, None) => {
                let mut stmt = conn.prepare(&query)?;
                stmt.query_map(params![], |row| self.row_to_record(row))?
                    .collect::<std::result::Result<Vec<_>, _>>()?
            }
        };

        Ok(records)
    }

    /// Transform database row to CostRecord.
    fn row_to_record(&self, row: &rusqlite::Row) -> rusqlite::Result<CostRecord> {
        let timestamp_secs: i64 = row.get(1)?;
        let timestamp = DateTime::<Utc>::from_timestamp(timestamp_secs, 0)
            .unwrap_or_else(|| Utc::now());

        Ok(CostRecord {
            timestamp,
            agent_id: row.get(0)?,
            plan_id: row.get(9)?,
            model: row.get(7)?,
            provider: row.get(8)?,
            input_tokens: row.get(2)?,
            output_tokens: row.get(3)?,
            cached_tokens: row.get(4)?,
            total_tokens: row.get(5)?,
            estimated_cost: row.get(6)?,
            model_tier: row.get(10).ok(),
        })
    }

    /// Generate cost summary from records.
    ///
    /// # Arguments
    /// * `records` - Cost records to aggregate
    ///
    /// # Returns
    /// Aggregated cost summary
    pub fn generate_summary(&self, records: &[CostRecord]) -> CostSummary {
        if records.is_empty() {
            let now = Utc::now();
            return CostSummary {
                period: (now, now),
                total_cost: 0.0,
                total_tokens: 0,
                breakdown_by_provider: HashMap::new(),
                breakdown_by_model: HashMap::new(),
                breakdown_by_plan: HashMap::new(),
                top_plans: Vec::new(),
                tier_breakdown: None,
            };
        }

        // Calculate period
        let timestamps: Vec<DateTime<Utc>> = records.iter().map(|r| r.timestamp).collect();
        let start = *timestamps.iter().min().unwrap();
        let end = *timestamps.iter().max().unwrap();

        // Calculate totals
        let total_cost: f64 = records.iter().map(|r| r.estimated_cost).sum();
        let total_tokens: u64 = records.iter().map(|r| r.total_tokens).sum();

        // Build breakdowns
        let mut breakdown_by_provider: HashMap<String, f64> = HashMap::new();
        let mut breakdown_by_model: HashMap<String, f64> = HashMap::new();
        let mut breakdown_by_plan: HashMap<String, f64> = HashMap::new();

        for record in records {
            // Provider breakdown
            if let Some(provider) = &record.provider {
                *breakdown_by_provider.entry(provider.clone()).or_insert(0.0) += record.estimated_cost;
            }

            // Model breakdown
            if let Some(model) = &record.model {
                *breakdown_by_model.entry(model.clone()).or_insert(0.0) += record.estimated_cost;
            }

            // Plan breakdown
            if let Some(plan_id) = &record.plan_id {
                *breakdown_by_plan.entry(plan_id.clone()).or_insert(0.0) += record.estimated_cost;
            }
        }

        // Top plans (sorted by cost, descending)
        let mut top_plans: Vec<(String, f64)> = breakdown_by_plan
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        top_plans.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        top_plans.truncate(10);

        // Calculate tier breakdown if tier data is available
        let tier_breakdown = self.calculate_tier_breakdown(records);

        CostSummary {
            period: (start, end),
            total_cost,
            total_tokens,
            breakdown_by_provider,
            breakdown_by_model,
            breakdown_by_plan,
            top_plans,
            tier_breakdown,
        }
    }

    /// Calculates tier breakdown from cost records.
    ///
    /// Aggregates costs by model tier (Smart vs Eco) and calculates
    /// estimated savings vs all-Smart baseline.
    ///
    /// # Arguments
    /// * `records` - Cost records to analyze
    ///
    /// # Returns
    /// TierBreakdown if tier data is available, None otherwise
    fn calculate_tier_breakdown(&self, records: &[CostRecord]) -> Option<TierBreakdown> {
        // Check if any records have tier data
        let has_tier_data = records.iter().any(|r| r.model_tier.is_some());
        if !has_tier_data {
            return None;
        }

        let mut smart_tier = TierMetrics {
            request_count: 0,
            input_tokens: 0,
            output_tokens: 0,
            cost: 0.0,
        };
        let mut eco_tier = TierMetrics {
            request_count: 0,
            input_tokens: 0,
            output_tokens: 0,
            cost: 0.0,
        };

        // Aggregate by tier
        for record in records {
            if let Some(ref tier) = record.model_tier {
                match tier.as_str() {
                    "smart" => {
                        smart_tier.request_count += 1;
                        smart_tier.input_tokens += record.input_tokens;
                        smart_tier.output_tokens += record.output_tokens;
                        smart_tier.cost += record.estimated_cost;
                    }
                    "eco" => {
                        eco_tier.request_count += 1;
                        eco_tier.input_tokens += record.input_tokens;
                        eco_tier.output_tokens += record.output_tokens;
                        eco_tier.cost += record.estimated_cost;
                    }
                    _ => {
                        // Unknown tier, skip
                    }
                }
            }
        }

        // Calculate estimated savings vs all-Smart baseline
        // Assume Smart tier pricing: $3/$15 per 1M tokens (input/output)
        // This is a rough estimate; actual savings depend on specific model pricing
        let smart_input_price = 3.0;
        let smart_output_price = 15.0;
        
        let total_input_tokens = smart_tier.input_tokens + eco_tier.input_tokens;
        let total_output_tokens = smart_tier.output_tokens + eco_tier.output_tokens;
        
        let all_smart_cost = (total_input_tokens as f64 / 1_000_000.0) * smart_input_price
            + (total_output_tokens as f64 / 1_000_000.0) * smart_output_price;
        
        let actual_cost = smart_tier.cost + eco_tier.cost;
        let estimated_savings = all_smart_cost - actual_cost;

        Some(TierBreakdown {
            smart_tier,
            eco_tier,
            estimated_savings,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytics::export::ExportFormat;
    use crate::monitoring::{AgentRecord, AgentStatus, TelemetryRecord, TelemetryTracking};
    use chrono::Utc;
    use std::path::PathBuf;
    use tempfile::TempDir;

    async fn setup_test_service() -> (MonitoringService, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let monitoring = MonitoringService::open(&db_path).unwrap();
        (monitoring, temp_dir)
    }

    #[tokio::test]
    async fn test_query_no_filters() {
        let (monitoring, _temp) = setup_test_service().await;
        let service = CostQueryService::new(&monitoring);

        // Create agent with plan_id
        let mut agent = AgentRecord::new("agent-1".to_string(), "test".to_string());
        agent.plan_id = Some("REQ-123".to_string());
        monitoring.register_agent(&agent).unwrap();

        // Create telemetry
        let telemetry = TelemetryRecord::new("agent-1".to_string())
            .with_tokens(1000, 500)
            .with_model("claude-3.5-sonnet".to_string(), "anthropic".to_string());
        monitoring.record_telemetry(&telemetry).await.unwrap();

        let options = ExportOptions {
            format: ExportFormat::Csv,
            start_date: None,
            end_date: None,
            plan_id: None,
            provider: None,
            output_path: None,
        };

        let records = service.query_records(&options).unwrap();
        assert!(!records.is_empty());
        assert_eq!(records[0].agent_id, "agent-1");
        assert_eq!(records[0].plan_id, Some("REQ-123".to_string()));
    }

    #[tokio::test]
    async fn test_query_with_plan_filter() {
        let (monitoring, _temp) = setup_test_service().await;
        let service = CostQueryService::new(&monitoring);

        // Create agents with different plan_ids
        let mut agent1 = AgentRecord::new("agent-1".to_string(), "test".to_string());
        agent1.plan_id = Some("REQ-123".to_string());
        let mut agent2 = AgentRecord::new("agent-2".to_string(), "test".to_string());
        agent2.plan_id = Some("REQ-124".to_string());
        monitoring.register_agent(&agent1).unwrap();
        monitoring.register_agent(&agent2).unwrap();

        // Create telemetry for both
        let telemetry1 = TelemetryRecord::new("agent-1".to_string())
            .with_tokens(1000, 500)
            .with_model("claude-3.5-sonnet".to_string(), "anthropic".to_string());
        let telemetry2 = TelemetryRecord::new("agent-2".to_string())
            .with_tokens(2000, 1000)
            .with_model("gpt-4o".to_string(), "openai".to_string());
        monitoring.record_telemetry(&telemetry1).await.unwrap();
        monitoring.record_telemetry(&telemetry2).await.unwrap();

        let options = ExportOptions {
            format: ExportFormat::Csv,
            start_date: None,
            end_date: None,
            plan_id: Some("REQ-123".to_string()),
            provider: None,
            output_path: None,
        };

        let records = service.query_records(&options).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].plan_id, Some("REQ-123".to_string()));
    }

    #[test]
    fn test_generate_summary_empty() {
        let (monitoring, _temp) = futures::executor::block_on(setup_test_service());
        let service = CostQueryService::new(&monitoring);
        let summary = service.generate_summary(&[]);
        assert_eq!(summary.total_cost, 0.0);
        assert_eq!(summary.total_tokens, 0);
    }

    #[test]
    fn test_generate_summary() {
        let (monitoring, _temp) = futures::executor::block_on(setup_test_service());
        let service = CostQueryService::new(&monitoring);
        let records = vec![
            CostRecord {
                timestamp: Utc::now(),
                agent_id: "agent-1".to_string(),
                plan_id: Some("REQ-123".to_string()),
                model: Some("claude-3.5-sonnet".to_string()),
                provider: Some("anthropic".to_string()),
                input_tokens: 1000,
                output_tokens: 500,
                cached_tokens: 0,
                total_tokens: 1500,
                estimated_cost: 10.0,
                model_tier: None,
            },
            CostRecord {
                timestamp: Utc::now(),
                agent_id: "agent-2".to_string(),
                plan_id: Some("REQ-123".to_string()),
                model: Some("claude-3.5-sonnet".to_string()),
                provider: Some("anthropic".to_string()),
                input_tokens: 2000,
                output_tokens: 1000,
                cached_tokens: 0,
                total_tokens: 3000,
                estimated_cost: 20.0,
                model_tier: None,
            },
        ];

        let summary = service.generate_summary(&records);
        assert_eq!(summary.total_cost, 30.0);
        assert_eq!(summary.total_tokens, 4500);
        assert_eq!(summary.breakdown_by_provider.get("anthropic"), Some(&30.0));
        assert_eq!(summary.breakdown_by_plan.get("REQ-123"), Some(&30.0));
    }

    #[test]
    fn test_tier_breakdown_calculation() {
        let (monitoring, _temp) = futures::executor::block_on(setup_test_service());
        let service = CostQueryService::new(&monitoring);
        
        let records = vec![
            CostRecord {
                timestamp: Utc::now(),
                agent_id: "agent-1".to_string(),
                plan_id: Some("REQ-123".to_string()),
                model: Some("claude-sonnet".to_string()),
                provider: Some("anthropic".to_string()),
                input_tokens: 1000,
                output_tokens: 500,
                cached_tokens: 0,
                total_tokens: 1500,
                estimated_cost: 0.021, // Smart tier cost
                model_tier: Some("smart".to_string()),
            },
            CostRecord {
                timestamp: Utc::now(),
                agent_id: "agent-2".to_string(),
                plan_id: Some("REQ-123".to_string()),
                model: Some("claude-haiku".to_string()),
                provider: Some("anthropic".to_string()),
                input_tokens: 2000,
                output_tokens: 1000,
                cached_tokens: 0,
                total_tokens: 3000,
                estimated_cost: 0.003, // Eco tier cost
                model_tier: Some("eco".to_string()),
            },
        ];

        let summary = service.generate_summary(&records);
        
        // Should have tier breakdown
        assert!(summary.tier_breakdown.is_some());
        let tier_breakdown = summary.tier_breakdown.unwrap();
        
        assert_eq!(tier_breakdown.smart_tier.request_count, 1);
        assert_eq!(tier_breakdown.smart_tier.input_tokens, 1000);
        assert_eq!(tier_breakdown.smart_tier.output_tokens, 500);
        
        assert_eq!(tier_breakdown.eco_tier.request_count, 1);
        assert_eq!(tier_breakdown.eco_tier.input_tokens, 2000);
        assert_eq!(tier_breakdown.eco_tier.output_tokens, 1000);
        
        // Savings should be positive (eco tier is cheaper)
        assert!(tier_breakdown.estimated_savings > 0.0);
    }

    #[test]
    fn test_tier_breakdown_no_tier_data() {
        let (monitoring, _temp) = futures::executor::block_on(setup_test_service());
        let service = CostQueryService::new(&monitoring);
        
        let records = vec![
            CostRecord {
                timestamp: Utc::now(),
                agent_id: "agent-1".to_string(),
                plan_id: Some("REQ-123".to_string()),
                model: Some("claude-sonnet".to_string()),
                provider: Some("anthropic".to_string()),
                input_tokens: 1000,
                output_tokens: 500,
                cached_tokens: 0,
                total_tokens: 1500,
                estimated_cost: 10.0,
                model_tier: None, // No tier data
            },
        ];

        let summary = service.generate_summary(&records);
        
        // Should not have tier breakdown when no tier data
        assert!(summary.tier_breakdown.is_none());
    }
}

