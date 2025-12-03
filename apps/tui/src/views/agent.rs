//! Agent management view

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Row, Table},
};

use crate::app::AppState;
use radium_core::proto::{Agent, ListAgentsRequest};
use tonic::Request;

/// Agent view data
pub struct AgentViewData {
    pub agents: Vec<Agent>,
    pub selected_index: usize,
}

impl AgentViewData {
    pub async fn fetch(app_state: &AppState) -> anyhow::Result<Self> {
        let client_guard = app_state.client.lock().await;
        let mut client =
            client_guard.as_ref().ok_or_else(|| anyhow::anyhow!("Not connected"))?.clone();

        let request = Request::new(ListAgentsRequest {});
        let response = client.list_agents(request).await?;
        let agents = response.into_inner().agents;

        Ok(Self { agents, selected_index: 0 })
    }

    pub fn next_agent(&mut self) {
        if !self.agents.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.agents.len();
        }
    }

    pub fn previous_agent(&mut self) {
        if !self.agents.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.agents.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    pub fn selected_agent(&self) -> Option<&Agent> {
        self.agents.get(self.selected_index)
    }
}

/// Render the agent management view
pub fn render_agent_view(frame: &mut Frame, area: Rect, data: &AgentViewData) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(5),     // Agent list
            Constraint::Length(10), // Agent details
            Constraint::Length(3),  // Help
        ])
        .split(area);

    // Title
    let title = Paragraph::new("🤖 Agent Management")
        .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // Agent list
    let agent_rows: Vec<Row> = data
        .agents
        .iter()
        .enumerate()
        .map(|(idx, agent)| {
            let style = if idx == data.selected_index {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };
            Row::new(vec![agent.id.clone(), agent.name.clone(), agent.state.clone()]).style(style)
        })
        .collect();

    let agent_table = Table::new(
        agent_rows,
        [Constraint::Percentage(33), Constraint::Percentage(33), Constraint::Percentage(34)],
    )
    .block(Block::default().borders(Borders::ALL).title(" Agents "))
    .header(Row::new(vec!["ID", "Name", "State"]).style(Style::default().fg(Color::Yellow)));
    frame.render_widget(agent_table, chunks[1]);

    // Agent details
    if let Some(agent) = data.selected_agent() {
        let details = vec![
            Line::from(vec![
                Span::styled("ID: ", Style::default().fg(Color::Cyan)),
                Span::raw(&agent.id),
            ]),
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(Color::Cyan)),
                Span::raw(&agent.name),
            ]),
            Line::from(vec![
                Span::styled("Description: ", Style::default().fg(Color::Cyan)),
                Span::raw(&agent.description),
            ]),
            Line::from(vec![
                Span::styled("State: ", Style::default().fg(Color::Cyan)),
                Span::raw(&agent.state),
            ]),
        ];

        let details_para = Paragraph::new(details)
            .block(Block::default().borders(Borders::ALL).title(" Agent Details "))
            .wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(details_para, chunks[2]);
    } else {
        let no_agent = Paragraph::new("No agents available")
            .block(Block::default().borders(Borders::ALL).title(" Agent Details "));
        frame.render_widget(no_agent, chunks[2]);
    }

    // Help
    let help = Paragraph::new("  [↑↓] Navigate  [Enter] View Details  [1] Dashboard  [q] Quit")
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL).title(" Keys "));
    frame.render_widget(help, chunks[3]);
}
