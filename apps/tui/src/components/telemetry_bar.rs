//! Telemetry bar component for displaying token usage and costs.

use crate::state::TelemetryState;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge, Paragraph},
};

/// Telemetry bar component
pub struct TelemetryBar;

impl TelemetryBar {
    /// Renders the telemetry bar.
    pub fn render(frame: &mut Frame, area: Rect, telemetry: &TelemetryState) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40), // Token info
                Constraint::Percentage(30), // Cost info
                Constraint::Percentage(30), // Model info
            ])
            .split(area);

        // Token info
        let token_text = format!(
            "Tokens\n{}in / {}out = {}",
            format_number(telemetry.overall_tokens.input_tokens),
            format_number(telemetry.overall_tokens.output_tokens),
            format_number(telemetry.overall_tokens.total())
        );

        let tokens = Paragraph::new(token_text)
            .style(Style::default().fg(Color::Blue))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(tokens, chunks[0]);

        // Cost info
        let cost_text = format!("Cost\n${:.4}", telemetry.overall_cost);
        let cost_color = if telemetry.overall_cost > 1.0 {
            Color::Red
        } else if telemetry.overall_cost > 0.1 {
            Color::Yellow
        } else {
            Color::Green
        };

        let cost = Paragraph::new(cost_text)
            .style(Style::default().fg(cost_color))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(cost, chunks[1]);

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

        let model = Paragraph::new(model_info)
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Model "));
        frame.render_widget(model, chunks[2]);
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
        let summary = format!(
            "Tokens: {} | Cost: ${:.4} | Model: {}/{}",
            format_number(telemetry.overall_tokens.total()),
            telemetry.overall_cost,
            telemetry.provider.as_deref().unwrap_or("?"),
            telemetry.model.as_deref().unwrap_or("?")
        );

        let widget = Paragraph::new(summary)
            .style(Style::default().fg(Color::Blue))
            .block(Block::default().borders(Borders::ALL).title(" Telemetry "));
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
