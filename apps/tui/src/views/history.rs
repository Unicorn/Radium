//! Execution history view showing past workflow runs.

use crate::components::LogViewer;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

/// Execution history entry
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    /// Workflow ID
    pub workflow_id: String,
    /// Workflow name
    pub workflow_name: String,
    /// Runtime in seconds
    pub runtime_secs: u64,
    /// Status
    pub status: String,
    /// Completion time
    pub completed_at: String,
    /// Log file path (if available)
    pub log_path: Option<std::path::PathBuf>,
}

impl HistoryEntry {
    /// Formats runtime as HH:MM:SS
    pub fn format_runtime(&self) -> String {
        let hours = self.runtime_secs / 3600;
        let minutes = (self.runtime_secs % 3600) / 60;
        let seconds = self.runtime_secs % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}

/// Renders the execution history view.
pub fn render_history(
    frame: &mut Frame,
    area: Rect,
    entries: &[HistoryEntry],
    selected_index: usize,
    scroll_position: usize,
) {
    let theme = crate::theme::get_theme();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Title
            Constraint::Min(5),    // History list
            Constraint::Length(2), // Info
            Constraint::Length(1), // Footer
        ])
        .split(area);

    // Title
    let title = Paragraph::new("Execution History")
        .style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .style(Style::default().bg(theme.bg_panel)),
        );
    frame.render_widget(title, chunks[0]);

    // History list
    if entries.is_empty() {
        let empty = Paragraph::new("No execution history available")
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
        let visible_height = chunks[1].height.saturating_sub(2) as usize;
        let start = scroll_position.min(entries.len().saturating_sub(1));
        let end = (start + visible_height).min(entries.len());

        let items: Vec<ListItem> = entries[start..end]
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                let actual_idx = start + idx;
                let is_selected = actual_idx == selected_index;

                let status_color = match entry.status.as_str() {
                    "Completed" => theme.success,
                    "Failed" => theme.error,
                    "Cancelled" => theme.text_muted,
                    _ => theme.text,
                };

                let content = format!(
                    "{} {} | {} | {} | {}",
                    entry.workflow_name,
                    entry.status,
                    entry.format_runtime(),
                    entry.completed_at,
                    entry.workflow_id
                );

                let style = if is_selected {
                    Style::default()
                        .fg(theme.bg_primary)
                        .bg(theme.primary)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(status_color)
                };

                ListItem::new(content).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_panel))
                    .title(" History "),
            );
        frame.render_widget(list, chunks[1]);
    }

    // Info
    let info_text = if entries.is_empty() {
        "No history entries".to_string()
    } else {
        format!(
            "Showing {}-{} of {} entries",
            scroll_position + 1,
            (scroll_position + chunks[1].height.saturating_sub(2) as usize).min(entries.len()),
            entries.len()
        )
    };

    let info = Paragraph::new(info_text)
        .style(Style::default().fg(theme.text_muted))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .style(Style::default().bg(theme.bg_panel)),
        );
    frame.render_widget(info, chunks[2]);

    // Footer
    let footer_text = "[↑↓] Navigate | [Enter] View Logs | [Esc] Back | [Ctrl+C] Quit";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(theme.text_dim))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .style(Style::default().bg(theme.bg_panel)),
        );
    frame.render_widget(footer, chunks[3]);
}

/// Renders history view with log viewer for selected entry.
pub fn render_history_with_log(
    frame: &mut Frame,
    area: Rect,
    entry: &HistoryEntry,
    log_scroll_position: usize,
) {
    let theme = crate::theme::get_theme();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Header
            Constraint::Min(5),    // Log viewer
            Constraint::Length(1), // Footer
        ])
        .split(area);

    // Header
    let header_text = format!(
        "{} | {} | {} | {}",
        entry.workflow_name,
        entry.status,
        entry.format_runtime(),
        entry.completed_at
    );
    let header = Paragraph::new(header_text)
        .style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .style(Style::default().bg(theme.bg_panel)),
        );
    frame.render_widget(header, chunks[0]);

    // Log viewer
    if let Some(ref log_path) = entry.log_path {
        if log_path.exists() {
            LogViewer::render_fullscreen(
                frame,
                chunks[1],
                log_path,
                log_scroll_position,
                Some(&entry.workflow_name),
                false, // Not running
            );
        } else {
            let error = Paragraph::new("Log file not found")
                .style(Style::default().fg(theme.error))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme.border))
                        .style(Style::default().bg(theme.bg_panel)),
                );
            frame.render_widget(error, chunks[1]);
        }
    } else {
        let no_log = Paragraph::new("No log file available for this execution")
            .style(Style::default().fg(theme.text_muted))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_panel)),
            );
        frame.render_widget(no_log, chunks[1]);
    }

    // Footer
    let footer_text = "[Esc] Back to History";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(theme.text_dim))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .style(Style::default().bg(theme.bg_panel)),
        );
    frame.render_widget(footer, chunks[2]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_entry_runtime_formatting() {
        let entry = HistoryEntry {
            workflow_id: "wf-1".to_string(),
            workflow_name: "Test".to_string(),
            runtime_secs: 3661, // 1 hour, 1 minute, 1 second
            status: "Completed".to_string(),
            completed_at: "2024-01-01 12:00:00".to_string(),
            log_path: None,
        };

        assert_eq!(entry.format_runtime(), "01:01:01");
    }
}

