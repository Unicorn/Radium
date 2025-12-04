//! Status footer component for displaying overall status and help text.

use crate::state::WorkflowStatus;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

/// Status footer component
pub struct StatusFooter;

impl StatusFooter {
    /// Renders the status footer.
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        status: WorkflowStatus,
        status_message: &str,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area);

        // Status
        let status_color = Self::status_color(status);
        let status_text = format!("Status: {}", status.as_str());

        let status_widget = Paragraph::new(status_text)
            .style(Style::default().fg(status_color).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(status_widget, chunks[0]);

        // Message/Help
        let message = Paragraph::new(status_message)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Help "));
        frame.render_widget(message, chunks[1]);
    }

    /// Returns the color for a workflow status.
    fn status_color(status: WorkflowStatus) -> Color {
        match status {
            WorkflowStatus::Idle => Color::Gray,
            WorkflowStatus::Running => Color::Blue,
            WorkflowStatus::Paused => Color::Yellow,
            WorkflowStatus::Completed => Color::Green,
            WorkflowStatus::Failed => Color::Red,
            WorkflowStatus::Cancelled => Color::DarkGray,
        }
    }

    /// Renders a compact status footer in a single line.
    pub fn render_compact(frame: &mut Frame, area: Rect, status: WorkflowStatus, elapsed: f64) {
        let status_color = Self::status_color(status);
        let status_text = format!(
            "{} | {:.1}s | [q] Quit [p] Pause [r] Resume [c] Cancel",
            status.as_str(),
            elapsed
        );

        let widget = Paragraph::new(status_text)
            .style(Style::default().fg(status_color))
            .block(Block::default().borders(Borders::ALL).title(" Status "));
        frame.render_widget(widget, area);
    }

    /// Renders an extended status footer with additional information.
    pub fn render_extended(
        frame: &mut Frame,
        area: Rect,
        status: WorkflowStatus,
        status_message: &str,
        elapsed: f64,
        step: usize,
        total_steps: usize,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),  // Status
                Constraint::Percentage(25),  // Step info
                Constraint::Percentage(25),  // Time
                Constraint::Percentage(25),  // Help
            ])
            .split(area);

        // Status
        let status_color = Self::status_color(status);
        let status_widget = Paragraph::new(status.as_str())
            .style(Style::default().fg(status_color).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Status "));
        frame.render_widget(status_widget, chunks[0]);

        // Step info
        let step_text = format!("{}/{}", step, total_steps);
        let step_widget = Paragraph::new(step_text)
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Step "));
        frame.render_widget(step_widget, chunks[1]);

        // Time
        let time_text = format!("{:.1}s", elapsed);
        let time_widget = Paragraph::new(time_text)
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Elapsed "));
        frame.render_widget(time_widget, chunks[2]);

        // Help
        let help_widget = Paragraph::new(status_message)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Keys "));
        frame.render_widget(help_widget, chunks[3]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_color() {
        assert_eq!(StatusFooter::status_color(WorkflowStatus::Running), Color::Blue);
        assert_eq!(StatusFooter::status_color(WorkflowStatus::Completed), Color::Green);
        assert_eq!(StatusFooter::status_color(WorkflowStatus::Failed), Color::Red);
        assert_eq!(StatusFooter::status_color(WorkflowStatus::Paused), Color::Yellow);
    }
}
