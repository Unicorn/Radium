//! Session history view.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    layout::Constraint,
};

use crate::components::InteractiveTable;
use crate::icons::Icons;
use crate::session_manager::ChatSession;
use crate::state::SessionHistory;
use crate::theme::THEME;
use std::collections::HashMap;
use std::path::Path;

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

/// Render session history with previews (using new SessionHistory).
pub fn render_session_history_with_preview(
    frame: &mut Frame,
    area: Rect,
    workspace_root: &Path,
    selected_index: usize,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(10),    // Session list
            Constraint::Length(3),  // Help
        ])
        .split(area);

    // Title
    let title = Paragraph::new(format!("{} Session History", Icons::SESSION))
        .style(Style::default().fg(THEME.primary()).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(THEME.border())),
        );
    frame.render_widget(title, chunks[0]);

    // Load available sessions
    let session_ids = match SessionHistory::list_sessions(workspace_root) {
        Ok(ids) => ids,
        Err(_) => Vec::new(),
    };

    if session_ids.is_empty() {
        let empty_text = "No saved sessions found.\n\nStart chatting to create a new session.";
        let empty_widget = Paragraph::new(empty_text)
            .style(Style::default().fg(THEME.text_muted()))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(THEME.border()))
                    .title(" Session History "),
            );
        frame.render_widget(empty_widget, chunks[1]);
    } else {
        // Load session metadata for display
        let mut sessions_with_metadata: Vec<(String, SessionHistory)> = session_ids
            .iter()
            .filter_map(|id| {
                SessionHistory::load_from_disk(id, workspace_root)
                    .ok()
                    .map(|history| (id.clone(), history))
            })
            .collect();

        // Sort by last_active (most recent first)
        sessions_with_metadata.sort_by(|a, b| b.1.last_active.cmp(&a.1.last_active));

        // Build list items with previews
        let list_items: Vec<ListItem> = sessions_with_metadata
            .iter()
            .enumerate()
            .map(|(i, (_session_id, history))| {
                let is_selected = i == selected_index;

                // Format timestamp
                let time_label = format_timestamp(&history.last_active);

                // Get preview text
                let preview = history.preview();

                // Build session line
                let agent_style = if is_selected {
                    Style::default().fg(THEME.primary()).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(THEME.primary())
                };

                let preview_style = if is_selected {
                    Style::default().fg(THEME.text()).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(THEME.text_muted())
                };

                let time_style = if is_selected {
                    Style::default().fg(THEME.secondary())
                } else {
                    Style::default().fg(THEME.text_muted())
                };

                let lines = vec![
                    Line::from(vec![
                        Span::styled(
                            format!("{} ", if is_selected { "▶" } else { " " }),
                            Style::default().fg(THEME.primary()),
                        ),
                        Span::styled(format!("{} ", Icons::AGENT), agent_style),
                        Span::styled(&history.agent_id, agent_style),
                        Span::raw("  "),
                        Span::styled(
                            format!("({} messages)", history.message_count()),
                            time_style,
                        ),
                        Span::raw("  "),
                        Span::styled(time_label, time_style),
                    ]),
                    Line::from(vec![
                        Span::raw("  "),
                        Span::styled(preview, preview_style),
                    ]),
                    Line::from(""), // Spacer
                ];

                ListItem::new(lines)
            })
            .collect();

        let list = List::new(list_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(THEME.border()))
                    .title(" Session History "),
            )
            .highlight_style(
                Style::default()
                    .bg(THEME.bg_element())
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_widget(list, chunks[1]);
    }

    // Help line
    let help_text = vec![
        Line::from(vec![
            Span::styled("↑/↓", Style::default().fg(THEME.primary()).add_modifier(Modifier::BOLD)),
            Span::raw(" Navigate  "),
            Span::styled("Enter", Style::default().fg(THEME.primary()).add_modifier(Modifier::BOLD)),
            Span::raw(" Resume  "),
            Span::styled("d", Style::default().fg(THEME.primary()).add_modifier(Modifier::BOLD)),
            Span::raw(" Delete  "),
            Span::styled("Esc", Style::default().fg(THEME.primary()).add_modifier(Modifier::BOLD)),
            Span::raw(" Back"),
        ]),
    ];
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(THEME.text()))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[2]);
}

/// Format a SystemTime as a relative or absolute timestamp.
fn format_timestamp(time: &std::time::SystemTime) -> String {
    use std::time::SystemTime;

    let now = SystemTime::now();
    let duration = now.duration_since(*time).unwrap_or_default();

    let secs = duration.as_secs();
    if secs < 60 {
        "Just now".to_string()
    } else if secs < 3600 {
        let mins = secs / 60;
        format!("{} min{} ago", mins, if mins == 1 { "" } else { "s" })
    } else if secs < 86400 {
        let hours = secs / 3600;
        format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
    } else if secs < 604800 {
        let days = secs / 86400;
        format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
    } else {
        // Format as date
        match time.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(duration) => {
                let datetime = chrono::DateTime::<chrono::Utc>::from(
                    chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0)
                        .unwrap_or_default()
                );
                datetime.format("%b %d, %Y").to_string()
            }
            Err(_) => "Unknown".to_string(),
        }
    }
}
