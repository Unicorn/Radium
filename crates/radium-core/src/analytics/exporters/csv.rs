//! CSV exporter for cost data.

use crate::analytics::export::{CostRecord, CostSummary, ExportError, ExportOptions, Exporter};
use chrono::Utc;
use std::io::Write;

/// CSV exporter implementation.
pub struct CsvExporter;

impl CsvExporter {
    /// Format timestamp as ISO 8601 string.
    fn format_timestamp(dt: &chrono::DateTime<Utc>) -> String {
        dt.to_rfc3339()
    }

    /// Format cost with 4 decimal places.
    fn format_cost(cost: f64) -> String {
        format!("{:.4}", cost)
    }

    /// Escape CSV field (handle quotes and commas).
    fn escape_field(field: &str) -> String {
        if field.contains(',') || field.contains('"') || field.contains('\n') {
            format!("\"{}\"", field.replace('"', "\"\""))
        } else {
            field.to_string()
        }
    }
}

impl Exporter for CsvExporter {
    fn export(&self, records: &[CostRecord], _options: &ExportOptions) -> Result<String, ExportError> {
        let mut writer = csv::WriterBuilder::new()
            .has_headers(true)
            .from_writer(Vec::new());

        // Write header
        writer.write_record(&[
            "timestamp",
            "agent_id",
            "plan_id",
            "model",
            "provider",
            "input_tokens",
            "output_tokens",
            "cached_tokens",
            "total_tokens",
            "estimated_cost",
        ])?;

        // Write records
        for record in records {
            writer.write_record(&[
                Self::format_timestamp(&record.timestamp),
                record.agent_id.clone(),
                record.plan_id.as_deref().unwrap_or("").to_string(),
                record.model.as_deref().unwrap_or("").to_string(),
                record.provider.as_deref().unwrap_or("").to_string(),
                record.input_tokens.to_string(),
                record.output_tokens.to_string(),
                record.cached_tokens.to_string(),
                record.total_tokens.to_string(),
                Self::format_cost(record.estimated_cost),
            ])?;
        }

        writer.flush()?;
        let data = writer.into_inner().map_err(|e| {
            ExportError::GenerationFailed(format!("Failed to get CSV data: {}", e))
        })?;

        Ok(String::from_utf8(data).map_err(|e| {
            ExportError::GenerationFailed(format!("Invalid UTF-8 in CSV: {}", e))
        })?)
    }

    fn export_summary(
        &self,
        summary: &CostSummary,
        _options: &ExportOptions,
    ) -> Result<String, ExportError> {
        let mut output = Vec::new();

        // Period Summary Section
        writeln!(output, "# Period Summary")?;
        writeln!(
            output,
            "Start Date,{}",
            Self::format_timestamp(&summary.period.0)
        )?;
        writeln!(
            output,
            "End Date,{}",
            Self::format_timestamp(&summary.period.1)
        )?;
        writeln!(output, "Total Cost,{}", Self::format_cost(summary.total_cost))?;
        writeln!(output, "Total Tokens,{}", summary.total_tokens)?;
        writeln!(output)?;

        // By Provider Section
        writeln!(output, "# Cost by Provider")?;
        writeln!(output, "Provider,Cost,Percentage")?;
        let total_cost = summary.total_cost;
        let mut provider_vec: Vec<_> = summary.breakdown_by_provider.iter().collect();
        provider_vec.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));
        for (provider, cost) in provider_vec {
            let percentage = if total_cost > 0.0 {
                (cost / total_cost) * 100.0
            } else {
                0.0
            };
            writeln!(
                output,
                "{},{},{:.2}%",
                Self::escape_field(provider),
                Self::format_cost(*cost),
                percentage
            )?;
        }
        writeln!(output)?;

        // By Model Section
        writeln!(output, "# Cost by Model")?;
        writeln!(output, "Model,Cost,Percentage")?;
        let mut model_vec: Vec<_> = summary.breakdown_by_model.iter().collect();
        model_vec.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));
        for (model, cost) in model_vec {
            let percentage = if total_cost > 0.0 {
                (cost / total_cost) * 100.0
            } else {
                0.0
            };
            writeln!(
                output,
                "{},{},{:.2}%",
                Self::escape_field(model),
                Self::format_cost(*cost),
                percentage
            )?;
        }
        writeln!(output)?;

        // By Plan Section
        writeln!(output, "# Cost by Plan")?;
        writeln!(output, "Plan ID,Cost,Percentage")?;
        let mut plan_vec: Vec<_> = summary.breakdown_by_plan.iter().collect();
        plan_vec.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));
        for (plan_id, cost) in plan_vec {
            let percentage = if total_cost > 0.0 {
                (cost / total_cost) * 100.0
            } else {
                0.0
            };
            writeln!(
                output,
                "{},{},{:.2}%",
                Self::escape_field(plan_id),
                Self::format_cost(*cost),
                percentage
            )?;
        }
        writeln!(output)?;

        // Top Plans Section
        if !summary.top_plans.is_empty() {
            writeln!(output, "# Top Plans by Cost")?;
            writeln!(output, "Plan ID,Cost")?;
            for (plan_id, cost) in &summary.top_plans {
                writeln!(
                    output,
                    "{},{}",
                    Self::escape_field(plan_id),
                    Self::format_cost(*cost)
                )?;
            }
        }

        Ok(String::from_utf8(output).map_err(|e| {
            ExportError::GenerationFailed(format!("Invalid UTF-8 in CSV: {}", e))
        })?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytics::export::ExportFormat;
    use chrono::Utc;

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
        let exporter = CsvExporter;
        let options = ExportOptions {
            format: ExportFormat::Csv,
            start_date: None,
            end_date: None,
            plan_id: None,
            provider: None,
            output_path: None,
        };
        let result = exporter.export(&[], &options);
        assert!(result.is_ok());
        let csv = result.unwrap();
        assert!(csv.contains("timestamp"));
        assert!(csv.contains("agent_id"));
    }

    #[test]
    fn test_export_single_record() {
        let exporter = CsvExporter;
        let options = ExportOptions {
            format: ExportFormat::Csv,
            start_date: None,
            end_date: None,
            plan_id: None,
            provider: None,
            output_path: None,
        };
        let record = create_test_record();
        let result = exporter.export(&[record], &options);
        assert!(result.is_ok());
        let csv = result.unwrap();
        assert!(csv.contains("agent-1"));
        assert!(csv.contains("REQ-123"));
        assert!(csv.contains("0.0234"));
    }

    #[test]
    fn test_export_special_characters() {
        let exporter = CsvExporter;
        let options = ExportOptions {
            format: ExportFormat::Csv,
            start_date: None,
            end_date: None,
            plan_id: None,
            provider: None,
            output_path: None,
        };
        let mut record = create_test_record();
        record.plan_id = Some("REQ-123, \"Special\"".to_string());
        let result = exporter.export(&[record], &options);
        assert!(result.is_ok());
        let csv = result.unwrap();
        // Should be properly escaped
        assert!(csv.contains("REQ-123"));
    }

    #[test]
    fn test_export_summary() {
        let exporter = CsvExporter;
        let options = ExportOptions {
            format: ExportFormat::Csv,
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
                let mut map = std::collections::HashMap::new();
                map.insert("anthropic".to_string(), 60.0);
                map.insert("openai".to_string(), 40.0);
                map
            },
            breakdown_by_model: {
                let mut map = std::collections::HashMap::new();
                map.insert("claude-3.5-sonnet".to_string(), 60.0);
                map.insert("gpt-4o".to_string(), 40.0);
                map
            },
            breakdown_by_plan: {
                let mut map = std::collections::HashMap::new();
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
        let csv = result.unwrap();
        assert!(csv.contains("Period Summary"));
        assert!(csv.contains("Cost by Provider"));
        assert!(csv.contains("anthropic"));
        assert!(csv.contains("60.0000"));
    }
}

