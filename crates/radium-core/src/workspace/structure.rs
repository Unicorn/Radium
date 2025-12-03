//! Workspace directory structure management.
//!
//! Defines the directory structure for a Radium workspace.

use std::path::{Path, PathBuf};

/// Main internal workspace directory.
pub const DIR_RADIUM: &str = ".radium";

/// Directory name for internal configuration and artifacts.
pub const DIR_INTERNALS: &str = "_internals";

/// Directory name for plans.
pub const DIR_PLAN: &str = "plan";

/// Stage directory name for backlog plans.
pub const STAGE_BACKLOG: &str = "backlog";

/// Stage directory name for development plans.
pub const STAGE_DEVELOPMENT: &str = "development";

/// Stage directory name for review plans.
pub const STAGE_REVIEW: &str = "review";

/// Stage directory name for testing plans.
pub const STAGE_TESTING: &str = "testing";

/// Stage directory name for documentation.
pub const STAGE_DOCS: &str = "docs";

/// Internal directory for artifacts.
const INTERNAL_ARTIFACTS: &str = "artifacts";

/// Internal directory for memory.
const INTERNAL_MEMORY: &str = "memory";

/// Internal directory for logs.
const INTERNAL_LOGS: &str = "logs";

/// Internal directory for prompts.
const INTERNAL_PROMPTS: &str = "prompts";

/// Internal directory for inputs.
const INTERNAL_INPUTS: &str = "inputs";

/// Internal directory for agents.
const INTERNAL_AGENTS: &str = "agents";

/// Workspace directory structure.
///
/// Provides access to all workspace directories and methods to create them.
#[derive(Debug, Clone)]
pub struct WorkspaceStructure {
    root: PathBuf,
}

impl WorkspaceStructure {
    /// Create a new workspace structure accessor.
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self { root: root.as_ref().to_path_buf() }
    }

    /// Get the root directory.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Get the .radium internal directory.
    pub fn radium_root_dir(&self) -> PathBuf {
        self.root.join(DIR_RADIUM)
    }

    /// Get the internals directory (`.radium/_internals`).
    pub fn internals_dir(&self) -> PathBuf {
        self.radium_root_dir().join(DIR_INTERNALS)
    }

    /// Get the plan root directory (`.radium/plan`).
    pub fn plans_root_dir(&self) -> PathBuf {
        self.radium_root_dir().join(DIR_PLAN)
    }

    /// Get the backlog stage directory.
    pub fn backlog_dir(&self) -> PathBuf {
        self.plans_root_dir().join(STAGE_BACKLOG)
    }

    /// Get the development stage directory.
    pub fn development_dir(&self) -> PathBuf {
        self.plans_root_dir().join(STAGE_DEVELOPMENT)
    }

    /// Get the review stage directory.
    pub fn review_dir(&self) -> PathBuf {
        self.plans_root_dir().join(STAGE_REVIEW)
    }

    /// Get the testing stage directory.
    pub fn testing_dir(&self) -> PathBuf {
        self.plans_root_dir().join(STAGE_TESTING)
    }

    /// Get the docs directory.
    pub fn docs_dir(&self) -> PathBuf {
        self.plans_root_dir().join(STAGE_DOCS)
    }

    /// Get all stage directories.
    pub fn stage_dirs(&self) -> Vec<PathBuf> {
        vec![
            self.backlog_dir(),
            self.development_dir(),
            self.review_dir(),
            self.testing_dir(),
            self.docs_dir(),
        ]
    }

    /// Get the artifacts directory.
    pub fn artifacts_dir(&self) -> PathBuf {
        self.internals_dir().join(INTERNAL_ARTIFACTS)
    }

    /// Get the memory directory.
    pub fn memory_dir(&self) -> PathBuf {
        self.internals_dir().join(INTERNAL_MEMORY)
    }

    /// Get the logs directory.
    pub fn logs_dir(&self) -> PathBuf {
        self.internals_dir().join(INTERNAL_LOGS)
    }

    /// Get the prompts directory.
    pub fn prompts_dir(&self) -> PathBuf {
        self.internals_dir().join(INTERNAL_PROMPTS)
    }

    /// Get the inputs directory.
    pub fn inputs_dir(&self) -> PathBuf {
        self.internals_dir().join(INTERNAL_INPUTS)
    }

    /// Get the agents directory.
    pub fn agents_dir(&self) -> PathBuf {
        self.internals_dir().join(INTERNAL_AGENTS)
    }

    /// Get all internal directories.
    pub fn internal_dirs(&self) -> Vec<PathBuf> {
        vec![
            self.artifacts_dir(),
            self.memory_dir(),
            self.logs_dir(),
            self.prompts_dir(),
            self.inputs_dir(),
            self.agents_dir(),
        ]
    }

    /// Create all workspace directories.
    ///
    /// Creates the complete workspace structure, including:
    /// - `_internals/` and subdirectories
    /// - `plan/` and stage directories
    ///
    /// # Errors
    ///
    /// Returns error if any directory cannot be created.
    pub fn create_all(&self) -> std::io::Result<()> {
        // Create .radium directory first
        std::fs::create_dir_all(self.radium_root_dir())?;

        // Create internals directory and its subdirectories
        std::fs::create_dir_all(self.internals_dir())?;
        for dir in self.internal_dirs() {
            std::fs::create_dir_all(&dir)?;
        }

        // Create plans root directory and its stage directories
        std::fs::create_dir_all(self.plans_root_dir())?;
        for dir in self.stage_dirs() {
            std::fs::create_dir_all(&dir)?;
        }

        Ok(())
    }

    /// Check if all required directories exist.
    pub fn is_complete(&self) -> bool {
        if !self.radium_root_dir().exists() {
            return false;
        }

        if !self.internals_dir().exists() {
            return false;
        }

        if !self.plans_root_dir().exists() {
            return false;
        }

        for dir in self.stage_dirs() {
            if !dir.exists() {
                return false;
            }
        }

        for dir in self.internal_dirs() {
            if !dir.exists() {
                return false;
            }
        }

        true
    }

    /// Create a plan directory in a specific stage.
    ///
    /// # Errors
    ///
    /// Returns error if directory cannot be created.
    pub fn create_plan_dir(&self, stage: &str, plan_name: &str) -> std::io::Result<PathBuf> {
        let plan_dir = self.plans_root_dir().join(stage).join(plan_name);
        std::fs::create_dir_all(&plan_dir)?;
        Ok(plan_dir)
    }

    /// Get the path to a plan's memory directory.
    pub fn plan_memory_dir(&self, stage: &str, plan_name: &str) -> PathBuf {
        self.plans_root_dir().join(stage).join(plan_name).join("memory")
    }

    /// Get the path to a plan's artifacts directory.
    pub fn plan_artifacts_dir(&self, stage: &str, plan_name: &str) -> PathBuf {
        self.plans_root_dir().join(stage).join(plan_name).join("artifacts")
    }

    /// Get the path to a plan's prompts directory.
    pub fn plan_prompts_dir(&self, stage: &str, plan_name: &str) -> PathBuf {
        self.plans_root_dir().join(stage).join(plan_name).join("prompts")
    }

    /// Get the path to a plan's plan files directory.
    pub fn plan_files_dir(&self, stage: &str, plan_name: &str) -> PathBuf {
        self.plans_root_dir().join(stage).join(plan_name).join("plan")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_workspace_structure_paths() {
        let temp = TempDir::new().unwrap();
        let structure = WorkspaceStructure::new(temp.path());

        assert_eq!(structure.root(), temp.path());
        assert_eq!(structure.radium_root_dir(), temp.path().join(".radium"));
        assert_eq!(structure.internals_dir(), temp.path().join(".radium/_internals"));
        assert_eq!(structure.plans_root_dir(), temp.path().join(".radium/plan"));
        
        assert_eq!(structure.backlog_dir(), temp.path().join(".radium/plan/backlog"));
        assert_eq!(structure.development_dir(), temp.path().join(".radium/plan/development"));
        assert_eq!(structure.review_dir(), temp.path().join(".radium/plan/review"));
        assert_eq!(structure.testing_dir(), temp.path().join(".radium/plan/testing"));
        assert_eq!(structure.docs_dir(), temp.path().join(".radium/plan/docs"));
    }

    #[test]
    fn test_create_all() {
        let temp = TempDir::new().unwrap();
        let structure = WorkspaceStructure::new(temp.path());

        structure.create_all().unwrap();

        assert!(structure.is_complete());
        assert!(structure.radium_root_dir().exists());
        assert!(structure.internals_dir().exists());
        assert!(structure.plans_root_dir().exists());
        
        assert!(structure.backlog_dir().exists());
        assert!(structure.development_dir().exists());
        
        assert!(structure.artifacts_dir().exists());
        assert!(structure.memory_dir().exists());
    }

    #[test]
    fn test_create_plan_dir() {
        let temp = TempDir::new().unwrap();
        let structure = WorkspaceStructure::new(temp.path());

        structure.create_all().unwrap();

        let plan_dir = structure.create_plan_dir(STAGE_BACKLOG, "REQ-001-test").unwrap();
        assert!(plan_dir.exists());
        assert_eq!(plan_dir, temp.path().join(".radium/plan/backlog/REQ-001-test"));
    }

    #[test]
    fn test_plan_subdirectories() {
        let temp = TempDir::new().unwrap();
        let structure = WorkspaceStructure::new(temp.path());

        let memory_dir = structure.plan_memory_dir(STAGE_BACKLOG, "REQ-001-test");
        let artifacts_dir = structure.plan_artifacts_dir(STAGE_BACKLOG, "REQ-001-test");
        let prompts_dir = structure.plan_prompts_dir(STAGE_BACKLOG, "REQ-001-test");
        let plan_dir = structure.plan_files_dir(STAGE_BACKLOG, "REQ-001-test");

        assert_eq!(memory_dir, temp.path().join(".radium/plan/backlog/REQ-001-test/memory"));
        assert_eq!(artifacts_dir, temp.path().join(".radium/plan/backlog/REQ-001-test/artifacts"));
        assert_eq!(prompts_dir, temp.path().join(".radium/plan/backlog/REQ-001-test/prompts"));
        assert_eq!(plan_dir, temp.path().join(".radium/plan/backlog/REQ-001-test/plan"));
    }
}
