//! Checkpoint modal component for displaying and selecting checkpoints.

use crate::state::{CheckpointInfo, CheckpointState};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Clear},
};

/// Checkpoint modal component
pub struct CheckpointModal;

impl CheckpointModal {
    /// Renders the checkpoint modal dialog.
    pub fn render(frame: &mut Frame, area: Rect, checkpoint_state: &CheckpointState, selected: usize) {
        // Create a centered modal area
        let modal_area = Self::centered_rect(80, 70, area);

        // Clear the background
        frame.render_widget(Clear, modal_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),   // Title
                Constraint::Min(10),     // Checkpoint list
                Constraint::Length(5),   // Details
                Constraint::Length(2),   // Help
            ])
            .split(modal_area);

        // Title
        let title = Paragraph::new("Checkpoints")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)));
        frame.render_widget(title, chunks[0]);

        // Checkpoint list
        if checkpoint_state.checkpoints.is_empty() {
            let empty = Paragraph::new("No checkpoints created yet")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title(" Available Checkpoints "));
            frame.render_widget(empty, chunks[1]);
        } else {
            let items: Vec<ListItem> = checkpoint_state
                .checkpoints
                .iter()
                .enumerate()
                .map(|(idx, checkpoint)| {
                    let style = if idx == selected {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD | Modifier::REVERSED)
                    } else if !checkpoint.restorable {
                        Style::default().fg(Color::DarkGray)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    let restorable_marker = if checkpoint.restorable { "✓" } else { "✗" };
                    let content = format!("{} {}", restorable_marker, checkpoint.format());

                    ListItem::new(content).style(style)
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title(" Available Checkpoints "));
            frame.render_widget(list, chunks[1]);
        }

        // Details for selected checkpoint
        if let Some(checkpoint) = checkpoint_state.checkpoints.get(selected) {
            Self::render_checkpoint_details(frame, chunks[2], checkpoint);
        } else {
            let empty = Paragraph::new("No checkpoint selected")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title(" Details "));
            frame.render_widget(empty, chunks[2]);
        }

        // Help
        let help_text = "[↑/↓] Select | [Enter] Restore | [Esc] Close";
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(help, chunks[3]);
    }

    /// Renders checkpoint details.
    fn render_checkpoint_details(frame: &mut Frame, area: Rect, checkpoint: &CheckpointInfo) {
        let details_text = vec![
            format!("ID: {}", checkpoint.id),
            format!("Name: {}", checkpoint.name),
            format!("Step: {}", checkpoint.step_number),
            format!("Restorable: {}", if checkpoint.restorable { "Yes" } else { "No" }),
        ];

        let details = Paragraph::new(details_text.join("\n"))
            .style(Style::default().fg(Color::Green))
            .block(Block::default().borders(Borders::ALL).title(" Details "));
        frame.render_widget(details, area);
    }

    /// Renders a compact checkpoint indicator (not a modal).
    pub fn render_compact(frame: &mut Frame, area: Rect, checkpoint_state: &CheckpointState) {
        let text = if let Some(latest) = checkpoint_state.get_latest_checkpoint() {
            format!(
                "Last Checkpoint: [Step {}] {} (Total: {})",
                latest.step_number,
                latest.name,
                checkpoint_state.checkpoints.len()
            )
        } else {
            "No checkpoints yet".to_string()
        };

        let widget = Paragraph::new(text)
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::ALL).title(" Checkpoints "));
        frame.render_widget(widget, area);
    }

    /// Helper function to create a centered rectangle.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkpoint_modal_creation() {
        // This is a rendering component, so we just ensure it compiles
        let _component = CheckpointModal;
    }
}
