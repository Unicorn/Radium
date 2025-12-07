//! New unified prompt-based application.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use radium_core::auth::{CredentialStore, ProviderType};
use radium_core::mcp::{McpIntegration, SlashCommandRegistry};
use radium_core::workflow::RequirementExecutor;
use radium_core::agents::registry::AgentRegistry;
use radium_core::storage::Database;
use radium_models::{ModelFactory, ModelConfig, ModelType};
use radium_orchestrator::{OrchestrationConfig, OrchestrationService, AgentExecutor, Orchestrator};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::commands::{Command, DisplayContext};
use crate::components::{DialogManager, ToastManager};
use crate::config::TuiConfig;
use crate::requirement_progress::ActiveRequirement;
use crate::setup::SetupWizard;
use crate::state::WorkflowUIState;
use crate::theme::RadiumTheme;
use crate::views::PromptData;
use crate::workspace::WorkspaceStatus;

/// Default maximum conversation history if config is unavailable
const _DEFAULT_MAX_CONVERSATION_HISTORY: usize = 500;

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
    /// Available subcommands for hierarchical autocomplete
    pub available_subcommands: std::collections::HashMap<&'static str, Vec<(&'static str, &'static str)>>,
    /// Setup wizard (if running)
    pub setup_wizard: Option<SetupWizard>,
    /// Workspace status
    pub workspace_status: Option<WorkspaceStatus>,
    /// Orchestration service for natural conversation
    pub orchestration_service: Option<Arc<OrchestrationService>>,
    /// Whether orchestration is enabled
    pub orchestration_enabled: bool,
    /// MCP integration for external tools
    pub mcp_integration: Option<Arc<Mutex<McpIntegration>>>,
    /// MCP slash command registry
    pub mcp_slash_registry: SlashCommandRegistry,
    /// Current theme (loaded from config)
    pub theme: RadiumTheme,
    /// Whether to show keyboard shortcuts overlay
    pub show_shortcuts: bool,
    /// TUI configuration
    pub config: TuiConfig,
    /// Whether orchestration is currently running (for cancellation support)
    pub orchestration_running: bool,
    /// Toast notification manager
    pub toast_manager: ToastManager,
    /// Dialog manager for interactive selections
    pub dialog_manager: DialogManager,
    /// Workflow UI state (when in workflow mode)
    pub workflow_state: Option<WorkflowUIState>,
    /// Selected agent ID in workflow mode
    pub selected_agent_id: Option<String>,
    /// Active requirement execution (for async progress tracking)
    pub active_requirement: Option<ActiveRequirement>,
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
            ("orchestrator", "Manage orchestration settings"),
            ("requirement", "Execute a Braingrid requirement autonomously"),
            ("complete", "Complete a requirement from source (file, Jira, or REQ)"),
        ];

        // Define subcommands for hierarchical completion
        let available_subcommands: std::collections::HashMap<&str, Vec<(&str, &str)>> = [
            ("orchestrator", vec![
                ("status", "Show orchestration status"),
                ("toggle", "Enable/disable orchestration"),
                ("switch", "Switch orchestration provider"),
                ("config", "Show full configuration"),
                ("refresh", "Reload agent tool registry"),
            ]),
            ("requirement", vec![
                ("list", "List all requirements"),
            ]),
        ]
        .iter()
        .cloned()
        .collect();

        // Initialize workspace
        let workspace_status = crate::workspace::initialize_workspace().ok();

        // Initialize orchestration service
        let (orchestration_service, orchestration_enabled) = Self::init_orchestration();

        // Initialize MCP integration (will be loaded asynchronously)
        let mcp_integration = workspace_status.as_ref().and_then(|ws| {
            if let Some(root) = &ws.root {
                let _workspace = radium_core::Workspace::create(root.clone()).ok();
                let integration = Arc::new(Mutex::new(McpIntegration::new()));
                // Initialize asynchronously - will be done on first use
                Some(integration)
            } else {
                None
            }
        });

        // Load theme and config
        let theme = RadiumTheme::from_config();
        let config = TuiConfig::load().unwrap_or_default();

        let mut app = Self {
            should_quit: false,
            prompt_data: PromptData::new(),
            current_agent: None,
            current_session: None,
            setup_complete,
            available_commands,
            available_subcommands,
            setup_wizard: None,
            workspace_status,
            orchestration_service,
            orchestration_enabled,
            mcp_integration,
            mcp_slash_registry: SlashCommandRegistry::new(),
            theme,
            show_shortcuts: false,
            config,
            orchestration_running: false,
            toast_manager: ToastManager::new(),
            dialog_manager: DialogManager::new(),
            workflow_state: None,
            selected_agent_id: None,
            active_requirement: None,
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

    /// Initialize orchestration service
    fn init_orchestration() -> (Option<Arc<OrchestrationService>>, bool) {
        // Try to load config from workspace, fall back to default path, then defaults
        let config = if let Ok(workspace) = radium_core::Workspace::discover() {
            let workspace_config_path = workspace.structure().orchestration_config_file();
            if workspace_config_path.exists() {
                tracing::info!("Loading orchestration config from workspace: {}", workspace_config_path.display());
            }
            OrchestrationConfig::load_from_workspace_path(workspace_config_path)
        } else {
            tracing::info!("Using default orchestration config (no workspace found)");
            OrchestrationConfig::load_from_toml(OrchestrationConfig::default_config_path())
                .unwrap_or_else(|_| OrchestrationConfig::default())
        };
        let enabled = config.enabled;

        // Return None for now - will be initialized asynchronously on first use
        (None, enabled)
    }

    /// Ensure orchestration service is initialized (lazy initialization)
    async fn ensure_orchestration_service(&mut self) -> Result<()> {
        if self.orchestration_service.is_none() && self.orchestration_enabled {
            // Load config from workspace, fall back to defaults
            let config = if let Ok(workspace) = radium_core::Workspace::discover() {
                let workspace_config_path = workspace.structure().orchestration_config_file();
                OrchestrationConfig::load_from_workspace_path(workspace_config_path)
            } else {
                OrchestrationConfig::load_from_toml(OrchestrationConfig::default_config_path())
                    .unwrap_or_else(|_| OrchestrationConfig::default())
            };
            
            // Discover MCP tools if MCP integration is available
            let mcp_tools = if let Some(ref mcp_integration) = self.mcp_integration {
                // Ensure MCP is initialized
                if let Some(workspace) = &self.workspace_status {
                    if let Some(root) = &workspace.root {
                        let workspace = radium_core::Workspace::create(root.clone())?;
                        let integration = mcp_integration.lock().await;
                        if integration.connected_server_count().await == 0 {
                            // Initialize if not already initialized
                            drop(integration);
                            mcp_integration.lock().await.initialize(&workspace).await.ok();
                        }
                    }
                }
                
                // Discover MCP tools for orchestration
                use radium_core::mcp::orchestration_bridge::discover_mcp_tools_for_orchestration;
                discover_mcp_tools_for_orchestration(Arc::clone(mcp_integration))
                    .await
                    .ok()
            } else {
                None
            };
            
            match OrchestrationService::initialize(config, mcp_tools).await {
                Ok(service) => {
                    self.orchestration_service = Some(Arc::new(service));
                }
                Err(e) => {
                    // Failed to initialize - log and disable orchestration
                    self.prompt_data.add_output(format!(
                        "⚠️  Orchestration initialization failed: {}",
                        e
                    ));
                    self.orchestration_enabled = false;
                }
            }
        }
        Ok(())
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
            self.prompt_data
                .add_output("  /chat <agent> - Start chatting with an agent".to_string());
        } else {
            self.prompt_data.add_output("⚠️  No agents configured yet.".to_string());
            self.prompt_data.add_output("".to_string());
            self.prompt_data
                .add_output("To get started, create an agent configuration:".to_string());
            self.prompt_data.add_output("  1. Create ~/.radium/agents/ directory".to_string());
            self.prompt_data
                .add_output("  2. Add an agent JSON file (see example below)".to_string());
            self.prompt_data.add_output("".to_string());
            self.prompt_data
                .add_output("Example agent config (~/.radium/agents/assistant.json):".to_string());
            self.prompt_data.add_output("  {".to_string());
            self.prompt_data.add_output("    \"id\": \"assistant\",".to_string());
            self.prompt_data.add_output("    \"name\": \"Assistant\",".to_string());
            self.prompt_data
                .add_output("    \"description\": \"General purpose AI assistant\",".to_string());
            self.prompt_data.add_output(
                "    \"system_prompt\": \"You are a helpful AI assistant.\",".to_string(),
            );
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
        // If dialog is open, handle dialog input first
        if self.dialog_manager.is_open() {
            if let Some(_value) = self.dialog_manager.handle_key(key) {
                // Dialog was closed with a selection - handle it
                // This will be handled by the command that opened the dialog
                return Ok(());
            }
            // Dialog handled the key (or it was Esc to close)
            if !self.dialog_manager.is_open() {
                return Ok(());
            }
        }

        // If shortcuts overlay is active, handle dismissal
        if self.show_shortcuts {
            match key {
                KeyCode::Esc | KeyCode::Char('?') | KeyCode::F(1) => {
                    self.show_shortcuts = false;
                    return Ok(());
                }
                _ => {
                    // Any other key also dismisses
                    self.show_shortcuts = false;
                    return Ok(());
                }
            }
        }

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
            // Show shortcuts overlay
            KeyCode::Char('?') | KeyCode::F(1) => {
                self.show_shortcuts = true;
                return Ok(());
            }
            // Quit or cancel orchestration
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                if self.orchestration_running {
                    // Cancel orchestration if running
                    self.orchestration_running = false;
                    let max_history = self.config.performance.max_conversation_history;
                    self.prompt_data.add_conversation_message(
                        "⚠️  Cancellation requested. Current operation will complete due to timeout protection.".to_string(),
                        max_history
                    );
                } else {
                    self.should_quit = true;
                }
            }
            KeyCode::Char('d') if modifiers.contains(KeyModifiers::CONTROL) => {
                if self.orchestration_running {
                    // Cancel orchestration if running
                    self.orchestration_running = false;
                    let max_history = self.config.performance.max_conversation_history;
                    self.prompt_data.add_conversation_message(
                        "⚠️  Cancellation requested. Current operation will complete due to timeout protection.".to_string(),
                        max_history
                    );
                } else {
                    self.should_quit = true;
                }
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

            // Enter - autocomplete+execute if suggestions showing, otherwise process command
            KeyCode::Enter if !self.prompt_data.command_palette_active => {
                if !self.prompt_data.command_suggestions.is_empty() {
                    // Autocomplete selected suggestion
                    self.autocomplete_selected_command();
                    // Then execute it
                    self.handle_enter().await?;
                } else {
                    self.handle_enter().await?;
                }
            }

            // Backspace (unless in command palette)
            KeyCode::Backspace if !self.prompt_data.command_palette_active => {
                self.prompt_data.pop_char();
                self.update_command_suggestions();
            }

            // Scrollback navigation
            KeyCode::PageUp => {
                self.prompt_data.scrollback_offset = (self.prompt_data.scrollback_offset + 10)
                    .min(self.prompt_data.conversation.len().saturating_sub(1));
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
                if let Some(suggestion) = self
                    .prompt_data
                    .command_suggestions
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

        // Try to parse as command (starts with /)
        if let Some(cmd) = Command::parse(&input) {
            self.execute_command(cmd).await?;
        } else {
            // Non-command input - route through orchestration if enabled
            if self.orchestration_enabled {
                self.handle_orchestrated_input(input).await?;
            } else {
                // Fallback to regular chat
                self.send_chat_message(input).await?;
            }
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
            "orchestrator" => {
                self.handle_orchestrator_command(&cmd.args).await?;
            }
            "requirement" => {
                if cmd.args.is_empty() {
                    self.prompt_data.add_output("Usage: /requirement <REQ-ID> [project <PROJECT-ID>]".to_string());
                    self.prompt_data.add_output("       /requirement list [project <PROJECT-ID>]".to_string());
                    self.prompt_data.add_output("".to_string());
                    self.prompt_data.add_output("Examples:".to_string());
                    self.prompt_data.add_output("  /requirement REQ-173 project PROJ-14".to_string());
                    self.prompt_data.add_output("  /requirement REQ-173".to_string());
                    self.prompt_data.add_output("  /requirement list project PROJ-14".to_string());
                } else if cmd.args[0] == "list" {
                    // List requirements
                    let project_id = if cmd.args.len() >= 3 && cmd.args[1] == "project" {
                        Some(cmd.args[2].clone())
                    } else {
                        None
                    };
                    self.handle_requirement_list(project_id).await?;
                } else {
                    let req_id = cmd.args[0].clone();
                    let project_id = if cmd.args.len() >= 3 && cmd.args[1] == "project" {
                        Some(cmd.args[2].clone())
                    } else {
                        None
                    };
                    self.handle_requirement(&req_id, project_id).await?;
                }
            }
            "complete" => {
                if cmd.args.is_empty() {
                    self.prompt_data.add_output("Usage: /complete <source>".to_string());
                    self.prompt_data.add_output("  Source can be:".to_string());
                    self.prompt_data.add_output("    - File path: ./specs/feature.md".to_string());
                    self.prompt_data.add_output("    - Jira ticket: RAD-42".to_string());
                    self.prompt_data.add_output("    - Braingrid REQ: REQ-2025-001".to_string());
                } else {
                    self.handle_complete(&cmd.args[0]).await?;
                }
            }
            "mcp-commands" | "mcp-help" => {
                self.show_mcp_commands().await?;
            }
            "reload-config" => {
                self.reload_config().await?;
            }
            _ => {
                // Check if it's an MCP slash command
                let full_command = format!("/{}", cmd.name);
                let mcp_prompt = self.mcp_slash_registry.get_command(&full_command).cloned();
                if let Some(prompt) = mcp_prompt {
                    // Execute MCP prompt
                    self.execute_mcp_prompt(&full_command, &prompt, &cmd.args).await?;
                } else {
                    // Try to load MCP prompts if not found
                    self.ensure_mcp_loaded().await?;
                    let mcp_prompt = self.mcp_slash_registry.get_command(&full_command).cloned();
                    if let Some(prompt) = mcp_prompt {
                        self.execute_mcp_prompt(&full_command, &prompt, &cmd.args).await?;
                    } else {
                        self.prompt_data
                            .add_output(format!("Unknown command: /{}. Type /help for help.", cmd.name));
                    }
                }
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
        self.prompt_data.add_output("  /chat <agent>   - Start chat with an agent".to_string());
        self.prompt_data.add_output("  /agents         - List all available agents".to_string());
        self.prompt_data.add_output("  /sessions       - Show your chat sessions".to_string());
        self.prompt_data.add_output("  /dashboard      - Show dashboard stats".to_string());
        self.prompt_data.add_output("  /models         - Select AI model".to_string());
        self.prompt_data.add_output("  /orchestrator   - Manage orchestration (status, toggle, switch, config, refresh)".to_string());
        self.prompt_data.add_output("  /requirement    - Execute Braingrid requirement (use 'list' to list requirements)".to_string());
        self.prompt_data.add_output("  /complete       - Complete requirement from source".to_string());
        self.prompt_data.add_output("  /reload-config  - Reload configuration file".to_string());
        self.prompt_data.add_output("  /help           - Show this help".to_string());
        
        let mcp_count = self.mcp_slash_registry.get_all_commands().len();
        if mcp_count > 0 {
            self.prompt_data.add_output(format!("  /mcp-commands   - List MCP slash commands ({} available)", mcp_count));
        }
        
        self.prompt_data.add_output("".to_string());
        self.prompt_data.add_output("💡 Natural Conversation:".to_string());
        self.prompt_data.add_output(format!(
            "   Type naturally without / to use AI orchestration (currently: {})",
            if self.orchestration_enabled { "enabled" } else { "disabled" }
        ));
        self.prompt_data.add_output("".to_string());
        self.prompt_data.add_output("When in a chat, type normally to send messages.".to_string());
        self.prompt_data.add_output("".to_string());
        self.prompt_data.add_output("💡 Tips:".to_string());
        self.prompt_data.add_output("   Press Ctrl+C during orchestration to request cancellation".to_string());
        self.prompt_data.add_output("   (Note: Current operation will complete due to timeout protection)".to_string());
    }

    /// Show MCP commands
    async fn show_mcp_commands(&mut self) -> Result<()> {
        self.ensure_mcp_loaded().await?;
        
        let commands = self.mcp_slash_registry.get_all_commands();
        
        self.prompt_data.clear_output();
        if commands.is_empty() {
            self.prompt_data.add_output("No MCP commands available.".to_string());
            self.prompt_data.add_output("Configure MCP servers to enable slash commands.".to_string());
            return Ok(());
        }

        self.prompt_data.add_output("MCP Slash Commands:".to_string());
        self.prompt_data.add_output("".to_string());
        
        for (cmd_name, prompt) in commands {
            let desc = prompt
                .description
                .as_ref()
                .map(|d| d.as_str())
                .unwrap_or("No description");
            self.prompt_data.add_output(format!("  {} - {}", cmd_name, desc));
            
            // Show arguments if available
            if let Some(args) = &prompt.arguments {
                if !args.is_empty() {
                    for arg in args {
                        let required = if arg.required { "required" } else { "optional" };
                        let arg_desc = arg
                            .description
                            .as_ref()
                            .map(|d| d.as_str())
                            .unwrap_or("");
                        self.prompt_data.add_output(format!("      {} {}: {}", arg.name, required, arg_desc));
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Ensure MCP integration is loaded
    async fn ensure_mcp_loaded(&mut self) -> Result<()> {
        if let Some(ref integration) = self.mcp_integration {
            if integration.lock().await.connected_server_count().await == 0 {
                if let Some(ref ws_status) = self.workspace_status {
                    if let Some(ref root) = ws_status.root {
                        if let Some(workspace) = radium_core::Workspace::create(root.clone()).ok() {
                            integration.lock().await.initialize(&workspace).await?;
                        }
                        
                        // Load prompts
                        let prompts = integration.lock().await.get_all_prompts().await;
                        for (server_name, prompt) in prompts {
                            self.mcp_slash_registry.register_prompt_with_server(server_name, prompt);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Execute an MCP prompt
    async fn execute_mcp_prompt(
        &mut self,
        command: &str,
        prompt: &radium_core::mcp::McpPrompt,
        args: &[String],
    ) -> Result<()> {
        // Get server name from registry
        let server_name = self
            .mcp_slash_registry
            .get_server_for_command(command)
            .ok_or_else(|| anyhow::anyhow!("Could not find server for command: {}", command))?;

        // Parse arguments
        let mcp_args = if !args.is_empty() {
            let mut arg_map = serde_json::Map::new();
            for (i, arg) in args.iter().enumerate() {
                if let Some(arg_def) = prompt.arguments.as_ref().and_then(|args| args.get(i)) {
                    arg_map.insert(arg_def.name.clone(), serde_json::Value::String(arg.clone()));
                } else {
                    arg_map.insert(format!("arg{}", i), serde_json::Value::String(arg.clone()));
                }
            }
            Some(serde_json::Value::Object(arg_map))
        } else {
            None
        };

        // Execute prompt
        if let Some(ref integration) = self.mcp_integration {
            match integration
                .lock()
                .await
                .execute_prompt(server_name, &prompt.name, mcp_args)
                .await
            {
                Ok(result) => {
                    // Format result for display
                    if let Some(messages) = result.get("messages").and_then(|m| m.as_array()) {
                        for message in messages {
                            if let Some(content) = message.get("content").and_then(|c| c.as_array()) {
                                for item in content {
                                    if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                                        self.prompt_data.add_output(text.to_string());
                                    }
                                }
                            } else if let Some(text) = message.get("content").and_then(|c| c.as_str()) {
                                self.prompt_data.add_output(text.to_string());
                            }
                        }
                    } else {
                        // Fallback: show JSON
                        self.prompt_data.add_output(
                            serde_json::to_string_pretty(&result)
                                .unwrap_or_else(|_| format!("Prompt '{}' executed", prompt.name))
                        );
                    }
                }
                Err(e) => {
                    self.prompt_data
                        .add_output(format!("MCP Error: {}", e));
                }
            }
        } else {
            self.prompt_data
                .add_output("MCP integration not available".to_string());
        }

        Ok(())
    }

    fn start_auth_wizard(&mut self) {
        // Trigger the setup wizard for authentication, skip welcome screen
        self.setup_wizard = Some(SetupWizard::new_skip_welcome());
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
        let workspace_root = self.workspace_status.as_ref().and_then(|s| s.root.clone());
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
        self.prompt_data.add_output(format!("Agents: {}", self.prompt_data.agents.len()));

        // Check auth status using CredentialStore
        let (gemini_auth, openai_auth) = if let Ok(store) = CredentialStore::new() {
            (store.is_configured(ProviderType::Gemini), store.is_configured(ProviderType::OpenAI))
        } else {
            (false, false)
        };

        self.prompt_data.add_output("".to_string());
        self.prompt_data.add_output("Authentication:".to_string());
        self.prompt_data.add_output(format!(
            "  Gemini: {}",
            if gemini_auth {
                "✓ Configured"
            } else {
                "✗ Not configured (run: rad auth login gemini)"
            }
        ));
        self.prompt_data.add_output(format!(
            "  OpenAI: {}",
            if openai_auth {
                "✓ Configured"
            } else {
                "✗ Not configured (run: rad auth login openai)"
            }
        ));
        self.prompt_data.add_output("".to_string());
        self.prompt_data
            .add_output("Credentials stored in: ~/.radium/auth/credentials.json".to_string());

        Ok(())
    }

    async fn start_chat(&mut self, agent_id: &str) -> Result<()> {
        // Generate session ID
        let session_id = format!("session_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S"));

        self.current_agent = Some(agent_id.to_string());
        self.current_session = Some(session_id.clone());

        self.prompt_data.context =
            DisplayContext::Chat { agent_id: agent_id.to_string(), session_id: session_id.clone() };

        self.prompt_data.conversation.clear();
        let max_history = self.config.performance.max_conversation_history;
        self.prompt_data.add_conversation_message(
            format!("Started new chat with {} (session: {})", agent_id, session_id),
            max_history,
        );
        self.prompt_data.add_conversation_message("Type your message below.".to_string(), max_history);

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

        // Add user message to conversation with limit
        let max_history = self.config.performance.max_conversation_history;
        self.prompt_data.add_conversation_message(format!("You: {}", message), max_history);

        // Save session update
        let workspace_root = self.workspace_status.as_ref().and_then(|s| s.root.clone());
        if let Ok(session_manager) = crate::session_manager::SessionManager::new(workspace_root) {
            let _ = session_manager.update_session(&session_id, &agent_id, &message);
        }

        // Execute agent
        match crate::chat_executor::execute_chat_message(&agent_id, &message, &session_id).await {
            Ok(result) => {
                if result.success {
                    let response = result.response.clone();
                    let max_history = self.config.performance.max_conversation_history;
                    self.prompt_data.add_conversation_message(format!("Agent: {}", response), max_history);

                    // Save agent response to session
                    let workspace_root_clone =
                        self.workspace_status.as_ref().and_then(|s| s.root.clone());
                    if let Ok(session_manager) =
                        crate::session_manager::SessionManager::new(workspace_root_clone)
                    {
                        let _ = session_manager.update_session(&session_id, &agent_id, &response);
                    }
                } else {
                    let error_msg = result.error.unwrap_or_else(|| "Unknown error".to_string());
                    let max_history = self.config.performance.max_conversation_history;
                    self.prompt_data.add_conversation_message(format!("Error: {}", error_msg), max_history);
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

        // Parse input to detect main command vs subcommand
        let parts: Vec<&str> = partial.split_whitespace().collect();

        let mut suggestions: Vec<String> = Vec::new();

        if parts.is_empty() || (parts.len() == 1 && !partial.ends_with(' ')) {
            // Typing main command - show matching main commands with fuzzy search
            let query = parts.first().unwrap_or(&"");

            // Use fuzzy matching for better UX
            let matcher = SkimMatcherV2::default();
            let mut scored_commands: Vec<(i64, String)> = Vec::new();

            // Score built-in commands with fuzzy matching
            for (cmd, desc) in &self.available_commands {
                if let Some(score) = matcher.fuzzy_match(cmd, query) {
                    scored_commands.push((score, format!("/{} - {}", cmd, desc)));
                }
            }

            // Score MCP commands with fuzzy matching
            for (cmd_name, prompt) in self.mcp_slash_registry.get_all_commands() {
                let cmd_without_slash = &cmd_name[1..]; // Remove leading '/'
                if let Some(score) = matcher.fuzzy_match(cmd_without_slash, query) {
                    let desc = prompt
                        .description
                        .as_ref()
                        .map(|d| d.as_str())
                        .unwrap_or("MCP command");
                    scored_commands.push((score, format!("{} - {}", cmd_name, desc)));
                }
            }

            // Sort by score (highest first) and take suggestions
            scored_commands.sort_by(|a, b| b.0.cmp(&a.0));
            suggestions.extend(scored_commands.into_iter().map(|(_, cmd)| cmd));
        } else if parts.len() >= 1 {
            // Main command is complete - show subcommands or arguments
            let main_cmd = parts[0];

            // First check for subcommands
            if let Some(subcommands) = self.available_subcommands.get(main_cmd) {
                let subquery = if parts.len() > 1 { parts[1] } else { "" };

                suggestions.extend(
                    subcommands
                        .iter()
                        .filter(|(subcmd, _desc)| subcmd.starts_with(subquery))
                        .map(|(subcmd, desc)| format!("/{} {} - {}", main_cmd, subcmd, desc))
                );
            } else {
                // No subcommands - check for dynamic argument completion
                match main_cmd {
                    "chat" if parts.len() <= 2 => {
                        // Suggest agent IDs for /chat command
                        if let Ok(agents) = crate::chat_executor::get_available_agents() {
                            let query = if parts.len() > 1 { parts[1] } else { "" };
                            suggestions.extend(
                                agents
                                    .iter()
                                    .filter(|(agent_id, _desc)| agent_id.to_lowercase().contains(&query.to_lowercase()))
                                    .map(|(agent_id, desc)| format!("/chat {} - {}", agent_id, desc))
                            );
                        }
                    }
                    _ => {
                        // No dynamic completion for this command
                    }
                }
            }
        }

        self.prompt_data.command_suggestions = suggestions;

        // Reset selection if list changed
        self.prompt_data.selected_suggestion_index = 0;
    }

    fn autocomplete_selected_command(&mut self) {
        if let Some(suggestion) =
            self.prompt_data.command_suggestions.get(self.prompt_data.selected_suggestion_index)
        {
            // Extract just the command part (before the ' - ')
            if let Some(cmd) = suggestion.split(" - ").next() {
                // Check if this is a main command with subcommands
                let cmd_without_slash = cmd.trim_start_matches('/');
                let parts: Vec<&str> = cmd_without_slash.split_whitespace().collect();

                let has_subcommands = if parts.len() == 1 {
                    self.available_subcommands.contains_key(parts[0])
                } else {
                    false
                };

                // Set the input
                self.prompt_data.input = if has_subcommands {
                    // Add a space to trigger subcommand suggestions
                    format!("{} ", cmd)
                } else {
                    // Use the command as-is
                    cmd.to_string()
                };

                // Clear suggestions - they'll be regenerated if needed
                self.prompt_data.command_suggestions.clear();
                self.prompt_data.selected_suggestion_index = 0;

                // If has subcommands, trigger update to show them
                if has_subcommands {
                    self.update_command_suggestions();
                }
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
                cmd.contains(query) || cmd.starts_with(query) || fuzzy_match(cmd, query)
            })
            .map(|(cmd, desc)| format!("/{} - {}", cmd, desc))
            .collect();
    }

    /// Handle input through orchestration
    async fn handle_orchestrated_input(&mut self, input: String) -> Result<()> {
        // Ensure service is initialized
        self.ensure_orchestration_service().await?;

        // Get or create session ID
        let session_id = self.current_session.clone().unwrap_or_else(|| {
            let id = format!("session_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
            self.current_session = Some(id.clone());
            id
        });

        let max_history = self.config.performance.max_conversation_history;

        // Add user message to conversation
        self.prompt_data.add_conversation_message(format!("You: {}", input), max_history);

        // Show thinking indicator
        self.prompt_data.add_conversation_message("🤔 Analyzing...".to_string(), max_history);

        // Record start time for elapsed time calculation
        let start_time = std::time::Instant::now();

        // Mark orchestration as running
        self.orchestration_running = true;

        // Execute orchestration
        if let Some(ref service) = self.orchestration_service {
            match service.handle_input(&session_id, &input).await {
                Ok(result) => {
                    // Remove thinking indicator
                    self.prompt_data.conversation.pop();

                    // Calculate elapsed time
                    let elapsed = start_time.elapsed();
                    let elapsed_secs = elapsed.as_secs_f64();
                    let elapsed_secs_u64 = elapsed.as_secs();

                    // Show timeout warnings if operation took too long
                    let max_history = self.config.performance.max_conversation_history;
                    if elapsed_secs_u64 >= 90 {
                        self.prompt_data.add_conversation_message(
                            "⏱️  Operation took longer than expected (approaching 120s timeout limit)".to_string(),
                            max_history
                        );
                    } else if elapsed_secs_u64 >= 30 {
                        self.prompt_data.add_conversation_message(
                            "⏰ Operation took longer than expected...".to_string(),
                            max_history
                        );
                    }

                    // Show tool calls if any
                    if !result.tool_calls.is_empty() {
                        let tool_count = result.tool_calls.len();
                        for (index, tool_call) in result.tool_calls.iter().enumerate() {
                            let max_history = self.config.performance.max_conversation_history;
                            if tool_count > 1 {
                                // Multi-agent workflow - show numbered steps
                                self.prompt_data.add_conversation_message(format!(
                                    "{}. 📋 Invoking: {}",
                                    index + 1,
                                    tool_call.name
                                ), max_history);
                            } else {
                                // Single agent - simple format
                                self.prompt_data.add_conversation_message(format!(
                                    "📋 Invoking: {}",
                                    tool_call.name
                                ), max_history);
                            }

                            // Show tool parameters (formatted nicely)
                            if let Some(args) = tool_call.arguments.as_object() {
                                if !args.is_empty() {
                                    let params_str = args.iter()
                                        .map(|(k, v)| {
                                            let v_str = if v.is_string() {
                                                v.as_str().unwrap_or("")
                                            } else {
                                                &serde_json::to_string(v).unwrap_or_default()
                                            };
                                            format!("{}: {}", k, v_str)
                                        })
                                        .collect::<Vec<_>>()
                                        .join(", ");
                                    if !params_str.is_empty() {
                                        self.prompt_data.add_conversation_message(
                                            format!("   Parameters: {}", params_str),
                                            max_history
                                        );
                                    }
                                }
                            }
                        }
                        let max_history = self.config.performance.max_conversation_history;
                        if tool_count > 1 {
                            self.prompt_data.add_conversation_message(format!(
                                "⏳ Executing {} agent{}...",
                                tool_count,
                                if tool_count == 1 { "" } else { "s" }
                            ), max_history);
                        } else {
                            self.prompt_data.add_conversation_message(
                                "⏳ Executing...".to_string(),
                                max_history
                            );
                        }
                    }

                    let max_history = self.config.performance.max_conversation_history;
                    // Add response
                    if !result.response.is_empty() {
                        self.prompt_data.add_conversation_message(format!("Assistant: {}", result.response), max_history);
                    }

                    // Show completion status
                    if result.is_success() {
                        self.prompt_data.add_conversation_message(format!(
                            "✅ Complete ({}s)",
                            format!("{:.2}", elapsed_secs)
                        ), max_history);
                    } else {
                        self.prompt_data.add_conversation_message(format!(
                            "⚠️  Finished with: {} ({}s)",
                            result.finish_reason,
                            format!("{:.2}", elapsed_secs)
                        ), max_history);
                    }
                }
                Err(e) => {
                    // Remove thinking indicator
                    self.prompt_data.conversation.pop();

                    let elapsed = start_time.elapsed();
                    let elapsed_secs = elapsed.as_secs_f64();
                    let elapsed_secs_u64 = elapsed.as_secs();

                    // Show timeout warnings if operation took too long
                    let max_history = self.config.performance.max_conversation_history;
                    if elapsed_secs_u64 >= 90 {
                        self.prompt_data.add_conversation_message(
                            "⏱️  Operation took longer than expected (approaching 120s timeout limit)".to_string(),
                            max_history
                        );
                    } else if elapsed_secs_u64 >= 30 {
                        self.prompt_data.add_conversation_message(
                            "⏰ Operation took longer than expected...".to_string(),
                            max_history
                        );
                    }

                    let max_history = self.config.performance.max_conversation_history;
                    self.prompt_data.add_conversation_message(format!(
                        "❌ Orchestration error: {} ({}s)",
                        e,
                        format!("{:.2}", elapsed_secs)
                    ), max_history);
                    self.prompt_data.add_conversation_message(
                        "💡 Tip: Use '/orchestrator toggle' to disable orchestration or '/orchestrator switch <provider>' to try a different provider.".to_string(),
                        max_history
                    );
                }
            }
        } else {
            // Remove thinking indicator
            self.prompt_data.conversation.pop();

            self.prompt_data
                .conversation
                .push("❌ Orchestration service not available".to_string());
            self.prompt_data.conversation.push(
                "💡 Tip: Use '/orchestrator toggle' to enable orchestration or check your API keys.".to_string()
            );
        }

        // Mark orchestration as complete
        self.orchestration_running = false;

        Ok(())
    }

    /// Handle /orchestrator command
    async fn handle_orchestrator_command(&mut self, args: &[String]) -> Result<()> {
        if args.is_empty() {
            // Show current configuration
            self.show_orchestrator_status();
            return Ok(());
        }

        match args[0].as_str() {
            "status" => {
                self.show_orchestrator_status();
            }
            "toggle" => {
                self.orchestration_enabled = !self.orchestration_enabled;
                
                // Save config state to workspace (or default path)
                let mut config = if let Ok(workspace) = radium_core::Workspace::discover() {
                    let workspace_config_path = workspace.structure().orchestration_config_file();
                    OrchestrationConfig::load_from_workspace_path(workspace_config_path)
                } else {
                    OrchestrationConfig::load_from_toml(OrchestrationConfig::default_config_path())
                        .unwrap_or_else(|_| OrchestrationConfig::default())
                };
                config.enabled = self.orchestration_enabled;
                let save_result = if let Ok(workspace) = radium_core::Workspace::discover() {
                    let workspace_config_path = workspace.structure().orchestration_config_file();
                    config.save_to_workspace_path(workspace_config_path)
                } else {
                    config.save_to_file(OrchestrationConfig::default_config_path())
                };
                if let Err(e) = save_result {
                    self.prompt_data.add_output(format!(
                        "⚠️  Failed to save config: {}",
                        e
                    ));
                }
                
                // Clear service if disabling
                if !self.orchestration_enabled {
                    self.orchestration_service = None;
                }
                
                self.prompt_data.add_output(format!(
                    "Orchestration {}",
                    if self.orchestration_enabled { "enabled" } else { "disabled" }
                ));
            }
            "switch" => {
                if args.len() < 2 {
                    self.prompt_data.add_output("Usage: /orchestrator switch <provider>".to_string());
                    self.prompt_data
                        .add_output("Available providers: gemini, claude, openai, prompt_based".to_string());
                } else {
                    self.switch_orchestrator_provider(&args[1]).await?;
                }
            }
            "config" => {
                self.show_orchestrator_config();
            }
            "refresh" => {
                self.refresh_orchestrator_tools().await?;
            }
            _ => {
                self.prompt_data.add_output(format!("Unknown orchestrator command: {}", args[0]));
                self.prompt_data.add_output("Available commands:".to_string());
                self.prompt_data.add_output("  /orchestrator          - Show status".to_string());
                self.prompt_data.add_output("  /orchestrator toggle   - Enable/disable".to_string());
                self.prompt_data
                    .add_output("  /orchestrator switch <provider>  - Switch provider".to_string());
                self.prompt_data
                    .add_output("  /orchestrator config   - Show full configuration".to_string());
                self.prompt_data
                    .add_output("  /orchestrator refresh  - Reload agent tool registry".to_string());
            }
        }

        Ok(())
    }

    /// Show orchestrator status
    fn show_orchestrator_status(&mut self) {
        self.prompt_data.add_output("Orchestration Status:".to_string());
        self.prompt_data.add_output("".to_string());
        self.prompt_data.add_output(format!(
            "  Enabled: {}",
            if self.orchestration_enabled { "✓ Yes" } else { "✗ No" }
        ));

        if let Some(ref service) = self.orchestration_service {
            self.prompt_data.add_output(format!("  Provider: {}", service.provider_name()));
            let config = service.config();
            self.prompt_data.add_output(format!("  Default: {}", config.default_provider));
        } else {
            self.prompt_data.add_output("  Service: Not initialized".to_string());
        }

        self.prompt_data.add_output("".to_string());
        self.prompt_data.add_output("Commands:".to_string());
        self.prompt_data.add_output("  /orchestrator          - Show status".to_string());
        self.prompt_data.add_output("  /orchestrator toggle   - Enable/disable orchestration".to_string());
        self.prompt_data
            .add_output("  /orchestrator switch <provider>  - Switch AI provider".to_string());
        self.prompt_data
            .add_output("  /orchestrator config   - Show full configuration".to_string());
        self.prompt_data
            .add_output("  /orchestrator refresh  - Reload agent tool registry".to_string());
    }

    /// Switch orchestrator provider
    async fn switch_orchestrator_provider(&mut self, provider_name: &str) -> Result<()> {
        use radium_orchestrator::ProviderType;

        // Parse provider name
        let provider_type = match provider_name.to_lowercase().as_str() {
            "gemini" => ProviderType::Gemini,
            "claude" => ProviderType::Claude,
            "openai" => ProviderType::OpenAI,
            "prompt_based" | "prompt-based" => ProviderType::PromptBased,
            _ => {
                self.prompt_data.add_output(format!("❌ Invalid provider: {}", provider_name));
                self.prompt_data.add_output("".to_string());
                self.prompt_data.add_output("Available providers:".to_string());
                self.prompt_data.add_output("  • gemini       - Google Gemini models".to_string());
                self.prompt_data.add_output("  • claude       - Anthropic Claude models".to_string());
                self.prompt_data.add_output("  • openai       - OpenAI GPT models".to_string());
                self.prompt_data
                    .add_output("  • prompt_based - Prompt-based fallback".to_string());
                return Ok(());
            }
        };

        // Create new configuration with selected provider
        let config = OrchestrationConfig::default().with_provider(provider_type);

        // Reinitialize service with new provider
        self.prompt_data.add_output(format!("🔄 Switching to {}...", provider_type));

        // Discover MCP tools if available
        let mcp_tools = if let Some(ref mcp_integration) = self.mcp_integration {
            use radium_core::mcp::orchestration_bridge::discover_mcp_tools_for_orchestration;
            discover_mcp_tools_for_orchestration(Arc::clone(mcp_integration))
                .await
                .ok()
        } else {
            None
        };

        match OrchestrationService::initialize(config.clone(), mcp_tools).await {
            Ok(service) => {
                self.orchestration_service = Some(Arc::new(service));
                
                // Save config to workspace (or default path)
                let save_result = if let Ok(workspace) = radium_core::Workspace::discover() {
                    let workspace_config_path = workspace.structure().orchestration_config_file();
                    config.save_to_workspace_path(workspace_config_path)
                } else {
                    config.save_to_file(OrchestrationConfig::default_config_path())
                };
                if let Err(e) = save_result {
                    self.prompt_data.add_output(format!(
                        "⚠️  Switched provider but failed to save config: {}",
                        e
                    ));
                }
                
                self.prompt_data.add_output(format!(
                    "✅ Switched to {} successfully",
                    self.orchestration_service.as_ref().unwrap().provider_name()
                ));

                // Show new configuration
                if let Some(ref svc) = self.orchestration_service {
                    let cfg = svc.config();
                    self.prompt_data.add_output("".to_string());
                    self.prompt_data.add_output("New configuration:".to_string());
                    self.prompt_data.add_output(format!("  Provider: {}", cfg.default_provider));

                    match provider_type {
                        ProviderType::Gemini => {
                            self.prompt_data.add_output(format!("  Model: {}", cfg.gemini.model));
                            self.prompt_data
                                .add_output(format!("  Temperature: {}", cfg.gemini.temperature));
                        }
                        ProviderType::Claude => {
                            self.prompt_data.add_output(format!("  Model: {}", cfg.claude.model));
                            self.prompt_data
                                .add_output(format!("  Temperature: {}", cfg.claude.temperature));
                        }
                        ProviderType::OpenAI => {
                            self.prompt_data.add_output(format!("  Model: {}", cfg.openai.model));
                            self.prompt_data
                                .add_output(format!("  Temperature: {}", cfg.openai.temperature));
                        }
                        ProviderType::PromptBased => {
                            self.prompt_data.add_output("  Model: prompt-based".to_string());
                            self.prompt_data
                                .add_output(format!("  Temperature: {}", cfg.prompt_based.temperature));
                        }
                    }
                }
            }
            Err(e) => {
                self.prompt_data.add_output(format!("❌ Failed to switch provider: {}", e));
                self.prompt_data.add_output("".to_string());
                self.prompt_data
                    .add_output("This could be due to:".to_string());
                self.prompt_data.add_output("  • Missing API key for the provider".to_string());
                self.prompt_data.add_output("  • Network connectivity issues".to_string());
                self.prompt_data.add_output("  • Invalid provider configuration".to_string());
                self.prompt_data.add_output("".to_string());
                self.prompt_data.add_output("Check your API keys:".to_string());
                self.prompt_data
                    .add_output("  • Gemini:  GEMINI_API_KEY environment variable".to_string());
                self.prompt_data
                    .add_output("  • Claude:  ANTHROPIC_API_KEY environment variable".to_string());
                self.prompt_data
                    .add_output("  • OpenAI:  OPENAI_API_KEY environment variable".to_string());
            }
        }

        Ok(())
    }

    /// Show full orchestrator configuration
    fn show_orchestrator_config(&mut self) {
        self.prompt_data.add_output("Orchestration Configuration:".to_string());
        self.prompt_data.add_output("".to_string());

        if let Some(ref service) = self.orchestration_service {
            let config = service.config();
            
            // General settings
            self.prompt_data.add_output("General:".to_string());
            self.prompt_data.add_output(format!("  Enabled: {}", if config.enabled { "Yes" } else { "No" }));
            self.prompt_data.add_output(format!("  Default Provider: {}", config.default_provider));
            self.prompt_data.add_output("".to_string());

            // Gemini configuration
            self.prompt_data.add_output("Gemini Provider:".to_string());
            self.prompt_data.add_output(format!("  Model: {}", config.gemini.model));
            self.prompt_data.add_output(format!("  Temperature: {:.2}", config.gemini.temperature));
            self.prompt_data.add_output(format!("  Max Tool Iterations: {}", config.gemini.max_tool_iterations));
            if let Some(endpoint) = &config.gemini.api_endpoint {
                self.prompt_data.add_output(format!("  API Endpoint: {}", endpoint));
            }
            self.prompt_data.add_output("".to_string());

            // Claude configuration
            self.prompt_data.add_output("Claude Provider:".to_string());
            self.prompt_data.add_output(format!("  Model: {}", config.claude.model));
            self.prompt_data.add_output(format!("  Temperature: {:.2}", config.claude.temperature));
            self.prompt_data.add_output(format!("  Max Tool Iterations: {}", config.claude.max_tool_iterations));
            self.prompt_data.add_output(format!("  Max Tokens: {}", config.claude.max_tokens));
            if let Some(endpoint) = &config.claude.api_endpoint {
                self.prompt_data.add_output(format!("  API Endpoint: {}", endpoint));
            }
            self.prompt_data.add_output("".to_string());

            // OpenAI configuration
            self.prompt_data.add_output("OpenAI Provider:".to_string());
            self.prompt_data.add_output(format!("  Model: {}", config.openai.model));
            self.prompt_data.add_output(format!("  Temperature: {:.2}", config.openai.temperature));
            self.prompt_data.add_output(format!("  Max Tool Iterations: {}", config.openai.max_tool_iterations));
            if let Some(endpoint) = &config.openai.api_endpoint {
                self.prompt_data.add_output(format!("  API Endpoint: {}", endpoint));
            }
            self.prompt_data.add_output("".to_string());

            // Prompt-based configuration
            self.prompt_data.add_output("Prompt-Based Provider:".to_string());
            self.prompt_data.add_output(format!("  Temperature: {:.2}", config.prompt_based.temperature));
            self.prompt_data.add_output(format!("  Max Tool Iterations: {}", config.prompt_based.max_tool_iterations));
            self.prompt_data.add_output("".to_string());

            // Fallback configuration
            self.prompt_data.add_output("Fallback:".to_string());
            self.prompt_data.add_output(format!("  Enabled: {}", if config.fallback.enabled { "Yes" } else { "No" }));
            if !config.fallback.chain.is_empty() {
                let chain_str = config.fallback.chain.iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                self.prompt_data.add_output(format!("  Chain: {}", chain_str));
            }
            self.prompt_data.add_output(format!("  Max Retries: {}", config.fallback.max_retries));
        } else {
            self.prompt_data.add_output("  Service: Not initialized".to_string());
            self.prompt_data.add_output("".to_string());
            self.prompt_data.add_output("Use '/orchestrator toggle' to enable orchestration.".to_string());
        }
    }

    /// Refresh orchestrator tool registry
    async fn refresh_orchestrator_tools(&mut self) -> Result<()> {
        self.prompt_data.add_output("🔄 Refreshing agent tool registry...".to_string());
        
        if let Some(ref service) = self.orchestration_service {
            match service.refresh_tools().await {
                Ok(()) => {
                    // Get count of tools after refresh
                    // Note: We can't easily get the count without exposing it from the service
                    // For now, just show success
                    self.prompt_data.add_output("✅ Agent tool registry refreshed successfully".to_string());
                    self.prompt_data.add_output("".to_string());
                    self.prompt_data.add_output("All available agents have been reloaded and are ready for use.".to_string());
                }
                Err(e) => {
                    self.prompt_data.add_output(format!("❌ Failed to refresh tool registry: {}", e));
                    self.prompt_data.add_output("".to_string());
                    self.prompt_data.add_output("This could be due to:".to_string());
                    self.prompt_data.add_output("  • Invalid agent configuration files".to_string());
                    self.prompt_data.add_output("  • File system permission issues".to_string());
                    self.prompt_data.add_output("  • Network issues (if loading from remote)".to_string());
                }
            }
        } else {
            self.prompt_data.add_output("⚠️  Orchestration service not initialized".to_string());
            self.prompt_data.add_output("".to_string());
            self.prompt_data.add_output("Use '/orchestrator toggle' to enable orchestration first.".to_string());
        }

        Ok(())
    }

    /// Handle /requirement command
    async fn handle_requirement(&mut self, req_id: &str, project_id: Option<String>) -> Result<()> {
        self.prompt_data.clear_output();
        self.prompt_data.add_output("🚀 Radium Autonomous Requirement Execution".to_string());
        self.prompt_data.add_output("".to_string());

        // Validate requirement ID format
        if !req_id.starts_with("REQ-") {
            self.prompt_data.add_output("❌ Invalid requirement ID format".to_string());
            self.prompt_data.add_output("   Expected format: REQ-XXX (e.g., REQ-173)".to_string());
            return Ok(());
        }

        // Get project ID from parameter or environment
        let project_id = project_id
            .or_else(|| std::env::var("BRAINGRID_PROJECT_ID").ok())
            .unwrap_or_else(|| {
                self.prompt_data.add_output("⚠️  No project ID specified, using default PROJ-14".to_string());
                "PROJ-14".to_string()
            });

        self.prompt_data.add_output(format!("📋 Configuration:"));
        self.prompt_data.add_output(format!("   Requirement ID: {}", req_id));
        self.prompt_data.add_output(format!("   Project ID: {}", project_id));
        self.prompt_data.add_output("".to_string());

        // Initialize workspace
        self.prompt_data.add_output("🔧 Initializing workspace...".to_string());
        let workspace = match radium_core::Workspace::discover() {
            Ok(ws) => {
                ws.ensure_structure()?;
                self.prompt_data.add_output("   ✓ Workspace initialized".to_string());
                ws
            }
            Err(e) => {
                self.prompt_data.add_output(format!("   ❌ Failed to discover workspace: {}", e));
                return Ok(());
            }
        };

        // Initialize database
        self.prompt_data.add_output("💾 Initializing database...".to_string());
        let db_path = workspace.radium_dir().join("database.db");
        let db = match Database::open(db_path.to_str().unwrap()) {
            Ok(database) => {
                self.prompt_data.add_output("   ✓ Database initialized".to_string());
                Arc::new(std::sync::Mutex::new(database))
            }
            Err(e) => {
                self.prompt_data.add_output(format!("   ❌ Failed to open database: {}", e));
                return Ok(());
            }
        };

        // Initialize orchestrator
        self.prompt_data.add_output("🎯 Initializing orchestrator...".to_string());
        let orchestrator = Arc::new(Orchestrator::new());

        // Initialize agent executor
        let executor = Arc::new(AgentExecutor::new(
            ModelType::Gemini,
            "gemini-2.0-flash-exp".to_string(),
        ));
        self.prompt_data.add_output("   ✓ Orchestrator initialized".to_string());

        // Initialize agent registry
        let agent_registry = Arc::new(AgentRegistry::new());

        // Initialize AI model
        self.prompt_data.add_output("🤖 Initializing AI model...".to_string());
        let config = ModelConfig {
            model_type: ModelType::Gemini,
            model_id: "gemini-2.0-flash-exp".to_string(),
            api_key: std::env::var("GEMINI_API_KEY").ok(),
        };
        let model = match ModelFactory::create(config) {
            Ok(m) => {
                self.prompt_data.add_output("   ✓ Model initialized (Gemini)".to_string());
                m
            }
            Err(e) => {
                self.prompt_data.add_output(format!("   ❌ Failed to create model: {}", e));
                self.prompt_data.add_output("   💡 Set GEMINI_API_KEY environment variable".to_string());
                return Ok(());
            }
        };

        // Create requirement executor
        self.prompt_data.add_output("⚙️  Creating requirement executor...".to_string());
        let executor_instance = match RequirementExecutor::new(
            project_id.clone(),
            &orchestrator,
            &executor,
            &db,
            agent_registry,
            model,
        ) {
            Ok(exec) => {
                self.prompt_data.add_output("   ✓ Executor created".to_string());
                exec
            }
            Err(e) => {
                self.prompt_data.add_output(format!("   ❌ Failed to create executor: {}", e));
                return Ok(());
            }
        };

        self.prompt_data.add_output("".to_string());
        self.prompt_data.add_output(format!("🚀 Starting async execution for {}...", req_id));
        self.prompt_data.add_output("   UI will remain responsive during execution".to_string());
        self.prompt_data.add_output("─".repeat(60));
        self.prompt_data.add_output("".to_string());

        // Create progress channel
        let (progress_tx, progress_rx) = tokio::sync::mpsc::channel(100);

        // Spawn async execution
        let req_id_clone = req_id.to_string();
        tokio::spawn(async move {
            let _ = executor_instance.execute_requirement_with_progress(&req_id_clone, progress_tx).await;
        });

        // Store active requirement for progress tracking
        self.active_requirement = Some(ActiveRequirement::new(req_id.to_string(), progress_rx));

        self.prompt_data.add_output("⏳ Execution started in background...".to_string());
        self.prompt_data.add_output("   Progress updates will appear below".to_string());

        Ok(())
    }

    /// Handle /requirement list command (list requirements)
    async fn handle_requirement_list(&mut self, project_id: Option<String>) -> Result<()> {
        use std::process::Command;
        use serde::{Deserialize};

        #[derive(Debug, Deserialize)]
        struct RequirementListResponse {
            requirements: Vec<RequirementSummary>,
            pagination: Pagination,
        }

        #[derive(Debug, Deserialize)]
        struct RequirementSummary {
            short_id: String,
            name: String,
            status: String,
            task_progress: TaskProgress,
        }

        #[derive(Debug, Deserialize)]
        struct TaskProgress {
            total: usize,
            completed: usize,
            progress_percentage: u8,
        }

        #[derive(Debug, Deserialize)]
        struct Pagination {
            page: usize,
            total: usize,
            total_pages: usize,
        }

        self.prompt_data.clear_output();
        self.prompt_data.add_output("📋 Braingrid Requirements List".to_string());
        self.prompt_data.add_output("".to_string());

        // Get project ID from parameter or environment
        let project_id = project_id
            .or_else(|| std::env::var("BRAINGRID_PROJECT_ID").ok())
            .unwrap_or_else(|| {
                self.prompt_data.add_output("⚠️  No project ID specified, using default PROJ-14".to_string());
                "PROJ-14".to_string()
            });

        self.prompt_data.add_output(format!("Project ID: {}", project_id));
        self.prompt_data.add_output("".to_string());

        // Call braingrid CLI
        self.prompt_data.add_output("Fetching requirements...".to_string());
        let output = match Command::new("braingrid")
            .args(&[
                "requirement",
                "list",
                "-p",
                &project_id,
                "--format",
                "json",
            ])
            .output()
        {
            Ok(out) => out,
            Err(e) => {
                self.prompt_data.add_output(format!("❌ Failed to execute braingrid command: {}", e));
                return Ok(());
            }
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            self.prompt_data.add_output(format!("❌ Failed to list requirements: {}", stderr));
            return Ok(());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Strip spinner animation and extract JSON (starts with '{')
        let json_start = stdout.find('{').unwrap_or(0);
        let json_str = &stdout[json_start..];

        let response: RequirementListResponse = match serde_json::from_str(json_str) {
            Ok(resp) => resp,
            Err(e) => {
                self.prompt_data.add_output(format!("❌ Failed to parse JSON: {}", e));
                return Ok(());
            }
        };

        self.prompt_data.add_output(format!("✓ Fetched {} requirements", response.requirements.len()));
        self.prompt_data.add_output("".to_string());

        // Display requirements table
        self.prompt_data.add_output("─".repeat(100));
        self.prompt_data.add_output(format!(
            "{:<12} {:<45} {:<15} {:<10} {:<10}",
            "ID", "Name", "Status", "Progress", "Tasks"
        ));
        self.prompt_data.add_output("─".repeat(100));

        for req in &response.requirements {
            let status_display = match req.status.as_str() {
                "COMPLETED" => format!("✓ {}", req.status),
                "IN_PROGRESS" => format!("⚙ {}", req.status),
                "REVIEW" => format!("👁 {}", req.status),
                "PLANNED" => format!("📝 {}", req.status),
                "IDEA" => format!("💡 {}", req.status),
                "CANCELLED" => format!("✗ {}", req.status),
                _ => req.status.clone(),
            };

            let progress_display = format!("{}%", req.task_progress.progress_percentage);
            let tasks_display = format!("{}/{}", req.task_progress.completed, req.task_progress.total);

            // Truncate name if too long
            let name = if req.name.len() > 43 {
                format!("{}...", &req.name[..40])
            } else {
                req.name.clone()
            };

            self.prompt_data.add_output(format!(
                "{:<12} {:<45} {:<15} {:<10} {:<10}",
                req.short_id, name, status_display, progress_display, tasks_display
            ));
        }

        self.prompt_data.add_output("─".repeat(100));
        self.prompt_data.add_output("".to_string());

        // Display pagination info
        self.prompt_data.add_output(format!(
            "Showing page {} of {} ({} total requirements)",
            response.pagination.page,
            response.pagination.total_pages,
            response.pagination.total
        ));
        self.prompt_data.add_output("".to_string());

        Ok(())
    }

    /// Handle /complete command
    async fn handle_complete(&mut self, source: &str) -> Result<()> {
        self.prompt_data.clear_output();
        self.prompt_data.add_output(format!("🚀 Starting completion workflow for: {}", source));
        self.prompt_data.add_output("".to_string());

        // Check if it's a Braingrid REQ
        if source.starts_with("REQ-") {
            self.prompt_data.add_output("📋 Detected Braingrid requirement".to_string());
            self.prompt_data.add_output(format!("   Redirecting to /requirement {}...", source));
            self.prompt_data.add_output("".to_string());
            self.prompt_data.add_output("💡 Tip: Use /requirement directly for Braingrid requirements".to_string());
            self.prompt_data.add_output("".to_string());

            // Delegate to handle_requirement
            return self.handle_requirement(source, None).await;
        }

        // For other sources (files, Jira, etc.), show not implemented message
        self.prompt_data.add_output("⚠️  Completion service for files and Jira is not yet implemented".to_string());
        self.prompt_data.add_output("".to_string());
        self.prompt_data.add_output("Currently supported:".to_string());
        self.prompt_data.add_output("  ✓ Braingrid requirements (REQ-XXX) - use /requirement command".to_string());
        self.prompt_data.add_output("".to_string());
        self.prompt_data.add_output("Coming soon:".to_string());
        self.prompt_data.add_output("  • File-based requirements (./specs/feature.md)".to_string());
        self.prompt_data.add_output("  • Jira ticket integration (RAD-42)".to_string());

        Ok(())
    }

    /// Handle /reload-config command
    async fn reload_config(&mut self) -> Result<()> {
        self.prompt_data.clear_output();
        self.prompt_data.add_output("🔄 Reloading configuration...".to_string());
        self.prompt_data.add_output("".to_string());

                match TuiConfig::reload() {
            Ok(config) => {
                // Reload theme from config
                let new_theme = RadiumTheme::from_config();
                self.theme = new_theme.clone();
                
                // Update global theme for views
                crate::theme::update_theme(new_theme);
                
                // Update config
                self.config = config.clone();
                
                self.prompt_data.add_output("✅ Configuration reloaded successfully!".to_string());
                self.prompt_data.add_output(format!("   Theme preset: {}", config.theme.preset));
                if config.theme.preset == "custom" {
                    self.prompt_data.add_output("   Using custom colors".to_string());
                }
                self.prompt_data.add_output(format!("   Max conversation history: {}", config.performance.max_conversation_history));
                self.prompt_data.add_output("   Theme updated (changes visible immediately)".to_string());
            }
            Err(e) => {
                self.prompt_data.add_output(format!("❌ Failed to reload configuration: {}", e));
                self.prompt_data.add_output("   Using default theme".to_string());
            }
        }

        Ok(())
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
