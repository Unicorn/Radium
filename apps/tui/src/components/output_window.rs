//! Output window component for displaying streaming agent output.

use crate::state::OutputBuffer;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};

/// Output window component
pub struct OutputWindow;

impl OutputWindow {
    /// Renders the output window.
    pub fn render(frame: &mut Frame, area: Rect, buffer: &OutputBuffer, title: &str) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),    // Output
                Constraint::Length(1), // Status line
            ])
            .split(area);

        // Calculate visible height for output
        let output_height = chunks[0].height.saturating_sub(2) as usize; // Subtract borders

        // Get visible lines
        let visible_lines = buffer.visible_lines(output_height);
        let output_text = visible_lines.join("\n");

        // Output content
        let output = Paragraph::new(output_text)
            .style(Style::default())
            .block(Block::default().borders(Borders::ALL).title(format!(" {} ", title)))
            .wrap(Wrap { trim: true });
        frame.render_widget(output, chunks[0]);

        // Status line
        let total_lines = buffer.lines.len();
        let scroll_pos = buffer.scroll_position;
        let at_bottom = buffer.is_at_bottom();

        let status_text = if total_lines == 0 {
            "No output yet".to_string()
        } else if at_bottom {
            format!("Lines: {} [At Bottom]", total_lines)
        } else {
            format!("Lines: {} [Scroll: {}/{}]", total_lines, scroll_pos + 1, total_lines)
        };

        let status_style = if at_bottom {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Yellow)
        };

        let status = Paragraph::new(status_text).style(status_style).alignment(Alignment::Right);
        frame.render_widget(status, chunks[1]);
    }

    /// Renders output window with scroll indicators.
    pub fn render_with_scroll(frame: &mut Frame, area: Rect, buffer: &OutputBuffer, title: &str) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Title with scroll indicator
                Constraint::Min(3),    // Output
                Constraint::Length(1), // Help text
            ])
            .split(area);

        // Title with scroll indicator
        let scroll_indicator = if buffer.is_at_bottom() { "▼" } else { "▲" };

        let title_text = format!(" {} {} ", title, scroll_indicator);
        let title_widget = Paragraph::new(title_text)
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::TOP | Borders::LEFT | Borders::RIGHT));
        frame.render_widget(title_widget, chunks[0]);

        // Calculate visible height
        let output_height = chunks[1].height as usize;

        // Get visible lines
        let visible_lines = buffer.visible_lines(output_height);
        let output_text = visible_lines.join("\n");

        // Output content
        let output = Paragraph::new(output_text)
            .style(Style::default())
            .block(Block::default().borders(Borders::LEFT | Borders::RIGHT))
            .wrap(Wrap { trim: true });
        frame.render_widget(output, chunks[1]);

        // Help text
        let help_text = if buffer.is_at_bottom() {
            "[↑/↓] Scroll | [PgUp/PgDn] Page | [Home] Top"
        } else {
            "[↑/↓] Scroll | [PgUp/PgDn] Page | [End] Bottom | [Home] Top"
        };

        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT));
        frame.render_widget(help, chunks[2]);
    }

    /// Renders a split view with two output windows.
    pub fn render_split(
        frame: &mut Frame,
        area: Rect,
        left_buffer: &OutputBuffer,
        left_title: &str,
        right_buffer: &OutputBuffer,
        right_title: &str,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        Self::render(frame, chunks[0], left_buffer, left_title);
        Self::render(frame, chunks[1], right_buffer, right_title);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_window_creation() {
        // This is a rendering component, so we just ensure it compiles
        let _component = OutputWindow;
    }
}
