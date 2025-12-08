//! Caching and aggregation layer for budget analytics performance optimization.

use crate::monitoring::{MonitoringService, Result as MonitoringResult};
use chrono::{DateTime, Utc};
use rusqlite::params;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

/// Daily spend summary for pre-aggregated analytics.
#[derive(Debug, Clone)]
pub struct DailySpendSummary {
    /// Date in YYYY-MM-DD format.
    pub date: String,
    /// Total cost for the day.
    pub total_cost: f64,
    /// Total tokens for the day.
    pub total_tokens: u64,
    /// Number of requirements executed.
    pub requirement_count: u32,
    /// Average cost per requirement.
    pub avg_cost_per_requirement: f64,
}

/// Cached forecast result with timestamp.
#[derive(Debug, Clone)]
struct CachedForecast {
    /// The forecast result.
    result: crate::analytics::budget::forecasting::ForecastResult,
    /// When this was cached.
    cached_at: DateTime<Utc>,
}

/// In-memory cache for forecast results with TTL.
pub struct AnalyticsCache {
    /// Cache storage for forecast results.
    forecast_cache: Arc<Mutex<HashMap<String, CachedForecast>>>,
    /// Time-to-live in seconds (default 3600 = 1 hour).
    ttl_seconds: u64,
}

impl AnalyticsCache {
    /// Creates a new analytics cache with default TTL (1 hour).
    pub fn new() -> Self {
        Self::with_ttl(3600)
    }

    /// Creates a new analytics cache with custom TTL.
    ///
    /// # Arguments
    /// * `ttl_seconds` - Time-to-live in seconds
    pub fn with_ttl(ttl_seconds: u64) -> Self {
        Self {
            forecast_cache: Arc::new(Mutex::new(HashMap::new())),
            ttl_seconds,
        }
    }

    /// Gets a cached forecast result if it exists and hasn't expired.
    ///
    /// # Arguments
    /// * `cache_key` - Unique key for the cache entry
    ///
    /// # Returns
    /// Cached forecast result if valid, None otherwise
    pub fn get_forecast(
        &self,
        cache_key: &str,
    ) -> Option<crate::analytics::budget::forecasting::ForecastResult> {
        let cache = self.forecast_cache.lock().ok()?;
        let cached = cache.get(cache_key)?;

        // Check if expired
        let now = Utc::now();
        let age = now.signed_duration_since(cached.cached_at);
        if age.num_seconds() > self.ttl_seconds as i64 {
            return None;
        }

        Some(cached.result.clone())
    }

    /// Stores a forecast result in the cache.
    ///
    /// # Arguments
    /// * `cache_key` - Unique key for the cache entry
    /// * `result` - Forecast result to cache
    pub fn set_forecast(
        &self,
        cache_key: &str,
        result: crate::analytics::budget::forecasting::ForecastResult,
    ) {
        if let Ok(mut cache) = self.forecast_cache.lock() {
            // Clean up expired entries
            self.cleanup_expired(&mut cache);

            let cached = CachedForecast {
                result,
                cached_at: Utc::now(),
            };
            cache.insert(cache_key.to_string(), cached);
        }
    }

    /// Invalidates a specific cache entry.
    ///
    /// # Arguments
    /// * `cache_key` - Key to invalidate
    pub fn invalidate(&self, cache_key: &str) {
        if let Ok(mut cache) = self.forecast_cache.lock() {
            cache.remove(cache_key);
        }
    }

    /// Cleans up expired entries from the cache.
    fn cleanup_expired(&self, cache: &mut HashMap<String, CachedForecast>) {
        let now = Utc::now();
        cache.retain(|_, cached| {
            let age = now.signed_duration_since(cached.cached_at);
            age.num_seconds() <= self.ttl_seconds as i64
        });
    }
}

impl Default for AnalyticsCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Daily spend aggregator for pre-computing daily summaries.
pub struct DailySpendAggregator {
    /// Monitoring service for database access.
    telemetry_store: Arc<MonitoringService>,
}

impl DailySpendAggregator {
    /// Creates a new daily spend aggregator.
    pub fn new(telemetry_store: Arc<MonitoringService>) -> Self {
        Self { telemetry_store }
    }

    /// Aggregates telemetry data for a specific date.
    ///
    /// # Arguments
    /// * `date` - Date in YYYY-MM-DD format
    ///
    /// # Returns
    /// Daily spend summary for the date
    pub fn aggregate_day(&self, date: &str) -> MonitoringResult<DailySpendSummary> {
        let conn = self.telemetry_store.conn();

        // Parse date and get start/end timestamps for the day
        let date_obj = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
            .map_err(|e| crate::monitoring::MonitoringError::Other(e.to_string()))?;
        let start_datetime = date_obj.and_hms_opt(0, 0, 0)
            .ok_or_else(|| crate::monitoring::MonitoringError::Other("Invalid date".to_string()))?;
        let end_datetime = date_obj.and_hms_opt(23, 59, 59)
            .ok_or_else(|| crate::monitoring::MonitoringError::Other("Invalid date".to_string()))?;
        
        let start_timestamp = start_datetime.and_utc().timestamp() as i64;
        let end_timestamp = end_datetime.and_utc().timestamp() as i64;

        // Query telemetry for the day
        let mut stmt = conn.prepare(
            "SELECT 
                SUM(estimated_cost) as total_cost,
                SUM(total_tokens) as total_tokens,
                COUNT(DISTINCT agent_id) as requirement_count
             FROM telemetry
             WHERE timestamp >= ?1 AND timestamp <= ?2"
        )?;

        let row = stmt.query_row(params![start_timestamp, end_timestamp], |row| {
            Ok((
                row.get::<_, Option<f64>>(0)?.unwrap_or(0.0),
                row.get::<_, Option<i64>>(1)?.unwrap_or(0) as u64,
                row.get::<_, Option<i64>>(2)?.unwrap_or(0) as u32,
            ))
        })?;

        let (total_cost, total_tokens, requirement_count) = row;
        let avg_cost_per_requirement = if requirement_count > 0 {
            total_cost / requirement_count as f64
        } else {
            0.0
        };

        Ok(DailySpendSummary {
            date: date.to_string(),
            total_cost,
            total_tokens,
            requirement_count,
            avg_cost_per_requirement,
        })
    }

    /// Stores a daily spend summary in the database.
    ///
    /// # Arguments
    /// * `summary` - Daily spend summary to store
    pub fn store_summary(&self, summary: &DailySpendSummary) -> MonitoringResult<()> {
        let conn = self.telemetry_store.conn();
        let created_at = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        conn.execute(
            "INSERT OR REPLACE INTO daily_spend_summary 
             (date, total_cost, total_tokens, requirement_count, avg_cost_per_requirement, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                summary.date,
                summary.total_cost,
                summary.total_tokens as i64,
                summary.requirement_count as i64,
                summary.avg_cost_per_requirement,
                created_at,
            ],
        )?;

        Ok(())
    }

    /// Aggregates and stores summaries for all missing days.
    ///
    /// # Returns
    /// Number of days aggregated
    pub fn aggregate_missing_days(&self) -> MonitoringResult<u32> {
        let conn = self.telemetry_store.conn();

        // Find dates in telemetry that don't have summaries
        let mut stmt = conn.prepare(
            "SELECT DISTINCT date(timestamp, 'unixepoch') as day
             FROM telemetry
             WHERE date(timestamp, 'unixepoch') NOT IN (SELECT date FROM daily_spend_summary)
             ORDER BY day"
        )?;

        let missing_dates: Vec<String> = stmt
            .query_map([], |row| {
                Ok(row.get::<_, String>(0)?)
            })?
            .collect::<std::result::Result<Vec<_>, rusqlite::Error>>()?;

        let mut count = 0;
        for date in missing_dates {
            let summary = self.aggregate_day(&date)?;
            self.store_summary(&summary)?;
            count += 1;
        }

        Ok(count)
    }

    /// Gets daily spend summaries for a date range.
    ///
    /// # Arguments
    /// * `start_date` - Start date in YYYY-MM-DD format
    /// * `end_date` - End date in YYYY-MM-DD format
    ///
    /// # Returns
    /// Vector of daily spend summaries
    pub fn get_summaries(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> MonitoringResult<Vec<DailySpendSummary>> {
        // Ensure summaries exist
        self.aggregate_missing_days()?;

        let conn = self.telemetry_store.conn();
        let mut stmt = conn.prepare(
            "SELECT date, total_cost, total_tokens, requirement_count, avg_cost_per_requirement
             FROM daily_spend_summary
             WHERE date >= ?1 AND date <= ?2
             ORDER BY date"
        )?;

        let summaries: Vec<DailySpendSummary> = stmt
            .query_map(params![start_date, end_date], |row| {
                Ok(DailySpendSummary {
                    date: row.get(0)?,
                    total_cost: row.get(1)?,
                    total_tokens: row.get(2)?,
                    requirement_count: row.get(3)?,
                    avg_cost_per_requirement: row.get(4)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, rusqlite::Error>>()?;

        Ok(summaries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitoring::{MonitoringService, TelemetryRecord};

    #[test]
    fn test_cache_ttl() {
        let cache = AnalyticsCache::with_ttl(1); // 1 second TTL for testing
        let forecast = crate::analytics::budget::forecasting::ForecastResult {
            exhaustion_date: Utc::now(),
            confidence_min: Utc::now(),
            confidence_max: Utc::now(),
            days_remaining: 10,
        };

        cache.set_forecast("test-key", forecast.clone());
        
        // Should be available immediately
        assert!(cache.get_forecast("test-key").is_some());

        // Wait for expiration
        std::thread::sleep(Duration::from_secs(2));
        
        // Should be expired
        assert!(cache.get_forecast("test-key").is_none());
    }

    #[test]
    fn test_cache_invalidate() {
        let cache = AnalyticsCache::new();
        let forecast = crate::analytics::budget::forecasting::ForecastResult {
            exhaustion_date: Utc::now(),
            confidence_min: Utc::now(),
            confidence_max: Utc::now(),
            days_remaining: 10,
        };

        cache.set_forecast("test-key", forecast);
        assert!(cache.get_forecast("test-key").is_some());

        cache.invalidate("test-key");
        assert!(cache.get_forecast("test-key").is_none());
    }

    #[test]
    fn test_aggregate_day() {
        let service = Arc::new(MonitoringService::new().unwrap());
        let aggregator = DailySpendAggregator::new(service.clone());

        // Register agent
        service.register_agent(&crate::monitoring::AgentRecord::new(
            "test-agent".to_string(),
            "test".to_string(),
        )).unwrap();

        // Insert telemetry for today
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let today_start = Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap();
        let timestamp = today_start.and_utc().timestamp() as u64;

        for i in 0..5 {
            let mut record = TelemetryRecord::new("test-agent".to_string());
            record.estimated_cost = 2.0;
            record.total_tokens = 1000;
            record.timestamp = timestamp + (i * 3600) as u64; // Spread across hours
            service.record_telemetry_sync(&record).unwrap();
        }

        let summary = aggregator.aggregate_day(&today).unwrap();
        assert!((summary.total_cost - 10.0).abs() < 0.01);
        assert_eq!(summary.total_tokens, 5000);
        assert_eq!(summary.requirement_count, 1); // Same agent_id
    }
}

