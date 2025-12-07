//! Status footer component for displaying overall status and help text.

use crate::commands::DisplayContext;
use crate::state::WorkflowStatus;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

/// Application mode for context-aware footer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Prompt,
    Workflow,
    Chat,
    History,
    Setup,
    Requirement,
}

impl AppMode {
    /// Returns display name for the mode.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Prompt => "Prompt",
            Self::Workflow => "Workflow",
            Self::Chat => "Chat",
            Self::History => "History",
            Self::Setup => "Setup",
            Self::Requirement => "Requirement",
        }
    }

    /// Returns keyboard shortcuts for the mode.
    pub fn shortcuts(&self) -> &'static str {
        match self {
            Self::Prompt => "[Ctrl+P] Command Palette | [Ctrl+C] Quit | [?] Help",
            Self::Workflow => "[↑↓] Navigate | [Enter] Select | [Esc] Close | [Ctrl+C] Cancel",
            Self::Chat => "[Enter] Send | [↑↓] Scroll | [Esc] Back | [Ctrl+C] Quit",
            Self::History => "[↑↓] Navigate | [Enter] View | [Esc] Back | [Ctrl+C] Quit",
            Self::Setup => "[Enter] Continue | [Esc] Skip | [Ctrl+C] Quit",
            Self::Requirement => "[↑↓] Scroll | [Esc] Cancel | [Ctrl+C] Force Quit",
        }
    }
}

/// Status footer component
pub struct StatusFooter;

impl StatusFooter {
    /// Renders a context-aware status footer.
    pub fn render_context_aware(
        frame: &mut Frame,
        area: Rect,
        mode: AppMode,
        context: Option<&DisplayContext>,
        selection_info: Option<&str>,
    ) {
        let theme = crate::theme::get_theme();
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(12), // Mode
                Constraint::Min(10),    // Selection info
                Constraint::Percentage(50), // Shortcuts
            ])
            .split(area);

        // Mode indicator
        let mode_text = format!("Mode: {}", mode.as_str());
        let mode_widget = Paragraph::new(mode_text)
            .style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_panel)),
            );
        frame.render_widget(mode_widget, chunks[0]);

        // Selection/Context info
        let info_text = if let Some(info) = selection_info {
            info.to_string()
        } else if let Some(ctx) = context {
            match ctx {
                DisplayContext::Chat { agent_id, session_id } => {
                    format!("Agent: {} | Session: {}", agent_id, session_id)
                }
                DisplayContext::AgentList => "Select an agent".to_string(),
                DisplayContext::SessionList => "Select a session".to_string(),
                DisplayContext::ModelSelector => "Select a model".to_string(),
                DisplayContext::Dashboard => "Dashboard".to_string(),
                DisplayContext::Help => "Help".to_string(),
            }
        } else {
            String::new()
        };

        let info_widget = Paragraph::new(info_text)
            .style(Style::default().fg(theme.text_muted))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_panel)),
            );
        frame.render_widget(info_widget, chunks[1]);

        // Keyboard shortcuts
        let shortcuts_text = mode.shortcuts();
        let shortcuts_widget = Paragraph::new(shortcuts_text)
            .style(Style::default().fg(theme.text_dim))
            .alignment(Alignment::Right)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_panel)),
            );
        frame.render_widget(shortcuts_widget, chunks[2]);
    }

    /// Renders the status footer (legacy method for backward compatibility).
    pub fn render(frame: &mut Frame, area: Rect, status: WorkflowStatus, status_message: &str) {
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
        Self::render_extended_with_provider(frame, area, status, status_message, elapsed, step, total_steps, None)
    }

    /// Renders an extended status footer with provider information.
    pub fn render_extended_with_provider(
        frame: &mut Frame,
        area: Rect,
        status: WorkflowStatus,
        status_message: &str,
        elapsed: f64,
        step: usize,
        total_steps: usize,
        provider: Option<&str>,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20), // Status
                Constraint::Percentage(20), // Step info
                Constraint::Percentage(20), // Time
                Constraint::Percentage(20), // Provider
                Constraint::Percentage(20), // Help
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

        // Provider
        let provider_text = provider.unwrap_or("N/A");
        let provider_widget = Paragraph::new(provider_text)
            .style(Style::default().fg(Color::Magenta))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Provider "));
        frame.render_widget(provider_widget, chunks[3]);

        // Help
        let help_widget = Paragraph::new(status_message)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Keys "));
        frame.render_widget(help_widget, chunks[4]);
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
