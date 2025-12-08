//! Session analytics and reporting for Radium.
//!
//! Provides comprehensive session reporting with metrics, token tracking,
//! and cost transparency.

pub mod code_changes;
#[cfg(feature = "monitoring")]
pub mod budget;
#[cfg(feature = "monitoring")]
pub mod comparison;
#[cfg(feature = "monitoring")]
pub mod cost_history;
#[cfg(feature = "monitoring")]
pub mod cost_query;
#[cfg(feature = "monitoring")]
pub mod export;
#[cfg(feature = "monitoring")]
pub mod exporters;
#[cfg(feature = "monitoring")]
pub mod report;
#[cfg(feature = "monitoring")]
pub mod session;
pub mod storage;

#[cfg(feature = "monitoring")]
pub use budget::{
    AnalyticsCache, AnomalyCategory, AnomalyDetector, AnomalySeverity, BudgetForecaster,
    CostAnomaly, CostStatistics, DailySpendAggregator, DailySpendSummary, ForecastResult,
    ScenarioResult,
};
#[cfg(feature = "monitoring")]
pub use comparison::{ComparisonFormatter, SessionComparison};
#[cfg(feature = "monitoring")]
pub use cost_history::{
    CostAnalytics, CostBreakdown, CostEvent, CostSummary as CostHistorySummary, DateRange, TimePeriod,
};
#[cfg(feature = "monitoring")]
pub use cost_query::CostQueryService;
#[cfg(feature = "monitoring")]
pub use export::{CostRecord, CostSummary, ExportError, ExportFormat, ExportOptions, Exporter};
#[cfg(feature = "monitoring")]
pub use exporters::{CsvExporter, JsonExporter, MarkdownExporter};
#[cfg(feature = "monitoring")]
pub use report::{ReportFormatter, SessionReport};
#[cfg(feature = "monitoring")]
pub use session::{ModelUsageStats, SessionAnalytics, SessionMetrics};
pub use storage::{SessionMetadata, SessionStorage};
