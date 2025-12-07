//! Model selection UI.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::icons::Icons;
use crate::theme::THEME;

/// Model information.
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub description: Option<String>,
    pub is_selected: bool,
}

/// Render the model selector view.
pub fn render_model_selector(
    frame: &mut Frame,
    area: Rect,
    models: &[ModelInfo],
    selected_index: usize,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Model list
            Constraint::Length(2), // Help
        ])
        .split(area);

    // Title
    let title = Paragraph::new(format!("{} Available Models", Icons::SETTINGS))
        .style(Style::default().fg(THEME.primary()).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default().borders(Borders::ALL).border_style(Style::default().fg(THEME.border)),
        );
    frame.render_widget(title, chunks[0]);

    // Build model list items
    let items: Vec<ListItem> = models
        .iter()
        .enumerate()
        .map(|(i, model)| {
            let is_selected = i == selected_index;
            let is_current = model.is_selected;

            let style = if is_selected {
                Style::default().fg(THEME.bg_primary()).bg(THEME.primary()).add_modifier(Modifier::BOLD)
            } else if is_current {
                Style::default().fg(THEME.success()).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(THEME.text)
            };

            let prefix = if is_current {
                "[x]"
            } else if is_selected {
                "[▶]"
            } else {
                "[ ]"
            };

            let model_text = if let Some(desc) = &model.description {
                format!("{} {} ({}) - {}", prefix, model.name, model.provider, desc)
            } else {
                format!("{} {} ({})", prefix, model.name, model.provider)
            };

            ListItem::new(Line::from(Span::styled(model_text, style)))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default().borders(Borders::ALL).border_style(Style::default().fg(THEME.border)),
        )
        .style(Style::default().fg(THEME.text));

    frame.render_widget(list, chunks[1]);

    // Help line
    let help_text = "Press number to select | Enter to confirm | Esc to cancel";
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(THEME.text_muted()))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[2]);
}
