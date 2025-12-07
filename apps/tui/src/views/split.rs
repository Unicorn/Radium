//! Split view for complex workflows.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::icons::Icons;
use crate::theme::THEME;

/// Split view state.
#[derive(Debug, Clone)]
pub struct SplitViewState {
    /// Left panel focus (true) or right panel (false)
    pub left_focused: bool,
    /// Left panel content (agent list)
    pub left_items: Vec<String>,
    /// Right panel content (chat)
    pub right_content: Vec<String>,
    /// Selected index in left panel
    pub selected_index: usize,
}

impl Default for SplitViewState {
    fn default() -> Self {
        Self {
            left_focused: true,
            left_items: Vec::new(),
            right_content: Vec::new(),
            selected_index: 0,
        }
    }
}

/// Render split view.
pub fn render_split_view(frame: &mut Frame, area: Rect, state: &SplitViewState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),  // Left panel
            Constraint::Percentage(70),  // Right panel
        ])
        .split(area);

    // Left panel - Agent list
    let left_title = if state.left_focused {
        format!("{} Agents [FOCUSED]", Icons::AGENT)
    } else {
        format!("{} Agents", Icons::AGENT)
    };

    let left_items: Vec<ListItem> = state
        .left_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_selected = i == state.selected_index && state.left_focused;
            let style = if is_selected {
                Style::default()
                    .fg(THEME.bg_primary)
                    .bg(THEME.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(THEME.text)
            };

            ListItem::new(Line::from(Span::styled(
                format!("  {}", item),
                style,
            )))
        })
        .collect();

    let left_list = List::new(left_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(if state.left_focused {
                    Style::default().fg(THEME.primary)
                } else {
                    Style::default().fg(THEME.border)
                })
                .title(left_title)
        )
        .style(Style::default().fg(THEME.text));

    frame.render_widget(left_list, chunks[0]);

    // Right panel - Chat
    let right_title = if !state.left_focused {
        "Chat [FOCUSED]"
    } else {
        "Chat"
    };

    let right_content = state.right_content.join("\n");
    let right_widget = Paragraph::new(right_content)
        .wrap(ratatui::widgets::Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(if !state.left_focused {
                    Style::default().fg(THEME.primary)
                } else {
                    Style::default().fg(THEME.border)
                })
                .title(right_title)
        )
        .style(Style::default().fg(THEME.text));

    frame.render_widget(right_widget, chunks[1]);
}

