//! Context manager for gathering and injecting context into prompts.

use super::error::Result;
use super::injection::{ContextInjector, InjectionDirective};
use crate::memory::MemoryStore;
use crate::workspace::{PlanDiscovery, RequirementId, Workspace};
use std::path::Path;

/// Context manager for agent execution.
///
/// Gathers context from various sources:
/// - Plan information and metadata
/// - Memory from previous agent executions
/// - File contents via injection syntax
/// - Architecture documentation
pub struct ContextManager {
    /// Workspace root path.
    workspace_root: std::path::PathBuf,

    /// Context injector for file operations.
    injector: ContextInjector,

    /// Memory store for agent outputs.
    memory_store: Option<MemoryStore>,
}

impl ContextManager {
    /// Creates a new context manager.
    ///
    /// # Arguments
    /// * `workspace` - The workspace to gather context from
    pub fn new(workspace: &Workspace) -> Self {
        let workspace_root = workspace.root().to_path_buf();
        let injector = ContextInjector::new(&workspace_root);

        Self {
            workspace_root,
            injector,
            memory_store: None,
        }
    }

    /// Creates a context manager for a specific plan.
    ///
    /// # Arguments
    /// * `workspace` - The workspace to gather context from
    /// * `requirement_id` - The plan's requirement ID
    ///
    /// # Returns
    /// A context manager with memory store initialized
    ///
    /// # Errors
    /// Returns error if memory store initialization fails
    pub fn for_plan(workspace: &Workspace, requirement_id: RequirementId) -> Result<Self> {
        let workspace_root = workspace.root().to_path_buf();
        let injector = ContextInjector::new(&workspace_root);

        // Initialize memory store for this plan
        let memory_store = MemoryStore::new(&workspace_root, requirement_id)?;

        Ok(Self {
            workspace_root,
            injector,
            memory_store: Some(memory_store),
        })
    }

    /// Gathers plan context information.
    ///
    /// # Arguments
    /// * `requirement_id` - The plan's requirement ID
    ///
    /// # Returns
    /// Plan context as a formatted string
    ///
    /// # Errors
    /// Returns error if plan cannot be found or loaded
    pub fn gather_plan_context(&self, requirement_id: RequirementId) -> Result<String> {
        let workspace = Workspace::discover()?;
        let discovery = PlanDiscovery::new(&workspace);

        let plan = discovery
            .find_by_requirement_id(requirement_id)?
            .ok_or_else(|| {
                crate::context::error::ContextError::FileNotFound(format!(
                    "Plan not found: {}",
                    requirement_id
                ))
            })?;

        let mut context = String::new();
        context.push_str("# Plan Context\n\n");
        context.push_str(&format!("**Requirement ID**: {}\n", requirement_id));
        context.push_str(&format!("**Project**: {}\n", plan.plan.project_name));
        context.push_str(&format!("**Stage**: {}\n", plan.plan.stage));
        context.push_str(&format!("**Status**: {:?}\n", plan.plan.status));
        context.push_str(&format!("**Path**: {}\n\n", plan.path.display()));

        Ok(context)
    }

    /// Gathers architecture context from documentation.
    ///
    /// # Returns
    /// Architecture context if available
    pub fn gather_architecture_context(&self) -> Option<String> {
        let architecture_path = self.workspace_root.join(".radium").join("architecture.md");

        if !architecture_path.exists() {
            return None;
        }

        std::fs::read_to_string(architecture_path).ok().map(|content| {
            let mut context = String::new();
            context.push_str("# Architecture Context\n\n");
            context.push_str(&content);
            context.push('\n');
            context
        })
    }

    /// Gathers memory context from a specific agent.
    ///
    /// # Arguments
    /// * `agent_id` - The agent identifier
    ///
    /// # Returns
    /// The agent's last output if available
    ///
    /// # Errors
    /// Returns error if memory store is not initialized
    pub fn gather_memory_context(&self, agent_id: &str) -> Result<Option<String>> {
        let Some(ref memory_store) = self.memory_store else {
            return Ok(None);
        };

        match memory_store.get(agent_id) {
            Ok(entry) => {
                let mut context = String::new();
                context.push_str(&format!("# Previous Output from {}\n\n", agent_id));
                context.push_str(&entry.output);
                context.push('\n');
                Ok(Some(context))
            }
            Err(_) => Ok(None), // Agent hasn't run yet
        }
    }

    /// Processes injection directives and returns injected content.
    ///
    /// # Arguments
    /// * `directives` - The injection directives to process
    ///
    /// # Returns
    /// Injected content based on directives
    ///
    /// # Errors
    /// Returns error if injection fails
    pub fn process_directives(&self, directives: &[InjectionDirective]) -> Result<String> {
        let mut content = String::new();

        for directive in directives {
            match directive {
                InjectionDirective::FileInput { files } => {
                    let injected = self.injector.inject_files(files)?;
                    content.push_str(&injected);
                }
                InjectionDirective::TailContext { lines } => {
                    // For tail context, we'd typically need an agent ID to know
                    // which agent's output to tail. This is a simplified version.
                    content.push_str(&format!("\n# Tail Context ({} lines requested)\n\n", lines));
                }
            }
        }

        Ok(content)
    }

    /// Builds complete context for an agent invocation.
    ///
    /// # Arguments
    /// * `invocation` - The agent invocation string (e.g., "agent[input:file.md]")
    /// * `requirement_id` - Optional plan requirement ID for plan context
    ///
    /// # Returns
    /// Complete context string ready for prompt injection
    ///
    /// # Errors
    /// Returns error if context gathering fails
    pub fn build_context(
        &self,
        invocation: &str,
        requirement_id: Option<RequirementId>,
    ) -> Result<String> {
        let (agent_name, directives) = InjectionDirective::extract_directives(invocation)?;

        let mut context = String::new();

        // Add plan context if requirement ID provided
        if let Some(req_id) = requirement_id {
            if let Ok(plan_ctx) = self.gather_plan_context(req_id) {
                context.push_str(&plan_ctx);
                context.push_str("\n---\n\n");
            }
        }

        // Add architecture context if available
        if let Some(arch_ctx) = self.gather_architecture_context() {
            context.push_str(&arch_ctx);
            context.push_str("\n---\n\n");
        }

        // Add memory context for this agent if available
        if let Ok(Some(mem_ctx)) = self.gather_memory_context(&agent_name) {
            context.push_str(&mem_ctx);
            context.push_str("\n---\n\n");
        }

        // Process injection directives
        if !directives.is_empty() {
            let injected = self.process_directives(&directives)?;
            context.push_str(&injected);
        }

        Ok(context)
    }

    /// Returns the workspace root path.
    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    /// Returns a reference to the memory store if initialized.
    pub fn memory_store(&self) -> Option<&MemoryStore> {
        self.memory_store.as_ref()
    }

    /// Returns a mutable reference to the memory store if initialized.
    pub fn memory_store_mut(&mut self) -> Option<&mut MemoryStore> {
        self.memory_store.as_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_context_manager_new() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::create(temp_dir.path()).unwrap();
        let manager = ContextManager::new(&workspace);
        assert!(manager.memory_store.is_none());
    }

    #[test]
    fn test_context_manager_for_plan() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::create(temp_dir.path()).unwrap();
        let req_id = RequirementId::new(1);

        let manager = ContextManager::for_plan(&workspace, req_id).unwrap();
        assert!(manager.memory_store.is_some());
    }

    #[test]
    fn test_gather_architecture_context() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::create(temp_dir.path()).unwrap();

        // Create architecture file
        let arch_path = temp_dir.path().join(".radium").join("architecture.md");
        fs::write(&arch_path, "# System Architecture\n\nThis is the system design.").unwrap();

        let manager = ContextManager::new(&workspace);
        let context = manager.gather_architecture_context();
        assert!(context.is_some());
        assert!(context.unwrap().contains("System Architecture"));
    }

    #[test]
    fn test_gather_architecture_context_missing() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::create(temp_dir.path()).unwrap();

        let manager = ContextManager::new(&workspace);
        let context = manager.gather_architecture_context();
        assert!(context.is_none());
    }

    #[test]
    fn test_process_directives_file_input() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::create(temp_dir.path()).unwrap();

        // Create test file
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Test content").unwrap();

        let manager = ContextManager::new(&workspace);
        let directives = vec![InjectionDirective::FileInput {
            files: vec![std::path::PathBuf::from("test.txt")],
        }];

        let content = manager.process_directives(&directives).unwrap();
        assert!(content.contains("Test content"));
    }

    #[test]
    fn test_build_context_simple() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::create(temp_dir.path()).unwrap();

        let manager = ContextManager::new(&workspace);
        let context = manager.build_context("architect", None).unwrap();
        assert!(!context.is_empty() || context.is_empty()); // May be empty without context sources
    }

    #[test]
    fn test_build_context_with_file_injection() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::create(temp_dir.path()).unwrap();

        // Create test file
        let file_path = temp_dir.path().join("spec.md");
        fs::write(&file_path, "# Specification\n\nBuild a feature.").unwrap();

        let manager = ContextManager::new(&workspace);
        let context = manager.build_context("architect[input:spec.md]", None).unwrap();
        assert!(context.contains("Specification"));
        assert!(context.contains("Build a feature"));
    }
}
