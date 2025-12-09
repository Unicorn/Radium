//! Anomaly detection for identifying unusual costs using statistical analysis.

use crate::monitoring::{MonitoringService, Result as MonitoringResult};
use chrono::{DateTime, Utc};
use rusqlite::params;
use std::sync::Arc;

/// Anomaly detector for identifying unusual costs using z-score analysis.
pub struct AnomalyDetector {
    /// Monitoring service for accessing telemetry data.
    telemetry_store: Arc<MonitoringService>,
}

/// Cost statistics for anomaly detection.
#[derive(Debug, Clone)]
pub struct CostStatistics {
    /// Mean cost per requirement.
    pub mean: f64,
    /// Standard deviation of costs.
    pub std_dev: f64,
    /// Median cost.
    pub median: f64,
    /// 95th percentile cost.
    pub percentile_95: f64,
}

/// Detected cost anomaly.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CostAnomaly {
    /// Requirement/agent ID with anomalous cost.
    pub requirement_id: String,
    /// Cost of the requirement.
    pub cost: f64,
    /// Z-score indicating how many standard deviations from mean.
    pub z_score: f64,
    /// Severity of the anomaly.
    pub severity: AnomalySeverity,
    /// Category of the anomaly.
    pub category: AnomalyCategory,
    /// Timestamp when anomaly was detected.
    pub timestamp: DateTime<Utc>,
}

/// Severity level of an anomaly.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum AnomalySeverity {
    /// Minor anomaly: 2-3 standard deviations from mean.
    Minor,
    /// Major anomaly: >3 standard deviations from mean.
    Major,
}

/// Category of anomaly.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum AnomalyCategory {
    /// Unusually high token usage.
    TokenSpike,
    /// Model routing issue (expensive model used unexpectedly).
    ModelRoutingIssue,
    /// Legitimate complexity (high execution time, many tokens).
    LegitimateComplexity,
    /// Unknown cause.
    Unknown,
}

impl AnomalyDetector {
    /// Creates a new anomaly detector.
    pub fn new(telemetry_store: Arc<MonitoringService>) -> Self {
        Self { telemetry_store }
    }

    /// Calculates cost statistics for the given time window.
    ///
    /// # Arguments
    /// * `window_days` - Number of days to analyze
    ///
    /// # Returns
    /// Cost statistics including mean, std_dev, median, and 95th percentile
    pub fn calculate_statistics(&self, window_days: u32) -> MonitoringResult<CostStatistics> {
        let conn = self.telemetry_store.conn();
        let now = Utc::now().timestamp() as i64;
        let start_timestamp = now - (window_days as i64 * 86400);

        // Get cost per requirement (agent_id)
        let mut stmt = conn.prepare(
            "SELECT agent_id, SUM(estimated_cost) as total_cost
             FROM telemetry
             WHERE timestamp >= ?1 AND timestamp <= ?2
             GROUP BY agent_id"
        )?;

        let costs: Vec<f64> = stmt
            .query_map(params![start_timestamp, now], |row| {
                Ok(row.get::<_, f64>(1)?)
            })?
            .collect::<std::result::Result<Vec<_>, rusqlite::Error>>()?;

        if costs.is_empty() {
            return Ok(CostStatistics {
                mean: 0.0,
                std_dev: 0.0,
                median: 0.0,
                percentile_95: 0.0,
            });
        }

        let mean = calculate_mean(&costs);
        let std_dev = calculate_std_dev(&costs, mean);
        let median = calculate_median(&costs);
        let percentile_95 = calculate_percentile(&costs, 95);

        Ok(CostStatistics {
            mean,
            std_dev,
            median,
            percentile_95,
        })
    }

    /// Detects anomalies in requirement costs using z-score analysis.
    ///
    /// # Arguments
    /// * `window_days` - Number of days to analyze
    ///
    /// # Returns
    /// Vector of detected anomalies, sorted by z-score (descending)
    pub fn detect_anomalies(&self, window_days: u32) -> MonitoringResult<Vec<CostAnomaly>> {
        // Check if we have sufficient data
        if !self.is_sufficient_data(window_days)? {
            return Ok(Vec::new());
        }

        let stats = self.calculate_statistics(window_days)?;

        if stats.std_dev == 0.0 {
            // No variance - all costs are the same, no anomalies
            return Ok(Vec::new());
        }

        let conn = self.telemetry_store.conn();
        let now = Utc::now().timestamp() as i64;
        let start_timestamp = now - (window_days as i64 * 86400);

        // Get cost per requirement
        let mut stmt = conn.prepare(
            "SELECT agent_id, SUM(estimated_cost) as total_cost, MAX(timestamp) as last_timestamp
             FROM telemetry
             WHERE timestamp >= ?1 AND timestamp <= ?2
             GROUP BY agent_id"
        )?;

        let mut anomalies = Vec::new();

        let rows = stmt.query_map(params![start_timestamp, now], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })?;

        for row_result in rows {
            let (agent_id, cost, timestamp) = row_result?;

            // Calculate z-score
            let z_score = (cost - stats.mean) / stats.std_dev;

            // Flag anomalies at 2σ threshold
            if z_score > 2.0 {
                let severity = if z_score >= 3.0 {
                    AnomalySeverity::Major
                } else {
                    AnomalySeverity::Minor
                };

                // Categorize the anomaly
                let category = self.categorize_anomaly(&agent_id, cost, &stats)?;

                let anomaly = CostAnomaly {
                    requirement_id: agent_id,
                    cost,
                    z_score,
                    severity,
                    category,
                    timestamp: DateTime::from_timestamp(timestamp, 0)
                        .unwrap_or_else(Utc::now),
                };

                anomalies.push(anomaly);
            }
        }

        // Sort by z-score descending (most anomalous first)
        anomalies.sort_by(|a, b| b.z_score.partial_cmp(&a.z_score).unwrap_or(std::cmp::Ordering::Equal));

        Ok(anomalies)
    }

    /// Categorizes an anomaly based on telemetry details.
    ///
    /// # Arguments
    /// * `requirement_id` - Agent/requirement ID
    /// * `cost` - Cost of the requirement
    /// * `stats` - Cost statistics for comparison
    ///
    /// # Returns
    /// Anomaly category
    fn categorize_anomaly(
        &self,
        requirement_id: &str,
        cost: f64,
        stats: &CostStatistics,
    ) -> MonitoringResult<AnomalyCategory> {
        let conn = self.telemetry_store.conn();

        // Get telemetry details for this requirement
        let mut stmt = conn.prepare(
            "SELECT SUM(total_tokens) as total_tokens, 
                    GROUP_CONCAT(DISTINCT model) as models,
                    COUNT(*) as record_count
             FROM telemetry
             WHERE agent_id = ?1"
        )?;

        let result: Option<(Option<u64>, Option<String>, i64)> = stmt
            .query_row(params![requirement_id], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                ))
            })
            .ok();

        if let Some((total_tokens, models, _record_count)) = result {
            // Get average tokens for comparison
            let mut avg_stmt = conn.prepare(
                "SELECT AVG(total_tokens) FROM telemetry WHERE agent_id != ?1"
            )?;
            let avg_tokens: Option<f64> = avg_stmt.query_row(params![requirement_id], |row| row.get(0))?;

            if let (Some(tokens), Some(avg)) = (total_tokens, avg_tokens) {
                let token_ratio = tokens as f64 / avg;
                
                // If tokens are >2x average, likely a token spike
                if token_ratio > 2.0 {
                    return Ok(AnomalyCategory::TokenSpike);
                }

                // Check if expensive model was used
                if let Some(ref model_list) = models {
                    let expensive_models = ["gpt-4", "claude-3-opus", "gpt-4-turbo"];
                    if expensive_models.iter().any(|m| model_list.contains(m)) {
                        // Check if this is unexpected (cost is high but tokens aren't)
                        if token_ratio < 1.5 {
                            return Ok(AnomalyCategory::ModelRoutingIssue);
                        }
                    }
                }

                // If tokens are high and cost is high, likely legitimate complexity
                if token_ratio > 1.5 {
                    return Ok(AnomalyCategory::LegitimateComplexity);
                }
            }
        }

        Ok(AnomalyCategory::Unknown)
    }

    /// Checks if there is sufficient data for reliable anomaly detection.
    ///
    /// Requires at least 30 data points (requirements) for accurate statistics.
    ///
    /// # Arguments
    /// * `window_days` - Number of days to check
    ///
    /// # Returns
    /// True if sufficient data exists, false otherwise
    pub fn is_sufficient_data(&self, window_days: u32) -> MonitoringResult<bool> {
        let conn = self.telemetry_store.conn();
        let now = Utc::now().timestamp() as i64;
        let start_timestamp = now - (window_days as i64 * 86400);

        let mut stmt = conn.prepare(
            "SELECT COUNT(DISTINCT agent_id) FROM telemetry WHERE timestamp >= ?1 AND timestamp <= ?2"
        )?;

        let count: i64 = stmt.query_row(params![start_timestamp, now], |row| row.get(0))?;

        Ok(count >= 30)
    }
}

/// Calculates the mean of a vector of values.
fn calculate_mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

/// Calculates the standard deviation of a vector of values.
fn calculate_std_dev(values: &[f64], mean: f64) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }

    let variance = values
        .iter()
        .map(|v| (v - mean).powi(2))
        .sum::<f64>()
        / (values.len() - 1) as f64;

    variance.sqrt()
}

/// Calculates the median of a vector of values.
fn calculate_median(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    } else {
        sorted[mid]
    }
}

/// Calculates the nth percentile of a vector of values.
fn calculate_percentile(values: &[f64], percentile: u8) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let index = ((sorted.len() - 1) as f64 * percentile as f64 / 100.0).ceil() as usize;
    sorted[index.min(sorted.len() - 1)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitoring::{MonitoringService, TelemetryRecord};

    fn create_test_service() -> MonitoringService {
        let service = MonitoringService::new().unwrap();
        service
    }

    #[test]
    fn test_calculate_median() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((calculate_median(&values) - 3.0).abs() < 0.001);

        let values_even = vec![1.0, 2.0, 3.0, 4.0];
        assert!((calculate_median(&values_even) - 2.5).abs() < 0.001);
    }

    #[test]
    fn test_calculate_percentile() {
        let values: Vec<f64> = (1..=100).map(|i| i as f64).collect();
        let p95 = calculate_percentile(&values, 95);
        assert!((p95 - 95.0).abs() < 1.0);
    }

    #[test]
    fn test_detect_anomalies() {
        let service = Arc::new(create_test_service());
        let detector = AnomalyDetector::new(service.clone());

        // Insert 50 requirements with mean=$5, std_dev≈$1
        let now = Utc::now().timestamp() as i64;
        for i in 0..50 {
            let agent_id = format!("agent-{}", i);
            service.register_agent(&crate::monitoring::AgentRecord::new(
                agent_id.clone(),
                "test".to_string(),
            )).unwrap();

            // Most requirements cost $5, but add some variance
            let cost = if i == 0 {
                8.0 // Outlier: z-score ~3
            } else if i == 1 {
                7.0 // Outlier: z-score ~2
            } else {
                5.0 + (i % 3) as f64 * 0.3 - 0.3 // $4.7-$5.6
            };

            let mut record = TelemetryRecord::new(agent_id);
            record.estimated_cost = cost;
            record.timestamp = now as u64;
            service.record_telemetry_sync(&record).unwrap();
        }

        let anomalies = detector.detect_anomalies(30).unwrap();
        
        // Should detect at least the two outliers
        assert!(anomalies.len() >= 2);
        
        // First anomaly should be the $8 one (Major)
        let major_anomaly = anomalies.iter().find(|a| a.cost == 8.0);
        assert!(major_anomaly.is_some());
        assert_eq!(major_anomaly.unwrap().severity, AnomalySeverity::Major);
    }
}

