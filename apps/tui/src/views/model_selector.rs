//! Model selection UI.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
    layout::Constraint,
};

use crate::components::InteractiveTable;
use crate::icons::Icons;
use crate::theme::THEME;

/// Format cost per million tokens
fn format_cost(cost: Option<f64>) -> String {
    cost.map(|c| format!("${:.2}", c)).unwrap_or_else(|| "N/A".to_string())
}

/// Format context window size
fn format_context_window(tokens: Option<u32>) -> String {
    tokens.map(|t| {
        if t >= 1_000_000 {
            format!("{}M", t / 1_000_000)
        } else if t >= 1_000 {
            format!("{}K", t / 1_000)
        } else {
            format!("{}", t)
        }
    }).unwrap_or_else(|| "N/A".to_string())
}

/// Format capabilities as icons
fn format_capabilities(caps: &[String]) -> String {
    caps.iter().map(|c| match c.as_str() {
        "vision" => "👁️",
        "tools" => "🔧",
        "reasoning" => "🧠",
        _ => "",
    }).collect::<Vec<_>>().join(" ")
}

/// Model information.
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub description: Option<String>,
    pub is_selected: bool,
    pub input_cost_per_million: Option<f64>,
    pub output_cost_per_million: Option<f64>,
    pub context_window: Option<u32>,
    pub capabilities: Vec<String>,
}

/// Render the model selector view.
pub fn render_model_selector(
    frame: &mut Frame,
    area: Rect,
    models: &[ModelInfo],
    selected_index: usize,
    filter: Option<&crate::app::ModelFilter>,
) {
    // Apply filters if provided
    let filtered_models: Vec<&ModelInfo> = if let Some(f) = filter {
        models.iter().filter(|m| f.matches(&m.capabilities)).collect()
    } else {
        models.iter().collect()
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(1), // Filter indicator
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

    // Filter indicator
    let filter_text = if let Some(f) = filter {
        if f.is_active() {
            let mut filters = Vec::new();
            if f.vision { filters.push("👁️"); }
            if f.tools { filters.push("🔧"); }
            if f.reasoning { filters.push("🧠"); }
            format!("Filters: {}", filters.join(" "))
        } else {
            "Filters: None".to_string()
        }
    } else {
        "Filters: None".to_string()
    };
    let filter_widget = Paragraph::new(filter_text)
        .style(Style::default().fg(THEME.text_muted()))
        .alignment(Alignment::Center);
    frame.render_widget(filter_widget, chunks[1]);

    // Build model table items
    let table_items: Vec<Vec<String>> = filtered_models
        .iter()
        .map(|model| {
            let status = if model.is_selected { "✓" } else { " " };
            let desc = model.description.clone().unwrap_or_else(|| "N/A".to_string());
            let capabilities = format_capabilities(&model.capabilities);
            let desc_with_caps = if capabilities.is_empty() {
                desc
            } else {
                format!("{} {}", desc, capabilities)
            };
            vec![
                status.to_string(),
                model.name.clone(),
                model.provider.clone(),
                desc_with_caps,
            ]
        })
        .collect();

    if table_items.is_empty() {
        let empty_text = if let Some(f) = filter {
            if f.is_active() {
                "No models match the active filters"
            } else {
                "No models available"
            }
        } else {
            "No models available"
        };
        let empty_widget = Paragraph::new(empty_text)
            .style(Style::default().fg(THEME.text_muted()))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(THEME.border()))
                    .title(" Available Models "),
            );
        frame.render_widget(empty_widget, chunks[2]);
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
        table.render(frame, chunks[2], Some(" Available Models "));
    }

    // Help line
    let help_text = "v=Vision | t=Tools | r=Reasoning | a=All | Enter=Select | Esc=Cancel";
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(THEME.text_muted()))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[2]);
}
