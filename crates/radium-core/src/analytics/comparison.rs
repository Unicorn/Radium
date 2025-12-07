//! Session comparison and diff functionality.

use std::time::Duration;

use super::report::SessionReport;
use super::session::SessionMetrics;

/// Comparison between two session reports.
#[derive(Debug, Clone)]
pub struct SessionComparison {
    /// First session ID
    pub session_a_id: String,
    /// Second session ID
    pub session_b_id: String,
    /// Delta in total tokens (input + output)
    pub token_delta: i64,
    /// Delta in total cost
    pub cost_delta: f64,
    /// Delta in wall time
    pub wall_time_delta: Duration,
    /// Delta in agent active time
    pub agent_active_time_delta: Duration,
    /// Delta in tool calls
    pub tool_calls_delta: i64,
    /// Delta in success rate (percentage points)
    pub success_rate_delta: f64,
    /// Delta in lines added
    pub lines_added_delta: i64,
    /// Delta in lines removed
    pub lines_removed_delta: i64,
    /// Metrics from session A
    pub metrics_a: SessionMetrics,
    /// Metrics from session B
    pub metrics_b: SessionMetrics,
}

impl SessionComparison {
    /// Create a new session comparison from two reports.
    pub fn new(report_a: &SessionReport, report_b: &SessionReport) -> Self {
        let metrics_a = &report_a.metrics;
        let metrics_b = &report_b.metrics;

        // Calculate token deltas
        let (tokens_a_in, tokens_a_out) = metrics_a.total_tokens();
        let (tokens_b_in, tokens_b_out) = metrics_b.total_tokens();
        let total_tokens_a = tokens_a_in + tokens_a_out;
        let total_tokens_b = tokens_b_in + tokens_b_out;
        let token_delta = total_tokens_b as i64 - total_tokens_a as i64;

        // Calculate cost delta
        let cost_delta = metrics_b.total_cost - metrics_a.total_cost;

        // Calculate time deltas
        let wall_time_delta = if metrics_b.wall_time > metrics_a.wall_time {
            metrics_b.wall_time - metrics_a.wall_time
        } else {
            metrics_a.wall_time - metrics_b.wall_time
        };
        let agent_active_time_delta = if metrics_b.agent_active_time > metrics_a.agent_active_time {
            metrics_b.agent_active_time - metrics_a.agent_active_time
        } else {
            metrics_a.agent_active_time - metrics_b.agent_active_time
        };

        // Calculate tool calls delta
        let tool_calls_delta = metrics_b.tool_calls as i64 - metrics_a.tool_calls as i64;

        // Calculate success rate delta (percentage points)
        let success_rate_delta = metrics_b.success_rate() - metrics_a.success_rate();

        // Calculate code changes deltas
        let lines_added_delta = metrics_b.lines_added - metrics_a.lines_added;
        let lines_removed_delta = metrics_b.lines_removed - metrics_a.lines_removed;

        Self {
            session_a_id: metrics_a.session_id.clone(),
            session_b_id: metrics_b.session_id.clone(),
            token_delta,
            cost_delta,
            wall_time_delta,
            agent_active_time_delta,
            tool_calls_delta,
            success_rate_delta,
            lines_added_delta,
            lines_removed_delta,
            metrics_a: metrics_a.clone(),
            metrics_b: metrics_b.clone(),
        }
    }

    /// Calculate percentage change for a value.
    fn percentage_change(old: f64, new: f64) -> f64 {
        if old == 0.0 {
            if new == 0.0 {
                0.0
            } else {
                100.0 // 100% increase from zero
            }
        } else {
            ((new - old) / old) * 100.0
        }
    }

    /// Get token percentage change.
    pub fn token_percentage_change(&self) -> f64 {
        let (tokens_a_in, tokens_a_out) = self.metrics_a.total_tokens();
        let total_a = (tokens_a_in + tokens_a_out) as f64;
        let total_b = total_a + self.token_delta as f64;
        Self::percentage_change(total_a, total_b)
    }

    /// Get cost percentage change.
    pub fn cost_percentage_change(&self) -> f64 {
        Self::percentage_change(self.metrics_a.total_cost, self.metrics_b.total_cost)
    }

    /// Get wall time percentage change.
    pub fn wall_time_percentage_change(&self) -> f64 {
        let time_a = self.metrics_a.wall_time.as_secs_f64();
        let time_b = self.metrics_b.wall_time.as_secs_f64();
        Self::percentage_change(time_a, time_b)
    }
}

/// Formatter for session comparisons.
pub struct ComparisonFormatter;

impl ComparisonFormatter {
    /// Format a session comparison as human-readable text.
    pub fn format(&self, comparison: &SessionComparison) -> String {
        let mut output = String::new();

        output.push_str("Session Comparison\n");
        output.push_str("═══════════════════\n\n");
        output.push_str(&format!("Session A: {}\n", comparison.session_a_id));
        output.push_str(&format!("Session B: {}\n\n", comparison.session_b_id));

        // Token comparison
        output.push_str("Token Usage\n");
        output.push_str("───────────\n");
        let (tokens_a_in, tokens_a_out) = comparison.metrics_a.total_tokens();
        let (tokens_b_in, tokens_b_out) = comparison.metrics_b.total_tokens();
        output.push_str(&format!(
            "  Session A: {} input, {} output (total: {})\n",
            tokens_a_in,
            tokens_a_out,
            tokens_a_in + tokens_a_out
        ));
        output.push_str(&format!(
            "  Session B: {} input, {} output (total: {})\n",
            tokens_b_in,
            tokens_b_out,
            tokens_b_in + tokens_b_out
        ));
        output.push_str(&format!(
            "  Delta: {} ({})\n\n",
            Self::format_delta(comparison.token_delta),
            Self::format_percentage(comparison.token_percentage_change())
        ));

        // Cost comparison
        output.push_str("Cost\n");
        output.push_str("────\n");
        output.push_str(&format!("  Session A: ${:.4}\n", comparison.metrics_a.total_cost));
        output.push_str(&format!("  Session B: ${:.4}\n", comparison.metrics_b.total_cost));
        output.push_str(&format!(
            "  Delta: {} ({})\n\n",
            Self::format_delta_f64(comparison.cost_delta),
            Self::format_percentage(comparison.cost_percentage_change())
        ));

        // Time comparison
        output.push_str("Performance\n");
        output.push_str("───────────\n");
        output.push_str(&format!(
            "  Wall Time: {} ({})\n",
            Self::format_duration_delta(comparison.wall_time_delta, comparison.metrics_a.wall_time, comparison.metrics_b.wall_time),
            Self::format_percentage(comparison.wall_time_percentage_change())
        ));
        output.push_str(&format!(
            "  Agent Active: {} ({})\n",
            Self::format_duration_delta(comparison.agent_active_time_delta, comparison.metrics_a.agent_active_time, comparison.metrics_b.agent_active_time),
            Self::format_percentage(Self::percentage_change(
                comparison.metrics_a.agent_active_time.as_secs_f64(),
                comparison.metrics_b.agent_active_time.as_secs_f64()
            ))
        ));
        output.push_str("\n");

        // Tool calls comparison
        output.push_str("Tool Calls\n");
        output.push_str("──────────\n");
        output.push_str(&format!("  Session A: {}\n", comparison.metrics_a.tool_calls));
        output.push_str(&format!("  Session B: {}\n", comparison.metrics_b.tool_calls));
        output.push_str(&format!(
            "  Delta: {} ({})\n",
            Self::format_delta(comparison.tool_calls_delta),
            Self::format_percentage(Self::percentage_change(
                comparison.metrics_a.tool_calls as f64,
                comparison.metrics_b.tool_calls as f64
            ))
        ));
        output.push_str(&format!(
            "  Success Rate: {:.1}% → {:.1}% ({})\n\n",
            comparison.metrics_a.success_rate(),
            comparison.metrics_b.success_rate(),
            Self::format_delta_f64(comparison.success_rate_delta)
        ));

        // Code changes comparison
        output.push_str("Code Changes\n");
        output.push_str("────────────\n");
        output.push_str(&format!(
            "  Session A: +{} -{}\n",
            comparison.metrics_a.lines_added, comparison.metrics_a.lines_removed
        ));
        output.push_str(&format!(
            "  Session B: +{} -{}\n",
            comparison.metrics_b.lines_added, comparison.metrics_b.lines_removed
        ));
        output.push_str(&format!(
            "  Delta: {} / {}\n",
            Self::format_delta(comparison.lines_added_delta),
            Self::format_delta(comparison.lines_removed_delta)
        ));

        output
    }

    fn format_delta(delta: i64) -> String {
        if delta >= 0 {
            format!("+{}", delta)
        } else {
            delta.to_string()
        }
    }

    fn format_delta_f64(delta: f64) -> String {
        if delta >= 0.0 {
            format!("+{:.4}", delta)
        } else {
            format!("{:.4}", delta)
        }
    }

    fn format_percentage(pct: f64) -> String {
        if pct >= 0.0 {
            format!("+{:.1}%", pct)
        } else {
            format!("{:.1}%", pct)
        }
    }

    fn format_duration_delta(delta: Duration, time_a: Duration, time_b: Duration) -> String {
        let direction = if time_b > time_a { "+" } else { "-" };
        format!("{}{}", direction, Self::format_duration(delta))
    }

    fn format_duration(d: Duration) -> String {
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

    fn percentage_change(old: f64, new: f64) -> f64 {
        if old == 0.0 {
            if new == 0.0 {
                0.0
            } else {
                100.0
            }
        } else {
            ((new - old) / old) * 100.0
        }
    }
}

