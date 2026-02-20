//! Hints bar component for contextual keyboard shortcuts (gitui-style).
//!
//! Displays a bottom bar showing available actions based on current app mode.

use ratatui::{
    prelude::*,
    widgets::Paragraph,
};

use crate::components::AppMode;
use crate::theme::THEME;

/// A single hint (keyboard shortcut + description).
#[derive(Debug, Clone)]
pub struct Hint {
    /// Keyboard shortcut (e.g., "?", "Tab", "Enter")
    pub key: String,
    /// Action description (e.g., "Help", "Send Message")
    pub description: String,
    /// Whether this action is currently available
    pub available: bool,
}

impl Hint {
    /// Creates a new available hint.
    pub fn new(key: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            description: description.into(),
            available: true,
        }
    }

    /// Creates an unavailable (grayed out) hint.
    pub fn unavailable(key: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            description: description.into(),
            available: false,
        }
    }

    /// Marks this hint as available or unavailable.
    pub fn set_available(mut self, available: bool) -> Self {
        self.available = available;
        self
    }
}

/// Hints bar showing contextual keyboard shortcuts.
pub struct HintsBar {
    hints: Vec<Hint>,
}

impl HintsBar {
    /// Creates a hints bar for the given app mode.
    pub fn for_mode(mode: AppMode) -> Self {
        let hints = match mode {
            AppMode::Chat => vec![
                Hint::new("?", "Help"),
                Hint::new("Tab", "Next Pane"),
                Hint::new("Enter", "Send"),
                Hint::new("Ctrl+R", "Retry"),
                Hint::new("Esc", "Clear"),
            ],
            AppMode::Setup => vec![
                Hint::new("?", "Help"),
                Hint::new("Enter", "Continue"),
                Hint::new("Esc", "Skip"),
            ],
            AppMode::Workflow => vec![
                Hint::new("?", "Help"),
                Hint::new("↑/↓", "Navigate"),
                Hint::new("Enter", "Select"),
                Hint::new("r", "Refresh"),
                Hint::new("Esc", "Back"),
            ],
            AppMode::History => vec![
                Hint::new("?", "Help"),
                Hint::new("↑/↓", "Navigate"),
                Hint::new("Enter", "View Details"),
                Hint::new("Esc", "Back"),
            ],
            AppMode::Prompt => vec![
                Hint::new("?", "Help"),
                Hint::new("Enter", "Send"),
                Hint::new("Shift+Enter", "Newline"),
                Hint::new("Ctrl+C", "Quit"),
            ],
            AppMode::Requirement => vec![
                Hint::new("Ctrl+S", "Checkpoint"),
                Hint::new("↑/↓", "Scroll"),
                Hint::new("Esc", "Cancel"),
                Hint::new("Ctrl+C", "Force Quit"),
            ],
        };

        Self { hints }
    }

    /// Creates a custom hints bar with specific hints.
    pub fn custom(hints: Vec<Hint>) -> Self {
        Self { hints }
    }

    /// Updates a specific hint's availability.
    pub fn set_hint_available(&mut self, key: &str, available: bool) {
        if let Some(hint) = self.hints.iter_mut().find(|h| h.key == key) {
            hint.available = available;
        }
    }

    /// Adds a new hint to the bar.
    pub fn add_hint(&mut self, hint: Hint) {
        self.hints.push(hint);
    }

    /// Renders the hints bar.
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let hint_spans: Vec<Span> = self
            .hints
            .iter()
            .flat_map(|hint| {
                let key_style = if hint.available {
                    Style::default()
                        .fg(THEME.primary())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(THEME.text_dim())
                };

                let desc_style = if hint.available {
                    Style::default().fg(THEME.text())
                } else {
                    Style::default().fg(THEME.text_dim())
                };

                vec![
                    Span::raw(" "),
                    Span::styled(&hint.key, key_style),
                    Span::raw(" "),
                    Span::styled(&hint.description, desc_style),
                    Span::raw("  "),
                ]
            })
            .collect();

        let hints_line = Line::from(hint_spans);

        let hints_widget = Paragraph::new(hints_line)
            .style(Style::default().bg(THEME.bg_panel()))
            .alignment(Alignment::Left);

        frame.render_widget(hints_widget, area);
    }
}

/// Renders a hints bar for the current app mode.
pub fn render_hints_bar(frame: &mut Frame, area: Rect, mode: AppMode) {
    let hints = HintsBar::for_mode(mode);
    hints.render(frame, area);
}

/// Renders a hints bar with dynamic hint availability.
///
/// This allows conditionally graying out hints based on app state.
pub fn render_hints_bar_with_state(
    frame: &mut Frame,
    area: Rect,
    mode: AppMode,
    has_last_task: bool,
    has_streaming: bool,
) {
    let mut hints = HintsBar::for_mode(mode);

    // Conditionally disable hints based on state
    if mode == AppMode::Chat {
        if !has_last_task {
            hints.set_hint_available("Ctrl+R", false);
        }
        if has_streaming {
            hints.set_hint_available("Enter", false);
        }
    }

    hints.render(frame, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hint_creation() {
        let hint = Hint::new("Enter", "Submit");
        assert_eq!(hint.key, "Enter");
        assert_eq!(hint.description, "Submit");
        assert!(hint.available);

        let unavailable = Hint::unavailable("Esc", "Cancel");
        assert!(!unavailable.available);
    }

    #[test]
    fn test_hints_bar_for_mode() {
        let hints = HintsBar::for_mode(AppMode::Chat);
        assert!(!hints.hints.is_empty());
        assert!(hints.hints.iter().any(|h| h.key == "Enter"));
    }

    #[test]
    fn test_set_hint_available() {
        let mut hints = HintsBar::for_mode(AppMode::Chat);
        hints.set_hint_available("Enter", false);

        let enter_hint = hints.hints.iter().find(|h| h.key == "Enter").unwrap();
        assert!(!enter_hint.available);
    }

    #[test]
    fn test_custom_hints() {
        let hints = HintsBar::custom(vec![
            Hint::new("a", "Action A"),
            Hint::new("b", "Action B"),
        ]);
        assert_eq!(hints.hints.len(), 2);
    }
}
