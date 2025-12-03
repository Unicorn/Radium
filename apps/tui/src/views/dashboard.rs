//! Dashboard view

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Row, Table},
};

use crate::app::AppState;
use radium_core::proto::{ListAgentsRequest, ListTasksRequest, ListWorkflowsRequest};
use tonic::Request;

/// Dashboard data
pub struct DashboardData {
    pub agent_count: usize,
    pub workflow_count: usize,
    pub task_count: usize,
    pub recent_tasks: Vec<String>,
}

impl DashboardData {
    pub async fn fetch(app_state: &AppState) -> anyhow::Result<Self> {
        use tracing::debug;

        let client_guard = app_state.client.lock().await;
        let mut client = client_guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not connected to server"))?
            .clone();
        drop(client_guard); // Release lock early

        debug!("Fetching dashboard data...");

        // Fetch agents
        let agent_request = Request::new(ListAgentsRequest {});
        let agent_response = client
            .list_agents(agent_request)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch agents: {}", e))?;
        let agent_count = agent_response.into_inner().agents.len();
        debug!("Fetched {} agents", agent_count);

        // Fetch workflows
        let workflow_request = Request::new(ListWorkflowsRequest {});
        let workflow_response = client
            .list_workflows(workflow_request)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch workflows: {}", e))?;
        let workflow_count = workflow_response.into_inner().workflows.len();
        debug!("Fetched {} workflows", workflow_count);

        // Fetch tasks
        let task_request = Request::new(ListTasksRequest {});
        let task_response = client
            .list_tasks(task_request)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch tasks: {}", e))?;
        let tasks = task_response.into_inner().tasks;
        let task_count = tasks.len();
        let recent_tasks = tasks.iter().take(5).map(|t| format!("{} - {}", t.id, t.name)).collect();
        debug!("Fetched {} tasks", task_count);

        Ok(Self { agent_count, workflow_count, task_count, recent_tasks })
    }
}

/// Render the dashboard view
pub fn render_dashboard(
    frame: &mut Frame,
    area: Rect,
    _app_state: &AppState,
    data: &DashboardData,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Stats
            Constraint::Min(5),    // Content
            Constraint::Length(3), // Help
        ])
        .split(area);

    // Title
    let title = Paragraph::new("🔆 Radium Dashboard")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // Stats
    let stats_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(chunks[1]);

    let agent_stat = Paragraph::new(format!("Agents\n{}", data.agent_count))
        .style(Style::default().fg(Color::Green))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title(" Agents "));
    frame.render_widget(agent_stat, stats_chunks[0]);

    let workflow_stat = Paragraph::new(format!("Workflows\n{}", data.workflow_count))
        .style(Style::default().fg(Color::Blue))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title(" Workflows "));
    frame.render_widget(workflow_stat, stats_chunks[1]);

    let task_stat = Paragraph::new(format!("Tasks\n{}", data.task_count))
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title(" Tasks "));
    frame.render_widget(task_stat, stats_chunks[2]);

    // Recent tasks
    let task_rows: Vec<Row> =
        data.recent_tasks.iter().map(|task| Row::new(vec![task.clone()])).collect();

    let task_table = Table::new(task_rows, [Constraint::Percentage(100)])
        .block(Block::default().borders(Borders::ALL).title(" Recent Tasks "))
        .header(Row::new(vec!["Task"]).style(Style::default().fg(Color::Yellow)));
    frame.render_widget(task_table, chunks[2]);

    // Help
    let help = Paragraph::new("  [1] Dashboard  [2] Agents  [3] Workflows  [4] Tasks  [q] Quit")
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL).title(" Navigation "));
    frame.render_widget(help, chunks[3]);
}
