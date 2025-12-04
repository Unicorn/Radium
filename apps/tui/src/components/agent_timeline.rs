//! Agent timeline component for displaying agent execution progress.

use crate::state::{AgentState, AgentStatus};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

/// Agent timeline component
pub struct AgentTimeline;

impl AgentTimeline {
    /// Renders the agent timeline.
    pub fn render(frame: &mut Frame, area: Rect, agents: &[AgentState], selected_index: Option<usize>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Min(5),     // Agent list
            ])
            .split(area);

        // Title
        let title = Paragraph::new("Agent Timeline")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, chunks[0]);

        // Agent list
        let items: Vec<ListItem> = agents
            .iter()
            .enumerate()
            .map(|(idx, agent)| {
                let status_color = Self::status_color(agent.status);
                let elapsed = agent
                    .elapsed_time()
                    .map(|d| format!("{:.1}s", d.as_secs_f64()))
                    .unwrap_or_else(|| "-".to_string());

                let tool_info = if let Some(ref tool) = agent.current_tool {
                    format!(" [{}]", tool)
                } else {
                    String::new()
                };

                let content = format!(
                    "{} {} - {} ({}){} {} tokens",
                    agent.status.icon(),
                    agent.agent_name,
                    agent.status.as_str(),
                    elapsed,
                    tool_info,
                    format_tokens(agent.tokens_used)
                );

                let style = if Some(idx) == selected_index {
                    Style::default().fg(status_color).add_modifier(Modifier::BOLD | Modifier::REVERSED)
                } else {
                    Style::default().fg(status_color)
                };

                ListItem::new(content).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(" Agents "));
        frame.render_widget(list, chunks[1]);
    }

    /// Returns the color for an agent status.
    fn status_color(status: AgentStatus) -> Color {
        match status {
            AgentStatus::Idle => Color::Gray,
            AgentStatus::Starting => Color::Yellow,
            AgentStatus::Running => Color::Blue,
            AgentStatus::Thinking => Color::Cyan,
            AgentStatus::ExecutingTool => Color::Magenta,
            AgentStatus::Completed => Color::Green,
            AgentStatus::Failed => Color::Red,
            AgentStatus::Cancelled => Color::DarkGray,
        }
    }

    /// Renders agent details (expanded view).
    pub fn render_details(frame: &mut Frame, area: Rect, agent: &AgentState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Length(6),  // Info
                Constraint::Min(5),     // Output/sub-agents
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
        let status_color = Self::status_color(agent.status);
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
                    let status_color = Self::status_color(sub_agent.status);
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
            let output_lines = agent.output_buffer.lines.iter().take(10).cloned().collect::<Vec<_>>();
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
        assert_eq!(AgentTimeline::status_color(AgentStatus::Running), Color::Blue);
        assert_eq!(AgentTimeline::status_color(AgentStatus::Completed), Color::Green);
        assert_eq!(AgentTimeline::status_color(AgentStatus::Failed), Color::Red);
    }
}
