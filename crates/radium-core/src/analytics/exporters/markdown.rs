//! Markdown exporter for cost data with ASCII charts.

use crate::analytics::export::{CostRecord, CostSummary, ExportError, ExportOptions, Exporter};
use chrono::Utc;
use std::io::Write;

/// Markdown exporter implementation.
pub struct MarkdownExporter;

impl MarkdownExporter {
    /// Format currency amount with $ prefix and 4 decimal places.
    fn format_currency(amount: f64) -> String {
        format!("${:.4}", amount)
    }

    /// Calculate and format percentage.
    fn format_percentage(value: f64, total: f64) -> String {
        if total > 0.0 {
            format!("{:.2}%", (value / total) * 100.0)
        } else {
            "0.00%".to_string()
        }
    }

    /// Generate ASCII bar chart.
    ///
    /// # Arguments
    /// * `data` - Vector of (label, value) pairs
    /// * `max_width` - Maximum width of bars in characters
    ///
    /// # Returns
    /// Multi-line string with bars
    fn generate_bar_chart(data: &[(String, f64)], max_width: usize) -> String {
        if data.is_empty() {
            return "No data available.".to_string();
        }

        let max_value = data
            .iter()
            .map(|(_, v)| *v)
            .fold(0.0f64, |a, b| a.max(b));

        if max_value == 0.0 {
            return "No costs to display.".to_string();
        }

        let mut lines = Vec::new();
        for (label, value) in data {
            let bar_length = if max_value > 0.0 {
                ((value / max_value) * max_width as f64) as usize
            } else {
                0
            };
            let bar = "█".repeat(bar_length);
            lines.push(format!("{}: {} {}", label, bar, Self::format_currency(*value)));
        }

        lines.join("\n")
    }

    /// Create Markdown table row.
    fn create_table_row(cells: &[String]) -> String {
        format!("| {} |", cells.join(" | "))
    }

    /// Create Markdown table header with separator.
    fn create_table_header(headers: &[&str]) -> String {
        let header_row = format!("| {} |", headers.join(" | "));
        let separator = format!("|{}|", "---|".repeat(headers.len()));
        format!("{}\n{}", header_row, separator)
    }
}

impl Exporter for MarkdownExporter {
    fn export(&self, records: &[CostRecord], options: &ExportOptions) -> Result<String, ExportError> {
        let mut output = Vec::new();

        // Header
        writeln!(output, "# Cost Export Report")?;
        writeln!(output)?;
        writeln!(output, "**Generated:** {}", Utc::now().format("%Y-%m-%d %H:%M:%S UTC"))?;
        if let Some(start) = options.start_date {
            writeln!(output, "**Start Date:** {}", start.format("%Y-%m-%d"))?;
        }
        if let Some(end) = options.end_date {
            writeln!(output, "**End Date:** {}", end.format("%Y-%m-%d"))?;
        }
        if let Some(plan_id) = &options.plan_id {
            writeln!(output, "**Plan ID:** {}", plan_id)?;
        }
        if let Some(provider) = &options.provider {
            writeln!(output, "**Provider:** {}", provider)?;
        }
        writeln!(output)?;
        writeln!(output, "**Total Records:** {}", records.len())?;
        writeln!(output)?;

        // Table
        writeln!(output, "## Detailed Cost Records")?;
        writeln!(output)?;
        writeln!(
            output,
            "{}",
            Self::create_table_header(&[
                "Timestamp",
                "Agent ID",
                "Plan ID",
                "Model",
                "Provider",
                "Tokens",
                "Cost"
            ])
        )?;

        for record in records {
            let timestamp_str = record.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
            let plan_id_str = record.plan_id.as_deref().unwrap_or("-");
            let model_str = record.model.as_deref().unwrap_or("-");
            let provider_str = record.provider.as_deref().unwrap_or("-");
            writeln!(
                output,
                "{}",
                Self::create_table_row(&[
                    timestamp_str,
                    record.agent_id.clone(),
                    plan_id_str.to_string(),
                    model_str.to_string(),
                    provider_str.to_string(),
                    record.total_tokens.to_string(),
                    Self::format_currency(record.estimated_cost),
                ])
            )?;
        }

        Ok(String::from_utf8(output).map_err(|e| {
            ExportError::GenerationFailed(format!("Invalid UTF-8 in Markdown: {}", e))
        })?)
    }

    fn export_summary(
        &self,
        summary: &CostSummary,
        options: &ExportOptions,
    ) -> Result<String, ExportError> {
        let mut output = Vec::new();

        // Header
        writeln!(output, "# Cost Summary Report")?;
        writeln!(output)?;
        writeln!(
            output,
            "**Period:** {} to {}",
            summary.period.0.format("%Y-%m-%d"),
            summary.period.1.format("%Y-%m-%d")
        )?;
        if let Some(plan_id) = &options.plan_id {
            writeln!(output, "**Filtered by Plan:** {}", plan_id)?;
        }
        if let Some(provider) = &options.provider {
            writeln!(output, "**Filtered by Provider:** {}", provider)?;
        }
        writeln!(output)?;

        // Summary Section
        writeln!(output, "## Summary")?;
        writeln!(output)?;
        writeln!(
            output,
            "- **Total Cost:** {}",
            Self::format_currency(summary.total_cost)
        )?;
        writeln!(output, "- **Total Tokens:** {}", summary.total_tokens)?;
        writeln!(output)?;

        // Cost by Provider
        if !summary.breakdown_by_provider.is_empty() {
            writeln!(output, "## Cost by Provider")?;
            writeln!(output)?;
            let mut provider_vec: Vec<_> = summary.breakdown_by_provider.iter().collect();
            provider_vec.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));
            let chart_data: Vec<(String, f64)> = provider_vec
                .iter()
                .map(|(k, v)| (k.clone(), *v))
                .take(10)
                .collect();
            writeln!(output, "{}", Self::generate_bar_chart(&chart_data, 50))?;
            writeln!(output)?;
        }

        // Cost by Model
        if !summary.breakdown_by_model.is_empty() {
            writeln!(output, "## Cost by Model")?;
            writeln!(output)?;
            let mut model_vec: Vec<_> = summary.breakdown_by_model.iter().collect();
            model_vec.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));
            let chart_data: Vec<(String, f64)> = model_vec
                .iter()
                .map(|(k, v)| (k.clone(), **v))
                .take(10)
                .collect();
            writeln!(output, "{}", Self::generate_bar_chart(&chart_data, 50))?;
            writeln!(output)?;
        }

        // Top Plans
        if !summary.top_plans.is_empty() {
            writeln!(output, "## Top Plans by Cost")?;
            writeln!(output)?;
            writeln!(
                output,
                "{}",
                Self::create_table_header(&["Plan ID", "Cost", "Percentage"])
            )?;
            let total_cost = summary.total_cost;
            for (plan_id, cost) in &summary.top_plans {
                writeln!(
                    output,
                    "{}",
                    Self::create_table_row(&[
                        plan_id.clone(),
                        Self::format_currency(*cost),
                        Self::format_percentage(*cost, total_cost),
                    ])
                )?;
            }
            writeln!(output)?;
        }

        // Detailed Breakdown Tables
        if !summary.breakdown_by_provider.is_empty() {
            writeln!(output, "### Provider Breakdown")?;
            writeln!(output)?;
            writeln!(
                output,
                "{}",
                Self::create_table_header(&["Provider", "Cost", "Percentage"])
            )?;
            let mut provider_vec: Vec<_> = summary.breakdown_by_provider.iter().collect();
            provider_vec.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));
            let total_cost = summary.total_cost;
            for (provider, cost) in provider_vec {
                writeln!(
                    output,
                    "{}",
                    Self::create_table_row(&[
                        provider.clone(),
                        Self::format_currency(*cost),
                        Self::format_percentage(*cost, total_cost),
                    ])
                )?;
            }
            writeln!(output)?;
        }

        if !summary.breakdown_by_model.is_empty() {
            writeln!(output, "### Model Breakdown")?;
            writeln!(output)?;
            writeln!(
                output,
                "{}",
                Self::create_table_header(&["Model", "Cost", "Percentage"])
            )?;
            let mut model_vec: Vec<_> = summary.breakdown_by_model.iter().collect();
            model_vec.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));
            let total_cost = summary.total_cost;
            for (model, cost) in model_vec {
                writeln!(
                    output,
                    "{}",
                    Self::create_table_row(&[
                        model.clone(),
                        Self::format_currency(*cost),
                        Self::format_percentage(*cost, total_cost),
                    ])
                )?;
            }
            writeln!(output)?;
        }

        Ok(String::from_utf8(output).map_err(|e| {
            ExportError::GenerationFailed(format!("Invalid UTF-8 in Markdown: {}", e))
        })?)
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
        }
    }

    #[test]
    fn test_export_empty_records() {
        let exporter = MarkdownExporter;
        let options = ExportOptions {
            format: ExportFormat::Markdown,
            start_date: None,
            end_date: None,
            plan_id: None,
            provider: None,
            output_path: None,
        };
        let result = exporter.export(&[], &options);
        assert!(result.is_ok());
        let md = result.unwrap();
        assert!(md.contains("# Cost Export Report"));
        assert!(md.contains("Total Records"));
    }

    #[test]
    fn test_export_single_record() {
        let exporter = MarkdownExporter;
        let options = ExportOptions {
            format: ExportFormat::Markdown,
            start_date: None,
            end_date: None,
            plan_id: None,
            provider: None,
            output_path: None,
        };
        let record = create_test_record();
        let result = exporter.export(&[record], &options);
        assert!(result.is_ok());
        let md = result.unwrap();
        assert!(md.contains("agent-1"));
        assert!(md.contains("REQ-123"));
        assert!(md.contains("$0.0234"));
    }

    #[test]
    fn test_export_summary() {
        let exporter = MarkdownExporter;
        let options = ExportOptions {
            format: ExportFormat::Markdown,
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
        let md = result.unwrap();
        assert!(md.contains("# Cost Summary Report"));
        assert!(md.contains("$100.0000"));
        assert!(md.contains("Cost by Provider"));
        assert!(md.contains("█")); // Bar chart character
    }

    #[test]
    fn test_bar_chart_generation() {
        let data = vec![
            ("Provider A".to_string(), 50.0),
            ("Provider B".to_string(), 30.0),
            ("Provider C".to_string(), 20.0),
        ];
        let chart = MarkdownExporter::generate_bar_chart(&data, 50);
        assert!(chart.contains("Provider A"));
        assert!(chart.contains("Provider B"));
        assert!(chart.contains("$50.0000"));
    }

    #[test]
    fn test_currency_formatting() {
        assert_eq!(MarkdownExporter::format_currency(123.456789), "$123.4568");
        assert_eq!(MarkdownExporter::format_currency(0.0), "$0.0000");
    }

    #[test]
    fn test_percentage_formatting() {
        assert_eq!(MarkdownExporter::format_percentage(25.0, 100.0), "25.00%");
        assert_eq!(MarkdownExporter::format_percentage(0.0, 100.0), "0.00%");
        assert_eq!(MarkdownExporter::format_percentage(50.0, 0.0), "0.00%");
    }
}

