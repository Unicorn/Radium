//! Orchestrator view with split-panel layout.
//!
//! Shows chat log on the left and active agents on the right when orchestrator is running.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
    layout::Constraint,
};

use crate::components::InteractiveTable;
use crate::views::prompt::PromptData;

/// Renders the orchestrator view with split panels
pub fn render_orchestrator_view(
    frame: &mut Frame,
    area: Rect,
    prompt_data: &PromptData,
    active_agents: &[(String, String, String)], // (agent_id, agent_name, status)
) {
    let theme = crate::theme::get_theme();

    // Split main area horizontally: chat log (left) and agents (right)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1), // Chat log (50%)
            Constraint::Fill(1), // Active agents (50%)
        ])
        .split(area);

    // Left panel: Chat log
    render_chat_log(frame, chunks[0], prompt_data, &theme);

    // Right panel: Active agents
    render_active_agents(frame, chunks[1], active_agents, &theme);
}

/// Renders the chat log panel
fn render_chat_log(frame: &mut Frame, area: Rect, prompt_data: &PromptData, theme: &crate::theme::RadiumTheme) {
    // Calculate viewport height
    let viewport_height = area.height.saturating_sub(2) as usize;
    
    // Get visible conversation lines
    let visible_conversation = prompt_data.get_visible_conversation(viewport_height);

    // Parse and color-code messages
    let mut styled_lines = Vec::new();
    for line in &visible_conversation {
        let styled_line = if line.starts_with("You: ") {
            // User message - primary color
            Line::from(Span::styled(
                line.clone(),
                Style::default().fg(theme.primary),
            ))
        } else if line.starts_with("Agent: ") || line.starts_with("Assistant: ") {
            // Agent/assistant message - info color
            Line::from(Span::styled(
                line.clone(),
                Style::default().fg(theme.info),
            ))
        } else if line.starts_with("Error: ") || line.starts_with("❌") {
            // Error message - error color
            Line::from(Span::styled(
                line.clone(),
                Style::default().fg(theme.error),
            ))
        } else if line.starts_with("⚠️") || line.starts_with("⏰") || line.starts_with("⏱️") {
            // Warning/system message - warning color
            Line::from(Span::styled(
                line.clone(),
                Style::default().fg(theme.warning),
            ))
        } else if line.starts_with("📋") || line.starts_with("⏳") || line.starts_with("✅") {
            // System message - muted color
            Line::from(Span::styled(
                line.clone(),
                Style::default().fg(theme.text_muted),
            ))
        } else {
            // Default text color
            Line::from(Span::styled(
                line.clone(),
                Style::default().fg(theme.text),
            ))
        };
        styled_lines.push(styled_line);
        styled_lines.push(Line::from("")); // Add spacing between messages
    }

    let chat_widget = Paragraph::new(styled_lines)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .title(" Chat Log "),
        )
        .style(Style::default().fg(theme.text));

    frame.render_widget(chat_widget, area);
}

/// Renders the active agents panel
fn render_active_agents(
    frame: &mut Frame,
    area: Rect,
    active_agents: &[(String, String, String)],
    theme: &crate::theme::RadiumTheme,
) {
    if active_agents.is_empty() {
        let empty_text = "No active agents";
        let empty_widget = Paragraph::new(empty_text)
            .style(Style::default().fg(theme.text_muted))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(" Active Agents "),
            );
        frame.render_widget(empty_widget, area);
        return;
    }

    // Create table items
    let table_items: Vec<Vec<String>> = active_agents
        .iter()
        .map(|(id, name, status)| {
            vec![
                id.clone(),
                name.clone(),
                status.clone(),
            ]
        })
        .collect();

    let mut table = InteractiveTable::new(
        vec!["ID".to_string(), "Name".to_string(), "Status".to_string()],
        vec![
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ],
    );
    table.set_items(table_items);
    table.set_selected(Some(0));
    table.render(frame, area, Some(" Active Agents "));
}

