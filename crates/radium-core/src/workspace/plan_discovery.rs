//! Plan discovery and management across workspace stages.
//!
//! Provides functionality to discover plans across all workspace stages
//! (backlog, development, review, testing, docs) and load their metadata.

use crate::models::plan::{Plan, PlanManifest};
use crate::workspace::{RequirementId, Workspace, WorkspaceError};
use std::fs;
use std::path::{Path, PathBuf};

/// Discovered plan information.
#[derive(Debug, Clone)]
pub struct DiscoveredPlan {
    /// Plan metadata.
    pub plan: Plan,

    /// Path to the plan directory.
    pub path: PathBuf,

    /// Whether the plan has a valid manifest file.
    pub has_manifest: bool,
}

impl DiscoveredPlan {
    /// Load the plan manifest.
    ///
    /// # Errors
    ///
    /// Returns error if manifest cannot be loaded.
    pub fn load_manifest(&self) -> Result<PlanManifest, WorkspaceError> {
        let manifest_path = self.path.join("plan/plan_manifest.json");
        if !manifest_path.exists() {
            return Err(WorkspaceError::InvalidStructure(format!(
                "manifest not found at {}",
                manifest_path.display()
            )));
        }

        let content = fs::read_to_string(&manifest_path)?;
        let manifest: PlanManifest = serde_json::from_str(&content)?;
        Ok(manifest)
    }
}

/// Plan discovery options.
#[derive(Debug, Clone)]
pub struct PlanDiscoveryOptions {
    /// Filter by stage name.
    pub stage: Option<String>,

    /// Sort by field (created_at, updated_at, requirement_id).
    pub sort_by: SortBy,

    /// Sort order.
    pub sort_order: SortOrder,
}

impl Default for PlanDiscoveryOptions {
    fn default() -> Self {
        Self { stage: None, sort_by: SortBy::UpdatedAt, sort_order: SortOrder::Descending }
    }
}

/// Sort field.
#[derive(Debug, Clone, Copy)]
pub enum SortBy {
    /// Sort by creation time.
    CreatedAt,

    /// Sort by last updated time.
    UpdatedAt,

    /// Sort by requirement ID.
    RequirementId,
}

/// Sort order.
#[derive(Debug, Clone, Copy)]
pub enum SortOrder {
    /// Ascending order.
    Ascending,

    /// Descending order.
    Descending,
}

/// Plan discovery service.
pub struct PlanDiscovery<'a> {
    workspace: &'a Workspace,
}

impl<'a> PlanDiscovery<'a> {
    /// Create a new plan discovery service.
    pub fn new(workspace: &'a Workspace) -> Self {
        Self { workspace }
    }

    /// Discover all plans in the workspace.
    ///
    /// # Errors
    ///
    /// Returns error if plans cannot be discovered.
    pub fn discover_all(&self) -> Result<Vec<DiscoveredPlan>, WorkspaceError> {
        self.discover_with_options(&PlanDiscoveryOptions::default())
    }

    /// Discover plans with custom options.
    ///
    /// # Errors
    ///
    /// Returns error if plans cannot be discovered.
    pub fn discover_with_options(
        &self,
        options: &PlanDiscoveryOptions,
    ) -> Result<Vec<DiscoveredPlan>, WorkspaceError> {
        let mut plans = Vec::new();

        let stage_dirs = if let Some(stage) = &options.stage {
            vec![self.workspace.stage_dir(stage)]
        } else {
            self.workspace.stage_dirs()
        };

        for stage_dir in stage_dirs {
            if !stage_dir.exists() {
                continue;
            }

            for entry in fs::read_dir(&stage_dir)? {
                let entry = entry?;
                let path = entry.path();

                if !path.is_dir() {
                    continue;
                }

                // Check if this looks like a plan directory (REQ-XXX format)
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if !name.starts_with("REQ-") {
                        continue;
                    }

                    if let Some(discovered) = self.load_plan(&path)? {
                        plans.push(discovered);
                    }
                }
            }
        }

        // Sort plans
        self.sort_plans(&mut plans, options);

        Ok(plans)
    }

    /// Find a plan by requirement ID.
    ///
    /// # Errors
    ///
    /// Returns error if plan cannot be found or loaded.
    pub fn find_by_requirement_id(
        &self,
        req_id: RequirementId,
    ) -> Result<Option<DiscoveredPlan>, WorkspaceError> {
        let req_id_str = req_id.to_string();

        for stage_dir in self.workspace.stage_dirs() {
            if !stage_dir.exists() {
                continue;
            }

            for entry in fs::read_dir(&stage_dir)? {
                let entry = entry?;
                let path = entry.path();

                if !path.is_dir() {
                    continue;
                }

                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with(&req_id_str) {
                        return self.load_plan(&path);
                    }
                }
            }
        }

        Ok(None)
    }

    /// Find a plan by folder name.
    ///
    /// # Errors
    ///
    /// Returns error if plan cannot be found or loaded.
    pub fn find_by_folder_name(
        &self,
        folder_name: &str,
    ) -> Result<Option<DiscoveredPlan>, WorkspaceError> {
        for stage_dir in self.workspace.stage_dirs() {
            if !stage_dir.exists() {
                continue;
            }

            let plan_path = stage_dir.join(folder_name);
            if plan_path.exists() && plan_path.is_dir() {
                return self.load_plan(&plan_path);
            }
        }

        Ok(None)
    }

    /// Load a plan from a directory.
    fn load_plan(&self, path: &Path) -> Result<Option<DiscoveredPlan>, WorkspaceError> {
        let plan_json_path = path.join("plan.json");
        if !plan_json_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&plan_json_path)?;
        let plan: Plan = serde_json::from_str(&content)?;

        let manifest_path = path.join("plan/plan_manifest.json");
        let has_manifest = manifest_path.exists();

        Ok(Some(DiscoveredPlan { plan, path: path.to_path_buf(), has_manifest }))
    }

    /// Sort plans according to options.
    fn sort_plans(&self, plans: &mut [DiscoveredPlan], options: &PlanDiscoveryOptions) {
        plans.sort_by(|a, b| {
            let cmp = match options.sort_by {
                SortBy::CreatedAt => a.plan.created_at.cmp(&b.plan.created_at),
                SortBy::UpdatedAt => a.plan.updated_at.cmp(&b.plan.updated_at),
                SortBy::RequirementId => a.plan.requirement_id.cmp(&b.plan.requirement_id),
            };

            match options.sort_order {
                SortOrder::Ascending => cmp,
                SortOrder::Descending => cmp.reverse(),
            }
        });
    }
}

impl Workspace {
    /// Get the plan discovery service for this workspace.
    pub fn plans(&self) -> PlanDiscovery<'_> {
        PlanDiscovery::new(self)
    }

    /// Discover all plans in the workspace.
    ///
    /// # Errors
    ///
    /// Returns error if plans cannot be discovered.
    pub fn discover_plans(&self) -> Result<Vec<DiscoveredPlan>, WorkspaceError> {
        self.plans().discover_all()
    }

    /// Find a plan by requirement ID.
    ///
    /// # Errors
    ///
    /// Returns error if plan cannot be found or loaded.
    pub fn find_plan_by_id(
        &self,
        req_id: RequirementId,
    ) -> Result<Option<DiscoveredPlan>, WorkspaceError> {
        self.plans().find_by_requirement_id(req_id)
    }

    /// Find a plan by folder name.
    ///
    /// # Errors
    ///
    /// Returns error if plan cannot be found or loaded.
    pub fn find_plan_by_folder(
        &self,
        folder_name: &str,
    ) -> Result<Option<DiscoveredPlan>, WorkspaceError> {
        self.plans().find_by_folder_name(folder_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::plan::{Iteration, PlanManifest, PlanStatus};
    use std::str::FromStr;
    use tempfile::TempDir;

    fn create_test_plan(
        workspace: &Workspace,
        stage: &str,
        req_id: RequirementId,
        folder_name: &str,
    ) {
        let plan_dir = workspace.structure().create_plan_dir(stage, folder_name).unwrap();

        // Create plan.json
        let plan = Plan::new(
            req_id,
            "Test Project".to_string(),
            folder_name.to_string(),
            stage.to_string(),
        );
        let plan_json = serde_json::to_string_pretty(&plan).unwrap();
        fs::write(plan_dir.join("plan.json"), plan_json).unwrap();

        // Create plan manifest
        let plan_files_dir = plan_dir.join("plan");
        fs::create_dir_all(&plan_files_dir).unwrap();

        let mut manifest = PlanManifest::new(req_id, "Test Project".to_string());
        let mut iter = Iteration::new(1, "Iteration 1".to_string());
        iter.status = PlanStatus::NotStarted;
        manifest.add_iteration(iter);

        let manifest_json = serde_json::to_string_pretty(&manifest).unwrap();
        fs::write(plan_files_dir.join("plan_manifest.json"), manifest_json).unwrap();
    }

    #[test]
    fn test_discover_all_plans() {
        let temp = TempDir::new().unwrap();
        let workspace = Workspace::create(temp.path()).unwrap();

        // Create test plans
        let req1 = RequirementId::from_str("REQ-001").unwrap();
        let req2 = RequirementId::from_str("REQ-002").unwrap();
        let req3 = RequirementId::from_str("REQ-003").unwrap();

        create_test_plan(&workspace, "backlog", req1, "REQ-001-test1");
        create_test_plan(&workspace, "development", req2, "REQ-002-test2");
        create_test_plan(&workspace, "review", req3, "REQ-003-test3");

        let plans = workspace.discover_plans().unwrap();
        assert_eq!(plans.len(), 3);
    }

    #[test]
    fn test_find_by_requirement_id() {
        let temp = TempDir::new().unwrap();
        let workspace = Workspace::create(temp.path()).unwrap();

        let req_id = RequirementId::from_str("REQ-042").unwrap();
        create_test_plan(&workspace, "backlog", req_id, "REQ-042-test");

        let found = workspace.find_plan_by_id(req_id).unwrap();
        assert!(found.is_some());

        let plan = found.unwrap();
        assert_eq!(plan.plan.requirement_id, req_id);
        assert!(plan.has_manifest);
    }

    #[test]
    fn test_find_by_folder_name() {
        let temp = TempDir::new().unwrap();
        let workspace = Workspace::create(temp.path()).unwrap();

        let req_id = RequirementId::from_str("REQ-001").unwrap();
        create_test_plan(&workspace, "backlog", req_id, "REQ-001-myproject");

        let found = workspace.find_plan_by_folder("REQ-001-myproject").unwrap();
        assert!(found.is_some());

        let plan = found.unwrap();
        assert_eq!(plan.plan.folder_name, "REQ-001-myproject");
    }

    #[test]
    fn test_load_manifest() {
        let temp = TempDir::new().unwrap();
        let workspace = Workspace::create(temp.path()).unwrap();

        let req_id = RequirementId::from_str("REQ-001").unwrap();
        create_test_plan(&workspace, "backlog", req_id, "REQ-001-test");

        let found = workspace.find_plan_by_id(req_id).unwrap().unwrap();
        let manifest = found.load_manifest().unwrap();

        assert_eq!(manifest.requirement_id, req_id);
        assert_eq!(manifest.iterations.len(), 1);
    }

    #[test]
    fn test_discover_with_stage_filter() {
        let temp = TempDir::new().unwrap();
        let workspace = Workspace::create(temp.path()).unwrap();

        let req1 = RequirementId::from_str("REQ-001").unwrap();
        let req2 = RequirementId::from_str("REQ-002").unwrap();

        create_test_plan(&workspace, "backlog", req1, "REQ-001-test1");
        create_test_plan(&workspace, "development", req2, "REQ-002-test2");

        let options =
            PlanDiscoveryOptions { stage: Some("backlog".to_string()), ..Default::default() };

        let plans = workspace.plans().discover_with_options(&options).unwrap();
        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].plan.stage, "backlog");
    }
}
