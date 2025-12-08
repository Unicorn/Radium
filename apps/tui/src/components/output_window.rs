//! Output window component for displaying streaming agent output.

use crate::state::OutputBuffer;
use radium_core::code_blocks::CodeBlockParser;
use ratatui::{
    prelude::*,
    text::{Line, Span},
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

        // Parse code blocks and create annotated output
        let annotated_lines = annotate_with_code_blocks(&output_text);

        // Output content with annotations
        let output = Paragraph::new(annotated_lines)
            .style(Style::default())
            .block(Block::default().borders(Borders::ALL).title(format!(" {} ", title)))
            .wrap(Wrap { trim: true });
        frame.render_widget(output, chunks[0]);

        // Status line with code block count
        let total_lines = buffer.lines.len();
        let scroll_pos = buffer.scroll_position;
        let at_bottom = buffer.is_at_bottom();
        
        // Count code blocks in full buffer
        let full_text = buffer.lines.join("\n");
        let blocks = CodeBlockParser::parse(&full_text);
        let block_count = blocks.len();

        let status_text = if total_lines == 0 {
            "No output yet".to_string()
        } else {
            let base_text = if at_bottom {
                format!("Lines: {} [At Bottom]", total_lines)
            } else {
                format!("Lines: {} [Scroll: {}/{}]", total_lines, scroll_pos + 1, total_lines)
            };
            if block_count > 0 {
                format!("{} | 📋 {} blocks", base_text, block_count)
            } else {
                base_text
            }
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

        // Parse code blocks and create annotated output
        let annotated_lines = annotate_with_code_blocks(&output_text);

        // Output content with annotations
        let output = Paragraph::new(annotated_lines)
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

/// Annotate output text with code block markers.
fn annotate_with_code_blocks(text: &str) -> Vec<Line<'_>> {
    let blocks = CodeBlockParser::parse(text);
    let mut block_map: std::collections::HashMap<usize, (usize, String)> = blocks
        .iter()
        .map(|b| (b.start_line, (b.index, b.language.clone().unwrap_or_else(|| "text".to_string()))))
        .collect();

    let mut lines = Vec::new();
    let mut current_line = 1;
    let mut in_code_block = false;

    for line in text.lines() {
        if line.trim().starts_with("```") {
            if in_code_block {
                in_code_block = false;
            } else {
                in_code_block = true;
                // Add annotation if this is a block start
                if let Some((index, lang)) = block_map.get(&current_line) {
                    let annotation = format!("[Block {}: {}] ", index, lang);
                    lines.push(Line::from(vec![
                        Span::styled(annotation, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                        Span::raw(""),
                    ]));
                }
            }
            current_line += 1;
            continue;
        }

        if in_code_block {
            lines.push(Line::from(Span::raw(line)));
        } else {
            lines.push(Line::from(Span::raw(line)));
        }
        current_line += 1;
    }

    if lines.is_empty() {
        vec![Line::from(Span::raw(""))]
    } else {
        lines
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
