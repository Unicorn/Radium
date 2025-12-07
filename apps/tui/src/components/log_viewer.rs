//! Log viewer component for reading and displaying log files.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::fs;
use std::path::Path;

/// Formats bytes as human-readable string.
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{} {}", bytes, UNITS[unit_idx])
    } else {
        format!("{:.2} {}", size, UNITS[unit_idx])
    }
}

/// Log viewer component
pub struct LogViewer;

impl LogViewer {
    /// Renders a full-screen log viewer with enhanced navigation.
    pub fn render_fullscreen(
        frame: &mut Frame,
        area: Rect,
        log_path: &Path,
        scroll_position: usize,
        agent_name: Option<&str>,
        is_running: bool,
    ) {
        let theme = crate::theme::get_theme();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header with file info
                Constraint::Min(5),    // Log content
                Constraint::Length(2), // Scroll position
                Constraint::Length(1), // Footer with shortcuts
            ])
            .split(area);

        // Header with file path and size
        let file_name = log_path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown.log");
        let file_size = fs::metadata(log_path)
            .map(|m| format_bytes(m.len()))
            .unwrap_or_else(|_| "unknown".to_string());
        
        let agent_info = agent_name
            .map(|n| format!("Agent: {} | ", n))
            .unwrap_or_default();
        
        let _running_indicator = if is_running {
            " (Running)"
        } else {
            ""
        };
        
        let header_text = format!(
            "Full Logs: {}{} | Path: {} | Size: {}",
            agent_info,
            file_name,
            log_path.display(),
            file_size
        );
        
        let header = Paragraph::new(header_text)
            .style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_panel)),
            );
        frame.render_widget(header, chunks[0]);

        // Log content
        Self::render_content(frame, chunks[1], log_path, scroll_position, &theme);

        // Scroll position indicator
        if let Ok(content) = fs::read_to_string(log_path) {
            let lines: Vec<&str> = content.lines().collect();
            let total_lines = lines.len();
            let visible_height = chunks[1].height.saturating_sub(2) as usize;
            let start_line = scroll_position + 1;
            let end_line = (scroll_position + visible_height).min(total_lines);
            let scroll_percentage = if total_lines > visible_height {
                ((scroll_position as f64 / (total_lines - visible_height) as f64) * 100.0) as u8
            } else {
                100
            };

            let status_text = format!(
                "Lines {}-{} of {} ({}%){}",
                start_line,
                end_line,
                total_lines,
                scroll_percentage,
                if is_running { " • Live updates enabled" } else { "" }
            );

            let status = Paragraph::new(status_text)
                .style(Style::default().fg(if is_running { theme.info } else { theme.text_muted }))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme.border))
                        .style(Style::default().bg(theme.bg_panel)),
                );
            frame.render_widget(status, chunks[2]);
        }

        // Footer with navigation hints
        let footer_text = "[Esc] Close  [↑↓] Scroll  [PgUp/PgDn] Page  [Home] Top  [End] Bottom";
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

    /// Renders the log content area.
    fn render_content(frame: &mut Frame, area: Rect, log_path: &Path, scroll_position: usize, theme: &crate::theme::RadiumTheme) {
        match fs::read_to_string(log_path) {
            Ok(content) => {
                let lines: Vec<&str> = content.lines().collect();
                let total_lines = lines.len();

                // Calculate visible area
                let visible_height = area.height.saturating_sub(2) as usize;
                let start = scroll_position.min(total_lines.saturating_sub(1));
                let end = (start + visible_height).min(total_lines);

                // Get visible lines
                let visible_lines = if start < total_lines {
                    &lines[start..end]
                } else {
                    &[]
                };

                // Create list items with line numbers and syntax highlighting
                let items: Vec<ListItem> = visible_lines
                    .iter()
                    .enumerate()
                    .map(|(idx, line)| {
                        let line_num = start + idx + 1;
                        let content = format!("{:4} | {}", line_num, line);

                        // Color based on log level
                        let style = Self::style_for_line(line, theme);
                        ListItem::new(content).style(style)
                    })
                    .collect();

                let log_list = List::new(items)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(theme.border))
                            .style(Style::default().bg(theme.bg_panel))
                            .title(" Content "),
                    );
                frame.render_widget(log_list, area);
            }
            Err(e) => {
                let error_text = format!("Failed to read log file: {}", e);
                let error = Paragraph::new(error_text)
                    .style(Style::default().fg(theme.error))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(theme.border))
                            .style(Style::default().bg(theme.bg_panel))
                            .title(" Error "),
                    );
                frame.render_widget(error, area);
            }
        }
    }

    /// Renders the log viewer for a log file (legacy method).
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

        // Use enhanced content rendering
        let theme = crate::theme::get_theme();
        Self::render_content(frame, chunks[1], log_path, scroll_position, &theme);

        // Status
        if let Ok(content) = fs::read_to_string(log_path) {
            let lines: Vec<&str> = content.lines().collect();
            let total_lines = lines.len();
            let visible_height = chunks[1].height.saturating_sub(2) as usize;
            let start = scroll_position.min(total_lines.saturating_sub(1));
            let end = (start + visible_height).min(total_lines);

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
    }

    /// Returns the style for a log line based on its content.
    fn style_for_line(line: &str, theme: &crate::theme::RadiumTheme) -> Style {
        let line_upper = line.to_uppercase();

        if line_upper.contains("ERROR") || line_upper.contains("FAIL") {
            Style::default().fg(theme.error)
        } else if line_upper.contains("WARN") {
            Style::default().fg(theme.warning)
        } else if line_upper.contains("INFO") {
            Style::default().fg(theme.info)
        } else if line_upper.contains("DEBUG") {
            Style::default().fg(theme.text_dim)
        } else if line_upper.contains("SUCCESS") || line_upper.contains("COMPLETE") {
            Style::default().fg(theme.success)
        } else {
            Style::default().fg(theme.text)
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
        let theme = crate::theme::RadiumTheme::dark();
        assert_eq!(LogViewer::style_for_line("ERROR: test", &theme).fg, Some(theme.error));
        assert_eq!(LogViewer::style_for_line("WARN: test", &theme).fg, Some(theme.warning));
        assert_eq!(LogViewer::style_for_line("INFO: test", &theme).fg, Some(theme.info));
        assert_eq!(LogViewer::style_for_line("DEBUG: test", &theme).fg, Some(theme.text_dim));
        assert_eq!(LogViewer::style_for_line("SUCCESS: test", &theme).fg, Some(theme.success));
    }
}
