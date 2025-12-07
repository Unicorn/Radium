//! New unified prompt-based application.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};

use crate::commands::{Command, DisplayContext};
use crate::views::PromptData;

/// Main application with unified prompt interface.
pub struct App {
    /// Whether to quit
    pub should_quit: bool,
    /// Prompt data (unified interface)
    pub prompt_data: PromptData,
    /// Current agent for chat (if any)
    pub current_agent: Option<String>,
    /// Current session for chat (if any)
    pub current_session: Option<String>,
    /// Whether user has completed initial setup
    pub setup_complete: bool,
    /// Available commands for autocomplete
    pub available_commands: Vec<(&'static str, &'static str)>,
}

impl App {
    pub fn new() -> Self {
        // Check if any auth is configured
        let gemini_auth =
            std::env::var("GEMINI_API_KEY").is_ok() || std::env::var("GOOGLE_API_KEY").is_ok();
        let openai_auth = std::env::var("OPENAI_API_KEY").is_ok();
        let anthropic_auth = std::env::var("ANTHROPIC_API_KEY").is_ok();

        let setup_complete = gemini_auth || openai_auth || anthropic_auth;

        let available_commands = vec![
            ("help", "Show all available commands"),
            ("agents", "List all available agents"),
            ("chat", "Start chat with an agent"),
            ("sessions", "Show your chat sessions"),
            ("dashboard", "Show system dashboard"),
        ];

        let mut app = Self {
            should_quit: false,
            prompt_data: PromptData::new(),
            current_agent: None,
            current_session: None,
            setup_complete,
            available_commands,
        };

        // Show setup instructions if not configured
        if !setup_complete {
            app.show_setup_instructions();
        } else {
            // Start in direct chat mode with default agent
            app.start_default_chat();
        }

        app
    }

    fn show_setup_instructions(&mut self) {
        self.prompt_data.context = DisplayContext::Help;
        self.prompt_data.clear_output();
        self.prompt_data.add_output("Welcome to Radium! 🚀".to_string());
        self.prompt_data.add_output("".to_string());
        self.prompt_data.add_output("⚠️  No AI providers configured yet.".to_string());
        self.prompt_data.add_output("".to_string());
        self.prompt_data.add_output("To get started, set up at least one API key:".to_string());
        self.prompt_data.add_output("".to_string());
        self.prompt_data.add_output("  For Gemini:".to_string());
        self.prompt_data.add_output("    export GEMINI_API_KEY='your-key-here'".to_string());
        self.prompt_data.add_output("".to_string());
        self.prompt_data.add_output("  For OpenAI:".to_string());
        self.prompt_data.add_output("    export OPENAI_API_KEY='your-key-here'".to_string());
        self.prompt_data.add_output("".to_string());
        self.prompt_data.add_output("  For Anthropic:".to_string());
        self.prompt_data.add_output("    export ANTHROPIC_API_KEY='your-key-here'".to_string());
        self.prompt_data.add_output("".to_string());
        self.prompt_data.add_output("After setting your key, restart the TUI.".to_string());
        self.prompt_data.add_output("".to_string());
        self.prompt_data.add_output("Type /help to see available commands.".to_string());
    }

    fn start_default_chat(&mut self) {
        // Start in direct chat mode - user can just start typing
        let session_id = format!("session_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S"));

        // Use a default "assistant" agent
        let agent_id = "assistant".to_string();

        self.current_agent = Some(agent_id.clone());
        self.current_session = Some(session_id.clone());

        self.prompt_data.context = DisplayContext::Chat {
            agent_id: agent_id.clone(),
            session_id: session_id.clone(),
        };

        self.prompt_data.conversation.clear();
        self.prompt_data.conversation.push("Welcome to Radium! I'm your AI assistant.".to_string());
        self.prompt_data.conversation.push("".to_string());
        self.prompt_data.conversation.push("Just start typing to chat, or use /help for commands.".to_string());
        self.prompt_data.conversation.push("Available: /agents, /sessions, /dashboard".to_string());
    }

    pub async fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Result<()> {
        match key {
            // Quit
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Char('d') if modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }

            // Enter - process command or send message
            KeyCode::Enter => {
                self.handle_enter().await?;
            }

            // Backspace
            KeyCode::Backspace => {
                self.prompt_data.pop_char();
                self.update_command_suggestions();
            }

            // Regular characters
            KeyCode::Char(c) if !modifiers.contains(KeyModifiers::CONTROL) => {
                self.prompt_data.push_char(c);
                self.update_command_suggestions();
            }

            _ => {}
        }

        Ok(())
    }

    async fn handle_enter(&mut self) -> Result<()> {
        let input = self.prompt_data.input.clone();
        if input.trim().is_empty() {
            return Ok(());
        }

        // Try to parse as command
        if let Some(cmd) = Command::parse(&input) {
            self.execute_command(cmd).await?;
        } else {
            // Regular chat message
            self.send_chat_message(input).await?;
        }

        self.prompt_data.clear_input();
        Ok(())
    }

    async fn execute_command(&mut self, cmd: Command) -> Result<()> {
        match cmd.name.as_str() {
            "help" => self.show_help(),
            "agents" => self.show_agents().await?,
            "sessions" => self.show_sessions().await?,
            "dashboard" => self.show_dashboard().await?,
            "chat" => {
                if cmd.args.is_empty() {
                    self.prompt_data.add_output("Usage: /chat <agent-id>".to_string());
                } else {
                    self.start_chat(&cmd.args[0]).await?;
                }
            }
            _ => {
                self.prompt_data
                    .add_output(format!("Unknown command: /{}. Type /help for help.", cmd.name));
            }
        }
        Ok(())
    }

    fn show_help(&mut self) {
        self.prompt_data.context = DisplayContext::Help;
        self.prompt_data.clear_output();
        self.prompt_data.add_output("Radium TUI Commands:".to_string());
        self.prompt_data.add_output("".to_string());
        self.prompt_data
            .add_output("  /chat <agent>   - Start chat with an agent".to_string());
        self.prompt_data
            .add_output("  /agents         - List all available agents".to_string());
        self.prompt_data
            .add_output("  /sessions       - Show your chat sessions".to_string());
        self.prompt_data
            .add_output("  /dashboard      - Show dashboard stats".to_string());
        self.prompt_data.add_output("  /help           - Show this help".to_string());
        self.prompt_data.add_output("".to_string());
        self.prompt_data
            .add_output("When in a chat, type normally to send messages.".to_string());
    }

    async fn show_agents(&mut self) -> Result<()> {
        self.prompt_data.context = DisplayContext::AgentList;

        // Get available agents
        let agents = crate::chat_executor::get_available_agents()?;
        self.prompt_data.agents = agents;

        Ok(())
    }

    async fn show_sessions(&mut self) -> Result<()> {
        self.prompt_data.context = DisplayContext::SessionList;

        // TODO: Load actual sessions from history
        self.prompt_data.sessions = vec![];

        Ok(())
    }

    async fn show_dashboard(&mut self) -> Result<()> {
        self.prompt_data.context = DisplayContext::Dashboard;
        self.prompt_data.clear_output();

        // Show basic stats
        self.prompt_data.add_output("Radium Dashboard".to_string());
        self.prompt_data.add_output("".to_string());
        self.prompt_data
            .add_output(format!("Agents: {}", self.prompt_data.agents.len()));

        // Check auth status
        let gemini_auth =
            std::env::var("GEMINI_API_KEY").is_ok() || std::env::var("GOOGLE_API_KEY").is_ok();
        let openai_auth = std::env::var("OPENAI_API_KEY").is_ok();
        let anthropic_auth = std::env::var("ANTHROPIC_API_KEY").is_ok();

        self.prompt_data.add_output("".to_string());
        self.prompt_data.add_output("Authentication:".to_string());
        self.prompt_data.add_output(format!(
            "  Gemini: {}",
            if gemini_auth { "✓" } else { "✗ (export GEMINI_API_KEY=...)" }
        ));
        self.prompt_data.add_output(format!(
            "  OpenAI: {}",
            if openai_auth { "✓" } else { "✗ (export OPENAI_API_KEY=...)" }
        ));
        self.prompt_data.add_output(format!(
            "  Anthropic: {}",
            if anthropic_auth { "✓" } else { "✗ (export ANTHROPIC_API_KEY=...)" }
        ));

        Ok(())
    }

    async fn start_chat(&mut self, agent_id: &str) -> Result<()> {
        // Generate session ID
        let session_id =
            format!("session_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S"));

        self.current_agent = Some(agent_id.to_string());
        self.current_session = Some(session_id.clone());

        self.prompt_data.context = DisplayContext::Chat {
            agent_id: agent_id.to_string(),
            session_id: session_id.clone(),
        };

        self.prompt_data.conversation.clear();
        self.prompt_data
            .conversation
            .push(format!("Started new chat with {} (session: {})", agent_id, session_id));
        self.prompt_data
            .conversation
            .push("Type your message below.".to_string());

        Ok(())
    }

    async fn send_chat_message(&mut self, message: String) -> Result<()> {
        // Check if in chat context
        let (agent_id, session_id) = match &self.prompt_data.context {
            DisplayContext::Chat { agent_id, session_id } => (agent_id.clone(), session_id.clone()),
            _ => {
                self.prompt_data
                    .add_output("Not in a chat session. Use /chat <agent> first.".to_string());
                return Ok(());
            }
        };

        // Add user message to conversation
        self.prompt_data
            .conversation
            .push(format!("You: {}", message));

        // Execute agent
        match crate::chat_executor::execute_chat_message(&agent_id, &message, &session_id).await {
            Ok(result) => {
                if result.success {
                    self.prompt_data
                        .conversation
                        .push(format!("Agent: {}", result.response));
                } else {
                    let error_msg = result
                        .error
                        .unwrap_or_else(|| "Unknown error".to_string());
                    self.prompt_data
                        .conversation
                        .push(format!("Error: {}", error_msg));
                }
            }
            Err(e) => {
                self.prompt_data
                    .conversation
                    .push(format!("Error: Failed to execute message: {}", e));
            }
        }

        Ok(())
    }

    fn update_command_suggestions(&mut self) {
        let input = &self.prompt_data.input;

        // Only show suggestions if typing a slash command
        if !input.starts_with('/') {
            self.prompt_data.command_suggestions.clear();
            return;
        }

        let partial = &input[1..]; // Remove the '/'

        // Filter commands that match the partial input
        let suggestions: Vec<String> = self
            .available_commands
            .iter()
            .filter(|(cmd, _desc)| cmd.starts_with(partial))
            .map(|(cmd, desc)| format!("/{} - {}", cmd, desc))
            .collect();

        self.prompt_data.command_suggestions = suggestions;
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
