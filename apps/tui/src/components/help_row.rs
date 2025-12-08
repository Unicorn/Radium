//! Help row component for displaying command help in start page.

use ratatui::{
    prelude::*,
    widgets::Paragraph,
};

/// Renders a help row with command and description
pub fn render_help_row(frame: &mut Frame, area: Rect, command: &str, description: &str) {
    let theme = crate::theme::get_theme();
    
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(14), // Fixed width for command column
            Constraint::Min(1),     // Description
        ])
        .split(area);

    // Command in purple color
    let command_text = format!("/{}", command);
    let command_widget = Paragraph::new(command_text)
        .style(Style::default().fg(theme.purple));
    frame.render_widget(command_widget, chunks[0]);

    // Description in muted color
    let desc_widget = Paragraph::new(description)
        .style(Style::default().fg(theme.text_muted));
    frame.render_widget(desc_widget, chunks[1]);
}

