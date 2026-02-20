//! Cost data export functionality for analytics.
//!
//! Provides data structures and traits for exporting cost data in various formats
//! (CSV, JSON, Markdown) with filtering capabilities.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use thiserror::Error;

/// Export format options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    /// CSV format for Excel and finance systems.
    Csv,
    /// JSON format for programmatic access.
    Json,
    /// Markdown format for human-readable reports.
    Markdown,
}

impl fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExportFormat::Csv => write!(f, "csv"),
            ExportFormat::Json => write!(f, "json"),
            ExportFormat::Markdown => write!(f, "markdown"),
        }
    }
}

impl ExportFormat {
    /// Parse export format from string (case-insensitive).
    pub fn from_str(s: &str) -> Result<Self, ExportError> {
        match s.to_lowercase().as_str() {
            "csv" => Ok(ExportFormat::Csv),
            "json" => Ok(ExportFormat::Json),
            "markdown" | "md" => Ok(ExportFormat::Markdown),
            _ => Err(ExportError::InvalidFormat(s.to_string())),
        }
    }

    /// Get file extension for this format.
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Csv => "csv",
            ExportFormat::Json => "json",
            ExportFormat::Markdown => "md",
        }
    }
}

/// Individual cost record from telemetry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostRecord {
    /// Timestamp of the telemetry record.
    pub timestamp: DateTime<Utc>,
    /// Agent ID that generated this cost.
    pub agent_id: String,
    /// Plan/requirement ID (if available).
    pub plan_id: Option<String>,
    /// Model name used.
    pub model: Option<String>,
    /// Provider name (e.g., "anthropic", "openai").
    pub provider: Option<String>,
    /// Input tokens consumed.
    pub input_tokens: u64,
    /// Output tokens generated.
    pub output_tokens: u64,
    /// Cached tokens reused.
    pub cached_tokens: u64,
    /// Total tokens (input + output).
    pub total_tokens: u64,
    /// Estimated cost in USD.
    pub estimated_cost: f64,
    /// Model tier used ("smart" | "eco") if routing was used.
    pub model_tier: Option<String>,
    /// Engine ID (e.g., "ollama", "openai", "claude") for local model cost breakdown.
    pub engine_id: Option<String>,
}

/// Aggregated cost summary with breakdowns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostSummary {
    /// Time period covered (start, end).
    pub period: (DateTime<Utc>, DateTime<Utc>),
    /// Total cost across all records.
    pub total_cost: f64,
    /// Total tokens across all records.
    pub total_tokens: u64,
    /// Cost breakdown by provider.
    pub breakdown_by_provider: HashMap<String, f64>,
    /// Cost breakdown by model.
    pub breakdown_by_model: HashMap<String, f64>,
    /// Cost breakdown by plan/requirement.
    pub breakdown_by_plan: HashMap<String, f64>,
    /// Top plans by cost (sorted descending).
    pub top_plans: Vec<(String, f64)>,
    /// Tier breakdown with Smart/Eco metrics and savings.
    pub tier_breakdown: Option<TierBreakdown>,
    /// Local model cost breakdown by engine (e.g., {"ollama": 12.30, "lm-studio": 8.20}).
    /// Only populated when local models are used.
    pub local_breakdown: Option<HashMap<String, f64>>,
}

/// Breakdown of costs by model tier (Smart vs Eco).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierBreakdown {
    /// Smart tier metrics.
    pub smart_tier: TierMetrics,
    /// Eco tier metrics.
    pub eco_tier: TierMetrics,
    /// Estimated savings vs using all-Smart baseline.
    pub estimated_savings: f64,
}

/// Metrics for a single tier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierMetrics {
    /// Number of requests.
    pub request_count: u64,
    /// Total input tokens.
    pub input_tokens: u64,
    /// Total output tokens.
    pub output_tokens: u64,
    /// Total cost in USD.
    pub cost: f64,
}

/// Options for cost data export.
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// Export format to use.
    pub format: ExportFormat,
    /// Start date for filtering (inclusive).
    pub start_date: Option<DateTime<Utc>>,
    /// End date for filtering (inclusive).
    pub end_date: Option<DateTime<Utc>>,
    /// Filter by plan/requirement ID.
    pub plan_id: Option<String>,
    /// Filter by provider.
    pub provider: Option<String>,
    /// Output file path (if None, will use default).
    pub output_path: Option<PathBuf>,
}

/// Trait for exporting cost data in various formats.
pub trait Exporter {
    /// Export detailed cost records.
    ///
    /// # Arguments
    /// * `records` - Slice of cost records to export
    /// * `options` - Export options including format and filters
    ///
    /// # Returns
    /// Formatted string ready to write to file
    fn export(&self, records: &[CostRecord], options: &ExportOptions) -> Result<String, ExportError>;

    /// Export aggregated cost summary.
    ///
    /// # Arguments
    /// * `summary` - Aggregated cost summary
    /// * `options` - Export options including format
    ///
    /// # Returns
    /// Formatted string ready to write to file
    fn export_summary(
        &self,
        summary: &CostSummary,
        options: &ExportOptions,
    ) -> Result<String, ExportError>;
}

/// Errors that can occur during export operations.
#[derive(Debug, Error)]
pub enum ExportError {
    #[error("Invalid export format: {0}")]
    InvalidFormat(String),

    #[error("Export generation failed: {0}")]
    GenerationFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

