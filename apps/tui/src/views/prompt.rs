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
            Constraint::Length(3),  // Title
            Constraint::Min(10),    // Main display area
            Constraint::Length(3),  // Input prompt
            Constraint::Length(2),  // Help line
        ])
        .split(area);

    // Title with current context - using theme colors
    let title = Paragraph::new(format!("{} Radium - {}", Icons::ROCKET, data.context.title()))
        .style(Style::default().fg(THEME.primary).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(THEME.border)));
    frame.render_widget(title, chunks[0]);

    // Main display area - content depends on context
    let main_content = match &data.context {
        DisplayContext::Chat { agent_id, .. } => {
            // Show conversation history
            if data.conversation.is_empty() {
                format!("Chat with {}\n\nNo messages yet. Type a message to start!", agent_id)
            } else {
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
            // Show session list
            if data.sessions.is_empty() {
                "No chat sessions found.\n\nUse /chat <agent> to start a new session.".to_string()
            } else {
                let mut output = String::from("Chat Sessions:\n\n");
                for (session_id, msg_count) in &data.sessions {
                    output.push_str(&format!("  {} ({} messages)\n", session_id, msg_count));
                }
                output
            }
        }
        DisplayContext::Dashboard | DisplayContext::Help => {
            // Show general output
            data.output.join("\n")
        }
    };

    let main_widget = Paragraph::new(main_content)
        .wrap(Wrap { trim: true })
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(THEME.border)))
        .style(Style::default().fg(THEME.text))
        .scroll((0, 0));
    frame.render_widget(main_widget, chunks[1]);

    // Input prompt with autocomplete suggestions
    let mut prompt_lines = vec![format!("> {}_", data.input)];

    // Show command suggestions if typing a slash command
    if !data.command_suggestions.is_empty() {
        prompt_lines.push("".to_string());
        prompt_lines.push("Suggestions:".to_string());
        for suggestion in &data.command_suggestions {
            prompt_lines.push(format!("  {}", suggestion));
        }
    }

    let prompt_text = prompt_lines.join("\n");
    let prompt = Paragraph::new(prompt_text)
        .style(Style::default().fg(THEME.primary))  // Cyan for input
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(THEME.border_active))
            .title(" Input "));
    frame.render_widget(prompt, chunks[2]);

    // Help line
    let help_text = "Type /help for commands | Ctrl+C to quit | Enter to send";
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(THEME.text_muted))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[3]);
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

    // Main wizard content
    let wizard_lines = wizard.display_lines();
    let content = wizard_lines.join("\n");

    let main_widget = Paragraph::new(content)
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
