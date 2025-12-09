//! Summary view component for aggregate execution statistics.
//!
//! Displays high-level insights into overall performance, costs, and success rates
//! for all task executions within a requirement.

use crate::state::AggregateStats;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Gauge, Paragraph},
};

/// Summary view component
#[derive(Debug, Clone)]
pub struct SummaryView {
    /// Aggregate statistics to display
    stats: AggregateStats,
    /// Requirement ID being summarized
    requirement_id: String,
}

impl SummaryView {
    /// Creates a new summary view with statistics.
    pub fn new(stats: AggregateStats, requirement_id: String) -> Self {
        Self {
            stats,
            requirement_id,
        }
    }

    /// Handles keyboard input.
    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> Option<Action> {
        match key.code {
            crossterm::event::KeyCode::Esc | crossterm::event::KeyCode::Char('q') => {
                Some(Action::Close)
            }
            crossterm::event::KeyCode::Char('r') => {
                Some(Action::Refresh)
            }
            _ => None,
        }
    }

    /// Renders the summary view.
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let theme = crate::theme::get_theme();

        // Create a centered modal area
        let modal_area = Self::centered_rect(80, 75, area);

        // Clear the background
        frame.render_widget(Clear, modal_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Length(4), // Success Rate
                Constraint::Length(3), // Token Usage
                Constraint::Length(3), // Cost Summary
                Constraint::Length(3), // Performance
                Constraint::Length(2), // Tools
                Constraint::Length(2), // Footer
            ])
            .split(modal_area);

        // Header: Requirement ID and total task count
        let header_text = format!(
            "Summary: {} | Total Tasks: {}",
            self.requirement_id, self.stats.total_tasks
        );
        let header = Paragraph::new(header_text)
            .style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(" Execution Summary "),
            );
        frame.render_widget(header, chunks[0]);

        // Success Rate: Gauge showing completed vs failed ratio
        let success_rate = self.calculate_success_rate();
        let success_color = if success_rate >= 90.0 {
            theme.success
        } else if success_rate >= 70.0 {
            theme.warning
        } else {
            theme.error
        };

        let success_label = format!(
            "{}% | {} completed, {} failed",
            success_rate as u8, self.stats.completed_tasks, self.stats.failed_tasks
        );

        let success_gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(" Success Rate "),
            )
            .gauge_style(Style::default().fg(success_color).bg(theme.bg_panel))
            .percent(success_rate as u16)
            .label(success_label);
        frame.render_widget(success_gauge, chunks[1]);

        // Token Usage: Total tokens with breakdown
        let tokens_text = self.stats.total_tokens.format();
        let tokens = Paragraph::new(tokens_text)
            .style(Style::default().fg(theme.info))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(" Token Usage "),
            );
        frame.render_widget(tokens, chunks[2]);

        // Cost Summary: Total cost with color thresholds
        let cost_color = if self.stats.total_cost > 10.0 {
            theme.error
        } else if self.stats.total_cost > 1.0 {
            theme.warning
        } else {
            theme.success
        };

        let cost_per_task = if self.stats.total_tasks > 0 {
            self.stats.total_cost / self.stats.total_tasks as f64
        } else {
            0.0
        };

        let cost_text = format!(
            "Total: ${:.4} | Per Task: ${:.4}",
            self.stats.total_cost, cost_per_task
        );
        let cost = Paragraph::new(cost_text)
            .style(Style::default().fg(cost_color))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(" Cost Summary "),
            );
        frame.render_widget(cost, chunks[3]);

        // Performance: Average execution time
        let avg_duration = if self.stats.completed_tasks > 0 {
            self.stats.total_duration_secs / self.stats.completed_tasks as u64
        } else {
            0
        };

        let total_duration_str = Self::format_duration(self.stats.total_duration_secs);
        let avg_duration_str = Self::format_duration(avg_duration);

        let performance_text = format!(
            "Total Duration: {} | Average: {}",
            total_duration_str, avg_duration_str
        );
        let performance = Paragraph::new(performance_text)
            .style(Style::default().fg(theme.text))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(" Performance "),
            );
        frame.render_widget(performance, chunks[4]);

        // Tools: Total tools used
        let avg_tools = if self.stats.total_tasks > 0 {
            self.stats.total_tools_used as f64 / self.stats.total_tasks as f64
        } else {
            0.0
        };

        let tools_text = format!(
            "Total Tools: {} | Average: {:.1} per task",
            self.stats.total_tools_used, avg_tools
        );
        let tools = Paragraph::new(tools_text)
            .style(Style::default().fg(theme.info))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(" Tools "),
            );
        frame.render_widget(tools, chunks[5]);

        // Footer: Keyboard shortcuts
        let footer_text = "Esc/q: Close | r: Refresh";
        let footer = Paragraph::new(footer_text)
            .style(Style::default().fg(theme.text_muted))
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(footer, chunks[6]);
    }

    /// Calculates success rate percentage.
    fn calculate_success_rate(&self) -> f64 {
        if self.stats.total_tasks == 0 {
            return 0.0;
        }
        (self.stats.completed_tasks as f64 / self.stats.total_tasks as f64) * 100.0
    }

    /// Formats duration in human-readable format.
    fn format_duration(secs: u64) -> String {
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

    /// Creates a centered rectangle for modal dialogs.
    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}

/// Action to take after handling input
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Close the summary view
    Close,
    /// Refresh statistics
    Refresh,
}

