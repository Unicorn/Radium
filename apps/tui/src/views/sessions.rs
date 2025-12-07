//! Session history view.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

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
            Constraint::Length(3),  // Title
            Constraint::Min(10),    // Session list
            Constraint::Length(2),  // Help
        ])
        .split(area);

    // Title
    let title = Paragraph::new(format!("{} Recent Sessions", Icons::SESSION))
        .style(Style::default().fg(THEME.primary).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(THEME.border))
        );
    frame.render_widget(title, chunks[0]);

    // Build session list items
    let mut items = Vec::new();
    let mut current_index = 0;

    // Sort dates (most recent first)
    let mut sorted_dates: Vec<_> = sessions_by_date.keys().collect();
    sorted_dates.sort_by(|a, b| b.cmp(a));

    for date in sorted_dates {
        // Add date header
        let date_label = format_date_label(date);
        items.push(ListItem::new(Line::from(
            Span::styled(
                date_label,
                Style::default()
                    .fg(THEME.text_muted)
                    .add_modifier(Modifier::BOLD),
            )
        )));

        // Add sessions for this date
        for session in &sessions_by_date[date] {
            let is_selected = current_index == selected_index;
            let style = if is_selected {
                Style::default()
                    .fg(THEME.bg_primary)
                    .bg(THEME.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(THEME.text)
            };

            let session_text = format!(
                "  {} {} ({}) - {} messages",
                if is_selected { "▶" } else { " " },
                session.agent_id,
                session.session_id,
                session.message_count
            );

            items.push(ListItem::new(Line::from(Span::styled(session_text, style))));
            current_index += 1;
        }
    }

    if items.is_empty() {
        items.push(ListItem::new(Line::from(
            Span::styled(
                "No sessions found. Use /chat <agent> to start a new session.",
                Style::default().fg(THEME.text_muted),
            )
        )));
    }

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(THEME.border))
        )
        .style(Style::default().fg(THEME.text));

    frame.render_widget(list, chunks[1]);

    // Help line
    let help_text = "Press number to resume | 'd' to delete | '/' for commands";
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(THEME.text_muted))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[2]);
}

fn format_date_label(date: &str) -> String {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let yesterday = (chrono::Utc::now() - chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();

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

