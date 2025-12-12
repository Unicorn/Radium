// Tool builder - Creates the standard set of tools for CLI and orchestration
//
// Provides a single source of truth for tool registration across the system.

use std::path::PathBuf;
use std::sync::Arc;

use super::{
    code_analysis_tool,
    file_tools::{self, WorkspaceRootProvider as FileWorkspaceRootProvider},
    git_extended_tools,
    project_scan_tool,
    search_tool,
    symbol_search_tool,
    terminal_tool::{self, WorkspaceRootProvider as TerminalWorkspaceRootProvider, SandboxManager as TerminalSandboxManager},
    tool::Tool,
};

/// Simple workspace root provider implementation
#[derive(Clone)]
pub struct SimpleWorkspaceRootProvider {
    pub root: PathBuf,
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

/// No-op sandbox manager for basic tool execution
pub struct NoOpSandboxManager;

#[async_trait::async_trait]
impl TerminalSandboxManager for NoOpSandboxManager {
    async fn initialize_sandbox(&self, _agent_id: &str) -> std::result::Result<(), String> {
        Ok(())
    }

    async fn cleanup_sandbox(&self, _agent_id: &str) {
        // No-op
    }

    fn get_sandbox_path(&self, _agent_id: &str) -> Option<PathBuf> {
        None
    }
}

/// Build the standard set of tools for a workspace
///
/// Returns a vector of all available tools including:
/// - File operations (read, write, list, search, grep)
/// - Project analysis (project_scan)
/// - Git tools (git_log, git_diff, git_blame, git_show, find_references)
/// - Code analysis (AST-based code structure analysis)
/// - Terminal commands (run_command)
pub fn build_standard_tools(
    workspace_root: PathBuf,
    sandbox_manager: Option<Arc<dyn TerminalSandboxManager>>,
) -> Vec<Tool> {
    let mut tools = Vec::new();

    // Create workspace provider
    let workspace_provider: Arc<dyn FileWorkspaceRootProvider> = Arc::new(SimpleWorkspaceRootProvider {
        root: workspace_root.clone(),
    });

    // File operation tools (read_file, write_file, list_directory, search_files, grep)
    let file_tools = file_tools::create_file_operation_tools(workspace_provider.clone());
    let file_count = file_tools.len();
    tools.extend(file_tools);
    tracing::info!("Added {} file operation tools", file_count);

    // Project analysis tools (project_scan)
    let project_tools = project_scan_tool::create_project_analysis_tools(workspace_provider.clone());
    let project_count = project_tools.len();
    tools.extend(project_tools);
    tracing::info!("Added {} project analysis tools", project_count);

    // Git extended tools (git_blame, git_show, find_references)
    let git_tools = git_extended_tools::create_git_extended_tools(workspace_provider.clone());
    let git_count = git_tools.len();
    tools.extend(git_tools);
    tracing::info!("Added {} git extended tools", git_count);

    // Code analysis tool (AST-based structure analysis)
    let code_tool = code_analysis_tool::create_code_analysis_tool(workspace_provider.clone());
    tools.push(code_tool);
    tracing::info!("Added code analysis tool");

    // Search tools (content search with context and filters)
    let search_tools = search_tool::create_search_tools(workspace_provider.clone());
    let search_count = search_tools.len();
    tools.extend(search_tools);
    tracing::info!("Added {} search tools", search_count);

    // Symbol search tools (AST-based symbol extraction)
    let symbol_tools = symbol_search_tool::create_symbol_search_tools(workspace_provider.clone());
    let symbol_count = symbol_tools.len();
    tools.extend(symbol_tools);
    tracing::info!("Added {} symbol search tools", symbol_count);

    // Terminal command tool
    let terminal_workspace_provider: Arc<dyn TerminalWorkspaceRootProvider> = Arc::new(SimpleWorkspaceRootProvider {
        root: workspace_root,
    });
    let terminal_tool = terminal_tool::create_terminal_command_tool(
        terminal_workspace_provider,
        sandbox_manager.or_else(|| Some(Arc::new(NoOpSandboxManager) as Arc<dyn TerminalSandboxManager>)),
        None,
    );
    tools.push(terminal_tool);
    tracing::info!("Added terminal command tool");

    tracing::info!("Built {} total tools for workspace", tools.len());

    tools
}
