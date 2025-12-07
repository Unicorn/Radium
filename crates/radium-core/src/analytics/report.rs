//! Session report generation and formatting.

use std::fmt::Write;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::session::SessionMetrics;

/// Complete session report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionReport {
    /// Session metrics
    pub metrics: SessionMetrics,
    /// Report generation timestamp
    pub generated_at: DateTime<Utc>,
}

impl SessionReport {
    /// Create a new session report.
    pub fn new(metrics: SessionMetrics) -> Self {
        Self { metrics, generated_at: Utc::now() }
    }
}

/// Formatter for session reports.
pub struct ReportFormatter;

impl ReportFormatter {
    /// Format a session report as human-readable text.
    pub fn format(&self, report: &SessionReport) -> String {
        let m = &report.metrics;
        let mut output = String::new();

        // Interaction Summary
        output.push_str("Interaction Summary\n");
        writeln!(output, "Session ID:                 {}", m.session_id).unwrap();
        writeln!(
            output,
            "Tool Calls:                 {} ( ✓ {} x {} )",
            m.tool_calls, m.successful_tool_calls, m.failed_tool_calls
        )
        .unwrap();
        writeln!(output, "Success Rate:               {:.1}%", m.success_rate()).unwrap();
        writeln!(output, "Code Changes:               +{} -{}", m.lines_added, m.lines_removed)
            .unwrap();
        output.push('\n');

        // Performance
        output.push_str("Performance\n");
        writeln!(output, "Wall Time:                  {}", Self::format_duration(m.wall_time))
            .unwrap();
        writeln!(
            output,
            "Agent Active:               {}",
            Self::format_duration(m.agent_active_time)
        )
        .unwrap();
        writeln!(
            output,
            "  » API Time:               {} ({:.1}%)",
            Self::format_duration(m.api_time),
            if m.agent_active_time.as_secs() > 0 {
                (m.api_time.as_secs_f64() / m.agent_active_time.as_secs_f64()) * 100.0
            } else {
                0.0
            }
        )
        .unwrap();
        writeln!(
            output,
            "  » Tool Time:              {} ({:.1}%)",
            Self::format_duration(m.tool_time),
            if m.agent_active_time.as_secs() > 0 {
                (m.tool_time.as_secs_f64() / m.agent_active_time.as_secs_f64()) * 100.0
            } else {
                0.0
            }
        )
        .unwrap();
        output.push('\n');

        // Model Usage
        output.push_str("Model Usage                  Reqs   Input Tokens  Output Tokens\n");
        output.push_str("───────────────────────────────────────────────────────────────\n");
        for (model, stats) in &m.model_usage {
            writeln!(
                output,
                "{:<28} {:>6} {:>14} {:>14}",
                model, stats.requests, stats.input_tokens, stats.output_tokens
            )
            .unwrap();
        }
        output.push('\n');

        // Cache savings
        if m.total_cached_tokens > 0 {
            let (total_input, _) = m.total_tokens();
            let cache_percentage = if total_input > 0 {
                (m.total_cached_tokens as f64 / total_input as f64) * 100.0
            } else {
                0.0
            };
            writeln!(
                output,
                "Savings Highlight: {} ({:.1}%) of input tokens were served from",
                Self::format_number(m.total_cached_tokens),
                cache_percentage
            )
            .unwrap();
            output.push_str("cache, reducing costs.\n");
            output.push('\n');
        }

        // Tip
        output.push_str("» Tip: For a full token breakdown, run `rad stats model`.\n");

        output
    }

    /// Format duration in human-readable format.
    fn format_duration(d: std::time::Duration) -> String {
        let secs = d.as_secs();
        let hours = secs / 3600;
        let minutes = (secs % 3600) / 60;
        let seconds = secs % 60;

        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }

    /// Format large numbers with commas.
    fn format_number(n: u64) -> String {
        let s = n.to_string();
        let mut result = String::new();
        let chars: Vec<char> = s.chars().rev().collect();

        for (i, ch) in chars.iter().enumerate() {
            if i > 0 && i % 3 == 0 {
                result.push(',');
            }
            result.push(*ch);
        }

        result.chars().rev().collect()
    }

    /// Format report as JSON.
    ///
    /// # Arguments
    /// * `report` - The session report to format
    /// * `compact` - If true, use compact JSON (no whitespace). If false, use pretty-printed JSON.
    pub fn format_json(&self, report: &SessionReport, compact: bool) -> anyhow::Result<String> {
        if compact {
            Ok(serde_json::to_string(report)?)
        } else {
            Ok(serde_json::to_string_pretty(report)?)
        }
    }
}
