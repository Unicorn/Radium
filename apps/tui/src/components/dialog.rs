//! Dialog/modal component for interactive selections.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

/// A choice in a dialog menu.
#[derive(Debug, Clone)]
pub struct DialogChoice {
    /// Display title
    pub title: String,
    /// Optional description
    pub description: Option<String>,
    /// Value to return when selected
    pub value: String,
}

impl DialogChoice {
    /// Creates a new dialog choice.
    pub fn new(title: String, value: String) -> Self {
        Self {
            title,
            value,
            description: None,
        }
    }

    /// Creates a dialog choice with description.
    pub fn with_description(title: String, value: String, description: String) -> Self {
        Self {
            title,
            value,
            description: Some(description),
        }
    }
}

/// Dialog state.
#[derive(Debug, Clone)]
pub struct Dialog {
    /// Dialog title/message
    pub message: String,
    /// Available choices
    pub choices: Vec<DialogChoice>,
    /// Currently selected index
    pub selected_index: usize,
    /// Whether dialog is visible
    pub visible: bool,
}

impl Dialog {
    /// Creates a new dialog.
    pub fn new(message: String, choices: Vec<DialogChoice>) -> Self {
        Self {
            message,
            choices,
            selected_index: 0,
            visible: true,
        }
    }

    /// Moves selection up.
    pub fn move_up(&mut self) {
        if !self.choices.is_empty() {
            self.selected_index = self.selected_index.saturating_sub(1);
        }
    }

    /// Moves selection down.
    pub fn move_down(&mut self) {
        if !self.choices.is_empty() {
            let max_index = self.choices.len().saturating_sub(1);
            self.selected_index = (self.selected_index + 1).min(max_index);
        }
    }

    /// Returns the selected choice value, if any.
    pub fn selected_value(&self) -> Option<String> {
        self.choices
            .get(self.selected_index)
            .map(|choice| choice.value.clone())
    }

    /// Returns the selected choice, if any.
    pub fn selected_choice(&self) -> Option<&DialogChoice> {
        self.choices.get(self.selected_index)
    }
}

/// Dialog manager for handling dialog state.
#[derive(Debug, Default)]
pub struct DialogManager {
    /// Current dialog (if any)
    current: Option<Dialog>,
}

impl DialogManager {
    /// Creates a new dialog manager.
    pub fn new() -> Self {
        Self { current: None }
    }

    /// Shows a dialog.
    pub fn show(&mut self, dialog: Dialog) {
        self.current = Some(dialog);
    }

    /// Shows a select menu dialog.
    pub fn show_select_menu(&mut self, message: String, choices: Vec<DialogChoice>) {
        self.show(Dialog::new(message, choices));
    }

    /// Closes the current dialog.
    pub fn close(&mut self) {
        self.current = None;
    }

    /// Returns whether a dialog is currently open.
    pub fn is_open(&self) -> bool {
        self.current.is_some()
    }

    /// Returns a mutable reference to the current dialog.
    pub fn current_mut(&mut self) -> Option<&mut Dialog> {
        self.current.as_mut()
    }

    /// Returns a reference to the current dialog.
    pub fn current(&self) -> Option<&Dialog> {
        self.current.as_ref()
    }

    /// Handles keyboard input for dialog navigation.
    pub fn handle_key(&mut self, key: crossterm::event::KeyCode) -> Option<String> {
        if let Some(dialog) = &mut self.current {
            match key {
                crossterm::event::KeyCode::Up => {
                    dialog.move_up();
                    None
                }
                crossterm::event::KeyCode::Down => {
                    dialog.move_down();
                    None
                }
                crossterm::event::KeyCode::Enter => {
                    let value = dialog.selected_value();
                    self.close();
                    value
                }
                crossterm::event::KeyCode::Esc => {
                    self.close();
                    None
                }
                _ => None,
            }
        } else {
            None
        }
    }
}

/// Renders a dialog overlay.
pub fn render_dialog(frame: &mut Frame, area: Rect, dialog: &Dialog) {
    let theme = crate::theme::get_theme();

    // Calculate dialog size
    let dialog_width = 60;
    let dialog_height = (dialog.choices.len() + 6).min(20) as u16; // Message + choices + borders

    // Center dialog
    let dialog_area = Rect {
        x: (area.width.saturating_sub(dialog_width)) / 2,
        y: area.height / 3,
        width: dialog_width.min(area.width),
        height: dialog_height.min(area.height.saturating_sub(area.height / 3)),
    };

    // Render backdrop (semi-transparent effect via dimmed background)
    let backdrop = Paragraph::new("")
        .style(Style::default().bg(Color::Black))
        .block(Block::default().style(Style::default().bg(Color::Black)));
    frame.render_widget(backdrop, area);

    // Split dialog area
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Message
            Constraint::Min(3),    // Choices
            Constraint::Length(2), // Help
        ])
        .split(dialog_area);

    // Message with codemachine-style prefix
    let message_text = format!("◆ {}", dialog.message);
    let message_widget = Paragraph::new(message_text)
        .style(Style::default().fg(theme.primary))
        .alignment(Alignment::Left)
        .wrap(ratatui::widgets::Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::NONE)
                .padding(ratatui::widgets::Padding::new(1, 1, 0, 1)),
        );
    frame.render_widget(message_widget, chunks[0]);

    // Choices list
    let items: Vec<ListItem> = dialog
        .choices
        .iter()
        .enumerate()
        .map(|(idx, choice)| {
            let is_selected = idx == dialog.selected_index;
            let style = if is_selected {
                Style::default()
                    .fg(theme.bg_primary)
                    .bg(theme.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };

            // Build content with title and optional description (codemachine style)
            let prefix = if is_selected { "● " } else { "○ " };
            let content = if let Some(ref desc) = choice.description {
                format!("{}{} - {}", prefix, choice.title, desc)
            } else {
                format!("{}{}", prefix, choice.title)
            };
            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::NONE)
                .padding(ratatui::widgets::Padding::new(0, 1, 0, 1)),
        );
    frame.render_widget(list, chunks[1]);

    // Help text (codemachine style)
    let help_text = "↑/↓ Navigate • Enter to select • Esc to cancel";
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(theme.text_muted))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::NONE)
                .padding(ratatui::widgets::Padding::new(1, 1, 1, 1)),
        );
    frame.render_widget(help, chunks[2]);
    
    // Render border around entire dialog
    let dialog_border = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary))
        .style(Style::default().bg(theme.bg_panel));
    frame.render_widget(dialog_border, dialog_area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialog_creation() {
        let choices = vec![
            DialogChoice::new("Option 1".to_string(), "opt1".to_string()),
            DialogChoice::new("Option 2".to_string(), "opt2".to_string()),
        ];
        let dialog = Dialog::new("Choose an option".to_string(), choices);
        assert_eq!(dialog.choices.len(), 2);
        assert_eq!(dialog.selected_index, 0);
    }

    #[test]
    fn test_dialog_navigation() {
        let choices = vec![
            DialogChoice::new("Option 1".to_string(), "opt1".to_string()),
            DialogChoice::new("Option 2".to_string(), "opt2".to_string()),
            DialogChoice::new("Option 3".to_string(), "opt3".to_string()),
        ];
        let mut dialog = Dialog::new("Choose".to_string(), choices);

        assert_eq!(dialog.selected_index, 0);
        dialog.move_down();
        assert_eq!(dialog.selected_index, 1);
        dialog.move_down();
        assert_eq!(dialog.selected_index, 2);
        dialog.move_down(); // Should not go beyond
        assert_eq!(dialog.selected_index, 2);
        dialog.move_up();
        assert_eq!(dialog.selected_index, 1);
        dialog.move_up();
        assert_eq!(dialog.selected_index, 0);
        dialog.move_up(); // Should not go below 0
        assert_eq!(dialog.selected_index, 0);
    }

    #[test]
    fn test_dialog_manager() {
        let mut manager = DialogManager::new();
        assert!(!manager.is_open());

        let choices = vec![DialogChoice::new("Test".to_string(), "test".to_string())];
        manager.show_select_menu("Test dialog".to_string(), choices);
        assert!(manager.is_open());

        manager.close();
        assert!(!manager.is_open());
    }

    #[test]
    fn test_dialog_selection() {
        let choices = vec![
            DialogChoice::new("Option 1".to_string(), "opt1".to_string()),
            DialogChoice::new("Option 2".to_string(), "opt2".to_string()),
        ];
        let dialog = Dialog::new("Choose".to_string(), choices);
        assert_eq!(dialog.selected_value(), Some("opt1".to_string()));

        let mut dialog = Dialog::new("Choose".to_string(), dialog.choices);
        dialog.move_down();
        assert_eq!(dialog.selected_value(), Some("opt2".to_string()));
    }
}

