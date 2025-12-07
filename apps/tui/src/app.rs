//! New unified prompt-based application.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use radium_core::auth::{CredentialStore, ProviderType};

use crate::commands::{Command, DisplayContext};
use crate::setup::SetupWizard;
use crate::views::PromptData;
use crate::workspace::WorkspaceStatus;

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
    /// Setup wizard (if running)
    pub setup_wizard: Option<SetupWizard>,
    /// Workspace status
    pub workspace_status: Option<WorkspaceStatus>,
}

impl App {
    pub fn new() -> Self {
        // Check if any auth is configured using CredentialStore
        let setup_complete = if let Ok(store) = CredentialStore::new() {
            store.is_configured(ProviderType::Gemini) || store.is_configured(ProviderType::OpenAI)
        } else {
            false
        };

        let available_commands = vec![
            ("help", "Show all available commands"),
            ("auth", "Authenticate with AI providers"),
            ("agents", "List all available agents"),
            ("chat", "Start chat with an agent"),
            ("sessions", "Show your chat sessions"),
            ("dashboard", "Show system dashboard"),
            ("models", "Select AI model"),
        ];

        // Initialize workspace
        let workspace_status = crate::workspace::initialize_workspace().ok();

        let mut app = Self {
            should_quit: false,
            prompt_data: PromptData::new(),
            current_agent: None,
            current_session: None,
            setup_complete,
            available_commands,
            setup_wizard: None,
            workspace_status,
        };

        // Show setup wizard if not configured, otherwise start chat
        if !setup_complete {
            app.setup_wizard = Some(SetupWizard::new());
        } else {
            // Start in direct chat mode with default agent
            app.start_default_chat();
        }

        app
    }

    fn start_default_chat(&mut self) {
        // Show welcome screen instead of trying to start chat
        // This avoids the "agent not found" error
        self.prompt_data.context = DisplayContext::Help;
        self.prompt_data.clear_output();

        self.prompt_data.add_output("Welcome to Radium! 🚀".to_string());
        self.prompt_data.add_output("".to_string());
        self.prompt_data.add_output("Radium is your AI-powered development assistant.".to_string());
        self.prompt_data.add_output("".to_string());

        // Check if we have any agents available
        let has_agents = crate::chat_executor::get_available_agents()
            .map(|agents| !agents.is_empty())
            .unwrap_or(false);

        if has_agents {
            self.prompt_data.add_output("🤖 Quick Start:".to_string());
            self.prompt_data.add_output("  /agents - See available AI agents".to_string());
            self.prompt_data.add_output("  /chat <agent> - Start chatting with an agent".to_string());
        } else {
            self.prompt_data.add_output("⚠️  No agents configured yet.".to_string());
            self.prompt_data.add_output("".to_string());
            self.prompt_data.add_output("To get started, create an agent configuration:".to_string());
            self.prompt_data.add_output("  1. Create ~/.radium/agents/ directory".to_string());
            self.prompt_data.add_output("  2. Add an agent JSON file (see example below)".to_string());
            self.prompt_data.add_output("".to_string());
            self.prompt_data.add_output("Example agent config (~/.radium/agents/assistant.json):".to_string());
            self.prompt_data.add_output("  {".to_string());
            self.prompt_data.add_output("    \"id\": \"assistant\",".to_string());
            self.prompt_data.add_output("    \"name\": \"Assistant\",".to_string());
            self.prompt_data.add_output("    \"description\": \"General purpose AI assistant\",".to_string());
            self.prompt_data.add_output("    \"system_prompt\": \"You are a helpful AI assistant.\",".to_string());
            self.prompt_data.add_output("    \"model\": \"gemini-1.5-flash\"".to_string());
            self.prompt_data.add_output("  }".to_string());
        }

        self.prompt_data.add_output("".to_string());
        self.prompt_data.add_output("📚 Available Commands:".to_string());
        self.prompt_data.add_output("  /help - Show all commands".to_string());
        self.prompt_data.add_output("  /auth - Manage authentication".to_string());
        self.prompt_data.add_output("  /dashboard - View system status".to_string());
        self.prompt_data.add_output("".to_string());
        self.prompt_data.add_output("Type a command to get started!".to_string());
    }

    pub async fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Result<()> {
        // If setup wizard is active, delegate to it
        if let Some(wizard) = &mut self.setup_wizard {
            let done = wizard.handle_key(key, modifiers).await?;
            if done {
                // Setup complete or skipped
                self.setup_wizard = None;
                self.setup_complete = true;
                self.start_default_chat();
            }
            return Ok(());
        }

        // Normal key handling
        match key {
            // Quit
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Char('d') if modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }

            // Arrow keys for command menu navigation
            KeyCode::Up if !self.prompt_data.command_suggestions.is_empty() => {
                self.prompt_data.selected_suggestion_index =
                    self.prompt_data.selected_suggestion_index.saturating_sub(1);
            }
            KeyCode::Down if !self.prompt_data.command_suggestions.is_empty() => {
                let max_index = self.prompt_data.command_suggestions.len().saturating_sub(1);
                self.prompt_data.selected_suggestion_index =
                    (self.prompt_data.selected_suggestion_index + 1).min(max_index);
            }

            // Tab to autocomplete selected command
            KeyCode::Tab if !self.prompt_data.command_suggestions.is_empty() => {
                self.autocomplete_selected_command();
            }

            // Escape to cancel command menu
            KeyCode::Esc if !self.prompt_data.command_suggestions.is_empty() => {
                self.prompt_data.command_suggestions.clear();
                self.prompt_data.selected_suggestion_index = 0;
            }

            // Enter - process command or send message (unless in command palette)
            KeyCode::Enter if !self.prompt_data.command_palette_active => {
                self.handle_enter().await?;
            }

            // Backspace (unless in command palette)
            KeyCode::Backspace if !self.prompt_data.command_palette_active => {
                self.prompt_data.pop_char();
                self.update_command_suggestions();
            }

            // Scrollback navigation
            KeyCode::PageUp => {
                self.prompt_data.scrollback_offset =
                    (self.prompt_data.scrollback_offset + 10).min(
                        self.prompt_data.conversation.len().saturating_sub(1)
                    );
            }
            KeyCode::PageDown => {
                self.prompt_data.scrollback_offset =
                    self.prompt_data.scrollback_offset.saturating_sub(10);
            }
            KeyCode::Home => {
                self.prompt_data.scrollback_offset = 0;
            }
            KeyCode::End => {
                self.prompt_data.scrollback_offset = self.prompt_data.conversation.len();
            }

            // Command palette (Ctrl+P)
            KeyCode::Char('p') if modifiers.contains(KeyModifiers::CONTROL) => {
                self.prompt_data.command_palette_active = true;
                self.prompt_data.command_palette_query.clear();
                self.update_command_palette();
            }

            // Escape to close command palette
            KeyCode::Esc if self.prompt_data.command_palette_active => {
                self.prompt_data.command_palette_active = false;
                self.prompt_data.command_palette_query.clear();
                self.prompt_data.command_suggestions.clear();
            }

            // Enter in command palette
            KeyCode::Enter if self.prompt_data.command_palette_active => {
                if let Some(suggestion) = self.prompt_data.command_suggestions
                    .get(self.prompt_data.selected_suggestion_index)
                {
                    if let Some(cmd) = suggestion.split(" - ").next() {
                        self.prompt_data.input = cmd.to_string();
                        self.prompt_data.command_palette_active = false;
                        self.prompt_data.command_palette_query.clear();
                        // Execute the command
                        self.handle_enter().await?;
                    }
                }
            }

            // Arrow keys in command palette
            KeyCode::Up if self.prompt_data.command_palette_active => {
                self.prompt_data.selected_suggestion_index =
                    self.prompt_data.selected_suggestion_index.saturating_sub(1);
            }
            KeyCode::Down if self.prompt_data.command_palette_active => {
                let max_index = self.prompt_data.command_suggestions.len().saturating_sub(1);
                self.prompt_data.selected_suggestion_index =
                    (self.prompt_data.selected_suggestion_index + 1).min(max_index);
            }

            // Backspace in command palette
            KeyCode::Backspace if self.prompt_data.command_palette_active => {
                self.prompt_data.command_palette_query.pop();
                self.update_command_palette();
            }

            // Regular characters
            KeyCode::Char(c) if !modifiers.contains(KeyModifiers::CONTROL) => {
                if self.prompt_data.command_palette_active {
                    self.prompt_data.command_palette_query.push(c);
                    self.update_command_palette();
                } else {
                    self.prompt_data.push_char(c);
                    self.update_command_suggestions();
                }
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
            "auth" => self.start_auth_wizard(),
            "agents" => self.show_agents().await?,
            "sessions" => self.show_sessions().await?,
            "dashboard" => self.show_dashboard().await?,
            "models" => self.show_models().await?,
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
            .add_output("  /auth           - Authenticate with AI providers".to_string());
        self.prompt_data
            .add_output("  /chat <agent>   - Start chat with an agent".to_string());
        self.prompt_data
            .add_output("  /agents         - List all available agents".to_string());
        self.prompt_data
            .add_output("  /sessions       - Show your chat sessions".to_string());
        self.prompt_data
            .add_output("  /dashboard      - Show dashboard stats".to_string());
        self.prompt_data
            .add_output("  /models         - Select AI model".to_string());
        self.prompt_data.add_output("  /help           - Show this help".to_string());
        self.prompt_data.add_output("".to_string());
        self.prompt_data
            .add_output("When in a chat, type normally to send messages.".to_string());
    }

    fn start_auth_wizard(&mut self) {
        // Trigger the setup wizard for authentication
        self.setup_wizard = Some(SetupWizard::new());
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

        // Load actual sessions from history
        let workspace_root = self.workspace_status.as_ref()
            .and_then(|s| s.root.clone());
        let session_manager = crate::session_manager::SessionManager::new(workspace_root)?;
        let sessions_by_date = session_manager.load_sessions()?;

        // Flatten sessions for display
        self.prompt_data.sessions = sessions_by_date
            .values()
            .flatten()
            .map(|s| (s.session_id.clone(), s.message_count))
            .collect();

        Ok(())
    }

    async fn show_models(&mut self) -> Result<()> {
        self.prompt_data.context = DisplayContext::ModelSelector;
        // Model selection will be handled in render
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

        // Check auth status using CredentialStore
        let (gemini_auth, openai_auth) = if let Ok(store) = CredentialStore::new() {
            (
                store.is_configured(ProviderType::Gemini),
                store.is_configured(ProviderType::OpenAI),
            )
        } else {
            (false, false)
        };

        self.prompt_data.add_output("".to_string());
        self.prompt_data.add_output("Authentication:".to_string());
        self.prompt_data.add_output(format!(
            "  Gemini: {}",
            if gemini_auth { "✓ Configured" } else { "✗ Not configured (run: rad auth login gemini)" }
        ));
        self.prompt_data.add_output(format!(
            "  OpenAI: {}",
            if openai_auth { "✓ Configured" } else { "✗ Not configured (run: rad auth login openai)" }
        ));
        self.prompt_data.add_output("".to_string());
        self.prompt_data.add_output("Credentials stored in: ~/.radium/auth/credentials.json".to_string());

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

        // Save session update
        let workspace_root = self.workspace_status.as_ref()
            .and_then(|s| s.root.clone());
        if let Ok(session_manager) = crate::session_manager::SessionManager::new(workspace_root) {
            let _ = session_manager.update_session(&session_id, &agent_id, &message);
        }

        // Execute agent
        match crate::chat_executor::execute_chat_message(&agent_id, &message, &session_id).await {
            Ok(result) => {
                if result.success {
                    let response = result.response.clone();
                    self.prompt_data
                        .conversation
                        .push(format!("Agent: {}", response));

                    // Save agent response to session
                    let workspace_root_clone = self.workspace_status.as_ref()
                        .and_then(|s| s.root.clone());
                    if let Ok(session_manager) = crate::session_manager::SessionManager::new(workspace_root_clone) {
                        let _ = session_manager.update_session(&session_id, &agent_id, &response);
                    }
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
            self.prompt_data.selected_suggestion_index = 0;
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

        // Reset selection if list changed
        self.prompt_data.selected_suggestion_index = 0;
    }

    fn autocomplete_selected_command(&mut self) {
        if let Some(suggestion) = self.prompt_data.command_suggestions
            .get(self.prompt_data.selected_suggestion_index)
        {
            // Extract just the command part (before the ' - ')
            if let Some(cmd) = suggestion.split(" - ").next() {
                self.prompt_data.input = cmd.to_string();
                self.prompt_data.command_suggestions.clear();
                self.prompt_data.selected_suggestion_index = 0;
            }
        }
    }

    fn update_command_palette(&mut self) {
        // Use fuzzy matching for command palette
        let query = &self.prompt_data.command_palette_query.to_lowercase();
        
        self.prompt_data.command_suggestions = self
            .available_commands
            .iter()
            .filter(|(cmd, _desc)| {
                cmd.contains(query) || 
                cmd.starts_with(query) ||
                fuzzy_match(cmd, query)
            })
            .map(|(cmd, desc)| format!("/{} - {}", cmd, desc))
            .collect();
    }
}

/// Simple fuzzy match helper (basic implementation).
fn fuzzy_match(text: &str, query: &str) -> bool {
    let text_lower = text.to_lowercase();
    let mut query_chars = query.chars();
    let mut text_chars = text_lower.chars();
    
    while let Some(qc) = query_chars.next() {
        loop {
            match text_chars.next() {
                Some(tc) if tc == qc => break,
                Some(_) => continue,
                None => return false,
            }
        }
    }
    true
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
