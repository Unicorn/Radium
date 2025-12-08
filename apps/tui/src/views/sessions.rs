//! Session history view.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
    layout::Constraint,
};

use crate::components::InteractiveTable;
use crate::icons::Icons;
use crate::session_manager::ChatSession;
use crate::theme::THEME;
use std::collections::HashMap;

/// Render the session history view.
pub fn render_sessions(
    frame: &mut Frame,
    area: Rect,
    sessions_by_date: &HashMap<String, Vec<ChatSession>>,
    selected_index: usize,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Session list
            Constraint::Length(2), // Help
        ])
        .split(area);

    // Title
    let title = Paragraph::new(format!("{} Recent Sessions", Icons::SESSION))
        .style(Style::default().fg(THEME.primary()).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default().borders(Borders::ALL).border_style(Style::default().fg(THEME.border())),
        );
    frame.render_widget(title, chunks[0]);

    // Build session table items
    let mut table_items = Vec::new();

    // Sort dates (most recent first)
    let mut sorted_dates: Vec<_> = sessions_by_date.keys().collect();
    sorted_dates.sort_by(|a, b| b.cmp(a));

    for date in sorted_dates {
        // Add sessions for this date
        for session in &sessions_by_date[date] {
            let date_label = format_date_label(date);
            table_items.push(vec![
                session.agent_id.clone(),
                session.session_id.clone(),
                format!("{}", session.message_count),
                date_label,
            ]);
        }
    }

    if table_items.is_empty() {
        let empty_text = "No sessions found. Use /chat <agent> to start a new session.";
        let empty_widget = Paragraph::new(empty_text)
            .style(Style::default().fg(THEME.text_muted()))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(THEME.border()))
                    .title(" Recent Sessions "),
            );
        frame.render_widget(empty_widget, chunks[1]);
    } else {
        let mut table = InteractiveTable::new(
            vec!["Agent".to_string(), "Session ID".to_string(), "Messages".to_string(), "Date".to_string()],
            vec![
                Constraint::Percentage(20),
                Constraint::Percentage(35),
                Constraint::Percentage(15),
                Constraint::Percentage(30),
            ],
        );
        let items_len = table_items.len();
        table.set_items(table_items);
        table.set_selected(Some(selected_index.min(items_len.saturating_sub(1))));
        table.render(frame, chunks[1], Some(" Recent Sessions "));
    }

    // Help line
    let help_text = "Press number to resume | 'd' to delete | '/' for commands";
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(THEME.text_muted()))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[2]);
}

fn format_date_label(date: &str) -> String {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let yesterday = (chrono::Utc::now() - chrono::Duration::days(1)).format("%Y-%m-%d").to_string();

    if date == &today {
        "Today".to_string()
    } else if date == &yesterday {
        "Yesterday".to_string()
    } else {
        // Try to format as readable date
        if let Ok(parsed) = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
            parsed.format("%B %d, %Y").to_string()
        } else {
            date.to_string()
        }
    }
}
