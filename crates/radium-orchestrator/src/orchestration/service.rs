// Orchestration service for TUI integration
//
// Manages orchestration provider lifecycle, session state, and request handling
// to enable natural conversation in the TUI without requiring explicit commands.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};

use super::{
    OrchestrationProvider, OrchestrationResult,
    agent_tools::AgentToolRegistry,
    code_analysis_tool,
    config::{OrchestrationConfig, ProviderType},
    context::{Message, OrchestrationContext},
    context_loader::ContextFileLoaderTrait,
    engine::{EngineConfig, OrchestrationEngine},
    events::OrchestrationEvent,
    file_tools::{self, WorkspaceRootProvider as FileWorkspaceRootProvider},
    git_extended_tools,
    hooks::ToolHookExecutor,
    definition_tool,
    project_scan_tool,
    search_tool,
    symbol_search_tool,
    terminal_tool::{self, WorkspaceRootProvider as TerminalWorkspaceRootProvider, SandboxManager as TerminalSandboxManager},
    tool::Tool,
    providers::{
        claude::ClaudeOrchestrator, gemini::GeminiOrchestrator, openai::OpenAIOrchestrator,
        prompt_based::PromptBasedOrchestrator,
    },
};
use crate::error::Result;
use std::path::PathBuf;

/// Session state for maintaining conversation context
#[derive(Debug, Clone)]
pub struct SessionState {
    /// Unique session identifier
    pub session_id: String,
    /// Conversation history
    pub conversation_history: Vec<Message>,
    /// Agents invoked in this session
    pub invoked_agents: Vec<String>,
    /// Session creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl SessionState {
    /// Create a new session
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            conversation_history: Vec::new(),
            invoked_agents: Vec::new(),
            created_at: chrono::Utc::now(),
        }
    }

    /// Get orchestration context from session state
    pub fn to_context(&self) -> OrchestrationContext {
        let mut context = OrchestrationContext::new(&self.session_id);
        context.conversation_history = self.conversation_history.clone();
        context
    }

    /// Update session from orchestration context
    pub fn update_from_context(&mut self, context: &OrchestrationContext) {
        self.conversation_history = context.conversation_history.clone();
    }
}

/// Orchestration service managing provider lifecycle and request handling
pub struct OrchestrationService {
    /// Current configuration
    config: OrchestrationConfig,
    /// Tool registry for agent discovery
    tool_registry: Arc<RwLock<AgentToolRegistry>>,
    /// Active sessions
    sessions: Arc<RwLock<HashMap<String, SessionState>>>,
    /// Current provider
    provider: Arc<dyn OrchestrationProvider>,
    /// Orchestration engine
    engine: Arc<OrchestrationEngine>,
    /// Broadcast sender for orchestration events (tokens/tool calls/progress/approvals)
    event_tx: broadcast::Sender<OrchestrationEvent>,
    /// Context file loader (optional)
    context_loader: Option<Arc<dyn ContextFileLoaderTrait>>,
    /// Workspace root for context loading
    workspace_root: Option<PathBuf>,
}

impl OrchestrationService {
    /// Initialize orchestration service with configuration
    ///
    /// # Arguments
    /// * `config` - Orchestration configuration
    /// * `mcp_tools` - Optional list of MCP tools to include (initialized at application level)
    /// * `workspace_root` - Optional workspace root path for file operations
    /// * `sandbox_manager` - Optional sandbox manager for terminal command execution
    /// * `context_loader` - Optional context file loader for loading GEMINI.md files
    /// * `hook_executor` - Optional hook executor for BeforeTool/AfterTool hooks
    pub async fn initialize(
        config: OrchestrationConfig,
        mcp_tools: Option<Vec<Tool>>,
        workspace_root: Option<PathBuf>,
        sandbox_manager: Option<Arc<dyn TerminalSandboxManager>>,
        context_loader: Option<Arc<dyn ContextFileLoaderTrait>>,
        hook_executor: Option<Arc<dyn ToolHookExecutor>>,
    ) -> Result<Self> {
        // Event stream channel (consumed by CLI/TUI/daemon clients)
        let (event_tx, _) = broadcast::channel(1024);

        // Initialize tool registry
        let mut tool_registry = AgentToolRegistry::new();
        tool_registry.load_agents()?;
        let tool_registry = Arc::new(RwLock::new(tool_registry));

        // Collect all tools
        let mut tools = Vec::new();

        // Add file operation tools if workspace root is provided
        if let Some(ref root) = workspace_root {
            let workspace_provider: Arc<dyn FileWorkspaceRootProvider> = Arc::new(SimpleWorkspaceRootProvider {
                root: root.clone(),
            });
            let file_tools = file_tools::create_file_operation_tools(workspace_provider.clone());
            tools.extend(file_tools);
            tracing::info!("Added {} file operation tools to orchestration", 6);

            // Add project analysis tools (project_scan)
            let project_tools = project_scan_tool::create_project_analysis_tools(workspace_provider.clone());
            let project_tool_count = project_tools.len();
            tools.extend(project_tools);
            tracing::info!("Added {} project analysis tools to orchestration", project_tool_count);

            // Add git extended tools (git_blame, git_show, find_references)
            let git_tools = git_extended_tools::create_git_extended_tools(workspace_provider.clone());
            let git_tool_count = git_tools.len();
            tools.extend(git_tools);
            tracing::info!("Added {} git extended tools to orchestration", git_tool_count);

            // Add code analysis tool
            let code_tool = code_analysis_tool::create_code_analysis_tool(workspace_provider.clone());
            tools.push(code_tool);
            tracing::info!("Added code analysis tool to orchestration");

            // Add search tools
            let search_tools = search_tool::create_search_tools(workspace_provider.clone());
            let search_tool_count = search_tools.len();
            tools.extend(search_tools);
            tracing::info!("Added {} search tools to orchestration", search_tool_count);

            // Add symbol search tools
            let symbol_tools = symbol_search_tool::create_symbol_search_tools(workspace_provider.clone());
            let symbol_tool_count = symbol_tools.len();
            tools.extend(symbol_tools);
            tracing::info!("Added {} symbol search tools to orchestration", symbol_tool_count);

            // Add definition lookup tools
            let definition_tools = definition_tool::create_definition_tools(workspace_provider.clone());
            let definition_tool_count = definition_tools.len();
            tools.extend(definition_tools);
            tracing::info!("Added {} definition lookup tools to orchestration", definition_tool_count);

            // Add terminal command tool
            let terminal_workspace_provider: Arc<dyn TerminalWorkspaceRootProvider> = Arc::new(SimpleWorkspaceRootProvider {
                root: root.clone(),
            });
            let terminal_tool = terminal_tool::create_terminal_command_tool(
                terminal_workspace_provider,
                sandbox_manager,
                None,
            );
            tools.push(terminal_tool);
            tracing::info!("Added terminal command tool to orchestration");
        }

        // Add agent tools
        tools.extend(tool_registry.read().await.get_tools().to_vec());

        // Add MCP tools if provided
        if let Some(mcp_tools) = mcp_tools {
            let mcp_count = mcp_tools.len();
            tools.extend(mcp_tools);
            tracing::info!("Added {} MCP tools to orchestration", mcp_count);
        }

        // Create provider based on configuration
        let provider = Self::create_provider(&config)?;

        // Create orchestration engine with all tools (agent + MCP)
        let engine_config = EngineConfig {
            max_iterations: config.default_provider_config().max_tool_iterations,
            timeout_seconds: 120,
            tool_execution: super::ToolExecutionConfig::default(),
        };
        let mut engine = OrchestrationEngine::with_hook_executor(
            Arc::clone(&provider),
            tools,
            engine_config,
            hook_executor,
        );
        engine.set_event_sender(Some(event_tx.clone()));
        let engine = Arc::new(engine);

        Ok(Self {
            config,
            tool_registry,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            provider,
            engine,
            event_tx,
            context_loader,
            workspace_root,
        })
    }

    /// Subscribe to orchestration events.
    ///
    /// Clients (CLI/TUI/daemon) should consume this stream for tool call lifecycle,
    /// approvals, progress, and completion.
    pub fn subscribe_events(&self) -> broadcast::Receiver<OrchestrationEvent> {
        self.event_tx.subscribe()
    }

    /// Ensure a session exists, optionally seeding it with initial conversation history.
    ///
    /// This is useful for clients (e.g. TUI) that want to inject a system prompt when a
    /// chat session is first created, while still letting the service own subsequent
    /// conversation history.
    pub async fn ensure_session_initialized(
        &self,
        session_id: &str,
        initial_history: Vec<Message>,
    ) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.entry(session_id.to_string()).or_insert_with(|| {
            let mut session = SessionState::new(session_id);
            session.conversation_history = initial_history;
            session
        });
        Ok(())
    }

    /// Create provider based on configuration
    /// Create a model instance from configuration
    fn create_model_from_config(config: &OrchestrationConfig) -> Result<Box<dyn radium_abstraction::Model>> {
        let api_key = config
            .get_api_key(config.default_provider)
            .ok_or_else(|| crate::error::OrchestrationError::Other(
                format!("API key not found for provider: {}. Set {} environment variable.",
                    config.default_provider,
                    Self::api_key_env_var(config.default_provider))
            ))?;

        let model: Box<dyn radium_abstraction::Model> = match config.default_provider {
            ProviderType::Gemini => {
                Box::new(radium_models::GeminiModel::with_api_key(
                    config.gemini.model.clone(),
                    api_key
                ))
            }
            ProviderType::Claude => {
                Box::new(radium_models::ClaudeModel::with_api_key(
                    config.claude.model.clone(),
                    api_key
                ))
            }
            ProviderType::OpenAI => {
                Box::new(radium_models::OpenAIModel::with_api_key(
                    config.openai.model.clone(),
                    api_key
                ))
            }
            ProviderType::PromptBased => {
                // For PromptBased, default to Gemini if API key is available
                if let Some(api_key) = config.get_api_key(ProviderType::Gemini) {
                    Box::new(radium_models::GeminiModel::with_api_key(
                        config.gemini.model.clone(),
                        api_key
                    ))
                } else {
                    return Err(crate::error::OrchestrationError::Other(
                        "PromptBased provider requires at least one AI provider configured (Gemini, Claude, or OpenAI)".to_string()
                    ));
                }
            }
        };

        Ok(model)
    }

    fn create_provider(config: &OrchestrationConfig) -> Result<Arc<dyn OrchestrationProvider>> {
        let api_key = config
            .get_api_key(config.default_provider)
            .ok_or_else(|| crate::error::OrchestrationError::Other(
                format!("API key not found for provider: {}. Set {} environment variable.",
                    config.default_provider,
                    Self::api_key_env_var(config.default_provider))
            ))?;

        let provider: Arc<dyn OrchestrationProvider> = match config.default_provider {
            ProviderType::Gemini => Arc::new(
                GeminiOrchestrator::new(&config.gemini.model, &api_key)
                    .with_temperature(config.gemini.temperature)
                    .with_max_iterations(config.gemini.max_tool_iterations as u32),
            ),
            ProviderType::Claude => Arc::new(
                ClaudeOrchestrator::new(&config.claude.model, &api_key)
                    .with_temperature(config.claude.temperature)
                    .with_max_tokens(config.claude.max_tokens),
            ),
            ProviderType::OpenAI => Arc::new(
                OpenAIOrchestrator::new(&config.openai.model, &api_key)
                    .with_temperature(config.openai.temperature),
            ),
            ProviderType::PromptBased => {
                // For prompt-based, create a real model based on configuration
                let model = Self::create_model_from_config(config)?;
                Arc::new(PromptBasedOrchestrator::new(model))
            }
        };

        Ok(provider)
    }

    /// Get environment variable name for API key
    fn api_key_env_var(provider: ProviderType) -> &'static str {
        match provider {
            ProviderType::Gemini => "GEMINI_API_KEY",
            ProviderType::Claude => "ANTHROPIC_API_KEY",
            ProviderType::OpenAI => "OPENAI_API_KEY",
            ProviderType::PromptBased => "N/A",
        }
    }

    /// Handle user input with orchestration
    ///
    /// # Arguments
    /// * `session_id` - Unique session identifier
    /// * `input` - User input to process
    /// * `current_dir` - Optional current working directory for context file loading
    ///
    /// # Returns
    /// Orchestration result with response and tool calls
    ///
    /// This method handles automatic fallback to prompt-based orchestration
    /// when function calling fails and fallback is enabled.
    pub async fn handle_input(
        &self,
        session_id: &str,
        input: &str,
        current_dir: Option<&std::path::Path>,
    ) -> Result<OrchestrationResult> {
        // Get or create session
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .entry(session_id.to_string())
            .or_insert_with(|| SessionState::new(session_id));

        // Add user message to history
        session.conversation_history.push(Message {
            role: "user".to_string(),
            content: input.to_string(),
            timestamp: chrono::Utc::now(),
        });

        // Build context from session
        let mut context = session.to_context();

        // Load and inject context files if loader is available
        if let Some(ref loader) = self.context_loader {
            if let Some(dir) = current_dir.or_else(|| self.workspace_root.as_deref()) {
                match loader.load_hierarchical(dir) {
                    Ok(context_content) => {
                        if !context_content.is_empty() {
                            // Add context files as a system message
                            let system_message = format!(
                                "Context files (GEMINI.md):\n\n{}",
                                context_content
                            );
                            context.add_message(Message::system(system_message));
                            tracing::debug!("Loaded and injected context files for path: {}", dir.display());
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load context files for {}: {}", dir.display(), e);
                    }
                }
            }
        }

        // Try primary provider first
        match self.engine.execute(input, &mut context).await {
            Ok(r) => {
                // Update session from context
                session.update_from_context(&context);
                Ok(r)
            }
            Err(e) => {
                // Check if fallback is enabled and error is function-calling related
                if self.config.fallback.enabled && self.provider.supports_function_calling() {
                    tracing::warn!(
                        "Function calling failed with primary provider ({}): {}. Attempting fallback to prompt-based orchestration.",
                        self.provider.provider_name(),
                        e
                    );

                    // Try fallback to prompt-based
                    match self.try_fallback(input, &mut context).await {
                        Ok(r) => {
                            session.update_from_context(&context);
                            Ok(r)
                        }
                        Err(fallback_err) => {
                            // Both primary and fallback failed
                            session.update_from_context(&context);
                            Err(crate::error::OrchestrationError::Other(format!(
                                "Orchestration failed: Primary provider error: {}. Fallback error: {}",
                                e, fallback_err
                            )))
                        }
                    }
                } else {
                    // No fallback or not a function-calling provider
                    session.update_from_context(&context);
                    Err(e)
                }
            }
        }
    }

    /// Try fallback orchestration with prompt-based provider
    async fn try_fallback(
        &self,
        input: &str,
        context: &mut OrchestrationContext,
    ) -> Result<OrchestrationResult> {
        // Create prompt-based provider with real model based on configuration
        let model = Self::create_model_from_config(&self.config)?;
        let fallback_provider: Arc<dyn OrchestrationProvider> =
            Arc::new(super::providers::prompt_based::PromptBasedOrchestrator::new(model));

        // Get tools from registry
        let tools = self.tool_registry.read().await.get_tools().to_vec();

        // Create fallback engine (without hooks for now - could be enhanced later)
        let engine_config = EngineConfig {
            max_iterations: self.config.prompt_based.max_tool_iterations,
            timeout_seconds: self.config.default_provider_config().max_tool_iterations as u64 * 24, // 2 minutes per iteration
            tool_execution: super::ToolExecutionConfig::default(),
        };
        let fallback_engine = OrchestrationEngine::new(
            fallback_provider,
            tools,
            engine_config,
        );

        // Execute with fallback engine
        fallback_engine.execute(input, context).await
    }

    /// Get session state
    pub async fn get_session(&self, session_id: &str) -> Option<SessionState> {
        self.sessions.read().await.get(session_id).cloned()
    }

    /// Clear session history
    pub async fn clear_session(&self, session_id: &str) {
        if let Some(session) = self.sessions.write().await.get_mut(session_id) {
            session.conversation_history.clear();
            session.invoked_agents.clear();
        }
    }

    /// Get current provider name
    pub fn provider_name(&self) -> &'static str {
        self.provider.provider_name()
    }

    /// Check if orchestration is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get current configuration
    pub fn config(&self) -> &OrchestrationConfig {
        &self.config
    }

    /// Refresh tool registry (reload agents)
    pub async fn refresh_tools(&self) -> Result<()> {
        self.tool_registry.write().await.refresh()
    }
}

impl OrchestrationConfig {
    /// Get configuration for default provider
    fn default_provider_config(&self) -> ProviderConfig<'_> {
        match self.default_provider {
            ProviderType::Gemini => ProviderConfig {
                model: &self.gemini.model,
                temperature: self.gemini.temperature,
                max_tool_iterations: self.gemini.max_tool_iterations,
            },
            ProviderType::Claude => ProviderConfig {
                model: &self.claude.model,
                temperature: self.claude.temperature,
                max_tool_iterations: self.claude.max_tool_iterations,
            },
            ProviderType::OpenAI => ProviderConfig {
                model: &self.openai.model,
                temperature: self.openai.temperature,
                max_tool_iterations: self.openai.max_tool_iterations,
            },
            ProviderType::PromptBased => ProviderConfig {
                model: "prompt-based",
                temperature: self.prompt_based.temperature,
                max_tool_iterations: self.prompt_based.max_tool_iterations,
            },
        }
    }
}

struct ProviderConfig<'a> {
    #[allow(dead_code)]
    model: &'a str,
    #[allow(dead_code)]
    temperature: f32,
    max_tool_iterations: usize,
}

/// Simple workspace root provider implementation
struct SimpleWorkspaceRootProvider {
    root: PathBuf,
}

impl FileWorkspaceRootProvider for SimpleWorkspaceRootProvider {
    fn workspace_root(&self) -> Option<PathBuf> {
        Some(self.root.clone())
    }
}

impl TerminalWorkspaceRootProvider for SimpleWorkspaceRootProvider {
    fn workspace_root(&self) -> Option<PathBuf> {
        Some(self.root.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_state_creation() {
        let session = SessionState::new("test-session");
        assert_eq!(session.session_id, "test-session");
        assert!(session.conversation_history.is_empty());
        assert!(session.invoked_agents.is_empty());
    }

    #[test]
    fn test_session_to_context() {
        let mut session = SessionState::new("test-session");
        session.conversation_history.push(Message {
            role: "user".to_string(),
            content: "Hello".to_string(),
            timestamp: chrono::Utc::now(),
        });

        let context = session.to_context();
        assert_eq!(context.session_id, "test-session");
        assert_eq!(context.conversation_history.len(), 1);
    }

    #[tokio::test]
    async fn test_service_initialization_without_api_key() {
        // This should fail without API keys
        let config = OrchestrationConfig::default();
        let result = OrchestrationService::initialize(config, None, None, None, None, None).await;
        // Will fail without API keys - that's expected
        assert!(result.is_err());
    }
}
