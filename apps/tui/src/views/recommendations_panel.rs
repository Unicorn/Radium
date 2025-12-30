//! Recommendations and next steps panel with interactive execution.
//!
//! Displays actionable recommendations that users can choose to execute.
//! Provides a transparent way for Radium to propose and perform actions.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

/// A single actionable recommendation.
#[derive(Debug, Clone)]
pub struct Recommendation {
    /// Short description of the action
    pub description: String,

    /// Detailed explanation of what this will do
    pub details: Option<String>,

    /// Command to execute (if applicable)
    pub command: Option<String>,

    /// Execution status
    pub status: RecommendationStatus,

    /// Execution result/output
    pub result: Option<String>,
}

/// Status of a recommendation.
#[derive(Debug, Clone, PartialEq)]
pub enum RecommendationStatus {
    /// Not yet executed
    Pending,

    /// Currently executing
    Executing,

    /// Executed successfully
    Completed,

    /// Execution failed
    Failed,

    /// Skipped by user
    Skipped,
}

impl RecommendationStatus {
    /// Get the symbol for this status
    pub fn symbol(&self) -> &str {
        match self {
            RecommendationStatus::Pending => "○",
            RecommendationStatus::Executing => "⠋",
            RecommendationStatus::Completed => "✓",
            RecommendationStatus::Failed => "✗",
            RecommendationStatus::Skipped => "⊘",
        }
    }

    /// Get the color for this status
    pub fn color(&self) -> Color {
        match self {
            RecommendationStatus::Pending => Color::White,
            RecommendationStatus::Executing => Color::Yellow,
            RecommendationStatus::Completed => Color::Green,
            RecommendationStatus::Failed => Color::Red,
            RecommendationStatus::Skipped => Color::DarkGray,
        }
    }
}

/// Interaction mode for the recommendations panel.
#[derive(Debug, Clone, PartialEq)]
pub enum InteractionMode {
    /// Just displaying recommendations (no interaction)
    Display,

    /// Asking user if they want to execute
    AwaitingConfirmation,

    /// Currently executing recommendations
    Executing,

    /// Execution complete
    Completed,
}

/// Recommendations panel state.
pub struct RecommendationsPanel {
    /// List of recommendations
    recommendations: Vec<Recommendation>,

    /// Current interaction mode
    mode: InteractionMode,

    /// Whether the panel is visible
    visible: bool,

    /// Summary/context for these recommendations
    context: Option<String>,

    /// Current recommendation being executed (index)
    current_execution_index: Option<usize>,
}

impl RecommendationsPanel {
    /// Creates a new recommendations panel.
    pub fn new() -> Self {
        Self {
            recommendations: Vec::new(),
            mode: InteractionMode::Display,
            visible: false,
            context: None,
            current_execution_index: None,
        }
    }

    /// Starts a new recommendations session.
    pub fn start_session(&mut self, context: impl Into<String>) {
        self.recommendations.clear();
        self.context = Some(context.into());
        self.mode = InteractionMode::Display;
        self.visible = true;
        self.current_execution_index = None;
    }

    /// Adds a recommendation.
    pub fn add_recommendation(
        &mut self,
        description: impl Into<String>,
        command: Option<String>,
        details: Option<String>,
    ) {
        self.recommendations.push(Recommendation {
            description: description.into(),
            details,
            command,
            status: RecommendationStatus::Pending,
            result: None,
        });
    }

    /// Prompts user for execution confirmation.
    pub fn request_execution(&mut self) {
        if !self.recommendations.is_empty() {
            self.mode = InteractionMode::AwaitingConfirmation;
        }
    }

    /// User confirmed execution.
    pub fn confirm_execution(&mut self) -> bool {
        if self.mode == InteractionMode::AwaitingConfirmation {
            self.mode = InteractionMode::Executing;
            self.current_execution_index = Some(0);
            true
        } else {
            false
        }
    }

    /// User declined execution.
    pub fn decline_execution(&mut self) {
        self.mode = InteractionMode::Display;
        // Mark all as skipped
        for rec in &mut self.recommendations {
            if rec.status == RecommendationStatus::Pending {
                rec.status = RecommendationStatus::Skipped;
            }
        }
    }

    /// Gets the next recommendation to execute.
    pub fn get_next_to_execute(&self) -> Option<(usize, &Recommendation)> {
        if self.mode != InteractionMode::Executing {
            return None;
        }

        self.current_execution_index
            .and_then(|idx| self.recommendations.get(idx).map(|rec| (idx, rec)))
    }

    /// Marks current recommendation as executing.
    pub fn mark_executing(&mut self, index: usize) {
        if let Some(rec) = self.recommendations.get_mut(index) {
            rec.status = RecommendationStatus::Executing;
        }
    }

    /// Marks current recommendation as completed.
    pub fn mark_completed(&mut self, index: usize, result: impl Into<String>) {
        if let Some(rec) = self.recommendations.get_mut(index) {
            rec.status = RecommendationStatus::Completed;
            rec.result = Some(result.into());
        }

        // Move to next
        self.current_execution_index = Some(index + 1);

        // Check if all done
        if index + 1 >= self.recommendations.len() {
            self.mode = InteractionMode::Completed;
            self.current_execution_index = None;
        }
    }

    /// Marks current recommendation as failed.
    pub fn mark_failed(&mut self, index: usize, error: impl Into<String>) {
        if let Some(rec) = self.recommendations.get_mut(index) {
            rec.status = RecommendationStatus::Failed;
            rec.result = Some(error.into());
        }

        // Stop execution on failure
        self.mode = InteractionMode::Completed;
        self.current_execution_index = None;
    }

    /// Checks if panel is visible.
    pub fn is_visible(&self) -> bool {
        self.visible && !self.recommendations.is_empty()
    }

    /// Checks if awaiting user confirmation.
    pub fn is_awaiting_confirmation(&self) -> bool {
        self.mode == InteractionMode::AwaitingConfirmation
    }

    /// Checks if currently executing.
    pub fn is_executing(&self) -> bool {
        self.mode == InteractionMode::Executing
    }

    /// Gets current mode.
    pub fn mode(&self) -> &InteractionMode {
        &self.mode
    }

    /// Clears all recommendations.
    pub fn clear(&mut self) {
        self.recommendations.clear();
        self.context = None;
        self.visible = false;
        self.mode = InteractionMode::Display;
        self.current_execution_index = None;
    }

    /// Renders the recommendations panel.
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.is_visible() {
            return;
        }

        let theme = crate::theme::get_theme();

        // Create title based on mode
        let title = match self.mode {
            InteractionMode::Display => " 📋 Next Steps ",
            InteractionMode::AwaitingConfirmation => " 📋 Execute? (Y/N) ",
            InteractionMode::Executing => " 📋 Executing... ",
            InteractionMode::Completed => " 📋 Completed ",
        };

        let border_color = match self.mode {
            InteractionMode::AwaitingConfirmation => theme.warning,
            InteractionMode::Executing => theme.info,
            InteractionMode::Completed => theme.success,
            _ => theme.border,
        };

        let block = Block::default()
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .border_style(Style::default().fg(border_color))
            .border_type(ratatui::widgets::BorderType::Rounded)
            .title(title)
            .padding(ratatui::widgets::Padding::new(1, 1, 0, 0));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let mut lines = Vec::new();

        // Add context if present
        if let Some(ref context) = self.context {
            lines.push(Line::from(vec![
                Span::styled(context, Style::default().fg(theme.text).add_modifier(Modifier::ITALIC)),
            ]));
            lines.push(Line::from(""));
        }

        // Add each recommendation
        for (idx, rec) in self.recommendations.iter().enumerate() {
            // Number, status, and description
            lines.push(Line::from(vec![
                Span::styled(format!("{}. ", idx + 1), Style::default().fg(theme.text)),
                Span::styled(rec.status.symbol(), Style::default().fg(rec.status.color())),
                Span::raw(" "),
                Span::styled(&rec.description, Style::default().fg(theme.text)),
            ]));

            // Show command if present (dimmed)
            if let Some(ref command) = rec.command {
                lines.push(Line::from(vec![
                    Span::raw("   "),
                    Span::styled("$ ", Style::default().fg(theme.text_muted)),
                    Span::styled(command, Style::default().fg(theme.text_muted).add_modifier(Modifier::DIM)),
                ]));
            }

            // Show details if present
            if let Some(ref details) = rec.details {
                lines.push(Line::from(vec![
                    Span::raw("   "),
                    Span::styled(details, Style::default().fg(theme.text_muted)),
                ]));
            }

            // Show result if executed
            if let Some(ref result) = rec.result {
                let result_color = match rec.status {
                    RecommendationStatus::Completed => theme.success,
                    RecommendationStatus::Failed => theme.error,
                    _ => theme.text_muted,
                };

                for result_line in result.lines() {
                    lines.push(Line::from(vec![
                        Span::raw("   "),
                        Span::styled("→ ", Style::default().fg(result_color)),
                        Span::styled(result_line, Style::default().fg(result_color)),
                    ]));
                }
            }

            // Spacing between recommendations
            if idx < self.recommendations.len() - 1 {
                lines.push(Line::from(""));
            }
        }

        // Add prompt based on mode
        if self.mode == InteractionMode::AwaitingConfirmation {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("→ ", Style::default().fg(theme.warning)),
                Span::styled(
                    "Execute these steps? Press Y to proceed, N to skip",
                    Style::default().fg(theme.warning).add_modifier(Modifier::BOLD),
                ),
            ]));
        }

        let paragraph = Paragraph::new(lines)
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(paragraph, inner);
    }
}

impl Default for RecommendationsPanel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recommendations_panel_creation() {
        let panel = RecommendationsPanel::new();
        assert!(!panel.is_visible());
        assert_eq!(panel.recommendations.len(), 0);
    }

    #[test]
    fn test_add_recommendation() {
        let mut panel = RecommendationsPanel::new();
        panel.start_session("Test coverage improvements");
        panel.add_recommendation(
            "Install coverage tool",
            Some("cargo install cargo-tarpaulin".to_string()),
            None,
        );

        assert_eq!(panel.recommendations.len(), 1);
        assert_eq!(panel.recommendations[0].description, "Install coverage tool");
    }

    #[test]
    fn test_execution_flow() {
        let mut panel = RecommendationsPanel::new();
        panel.start_session("Test");
        panel.add_recommendation("Step 1", Some("echo test".to_string()), None);

        // Request execution
        panel.request_execution();
        assert!(panel.is_awaiting_confirmation());

        // Confirm
        assert!(panel.confirm_execution());
        assert!(panel.is_executing());

        // Mark completed
        panel.mark_completed(0, "Success");
        assert_eq!(panel.recommendations[0].status, RecommendationStatus::Completed);
        assert_eq!(panel.mode, InteractionMode::Completed);
    }

    #[test]
    fn test_decline_execution() {
        let mut panel = RecommendationsPanel::new();
        panel.start_session("Test");
        panel.add_recommendation("Step 1", None, None);
        panel.request_execution();

        panel.decline_execution();
        assert_eq!(panel.recommendations[0].status, RecommendationStatus::Skipped);
        assert_eq!(panel.mode, InteractionMode::Display);
    }
}
