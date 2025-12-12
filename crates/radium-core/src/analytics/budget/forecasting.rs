//! Budget forecasting engine for predicting budget exhaustion and spend velocity.

use crate::monitoring::{MonitoringService, Result as MonitoringResult};
use chrono::{DateTime, Utc, Duration};
use rusqlite::params;
use std::sync::Arc;

/// Budget forecaster for calculating spend velocity and projecting exhaustion dates.
pub struct BudgetForecaster {
    /// Monitoring service for accessing telemetry data.
    telemetry_store: Arc<MonitoringService>,
    /// Optional cache for forecast results.
    cache: Option<Arc<AnalyticsCache>>,
}

/// Re-export AnalyticsCache for convenience.
pub use crate::analytics::budget::cache::AnalyticsCache;

/// Result of budget exhaustion forecast.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ForecastResult {
    /// Projected date when budget will be exhausted.
    pub exhaustion_date: DateTime<Utc>,
    /// Minimum date in confidence interval (95%).
    pub confidence_min: DateTime<Utc>,
    /// Maximum date in confidence interval (95%).
    pub confidence_max: DateTime<Utc>,
    /// Number of days remaining at current velocity.
    pub days_remaining: u32,
}

/// Result of scenario modeling (what-if analysis).
#[derive(Debug, Clone)]
pub struct ScenarioResult {
    /// Estimated cost for the scenario.
    pub estimated_cost: f64,
    /// Remaining budget after scenario execution.
    pub remaining_budget: f64,
    /// Days remaining after scenario execution.
    pub days_remaining: u32,
    /// Confidence interval for cost estimate (±).
    pub cost_confidence_interval: f64,
}

impl BudgetForecaster {
    /// Creates a new budget forecaster without caching.
    pub fn new(telemetry_store: Arc<MonitoringService>) -> Self {
        Self {
            telemetry_store,
            cache: None,
        }
    }

    /// Creates a new budget forecaster with caching.
    pub fn with_cache(telemetry_store: Arc<MonitoringService>, cache: Arc<AnalyticsCache>) -> Self {
        Self {
            telemetry_store,
            cache: Some(cache),
        }
    }

    /// Calculates the average daily spend velocity over the last N days.
    ///
    /// Uses daily_spend_summary table if available for better performance.
    ///
    /// # Arguments
    /// * `days` - Number of days to analyze
    ///
    /// # Returns
    /// Average daily spend in USD, or 0.0 if no data available
    pub fn calculate_spend_velocity(&self, days: u32) -> MonitoringResult<f64> {
        let conn = self.telemetry_store.conn();
        
        // Try to use daily summaries first (faster for 30+ days)
        if days >= 7 {
            let end_date = Utc::now().format("%Y-%m-%d").to_string();
            let start_date = (Utc::now() - Duration::days(days as i64))
                .format("%Y-%m-%d")
                .to_string();

            let stmt = conn.prepare(
                "SELECT SUM(total_cost) FROM daily_spend_summary 
                 WHERE date >= ?1 AND date <= ?2"
            );

            if let Ok(mut stmt) = stmt {
                if let Ok(total) = stmt.query_row(
                    params![start_date, end_date],
                    |row| row.get::<_, f64>(0)
                ) {
                    return Ok(total / days as f64);
                }
            }
        }

        // Fallback to raw telemetry query
        let now = Utc::now().timestamp() as i64;
        let start_timestamp = now - (days as i64 * 86400);

        let mut stmt = conn.prepare(
            "SELECT SUM(estimated_cost) FROM telemetry WHERE timestamp >= ?1 AND timestamp <= ?2"
        )?;

        let total_spent: Option<f64> = stmt.query_row(
            params![start_timestamp, now],
            |row| row.get(0)
        )?;

        let total = total_spent.unwrap_or(0.0);
        let velocity = if days > 0 {
            total / days as f64
        } else {
            0.0
        };

        Ok(velocity)
    }

    /// Forecasts when the budget will be exhausted based on current spend velocity.
    ///
    /// # Arguments
    /// * `remaining_budget` - Remaining budget in USD
    ///
    /// # Returns
    /// Forecast result with exhaustion date and confidence interval
    pub fn forecast_exhaustion(&self, remaining_budget: f64) -> MonitoringResult<ForecastResult> {
        // Check cache first
        if let Some(ref cache) = self.cache {
            let cache_key = format!("forecast_{:.2}", remaining_budget);
            if let Some(cached) = cache.get_forecast(&cache_key) {
                return Ok(cached);
            }
        }
        // Calculate velocity over last 30 days (or available data)
        let velocity = self.calculate_spend_velocity(30)?;

        if velocity <= 0.0 {
            // No spending data or zero velocity - return a far future date
            let far_future = Utc::now() + Duration::days(365);
            return Ok(ForecastResult {
                exhaustion_date: far_future,
                confidence_min: far_future,
                confidence_max: far_future,
                days_remaining: 365,
            });
        }

        // Calculate days until exhaustion
        let days_remaining = (remaining_budget / velocity).ceil() as u32;
        let exhaustion_date = Utc::now() + Duration::days(days_remaining as i64);

        // Calculate confidence interval based on variance in daily spend
        let (confidence_min, confidence_max) = self.calculate_confidence_interval(velocity, days_remaining)?;

        let result = ForecastResult {
            exhaustion_date,
            confidence_min,
            confidence_max,
            days_remaining,
        };

        // Cache the result
        if let Some(ref cache) = self.cache {
            let cache_key = format!("forecast_{:.2}", remaining_budget);
            cache.set_forecast(&cache_key, result.clone());
        }

        Ok(result)
    }

    /// Calculates confidence interval for forecast using variance in daily spend.
    ///
    /// Uses 95% confidence interval (mean ± 1.96 * std_dev).
    ///
    /// # Arguments
    /// * `mean_velocity` - Average daily spend velocity
    /// * `days_remaining` - Projected days until exhaustion
    ///
    /// # Returns
    /// Tuple of (min_date, max_date) for confidence interval
    fn calculate_confidence_interval(
        &self,
        mean_velocity: f64,
        days_remaining: u32,
    ) -> MonitoringResult<(DateTime<Utc>, DateTime<Utc>)> {
        let conn = self.telemetry_store.conn();
        let now = Utc::now().timestamp() as i64;
        let start_timestamp = now - (30 * 86400); // Last 30 days

        // Get daily spend amounts for variance calculation
        let mut stmt = conn.prepare(
            "SELECT date(timestamp, 'unixepoch') as day, SUM(estimated_cost) as daily_cost
             FROM telemetry
             WHERE timestamp >= ?1 AND timestamp <= ?2
             GROUP BY day
             ORDER BY day"
        )?;

        let daily_costs: Vec<f64> = stmt
            .query_map(params![start_timestamp, now], |row| {
                Ok(row.get::<_, f64>(1)?)
            })?
            .collect::<Result<Vec<_>, _>>()?;

        if daily_costs.is_empty() {
            // No data - return same date for both bounds
            let date = Utc::now() + Duration::days(days_remaining as i64);
            return Ok((date, date));
        }

        // Calculate standard deviation
        let std_dev = calculate_std_dev(&daily_costs, mean_velocity);

        // 95% confidence interval: mean ± 1.96 * std_dev
        let confidence_multiplier = 1.96;
        let velocity_min = (mean_velocity - confidence_multiplier * std_dev).max(0.01);
        let velocity_max = mean_velocity + confidence_multiplier * std_dev;

        // Calculate days with min and max velocities
        let conn = self.telemetry_store.conn();
        let mut stmt = conn.prepare(
            "SELECT SUM(estimated_cost) FROM telemetry"
        )?;
        let _total_spent: f64 = stmt.query_row([], |row| row.get::<_, Option<f64>>(0))?.unwrap_or(0.0);
        
        // For confidence interval, we need remaining budget
        // We'll estimate it from the mean velocity and days remaining
        let estimated_remaining = mean_velocity * days_remaining as f64;
        
        let days_min = if velocity_max > 0.0 {
            (estimated_remaining / velocity_max).ceil() as i64
        } else {
            days_remaining as i64
        };
        
        let days_max = if velocity_min > 0.0 {
            (estimated_remaining / velocity_min).ceil() as i64
        } else {
            days_remaining as i64
        };

        let confidence_min = Utc::now() + Duration::days(days_min);
        let confidence_max = Utc::now() + Duration::days(days_max);

        Ok((confidence_min, confidence_max))
    }

    /// Models a scenario to estimate cost impact of running specific requirements.
    ///
    /// # Arguments
    /// * `requirement_ids` - List of requirement/agent IDs to model
    /// * `remaining_budget` - Current remaining budget
    ///
    /// # Returns
    /// Scenario result with estimated cost and impact
    pub fn model_scenario(
        &self,
        requirement_ids: &[String],
        remaining_budget: f64,
    ) -> MonitoringResult<ScenarioResult> {
        if requirement_ids.is_empty() {
            return Ok(ScenarioResult {
                estimated_cost: 0.0,
                remaining_budget,
                days_remaining: 0,
                cost_confidence_interval: 0.0,
            });
        }

        let conn = self.telemetry_store.conn();

        // Get historical costs for similar requirements
        let mut costs = Vec::new();
        for req_id in requirement_ids {
            let mut stmt = conn.prepare(
                "SELECT SUM(estimated_cost) FROM telemetry WHERE agent_id = ?1"
            )?;
            if let Ok(Some(cost)) = stmt.query_row(params![req_id], |row| row.get(0)) {
                costs.push(cost);
            }
        }

        // If no historical data for these requirements, use average of all requirements
        if costs.is_empty() {
            let mut stmt = conn.prepare(
                "SELECT AVG(daily_cost) FROM (
                    SELECT agent_id, SUM(estimated_cost) as daily_cost
                    FROM telemetry
                    GROUP BY agent_id
                )"
            )?;
            let avg_cost: Option<f64> = stmt.query_row([], |row| row.get(0))?;
            if let Some(avg) = avg_cost {
                costs.push(avg * requirement_ids.len() as f64);
            } else {
                costs.push(0.0);
            }
        }

        let estimated_cost = if costs.is_empty() {
            0.0
        } else {
            calculate_mean(&costs)
        };

        // Calculate confidence interval
        let std_dev = if costs.len() > 1 {
            calculate_std_dev(&costs, estimated_cost)
        } else {
            estimated_cost * 0.2 // 20% default uncertainty
        };
        let cost_confidence_interval = 1.96 * std_dev;

        let remaining_budget_after = remaining_budget - estimated_cost;

        // Calculate days remaining after scenario
        let velocity = self.calculate_spend_velocity(30)?;
        let days_remaining = if velocity > 0.0 {
            (remaining_budget_after / velocity).ceil() as u32
        } else {
            0
        };

        Ok(ScenarioResult {
            estimated_cost,
            remaining_budget: remaining_budget_after.max(0.0),
            days_remaining,
            cost_confidence_interval,
        })
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

/// Calculates the variance of a vector of values.
#[allow(dead_code)]
fn calculate_variance(values: &[f64], mean: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    values
        .iter()
        .map(|v| (v - mean).powi(2))
        .sum::<f64>()
        / values.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitoring::{MonitoringService, TelemetryRecord};

    fn create_test_service() -> MonitoringService {
        let service = MonitoringService::new().unwrap();
        // Insert test agent
        service.register_agent(&crate::monitoring::AgentRecord::new(
            "test-agent".to_string(),
            "test".to_string(),
        )).unwrap();
        service
    }

    #[test]
    fn test_calculate_mean() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((calculate_mean(&values) - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_std_dev() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let mean = calculate_mean(&values);
        let std_dev = calculate_std_dev(&values, mean);
        // Standard deviation of [1,2,3,4,5] is approximately 1.58
        assert!((std_dev - 1.58).abs() < 0.1);
    }

    #[test]
    fn test_calculate_spend_velocity() {
        let service = Arc::new(create_test_service());
        let forecaster = BudgetForecaster::new(service.clone());

        // Insert 30 days of telemetry at $3/day
        let now = Utc::now().timestamp() as i64;
        for i in 0..30 {
            let timestamp = now - ((30 - i) * 86400);
            let mut record = TelemetryRecord::new("test-agent".to_string());
            record.estimated_cost = 3.0;
            record.timestamp = timestamp as u64;
            service.record_telemetry_sync(&record).unwrap();
        }

        let velocity = forecaster.calculate_spend_velocity(30).unwrap();
        // Should be approximately $3/day
        assert!((velocity - 3.0).abs() < 0.1);
    }

    #[test]
    fn test_forecast_exhaustion() {
        let service = Arc::new(create_test_service());
        let forecaster = BudgetForecaster::new(service.clone());

        // Insert 30 days of telemetry at $3/day = $90 total
        let now = Utc::now().timestamp() as i64;
        for i in 0..30 {
            let timestamp = now - ((30 - i) * 86400);
            let mut record = TelemetryRecord::new("test-agent".to_string());
            record.estimated_cost = 3.0;
            record.timestamp = timestamp as u64;
            service.record_telemetry_sync(&record).unwrap();
        }

        // Forecast with $40 remaining budget
        let forecast = forecaster.forecast_exhaustion(40.0).unwrap();
        
        // At $3/day, $40 should last about 13-14 days
        assert!(forecast.days_remaining >= 13 && forecast.days_remaining <= 14);
        assert!(forecast.exhaustion_date > Utc::now());
    }
}

