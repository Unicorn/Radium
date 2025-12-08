//! Budget analytics and forecasting for Radium.
//!
//! This module provides budget forecasting, anomaly detection, and cost analytics
//! to help users understand spending patterns and predict budget exhaustion.

pub mod forecasting;
pub mod anomaly_detection;
pub mod cache;

pub use forecasting::{BudgetForecaster, ForecastResult, ScenarioResult};
pub use anomaly_detection::{AnomalyDetector, CostAnomaly, CostStatistics, AnomalySeverity, AnomalyCategory};
pub use cache::{AnalyticsCache, DailySpendAggregator, DailySpendSummary};

