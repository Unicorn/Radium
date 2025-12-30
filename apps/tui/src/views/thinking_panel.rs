//! Thinking process visualization panel.
//!
//! Displays the agent's reasoning process in real-time with collapsible sections.
//! Provides transparency into how the agent is analyzing problems and making decisions.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

/// A single thinking step in the reasoning process.
#[derive(Debug, Clone)]
pub struct ThinkingStep {
    /// Step description (e.g., "Checking for coverage reports")
    pub description: String,

    /// Current status of this step
    pub status: ThinkingStepStatus,

    /// Timestamp when step started
    pub timestamp: std::time::Instant,

    /// Optional details or findings
    pub details: Option<String>,
}

/// Status of a thinking step.
#[derive(Debug, Clone, PartialEq)]
pub enum ThinkingStepStatus {
    /// Step is currently in progress
    InProgress,

    /// Step completed successfully
    Completed,

    /// Step completed with findings
    CompletedWithFindings,

    /// Step failed or found nothing
    Failed,
}

impl ThinkingStepStatus {
    /// Get the symbol for this status
    pub fn symbol(&self) -> &str {
        match self {
            ThinkingStepStatus::InProgress => "⠋",
            ThinkingStepStatus::Completed => "✓",
            ThinkingStepStatus::CompletedWithFindings => "●",
            ThinkingStepStatus::Failed => "✗",
        }
    }

    /// Get the color for this status
    pub fn color(&self) -> Color {
        match self {
            ThinkingStepStatus::InProgress => Color::Yellow,
            ThinkingStepStatus::Completed => Color::Green,
            ThinkingStepStatus::CompletedWithFindings => Color::Cyan,
            ThinkingStepStatus::Failed => Color::Red,
        }
    }
}

/// Thinking panel state and configuration.
pub struct ThinkingPanel {
    /// List of thinking steps
    steps: Vec<ThinkingStep>,

    /// Whether the panel is currently visible
    visible: bool,

    /// Whether the panel is expanded (showing details)
    expanded: bool,

    /// Current reasoning context
    context: Option<String>,
}

impl ThinkingPanel {
    /// Creates a new thinking panel.
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            visible: false,
            expanded: true,
            context: None,
        }
    }

    /// Starts a new thinking session with context.
    pub fn start_session(&mut self, context: impl Into<String>) {
        self.steps.clear();
        self.context = Some(context.into());
        self.visible = true;
    }

    /// Adds a new thinking step.
    pub fn add_step(&mut self, description: impl Into<String>) {
        self.steps.push(ThinkingStep {
            description: description.into(),
            status: ThinkingStepStatus::InProgress,
            timestamp: std::time::Instant::now(),
            details: None,
        });
    }

    /// Updates the status of the most recent step.
    pub fn update_last_step(&mut self, status: ThinkingStepStatus, details: Option<String>) {
        if let Some(step) = self.steps.last_mut() {
            step.status = status;
            step.details = details;
        }
    }

    /// Marks a specific step as completed with findings.
    pub fn complete_step(&mut self, index: usize, findings: impl Into<String>) {
        if let Some(step) = self.steps.get_mut(index) {
            step.status = ThinkingStepStatus::CompletedWithFindings;
            step.details = Some(findings.into());
        }
    }

    /// Ends the current thinking session.
    pub fn end_session(&mut self) {
        // Mark any in-progress steps as completed
        for step in &mut self.steps {
            if step.status == ThinkingStepStatus::InProgress {
                step.status = ThinkingStepStatus::Completed;
            }
        }
    }

    /// Toggles panel visibility.
    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }

    /// Toggles expanded/collapsed state.
    pub fn toggle_expanded(&mut self) {
        self.expanded = !self.expanded;
    }

    /// Checks if panel is currently visible.
    pub fn is_visible(&self) -> bool {
        self.visible && !self.steps.is_empty()
    }

    /// Checks if panel is expanded.
    pub fn is_expanded(&self) -> bool {
        self.expanded
    }

    /// Clears all thinking steps.
    pub fn clear(&mut self) {
        self.steps.clear();
        self.context = None;
        self.visible = false;
    }

    /// Renders the thinking panel.
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.is_visible() {
            return;
        }

        let theme = crate::theme::get_theme();

        // Create border style based on expanded state
        let border_style = if self.expanded {
            Style::default().fg(theme.info)
        } else {
            Style::default().fg(theme.text_muted)
        };

        // Build title with toggle hint
        let title = if self.expanded {
            " 💭 Thinking "
        } else {
            " 💭 Thinking "
        };

        let block = Block::default()
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .border_style(border_style)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .title(title)
            .padding(ratatui::widgets::Padding::new(1, 1, 0, 0));

        if !self.expanded {
            // Collapsed view: just show count
            let inner = block.inner(area);
            frame.render_widget(block, area);

            let summary = if let Some(ref context) = self.context {
                format!("{} steps • {}", self.steps.len(), context)
            } else {
                format!("{} steps", self.steps.len())
            };

            let summary_widget = Paragraph::new(summary)
                .style(Style::default().fg(theme.text_muted));

            frame.render_widget(summary_widget, inner);
            return;
        }

        // Expanded view: show all steps
        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Build list of steps
        let mut lines = Vec::new();

        // Add context if present
        if let Some(ref context) = self.context {
            lines.push(Line::from(vec![
                Span::styled("Context: ", Style::default().fg(theme.text_muted)),
                Span::styled(context, Style::default().fg(theme.text).add_modifier(Modifier::ITALIC)),
            ]));
            lines.push(Line::from(""));
        }

        // Add each thinking step
        for (idx, step) in self.steps.iter().enumerate() {
            let elapsed = step.timestamp.elapsed();
            let elapsed_str = if elapsed.as_secs() > 0 {
                format!("{}s", elapsed.as_secs())
            } else {
                format!("{}ms", elapsed.as_millis())
            };

            // Step number and description
            lines.push(Line::from(vec![
                Span::styled(format!("{}. ", idx + 1), Style::default().fg(theme.text_muted)),
                Span::styled(step.status.symbol(), Style::default().fg(step.status.color())),
                Span::raw(" "),
                Span::styled(&step.description, Style::default().fg(theme.text)),
                Span::raw(" "),
                Span::styled(format!("({})", elapsed_str), Style::default().fg(theme.text_muted).add_modifier(Modifier::DIM)),
            ]));

            // Add details if present and expanded
            if let Some(ref details) = step.details {
                for detail_line in details.lines() {
                    lines.push(Line::from(vec![
                        Span::raw("   "),
                        Span::styled("→ ", Style::default().fg(theme.text_muted)),
                        Span::styled(detail_line, Style::default().fg(theme.text_muted)),
                    ]));
                }
            }

            // Add spacing between steps
            if idx < self.steps.len() - 1 {
                lines.push(Line::from(""));
            }
        }

        let paragraph = Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .style(Style::default());

        frame.render_widget(paragraph, inner);
    }
}

impl Default for ThinkingPanel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thinking_panel_creation() {
        let panel = ThinkingPanel::new();
        assert!(!panel.is_visible());
        assert_eq!(panel.steps.len(), 0);
    }

    #[test]
    fn test_start_session() {
        let mut panel = ThinkingPanel::new();
        panel.start_session("Analyzing test coverage");

        assert!(panel.is_visible());
        assert_eq!(panel.context, Some("Analyzing test coverage".to_string()));
    }

    #[test]
    fn test_add_step() {
        let mut panel = ThinkingPanel::new();
        panel.start_session("Test");
        panel.add_step("Checking files");

        assert_eq!(panel.steps.len(), 1);
        assert_eq!(panel.steps[0].description, "Checking files");
        assert_eq!(panel.steps[0].status, ThinkingStepStatus::InProgress);
    }

    #[test]
    fn test_update_last_step() {
        let mut panel = ThinkingPanel::new();
        panel.start_session("Test");
        panel.add_step("Step 1");
        panel.update_last_step(ThinkingStepStatus::Completed, Some("Found 5 files".to_string()));

        assert_eq!(panel.steps[0].status, ThinkingStepStatus::Completed);
        assert_eq!(panel.steps[0].details, Some("Found 5 files".to_string()));
    }

    #[test]
    fn test_toggle_visibility() {
        let mut panel = ThinkingPanel::new();
        panel.start_session("Test");

        assert!(panel.is_visible());
        panel.toggle_visibility();
        assert!(!panel.is_visible());
    }

    #[test]
    fn test_toggle_expanded() {
        let mut panel = ThinkingPanel::new();

        assert!(panel.is_expanded());
        panel.toggle_expanded();
        assert!(!panel.is_expanded());
    }
}
