//! Splash screen for Radium TUI startup.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

use crate::theme::THEME;

/// Render the splash screen.
pub fn render_splash(frame: &mut Frame, area: Rect, message: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(7),
            Constraint::Percentage(40),
        ])
        .split(area);

    // Branded logo area
    let logo_text = vec![
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "Radium",
            Style::default().fg(THEME.primary).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled("━━━━━━━━━━━━", Style::default().fg(THEME.primary))),
        Line::from(""),
        Line::from(Span::styled(message, Style::default().fg(THEME.text_muted))),
    ];

    let logo = Paragraph::new(logo_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));

    frame.render_widget(logo, chunks[1]);
}
