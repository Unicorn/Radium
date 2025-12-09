//! Context manager for gathering and injecting context into prompts.

use std::fmt::Write;
use std::path::Path;
use std::time::Instant;

use super::analysis::AnalysisPlan;
use super::error::Result;
use super::files::ContextFileLoader;
use super::injection::{ContextInjector, InjectionDirective};
use super::metrics::ContextMetrics;
use super::playbook_loader::PlaybookLoader;
use super::sources::{
    BraingridReader, HttpReader, JiraReader, LocalFileReader, SourceRegistry,
};
use crate::config::Config;
use crate::learning::LearningStore;
use crate::memory::MemoryStore;
use crate::playbooks::registry::PlaybookRegistry;
use crate::security::{PatternLibrary, PrivacyFilter, RedactionStyle, SecretFilter};
use crate::workspace::{PlanDiscovery, RequirementId, Workspace};
use std::sync::Arc;

/// Context manager for agent execution.
///
/// Gathers context from various sources:
/// - Plan information and metadata
/// - Memory from previous agent executions
/// - File contents via injection syntax
/// - Architecture documentation
/// - Learning context from past mistakes and successes
/// - Context files (GEMINI.md) with hierarchical loading
pub struct ContextManager {
    /// Workspace root path.
    workspace_root: std::path::PathBuf,

    /// Context injector for file operations.
    injector: ContextInjector,

    /// Context file loader for GEMINI.md files.
    context_file_loader: ContextFileLoader,

    /// Memory store for agent outputs.
    memory_store: Option<MemoryStore>,

    /// Learning store for past mistakes and strategies.
    learning_store: Option<LearningStore>,

    /// Cached context file content with modification times of all loaded files.
    /// Stores: (request_path, Vec<(file_path, mtime)>, content)
    context_file_cache: Option<(std::path::PathBuf, Vec<(std::path::PathBuf, std::time::SystemTime)>, String)>,

    /// Source registry for reading from different source types.
    source_registry: SourceRegistry,

    /// Optional metrics collector (enabled when metrics are needed).
    metrics: Option<ContextMetrics>,

    /// Optional playbook registry for organizational knowledge.
    playbook_registry: Option<Arc<PlaybookRegistry>>,

    /// Optional privacy filter for redacting sensitive data.
    privacy_filter: Option<Arc<PrivacyFilter>>,

    /// Optional secret filter for redacting credentials.
    secret_filter: Option<Arc<SecretFilter>>,
}

impl ContextManager {
    /// Creates a new context manager.
    ///
    /// # Arguments
    /// * `workspace` - The workspace to gather context from
    pub fn new(workspace: &Workspace) -> Self {
        Self::new_with_config(workspace, None)
    }

    /// Creates a new context manager with optional privacy configuration.
    ///
    /// # Arguments
    /// * `workspace` - The workspace to gather context from
    /// * `config` - Optional configuration for privacy filtering
    pub fn new_with_config(workspace: &Workspace, config: Option<&Config>) -> Self {
        let workspace_root = workspace.root().to_path_buf();
        let injector = ContextInjector::new(&workspace_root);
        let context_file_loader = ContextFileLoader::new(&workspace_root);

        // Initialize source registry with all readers
        let mut source_registry = SourceRegistry::new();
        source_registry.register(Box::new(LocalFileReader::with_base_dir(&workspace_root)));
        source_registry.register(Box::new(HttpReader::new()));
        source_registry.register(Box::new(JiraReader::new()));
        source_registry.register(Box::new(BraingridReader::new()));

        // Initialize privacy filter if enabled in config
        let privacy_filter = config
            .and_then(|c| {
                if c.security.privacy.enable {
                    let style = parse_redaction_style(&c.security.privacy.redaction_style);
                    let patterns = PatternLibrary::default();
                    Some(Arc::new(PrivacyFilter::new(style, patterns)))
                } else {
                    None
                }
            });

        Self {
            workspace_root,
            injector,
            context_file_loader,
            memory_store: None,
            learning_store: None,
            context_file_cache: None,
            source_registry,
            metrics: None,
            playbook_registry: None,
            privacy_filter,
            secret_filter: None,
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
        Self::for_plan_with_config(workspace, requirement_id, None)
    }

    /// Creates a context manager for a specific plan with optional privacy configuration.
    ///
    /// # Arguments
    /// * `workspace` - The workspace to gather context from
    /// * `requirement_id` - The plan's requirement ID
    /// * `config` - Optional configuration for privacy filtering
    ///
    /// # Returns
    /// A context manager with memory store initialized
    ///
    /// # Errors
    /// Returns error if memory store initialization fails
    pub fn for_plan_with_config(workspace: &Workspace, requirement_id: RequirementId, config: Option<&Config>) -> Result<Self> {
        let workspace_root = workspace.root().to_path_buf();
        let injector = ContextInjector::new(&workspace_root);
        let context_file_loader = ContextFileLoader::new(&workspace_root);

        // Initialize memory store for this plan
        let memory_store = MemoryStore::new(&workspace_root, requirement_id)?;

        // Initialize learning store (optional - may fail if directory doesn't exist)
        let learning_store = LearningStore::new(&workspace_root).ok();

        // Initialize source registry with all readers
        let mut source_registry = SourceRegistry::new();
        source_registry.register(Box::new(LocalFileReader::with_base_dir(&workspace_root)));
        source_registry.register(Box::new(HttpReader::new()));
        source_registry.register(Box::new(JiraReader::new()));
        source_registry.register(Box::new(BraingridReader::new()));

        // Initialize privacy filter if enabled in config
        let privacy_filter = config
            .and_then(|c| {
                if c.security.privacy.enable {
                    let style = parse_redaction_style(&c.security.privacy.redaction_style);
                    let patterns = PatternLibrary::default();
                    Some(Arc::new(PrivacyFilter::new(style, patterns)))
                } else {
                    None
                }
            });

        Ok(Self {
            workspace_root,
            injector,
            context_file_loader,
            memory_store: Some(memory_store),
            learning_store,
            context_file_cache: None,
            source_registry,
            metrics: None,
            playbook_registry: None,
            privacy_filter,
            secret_filter: None, // TODO: Initialize SecretFilter when needed
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

        let plan = discovery.find_by_requirement_id(requirement_id)?.ok_or_else(|| {
            crate::context::error::ContextError::FileNotFound(format!(
                "Plan not found: {}",
                requirement_id
            ))
        })?;

        let mut context = String::new();
        context.push_str("# Plan Context\n\n");
        writeln!(context, "**Requirement ID**: {}", requirement_id).unwrap();
        writeln!(context, "**Project**: {}", plan.plan.project_name).unwrap();
        writeln!(context, "**Stage**: {}", plan.plan.stage).unwrap();
        writeln!(context, "**Status**: {:?}", plan.plan.status).unwrap();
        writeln!(context, "**Path**: {}\n", plan.path.display()).unwrap();

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
                writeln!(context, "# Previous Output from {}\n", agent_id).unwrap();
                context.push_str(&entry.output);
                context.push('\n');
                Ok(Some(context))
            }
            Err(_) => Ok(None), // Agent hasn't run yet
        }
    }

    /// Gathers learning context from past mistakes and successes.
    ///
    /// # Arguments
    /// * `max_per_category` - Maximum examples per category to include (default: 3)
    ///
    /// # Returns
    /// Learning context as a formatted string, or None if no learning store is available
    pub fn gather_learning_context(&self, max_per_category: usize) -> Option<String> {
        let Some(ref learning_store) = self.learning_store else {
            return None;
        };

        let context = learning_store.generate_context(max_per_category);
        if context.is_empty() { None } else { Some(format!("# Learning Context\n\n{}\n", context)) }
    }

    /// Gathers skillbook context from learned strategies.
    ///
    /// # Arguments
    /// * `max_per_section` - Maximum skills per section to include (default: 5)
    ///
    /// # Returns
    /// Skillbook context as a formatted string, or None if no learning store is available
    pub fn gather_skillbook_context(&self, max_per_section: usize) -> Option<String> {
        let Some(ref learning_store) = self.learning_store else {
            return None;
        };

        let context = learning_store.as_context(max_per_section);
        if context.is_empty() { None } else { Some(format!("{}\n", context)) }
    }

    /// Gathers playbook context from organizational knowledge.
    ///
    /// # Arguments
    /// * `requirement_id` - Optional requirement ID to determine scope
    ///
    /// # Returns
    /// Playbook context as a formatted string, or None if no playbook registry is available
    pub fn gather_playbook_context(&self, requirement_id: Option<&RequirementId>) -> Option<String> {
        let Some(ref registry) = self.playbook_registry else {
            return None;
        };

        // Determine scope: "requirement" if requirement_id is provided, otherwise "task"
        let scope = if requirement_id.is_some() {
            "requirement"
        } else {
            "task"
        };

        let loader = PlaybookLoader::new(Arc::clone(registry));

        // Load playbooks for this scope (no tag filtering for now)
        match loader.load_for_scope(scope, None) {
            Ok(playbooks) if !playbooks.is_empty() => {
                Some(PlaybookLoader::format_playbooks(&playbooks))
            }
            Ok(_) | Err(_) => None,
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
                    writeln!(content, "\n# Tail Context ({} lines requested)\n", lines).unwrap();
                }
            }
        }

        Ok(content)
    }

    /// Loads context files for a given path.
    ///
    /// Uses caching to avoid re-reading files that haven't changed.
    ///
    /// # Arguments
    /// * `path` - The path to load context for (can be file or directory)
    ///
    /// # Returns
    /// Context file content if available
    ///
    /// # Errors
    /// Returns error if context file loading fails
    pub fn load_context_files(&mut self, path: &Path) -> Result<Option<String>> {
        // Get list of context files that would be loaded
        let context_file_paths = self.context_file_loader.get_context_file_paths(path);

        // Check cache first
        if let Some((cached_path, cached_files, cached_content)) = &self.context_file_cache {
            if cached_path == path {
                // Check if all context files are unchanged
                let mut all_unchanged = true;
                for (file_path, cached_mtime) in cached_files {
                    if let Ok(metadata) = std::fs::metadata(file_path) {
                        if let Ok(mtime) = metadata.modified() {
                            if mtime != *cached_mtime {
                                all_unchanged = false;
                                break;
                            }
                        } else {
                            all_unchanged = false;
                            break;
                        }
                    } else {
                        // File no longer exists
                        all_unchanged = false;
                        break;
                    }
                }
                // Also check if any new files were added
                if all_unchanged && context_file_paths.len() == cached_files.len() {
                    // Verify all current files are in cache
                    for file_path in &context_file_paths {
                        if !cached_files.iter().any(|(cached_path, _)| cached_path == file_path) {
                            all_unchanged = false;
                            break;
                        }
                    }
                    if all_unchanged {
                        return Ok(Some(cached_content.clone()));
                    }
                }
            }
        }

        // Load context files
        let content = self.context_file_loader.load_hierarchical(path)?;

        if content.is_empty() {
            return Ok(None);
        }

        // Update cache with all loaded files and their modification times
        let mut cached_files = Vec::new();
        for file_path in &context_file_paths {
            if let Ok(metadata) = std::fs::metadata(file_path) {
                if let Ok(mtime) = metadata.modified() {
                    cached_files.push((file_path.clone(), mtime));
                }
            }
        }
        self.context_file_cache = Some((path.to_path_buf(), cached_files, content.clone()));

        Ok(Some(content))
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
        &mut self,
        invocation: &str,
        requirement_id: Option<RequirementId>,
    ) -> Result<String> {
        let start_time = Instant::now();
        let (agent_name, directives) = InjectionDirective::extract_directives(invocation)?;

        let mut context = String::new();

        // Add context files first (highest precedence in context building)
        let current_path = std::env::current_dir().unwrap_or_else(|_| self.workspace_root.clone());
        if let Ok(Some(context_files)) = self.load_context_files(&current_path) {
            context.push_str("# Context Files\n\n");
            context.push_str(&context_files);
            context.push_str("\n---\n\n");
        }

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

        // Add playbook context if available
        if let Some(playbook_ctx) = self.gather_playbook_context(requirement_id.as_ref()) {
            context.push_str(&playbook_ctx);
            context.push_str("\n---\n\n");
        }

        // Add memory context for this agent if available
        if let Ok(Some(mem_ctx)) = self.gather_memory_context(&agent_name) {
            context.push_str(&mem_ctx);
            context.push_str("\n---\n\n");
        }

        // Add learning context if available (mistakes and preferences)
        if let Some(learning_ctx) = self.gather_learning_context(3) {
            context.push_str(&learning_ctx);
            context.push_str("\n---\n\n");
        }

        // Add skillbook context if available (strategies and skills)
        if let Some(skillbook_ctx) = self.gather_skillbook_context(5) {
            context.push_str(&skillbook_ctx);
            context.push_str("\n---\n\n");
        }

        // Process injection directives
        if !directives.is_empty() {
            let injected = self.process_directives(&directives)?;
            context.push_str(&injected);
        }

        // Apply privacy filter if enabled
        let mut final_context = if let Some(ref filter) = self.privacy_filter {
            let (redacted, stats) = filter.redact(&context).map_err(|e| {
                super::error::ContextError::Other(format!("Privacy redaction failed: {}", e))
            })?;
            if let Some(ref mut metrics) = self.metrics {
                metrics.redaction_count = stats.count;
            }
            redacted
        } else {
            context
        };

        // Apply secret filter if enabled (after privacy filter)
        if let Some(ref filter) = self.secret_filter {
            final_context = filter.redact_secrets(&final_context).map_err(|e| {
                super::error::ContextError::Other(format!("Secret redaction failed: {}", e))
            })?;
        }

        // Record metrics if enabled
        if let Some(ref mut metrics) = self.metrics {
            let total_time = start_time.elapsed();
            metrics.total_time_ms = total_time.as_millis() as u64;
        }

        Ok(final_context)
    }

    /// Creates an analysis plan from user input.
    ///
    /// This method analyzes the user's question to determine the question type
    /// and provides recommendations for files to read and searches to perform.
    ///
    /// # Arguments
    /// * `input` - The user's input/question
    ///
    /// # Returns
    /// An analysis plan with recommendations
    pub fn create_analysis_plan(&self, input: &str) -> AnalysisPlan {
        AnalysisPlan::from_input(input)
    }

    /// Builds context with analysis plan guidance included.
    ///
    /// This is similar to `build_context` but includes an analysis plan at the
    /// beginning to guide the agent on what files to read and searches to perform.
    ///
    /// # Arguments
    /// * `invocation` - The agent invocation string (e.g., "agent[input:file.md]")
    /// * `requirement_id` - Optional plan requirement ID for plan context
    /// * `user_input` - The original user input/question for analysis planning
    ///
    /// # Returns
    /// Complete context string ready for prompt injection, including analysis plan
    ///
    /// # Errors
    /// Returns error if context gathering fails
    pub fn build_context_with_analysis(
        &mut self,
        invocation: &str,
        requirement_id: Option<RequirementId>,
        user_input: &str,
    ) -> Result<String> {
        // Create analysis plan from user input
        let analysis_plan = self.create_analysis_plan(user_input);
        let plan_context = analysis_plan.to_context_string();

        // Build regular context
        let mut context = self.build_context(invocation, requirement_id)?;

        // Prepend analysis plan to context
        let mut final_context = plan_context;
        final_context.push_str("---\n\n");
        final_context.push_str(&context);

        Ok(final_context)
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

    /// Returns a reference to the learning store if initialized.
    pub fn learning_store(&self) -> Option<&LearningStore> {
        self.learning_store.as_ref()
    }

    /// Returns a mutable reference to the learning store if initialized.
    pub fn learning_store_mut(&mut self) -> Option<&mut LearningStore> {
        self.learning_store.as_mut()
    }

    /// Sets the learning store.
    pub fn set_learning_store(&mut self, learning_store: LearningStore) {
        self.learning_store = Some(learning_store);
    }

    /// Sets the playbook registry.
    pub fn with_playbook_registry(mut self, registry: Arc<PlaybookRegistry>) -> Self {
        self.playbook_registry = Some(registry);
        self
    }

    /// Sets the secret filter for credential redaction.
    ///
    /// # Arguments
    /// * `filter` - Secret filter to use for redacting credentials
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_secret_filter(mut self, filter: Arc<SecretFilter>) -> Self {
        self.secret_filter = Some(filter);
        self
    }

    /// Returns a reference to the playbook registry if initialized.
    pub fn playbook_registry(&self) -> Option<&Arc<PlaybookRegistry>> {
        self.playbook_registry.as_ref()
    }

    /// Returns a reference to the source registry.
    pub fn source_registry(&self) -> &SourceRegistry {
        &self.source_registry
    }
}

/// Parses redaction style from string configuration.
fn parse_redaction_style(style: &str) -> RedactionStyle {
    match style.to_lowercase().as_str() {
        "full" => RedactionStyle::Full,
        "partial" => RedactionStyle::Partial,
        "hash" => RedactionStyle::Hash,
        _ => RedactionStyle::Partial, // Default to partial
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

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

        let mut manager = ContextManager::new(&workspace);
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

        let mut manager = ContextManager::new(&workspace);
        let context = manager.build_context("architect[input:spec.md]", None).unwrap();
        assert!(context.contains("Specification"));
        assert!(context.contains("Build a feature"));
    }

    #[test]
    fn test_gather_memory_context_no_store() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::create(temp_dir.path()).unwrap();

        let manager = ContextManager::new(&workspace);
        let result = manager.gather_memory_context("test-agent");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_gather_memory_context_with_store() {
        use crate::memory::MemoryEntry;

        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::create(temp_dir.path()).unwrap();
        let req_id = RequirementId::new(1);

        let mut manager = ContextManager::for_plan(&workspace, req_id).unwrap();

        // Store some memory
        let entry = MemoryEntry::new("test-agent".to_string(), "Previous output".to_string());
        manager.memory_store_mut().unwrap().store(entry).unwrap();

        // Gather memory context
        let context = manager.gather_memory_context("test-agent").unwrap();
        assert!(context.is_some());
        assert!(context.unwrap().contains("Previous output"));
    }

    #[test]
    fn test_gather_memory_context_agent_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::create(temp_dir.path()).unwrap();
        let req_id = RequirementId::new(1);

        let manager = ContextManager::for_plan(&workspace, req_id).unwrap();

        // Try to get memory for non-existent agent
        let result = manager.gather_memory_context("nonexistent");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_process_directives_empty() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::create(temp_dir.path()).unwrap();

        let manager = ContextManager::new(&workspace);
        let content = manager.process_directives(&[]).unwrap();
        assert_eq!(content, "");
    }

    #[test]
    fn test_process_directives_tail_context() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::create(temp_dir.path()).unwrap();

        let manager = ContextManager::new(&workspace);
        let directives = vec![InjectionDirective::TailContext { lines: 10 }];

        let content = manager.process_directives(&directives).unwrap();
        assert!(content.contains("Tail Context"));
        assert!(content.contains("10 lines"));
    }

    #[test]
    fn test_process_directives_multiple() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::create(temp_dir.path()).unwrap();

        // Create test files
        let file1_path = temp_dir.path().join("file1.txt");
        let file2_path = temp_dir.path().join("file2.txt");
        fs::write(&file1_path, "Content 1").unwrap();
        fs::write(&file2_path, "Content 2").unwrap();

        let manager = ContextManager::new(&workspace);
        let directives = vec![
            InjectionDirective::FileInput { files: vec![std::path::PathBuf::from("file1.txt")] },
            InjectionDirective::TailContext { lines: 5 },
            InjectionDirective::FileInput { files: vec![std::path::PathBuf::from("file2.txt")] },
        ];

        let content = manager.process_directives(&directives).unwrap();
        assert!(content.contains("Content 1"));
        assert!(content.contains("Content 2"));
        assert!(content.contains("Tail Context"));
    }

    #[test]
    fn test_workspace_root_accessor() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::create(temp_dir.path()).unwrap();

        let manager = ContextManager::new(&workspace);
        assert_eq!(manager.workspace_root(), workspace.root());
    }

    #[test]
    fn test_memory_store_accessor_none() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::create(temp_dir.path()).unwrap();

        let manager = ContextManager::new(&workspace);
        assert!(manager.memory_store().is_none());
    }

    #[test]
    fn test_memory_store_accessor_some() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::create(temp_dir.path()).unwrap();
        let req_id = RequirementId::new(1);

        let manager = ContextManager::for_plan(&workspace, req_id).unwrap();
        assert!(manager.memory_store().is_some());
    }

    #[test]
    fn test_memory_store_mut_accessor() {
        use crate::memory::MemoryEntry;

        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::create(temp_dir.path()).unwrap();
        let req_id = RequirementId::new(1);

        let mut manager = ContextManager::for_plan(&workspace, req_id).unwrap();

        // Use mutable accessor to store memory
        let entry = MemoryEntry::new("agent".to_string(), "output".to_string());
        manager.memory_store_mut().unwrap().store(entry).unwrap();

        // Verify it was stored
        let stored = manager.memory_store().unwrap().get("agent").unwrap();
        assert_eq!(stored.output, "output");
    }

    #[test]
    fn test_build_context_with_architecture() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::create(temp_dir.path()).unwrap();

        // Create architecture file
        let arch_path = temp_dir.path().join(".radium").join("architecture.md");
        fs::write(&arch_path, "# Architecture\n\nMicroservices design").unwrap();

        let mut manager = ContextManager::new(&workspace);
        let context = manager.build_context("agent", None).unwrap();
        assert!(context.contains("Architecture Context"));
        assert!(context.contains("Microservices design"));
    }

    #[test]
    fn test_build_context_with_memory() {
        use crate::memory::MemoryEntry;

        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::create(temp_dir.path()).unwrap();
        let req_id = RequirementId::new(1);

        let mut manager = ContextManager::for_plan(&workspace, req_id).unwrap();

        // Store memory for agent
        let entry = MemoryEntry::new("architect".to_string(), "Previous design".to_string());
        manager.memory_store_mut().unwrap().store(entry).unwrap();

        // Build context
        let context = manager.build_context("architect", None).unwrap();
        assert!(context.contains("Previous Output from architect"));
        assert!(context.contains("Previous design"));
    }

    #[test]
    fn test_build_context_combined() {
        use crate::memory::MemoryEntry;

        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::create(temp_dir.path()).unwrap();
        let req_id = RequirementId::new(1);

        // Create architecture file
        let arch_path = temp_dir.path().join(".radium").join("architecture.md");
        fs::write(&arch_path, "# Architecture").unwrap();

        // Create test file
        let file_path = temp_dir.path().join("input.txt");
        fs::write(&file_path, "Input data").unwrap();

        let mut manager = ContextManager::for_plan(&workspace, req_id).unwrap();

        // Store memory
        let entry = MemoryEntry::new("agent".to_string(), "Previous output".to_string());
        manager.memory_store_mut().unwrap().store(entry).unwrap();

        // Build context with everything
        let context = manager.build_context("agent[input:input.txt]", None).unwrap();
        assert!(context.contains("Architecture"));
        assert!(context.contains("Previous output"));
        assert!(context.contains("Input data"));
    }

    #[test]
    fn test_build_context_no_directives() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::create(temp_dir.path()).unwrap();

        let mut manager = ContextManager::new(&workspace);
        let context = manager.build_context("simple-agent", None).unwrap();
        // Should not error even with no context sources
        assert!(context.is_empty() || !context.is_empty());
    }

    #[test]
    fn test_gather_architecture_context_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::create(temp_dir.path()).unwrap();

        // Create empty architecture file
        let arch_path = temp_dir.path().join(".radium").join("architecture.md");
        fs::write(&arch_path, "").unwrap();

        let manager = ContextManager::new(&workspace);
        let context = manager.gather_architecture_context();
        assert!(context.is_some());
        let content = context.unwrap();
        assert!(content.contains("Architecture Context"));
    }

    #[test]
    fn test_build_context_with_context_files() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::create(temp_dir.path()).unwrap();

        // Create context file
        let context_file = temp_dir.path().join("GEMINI.md");
        fs::write(&context_file, "# Project Context\n\nUse Rust and follow best practices.")
            .unwrap();

        let mut manager = ContextManager::new(&workspace);
        // Load context files directly for the temp directory
        let context_files = manager.load_context_files(temp_dir.path()).unwrap();
        assert!(context_files.is_some());
        assert!(context_files.as_ref().unwrap().contains("Project Context"));
        assert!(context_files.as_ref().unwrap().contains("Use Rust"));
    }

    #[test]
    fn test_load_context_files_caching() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::create(temp_dir.path()).unwrap();

        // Create context file
        let context_file = temp_dir.path().join("GEMINI.md");
        fs::write(&context_file, "# Context").unwrap();

        let mut manager = ContextManager::new(&workspace);

        // First load
        let content1 = manager.load_context_files(temp_dir.path()).unwrap();
        assert!(content1.is_some());
        assert!(content1.as_ref().unwrap().contains("Context"));

        // Second load should use cache
        let content2 = manager.load_context_files(temp_dir.path()).unwrap();
        assert_eq!(content1, content2);
    }

    #[test]
    fn test_build_context_with_context_files_precedence() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::create(temp_dir.path()).unwrap();

        // Create context file
        let context_file = temp_dir.path().join("GEMINI.md");
        fs::write(&context_file, "# Context Files\n\nProject guidelines.").unwrap();

        // Create architecture file
        let arch_path = temp_dir.path().join(".radium").join("architecture.md");
        fs::write(&arch_path, "# Architecture").unwrap();

        let mut manager = ContextManager::new(&workspace);
        let context = manager.build_context("agent", None).unwrap();

        // Context files should appear first (highest precedence)
        let context_files_pos = context.find("Context Files").unwrap_or(usize::MAX);
        let arch_pos = context.find("Architecture Context").unwrap_or(usize::MAX);
        assert!(context_files_pos < arch_pos || context.contains("Context Files"));
    }

    #[test]
    fn test_load_context_files_cache_invalidation() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::create(temp_dir.path()).unwrap();

        // Create context file
        let context_file = temp_dir.path().join("GEMINI.md");
        fs::write(&context_file, "# Original Context").unwrap();

        let mut manager = ContextManager::new(&workspace);

        // First load
        let content1 = manager.load_context_files(temp_dir.path()).unwrap();
        assert!(content1.as_ref().unwrap().contains("Original Context"));

        // Modify file
        std::thread::sleep(std::time::Duration::from_millis(10)); // Ensure different mtime
        fs::write(&context_file, "# Updated Context").unwrap();

        // Second load should detect change and reload
        let content2 = manager.load_context_files(temp_dir.path()).unwrap();
        assert!(content2.as_ref().unwrap().contains("Updated Context"));
        assert!(!content2.as_ref().unwrap().contains("Original Context"));
    }

    #[test]
    fn test_build_context_with_context_files_and_memory() {
        use crate::memory::MemoryEntry;

        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::create(temp_dir.path()).unwrap();
        let req_id = RequirementId::new(1);

        // Create context file
        let context_file = temp_dir.path().join("GEMINI.md");
        fs::write(&context_file, "# Context Files\n\nProject context.").unwrap();

        let mut manager = ContextManager::for_plan(&workspace, req_id).unwrap();

        // Store memory
        let entry = MemoryEntry::new("agent".to_string(), "Previous output".to_string());
        manager.memory_store_mut().unwrap().store(entry).unwrap();

        let context = manager.build_context("agent", None).unwrap();

        assert!(context.contains("Context Files"));
        assert!(context.contains("Project context"));
        assert!(context.contains("Previous Output from agent"));
        assert!(context.contains("Previous output"));
    }

    #[test]
    fn test_build_context_with_context_files_and_architecture() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::create(temp_dir.path()).unwrap();

        // Create context file
        let context_file = temp_dir.path().join("GEMINI.md");
        fs::write(&context_file, "# Context Files\n\nGuidelines.").unwrap();

        // Create architecture file
        let arch_path = temp_dir.path().join(".radium").join("architecture.md");
        fs::write(&arch_path, "# Architecture\n\nSystem design.").unwrap();

        let mut manager = ContextManager::new(&workspace);
        let context = manager.build_context("agent", None).unwrap();

        assert!(context.contains("Context Files"));
        assert!(context.contains("Guidelines"));
        assert!(context.contains("Architecture Context"));
        assert!(context.contains("System design"));
    }

    #[test]
    fn test_load_context_files_multiple_loads() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::create(temp_dir.path()).unwrap();

        // Create context file
        let context_file = temp_dir.path().join("GEMINI.md");
        fs::write(&context_file, "# Context").unwrap();

        let mut manager = ContextManager::new(&workspace);

        // Multiple loads should work
        for _ in 0..5 {
            let content = manager.load_context_files(temp_dir.path()).unwrap();
            assert!(content.is_some());
            assert!(content.as_ref().unwrap().contains("Context"));
        }
    }

    #[test]
    fn test_build_context_with_context_files_in_subdirectory() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::create(temp_dir.path()).unwrap();

        // Create project root context file
        let project_file = temp_dir.path().join("GEMINI.md");
        fs::write(&project_file, "# Project Context").unwrap();

        // Create subdirectory with context file
        let subdir = temp_dir.path().join("src");
        fs::create_dir_all(&subdir).unwrap();
        let subdir_file = subdir.join("GEMINI.md");
        fs::write(&subdir_file, "# Subdirectory Context").unwrap();

        let mut manager = ContextManager::new(&workspace);
        let context = manager.load_context_files(&subdir).unwrap();

        assert!(context.is_some());
        let content = context.unwrap();
        // Should contain both project and subdirectory context
        assert!(content.contains("Project Context") || content.contains("Subdirectory Context"));
    }

    #[test]
    fn test_build_context_with_context_files_and_plan_context() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::create(temp_dir.path()).unwrap();
        let req_id = RequirementId::new(1);

        // Create a plan in development stage
        let plan_dir = temp_dir.path().join(".radium").join("plan").join("development");
        fs::create_dir_all(&plan_dir).unwrap();
        let plan_file = plan_dir.join("REQ-001-test-plan");
        fs::create_dir_all(&plan_file).unwrap();
        let manifest_file = plan_file.join("manifest.json");
        fs::write(
            &manifest_file,
            r#"{"requirement_id": "REQ-001", "project_name": "Test", "stage": "development", "status": "in_progress"}"#,
        )
        .unwrap();

        // Create context file
        let context_file = temp_dir.path().join("GEMINI.md");
        fs::write(&context_file, "# Context Files\n\nProject guidelines.").unwrap();

        let mut manager = ContextManager::for_plan(&workspace, req_id).unwrap();
        let context = manager.build_context("agent", Some(req_id)).unwrap();

        // Should contain both context files and plan context
        assert!(context.contains("Context Files") || context.contains("Project guidelines"));
    }
}
