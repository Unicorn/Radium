//! Agent monitoring and telemetry tracking.
//!
//! This module provides agent lifecycle tracking, telemetry parsing,
//! and monitoring database management.
//!
//! # Example
//!
//! ```rust,no_run
//! use radium_core::monitoring::{MonitoringService, AgentRecord, AgentStatus};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let service = MonitoringService::new()?;
//!
//! // Register a new agent
//! let record = AgentRecord::new("agent-1".to_string(), "developer".to_string());
//! service.register_agent(&record)?;
//!
//! // Update status
//! service.update_status("agent-1", AgentStatus::Running)?;
//!
//! // Complete agent
//! service.complete_agent("agent-1", 0)?;
//! # Ok(())
//! # }
//! ```

mod attribution;
mod budget;
mod error;
mod logs;
#[cfg(feature = "monitoring")]
pub mod permission_analytics;
pub(crate) mod schema;
pub(crate) mod service;
pub(crate) mod telemetry;

pub use attribution::{AttributionMetadata, generate_api_key_id};
pub use budget::{
    BudgetConfig, BudgetError, BudgetManager, BudgetStatus, ModelTier, ProviderCostBreakdown,
    ProviderComparison, ProviderCostInfo, TeamCostBreakdown, get_provider_comparison,
};
pub use error::{MonitoringError, Result};
pub use logs::LogManager;
pub use schema::initialize_schema;
pub use service::{AgentRecord, AgentStatus, AgentUsage, MonitoringService, UsageFilter};
pub use telemetry::{TelemetryParser, TelemetryRecord, TelemetrySummary, TelemetryTracking};
#[cfg(feature = "monitoring")]
pub use permission_analytics::{
    AgentUsageStats, Anomaly, AnomalyCategory, AnomalySeverity, PermissionAnalytics,
    PermissionEvent, PermissionOutcome, RuleEffectivenessStats, TimeSeriesPoint, ToolUsageStats,
};
