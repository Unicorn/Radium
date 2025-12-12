//! Command Confirmation Modal Component
//!
//! Displays a confirmation dialog for dangerous shell commands.
//! Shows command details and provides options to execute, deny, or add to allowlist.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::path::PathBuf;
use crate::command_safety::CommandAnalysis;

/// Request for command confirmation sent from executor to main event loop
#[derive(Debug)]
pub struct ConfirmationRequest {
    /// Command to be executed
    pub command: String,
    /// Working directory where command will run
    pub working_dir: PathBuf,
    /// Command analysis (classification and danger reason)
    pub analysis: CommandAnalysis,
    /// Channel to send back the user's decision
    pub response_tx: tokio::sync::oneshot::Sender<ConfirmationOutcome>,
}

/// Outcome of command confirmation dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmationOutcome {
    /// Execute command once
    Approved,
    /// Execute command and add to allowlist for future auto-execution
    ApprovedAlways,
    /// Do not execute command
    Denied,
    /// Cancel dialog without action
    Cancelled,
}

/// Command confirmation modal state
#[derive(Debug)]
pub struct CommandConfirmationModal {
    /// Command to be executed
    pub command: String,
    /// Working directory where command will run
    pub working_dir: PathBuf,
    /// Command analysis (classification and danger reason)
    pub analysis: CommandAnalysis,
    /// Currently selected option index
    pub selected_index: usize,
    /// Available options
    options: Vec<ConfirmationOption>,
    /// Channel to send the user's decision back
    pub response_tx: Option<tokio::sync::oneshot::Sender<ConfirmationOutcome>>,
}

/// A selectable option in the confirmation modal
#[derive(Debug, Clone)]
struct ConfirmationOption {
    /// Display text
    label: String,
    /// Short description
    description: String,
    /// Outcome when selected
    outcome: ConfirmationOutcome,
}

impl CommandConfirmationModal {
    /// Creates a new command confirmation modal
    pub fn new(
        command: String,
        working_dir: PathBuf,
        analysis: CommandAnalysis,
        response_tx: tokio::sync::oneshot::Sender<ConfirmationOutcome>,
    ) -> Self {
        let options = vec![
            ConfirmationOption {
                label: "Execute Once".to_string(),
                description: "Run this command now".to_string(),
                outcome: ConfirmationOutcome::Approved,
            },
            ConfirmationOption {
                label: "Execute and Always Allow".to_string(),
                description: "Run now and auto-approve future instances".to_string(),
                outcome: ConfirmationOutcome::ApprovedAlways,
            },
            ConfirmationOption {
                label: "Deny".to_string(),
                description: "Do not execute this command".to_string(),
                outcome: ConfirmationOutcome::Denied,
            },
            ConfirmationOption {
                label: "Cancel".to_string(),
                description: "Go back without any action".to_string(),
                outcome: ConfirmationOutcome::Cancelled,
            },
        ];

        Self {
            command,
            working_dir,
            analysis,
            selected_index: 0,
            options,
            response_tx: Some(response_tx),
        }
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(1);
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        let max_index = self.options.len().saturating_sub(1);
        self.selected_index = (self.selected_index + 1).min(max_index);
    }

    /// Get the outcome of the currently selected option
    pub fn selected_outcome(&self) -> ConfirmationOutcome {
        self.options
            .get(self.selected_index)
            .map(|opt| opt.outcome)
            .unwrap_or(ConfirmationOutcome::Cancelled)
    }
}

/// Renders the command confirmation modal
pub fn render_command_confirmation(frame: &mut Frame, area: Rect, modal: &CommandConfirmationModal) {
    let theme = crate::theme::get_theme();

    // Calculate modal size (larger than dialog for command details)
    let modal_width = 80;
    let modal_height = 18; // Header + command details + options + help

    // Center modal
    let modal_area = Rect {
        x: (area.width.saturating_sub(modal_width)) / 2,
        y: (area.height.saturating_sub(modal_height)) / 2,
        width: modal_width.min(area.width),
        height: modal_height.min(area.height),
    };

    // Render backdrop
    let backdrop = Paragraph::new("")
        .style(Style::default().bg(Color::Black))
        .block(Block::default().style(Style::default().bg(Color::Black)));
    frame.render_widget(backdrop, area);

    // Split modal area
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Title
            Constraint::Length(7), // Command details
            Constraint::Min(5),    // Options list
            Constraint::Length(2), // Help text
        ])
        .split(modal_area);

    // Title
    let title = Paragraph::new("⚠  Confirm Shell Command")
        .style(Style::default().fg(theme.warning).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::NONE)
                .padding(ratatui::widgets::Padding::new(1, 1, 0, 0)),
        );
    frame.render_widget(title, chunks[0]);

    // Command details
    let command_display = if modal.command.len() > 60 {
        format!("{}...", &modal.command[..57])
    } else {
        modal.command.clone()
    };

    let dir_display = modal
        .working_dir
        .to_str()
        .unwrap_or("<unknown>")
        .to_string();
    let dir_display = if dir_display.len() > 60 {
        format!("{}...", &dir_display[..57])
    } else {
        dir_display
    };

    let danger_reason = modal
        .analysis
        .danger_reason
        .as_ref()
        .unwrap_or(&"This command may modify system state.".to_string())
        .clone();

    let details_text = format!(
        "Command: {}\nDirectory: {}\nRoot: {}\n\nReason: {}",
        command_display, dir_display, modal.analysis.root_command, danger_reason
    );

    let details = Paragraph::new(details_text)
        .style(Style::default().fg(theme.text))
        .alignment(Alignment::Left)
        .wrap(ratatui::widgets::Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::NONE)
                .padding(ratatui::widgets::Padding::new(2, 2, 0, 1)),
        );
    frame.render_widget(details, chunks[1]);

    // Options list
    let items: Vec<ListItem> = modal
        .options
        .iter()
        .enumerate()
        .map(|(idx, option)| {
            let is_selected = idx == modal.selected_index;
            let style = if is_selected {
                Style::default()
                    .fg(theme.bg_primary)
                    .bg(theme.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };

            let prefix = if is_selected { "● " } else { "○ " };
            let content = format!("{}{} - {}", prefix, option.label, option.description);
            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::NONE)
            .padding(ratatui::widgets::Padding::new(2, 2, 0, 1)),
    );
    frame.render_widget(list, chunks[2]);

    // Help text
    let help_text = "↑/↓ Navigate • Enter to confirm • Esc to cancel";
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(theme.text_muted))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::NONE)
                .padding(ratatui::widgets::Padding::new(1, 1, 0, 0)),
        );
    frame.render_widget(help, chunks[3]);

    // Render border around entire modal
    let modal_border = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.warning))
        .style(Style::default().bg(theme.bg_panel));
    frame.render_widget(modal_border, modal_area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command_safety::{CommandAnalysis, CommandClassification};

    #[test]
    fn test_modal_creation() {
        let analysis = CommandAnalysis {
            classification: CommandClassification::Dangerous,
            root_command: "rm".to_string(),
            full_command: "rm -rf /tmp/test".to_string(),
            danger_reason: Some("rm can delete files".to_string()),
        };

        let (response_tx, _response_rx) = tokio::sync::oneshot::channel();
        let modal = CommandConfirmationModal::new(
            "rm -rf /tmp/test".to_string(),
            PathBuf::from("/home/user"),
            analysis,
            response_tx,
        );

        assert_eq!(modal.selected_index, 0);
        assert_eq!(modal.options.len(), 4);
        assert_eq!(modal.selected_outcome(), ConfirmationOutcome::Approved);
    }

    #[test]
    fn test_modal_navigation() {
        let analysis = CommandAnalysis {
            classification: CommandClassification::Dangerous,
            root_command: "rm".to_string(),
            full_command: "rm file".to_string(),
            danger_reason: Some("rm deletes files".to_string()),
        };

        let (response_tx, _response_rx) = tokio::sync::oneshot::channel();
        let mut modal = CommandConfirmationModal::new(
            "rm file".to_string(),
            PathBuf::from("/tmp"),
            analysis,
            response_tx,
        );

        assert_eq!(modal.selected_index, 0);
        assert_eq!(modal.selected_outcome(), ConfirmationOutcome::Approved);

        modal.move_down();
        assert_eq!(modal.selected_index, 1);
        assert_eq!(modal.selected_outcome(), ConfirmationOutcome::ApprovedAlways);

        modal.move_down();
        assert_eq!(modal.selected_index, 2);
        assert_eq!(modal.selected_outcome(), ConfirmationOutcome::Denied);

        modal.move_down();
        assert_eq!(modal.selected_index, 3);
        assert_eq!(modal.selected_outcome(), ConfirmationOutcome::Cancelled);

        modal.move_down(); // Should not go beyond
        assert_eq!(modal.selected_index, 3);

        modal.move_up();
        assert_eq!(modal.selected_index, 2);

        modal.move_up();
        modal.move_up();
        modal.move_up(); // Should not go below 0
        assert_eq!(modal.selected_index, 0);
    }

    #[test]
    fn test_outcome_selection() {
        let analysis = CommandAnalysis {
            classification: CommandClassification::Dangerous,
            root_command: "sudo".to_string(),
            full_command: "sudo apt install".to_string(),
            danger_reason: Some("sudo requires elevated privileges".to_string()),
        };

        let (response_tx, _response_rx) = tokio::sync::oneshot::channel();
        let mut modal = CommandConfirmationModal::new(
            "sudo apt install".to_string(),
            PathBuf::from("/"),
            analysis,
            response_tx,
        );

        // Test each outcome
        assert_eq!(modal.selected_outcome(), ConfirmationOutcome::Approved);

        modal.move_down();
        assert_eq!(modal.selected_outcome(), ConfirmationOutcome::ApprovedAlways);

        modal.move_down();
        assert_eq!(modal.selected_outcome(), ConfirmationOutcome::Denied);

        modal.move_down();
        assert_eq!(modal.selected_outcome(), ConfirmationOutcome::Cancelled);
    }
}
