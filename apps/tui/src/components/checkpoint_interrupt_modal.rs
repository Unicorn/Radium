//! Checkpoint interrupt modal component for displaying interrupt moments during workflow execution.

use crate::state::{CheckpointInterruptState, InterruptAction, InterruptTrigger};
use crate::theme::RadiumTheme;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

/// Checkpoint interrupt modal component
pub struct CheckpointInterruptModal;

impl CheckpointInterruptModal {
    /// Renders the checkpoint interrupt modal.
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        state: &CheckpointInterruptState,
        theme: &RadiumTheme,
    ) {
        // Create a centered modal area
        let modal_area = Self::centered_rect(80, 75, area);

        // Clear the background
        frame.render_widget(Clear, modal_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Length(4), // Context
                Constraint::Length(6), // Summary
                Constraint::Length(5), // Actions
                Constraint::Length(if state.show_details || state.show_diff { 10 } else { 3 }), // Details/Diff
                Constraint::Length(3), // Help
            ])
            .split(modal_area);

        // Header
        let reason = match &state.trigger {
            InterruptTrigger::AgentCheckpoint { reason, .. } => reason.clone(),
            InterruptTrigger::PolicyAskUser { reason, .. } => reason.clone(),
            InterruptTrigger::Error { message } => message.clone(),
        };
        let title = format!("⏸  CHECKPOINT: {}", reason);
        let header = Paragraph::new(title)
            .style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.primary)),
            );
        frame.render_widget(header, chunks[0]);

        // Context
        let agent_id = match &state.trigger {
            InterruptTrigger::AgentCheckpoint { agent_id, .. } => agent_id.clone(),
            InterruptTrigger::PolicyAskUser { .. } => "policy-engine".to_string(),
            InterruptTrigger::Error { .. } => "system".to_string(),
        };
        let timestamp = format_system_time(&state.timestamp);
        let context_text = vec![
            format!("Agent: {}", agent_id),
            format!("Workflow: {}", state.workflow_id),
            format!("Step: {}/{}", state.step_number, state.step_number + 10), // TODO: Get total steps
            format!("Time: {}", timestamp),
        ];
        let context = Paragraph::new(context_text.join("\n"))
            .style(Style::default().fg(theme.text_muted))
            .block(Block::default().borders(Borders::ALL).title(" Context "));
        frame.render_widget(context, chunks[1]);

        // Summary
        let summary_text = vec![
            "Summary:".to_string(),
            "• Workflow execution paused".to_string(),
            "• Review required before proceeding".to_string(),
            "".to_string(),
            format!("• Checkpoint ID: {}", state.checkpoint_id.as_ref().unwrap_or(&"N/A".to_string())),
        ];
        let summary = Paragraph::new(summary_text.join("\n"))
            .style(Style::default().fg(theme.text))
            .block(Block::default().borders(Borders::ALL).title(" Summary "));
        frame.render_widget(summary, chunks[2]);

        // Actions
        let actions = state.available_actions();
        let action_items: Vec<ListItem> = actions
            .iter()
            .enumerate()
            .map(|(idx, action)| {
                let is_selected = idx == state.selected_action_index;
                let prefix = if is_selected { "> " } else { "  " };
                let label = match action {
                    InterruptAction::Continue => "Continue - Resume execution",
                    InterruptAction::Rollback { checkpoint_id } => {
                        &format!("Rollback - Restore to checkpoint {}", checkpoint_id)
                    }
                    InterruptAction::Cancel => "Cancel - Stop workflow",
                };
                let style = if is_selected {
                    Style::default()
                        .fg(theme.primary)
                        .add_modifier(Modifier::BOLD | Modifier::REVERSED)
                } else {
                    match action {
                        InterruptAction::Continue => Style::default().fg(theme.primary),
                        InterruptAction::Rollback { .. } => Style::default().fg(theme.warning),
                        InterruptAction::Cancel => Style::default().fg(theme.error),
                    }
                };
                ListItem::new(format!("{}{}", prefix, label)).style(style)
            })
            .collect();

        let action_list = List::new(action_items)
            .block(Block::default().borders(Borders::ALL).title(" Actions "));
        frame.render_widget(action_list, chunks[3]);

        // Details/Diff view
        if state.show_details {
            Self::render_details(frame, chunks[4], state, theme);
        } else if state.show_diff {
            Self::render_diff(frame, chunks[4], state, theme);
        } else {
            let empty = Paragraph::new("[Press 'd' for details, 'g' for diff]")
                .style(Style::default().fg(theme.text_dim))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title(" Details "));
            frame.render_widget(empty, chunks[4]);
        }

        // Help
        let help_text = "[Tab/Shift+Tab] Navigate | [Enter] Confirm | [d] Details | [g] Diff | [Esc] Close";
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(theme.text_dim))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(help, chunks[5]);
    }

    /// Renders the details view.
    fn render_details(frame: &mut Frame, area: Rect, state: &CheckpointInterruptState, theme: &RadiumTheme) {
        let details_text = vec![
            "Agent Output:".to_string(),
            "".to_string(),
            "Full agent output would be displayed here.".to_string(),
            "This view shows the complete execution log".to_string(),
            "leading up to the checkpoint.".to_string(),
            "".to_string(),
            format!("Workflow: {}", state.workflow_id),
            format!("Step: {}", state.step_number),
        ];

        let details = Paragraph::new(details_text.join("\n"))
            .style(Style::default().fg(theme.text))
            .block(Block::default().borders(Borders::ALL).title(" Details (Press 'd' to toggle) "));
        frame.render_widget(details, area);
    }

    /// Renders the diff view.
    fn render_diff(frame: &mut Frame, area: Rect, state: &CheckpointInterruptState, theme: &RadiumTheme) {
        let diff_text = if let Some(ref diff) = state.diff_data {
            vec![
                format!("Files changed: {}", diff.files_changed()),
                format!("Added: {} | Modified: {} | Deleted: {}", 
                    diff.added.len(), diff.modified.len(), diff.deleted.len()),
                format!("Insertions: {} | Deletions: {}", diff.insertions, diff.deletions),
                "".to_string(),
                "Changed files:".to_string(),
            ]
            .into_iter()
            .chain(
                diff.added.iter().map(|f| format!("  + {}", f))
                    .chain(diff.modified.iter().map(|f| format!("  ~ {}", f)))
                    .chain(diff.deleted.iter().map(|f| format!("  - {}", f)))
            )
            .collect::<Vec<_>>()
        } else {
            vec![
                "Diff Preview:".to_string(),
                "".to_string(),
                "Loading diff data...".to_string(),
                "".to_string(),
                format!("Checkpoint: {}", state.checkpoint_id.as_ref().unwrap_or(&"N/A".to_string())),
            ]
        };

        let diff = Paragraph::new(diff_text.join("\n"))
            .style(Style::default().fg(theme.warning))
            .block(Block::default().borders(Borders::ALL).title(" Diff Preview (Press 'g' to toggle) "));
        frame.render_widget(diff, area);
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

/// Formats a SystemTime as a readable string.
fn format_system_time(time: &SystemTime) -> String {
    use std::time::UNIX_EPOCH;

    if let Ok(duration) = time.duration_since(UNIX_EPOCH) {
        let secs = duration.as_secs();
        let datetime = chrono::DateTime::<chrono::Utc>::from_timestamp(secs as i64, 0)
            .unwrap_or_else(|| chrono::Utc::now());
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    } else {
        "Unknown time".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkpoint_interrupt_modal_creation() {
        // This is a rendering component, so we just ensure it compiles
        let _component = CheckpointInterruptModal;
    }
}

