//! Diff preview component for showing file changes before applying

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarState},
};

/// Diff view state
#[derive(Debug, Clone)]
pub struct DiffView {
    /// File path being changed
    pub file_path: String,
    /// Original content
    pub original_content: String,
    /// New content
    pub new_content: String,
    /// Whether changes are approved
    pub approved: Option<bool>,
    /// Vertical scroll position
    pub scroll_offset: usize,
}

impl DiffView {
    /// Create a new diff view
    pub fn new(file_path: String, original_content: String, new_content: String) -> Self {
        Self {
            file_path,
            original_content,
            new_content,
            approved: None,
            scroll_offset: 0,
        }
    }

    /// Approve the changes
    pub fn approve(&mut self) {
        self.approved = Some(true);
    }

    /// Reject the changes
    pub fn reject(&mut self) {
        self.approved = Some(false);
    }

    /// Check if changes are approved
    pub fn is_approved(&self) -> Option<bool> {
        self.approved
    }

    /// Scroll down
    pub fn scroll_down(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_add(lines);
    }

    /// Scroll up
    pub fn scroll_up(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
    }

    /// Generate a simple diff representation
    pub fn generate_diff(&self) -> Vec<String> {
        let original_lines: Vec<&str> = self.original_content.lines().collect();
        let new_lines: Vec<&str> = self.new_content.lines().collect();
        let mut diff_lines = Vec::new();

        // Simple line-by-line comparison
        let max_len = original_lines.len().max(new_lines.len());
        for i in 0..max_len {
            let old_line = original_lines.get(i);
            let new_line = new_lines.get(i);

            match (old_line, new_line) {
                (Some(old), Some(new)) if old == new => {
                    diff_lines.push(format!("  {}", old));
                }
                (Some(old), Some(new)) => {
                    diff_lines.push(format!("- {}", old));
                    diff_lines.push(format!("+ {}", new));
                }
                (Some(old), None) => {
                    diff_lines.push(format!("- {}", old));
                }
                (None, Some(new)) => {
                    diff_lines.push(format!("+ {}", new));
                }
                (None, None) => {}
            }
        }

        diff_lines
    }
}

/// Render a diff view
pub fn render_diff_view(frame: &mut Frame, area: Rect, diff_view: &DiffView) {
    let theme = crate::theme::get_theme();

    // Split area into header and content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(5),   // Diff content
            Constraint::Length(2), // Footer/actions
        ])
        .split(area);

    // Header with file path
    let header_text = format!("File: {}", diff_view.file_path);
    let header = Paragraph::new(header_text)
        .style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.primary))
                .title("Diff Preview"),
        );
    frame.render_widget(header, chunks[0]);

    // Diff content
    let diff_lines = diff_view.generate_diff();
    let visible_lines = (chunks[1].height as usize).saturating_sub(2);
    let start_line = diff_view.scroll_offset.min(diff_lines.len().saturating_sub(visible_lines));
    let end_line = (start_line + visible_lines).min(diff_lines.len());

    let mut diff_text = String::new();
    for line in &diff_lines[start_line..end_line] {
        diff_text.push_str(line);
        diff_text.push('\n');
    }

    let diff_paragraph = Paragraph::new(diff_text)
        .style(Style::default().fg(theme.text))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border)),
        )
        .wrap(ratatui::widgets::Wrap { trim: true });

    frame.render_widget(diff_paragraph, chunks[1]);

    // Footer with actions
    let status_text = match diff_view.approved {
        Some(true) => "✓ Approved",
        Some(false) => "✗ Rejected",
        None => "Pending approval",
    };

    let footer_text = format!(
        "{} | ↑/↓ Scroll | Enter: Approve | Esc: Reject | Q: Close",
        status_text
    );
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(theme.text_muted))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border)),
        );
    frame.render_widget(footer, chunks[2]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_view_creation() {
        let view = DiffView::new(
            "test.rs".to_string(),
            "old content".to_string(),
            "new content".to_string(),
        );
        assert_eq!(view.file_path, "test.rs");
        assert_eq!(view.approved, None);
    }

    #[test]
    fn test_diff_generation() {
        let view = DiffView::new(
            "test.rs".to_string(),
            "line1\nline2".to_string(),
            "line1\nline3".to_string(),
        );
        let diff = view.generate_diff();
        assert!(diff.len() >= 2);
    }
}

