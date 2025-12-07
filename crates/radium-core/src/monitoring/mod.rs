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

mod error;
mod logs;
mod schema;
mod service;
mod telemetry;

pub use error::{MonitoringError, Result};
pub use logs::LogManager;
pub use schema::initialize_schema;
pub use service::{AgentRecord, AgentStatus, MonitoringService};
pub use telemetry::{TelemetryParser, TelemetryRecord, TelemetrySummary, TelemetryTracking};
