//! Telemetry bar component for displaying token usage and costs.

use crate::state::{TelemetryState, WorkflowStatus};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge, Paragraph, BarChart, Bar},
};

/// Telemetry bar component
pub struct TelemetryBar;

impl TelemetryBar {
    /// Renders the telemetry bar with runtime and status.
    pub fn render(frame: &mut Frame, area: Rect, telemetry: &TelemetryState) {
        Self::render_with_status(frame, area, telemetry, None, None, None)
    }

    /// Renders the telemetry bar with runtime, status, and loop iteration.
    pub fn render_with_status(
        frame: &mut Frame,
        area: Rect,
        telemetry: &TelemetryState,
        runtime: Option<&str>,
        status: Option<WorkflowStatus>,
        loop_iteration: Option<usize>,
    ) {
        let theme = crate::theme::get_theme();
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(12), // Runtime
                Constraint::Length(15), // Status
                Constraint::Percentage(25), // Token info
                Constraint::Percentage(20), // Cost info
                Constraint::Percentage(25), // Model info
            ])
            .split(area);

        // Runtime
        let runtime_text = runtime.unwrap_or("00:00:00");
        let runtime_widget = Paragraph::new(runtime_text)
            .style(Style::default().fg(theme.text))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_panel))
                    .title(" Runtime "),
            );
        frame.render_widget(runtime_widget, chunks[0]);

        // Status with animated indicator
        let status_text = if let Some(status) = status {
            match status {
                WorkflowStatus::Running => "Running...".to_string(),
                WorkflowStatus::Completed => "● Completed".to_string(),
                WorkflowStatus::Failed => "⏹ Failed".to_string(),
                WorkflowStatus::Paused => "⏸ Paused".to_string(),
                WorkflowStatus::Cancelled => "⏹ Cancelled".to_string(),
                WorkflowStatus::Idle => "Idle".to_string(),
            }
        } else {
            "Idle".to_string()
        };

        let status_color = status
            .map(|s| Self::status_color(s, &theme))
            .unwrap_or(theme.text_muted);

        let status_widget = Paragraph::new(status_text)
            .style(Style::default().fg(status_color))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_panel))
                    .title(" Status "),
            );
        frame.render_widget(status_widget, chunks[1]);

        // Token info with breakdown: "1,234 in / 5,678 out (6,912 total)"
        let total_tokens = telemetry.overall_tokens.total();
        let token_text = if telemetry.overall_tokens.cached_tokens > 0 {
            format!(
                "{} in / {} out\n({} total, {} cached)",
                format_number(telemetry.overall_tokens.input_tokens),
                format_number(telemetry.overall_tokens.output_tokens),
                format_number(total_tokens),
                format_number(telemetry.overall_tokens.cached_tokens)
            )
        } else {
            format!(
                "{} in / {} out\n({} total)",
                format_number(telemetry.overall_tokens.input_tokens),
                format_number(telemetry.overall_tokens.output_tokens),
                format_number(total_tokens)
            )
        };

        let tokens = Paragraph::new(token_text)
            .style(Style::default().fg(theme.info))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_panel))
                    .title(" Tokens "),
            );
        frame.render_widget(tokens, chunks[2]);

        // Cost info
        let cost_text = format!("${:.4}", telemetry.overall_cost);
        let cost_color = if telemetry.overall_cost > 1.0 {
            theme.error
        } else if telemetry.overall_cost > 0.1 {
            theme.warning
        } else {
            theme.success
        };

        let cost = Paragraph::new(cost_text)
            .style(Style::default().fg(cost_color))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_panel))
                    .title(" Cost "),
            );
        frame.render_widget(cost, chunks[3]);

        // Model info
        let model_info = if let Some(ref model) = telemetry.model {
            if let Some(ref provider) = telemetry.provider {
                format!("{}\n{}", provider, model)
            } else {
                model.clone()
            }
        } else {
            "No model\nselected".to_string()
        };

        // Add loop iteration if provided
        let model_text = if let Some(iter) = loop_iteration {
            format!("{}\nLoop: {}", model_info, iter)
        } else {
            model_info
        };

        let model = Paragraph::new(model_text)
            .style(Style::default().fg(theme.primary))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_panel))
                    .title(" Model "),
            );
        frame.render_widget(model, chunks[4]);
    }

    /// Returns the color for a workflow status.
    fn status_color(status: WorkflowStatus, theme: &crate::theme::RadiumTheme) -> Color {
        match status {
            WorkflowStatus::Running => theme.info,
            WorkflowStatus::Completed => theme.success,
            WorkflowStatus::Failed => theme.error,
            WorkflowStatus::Paused => theme.warning,
            WorkflowStatus::Cancelled => theme.text_muted,
            WorkflowStatus::Idle => theme.text_muted,
        }
    }

    /// Renders an extended telemetry view with progress bar.
    pub fn render_extended(
        frame: &mut Frame,
        area: Rect,
        telemetry: &TelemetryState,
        progress: u8,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Progress bar
                Constraint::Length(4), // Telemetry info
            ])
            .split(area);

        // Progress bar
        let progress_label = format!("Workflow Progress: {}%", progress);
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(" Progress "))
            .gauge_style(
                Style::default()
                    .fg(if progress == 100 { Color::Green } else { Color::Blue })
                    .bg(Color::Black),
            )
            .label(progress_label)
            .ratio(f64::from(progress) / 100.0);
        frame.render_widget(gauge, chunks[0]);

        // Telemetry info
        Self::render(frame, chunks[1], telemetry);
    }

    /// Renders a compact telemetry summary in a single line.
    pub fn render_compact(frame: &mut Frame, area: Rect, telemetry: &TelemetryState) {
        let token_breakdown = format!(
            "{} in / {} out ({} total)",
            format_number(telemetry.overall_tokens.input_tokens),
            format_number(telemetry.overall_tokens.output_tokens),
            format_number(telemetry.overall_tokens.total())
        );
        let summary = format!(
            "Tokens: {} | Cost: ${:.4} | Model: {}/{}",
            token_breakdown,
            telemetry.overall_cost,
            telemetry.provider.as_deref().unwrap_or("?"),
            telemetry.model.as_deref().unwrap_or("?")
        );

        let widget = Paragraph::new(summary)
            .style(Style::default().fg(Color::Blue))
            .block(Block::default().borders(Borders::ALL).title(" Telemetry "));
        frame.render_widget(widget, area);
    }

    /// Renders provider breakdown with horizontal bars.
    pub fn render_provider_breakdown(frame: &mut Frame, area: Rect, telemetry: &TelemetryState) {
        let theme = crate::theme::get_theme();
        
        if telemetry.provider_breakdown.is_empty() {
            let widget = Paragraph::new("No provider data available")
                .style(Style::default().fg(theme.text_muted))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme.border))
                        .title(" Provider Breakdown ")
                );
            frame.render_widget(widget, area);
            return;
        }

        // Create bar chart data
        let total_cost: f64 = telemetry.provider_breakdown.iter().map(|b| b.total_cost).sum();
        let max_cost = telemetry.provider_breakdown.iter()
            .map(|b| b.total_cost)
            .fold(0.0, f64::max);

        let bars: Vec<Bar> = telemetry.provider_breakdown.iter().enumerate().map(|(i, breakdown)| {
            let color = match i % 3 {
                0 => Color::Blue,      // OpenAI
                1 => Color::Magenta,   // Anthropic
                2 => Color::Green,     // Gemini
                _ => Color::Yellow,
            };
            Bar::default()
                .value(breakdown.total_cost as u64)
                .label(format!("{} {:.1}%", breakdown.provider, breakdown.percentage).into())
                .style(Style::default().fg(color))
        }).collect();

        let labels: Vec<String> = telemetry.provider_breakdown.iter().map(|b| {
            format!("{}\n${:.2}", b.provider, b.total_cost)
        }).collect();

        let chart = BarChart::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(" Provider Breakdown ")
            )
            .data(&bars)
            .bar_width(3)
            .bar_gap(1)
            .max(max_cost as u64);

        frame.render_widget(chart, area);
    }

    /// Renders team attribution breakdown.
    pub fn render_team_breakdown(frame: &mut Frame, area: Rect, telemetry: &TelemetryState) {
        let theme = crate::theme::get_theme();
        
        if telemetry.team_breakdown.is_empty() {
            let widget = Paragraph::new("No team attribution data available")
                .style(Style::default().fg(theme.text_muted))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme.border))
                        .title(" Team Attribution ")
                );
            frame.render_widget(widget, area);
            return;
        }

        // Build team list text
        let mut lines = Vec::new();
        for (i, breakdown) in telemetry.team_breakdown.iter().take(5).enumerate() {
            let project = breakdown.project_name.as_deref().unwrap_or("N/A");
            let line = format!(
                "{}. {} / {}\n   ${:.4} ({} execs)",
                i + 1,
                breakdown.team_name,
                project,
                breakdown.total_cost,
                breakdown.execution_count
            );
            lines.push(line);
        }

        let text = lines.join("\n\n");
        let widget = Paragraph::new(text)
            .style(Style::default().fg(theme.text))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(" Team Attribution (Top 5) ")
            );

        frame.render_widget(widget, area);
    }
}

/// Formats a number with commas.
fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let mut count = 0;

    for c in s.chars().rev() {
        if count > 0 && count % 3 == 0 {
            result.push(',');
        }
        result.push(c);
        count += 1;
    }

    result.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(999), "999");
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1234567), "1,234,567");
    }
}
