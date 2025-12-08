//! Model selection UI.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
    layout::Constraint,
};

use crate::components::InteractiveTable;
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
            Block::default().borders(Borders::ALL).border_style(Style::default().fg(THEME.border())),
        );
    frame.render_widget(title, chunks[0]);

    // Build model table items
    let table_items: Vec<Vec<String>> = models
        .iter()
        .map(|model| {
            let status = if model.is_selected { "✓" } else { " " };
            vec![
                status.to_string(),
                model.name.clone(),
                model.provider.clone(),
                model.description.clone().unwrap_or_else(|| "N/A".to_string()),
            ]
        })
        .collect();

    if table_items.is_empty() {
        let empty_text = "No models available";
        let empty_widget = Paragraph::new(empty_text)
            .style(Style::default().fg(THEME.text_muted()))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(THEME.border()))
                    .title(" Available Models "),
            );
        frame.render_widget(empty_widget, chunks[1]);
    } else {
        let mut table = InteractiveTable::new(
            vec!["".to_string(), "Name".to_string(), "Provider".to_string(), "Description".to_string()],
            vec![
                Constraint::Length(3),  // Status
                Constraint::Percentage(25), // Name
                Constraint::Percentage(20), // Provider
                Constraint::Percentage(52), // Description
            ],
        );
        let items_len = table_items.len();
        table.set_items(table_items);
        table.set_selected(Some(selected_index.min(items_len.saturating_sub(1))));
        table.render(frame, chunks[1], Some(" Available Models "));
    }

    // Help line
    let help_text = "Press number to select | Enter to confirm | Esc to cancel";
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(THEME.text_muted()))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[2]);
}
