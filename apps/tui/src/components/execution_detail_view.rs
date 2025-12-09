//! Execution detail view component for displaying detailed task execution metrics.
//!
//! Shows comprehensive information about a single task execution including timeline,
//! token usage, costs, and error information.

use crate::state::ExecutionRecord;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph},
};

/// Execution detail view component
#[derive(Debug, Clone)]
pub struct ExecutionDetailView {
    /// The execution record to display
    record: ExecutionRecord,
    /// Scroll position for long content
    scroll_position: usize,
}

impl ExecutionDetailView {
    /// Creates a new execution detail view for a record.
    pub fn new(record: ExecutionRecord) -> Self {
        Self {
            record,
            scroll_position: 0,
        }
    }

    /// Scrolls content up.
    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll_position = self.scroll_position.saturating_sub(amount);
    }

    /// Scrolls content down.
    pub fn scroll_down(&mut self, amount: usize) {
        // Scroll position will be limited by content height in render
        self.scroll_position += amount;
    }

    /// Scrolls to top.
    pub fn scroll_to_top(&mut self) {
        self.scroll_position = 0;
    }

    /// Scrolls to bottom.
    pub fn scroll_to_bottom(&mut self) {
        // Will be set in render based on content
        self.scroll_position = usize::MAX;
    }

    /// Handles keyboard input.
    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> Option<Action> {
        match key.code {
            crossterm::event::KeyCode::Up | crossterm::event::KeyCode::Char('k') => {
                self.scroll_up(1);
                None
            }
            crossterm::event::KeyCode::Down | crossterm::event::KeyCode::Char('j') => {
                self.scroll_down(1);
                None
            }
            crossterm::event::KeyCode::PageUp => {
                self.scroll_up(10);
                None
            }
            crossterm::event::KeyCode::PageDown => {
                self.scroll_down(10);
                None
            }
            crossterm::event::KeyCode::Home => {
                self.scroll_to_top();
                None
            }
            crossterm::event::KeyCode::End => {
                self.scroll_to_bottom();
                None
            }
            crossterm::event::KeyCode::Esc | crossterm::event::KeyCode::Char('q') => {
                Some(Action::Close)
            }
            _ => None,
        }
    }

    /// Renders the execution detail view.
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let theme = crate::theme::get_theme();

        // Create a centered modal area
        let modal_area = Self::centered_rect(85, 80, area);

        // Clear the background
        frame.render_widget(Clear, modal_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Length(3), // Status
                Constraint::Length(4), // Timeline
                Constraint::Length(3), // Tokens
                Constraint::Length(3), // Cost
                Constraint::Length(3), // Execution Context
                Constraint::Length(2), // Tools
                Constraint::Min(3),    // Error (if failed) or empty
                Constraint::Length(2), // Footer
            ])
            .split(modal_area);

        // Header: Task name and requirement ID
        let header_text = format!(
            "Task: {} | Requirement: {}",
            self.record.task_name, self.record.requirement_id
        );
        let header = Paragraph::new(header_text)
            .style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(" Execution Details "),
            );
        frame.render_widget(header, chunks[0]);

        // Status with color coding
        let status_color = Self::status_color(self.record.status, &theme);
        let status_text = format!("Status: {}", self.record.status.as_str());
        let status = Paragraph::new(status_text)
            .style(Style::default().fg(status_color).add_modifier(Modifier::BOLD))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(" Status "),
            );
        frame.render_widget(status, chunks[1]);

        // Timeline: Start time, end time, duration
        let start_time_str = Self::format_timestamp(self.record.start_time);
        let end_time_str = self
            .record
            .end_time
            .map(|t| Self::format_timestamp(t))
            .unwrap_or_else(|| "N/A".to_string());
        let duration_str = self
            .record
            .duration_secs
            .map(|d| Self::format_duration(d))
            .unwrap_or_else(|| "N/A".to_string());

        let timeline_text = format!(
            "Start: {} | End: {} | Duration: {}",
            start_time_str, end_time_str, duration_str
        );
        let timeline = Paragraph::new(timeline_text)
            .style(Style::default().fg(theme.text))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(" Timeline "),
            );
        frame.render_widget(timeline, chunks[2]);

        // Tokens: Breakdown
        let tokens_text = self.record.tokens.format();
        let tokens = Paragraph::new(tokens_text)
            .style(Style::default().fg(theme.info))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(" Token Usage "),
            );
        frame.render_widget(tokens, chunks[3]);

        // Cost: With color thresholds
        let cost_color = if self.record.cost > 1.0 {
            theme.error
        } else if self.record.cost > 0.1 {
            theme.warning
        } else {
            theme.success
        };
        let cost_text = format!("${:.4}", self.record.cost);
        let cost = Paragraph::new(cost_text)
            .style(Style::default().fg(cost_color))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(" Cost "),
            );
        frame.render_widget(cost, chunks[4]);

        // Execution Context: Engine, model, retry, cycle
        let context_text = format!(
            "Engine: {} | Model: {} | Retry: {}/{} | Cycle: {}",
            self.record.engine,
            self.record.model,
            self.record.retry_attempt,
            if self.record.retry_attempt > 0 {
                self.record.retry_attempt + 1
            } else {
                1
            },
            self.record.cycle_number
        );
        let context = Paragraph::new(context_text)
            .style(Style::default().fg(theme.text))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(" Execution Context "),
            );
        frame.render_widget(context, chunks[5]);

        // Tools: Tool usage count
        let tools_text = format!("Tools Used: {}", self.record.tool_count);
        let tools = Paragraph::new(tools_text)
            .style(Style::default().fg(theme.info))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(" Tools "),
            );
        frame.render_widget(tools, chunks[6]);

        // Error: Display error message if failed
        if self.record.status == crate::state::ExecutionStatus::Failed {
            if let Some(ref error) = self.record.error_message {
                let error_text = format!("Error: {}", error);
                let error_widget = Paragraph::new(error_text)
                    .style(Style::default().fg(theme.error))
                    .wrap(ratatui::widgets::Wrap { trim: true })
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(theme.border))
                            .title(" Error Message "),
                    );
                frame.render_widget(error_widget, chunks[7]);
            } else {
                let empty = Paragraph::new("No error message available")
                    .style(Style::default().fg(theme.text_muted))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(theme.border))
                            .title(" Error Message "),
                    );
                frame.render_widget(empty, chunks[7]);
            }
        } else {
            let empty = Paragraph::new("Execution completed successfully")
                .style(Style::default().fg(theme.success))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme.border))
                        .title(" Status "),
                );
            frame.render_widget(empty, chunks[7]);
        }

        // Footer: Keyboard shortcuts
        let footer_text = "↑/↓: Scroll | PageUp/PageDown: Page | Home/End: Top/Bottom | Esc/q: Close";
        let footer = Paragraph::new(footer_text)
            .style(Style::default().fg(theme.text_muted))
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(footer, chunks[8]);
    }

    /// Returns the color for an execution status.
    fn status_color(status: crate::state::ExecutionStatus, theme: &crate::theme::RadiumTheme) -> Color {
        match status {
            crate::state::ExecutionStatus::Running => theme.info,
            crate::state::ExecutionStatus::Completed => theme.success,
            crate::state::ExecutionStatus::Failed => theme.error,
            crate::state::ExecutionStatus::Pending => theme.warning,
            crate::state::ExecutionStatus::Cancelled => theme.text_muted,
        }
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

    /// Formats timestamp consistently.
    fn format_timestamp(dt: chrono::DateTime<chrono::Utc>) -> String {
        dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
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
    /// Close the detail view
    Close,
}

