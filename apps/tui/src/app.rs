//! New unified prompt-based application.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use radium_core::auth::{CredentialStore, ProviderType};
use radium_core::mcp::{McpIntegration, SlashCommandRegistry};
use radium_core::agents::registry::AgentRegistry;
use radium_core::storage::Database;
use radium_models::{ModelFactory, ModelConfig, ModelType};
use radium_orchestrator::{OrchestrationConfig, OrchestrationService, AgentExecutor, Orchestrator};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::commands::{Command, DisplayContext};
use crate::components::{DialogManager, ExecutionDetailView, ExecutionHistoryView, SummaryView, ToastManager};
use crate::config::TuiConfig;
use crate::effects::AppEffectManager;
use crate::requirement_progress::{ActiveRequirement, ActiveRequirementProgress};
use crate::requirement_executor::RequirementExecutor as TuiRequirementExecutor;
use crate::progress_channel::ExecutionResult;
use crate::setup::SetupWizard;
use crate::state::{CheckpointInterruptState, CommandSuggestion, CommandSuggestionState, ExecutionHistory, ExecutionRecord, SuggestionSource, WorkflowUIState};
use crate::theme::RadiumTheme;
use crate::views::PromptData;
use crate::workspace::WorkspaceStatus;

/// Default maximum conversation history if config is unavailable
const _DEFAULT_MAX_CONVERSATION_HISTORY: usize = 500;

/// Execution view mode
#[derive(Debug, Clone)]
pub enum ExecutionView {
    /// No execution view active
    None,
    /// History view showing all executions
    History(ExecutionHistoryView),
    /// Detail view for a specific execution
    Detail(ExecutionDetailView),
    /// Summary view with aggregate statistics
    Summary(SummaryView),
}

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
    /// Active requirement execution (for async progress tracking - old system)
    pub active_requirement: Option<ActiveRequirement>,
    /// Active requirement execution using new ProgressMessage system
    pub active_requirement_progress: Option<ActiveRequirementProgress>,
    /// Join handle for active requirement task
    pub active_requirement_handle: Option<JoinHandle<Result<ExecutionResult, String>>>,
    /// Effect manager for animations
    pub effect_manager: AppEffectManager,
    /// Previous display context (for view transition detection)
    pub previous_context: Option<DisplayContext>,
    /// Previous dialog state (for dialog animation detection)
    pub previous_dialog_open: bool,
    /// Previous toast count (for toast animation detection)
    pub previous_toast_count: usize,
    /// Frame counter for spinner animations (increments on each render)
    pub spinner_frame: usize,
    /// Execution history tracking
    pub execution_history: ExecutionHistory,
    /// Active execution view (History, Detail, or Summary)
    pub active_execution_view: ExecutionView,
    /// Checkpoint interrupt state (when workflow pauses for user action)
    pub checkpoint_interrupt_state: Option<CheckpointInterruptState>,
    /// Task list state for tracking tasks with agent assignments
    pub task_list_state: Option<TaskListState>,
    /// Orchestrator thinking panel for displaying orchestrator logs
    pub orchestrator_panel: OrchestratorThinkingPanel,
    /// Last time task list was polled
    pub last_task_poll: Instant,
    /// Last time orchestrator logs were polled
    pub last_log_poll: Instant,
    /// Whether task panel is visible
    pub task_panel_visible: bool,
    /// Whether orchestrator panel is visible
    pub orchestrator_panel_visible: bool,
    /// Currently focused panel
    pub panel_focus: crate::views::orchestrator_view::PanelFocus,
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

        // Load execution history from disk
        let execution_history = if let Some(ref ws) = workspace_status {
            if let Some(ref root) = ws.root {
                crate::state::ExecutionHistory::load_from_file(
                    &crate::state::ExecutionHistory::default_history_path(root)
                )
            } else {
                crate::state::ExecutionHistory::new()
            }
        } else {
            crate::state::ExecutionHistory::new()
        };

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
        
        // Extract animation config before moving config
        let anim_enabled = config.animations.enabled;
        let anim_duration_mult = config.animations.duration_multiplier;
        let anim_reduced_motion = config.animations.reduced_motion;

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
            active_requirement_progress: None,
            active_requirement_handle: None,
            effect_manager: AppEffectManager::with_config(
                anim_enabled,
                anim_duration_mult,
                anim_reduced_motion,
            ),
            previous_context: None,
            previous_dialog_open: false,
            previous_toast_count: 0,
            execution_history,
            active_execution_view: ExecutionView::None,
            checkpoint_interrupt_state: None,
            task_list_state: None,
            orchestrator_panel: OrchestratorThinkingPanel::new(),
            last_task_poll: Instant::now(),
            last_log_poll: Instant::now(),
            task_panel_visible: true,
            orchestrator_panel_visible: true,
            panel_focus: crate::views::orchestrator_view::PanelFocus::Chat,
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
        // If checkpoint interrupt is active, handle interrupt input first
        if self.is_interrupt_active() {
            return self.handle_checkpoint_interrupt_key(key).await;
        }

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

        // Handle execution view keyboard input
        match &mut self.active_execution_view {
            ExecutionView::History(view) => {
                if let Some(action) = view.handle_key(crossterm::event::KeyEvent {
                    code: key,
                    modifiers,
                    kind: crossterm::event::KeyEventKind::Press,
                    state: crossterm::event::KeyEventState::NONE,
                }) {
                    match action {
                        crate::components::ExecutionHistoryAction::ViewDetail => {
                            if let Some(record) = view.get_selected_record() {
                                let detail_view = ExecutionDetailView::new(record.clone());
                                self.active_execution_view = ExecutionView::Detail(detail_view);
                            }
                        }
                    }
                    return Ok(());
                }
                // If Esc pressed, close view
                if key == KeyCode::Esc {
                    self.active_execution_view = ExecutionView::None;
                    return Ok(());
                }
            }
            ExecutionView::Detail(view) => {
                if let Some(action) = view.handle_key(crossterm::event::KeyEvent {
                    code: key,
                    modifiers,
                    kind: crossterm::event::KeyEventKind::Press,
                    state: crossterm::event::KeyEventState::NONE,
                }) {
                    match action {
                        crate::components::ExecutionDetailAction::Close => {
                            self.active_execution_view = ExecutionView::None;
                        }
                    }
                    return Ok(());
                }
            }
            ExecutionView::Summary(view) => {
                if let Some(action) = view.handle_key(crossterm::event::KeyEvent {
                    code: key,
                    modifiers,
                    kind: crossterm::event::KeyEventKind::Press,
                    state: crossterm::event::KeyEventState::NONE,
                }) {
                    match action {
                        crate::components::SummaryAction::Close => {
                            self.active_execution_view = ExecutionView::None;
                        }
                        crate::components::SummaryAction::Refresh => {
                            // Refresh will be handled by re-rendering with updated stats
                            // For now, just close and reopen if needed
                        }
                    }
                    return Ok(());
                }
            }
            ExecutionView::None => {
                // Handle execution view shortcuts when no view is active
                match key {
                    KeyCode::Char('h') | KeyCode::F(2) => {
                        // Toggle history view
                        if let Some(req_id) = self.get_current_requirement_id() {
                            let records: Vec<ExecutionRecord> = self.execution_history
                                .get_records_for_requirement(req_id)
                                .into_iter()
                                .cloned()
                                .collect();
                            let history_view = ExecutionHistoryView::new(records);
                            self.active_execution_view = ExecutionView::History(history_view);
                        }
                        return Ok(());
                    }
                    KeyCode::Char('s') => {
                        // Show summary view
                        if let Some(req_id) = self.get_current_requirement_id() {
                            let stats = self.execution_history.get_aggregate_stats(req_id);
                            let summary_view = SummaryView::new(stats, req_id.to_string());
                            self.active_execution_view = ExecutionView::Summary(summary_view);
                        }
                        return Ok(());
                    }
                    _ => {}
                }
            }
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
            KeyCode::Up if self.prompt_data.command_state.is_active && !self.prompt_data.command_state.suggestions.is_empty() => {
                self.prompt_data.command_state.select_previous();
            }
            KeyCode::Down if self.prompt_data.command_state.is_active && !self.prompt_data.command_state.suggestions.is_empty() => {
                self.prompt_data.command_state.select_next();
            }

            // PageUp/PageDown for faster navigation
            KeyCode::PageUp if self.prompt_data.command_state.is_active && !self.prompt_data.command_state.suggestions.is_empty() => {
                self.prompt_data.command_state.select_page_up();
            }
            KeyCode::PageDown if self.prompt_data.command_state.is_active && !self.prompt_data.command_state.suggestions.is_empty() => {
                self.prompt_data.command_state.select_page_down();
            }

            // Home/End to jump to first/last
            KeyCode::Home if self.prompt_data.command_state.is_active && !self.prompt_data.command_state.suggestions.is_empty() => {
                self.prompt_data.command_state.select_first();
            }
            KeyCode::End if self.prompt_data.command_state.is_active && !self.prompt_data.command_state.suggestions.is_empty() => {
                self.prompt_data.command_state.select_last();
            }

            // Tab to autocomplete selected command
            KeyCode::Tab if self.prompt_data.command_state.is_active && !self.prompt_data.command_state.suggestions.is_empty() => {
                self.autocomplete_selected_command();
            }

            // Escape to cancel command menu
            KeyCode::Esc if self.prompt_data.command_state.is_active => {
                self.prompt_data.command_state.clear();
            }

            // Enter - handle Cmd+Enter for submission, plain Enter for newline
            KeyCode::Enter if !self.prompt_data.command_palette_active => {
                // Check for Cmd/Ctrl+Enter for submission
                // On macOS: META (Cmd), on others: CONTROL (Ctrl)
                let is_submit = modifiers.contains(KeyModifiers::META) 
                    || modifiers.contains(KeyModifiers::CONTROL);
                
                if is_submit {
                    // Submit the input
                    if self.prompt_data.command_state.is_active && !self.prompt_data.command_state.suggestions.is_empty() {
                        // Autocomplete selected suggestion
                        self.autocomplete_selected_command();
                        // Then execute it
                        self.handle_enter().await?;
                    } else {
                        self.handle_enter().await?;
                    }
                } else {
                    // Plain Enter - insert newline via TextArea
                    self.prompt_data.input.handle_key(key, modifiers);
                    self.update_command_suggestions();
                }
            }

            // Backspace (unless in command palette) - delegate to TextArea
            KeyCode::Backspace if !self.prompt_data.command_palette_active => {
                self.prompt_data.input.handle_key(key, modifiers);
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
                self.prompt_data.command_palette_suggestions.clear();
                self.prompt_data.command_palette_selected_index = 0;
            }

            // Enter in command palette
            KeyCode::Enter if self.prompt_data.command_palette_active => {
                // Get the command to execute without holding a borrow
                let cmd_to_execute = {
                    let selected_idx = self.prompt_data.command_palette_selected_index;
                    self.prompt_data
                        .command_palette_suggestions
                        .get(selected_idx)
                        .and_then(|s| s.split(" - ").next().map(|s| s.to_string()))
                };
                
                if let Some(cmd) = cmd_to_execute {
                    self.prompt_data.set_input(&cmd);
                    self.prompt_data.command_palette_active = false;
                    self.prompt_data.command_palette_query.clear();
                    // Execute the command
                    self.handle_enter().await?;
                }
            }

            // Arrow keys in command palette
            KeyCode::Up if self.prompt_data.command_palette_active => {
                self.prompt_data.command_state.select_previous();
            }
            KeyCode::Down if self.prompt_data.command_palette_active => {
                self.prompt_data.command_state.select_next();
            }

            // Backspace in command palette
            KeyCode::Backspace if self.prompt_data.command_palette_active => {
                self.prompt_data.command_palette_query.pop();
                self.update_command_palette();
            }

            // Regular characters - delegate to TextArea for normal input
            KeyCode::Char(c) if !modifiers.contains(KeyModifiers::CONTROL) => {
                if self.prompt_data.command_palette_active {
                    self.prompt_data.command_palette_query.push(c);
                    self.update_command_palette();
                } else {
                    // Delegate to TextArea for text input
                    self.prompt_data.input.handle_key(key, modifiers);
                    self.update_command_suggestions();
                }
            }

            // Delegate other navigation/editing keys to TextArea when not in special modes
            KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down
            | KeyCode::Delete | KeyCode::Tab
            if !self.prompt_data.command_palette_active 
                && !self.dialog_manager.is_open()
                && self.prompt_data.command_state.suggestions.is_empty() => {
                self.prompt_data.input.handle_key(key, modifiers);
            }

            _ => {}
        }

        Ok(())
    }

    async fn handle_enter(&mut self) -> Result<()> {
        let input = self.prompt_data.input_text();
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

        // Show basic stats (centered)
        self.prompt_data.add_output("".to_string());
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
        let input = self.prompt_data.input.text();

        // Only show suggestions if typing a slash command
        if !input.starts_with('/') {
            self.prompt_data.command_state.clear();
            return;
        }

        let partial = &input[1..]; // Remove the '/'
        let trigger_mode = self.config.completion.trigger_mode_enum();
        let min_chars = self.config.completion.min_chars;

        // Check trigger mode
        match trigger_mode {
            TriggerMode::Auto => {
                // Auto mode: trigger if input length >= min_chars
                if partial.len() < min_chars {
                    self.prompt_data.command_state.clear();
                    return;
                }
            }
            TriggerMode::Manual => {
                // Manual mode: only update if manually triggered
                if !self.prompt_data.command_state.triggered_manually {
                    return;
                }
            }
        }

        // Mark auto-completion as active
        self.prompt_data.command_state.is_active = true;

        // Parse input to detect main command vs subcommand
        let parts: Vec<&str> = partial.split_whitespace().collect();

        let mut suggestions: Vec<CommandSuggestion> = Vec::new();

        if parts.is_empty() || (parts.len() == 1 && !partial.ends_with(' ')) {
            // Typing main command - show matching main commands with fuzzy search
            let query = parts.first().unwrap_or(&"");

            // REQ-198: Performance optimization - use fuzzy matching for efficient filtering
            // SkimMatcherV2 is optimized for <50ms latency even with 100+ commands
            let matcher = SkimMatcherV2::default();
            let mut scored_commands: Vec<(i64, String, String, SuggestionSource)> = Vec::new();

            // Score built-in commands with fuzzy matching
            for (cmd, desc) in &self.available_commands {
                if let Some(score) = matcher.fuzzy_match(cmd, query) {
                    scored_commands.push((score, cmd.to_string(), desc.to_string(), SuggestionSource::BuiltIn));
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
                    scored_commands.push((score, cmd_name.clone(), desc.to_string(), SuggestionSource::MCP));
                }
            }

            // Sort by score (highest first) and take top suggestions
            // Limit to reasonable number for performance (REQ-198: <50ms requirement)
            scored_commands.sort_by(|a, b| b.0.cmp(&a.0));
            const MAX_SUGGESTIONS: usize = 50; // Limit suggestions for performance
            suggestions.extend(
                scored_commands
                    .into_iter()
                    .take(MAX_SUGGESTIONS)
                    .map(|(score, cmd, desc, source)| CommandSuggestion::new(cmd, desc, score, source))
            );
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
                        .map(|(subcmd, desc)| {
                            CommandSuggestion::new(
                                format!("/{} {}", main_cmd, subcmd),
                                desc.to_string(),
                                100, // High score for exact prefix match
                                SuggestionSource::BuiltIn,
                            )
                        })
                );
            } else {
                // No subcommands - check for dynamic argument completion
                match main_cmd {
                    "chat" if parts.len() <= 2 => {
                        // Suggest agent IDs for /chat command with fuzzy matching
                        if let Ok(agents) = crate::chat_executor::get_available_agents() {
                            let query = if parts.len() > 1 { parts[1] } else { "" };
                            
                            if query.len() >= 2 {
                                // Use fuzzy matching for better discovery
                                let matcher = SkimMatcherV2::default();
                                let mut scored_agents: Vec<(i64, (String, String))> = Vec::new();
                                
                                for (agent_id, desc) in &agents {
                                    // Try matching on both ID and description
                                    let id_score = matcher.fuzzy_match(agent_id, query).unwrap_or(0);
                                    let desc_score = matcher.fuzzy_match(desc, query).unwrap_or(0);
                                    let best_score = id_score.max(desc_score);
                                    
                                    if best_score > 0 {
                                        scored_agents.push((best_score, (agent_id.clone(), desc.clone())));
                                    }
                                }
                                
                                // Sort by score and take top matches
                                scored_agents.sort_by(|a, b| b.0.cmp(&a.0));
                                suggestions.extend(
                                    scored_agents
                                        .into_iter()
                                        .take(20) // Limit to top 20 agents
                                        .map(|(score, (agent_id, desc))| {
                                            CommandSuggestion::new_parameter(
                                                format!("/chat {}", agent_id),
                                                desc,
                                                score,
                                                SuggestionSource::Agent,
                                                "agent-id".to_string(),
                                            )
                                        })
                                );
                            } else {
                                // Show all agents if query is too short
                                suggestions.extend(
                                    agents
                                        .iter()
                                        .take(20)
                                        .map(|(agent_id, desc)| {
                                            CommandSuggestion::new_parameter(
                                                format!("/chat {}", agent_id),
                                                desc.clone(),
                                                50, // Default score
                                                SuggestionSource::Agent,
                                                "agent-id".to_string(),
                                            )
                                        })
                                );
                            }
                        }
                    }
                    _ => {
                        // No dynamic completion for this command
                    }
                }
            }
        }

        // Update state with new suggestions
        self.prompt_data.command_state.set_suggestions(suggestions);
    }

    fn autocomplete_selected_command(&mut self) {
        if let Some(suggestion) = self.prompt_data.command_state.get_selected() {
            // Extract just the command part
            let cmd = &suggestion.command;
            
            // Check if this is a main command with subcommands
            let cmd_without_slash = cmd.trim_start_matches('/');
            let parts: Vec<&str> = cmd_without_slash.split_whitespace().collect();

            let has_subcommands = if parts.len() == 1 {
                self.available_subcommands.contains_key(parts[0])
            } else {
                false
            };

            // Set the input
            let input_text = if has_subcommands {
                // Add a space to trigger subcommand suggestions
                format!("{} ", cmd)
            } else {
                // Use the command as-is
                cmd.clone()
            };
            self.prompt_data.set_input(&input_text);

            // Clear suggestions - they'll be regenerated if needed
            self.prompt_data.command_state.clear();

            // If has subcommands, trigger update to show them
            if has_subcommands {
                self.update_command_suggestions();
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
        let executor_instance = TuiRequirementExecutor::new(
            project_id.clone(),
            orchestrator,
            executor,
            db,
            agent_registry,
            model,
        );

        self.prompt_data.add_output("   ✓ Executor created".to_string());
        self.prompt_data.add_output("".to_string());
        self.prompt_data.add_output(format!("🚀 Starting async execution for {}...", req_id));
        self.prompt_data.add_output("   UI will remain responsive during execution".to_string());
        self.prompt_data.add_output("─".repeat(60));
        self.prompt_data.add_output("".to_string());

        // Spawn async execution task using new system
        let (task_handle, progress_rx) = executor_instance.spawn_requirement_task(req_id.to_string());

        // Store active requirement progress tracking and task handle
        self.active_requirement_progress = Some(ActiveRequirementProgress::new(req_id.to_string(), progress_rx));
        self.active_requirement_handle = Some(task_handle);

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

    /// Gets the current requirement ID from active requirement progress.
    fn get_current_requirement_id(&self) -> Option<&str> {
        self.active_requirement_progress
            .as_ref()
            .map(|p| p.requirement_id.as_str())
    }

    /// Activates a checkpoint interrupt.
    pub fn activate_checkpoint_interrupt(&mut self, state: CheckpointInterruptState) {
        // Store current display context if not already stored
        if self.previous_context.is_none() {
            self.previous_context = Some(self.prompt_data.context.clone());
        }

        // Set interrupt state
        self.checkpoint_interrupt_state = Some(state.clone());

        // Transition to Checkpoint display context
        let checkpoint_id = state.checkpoint_id.clone().unwrap_or_else(|| "unknown".to_string());
        let reason = match &state.trigger {
            crate::state::InterruptTrigger::AgentCheckpoint { reason, .. } => reason.clone(),
            crate::state::InterruptTrigger::PolicyAskUser { reason, .. } => reason.clone(),
            crate::state::InterruptTrigger::Error { message } => message.clone(),
        };
        let agent_id = match &state.trigger {
            crate::state::InterruptTrigger::AgentCheckpoint { agent_id, .. } => Some(agent_id.clone()),
            _ => None,
        };

        self.prompt_data.context = DisplayContext::Checkpoint {
            checkpoint_id,
            reason,
            agent_id,
        };
    }

    /// Deactivates the checkpoint interrupt.
    pub fn deactivate_checkpoint_interrupt(&mut self) {
        self.checkpoint_interrupt_state = None;

        // Restore previous context if available, otherwise default to Help
        if let Some(prev_context) = self.previous_context.take() {
            self.prompt_data.context = prev_context;
        } else {
            self.prompt_data.context = DisplayContext::Help;
        }
    }

    /// Checks if a checkpoint interrupt is currently active.
    pub fn is_interrupt_active(&self) -> bool {
        self.checkpoint_interrupt_state
            .as_ref()
            .map(|s| s.is_active())
            .unwrap_or(false)
    }

    /// Handles keyboard input for checkpoint interrupt modal.
    async fn handle_checkpoint_interrupt_key(&mut self, key: KeyCode) -> Result<()> {
        let interrupt_state = match &mut self.checkpoint_interrupt_state {
            Some(state) if state.is_active() => state,
            _ => return Ok(()),
        };

        match key {
            KeyCode::Tab => {
                interrupt_state.select_next_action();
            }
            KeyCode::BackTab => {
                interrupt_state.select_previous_action();
            }
            KeyCode::Enter => {
                // Confirm selected action
                if let Some(action) = interrupt_state.get_selected_action().cloned() {
                    match action {
                        InterruptAction::Continue => {
                            // Check if this is a policy interrupt (Approve)
                            let is_policy_approve = matches!(
                                interrupt_state.trigger,
                                crate::state::InterruptTrigger::PolicyAskUser { .. }
                            );
                            
                            if is_policy_approve {
                                self.toast_manager.info("Tool execution approved".to_string());
                            } else {
                                self.toast_manager.info("Continuing workflow execution...".to_string());
                            }
                            
                            // Resume workflow if paused
                            if let Some(ref mut workflow_state) = self.workflow_state {
                                workflow_state.resume();
                            }
                            
                            self.deactivate_checkpoint_interrupt();
                        }
                        InterruptAction::Rollback { checkpoint_id } => {
                            // Show confirmation dialog
                            use crate::components::DialogChoice;
                            let cp_id = checkpoint_id.clone();
                            let choices = vec![
                                DialogChoice::new("Yes".to_string(), "yes".to_string()),
                                DialogChoice::new("No".to_string(), "no".to_string()),
                            ];
                            self.dialog_manager.show_select_menu(
                                format!("Restore to checkpoint {}? This will discard current changes.", checkpoint_id),
                                choices,
                            );
                            // Store checkpoint_id for rollback - will execute rollback when dialog confirms
                            // For now, we'll handle this in the main event loop when dialog returns "yes"
                        }
                        InterruptAction::Cancel => {
                            // Show confirmation dialog
                            use crate::components::DialogChoice;
                            let choices = vec![
                                DialogChoice::new("Yes".to_string(), "yes".to_string()),
                                DialogChoice::new("No".to_string(), "no".to_string()),
                            ];
                            let workflow_id = interrupt_state.workflow_id.clone();
                            self.dialog_manager.show_select_menu(
                                "Cancel workflow execution? This cannot be undone.".to_string(),
                                choices,
                            );
                            // Store workflow_id for dialog callback - will be handled in main event loop
                            // For now, Cancel will be handled when dialog confirms
                        }
                    }
                }
            }
            KeyCode::Char('d') => {
                interrupt_state.toggle_details();
            }
            KeyCode::Up if interrupt_state.show_diff => {
                // Scroll diff view up
                interrupt_state.diff_scroll_offset = interrupt_state.diff_scroll_offset.saturating_sub(1);
            }
            KeyCode::Down if interrupt_state.show_diff => {
                // Scroll diff view down
                interrupt_state.diff_scroll_offset = interrupt_state.diff_scroll_offset.saturating_add(1);
            }
            KeyCode::Char('g') => {
                interrupt_state.toggle_diff();
                
                // Fetch diff if toggling on and not already fetched
                if interrupt_state.show_diff && interrupt_state.diff_data.is_none() {
                    use radium_core::workspace::Workspace;
                    use radium_core::checkpoint::CheckpointManager;
                    
                    if let Ok(workspace) = Workspace::discover() {
                        if let Ok(checkpoint_manager) = CheckpointManager::new(workspace.root()) {
                            if let Err(e) = interrupt_state.fetch_diff(&checkpoint_manager) {
                                self.toast_manager.warning(format!("Failed to fetch diff: {}", e));
                            }
                        }
                    }
                }
            }
            KeyCode::Esc => {
                // Only allow Esc if no destructive action is in progress
                // For now, just close the modal
                self.deactivate_checkpoint_interrupt();
            }
            _ => {
                // Other keys are ignored
            }
        }

        Ok(())
    }

    /// Activates a policy AskUser interrupt.
    /// This should be called when the policy engine returns an AskUser decision.
    pub fn activate_policy_interrupt(
        &mut self,
        workflow_id: &str,
        step_number: usize,
        tool_name: String,
        args: String,
        reason: String,
    ) -> Result<()> {
        use crate::state::{CheckpointInterruptState, InterruptTrigger};

        // Only activate if not already in interrupt state
        if self.is_interrupt_active() {
            return Ok(());
        }

        let trigger = InterruptTrigger::PolicyAskUser {
            tool_name,
            args,
            reason,
        };

        let mut interrupt_state = CheckpointInterruptState::new(
            trigger,
            workflow_id.to_string(),
            step_number,
        );

        // Policy interrupts don't have checkpoints
        interrupt_state.activate(None);

        // Pause workflow if it's running
        if let Some(ref mut workflow_state) = self.workflow_state {
            workflow_state.pause();
        }

        self.activate_checkpoint_interrupt(interrupt_state);
        Ok(())
    }

    /// Checks for checkpoint behavior and activates interrupt if detected.
    /// This should be called periodically during workflow execution.
    pub fn check_for_checkpoint_behavior(&mut self, workflow_id: &str, step_number: usize, agent_id: Option<&str>) -> Result<bool> {
        use radium_core::workspace::{Workspace, WorkspaceStructure};
        use radium_core::workflow::behaviors::checkpoint::CheckpointEvaluator;
        use std::path::Path;

        // Only check if not already in interrupt state
        if self.is_interrupt_active() {
            return Ok(false);
        }

        // Get workspace
        let workspace = match Workspace::discover() {
            Ok(ws) => ws,
            Err(_) => return Ok(false),
        };

        let ws_structure = WorkspaceStructure::new(workspace.root());
        let behavior_file = ws_structure.memory_dir().join("behavior.json");

        if !behavior_file.exists() {
            return Ok(false);
        }

        // Check for checkpoint behavior
        let evaluator = CheckpointEvaluator::new();
        let output = ""; // We don't have the output here, but it's not used by CheckpointEvaluator
        match evaluator.evaluate_checkpoint(&behavior_file, output) {
            Ok(Some(decision)) if decision.should_stop_workflow => {
                // Checkpoint detected - activate interrupt
                use crate::state::{CheckpointInterruptState, InterruptTrigger};

                let trigger = if let Some(agent_id) = agent_id {
                    InterruptTrigger::AgentCheckpoint {
                        reason: decision.reason.unwrap_or_else(|| "Checkpoint triggered".to_string()),
                        agent_id: agent_id.to_string(),
                    }
                } else {
                    InterruptTrigger::AgentCheckpoint {
                        reason: decision.reason.unwrap_or_else(|| "Checkpoint triggered".to_string()),
                        agent_id: "unknown".to_string(),
                    }
                };

                let mut interrupt_state = CheckpointInterruptState::new(
                    trigger,
                    workflow_id.to_string(),
                    step_number,
                );

                // Get available checkpoints from workflow state if available
                if let Some(ref workflow_state) = self.workflow_state {
                    let checkpoint_ids: Vec<String> = workflow_state
                        .checkpoint
                        .checkpoints
                        .iter()
                        .map(|cp| cp.id.clone())
                        .collect();
                    interrupt_state.available_checkpoints = checkpoint_ids;
                    
                    // Set current checkpoint ID if available
                    if let Some(current_cp) = workflow_state.checkpoint.current_checkpoint.as_ref() {
                        interrupt_state.checkpoint_id = Some(current_cp.clone());
                    }
                }

                // Activate interrupt
                interrupt_state.activate(interrupt_state.checkpoint_id.clone());

                // Pause workflow if it's running
                if let Some(ref mut workflow_state) = self.workflow_state {
                    workflow_state.pause();
                }

                self.activate_checkpoint_interrupt(interrupt_state);
                Ok(true)
            }
            Ok(_) => Ok(false),
            Err(_) => Ok(false),
        }
    }

    /// Executes checkpoint rollback to restore workspace to a previous checkpoint.
    pub fn execute_checkpoint_rollback(&mut self, checkpoint_id: String) -> Result<()> {
        use radium_core::workspace::Workspace;
        use radium_core::checkpoint::CheckpointManager;

        // Get workspace
        let workspace = Workspace::discover()
            .map_err(|e| anyhow::anyhow!("Failed to discover workspace: {}", e))?;

        // Create checkpoint manager
        let checkpoint_manager = CheckpointManager::new(workspace.root())
            .map_err(|e| anyhow::anyhow!("Failed to create checkpoint manager: {}", e))?;

        // Verify checkpoint exists
        checkpoint_manager
            .get_checkpoint(&checkpoint_id)
            .map_err(|e| anyhow::anyhow!("Checkpoint not found: {}", e))?;

        // Execute rollback
        checkpoint_manager
            .restore_checkpoint(&checkpoint_id)
            .map_err(|e| anyhow::anyhow!("Failed to restore checkpoint: {}", e))?;

        // Show success toast
        self.toast_manager.success(format!("Checkpoint {} restored successfully", checkpoint_id));

        // Update workflow state to reflect rollback
        if let Some(ref mut workflow_state) = self.workflow_state {
            // Find the checkpoint and reset workflow to that step
            if let Some(checkpoint) = workflow_state.checkpoint.get_checkpoint(&checkpoint_id) {
                workflow_state.current_step = checkpoint.step_number;
                workflow_state.pause(); // Pause after rollback - user can resume
            }
        }

        // Deactivate interrupt modal
        self.deactivate_checkpoint_interrupt();

        Ok(())
    }

    /// Polls task list from gRPC server and updates task list state.
    /// This method should be called periodically (every 500ms) when workflow is active.
    pub async fn poll_task_list(&mut self) -> Result<()> {
        // Check if 500ms has elapsed since last poll
        let elapsed = self.last_task_poll.elapsed();
        if elapsed.as_millis() < 500 {
            return Ok(());
        }

        // Try to connect to gRPC server and poll tasks
        // For now, we'll attempt to connect but handle errors gracefully
        // The actual connection will be established when needed
        match self.try_poll_tasks_from_grpc().await {
            Ok(tasks) => {
                // Update task list state
                if self.task_list_state.is_none() {
                    self.task_list_state = Some(TaskListState::new());
                }
                
                if let Some(ref mut task_state) = self.task_list_state {
                    // Clear and rebuild task list
                    task_state.clear();
                    for (idx, task) in tasks.iter().enumerate() {
                        task_state.add_task(
                            task.id.clone(),
                            task.name.clone(),
                            task.state.clone(),
                            task.agent_id.clone(),
                            idx,
                        );
                    }
                }
                
                self.last_task_poll = Instant::now();
            }
            Err(e) => {
                // Log error to orchestrator panel but don't fail
                self.orchestrator_panel.append_log(format!(
                    "[Orchestrator] Error polling tasks: {}. Retrying in 2s...",
                    e
                ));
                // Retry after 2s backoff
                self.last_task_poll = Instant::now() - std::time::Duration::from_millis(1500);
            }
        }

        Ok(())
    }

    /// Attempts to poll tasks from gRPC server.
    async fn try_poll_tasks_from_grpc(&self) -> Result<Vec<radium_core::models::task::Task>> {
        use radium_core::radium_client::RadiumClientManager;
        use radium_core::proto::{ListTasksRequest, ListTasksResponse};
        use tonic::Request;
        use radium_core::models::task::Task;

        // Try to get a gRPC client
        let mut client_manager = RadiumClientManager::new();
        let mut client = client_manager.connect().await
            .map_err(|e| anyhow::anyhow!("Failed to connect to gRPC server: {}", e))?;

        // Call ListTasks RPC
        let request = Request::new(ListTasksRequest {});
        let response = client.list_tasks(request).await
            .map_err(|e| anyhow::anyhow!("gRPC call failed: {}", e))?;

        let proto_tasks = response.into_inner().tasks;

        // Convert proto tasks to Task models
        let tasks: Result<Vec<Task>, _> = proto_tasks
            .into_iter()
            .map(|proto_task| Task::try_from(proto_task))
            .collect();

        tasks.map_err(|e| anyhow::anyhow!("Failed to parse tasks: {}", e))
    }

    /// Polls orchestrator logs from monitoring service.
    /// This method should be called periodically (every 500ms) when orchestrator is active.
    pub async fn poll_orchestrator_logs(&mut self) -> Result<()> {
        // Check if 500ms has elapsed since last poll
        let elapsed = self.last_log_poll.elapsed();
        if elapsed.as_millis() < 500 {
            return Ok(());
        }

        // For now, we'll add a placeholder that can be extended later
        // The orchestrator logs can come from monitoring service or a new gRPC method
        // For now, we'll just update the timestamp to prevent excessive polling
        self.last_log_poll = Instant::now();

        // TODO: Implement actual orchestrator log polling
        // This could query monitoring service for logs with agent_type="orchestrator"
        // or use a new StreamOrchestratorLogs RPC method

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
