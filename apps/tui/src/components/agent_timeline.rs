//! Agent timeline component for displaying agent execution progress.

use crate::state::{AgentState, AgentStatus, SubAgentState};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::collections::HashSet;

/// Agent timeline component
pub struct AgentTimeline;

impl AgentTimeline {
    /// Renders the agent timeline with hierarchical display and expandable sub-agents.
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        agents: &[AgentState],
        selected_index: Option<usize>,
    ) {
        Self::render_with_expansion(frame, area, agents, selected_index, &HashSet::new())
    }

    /// Renders the agent timeline with expansion support.
    pub fn render_with_expansion(
        frame: &mut Frame,
        area: Rect,
        agents: &[AgentState],
        selected_index: Option<usize>,
        expanded_agents: &HashSet<String>,
    ) {
        let theme = crate::theme::get_theme();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Title
                Constraint::Min(5),    // Agent list
            ])
            .split(area);

        // Title
        let title = Paragraph::new("Workflow Pipeline")
            .style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_panel)),
            );
        frame.render_widget(title, chunks[0]);

        // Build hierarchical list items
        let mut items: Vec<ListItem> = Vec::new();
        let mut _item_index = 0;

        for (agent_idx, agent) in agents.iter().enumerate() {
            let is_selected = Some(agent_idx) == selected_index;
            let is_expanded = expanded_agents.contains(&agent.agent_id);
            let has_sub_agents = !agent.sub_agents.is_empty();

            // Main agent node
            let status_color = Self::status_color(agent.status, &theme);
            let elapsed = agent
                .elapsed_time()
                .map(|d| format!("{:.1}s", d.as_secs_f64()))
                .unwrap_or_else(|| "-".to_string());

            let tool_info = if let Some(ref tool) = agent.current_tool {
                format!(" [{}]", tool)
            } else {
                String::new()
            };

            let expand_indicator = if has_sub_agents {
                if is_expanded { "▼ " } else { "▶ " }
            } else {
                "  "
            };

            let content = format!(
                "{} {} {} - {} ({}){}",
                expand_indicator,
                agent.status.icon(),
                agent.agent_name,
                agent.status.as_str(),
                elapsed,
                tool_info
            );

            let style = if is_selected {
                Style::default()
                    .fg(theme.bg_primary)
                    .bg(theme.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(status_color)
            };

            items.push(ListItem::new(content).style(style));
            _item_index += 1;

            // Sub-agents (if expanded)
            if is_expanded && has_sub_agents {
                let sub_agents: Vec<&SubAgentState> = agent.sub_agents.values().collect();
                for sub_agent in sub_agents {
                    let sub_status_color = Self::status_color(sub_agent.status, &theme);
                    let sub_elapsed = sub_agent
                        .elapsed_time()
                        .map(|d| format!("{:.1}s", d.as_secs_f64()))
                        .unwrap_or_else(|| "-".to_string());

                    let sub_content = format!(
                        "    {} {} - {} ({})",
                        sub_agent.status.icon(),
                        sub_agent.agent_name,
                        sub_agent.status.as_str(),
                        sub_elapsed
                    );

                    items.push(ListItem::new(sub_content).style(Style::default().fg(sub_status_color)));
                    _item_index += 1;
                }

                // Sub-agent summary if collapsed
                if !is_expanded && has_sub_agents {
                    let sub_count = agent.sub_agents.len();
                    let completed = agent
                        .sub_agents
                        .values()
                        .filter(|s| s.status == AgentStatus::Completed)
                        .count();
                    let summary = format!(
                        "    {} sub-agent{} ({}/{} completed)",
                        sub_count,
                        if sub_count == 1 { "" } else { "s" },
                        completed,
                        sub_count
                    );
                    items.push(ListItem::new(summary).style(Style::default().fg(theme.text_muted)));
                    _item_index += 1;
                }
            }
        }

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .style(Style::default().bg(theme.bg_panel))
                    .title(" Agents "),
            );
        frame.render_widget(list, chunks[1]);
    }

    /// Returns the color for an agent status.
    fn status_color(status: AgentStatus, theme: &crate::theme::RadiumTheme) -> Color {
        match status {
            AgentStatus::Idle => theme.text_muted,
            AgentStatus::Starting => theme.warning,
            AgentStatus::Running => theme.info,
            AgentStatus::Thinking => theme.primary,
            AgentStatus::ExecutingTool => theme.secondary,
            AgentStatus::Completed => theme.success,
            AgentStatus::Failed => theme.error,
            AgentStatus::Cancelled => theme.text_dim,
        }
    }

    /// Renders agent details (expanded view).
    pub fn render_details(frame: &mut Frame, area: Rect, agent: &AgentState) {
        let theme = crate::theme::get_theme();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(6), // Info
                Constraint::Min(5),    // Output/sub-agents
            ])
            .split(area);

        // Title
        let title = format!("Agent: {}", agent.agent_name);
        let title_widget = Paragraph::new(title)
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title_widget, chunks[0]);

        // Info
        let status_color = Self::status_color(agent.status, &theme);
        let elapsed = agent
            .elapsed_time()
            .map(|d| format!("{:.1}s", d.as_secs_f64()))
            .unwrap_or_else(|| "-".to_string());

        let info_text = vec![
            format!("Status: {} {}", agent.status.icon(), agent.status.as_str()),
            format!("ID: {}", agent.agent_id),
            format!("Elapsed: {}", elapsed),
            format!("Tokens: {}", format_tokens(agent.tokens_used)),
            format!("Cost: ${:.4}", agent.cost),
        ];

        let info = Paragraph::new(info_text.join("\n"))
            .style(Style::default().fg(status_color))
            .block(Block::default().borders(Borders::ALL).title(" Details "));
        frame.render_widget(info, chunks[1]);

        // Sub-agents or output
        if !agent.sub_agents.is_empty() {
            let sub_agent_items: Vec<ListItem> = agent
                .sub_agents
                .values()
                .map(|sub_agent| {
                    let status_color = Self::status_color(sub_agent.status, &theme);
                    let content = format!(
                        "  {} {} - {}",
                        sub_agent.status.icon(),
                        sub_agent.agent_name,
                        sub_agent.status.as_str()
                    );
                    ListItem::new(content).style(Style::default().fg(status_color))
                })
                .collect();

            let sub_agents = List::new(sub_agent_items)
                .block(Block::default().borders(Borders::ALL).title(" Sub-Agents "));
            frame.render_widget(sub_agents, chunks[2]);
        } else {
            let output_lines =
                agent.output_buffer.lines.iter().take(10).cloned().collect::<Vec<_>>();
            let output_text = output_lines.join("\n");

            let output = Paragraph::new(output_text)
                .style(Style::default())
                .block(Block::default().borders(Borders::ALL).title(" Recent Output "));
            frame.render_widget(output, chunks[2]);
        }
    }
}

/// Formats token count with commas.
fn format_tokens(tokens: u64) -> String {
    let s = tokens.to_string();
    let mut result = String::new();
    let mut count = 0;

    for c in s.chars().rev() {
        if count > 0 && count % 3 == 0 {
            result.push(',');
        }
        result.push(c);
        count += 1;
    }

    result.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_tokens() {
        assert_eq!(format_tokens(0), "0");
        assert_eq!(format_tokens(999), "999");
        assert_eq!(format_tokens(1000), "1,000");
        assert_eq!(format_tokens(1234567), "1,234,567");
    }

    #[test]
    fn test_status_color() {
        let theme = crate::theme::RadiumTheme::dark();
        assert_eq!(AgentTimeline::status_color(AgentStatus::Running, &theme), theme.info);
        assert_eq!(AgentTimeline::status_color(AgentStatus::Completed, &theme), theme.success);
        assert_eq!(AgentTimeline::status_color(AgentStatus::Failed, &theme), theme.error);
    }
}
