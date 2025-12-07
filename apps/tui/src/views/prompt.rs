//! Unified prompt interface view.
//!
//! Single interface with command prompt at bottom and context display at top.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::commands::DisplayContext;
use crate::icons::Icons;
use crate::setup::SetupWizard;
use crate::theme::THEME;
use crate::views::header::{render_header, HeaderInfo};

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
}

/// Render the unified prompt interface.
pub fn render_prompt(frame: &mut Frame, area: Rect, data: &PromptData) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Header
            Constraint::Min(10),    // Main display area
            Constraint::Length(3),  // Input prompt
            Constraint::Length(2),  // Help line
        ])
        .split(area);

    // Branded header
    let header_info = HeaderInfo::new()
        .with_agent(
            match &data.context {
                DisplayContext::Chat { agent_id, .. } => Some(agent_id.clone()),
                _ => None,
            }
            .unwrap_or_default()
        )
        .with_session(
            match &data.context {
                DisplayContext::Chat { session_id, .. } => Some(session_id.clone()),
                _ => None,
            }
            .unwrap_or_default()
        );
    render_header(frame, chunks[0], &header_info);

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
                "No agents found.\n\nPlace agent configs in ./agents/ or ~/.radium/agents/".to_string()
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
        DisplayContext::SessionList => {
            // Load and render sessions
            let workspace_root = None; // TODO: Get from app state
            if let Ok(session_manager) = crate::session_manager::SessionManager::new(workspace_root) {
                if let Ok(sessions_by_date) = session_manager.load_sessions() {
                    crate::views::sessions::render_sessions(
                        frame,
                        chunks[1],
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
                    chunks[1],
                    &models,
                    data.selected_index,
                );
            }
        }
        DisplayContext::Chat { .. } => {
            // Render chat with markdown support
            let scroll_offset = data.scrollback_offset;
            
            // Parse conversation lines as markdown
            let mut markdown_lines = Vec::new();
            for line in &data.conversation {
                let parsed = crate::views::markdown::render_markdown(line);
                markdown_lines.extend(parsed);
                markdown_lines.push(ratatui::text::Line::from("")); // Add spacing between messages
            }

            let main_widget = Paragraph::new(markdown_lines)
                .wrap(Wrap { trim: true })
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(THEME.border)))
                .style(Style::default().fg(THEME.text))
                .scroll((scroll_offset as u16, 0));
            frame.render_widget(main_widget, chunks[1]);
        }
        _ => {
            // Apply scrollback offset for other contexts
            let scroll_offset = data.scrollback_offset;

            let main_widget = Paragraph::new(main_content)
                .wrap(Wrap { trim: true })
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(THEME.border)))
                .style(Style::default().fg(THEME.text))
                .scroll((scroll_offset as u16, 0));
            frame.render_widget(main_widget, chunks[1]);
        }
    }

    // Render command palette overlay if active
    if data.command_palette_active {
        render_command_palette(frame, area, data);
    }

    // Input prompt
    let prompt_text = format!("> {}_", data.input);
    let prompt = Paragraph::new(prompt_text)
        .style(Style::default().fg(THEME.primary))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(THEME.border_active))
            .title(" Input "));
    frame.render_widget(prompt, chunks[2]);

    // Show command menu popup if there are suggestions
    if !data.command_suggestions.is_empty() {
        // Create a centered popup for command menu
        let popup_width = 60;
        let popup_height = (data.command_suggestions.len() + 4).min(15) as u16;

        let popup_area = Rect {
            x: chunks[2].x + 2,
            y: chunks[2].y.saturating_sub(popup_height),
            width: popup_width.min(chunks[2].width.saturating_sub(4)),
            height: popup_height,
        };

        // Build styled menu items
        let mut menu_lines = vec![
            ratatui::text::Line::from(
                Span::styled(
                    format!("{} Commands", Icons::INFO),
                    Style::default().fg(THEME.primary).add_modifier(Modifier::BOLD)
                )
            ),
            ratatui::text::Line::from(""),
        ];

        for (i, suggestion) in data.command_suggestions.iter().enumerate() {
            let is_selected = i == data.selected_suggestion_index;
            let style = if is_selected {
                Style::default().fg(THEME.bg_primary).bg(THEME.primary).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(THEME.text)
            };

            let prefix = if is_selected { "▶ " } else { "  " };
            menu_lines.push(ratatui::text::Line::from(
                Span::styled(format!("{}{}", prefix, suggestion), style)
            ));
        }

        menu_lines.push(ratatui::text::Line::from(""));
        menu_lines.push(ratatui::text::Line::from(
            Span::styled("↑↓ Navigate | Tab/Enter Select | Esc Cancel", Style::default().fg(THEME.text_dim))
        ));

        let menu_widget = Paragraph::new(menu_lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(THEME.primary))
                .style(Style::default().bg(THEME.bg_panel)))
            .wrap(Wrap { trim: false });

        frame.render_widget(menu_widget, popup_area);
    }

    // Help line
    let help_text = "Type /help for commands | Ctrl+C to quit | Enter to send";
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(THEME.text_muted))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[3]);
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
        Line::from(
            Span::styled(
                format!("{} Command Palette", Icons::SETTINGS),
                Style::default()
                    .fg(THEME.primary)
                    .add_modifier(Modifier::BOLD),
            )
        ),
        Line::from(""),
        Line::from(
            Span::styled(
                format!("> {}", data.command_palette_query),
                Style::default().fg(THEME.text),
            )
        ),
        Line::from(""),
    ];

    if data.command_suggestions.is_empty() {
        lines.push(Line::from(
            Span::styled(
                "No matching commands",
                Style::default().fg(THEME.text_muted),
            )
        ));
    } else {
        for (i, suggestion) in data.command_suggestions.iter().enumerate() {
            let is_selected = i == data.selected_suggestion_index;
            let style = if is_selected {
                Style::default()
                    .fg(THEME.bg_primary)
                    .bg(THEME.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(THEME.text)
            };

            let prefix = if is_selected { "▶ " } else { "  " };
            lines.push(Line::from(Span::styled(
                format!("{}{}", prefix, suggestion),
                style,
            )));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(
        Span::styled(
            "↑↓ Navigate | Enter Select | Esc Cancel",
            Style::default().fg(THEME.text_dim),
        )
    ));

    let palette = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(THEME.primary))
                .style(Style::default().bg(THEME.bg_panel))
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(palette, popup_area);
}


/// Render the setup wizard interface.
pub fn render_setup_wizard(frame: &mut Frame, area: Rect, wizard: &SetupWizard) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(10),    // Main wizard content
            Constraint::Length(2),  // Help line
        ])
        .split(area);

    // Title with wizard state
    let title = Paragraph::new(format!("{} Radium - {}", Icons::ROCKET, wizard.title()))
        .style(Style::default().fg(THEME.primary).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(THEME.border)));
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
                        Span::styled("✓ Connected", Style::default().fg(THEME.success)),
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
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(THEME.border_active))
            .title(" Setup Wizard "))
        .style(Style::default().fg(THEME.text))
        .alignment(Alignment::Left);
    frame.render_widget(main_widget, chunks[1]);

    // Help line
    let help_text = "Follow the instructions above | Ctrl+C to quit";
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(THEME.text_muted))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[2]);
}
