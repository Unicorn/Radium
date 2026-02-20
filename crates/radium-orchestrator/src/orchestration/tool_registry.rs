//! Unified tool registry for orchestration
//!
//! This module provides a unified interface for managing all tool types
//! (agent, file, terminal, MCP) with filtering and discovery support.

use super::tool::Tool;

/// Unified tool registry combining all tool types
pub struct UnifiedToolRegistry {
    /// File operation tools
    file_tools: Vec<Tool>,
    /// Terminal command tools
    terminal_tools: Vec<Tool>,
    /// Agent tools
    agent_tools: Vec<Tool>,
    /// MCP tools
    mcp_tools: Vec<Tool>,
    /// All tools combined (for quick access)
    all_tools: Vec<Tool>,
}

impl UnifiedToolRegistry {
    /// Create a new unified tool registry
    pub fn new() -> Self {
        Self {
            file_tools: Vec::new(),
            terminal_tools: Vec::new(),
            agent_tools: Vec::new(),
            mcp_tools: Vec::new(),
            all_tools: Vec::new(),
        }
    }

    /// Add file operation tools
    pub fn add_file_tools(&mut self, tools: Vec<Tool>) {
        self.file_tools = tools;
        self.rebuild_all_tools();
    }

    /// Add terminal command tools
    pub fn add_terminal_tools(&mut self, tools: Vec<Tool>) {
        self.terminal_tools = tools;
        self.rebuild_all_tools();
    }

    /// Add agent tools
    pub fn add_agent_tools(&mut self, tools: Vec<Tool>) {
        self.agent_tools = tools;
        self.rebuild_all_tools();
    }

    /// Add MCP tools
    pub fn add_mcp_tools(&mut self, tools: Vec<Tool>) {
        self.mcp_tools = tools;
        self.rebuild_all_tools();
    }

    /// Rebuild the combined tools list
    fn rebuild_all_tools(&mut self) {
        self.all_tools.clear();
        self.all_tools.extend(self.file_tools.iter().cloned());
        self.all_tools.extend(self.terminal_tools.iter().cloned());
        self.all_tools.extend(self.agent_tools.iter().cloned());
        self.all_tools.extend(self.mcp_tools.iter().cloned());
    }

    /// Get all tools
    pub fn get_all_tools(&self) -> &[Tool] {
        &self.all_tools
    }

    /// Get file operation tools
    pub fn get_file_tools(&self) -> &[Tool] {
        &self.file_tools
    }

    /// Get terminal command tools
    pub fn get_terminal_tools(&self) -> &[Tool] {
        &self.terminal_tools
    }

    /// Get agent tools
    pub fn get_agent_tools(&self) -> &[Tool] {
        &self.agent_tools
    }

    /// Get MCP tools
    pub fn get_mcp_tools(&self) -> &[Tool] {
        &self.mcp_tools
    }

    /// Find a tool by name
    pub fn find_tool(&self, name: &str) -> Option<&Tool> {
        self.all_tools.iter().find(|t| t.name == name || t.id == name)
    }

    /// Filter tools by category
    pub fn filter_by_category(&self, category: ToolCategory) -> Vec<&Tool> {
        match category {
            ToolCategory::File => self.file_tools.iter().collect(),
            ToolCategory::Terminal => self.terminal_tools.iter().collect(),
            ToolCategory::Agent => self.agent_tools.iter().collect(),
            ToolCategory::MCP => self.mcp_tools.iter().collect(),
            ToolCategory::All => self.all_tools.iter().collect(),
        }
    }

    /// Get tool count by category
    pub fn count_by_category(&self, category: ToolCategory) -> usize {
        match category {
            ToolCategory::File => self.file_tools.len(),
            ToolCategory::Terminal => self.terminal_tools.len(),
            ToolCategory::Agent => self.agent_tools.len(),
            ToolCategory::MCP => self.mcp_tools.len(),
            ToolCategory::All => self.all_tools.len(),
        }
    }

    /// Get total tool count
    pub fn total_count(&self) -> usize {
        self.all_tools.len()
    }
}

impl Default for UnifiedToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Tool categories for filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolCategory {
    /// File operation tools
    File,
    /// Terminal command tools
    Terminal,
    /// Agent tools
    Agent,
    /// MCP tools
    MCP,
    /// All tools
    All,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestration::tool::{Tool, ToolHandler, ToolParameters, ToolResult};
    use async_trait::async_trait;
    use std::sync::Arc;

    struct TestHandler;

    #[async_trait]
    impl ToolHandler for TestHandler {
        async fn execute(&self, _args: &crate::orchestration::tool::ToolArguments) -> Result<ToolResult, crate::OrchestrationError> {
            Ok(ToolResult::success("test"))
        }
    }

    fn create_test_tool(id: &str, name: &str) -> Tool {
        Tool::new(
            id,
            name,
            "Test tool",
            ToolParameters::new(),
            Arc::new(TestHandler),
        )
    }

    #[test]
    fn test_unified_registry_creation() {
        let registry = UnifiedToolRegistry::new();
        assert_eq!(registry.total_count(), 0);
    }

    #[test]
    fn test_add_tools() {
        let mut registry = UnifiedToolRegistry::new();
        
        registry.add_file_tools(vec![create_test_tool("read_file", "read_file")]);
        registry.add_terminal_tools(vec![create_test_tool("run_terminal_cmd", "run_terminal_cmd")]);
        registry.add_agent_tools(vec![create_test_tool("agent_test", "test_agent")]);
        
        assert_eq!(registry.count_by_category(ToolCategory::File), 1);
        assert_eq!(registry.count_by_category(ToolCategory::Terminal), 1);
        assert_eq!(registry.count_by_category(ToolCategory::Agent), 1);
        assert_eq!(registry.total_count(), 3);
    }

    #[test]
    fn test_find_tool() {
        let mut registry = UnifiedToolRegistry::new();
        registry.add_file_tools(vec![create_test_tool("read_file", "read_file")]);
        
        assert!(registry.find_tool("read_file").is_some());
        assert!(registry.find_tool("nonexistent").is_none());
    }

    #[test]
    fn test_filter_by_category() {
        let mut registry = UnifiedToolRegistry::new();
        registry.add_file_tools(vec![create_test_tool("read_file", "read_file")]);
        registry.add_agent_tools(vec![create_test_tool("agent_test", "test_agent")]);
        
        let file_tools = registry.filter_by_category(ToolCategory::File);
        assert_eq!(file_tools.len(), 1);
        
        let agent_tools = registry.filter_by_category(ToolCategory::Agent);
        assert_eq!(agent_tools.len(), 1);
    }
}

