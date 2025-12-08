//! JSON exporter for cost data.

use crate::analytics::export::{CostRecord, CostSummary, ExportError, ExportOptions, Exporter};
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Metadata about the export operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExportMetadata {
    /// When the export was generated.
    exported_at: chrono::DateTime<Utc>,
    /// Number of records in the export.
    record_count: usize,
    /// Filters applied to the export.
    filters: ExportFilters,
}

/// Filters applied to the export.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExportFilters {
    /// Start date filter (if applied).
    start_date: Option<chrono::DateTime<Utc>>,
    /// End date filter (if applied).
    end_date: Option<chrono::DateTime<Utc>>,
    /// Plan ID filter (if applied).
    plan_id: Option<String>,
    /// Provider filter (if applied).
    provider: Option<String>,
}

/// Detailed export wrapper with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DetailedExport {
    /// Export metadata.
    metadata: ExportMetadata,
    /// Cost records.
    records: Vec<CostRecord>,
}

/// Summary export wrapper with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SummaryExport {
    /// Export metadata.
    metadata: ExportMetadata,
    /// Cost summary.
    summary: CostSummary,
}

/// JSON exporter implementation.
pub struct JsonExporter;

impl JsonExporter {
    /// Create export metadata from options.
    fn create_metadata(record_count: usize, options: &ExportOptions) -> ExportMetadata {
        ExportMetadata {
            exported_at: Utc::now(),
            record_count,
            filters: ExportFilters {
                start_date: options.start_date,
                end_date: options.end_date,
                plan_id: options.plan_id.clone(),
                provider: options.provider.clone(),
            },
        }
    }
}

impl Exporter for JsonExporter {
    fn export(&self, records: &[CostRecord], options: &ExportOptions) -> Result<String, ExportError> {
        let metadata = Self::create_metadata(records.len(), options);
        let export = DetailedExport {
            metadata,
            records: records.to_vec(),
        };

        serde_json::to_string_pretty(&export).map_err(|e| {
            ExportError::Serialization(format!("JSON serialization failed: {}", e))
        })
    }

    fn export_summary(
        &self,
        summary: &CostSummary,
        options: &ExportOptions,
    ) -> Result<String, ExportError> {
        // Count records from breakdown (approximate)
        let record_count = summary.breakdown_by_plan.len().max(1);
        let metadata = Self::create_metadata(record_count, options);
        let export = SummaryExport {
            metadata,
            summary: summary.clone(),
        };

        serde_json::to_string_pretty(&export).map_err(|e| {
            ExportError::Serialization(format!("JSON serialization failed: {}", e))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytics::export::ExportFormat;
    use chrono::Utc;
    use std::collections::HashMap;

    fn create_test_record() -> CostRecord {
        CostRecord {
            timestamp: Utc::now(),
            agent_id: "agent-1".to_string(),
            plan_id: Some("REQ-123".to_string()),
            model: Some("claude-3.5-sonnet".to_string()),
            provider: Some("anthropic".to_string()),
            input_tokens: 1500,
            output_tokens: 800,
            cached_tokens: 200,
            total_tokens: 2300,
            estimated_cost: 0.0234,
            model_tier: None,
        }
    }

    #[test]
    fn test_export_empty_records() {
        let exporter = JsonExporter;
        let options = ExportOptions {
            format: ExportFormat::Json,
            start_date: None,
            end_date: None,
            plan_id: None,
            provider: None,
            output_path: None,
        };
        let result = exporter.export(&[], &options);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("metadata"));
        assert!(json.contains("records"));
        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.get("metadata").is_some());
        assert!(parsed.get("records").is_some());
    }

    #[test]
    fn test_export_single_record() {
        let exporter = JsonExporter;
        let options = ExportOptions {
            format: ExportFormat::Json,
            start_date: None,
            end_date: None,
            plan_id: None,
            provider: None,
            output_path: None,
        };
        let record = create_test_record();
        let result = exporter.export(&[record], &options);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("agent-1"));
        assert!(json.contains("REQ-123"));
        assert!(json.contains("0.0234"));
        // Verify round-trip
        let parsed: DetailedExport = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.records.len(), 1);
        assert_eq!(parsed.records[0].agent_id, "agent-1");
    }

    #[test]
    fn test_export_with_filters() {
        let exporter = JsonExporter;
        let start_date = Utc::now() - chrono::Duration::days(30);
        let options = ExportOptions {
            format: ExportFormat::Json,
            start_date: Some(start_date),
            end_date: Some(Utc::now()),
            plan_id: Some("REQ-123".to_string()),
            provider: Some("anthropic".to_string()),
            output_path: None,
        };
        let record = create_test_record();
        let result = exporter.export(&[record], &options);
        assert!(result.is_ok());
        let json = result.unwrap();
        let parsed: DetailedExport = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.metadata.filters.plan_id, Some("REQ-123".to_string()));
        assert_eq!(parsed.metadata.filters.provider, Some("anthropic".to_string()));
    }

    #[test]
    fn test_export_summary() {
        let exporter = JsonExporter;
        let options = ExportOptions {
            format: ExportFormat::Json,
            start_date: None,
            end_date: None,
            plan_id: None,
            provider: None,
            output_path: None,
        };
        let summary = CostSummary {
            period: (Utc::now(), Utc::now()),
            total_cost: 100.0,
            total_tokens: 10000,
            breakdown_by_provider: {
                let mut map = HashMap::new();
                map.insert("anthropic".to_string(), 60.0);
                map.insert("openai".to_string(), 40.0);
                map
            },
            breakdown_by_model: {
                let mut map = HashMap::new();
                map.insert("claude-3.5-sonnet".to_string(), 60.0);
                map.insert("gpt-4o".to_string(), 40.0);
                map
            },
            breakdown_by_plan: {
                let mut map = HashMap::new();
                map.insert("REQ-123".to_string(), 70.0);
                map.insert("REQ-124".to_string(), 30.0);
                map
            },
            top_plans: vec![
                ("REQ-123".to_string(), 70.0),
                ("REQ-124".to_string(), 30.0),
            ],
        };
        let result = exporter.export_summary(&summary, &options);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("summary"));
        assert!(json.contains("100.0"));
        // Verify round-trip
        let parsed: SummaryExport = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.summary.total_cost, 100.0);
    }

    #[test]
    fn test_export_none_values() {
        let exporter = JsonExporter;
        let options = ExportOptions {
            format: ExportFormat::Json,
            start_date: None,
            end_date: None,
            plan_id: None,
            provider: None,
            output_path: None,
        };
        let record = CostRecord {
            timestamp: Utc::now(),
            agent_id: "agent-1".to_string(),
            plan_id: None,
            model: None,
            provider: None,
            input_tokens: 100,
            output_tokens: 50,
            cached_tokens: 0,
            total_tokens: 150,
            estimated_cost: 0.001,
            model_tier: None,
        };
        let result = exporter.export(&[record], &options);
        assert!(result.is_ok());
        let json = result.unwrap();
        // None values should be null in JSON
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let records = parsed.get("records").unwrap().as_array().unwrap();
        let first_record = &records[0];
        assert!(first_record.get("plan_id").unwrap().is_null());
        assert!(first_record.get("model").unwrap().is_null());
    }
}

