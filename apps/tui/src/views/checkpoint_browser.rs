//! Checkpoint browser view for TUI checkpoint management.

use crate::state::CheckpointBrowserState;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Gauge},
};

/// Renders the checkpoint browser view.
pub fn render_checkpoint_browser(
    frame: &mut Frame,
    area: Rect,
    state: &CheckpointBrowserState,
) {
    let theme = crate::theme::get_theme();
    
    // Main layout: timeline on left, details on right
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // Left panel: Checkpoint timeline
    render_checkpoint_timeline(frame, chunks[0], state, theme);

    // Right panel: Details and diff preview
    render_checkpoint_details(frame, chunks[1], state, theme);

    // Restore confirmation dialog (overlay)
    if state.show_restore_confirmation {
        render_restore_confirmation(frame, area, state, theme);
    }
}

/// Renders the checkpoint timeline (left panel).
fn render_checkpoint_timeline(
    frame: &mut Frame,
    area: Rect,
    state: &CheckpointBrowserState,
    theme: &crate::theme::RadiumTheme,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(5),    // Checkpoint list
            Constraint::Length(1), // Footer
        ])
        .split(area);

    // Title
    let title = Paragraph::new("Checkpoints")
        .style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .style(Style::default().bg(theme.bg_panel)),
        );
    frame.render_widget(title, chunks[0]);

    // Checkpoint list
    if state.checkpoints.is_empty() {
        let empty = Paragraph::new("No checkpoints available")
            .style(Style::default().fg(theme.text_muted))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_panel)),
            );
        frame.render_widget(empty, chunks[1]);
    } else {
        let items: Vec<ListItem> = state
            .checkpoints
            .iter()
            .enumerate()
            .map(|(idx, checkpoint)| {
                let is_selected = idx == state.selected_index;
                
                // Format timestamp
                let timestamp = chrono::DateTime::from_timestamp(checkpoint.timestamp as i64, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| checkpoint.timestamp.to_string());
                
                // Format description
                let desc = checkpoint.description.as_deref().unwrap_or("No description");
                let desc_short = if desc.len() > 30 {
                    format!("{}...", &desc[..30])
                } else {
                    desc.to_string()
                };
                
                let content = format!("{} | {}", timestamp, desc_short);
                
                let style = if is_selected {
                    Style::default()
                        .fg(theme.primary)
                        .bg(theme.bg_selected)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text)
                };
                
                ListItem::new(content).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_panel)),
            )
            .highlight_style(
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            );

        let mut list_state = ratatui::widgets::ListState::default();
        list_state.select(Some(state.selected_index));
        frame.render_stateful_widget(list, chunks[1], &mut list_state);
    }

    // Footer with navigation hints
    let footer = Paragraph::new("↑/↓: Navigate | Enter: Restore | d: Diff | q: Close")
        .style(Style::default().fg(theme.text_muted))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(theme.border)),
        );
    frame.render_widget(footer, chunks[2]);
}

/// Renders checkpoint details and diff preview (right panel).
fn render_checkpoint_details(
    frame: &mut Frame,
    area: Rect,
    state: &CheckpointBrowserState,
    theme: &crate::theme::RadiumTheme,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(8),  // Metadata
            Constraint::Min(5),     // Diff preview
            Constraint::Length(1), // Footer
        ])
        .split(area);

    // Title
    let title = Paragraph::new("Checkpoint Details")
        .style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .style(Style::default().bg(theme.bg_panel)),
        );
    frame.render_widget(title, chunks[0]);

    // Metadata
    if let Some(checkpoint) = state.selected_checkpoint() {
        let timestamp = chrono::DateTime::from_timestamp(checkpoint.timestamp as i64, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
            .unwrap_or_else(|| checkpoint.timestamp.to_string());
        
        let metadata = format!(
            "ID: {}\nCommit: {}\nTime: {}\nDescription: {}\nAgent: {}\nTask: {}\nWorkflow: {}",
            checkpoint.id,
            &checkpoint.commit_hash[..12],
            timestamp,
            checkpoint.description.as_deref().unwrap_or("None"),
            checkpoint.agent_id.as_deref().unwrap_or("None"),
            checkpoint.task_id.as_deref().unwrap_or("None"),
            checkpoint.workflow_id.as_deref().unwrap_or("None"),
        );

        let metadata_para = Paragraph::new(metadata)
            .style(Style::default().fg(theme.text))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_panel))
                    .title("Metadata"),
            );
        frame.render_widget(metadata_para, chunks[1]);

        // Diff preview
        if let Some(ref diff) = state.diff_preview {
            let diff_text = format!(
                "Files changed: {}\nAdded: {} | Modified: {} | Deleted: {}\nInsertions: {} | Deletions: {}",
                diff.files_changed(),
                diff.added.len(),
                diff.modified.len(),
                diff.deleted.len(),
                diff.insertions,
                diff.deletions,
            );

            let diff_para = Paragraph::new(diff_text)
                .style(Style::default().fg(theme.text))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme.border))
                        .style(Style::default().bg(theme.bg_panel))
                        .title("Changes from Current State"),
                );
            frame.render_widget(diff_para, chunks[2]);
        } else {
            let no_diff = Paragraph::new("No diff available")
                .style(Style::default().fg(theme.text_muted))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme.border))
                        .style(Style::default().bg(theme.bg_panel))
                        .title("Changes from Current State"),
                );
            frame.render_widget(no_diff, chunks[2]);
        }
    } else {
        let empty = Paragraph::new("No checkpoint selected")
            .style(Style::default().fg(theme.text_muted))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_panel)),
            );
        frame.render_widget(empty, chunks[1]);
    }

    // Error display
    if let Some(ref error) = state.error {
        let error_para = Paragraph::new(format!("Error: {}", error))
            .style(Style::default().fg(theme.error))
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(theme.error)),
            );
        frame.render_widget(error_para, chunks[3]);
    }
}

/// Renders the restore confirmation dialog.
fn render_restore_confirmation(
    frame: &mut Frame,
    area: Rect,
    state: &CheckpointBrowserState,
    theme: &crate::theme::RadiumTheme,
) {
    // Create centered modal
    let popup_area = centered_rect(60, 30, area);
    
    // Clear background
    frame.render_widget(
        Block::default().style(Style::default().bg(Color::Black)),
        popup_area,
    );

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(5), // Message
            Constraint::Length(3), // Buttons
        ])
        .split(popup_area);

    // Title
    let title = Paragraph::new("Confirm Restore")
        .style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .style(Style::default().bg(theme.bg_panel)),
        );
    frame.render_widget(title, chunks[0]);

    // Message
    if let Some(checkpoint) = state.selected_checkpoint() {
        let message = format!(
            "Restore workspace to checkpoint:\n{}\n\nThis will overwrite current workspace state.",
            checkpoint.id
        );
        let msg_para = Paragraph::new(message)
            .style(Style::default().fg(theme.text))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_panel)),
            );
        frame.render_widget(msg_para, chunks[1]);
    }

    // Buttons
    let buttons = Paragraph::new("(y) Yes  |  (n) No  |  (Esc) Cancel")
        .style(Style::default().fg(theme.text_muted))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .style(Style::default().bg(theme.bg_panel)),
        );
    frame.render_widget(buttons, chunks[2]);
}

/// Helper to create a centered rectangle.
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

