//! Orchestrator view with split-panel layout.
//!
//! Shows chat log (60%), task list (20%), and orchestrator thinking (20%) when orchestrator is running.
//! Responsive layout: stacks vertically on narrow terminals.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
    layout::Constraint,
};

use crate::components::{InteractiveTable, TaskListPanel, OrchestratorThinkingPanel};
use crate::state::TaskListState;
use crate::views::prompt::PromptData;

/// Panel focus for keyboard navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelFocus {
    /// Chat panel is focused
    Chat,
    /// Task list panel is focused
    TaskList,
    /// Orchestrator thinking panel is focused
    Orchestrator,
}

/// Renders the orchestrator view with three-panel split layout
pub fn render_orchestrator_view(
    frame: &mut Frame,
    area: Rect,
    prompt_data: &PromptData,
    active_agents: &[(String, String, String)], // (agent_id, agent_name, status)
    task_state: Option<&TaskListState>,
    orchestrator_panel: &mut OrchestratorThinkingPanel,
    panel_visibility: (bool, bool), // (task_panel_visible, orchestrator_panel_visible)
    focused_panel: PanelFocus,
) {
    let theme = crate::theme::get_theme();
    let (task_panel_visible, orchestrator_panel_visible) = panel_visibility;

    // Determine layout based on terminal width
    if area.width >= 100 {
        // Wide terminal: 60/20/20 horizontal split
        render_wide_layout(
            frame,
            area,
            prompt_data,
            active_agents,
            task_state,
            orchestrator_panel,
            task_panel_visible,
            orchestrator_panel_visible,
            focused_panel,
            &theme,
        );
    } else if area.width >= 60 {
        // Narrow terminal: vertical stack (chat 60% top, task/orchestrator 40% bottom split)
        render_narrow_layout(
            frame,
            area,
            prompt_data,
            active_agents,
            task_state,
            orchestrator_panel,
            task_panel_visible,
            orchestrator_panel_visible,
            focused_panel,
            &theme,
        );
    } else {
        // Very narrow terminal: chat only with toggle indicators
        render_very_narrow_layout(
            frame,
            area,
            prompt_data,
            task_panel_visible,
            orchestrator_panel_visible,
            &theme,
        );
    }
}

/// Renders wide layout (≥100 cols): 60/20/20 horizontal split
fn render_wide_layout(
    frame: &mut Frame,
    area: Rect,
    prompt_data: &PromptData,
    _active_agents: &[(String, String, String)],
    task_state: Option<&TaskListState>,
    orchestrator_panel: &mut OrchestratorThinkingPanel,
    task_panel_visible: bool,
    orchestrator_panel_visible: bool,
    focused_panel: PanelFocus,
    theme: &crate::theme::RadiumTheme,
) {
    // Calculate constraints based on panel visibility
    let constraints = if task_panel_visible && orchestrator_panel_visible {
        vec![
            Constraint::Percentage(60), // Chat
            Constraint::Percentage(20),  // Task list
            Constraint::Percentage(20),  // Orchestrator thinking
        ]
    } else if task_panel_visible {
        vec![
            Constraint::Percentage(75), // Chat
            Constraint::Percentage(25), // Task list
        ]
    } else if orchestrator_panel_visible {
        vec![
            Constraint::Percentage(75), // Chat
            Constraint::Percentage(25), // Orchestrator thinking
        ]
    } else {
        vec![Constraint::Percentage(100)] // Chat only
    };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area);

    let mut chunk_idx = 0;

    // Always render chat log
    render_chat_log(
        frame,
        chunks[chunk_idx],
        prompt_data,
        theme,
        focused_panel == PanelFocus::Chat,
    );
    chunk_idx += 1;

    // Render task list if visible
    if task_panel_visible {
        if let Some(ref task_state) = task_state {
            let mut task_panel = TaskListPanel::new();
            task_panel.render(
                frame,
                chunks[chunk_idx],
                task_state,
                focused_panel == PanelFocus::TaskList,
            );
        } else {
            render_empty_panel(frame, chunks[chunk_idx], "Task List", "No active workflow", theme);
        }
        chunk_idx += 1;
    }

    // Render orchestrator thinking if visible
    if orchestrator_panel_visible {
        orchestrator_panel.render(
            frame,
            chunks[chunk_idx],
            focused_panel == PanelFocus::Orchestrator,
        );
    }
}

/// Renders narrow layout (60-99 cols): vertical stack
fn render_narrow_layout(
    frame: &mut Frame,
    area: Rect,
    prompt_data: &PromptData,
    _active_agents: &[(String, String, String)],
    task_state: Option<&TaskListState>,
    orchestrator_panel: &mut OrchestratorThinkingPanel,
    task_panel_visible: bool,
    orchestrator_panel_visible: bool,
    focused_panel: PanelFocus,
    theme: &crate::theme::RadiumTheme,
) {
    // Split vertically: chat 60% top, task/orchestrator 40% bottom
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60), // Chat top
            Constraint::Percentage(40), // Task/orchestrator bottom
        ])
        .split(area);

    // Render chat log in top area
    render_chat_log(
        frame,
        vertical_chunks[0],
        prompt_data,
        theme,
        focused_panel == PanelFocus::Chat,
    );

    // Split bottom area horizontally for task/orchestrator
    if task_panel_visible && orchestrator_panel_visible {
        let bottom_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // Task list
                Constraint::Percentage(50), // Orchestrator thinking
            ])
            .split(vertical_chunks[1]);

        if let Some(ref task_state) = task_state {
            let mut task_panel = TaskListPanel::new();
            task_panel.render(
                frame,
                bottom_chunks[0],
                task_state,
                focused_panel == PanelFocus::TaskList,
            );
        } else {
            render_empty_panel(frame, bottom_chunks[0], "Task List", "No active workflow", theme);
        }

        orchestrator_panel.render(
            frame,
            bottom_chunks[1],
            focused_panel == PanelFocus::Orchestrator,
        );
    } else if task_panel_visible {
        if let Some(ref task_state) = task_state {
            let mut task_panel = TaskListPanel::new();
            task_panel.render(
                frame,
                vertical_chunks[1],
                task_state,
                focused_panel == PanelFocus::TaskList,
            );
        } else {
            render_empty_panel(frame, vertical_chunks[1], "Task List", "No active workflow", theme);
        }
    } else if orchestrator_panel_visible {
        orchestrator_panel.render(
            frame,
            vertical_chunks[1],
            focused_panel == PanelFocus::Orchestrator,
        );
    }
}

/// Renders very narrow layout (<60 cols): chat only with toggle indicators
fn render_very_narrow_layout(
    frame: &mut Frame,
    area: Rect,
    prompt_data: &PromptData,
    task_panel_visible: bool,
    orchestrator_panel_visible: bool,
    theme: &crate::theme::RadiumTheme,
) {
    // Render chat with toggle indicators in title
    let mut title = " Chat Log ".to_string();
    if !task_panel_visible || !orchestrator_panel_visible {
        title.push_str(" [Ctrl+T: Tasks]");
    }
    if !orchestrator_panel_visible {
        title.push_str(" [Ctrl+O: Orchestrator]");
    }

    let viewport_height = area.height.saturating_sub(4) as usize; // Account for padding
    let visible_conversation = prompt_data.get_visible_conversation(viewport_height);

    let mut styled_lines = Vec::new();
    for line in &visible_conversation {
        if line.starts_with("You: ") {
            // User message - use primary color with box drawing character
            let content = line.strip_prefix("You: ").unwrap_or(line);
            styled_lines.push(Line::from(vec![
                Span::styled("┌─ ", Style::default().fg(theme.primary)),
                Span::styled("You", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            ]));
            styled_lines.push(Line::from(vec![
                Span::styled("│ ", Style::default().fg(theme.primary)),
                Span::styled(content, Style::default().fg(theme.text)),
            ]));
            styled_lines.push(Line::from(
                Span::styled("└─", Style::default().fg(theme.primary))
            ));
        } else if line.starts_with("Agent: ") || line.starts_with("Assistant: ") {
            // AI message - use info/secondary color with different box drawing
            let prefix = if line.starts_with("Agent: ") { "Agent: " } else { "Assistant: " };
            let content = line.strip_prefix(prefix).unwrap_or(line);
            styled_lines.push(Line::from(vec![
                Span::styled("╭─ ", Style::default().fg(theme.info)),
                Span::styled(prefix.trim_end_matches(": "), Style::default().fg(theme.info).add_modifier(Modifier::BOLD)),
            ]));
            styled_lines.push(Line::from(vec![
                Span::styled("│ ", Style::default().fg(theme.info)),
                Span::styled(content, Style::default().fg(theme.text)),
            ]));
            styled_lines.push(Line::from(
                Span::styled("╰─", Style::default().fg(theme.info))
            ));
        } else if line.starts_with("Error: ") || line.starts_with("❌") {
            styled_lines.push(Line::from(Span::styled(line.clone(), Style::default().fg(theme.error))));
        } else if line.starts_with("⚠️") || line.starts_with("⏰") || line.starts_with("⏱️") {
            styled_lines.push(Line::from(Span::styled(line.clone(), Style::default().fg(theme.warning))));
        } else if line.starts_with("📋") || line.starts_with("⏳") || line.starts_with("✅") {
            styled_lines.push(Line::from(Span::styled(line.clone(), Style::default().fg(theme.text_muted))));
        } else {
            styled_lines.push(Line::from(Span::styled(line.clone(), Style::default().fg(theme.text))));
        }
        styled_lines.push(Line::from(""));
    }

    let chat_widget = Paragraph::new(styled_lines)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .title(title)
                .padding(ratatui::widgets::Padding::new(1, 1, 1, 1)),
        )
        .style(Style::default().fg(theme.text));

    frame.render_widget(chat_widget, area);
}

/// Renders the chat log panel
fn render_chat_log(
    frame: &mut Frame,
    area: Rect,
    prompt_data: &PromptData,
    theme: &crate::theme::RadiumTheme,
    focused: bool,
) {
    // Calculate viewport height (account for padding)
    let viewport_height = area.height.saturating_sub(4) as usize; // Extra space for padding

    // Get visible conversation lines
    let visible_conversation = prompt_data.get_visible_conversation(viewport_height);

    // Parse and color-code messages with visual distinction
    let mut styled_lines = Vec::new();
    for line in &visible_conversation {
        if line.starts_with("You: ") {
            // User message - use primary color with box drawing character
            let content = line.strip_prefix("You: ").unwrap_or(line);
            styled_lines.push(Line::from(vec![
                Span::styled("┌─ ", Style::default().fg(theme.primary)),
                Span::styled("You", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            ]));
            styled_lines.push(Line::from(vec![
                Span::styled("│ ", Style::default().fg(theme.primary)),
                Span::styled(content, Style::default().fg(theme.text)),
            ]));
            styled_lines.push(Line::from(
                Span::styled("└─", Style::default().fg(theme.primary))
            ));
        } else if line.starts_with("Agent: ") || line.starts_with("Assistant: ") {
            // AI message - use info/secondary color with different box drawing
            let prefix = if line.starts_with("Agent: ") { "Agent: " } else { "Assistant: " };
            let content = line.strip_prefix(prefix).unwrap_or(line);
            styled_lines.push(Line::from(vec![
                Span::styled("╭─ ", Style::default().fg(theme.info)),
                Span::styled(prefix.trim_end_matches(": "), Style::default().fg(theme.info).add_modifier(Modifier::BOLD)),
            ]));
            styled_lines.push(Line::from(vec![
                Span::styled("│ ", Style::default().fg(theme.info)),
                Span::styled(content, Style::default().fg(theme.text)),
            ]));
            styled_lines.push(Line::from(
                Span::styled("╰─", Style::default().fg(theme.info))
            ));
        } else if line.starts_with("Error: ") || line.starts_with("❌") {
            // Error message - error color
            let _ = Line::from(Span::styled(
                line.clone(),
                Style::default().fg(theme.error),
            ));
            styled_lines.push(Line::from(Span::styled(line.clone(), Style::default().fg(theme.error))));
        } else if line.starts_with("⚠️") || line.starts_with("⏰") || line.starts_with("⏱️") {
            // Warning/system message - warning color
            styled_lines.push(Line::from(Span::styled(line.clone(), Style::default().fg(theme.warning))));
        } else if line.starts_with("📋") || line.starts_with("⏳") || line.starts_with("✅") {
            // System message - muted color
            styled_lines.push(Line::from(Span::styled(line.clone(), Style::default().fg(theme.text_muted))));
        } else {
            // Default text color
            styled_lines.push(Line::from(Span::styled(line.clone(), Style::default().fg(theme.text))));
        }
        styled_lines.push(Line::from("")); // Add spacing between messages
    }

    let chat_widget = Paragraph::new(styled_lines)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(if focused {
                    Style::default().fg(theme.border_active)
                } else {
                    Style::default().fg(theme.border)
                })
                .title(" Chat Log ")
                .padding(ratatui::widgets::Padding::new(1, 1, 1, 1)),
        )
        .style(Style::default().fg(theme.text));

    frame.render_widget(chat_widget, area);
}

/// Renders an empty panel with a message
fn render_empty_panel(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    message: &str,
    theme: &crate::theme::RadiumTheme,
) {
    let empty_widget = Paragraph::new(message)
        .style(Style::default().fg(theme.text_muted))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .title(format!(" {} ", title)),
        );
    frame.render_widget(empty_widget, area);
}

/// Renders the active agents panel (kept for backward compatibility, but not used in new layout)
#[allow(dead_code)]
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
