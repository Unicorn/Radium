//! Error Panel Component
//!
//! Displays errors with actionable buttons and keyboard shortcuts.
//! Provides options like "View Logs", "Retry", "Copy Error", etc.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use crate::errors::TuiError;
use crate::icons::Icons;
use crate::theme::THEME;

/// Actions available in the error panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorAction {
    /// Dismiss the error
    Dismiss,
    /// View application logs
    ViewLogs,
    /// Retry the failed operation
    Retry,
    /// Copy error details to clipboard
    CopyError,
    /// Edit configuration file
    EditConfig,
    /// Authenticate with provider
    Authenticate,
}

/// Error panel state and configuration.
#[derive(Debug, Clone)]
pub struct ErrorPanel {
    /// The error to display
    pub error: TuiError,
    /// Currently selected action index
    pub selected_action: usize,
    /// Available actions for this error
    pub available_actions: Vec<ErrorAction>,
}

impl ErrorPanel {
    /// Create a new error panel for the given error.
    pub fn new(error: TuiError) -> Self {
        let available_actions = Self::determine_actions(&error);
        Self {
            error,
            selected_action: 0,
            available_actions,
        }
    }

    /// Determine which actions are available for an error type.
    fn determine_actions(error: &TuiError) -> Vec<ErrorAction> {
        let mut actions = vec![];

        match error {
            TuiError::AuthRequired { .. } => {
                actions.push(ErrorAction::Authenticate);
                actions.push(ErrorAction::ViewLogs);
                actions.push(ErrorAction::Dismiss);
            }
            TuiError::NetworkError { .. } => {
                actions.push(ErrorAction::Retry);
                actions.push(ErrorAction::ViewLogs);
                actions.push(ErrorAction::CopyError);
                actions.push(ErrorAction::Dismiss);
            }
            TuiError::IoError { .. } => {
                actions.push(ErrorAction::Retry);
                actions.push(ErrorAction::ViewLogs);
                actions.push(ErrorAction::Dismiss);
            }
            TuiError::ConfigError { .. } => {
                actions.push(ErrorAction::EditConfig);
                actions.push(ErrorAction::ViewLogs);
                actions.push(ErrorAction::Dismiss);
            }
            _ => {
                actions.push(ErrorAction::CopyError);
                actions.push(ErrorAction::ViewLogs);
                actions.push(ErrorAction::Dismiss);
            }
        }

        actions
    }

    /// Move selection to next action.
    pub fn next_action(&mut self) {
        if !self.available_actions.is_empty() {
            self.selected_action = (self.selected_action + 1) % self.available_actions.len();
        }
    }

    /// Move selection to previous action.
    pub fn prev_action(&mut self) {
        if !self.available_actions.is_empty() {
            self.selected_action = (self.selected_action + self.available_actions.len() - 1)
                % self.available_actions.len();
        }
    }

    /// Get the currently selected action.
    pub fn get_selected_action(&self) -> Option<ErrorAction> {
        self.available_actions.get(self.selected_action).copied()
    }

    /// Get the action label and keyboard shortcut.
    fn action_info(action: ErrorAction) -> (&'static str, &'static str) {
        match action {
            ErrorAction::Dismiss => ("Dismiss", "Esc"),
            ErrorAction::ViewLogs => ("View Logs", "l"),
            ErrorAction::Retry => ("Retry", "r"),
            ErrorAction::CopyError => ("Copy Error", "c"),
            ErrorAction::EditConfig => ("Edit Config", "e"),
            ErrorAction::Authenticate => ("Authenticate", "a"),
        }
    }
}

/// Renders the error panel with actions.
pub fn render_error_panel(frame: &mut Frame, area: Rect, panel: &ErrorPanel) {
    // Calculate modal size
    let modal_width = 80;
    let modal_height = 24;

    // Center modal
    let modal_area = Rect {
        x: (area.width.saturating_sub(modal_width)) / 2,
        y: (area.height.saturating_sub(modal_height)) / 2,
        width: modal_width.min(area.width),
        height: modal_height.min(area.height),
    };

    // Render backdrop
    let backdrop = Block::default().style(Style::default().bg(Color::Black));
    frame.render_widget(backdrop, area);

    // Split modal area
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(10),    // Error message
            Constraint::Length(1),  // Separator
            Constraint::Length(6),  // Actions
            Constraint::Length(2),  // Help text
        ])
        .split(modal_area);

    // Title
    let title = Paragraph::new(panel.error.title())
        .style(Style::default().fg(THEME.error()).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
                .border_style(Style::default().fg(THEME.error()))
                .style(Style::default().bg(THEME.bg_panel())),
        );
    frame.render_widget(title, chunks[0]);

    // Error message
    let message_lines = panel.error.message_lines();
    let message_text = message_lines.join("\n");
    let message = Paragraph::new(message_text)
        .style(Style::default().fg(THEME.text()))
        .block(
            Block::default()
                .borders(Borders::LEFT | Borders::RIGHT)
                .border_style(Style::default().fg(THEME.error()))
                .style(Style::default().bg(THEME.bg_panel())),
        )
        .wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(message, chunks[1]);

    // Separator
    let separator = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT)
        .border_style(Style::default().fg(THEME.error()))
        .style(Style::default().bg(THEME.bg_panel()));
    frame.render_widget(separator, chunks[2]);

    // Actions
    let action_items: Vec<ListItem> = panel
        .available_actions
        .iter()
        .enumerate()
        .map(|(idx, action)| {
            let is_selected = idx == panel.selected_action;
            let (label, shortcut) = ErrorPanel::action_info(*action);

            let style = if is_selected {
                Style::default()
                    .fg(THEME.bg_primary())
                    .bg(THEME.primary())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(THEME.text())
            };

            let prefix = if is_selected { "▶ " } else { "  " };
            let content = format!("{}{} [{}]", prefix, label, shortcut);
            ListItem::new(content).style(style)
        })
        .collect();

    let actions_list = List::new(action_items).block(
        Block::default()
            .borders(Borders::LEFT | Borders::RIGHT)
            .border_style(Style::default().fg(THEME.error()))
            .style(Style::default().bg(THEME.bg_panel())),
    );
    frame.render_widget(actions_list, chunks[3]);

    // Help text
    let help_text = format!("↑/↓ Navigate  {} Select  Esc Close", Icons::SUCCESS);
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(THEME.text_muted()))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(THEME.error()))
                .style(Style::default().bg(THEME.bg_panel())),
        );
    frame.render_widget(help, chunks[4]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_panel_creation() {
        let error = TuiError::from_io_error(
            std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"),
            "Reading config"
        );
        let panel = ErrorPanel::new(error);

        assert!(!panel.available_actions.is_empty());
        assert_eq!(panel.selected_action, 0);
    }

    #[test]
    fn test_action_navigation() {
        let error = TuiError::model_error("Test error");
        let mut panel = ErrorPanel::new(error);

        let initial_count = panel.available_actions.len();
        assert!(initial_count > 0);

        panel.next_action();
        assert_eq!(panel.selected_action, 1);

        panel.prev_action();
        assert_eq!(panel.selected_action, 0);
    }

    #[test]
    fn test_action_selection() {
        let error = TuiError::from_network_error("Connection failed", "API call");
        let panel = ErrorPanel::new(error);

        let action = panel.get_selected_action();
        assert!(action.is_some());
        assert_eq!(action.unwrap(), ErrorAction::Retry);
    }

    #[test]
    fn test_auth_error_actions() {
        let error = TuiError::auth_required("Gemini");
        let panel = ErrorPanel::new(error);

        assert!(panel.available_actions.contains(&ErrorAction::Authenticate));
        assert!(panel.available_actions.contains(&ErrorAction::ViewLogs));
        assert!(panel.available_actions.contains(&ErrorAction::Dismiss));
    }

    #[test]
    fn test_config_error_actions() {
        let error = TuiError::from_config_error("Invalid TOML syntax");
        let panel = ErrorPanel::new(error);

        assert!(panel.available_actions.contains(&ErrorAction::EditConfig));
        assert!(panel.available_actions.contains(&ErrorAction::ViewLogs));
        assert!(panel.available_actions.contains(&ErrorAction::Dismiss));
    }
}
