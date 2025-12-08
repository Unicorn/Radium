//! Unified prompt interface view.
//!
//! Single interface with command prompt at bottom and context display at top.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::commands::DisplayContext;
use crate::components::InteractiveTable;
use crate::icons::Icons;
use crate::setup::SetupWizard;
use crate::theme::THEME;
use ratatui::layout::Constraint;

/// Unified prompt data.
pub struct PromptData {
    /// Current display context
    pub context: DisplayContext,
    /// Input buffer for prompt
    pub input: String,
    /// Output/display lines
    pub output: Vec<String>,
    /// Chat conversation history (when in Chat context)
    pub conversation: Vec<String>,
    /// Available agents (when in AgentList context)
    pub agents: Vec<(String, String)>,
    /// Chat sessions (when in SessionList context)
    pub sessions: Vec<(String, usize)>,
    /// Selected index for lists
    pub selected_index: usize,
    /// Command suggestions for autocomplete
    pub command_suggestions: Vec<String>,
    /// Selected suggestion index for command menu navigation
    pub selected_suggestion_index: usize,
    /// Scrollback offset for conversation history
    pub scrollback_offset: usize,
    /// Command palette state
    pub command_palette_active: bool,
    pub command_palette_query: String,
}

impl PromptData {
    pub fn new() -> Self {
        Self {
            context: DisplayContext::Help,
            input: String::new(),
            output: vec![
                "Welcome to Radium TUI!".to_string(),
                "".to_string(),
                "Available commands:".to_string(),
                "  /chat <agent>   - Start chat with agent".to_string(),
                "  /agents         - List available agents".to_string(),
                "  /sessions       - Show chat sessions".to_string(),
                "  /dashboard      - Show dashboard stats".to_string(),
                "  /help           - Show this help".to_string(),
                "".to_string(),
                "Type a command to get started!".to_string(),
            ],
            conversation: Vec::new(),
            agents: Vec::new(),
            sessions: Vec::new(),
            selected_index: 0,
            command_suggestions: Vec::new(),
            selected_suggestion_index: 0,
            scrollback_offset: 0,
            command_palette_active: false,
            command_palette_query: String::new(),
        }
    }

    pub fn push_char(&mut self, c: char) {
        self.input.push(c);
    }

    pub fn pop_char(&mut self) {
        self.input.pop();
    }

    pub fn clear_input(&mut self) {
        self.input.clear();
    }

    pub fn add_output(&mut self, line: String) {
        self.output.push(line);
        // Keep only last 1000 lines
        if self.output.len() > 1000 {
            self.output.remove(0);
        }
    }

    pub fn clear_output(&mut self) {
        self.output.clear();
    }

    /// Add a message to conversation history with automatic limiting.
    /// 
    /// If the conversation exceeds the limit, oldest messages are removed.
    pub fn add_conversation_message(&mut self, message: String, max_history: usize) {
        self.conversation.push(message);
        // Keep only last max_history messages
        if self.conversation.len() > max_history {
            self.conversation.remove(0);
            // Adjust scrollback offset if needed
            if self.scrollback_offset > 0 {
                self.scrollback_offset = self.scrollback_offset.saturating_sub(1);
            }
        }
    }

    /// Get visible conversation lines for viewport culling.
    /// 
    /// Returns only the lines that should be rendered based on viewport size and scroll position.
    pub fn get_visible_conversation(&self, viewport_height: usize) -> Vec<String> {
        let total_lines = self.conversation.len();
        if total_lines == 0 {
            return Vec::new();
        }

        // Calculate visible range
        let start = self.scrollback_offset.min(total_lines.saturating_sub(1));
        let end = (start + viewport_height).min(total_lines);

        if start >= total_lines {
            return Vec::new();
        }

        self.conversation[start..end].to_vec()
    }
}

/// Render the unified prompt interface.
/// Note: Input prompt is now rendered in the status bar, not here.
pub fn render_prompt(frame: &mut Frame, area: Rect, data: &PromptData) {
    // Main display area - content depends on context
    let main_content = match &data.context {
        DisplayContext::Chat { agent_id, .. } => {
            // Show conversation history
            if data.conversation.is_empty() {
                format!("Chat with {}\n\nNo messages yet. Type a message to start!", agent_id)
            } else {
                // Join conversation lines (markdown will be rendered separately)
                data.conversation.join("\n\n")
            }
        }
        DisplayContext::AgentList => {
            // Show agent list
            if data.agents.is_empty() {
                "No agents found.\n\nPlace agent configs in ./agents/ or ~/.radium/agents/"
                    .to_string()
            } else {
                let mut output = String::from("Available Agents:\n\n");
                for (id, name) in &data.agents {
                    output.push_str(&format!("  {} - {}\n", id, name));
                }
                output.push_str("\nUse /chat <agent-id> to start chatting");
                output
            }
        }
        DisplayContext::SessionList => {
            // Session list will be rendered separately
            "".to_string()
        }
        DisplayContext::ModelSelector => {
            // Model selector will be rendered separately
            "".to_string()
        }
        DisplayContext::Dashboard | DisplayContext::Help => {
            // Show general output
            data.output.join("\n")
        }
    };

    // Render context-specific views
    match &data.context {
        DisplayContext::AgentList => {
            // Render agent list with interactive table
            if data.agents.is_empty() {
                let empty_text = "No agents found.\n\nPlace agent configs in ./agents/ or ~/.radium/agents/";
                let empty_widget = Paragraph::new(empty_text)
                    .wrap(Wrap { trim: true })
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(THEME.border()))
                            .title(" Available Agents "),
                    )
                    .style(Style::default().fg(THEME.text()));
                frame.render_widget(empty_widget, area);
            } else {
                let mut table = InteractiveTable::new(
                    vec!["ID".to_string(), "Name".to_string()],
                    vec![Constraint::Percentage(30), Constraint::Percentage(70)],
                );
                
                let items: Vec<Vec<String>> = data.agents
                    .iter()
                    .map(|(id, name)| vec![id.clone(), name.clone()])
                    .collect();
                
                table.set_items(items);
                table.set_selected(Some(data.selected_index.min(data.agents.len().saturating_sub(1))));
                table.render(frame, area, Some(" Available Agents "));
            }
        }
        DisplayContext::SessionList => {
            // Load and render sessions
            let workspace_root = None; // TODO: Get from app state
            if let Ok(session_manager) = crate::session_manager::SessionManager::new(workspace_root)
            {
                if let Ok(sessions_by_date) = session_manager.load_sessions() {
                    crate::views::sessions::render_sessions(
                        frame,
                        area,
                        &sessions_by_date,
                        data.selected_index,
                    );
                }
            }
        }
        DisplayContext::ModelSelector => {
            // Load and render models
            if let Ok(models) = crate::commands::models::get_available_models() {
                crate::views::model_selector::render_model_selector(
                    frame,
                    area,
                    &models,
                    data.selected_index,
                );
            }
        }
        DisplayContext::Chat { .. } => {
            // Render chat with markdown support and viewport culling
            // Note: scrollback_offset is used in get_visible_conversation()
            
            // Calculate viewport height (subtract borders and padding)
            let viewport_height = area.height.saturating_sub(2) as usize; // Subtract top/bottom borders
            
            // Get only visible conversation lines (viewport culling)
            let visible_conversation = data.get_visible_conversation(viewport_height);

            // Parse visible conversation lines as markdown
            let mut markdown_lines = Vec::new();
            for line in &visible_conversation {
                let parsed = crate::views::markdown::render_markdown(line);
                markdown_lines.extend(parsed);
                markdown_lines.push(ratatui::text::Line::from("")); // Add spacing between messages
            }

            let main_widget = Paragraph::new(markdown_lines)
                .wrap(Wrap { trim: true })
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(THEME.border()))
                )
                .style(Style::default().fg(THEME.text()))
                .scroll((0, 0)); // No scroll needed since we're already culling
            frame.render_widget(main_widget, area);
        }
        _ => {
            // Apply scrollback offset for other contexts
            let scroll_offset = data.scrollback_offset;

            let main_widget = Paragraph::new(main_content)
                .wrap(Wrap { trim: true })
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(THEME.border())),
                )
                .style(Style::default().fg(THEME.text()))
                .scroll((scroll_offset as u16, 0));
            frame.render_widget(main_widget, area);
        }
    }

    // Render command palette overlay if active
    if data.command_palette_active {
        render_command_palette(frame, area, data);
    }

    // Note: Input prompt is now rendered in the status bar (not here)

    // Show command menu popup if there are suggestions
    if !data.command_suggestions.is_empty() {
        const MAX_SUGGESTIONS_TO_SHOW: usize = 8;

        let total_suggestions = data.command_suggestions.len();
        let selected_idx = data.selected_suggestion_index;

        // Calculate visible range (centered on selection when possible)
        let (visible_start, visible_end) = if total_suggestions <= MAX_SUGGESTIONS_TO_SHOW {
            (0, total_suggestions)
        } else {
            // Try to center the selection in the visible window
            let half_window = MAX_SUGGESTIONS_TO_SHOW / 2;
            let start = selected_idx.saturating_sub(half_window);
            let end = (start + MAX_SUGGESTIONS_TO_SHOW).min(total_suggestions);
            // Adjust start if we hit the end
            let start = if end == total_suggestions {
                total_suggestions.saturating_sub(MAX_SUGGESTIONS_TO_SHOW)
            } else {
                start
            };
            (start, end)
        };

        let visible_count = visible_end - visible_start;
        let has_more_above = visible_start > 0;
        let has_more_below = visible_end < total_suggestions;

        // Create popup with dynamic height
        let popup_width = 60;
        let popup_height = (visible_count + 4) as u16; // suggestions + header + footer + spacing

        let popup_area = Rect {
            x: area.x + 2,
            y: area.y.saturating_sub(popup_height),
            width: popup_width.min(area.width.saturating_sub(4)),
            height: popup_height,
        };

        // Build styled menu items
        let title = if total_suggestions > MAX_SUGGESTIONS_TO_SHOW {
            format!(
                "{} Commands ({}/{})",
                Icons::INFO,
                selected_idx + 1,
                total_suggestions
            )
        } else {
            format!("{} Commands", Icons::INFO)
        };

        let mut menu_lines = vec![
            ratatui::text::Line::from(Span::styled(
                title,
                Style::default().fg(THEME.primary()).add_modifier(Modifier::BOLD),
            )),
            ratatui::text::Line::from(""),
        ];

        // Add scroll indicator at top if needed
        if has_more_above {
            menu_lines.push(ratatui::text::Line::from(Span::styled(
                "  ▲ More above...",
                Style::default().fg(THEME.text_dim()),
            )));
        }

        // Show visible suggestions
        for (i, suggestion) in data.command_suggestions.iter().enumerate().skip(visible_start).take(visible_count) {
            let is_selected = i == selected_idx;
            let style = if is_selected {
                Style::default().fg(THEME.bg_primary()).bg(THEME.primary()).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(THEME.text())
            };

            let prefix = if is_selected { "▶ " } else { "  " };
            menu_lines.push(ratatui::text::Line::from(Span::styled(
                format!("{}{}", prefix, suggestion),
                style,
            )));
        }

        // Add scroll indicator at bottom if needed
        if has_more_below {
            menu_lines.push(ratatui::text::Line::from(Span::styled(
                "  ▼ More below...",
                Style::default().fg(THEME.text_dim()),
            )));
        }

        menu_lines.push(ratatui::text::Line::from(""));
        menu_lines.push(ratatui::text::Line::from(Span::styled(
            "↑↓ Navigate | Tab/Enter Select | Esc Cancel",
            Style::default().fg(THEME.text_dim()),
        )));

        let menu_widget = Paragraph::new(menu_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(THEME.primary()))
                    .style(Style::default().bg(THEME.bg_panel())),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(menu_widget, popup_area);
    }
}

/// Render command palette overlay.
fn render_command_palette(frame: &mut Frame, area: Rect, data: &PromptData) {
    let popup_width = 70;
    let popup_height = (data.command_suggestions.len() + 6).min(20) as u16;

    let popup_area = Rect {
        x: (area.width.saturating_sub(popup_width)) / 2,
        y: area.height / 3,
        width: popup_width.min(area.width),
        height: popup_height.min(area.height.saturating_sub(area.height / 3)),
    };

    let mut lines = vec![
        Line::from(Span::styled(
            format!("{} Command Palette", Icons::SETTINGS),
            Style::default().fg(THEME.primary()).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("> {}", data.command_palette_query),
            Style::default().fg(THEME.text()),
        )),
        Line::from(""),
    ];

    if data.command_suggestions.is_empty() {
        lines.push(Line::from(Span::styled(
            "No matching commands",
            Style::default().fg(THEME.text_muted()),
        )));
    } else {
        for (i, suggestion) in data.command_suggestions.iter().enumerate() {
            let is_selected = i == data.selected_suggestion_index;
            let style = if is_selected {
                Style::default().fg(THEME.bg_primary()).bg(THEME.primary()).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(THEME.text())
            };

            let prefix = if is_selected { "▶ " } else { "  " };
            lines.push(Line::from(Span::styled(format!("{}{}", prefix, suggestion), style)));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "↑↓ Navigate | Enter Select | Esc Cancel",
        Style::default().fg(THEME.text_dim()),
    )));

    let palette = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(THEME.primary()))
                .style(Style::default().bg(THEME.bg_panel())),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(palette, popup_area);
}

/// Render the setup wizard interface.
pub fn render_setup_wizard(frame: &mut Frame, area: Rect, wizard: &SetupWizard) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Main wizard content
            Constraint::Length(2), // Help line
        ])
        .split(area);

    // Title with wizard state
    let title = Paragraph::new(format!("{} Radium - {}", Icons::ROCKET, wizard.title()))
        .style(Style::default().fg(THEME.primary()).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default().borders(Borders::ALL).border_style(Style::default().fg(THEME.border())),
        );
    frame.render_widget(title, chunks[0]);

    // Main wizard content - with styled lines for connection status
    let wizard_lines = wizard.display_lines();

    // Convert plain strings to styled Lines
    let styled_lines: Vec<ratatui::text::Line> = wizard_lines
        .iter()
        .map(|line| {
            if line.contains("✓ Connected") {
                // Split the line to apply green color to "✓ Connected" part
                let parts: Vec<&str> = line.split("✓ Connected").collect();
                if parts.len() == 2 {
                    ratatui::text::Line::from(vec![
                        Span::raw(parts[0]),
                        Span::styled("✓ Connected", Style::default().fg(THEME.success())),
                        Span::raw(parts[1]),
                    ])
                } else {
                    ratatui::text::Line::from(line.clone())
                }
            } else {
                ratatui::text::Line::from(line.clone())
            }
        })
        .collect();

    let main_widget = Paragraph::new(styled_lines)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(THEME.border_active()))
                .title(" Setup Wizard "),
        )
        .style(Style::default().fg(THEME.text()))
        .alignment(Alignment::Left);
    frame.render_widget(main_widget, chunks[1]);

    // Help line
    let help_text = "Follow the instructions above | Ctrl+C to quit";
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(THEME.text_muted()))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[2]);
}
