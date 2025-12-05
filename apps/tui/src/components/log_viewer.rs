//! Log viewer component for reading and displaying log files.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::fs;
use std::path::Path;

/// Log viewer component
pub struct LogViewer;

impl LogViewer {
    /// Renders the log viewer for a log file.
    pub fn render(frame: &mut Frame, area: Rect, log_path: &Path, scroll_position: usize) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(5),    // Log content
                Constraint::Length(2), // Status
            ])
            .split(area);

        // Title
        let file_name = log_path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown.log");
        let title = format!("Log: {}", file_name);
        let title_widget = Paragraph::new(title)
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title_widget, chunks[0]);

        // Read log file
        match fs::read_to_string(log_path) {
            Ok(content) => {
                let lines: Vec<&str> = content.lines().collect();
                let total_lines = lines.len();

                // Calculate visible area
                let visible_height = chunks[1].height.saturating_sub(2) as usize;
                let start = scroll_position.min(total_lines.saturating_sub(1));
                let end = (start + visible_height).min(total_lines);

                // Get visible lines
                let visible_lines = &lines[start..end];

                // Create list items with line numbers
                let items: Vec<ListItem> = visible_lines
                    .iter()
                    .enumerate()
                    .map(|(idx, line)| {
                        let line_num = start + idx + 1;
                        let content = format!("{:4} | {}", line_num, line);

                        // Color based on log level
                        let style = Self::style_for_line(line);
                        ListItem::new(content).style(style)
                    })
                    .collect();

                let log_list = List::new(items)
                    .block(Block::default().borders(Borders::ALL).title(" Content "));
                frame.render_widget(log_list, chunks[1]);

                // Status
                let status_text = format!(
                    "Lines: {} | Scroll: {}-{}/{}",
                    total_lines,
                    start + 1,
                    end,
                    total_lines
                );
                let status = Paragraph::new(status_text)
                    .style(Style::default().fg(Color::Green))
                    .block(Block::default().borders(Borders::ALL).title(" Status "));
                frame.render_widget(status, chunks[2]);
            }
            Err(e) => {
                let error_text = format!("Failed to read log file: {}", e);
                let error = Paragraph::new(error_text)
                    .style(Style::default().fg(Color::Red))
                    .block(Block::default().borders(Borders::ALL).title(" Error "));
                frame.render_widget(error, chunks[1]);
            }
        }
    }

    /// Returns the style for a log line based on its content.
    fn style_for_line(line: &str) -> Style {
        let line_upper = line.to_uppercase();

        if line_upper.contains("ERROR") || line_upper.contains("FAIL") {
            Style::default().fg(Color::Red)
        } else if line_upper.contains("WARN") {
            Style::default().fg(Color::Yellow)
        } else if line_upper.contains("INFO") {
            Style::default().fg(Color::Blue)
        } else if line_upper.contains("DEBUG") {
            Style::default().fg(Color::Gray)
        } else if line_upper.contains("SUCCESS") || line_upper.contains("COMPLETE") {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        }
    }

    /// Renders a log viewer with file list selection.
    pub fn render_with_list(frame: &mut Frame, area: Rect, log_files: &[String], selected: usize) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(area);

        // File list
        let items: Vec<ListItem> = log_files
            .iter()
            .enumerate()
            .map(|(idx, file)| {
                let style = if idx == selected {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(file.clone()).style(style)
            })
            .collect();

        let file_list =
            List::new(items).block(Block::default().borders(Borders::ALL).title(" Log Files "));
        frame.render_widget(file_list, chunks[0]);

        // Selected log viewer
        if let Some(selected_file) = log_files.get(selected) {
            let log_path = Path::new(selected_file);
            Self::render(frame, chunks[1], log_path, 0);
        } else {
            let empty = Paragraph::new("No log file selected")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title(" Log Viewer "));
            frame.render_widget(empty, chunks[1]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_for_line() {
        assert_eq!(LogViewer::style_for_line("ERROR: test"), Style::default().fg(Color::Red));
        assert_eq!(LogViewer::style_for_line("WARN: test"), Style::default().fg(Color::Yellow));
        assert_eq!(LogViewer::style_for_line("INFO: test"), Style::default().fg(Color::Blue));
        assert_eq!(LogViewer::style_for_line("DEBUG: test"), Style::default().fg(Color::Gray));
        assert_eq!(LogViewer::style_for_line("SUCCESS: test"), Style::default().fg(Color::Green));
        assert_eq!(LogViewer::style_for_line("normal text"), Style::default());
    }
}
