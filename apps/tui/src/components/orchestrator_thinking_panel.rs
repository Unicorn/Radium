//! Orchestrator thinking panel component for displaying orchestrator decision-making logs.
//!
//! This component displays real-time orchestrator reasoning and decision-making process
//! in a scrollable text view with syntax highlighting.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Line, Paragraph, Wrap},
};
use crate::state::OutputBuffer;
use crate::theme::get_theme;

/// Orchestrator thinking panel component
#[derive(Debug, Clone)]
pub struct OrchestratorThinkingPanel {
    /// Output buffer for storing log lines
    output_buffer: OutputBuffer,
    /// Whether auto-scroll is enabled
    auto_scroll: bool,
    /// Whether the panel is focused
    focused: bool,
}

impl OrchestratorThinkingPanel {
    /// Creates a new orchestrator thinking panel.
    pub fn new() -> Self {
        Self {
            output_buffer: OutputBuffer::new(1000),
            auto_scroll: true,
            focused: false,
        }
    }

    /// Appends a new log line to the buffer.
    ///
    /// # Arguments
    /// * `line` - Log line to append
    pub fn append_log(&mut self, line: String) {
        self.output_buffer.append_line(line);
        // Auto-scroll to bottom when new logs arrive
        if self.auto_scroll {
            self.output_buffer.scroll_to_bottom();
        }
    }

    /// Appends multiple log lines to the buffer.
    ///
    /// # Arguments
    /// * `lines` - Log lines to append
    pub fn append_logs(&mut self, lines: Vec<String>) {
        for line in lines {
            self.append_log(line);
        }
    }

    /// Sets the auto-scroll behavior.
    ///
    /// # Arguments
    /// * `enabled` - Whether auto-scroll should be enabled
    pub fn set_auto_scroll(&mut self, enabled: bool) {
        self.auto_scroll = enabled;
    }

    /// Sets the focus state of the panel.
    ///
    /// # Arguments
    /// * `focused` - Whether the panel is focused
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Scrolls up by the specified number of lines.
    ///
    /// # Arguments
    /// * `amount` - Number of lines to scroll up
    pub fn scroll_up(&mut self, amount: usize) {
        self.output_buffer.scroll_up(amount);
        // Disable auto-scroll when user manually scrolls
        self.auto_scroll = false;
    }

    /// Scrolls down by the specified number of lines.
    ///
    /// # Arguments
    /// * `amount` - Number of lines to scroll down
    pub fn scroll_down(&mut self, amount: usize) {
        self.output_buffer.scroll_down(amount);
        // Disable auto-scroll when user manually scrolls
        self.auto_scroll = false;
    }

    /// Scrolls to the top.
    pub fn scroll_to_top(&mut self) {
        self.output_buffer.scroll_to_top();
        self.auto_scroll = false;
    }

    /// Scrolls to the bottom.
    pub fn scroll_to_bottom(&mut self) {
        self.output_buffer.scroll_to_bottom();
        self.auto_scroll = true;
    }

    /// Renders the orchestrator thinking panel.
    ///
    /// # Arguments
    /// * `frame` - Frame to render into
    /// * `area` - Area to render in
    /// * `focused` - Whether the panel is focused
    pub fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool) {
        self.focused = focused;
        let theme = get_theme();

        // Calculate viewport height (subtract borders)
        let viewport_height = area.height.saturating_sub(2) as usize;

        // Get visible lines from buffer
        let visible_lines = self.output_buffer.visible_lines(viewport_height);
        let total_lines = self.output_buffer.lines.len();
        let scroll_position = self.output_buffer.scroll_position;

        // Apply syntax highlighting to lines
        let styled_lines: Vec<Line> = visible_lines
            .iter()
            .map(|line| Self::apply_syntax_highlighting(line, &theme))
            .collect();

        // Create title with scroll position indicator
        let title = if total_lines > 0 {
            format!(
                " Orchestrator Thinking (line {}/{} - {:.0}%) ",
                scroll_position + 1,
                total_lines,
                if total_lines > 0 {
                    ((scroll_position as f64 / total_lines.saturating_sub(1).max(1) as f64) * 100.0)
                } else {
                    0.0
                }
            )
        } else {
            " Orchestrator Thinking ".to_string()
        };

        // Render empty state or content
        if styled_lines.is_empty() {
            let empty_text = "Waiting for orchestrator...";
            let empty_widget = Paragraph::new(empty_text)
                .style(Style::default().fg(theme.text_muted))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(if focused {
                            Style::default().fg(theme.border_active)
                        } else {
                            Style::default().fg(theme.border)
                        })
                        .title(title),
                );
            frame.render_widget(empty_widget, area);
        } else {
            let paragraph = Paragraph::new(styled_lines)
                .wrap(Wrap { trim: true })
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(if focused {
                            Style::default().fg(theme.border_active)
                        } else {
                            Style::default().fg(theme.border)
                        })
                        .title(title),
                );

            frame.render_widget(paragraph, area);
        }
    }

    /// Applies syntax highlighting to a log line.
    ///
    /// # Arguments
    /// * `line` - Line to highlight
    /// * `theme` - Theme for colors
    ///
    /// # Returns
    /// Styled line with syntax highlighting
    fn apply_syntax_highlighting(line: &str, theme: &crate::theme::RadiumTheme) -> Line {
        // Check for orchestrator prefix
        if line.starts_with("[Orchestrator]") {
            let prefix_end = "[Orchestrator]".len();
            let prefix = &line[..prefix_end];
            let rest = &line[prefix_end..];

            Line::from(vec![
                Span::styled(prefix, Style::default().fg(theme.primary)),
                Span::styled(rest, Style::default().fg(theme.text)),
            ])
        } else if line.contains("Analyzing") || line.contains("Selected") || line.contains("Executing") {
            // Keywords in info color
            Line::from(Span::styled(line, Style::default().fg(theme.info)))
        } else if line.contains("Error") || line.contains("Failed") || line.contains("error") || line.contains("failed") {
            // Errors in error color
            Line::from(Span::styled(line, Style::default().fg(theme.error)))
        } else if line.contains("Completed") || line.contains("Success") || line.contains("completed") || line.contains("success") {
            // Success in success color
            Line::from(Span::styled(line, Style::default().fg(theme.success)))
        } else {
            // Default text color
            Line::from(Span::styled(line, Style::default().fg(theme.text_muted)))
        }
    }

    /// Clears all logs from the buffer.
    pub fn clear(&mut self) {
        self.output_buffer.clear();
    }

    /// Returns the number of log lines in the buffer.
    pub fn len(&self) -> usize {
        self.output_buffer.lines.len()
    }

    /// Returns whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.output_buffer.lines.is_empty()
    }
}

impl Default for OrchestratorThinkingPanel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orchestrator_thinking_panel_new() {
        let panel = OrchestratorThinkingPanel::new();
        assert!(panel.is_empty());
        assert!(panel.auto_scroll);
        assert!(!panel.focused);
    }

    #[test]
    fn test_append_log() {
        let mut panel = OrchestratorThinkingPanel::new();
        panel.append_log("Test log line".to_string());
        assert_eq!(panel.len(), 1);
        assert!(!panel.is_empty());
    }

    #[test]
    fn test_scroll_operations() {
        let mut panel = OrchestratorThinkingPanel::new();
        
        // Add some logs
        for i in 0..10 {
            panel.append_log(format!("Log line {}", i));
        }
        
        assert_eq!(panel.len(), 10);
        
        // Scroll up should disable auto-scroll
        panel.scroll_up(2);
        assert!(!panel.auto_scroll);
        
        // Scroll to bottom should enable auto-scroll
        panel.scroll_to_bottom();
        assert!(panel.auto_scroll);
    }

    #[test]
    fn test_clear() {
        let mut panel = OrchestratorThinkingPanel::new();
        panel.append_log("Test log".to_string());
        assert!(!panel.is_empty());
        
        panel.clear();
        assert!(panel.is_empty());
    }
}

