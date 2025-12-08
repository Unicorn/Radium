//! Unified prompt interface view.
//!
//! Single interface with command prompt at bottom and context display at top.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};
use crate::components::textarea::TextArea;

use crate::commands::DisplayContext;
use crate::components::InteractiveTable;
use crate::icons::Icons;
use crate::setup::SetupWizard;
use crate::state::{CommandSuggestionState, SuggestionSource};
use crate::theme::THEME;
use ratatui::layout::Constraint;
use tachyonfx::{EffectTimer, Interpolation};

fn create_table_timer(duration_ms: u64) -> EffectTimer {
    EffectTimer::from_ms(duration_ms as u32, Interpolation::QuadOut)
}

/// Unified prompt data.
pub struct PromptData {
    /// Current display context
    pub context: DisplayContext,
    /// Input buffer for prompt (multiline textarea)
    pub input: TextArea,
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
    /// Command suggestion state for autocomplete
    pub command_state: CommandSuggestionState,
    /// Command suggestions for palette (temporary, uses simple strings)
    pub command_palette_suggestions: Vec<String>,
    /// Selected suggestion index for command palette
    pub command_palette_selected_index: usize,
    /// Scrollback offset for conversation history
    pub scrollback_offset: usize,
    /// Command palette state
    pub command_palette_active: bool,
    pub command_palette_query: String,
    /// Previous selected index (for table animation detection)
    pub previous_selected_index: usize,
    /// Whether chat history pane has focus (false = prompt editor focused)
    pub chat_history_focused: bool,
}

impl PromptData {
    pub fn new() -> Self {
        Self {
            context: DisplayContext::Help,
            input: TextArea::default(),
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
            command_state: CommandSuggestionState::new(),
            command_palette_suggestions: Vec::new(),
            command_palette_selected_index: 0,
            scrollback_offset: 0,
            command_palette_active: false,
            command_palette_query: String::new(),
            previous_selected_index: 0,
            chat_history_focused: false, // Prompt editor focused by default
        }
    }

    /// Get the current input text as a String.
    pub fn input_text(&self) -> String {
        self.input.text()
    }

    /// Clear the input textarea.
    pub fn clear_input(&mut self) {
        self.input.clear();
    }

    /// Set the input text content.
    pub fn set_input(&mut self, text: &str) {
        self.input.set_text(text);
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

    /// Toggle focus between chat history and prompt editor panes.
    pub fn toggle_focus(&mut self) {
        self.chat_history_focused = !self.chat_history_focused;
    }

    /// Check if chat history pane has focus.
    pub fn is_chat_focused(&self) -> bool {
        self.chat_history_focused
    }

    /// Check if prompt editor pane has focus.
    pub fn is_prompt_focused(&self) -> bool {
        !self.chat_history_focused
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_prompt_data_new() {
        let data = PromptData::new();
        assert_eq!(data.context, DisplayContext::Help);
        assert_eq!(data.input_text(), "");
        assert!(data.output.len() > 0);
    }

    #[test]
    fn test_input_text_empty() {
        let data = PromptData::new();
        assert_eq!(data.input_text(), "");
    }

    #[test]
    fn test_set_input() {
        let mut data = PromptData::new();
        data.set_input("test input");
        assert_eq!(data.input_text(), "test input");
    }

    #[test]
    fn test_set_input_multiline() {
        let mut data = PromptData::new();
        data.set_input("line1\nline2\nline3");
        assert_eq!(data.input_text(), "line1\nline2\nline3");
    }

    #[test]
    fn test_clear_input() {
        let mut data = PromptData::new();
        data.set_input("some text");
        assert_eq!(data.input_text(), "some text");
        data.clear_input();
        assert_eq!(data.input_text(), "");
    }

    #[test]
    fn test_textarea_basic_input() {
        let mut data = PromptData::new();
        data.input.handle_key(KeyCode::Char('h'), KeyModifiers::NONE);
        data.input.handle_key(KeyCode::Char('i'), KeyModifiers::NONE);
        assert_eq!(data.input_text(), "hi");
    }

    #[test]
    fn test_textarea_multiline_input() {
        let mut data = PromptData::new();
        // Type "hello"
        for c in "hello".chars() {
            data.input.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
        }
        // Press Enter
        data.input.handle_key(KeyCode::Enter, KeyModifiers::NONE);
        // Type "world"
        for c in "world".chars() {
            data.input.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
        }
        assert_eq!(data.input_text(), "hello\nworld");
    }

    #[test]
    fn test_textarea_backspace() {
        let mut data = PromptData::new();
        // Type "test"
        for c in "test".chars() {
            data.input.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
        }
        assert_eq!(data.input_text(), "test");
        // Press Backspace
        data.input.handle_key(KeyCode::Backspace, KeyModifiers::NONE);
        assert_eq!(data.input_text(), "tes");
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
                let selected_idx = data.selected_index.min(data.agents.len().saturating_sub(1));
                table.set_selected(Some(selected_idx));
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
            // Render split-pane layout: chat history (top) and prompt editor (bottom)
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(10),    // Chat history (minimum 10 lines)
                    Constraint::Length(6),  // Prompt editor (fixed 6 lines)
                ])
                .split(area);

            // Top pane: Chat history
            let viewport_height = chunks[0].height.saturating_sub(2) as usize;
            let visible_conversation = data.get_visible_conversation(viewport_height);

            // Parse visible conversation lines as markdown
            let mut markdown_lines = Vec::new();
            if visible_conversation.is_empty() {
                markdown_lines.push(ratatui::text::Line::from("No messages yet. Type a message to start!"));
            } else {
                for line in &visible_conversation {
                    let parsed = crate::views::markdown::render_markdown(line);
                    markdown_lines.extend(parsed);
                    markdown_lines.push(ratatui::text::Line::from("")); // Add spacing between messages
                }
            }

            let chat_title = if data.is_chat_focused() {
                format!("{} Chat History [FOCUSED]", Icons::CHAT)
            } else {
                format!("{} Chat History", Icons::CHAT)
            };

            let chat_widget = Paragraph::new(markdown_lines)
                .wrap(Wrap { trim: true })
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(
                            if data.is_chat_focused() {
                                THEME.primary()
                            } else {
                                THEME.border()
                            }
                        ))
                        .title(chat_title)
                )
                .style(Style::default().fg(THEME.text()))
                .scroll((0, 0)); // No scroll needed since we're already culling
            frame.render_widget(chat_widget, chunks[0]);

            // Bottom pane: Prompt editor
            let prompt_title = if data.is_prompt_focused() {
                format!("{} Prompt [FOCUSED]", Icons::CHAT) // Using CHAT icon for now, will add EDIT in Task 5
            } else {
                format!("{} Prompt", Icons::CHAT)
            };

            let prompt_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(
                    if data.is_prompt_focused() {
                        THEME.primary()
                    } else {
                        THEME.border()
                    }
                ))
                .title(prompt_title);

            // Render TextArea in the prompt pane
            let prompt_area = prompt_block.inner(chunks[1]);
            frame.render_widget(prompt_block, chunks[1]);
            frame.render_widget(data.input.clone(), prompt_area);
        }
        DisplayContext::Dashboard => {
            // Render dashboard with centered alignment
            let scroll_offset = data.scrollback_offset;

            let main_widget = Paragraph::new(main_content)
                .wrap(Wrap { trim: true })
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(THEME.border())),
                )
                .style(Style::default().fg(THEME.text()))
                .scroll((scroll_offset as u16, 0));
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

    // Show command menu popup if auto-completion is active (user has typed 2+ chars after '/')
    if data.command_state.is_active {
        let total_suggestions = data.command_state.suggestions.len();
        let selected_idx = data.command_state.selected_index;
        let (visible_start, visible_end) = data.command_state.visible_range;

        let visible_count = visible_end - visible_start;
        let has_more_above = visible_start > 0;
        let has_more_below = visible_end < total_suggestions;

        // Create popup with dynamic height
        let popup_width = 60;
        // If no suggestions, show minimal height for "No matches found" message
        let popup_height = if total_suggestions == 0 {
            5 // header + empty message + footer + spacing
        } else {
            (visible_count + 4) as u16 // suggestions + header + footer + spacing
        };

        let popup_area = Rect {
            x: area.x + 2,
            y: area.y.saturating_sub(popup_height),
            width: popup_width.min(area.width.saturating_sub(4)),
            height: popup_height,
        };

        // Build styled menu items
        const MAX_SUGGESTIONS_TO_SHOW: usize = 8;
        let trigger_indicator = if data.command_state.triggered_manually {
            " [Manual]"
        } else {
            ""
        };
        let title = if total_suggestions > MAX_SUGGESTIONS_TO_SHOW {
            format!(
                "{} Commands ({}/{}){}",
                Icons::INFO,
                selected_idx + 1,
                total_suggestions,
                trigger_indicator
            )
        } else {
            format!("{} Commands{}", Icons::INFO, trigger_indicator)
        };

        let mut menu_lines = vec![
            ratatui::text::Line::from(Span::styled(
                title,
                Style::default().fg(THEME.primary()).add_modifier(Modifier::BOLD),
            )),
            ratatui::text::Line::from(""),
        ];

        // REQ-198: Show error state or "No matches found" when active but no suggestions
        if let Some(error_msg) = &data.command_state.error_message {
            menu_lines.push(ratatui::text::Line::from(Span::styled(
                format!("  ⚠ {}", error_msg),
                Style::default().fg(THEME.error()),
            )));
        } else if total_suggestions == 0 {
            menu_lines.push(ratatui::text::Line::from(Span::styled(
                "  No matches found",
                Style::default().fg(THEME.text_muted()),
            )));
            menu_lines.push(ratatui::text::Line::from(""));
            menu_lines.push(ratatui::text::Line::from(Span::styled(
                "  Type /help to see all commands",
                Style::default().fg(THEME.text_dim()),
            )));
        } else {
            // Add scroll indicator at top if needed
            if has_more_above {
                let hidden_count = visible_start;
                menu_lines.push(ratatui::text::Line::from(Span::styled(
                    format!("  ▲ {} more above...", hidden_count),
                    Style::default().fg(THEME.text_dim()),
                )));
            }

            // Show visible suggestions
            for (i, suggestion) in data.command_state.suggestions.iter().enumerate().skip(visible_start).take(visible_count) {
                let is_selected = i == selected_idx;
                
                // Determine icon and color based on source
                let (source_icon, source_color) = match suggestion.source {
                    SuggestionSource::BuiltIn => ("", THEME.text()),
                    SuggestionSource::MCP => ("🔌 ", THEME.secondary()),
                    SuggestionSource::Agent => (Icons::AGENT, THEME.info()),
                    SuggestionSource::Workflow => ("⚙️ ", THEME.primary()),
                };
                
                let base_style = if is_selected {
                    Style::default().fg(THEME.bg_primary()).bg(THEME.primary()).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(THEME.text())
                };
                
                // Apply source color if not selected
                let style = if is_selected {
                    base_style
                } else {
                    base_style.fg(source_color)
                };

                let prefix = if is_selected { "▶ " } else { "  " };
                
                // Add parameter indicator for parameter suggestions
                let param_indicator = if suggestion.suggestion_type == crate::state::SuggestionType::Parameter {
                    if let Some(param_name) = &suggestion.parameter_name {
                        format!("<{}> ", param_name)
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };
                
                // Format suggestion with source icon, command and description
                let display_text = format!("{}{}{}{} - {}", prefix, source_icon, suggestion.command, param_indicator, suggestion.description);
                menu_lines.push(ratatui::text::Line::from(Span::styled(
                    display_text,
                    style,
                )));
            }

            // Add scroll indicator at bottom if needed
            if has_more_below {
                let hidden_count = total_suggestions - visible_end;
                menu_lines.push(ratatui::text::Line::from(Span::styled(
                    format!("  ▼ {} more below...", hidden_count),
                    Style::default().fg(THEME.text_dim()),
                )));
            }
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
    let popup_height = (data.command_palette_suggestions.len() + 6).min(20) as u16;

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

    if data.command_palette_suggestions.is_empty() {
        lines.push(Line::from(Span::styled(
            "No matching commands",
            Style::default().fg(THEME.text_muted()),
        )));
    } else {
        for (i, suggestion) in data.command_palette_suggestions.iter().enumerate() {
            let is_selected = i == data.command_palette_selected_index;
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
