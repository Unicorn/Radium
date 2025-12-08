//! Playbook status component for displaying active playbooks during execution.

use crate::theme::get_theme;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

/// Playbook status component.
pub struct PlaybookStatus;

impl PlaybookStatus {
    /// Render the playbook status showing active playbooks.
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        active_playbooks: &[String],
    ) {
        let theme = get_theme();

        let content = if active_playbooks.is_empty() {
            "No active playbooks".to_string()
        } else {
            format!(
                "Active Playbooks ({})\n{}",
                active_playbooks.len(),
                active_playbooks
                    .iter()
                    .map(|uri| format!("  • {}", uri))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        };

        let status = Paragraph::new(content)
            .style(Style::default().fg(theme.text))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Playbooks")
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_panel)),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(status, area);
    }
}

