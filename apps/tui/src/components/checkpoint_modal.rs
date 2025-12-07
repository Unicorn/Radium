//! Checkpoint modal component for displaying and selecting checkpoints.

use crate::state::{CheckpointInfo, CheckpointState};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

/// Checkpoint modal component
pub struct CheckpointModal;

impl CheckpointModal {
    /// Renders the checkpoint modal dialog.
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        checkpoint_state: &CheckpointState,
        selected: usize,
    ) {
        // Create a centered modal area
        let modal_area = Self::centered_rect(80, 70, area);

        // Clear the background
        frame.render_widget(Clear, modal_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(3), // Search/Filter
                Constraint::Min(10),   // Checkpoint list
                Constraint::Length(1), // Status bar
                Constraint::Length(if checkpoint_state.show_diff { 8 } else { 5 }), // Details or Diff
                Constraint::Length(3), // Help
            ])
            .split(modal_area);

        // Title
        let title = Paragraph::new("Checkpoints")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            );
        frame.render_widget(title, chunks[0]);

        // Search/Filter input
        let search_label = if checkpoint_state.filter_text.is_empty() {
            "Search: (Press '/' to focus)"
        } else {
            "Search:"
        };
        let search_text = if checkpoint_state.filter_text.is_empty() {
            "".to_string()
        } else {
            checkpoint_state.filter_text.clone()
        };
        let search = Paragraph::new(search_text)
            .style(Style::default().fg(Color::Cyan))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(search_label)
                    .border_style(Style::default().fg(Color::Cyan)),
            );
        frame.render_widget(search, chunks[1]);

        // Checkpoint list with pagination
        let filtered = checkpoint_state.filtered_checkpoints();
        let paginated = checkpoint_state.paginated_checkpoints();
        
        if filtered.is_empty() {
            let empty_text = if checkpoint_state.filter_text.is_empty() {
                "No checkpoints created yet"
            } else {
                "No checkpoints match filter"
            };
            let empty = Paragraph::new(empty_text)
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title(" Available Checkpoints "));
            frame.render_widget(empty, chunks[2]);
        } else {
            let items: Vec<ListItem> = paginated
                .iter()
                .enumerate()
                .map(|(idx, checkpoint)| {
                    let global_idx = checkpoint_state.current_page * checkpoint_state.items_per_page + idx;
                    let style = if global_idx == selected {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD | Modifier::REVERSED)
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
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!(
                            " Available Checkpoints (Page {}/{}) ",
                            checkpoint_state.current_page + 1,
                            checkpoint_state.total_pages()
                        )),
                );
            frame.render_widget(list, chunks[2]);
        }

        // Status bar
        let status_text = format!(
            "Total: {} | Filtered: {} | Page: {}/{}",
            checkpoint_state.checkpoints.len(),
            filtered.len(),
            checkpoint_state.current_page + 1,
            checkpoint_state.total_pages()
        );
        let status = Paragraph::new(status_text)
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(status, chunks[3]);

        // Details or Diff preview
        let selected_checkpoint = checkpoint_state
            .filtered_checkpoints()
            .get(selected)
            .copied();
        
        if checkpoint_state.show_diff && selected_checkpoint.is_some() {
            Self::render_diff_preview(frame, chunks[4], selected_checkpoint.unwrap());
        } else if let Some(checkpoint) = selected_checkpoint {
            Self::render_checkpoint_details(frame, chunks[4], checkpoint);
        } else {
            let empty = Paragraph::new("No checkpoint selected")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title(" Details "));
            frame.render_widget(empty, chunks[4]);
        }

        // Help
        let help_text = if checkpoint_state.show_diff {
            "[↑/↓] Select | [Enter] Restore | [d] Toggle Diff | [/] Search | [c] Clear | [PgUp/PgDn] Page | [Esc] Close"
        } else {
            "[↑/↓] Select | [Enter] Restore | [d] Show Diff | [/] Search | [c] Clear | [PgUp/PgDn] Page | [Esc] Close"
        };
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(help, chunks[5]);
    }

    /// Renders checkpoint details.
    fn render_checkpoint_details(frame: &mut Frame, area: Rect, checkpoint: &CheckpointInfo) {
        let details_text = vec![
            format!("ID: {}", checkpoint.id),
            format!("Name: {}", checkpoint.name),
            format!("Step: {}", checkpoint.step_number),
            format!("Restorable: {}", if checkpoint.restorable { "Yes" } else { "No" }),
            if let Some(ref hash) = checkpoint.commit_hash {
                format!("Commit: {}", &hash[..std::cmp::min(12, hash.len())])
            } else {
                String::new()
            },
        ];

        let details = Paragraph::new(details_text.join("\n"))
            .style(Style::default().fg(Color::Green))
            .block(Block::default().borders(Borders::ALL).title(" Details "));
        frame.render_widget(details, area);
    }

    /// Renders diff preview for selected checkpoint.
    /// Note: This is a placeholder - full integration requires CheckpointManager access.
    fn render_diff_preview(frame: &mut Frame, area: Rect, checkpoint: &CheckpointInfo) {
        let diff_text = vec![
            format!("Diff Preview: {}", checkpoint.id),
            "",
            "Note: Full diff preview requires CheckpointManager integration.",
            "This feature shows file-level changes between checkpoints.",
            "",
            "To see full diff, use: rad checkpoint show <id>",
        ];

        let diff = Paragraph::new(diff_text.join("\n"))
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title(" Diff Preview (Press 'd' to toggle) "));
        frame.render_widget(diff, area);
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
