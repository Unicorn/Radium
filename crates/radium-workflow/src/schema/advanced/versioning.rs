//! Workflow Versioning
//!
//! Implement workflow versioning for safe deployments:
//! - Version change points
//! - Version branching
//! - Migration paths
//! - Deterministic version checks

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

/// Version information for a workflow release
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionInfo {
    /// Version string (semver recommended)
    pub version: String,

    /// Release date
    pub released_at: DateTime<Utc>,

    /// Description of changes
    pub description: String,

    /// Whether this version has breaking changes
    #[serde(default)]
    pub breaking_changes: bool,

    /// List of change IDs introduced in this version
    #[serde(default)]
    pub changes: Vec<String>,
}

impl VersionInfo {
    /// Create new version info
    pub fn new(
        version: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            version: version.into(),
            released_at: Utc::now(),
            description: description.into(),
            breaking_changes: false,
            changes: vec![],
        }
    }

    /// Mark as breaking change
    pub fn breaking(mut self) -> Self {
        self.breaking_changes = true;
        self
    }

    /// Add change IDs
    pub fn with_changes(mut self, changes: Vec<impl Into<String>>) -> Self {
        self.changes = changes.into_iter().map(|c| c.into()).collect();
        self
    }
}

/// A version branch for handling different code paths
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionBranch {
    /// Minimum version for this branch (inclusive)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_version: Option<String>,

    /// Maximum version for this branch (exclusive)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_version: Option<String>,

    /// TypeScript code for this branch
    pub code: String,

    /// Description of this branch
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl VersionBranch {
    /// Create branch for versions >= min
    pub fn from_version(min: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            min_version: Some(min.into()),
            max_version: None,
            code: code.into(),
            description: None,
        }
    }

    /// Create branch for versions < max
    pub fn before_version(max: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            min_version: None,
            max_version: Some(max.into()),
            code: code.into(),
            description: None,
        }
    }

    /// Create branch for version range
    pub fn between(
        min: impl Into<String>,
        max: impl Into<String>,
        code: impl Into<String>,
    ) -> Self {
        Self {
            min_version: Some(min.into()),
            max_version: Some(max.into()),
            code: code.into(),
            description: None,
        }
    }

    /// Add description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// A version change point where behavior differs between versions
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct VersionChangePoint {
    /// Unique identifier for this change point
    #[validate(length(min = 1))]
    pub change_id: String,

    /// Version where this change was introduced
    pub introduced_version: String,

    /// Description of the change
    pub description: String,

    /// Branches for different versions
    #[validate(length(min = 1, message = "At least one branch required"))]
    pub branches: Vec<VersionBranch>,
}

impl VersionChangePoint {
    /// Create a new change point
    pub fn new(
        change_id: impl Into<String>,
        introduced_version: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            change_id: change_id.into(),
            introduced_version: introduced_version.into(),
            description: description.into(),
            branches: vec![],
        }
    }

    /// Add a version branch
    pub fn with_branch(mut self, branch: VersionBranch) -> Self {
        self.branches.push(branch);
        self
    }

    /// Generate TypeScript code for this change point
    pub fn to_typescript(&self) -> String {
        let mut code = String::new();

        // Add documentation comment
        code.push_str(&format!(
            r#"// Change point: {}
// Introduced in version: {}
// {}
"#,
            self.change_id, self.introduced_version, self.description
        ));

        // Generate patched() call
        let camel_id = to_camel_case(&self.change_id);
        code.push_str(&format!(
            "if (patched('{}')) {{\n",
            self.change_id
        ));

        // New version branch (first)
        if let Some(new_branch) = self.branches.first() {
            code.push_str(&format!("  // New behavior\n"));
            code.push_str(&format!("  {}\n", new_branch.code));
        }

        code.push_str("} else {\n");

        // Old version branch (second)
        if let Some(old_branch) = self.branches.get(1) {
            code.push_str(&format!("  // Legacy behavior\n"));
            code.push_str(&format!("  {}\n", old_branch.code));
        } else {
            code.push_str("  // No legacy behavior defined\n");
        }

        code.push_str("}\n");

        code
    }
}

/// Versioning configuration for a workflow
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct VersioningConfig {
    /// Current version identifier
    #[validate(length(min = 1))]
    pub current_version: String,

    /// Version history
    #[serde(default)]
    pub version_history: Vec<VersionInfo>,

    /// Version change points
    #[serde(default)]
    pub change_points: HashMap<String, VersionChangePoint>,
}

impl VersioningConfig {
    /// Create a new versioning config
    pub fn new(current_version: impl Into<String>) -> Self {
        Self {
            current_version: current_version.into(),
            version_history: vec![],
            change_points: HashMap::new(),
        }
    }

    /// Add version to history
    pub fn add_version(&mut self, version: VersionInfo) {
        self.version_history.push(version);
    }

    /// Add a change point
    pub fn add_change_point(&mut self, change_point: VersionChangePoint) {
        self.change_points
            .insert(change_point.change_id.clone(), change_point);
    }

    /// Get a change point by ID
    pub fn get_change_point(&self, id: &str) -> Option<&VersionChangePoint> {
        self.change_points.get(id)
    }

    /// Generate TypeScript imports for versioning
    pub fn typescript_imports() -> &'static str {
        "import { patched, deprecatePatch } from '@temporalio/workflow';"
    }

    /// Generate TypeScript version constant
    pub fn to_typescript_version_constant(&self) -> String {
        format!(
            "const WORKFLOW_VERSION = '{}';\n",
            self.current_version
        )
    }

    /// Generate TypeScript for all change points
    pub fn to_typescript_change_points(&self) -> String {
        let mut code = String::new();

        for change_point in self.change_points.values() {
            code.push_str(&change_point.to_typescript());
            code.push('\n');
        }

        code
    }
}

impl Default for VersioningConfig {
    fn default() -> Self {
        Self::new("1.0.0")
    }
}

/// Helper to create common versioning patterns
pub mod patterns {
    use super::*;

    /// Create a change point for adding a new feature
    pub fn new_feature(
        change_id: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
        new_code: impl Into<String>,
    ) -> VersionChangePoint {
        VersionChangePoint::new(change_id, version, description)
            .with_branch(VersionBranch::from_version("1.0.0", new_code))
            .with_branch(VersionBranch::before_version("1.0.0", "// Feature not available"))
    }

    /// Create a change point for a bug fix
    pub fn bug_fix(
        change_id: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
        fixed_code: impl Into<String>,
        buggy_code: impl Into<String>,
    ) -> VersionChangePoint {
        VersionChangePoint::new(change_id, version, description)
            .with_branch(VersionBranch::from_version("1.0.0", fixed_code).with_description("Fixed behavior"))
            .with_branch(VersionBranch::before_version("1.0.0", buggy_code).with_description("Original buggy behavior"))
    }

    /// Create a change point for behavior change
    pub fn behavior_change(
        change_id: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
        new_behavior: impl Into<String>,
        old_behavior: impl Into<String>,
    ) -> VersionChangePoint {
        VersionChangePoint::new(change_id, version, description)
            .with_branch(VersionBranch::from_version("1.0.0", new_behavior).with_description("New behavior"))
            .with_branch(VersionBranch::before_version("1.0.0", old_behavior).with_description("Legacy behavior"))
    }
}

// Helper functions
fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for (i, c) in s.chars().enumerate() {
        if c == '-' || c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else if i == 0 {
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info() {
        let version = VersionInfo::new("1.0.0", "Initial release")
            .breaking()
            .with_changes(vec!["change-1", "change-2"]);

        assert_eq!(version.version, "1.0.0");
        assert!(version.breaking_changes);
        assert_eq!(version.changes.len(), 2);
    }

    #[test]
    fn test_version_branch_from_version() {
        let branch = VersionBranch::from_version("2.0.0", "await newFeature();")
            .with_description("New feature implementation");

        assert_eq!(branch.min_version, Some("2.0.0".to_string()));
        assert!(branch.max_version.is_none());
        assert!(branch.description.is_some());
    }

    #[test]
    fn test_version_branch_before_version() {
        let branch = VersionBranch::before_version("2.0.0", "await legacyFeature();");

        assert!(branch.min_version.is_none());
        assert_eq!(branch.max_version, Some("2.0.0".to_string()));
    }

    #[test]
    fn test_version_branch_between() {
        let branch = VersionBranch::between("1.0.0", "2.0.0", "await v1Feature();");

        assert_eq!(branch.min_version, Some("1.0.0".to_string()));
        assert_eq!(branch.max_version, Some("2.0.0".to_string()));
    }

    #[test]
    fn test_version_change_point() {
        let change = VersionChangePoint::new(
            "new-validation",
            "2.0.0",
            "Added input validation",
        )
        .with_branch(VersionBranch::from_version("2.0.0", "validateInput(input);"))
        .with_branch(VersionBranch::before_version("2.0.0", "// No validation"));

        assert_eq!(change.change_id, "new-validation");
        assert_eq!(change.branches.len(), 2);
    }

    #[test]
    fn test_version_change_point_to_typescript() {
        let change = VersionChangePoint::new("test-change", "1.0.0", "Test change")
            .with_branch(VersionBranch::from_version("1.0.0", "newCode();"))
            .with_branch(VersionBranch::before_version("1.0.0", "oldCode();"));

        let ts = change.to_typescript();

        assert!(ts.contains("patched('test-change')"));
        assert!(ts.contains("newCode()"));
        assert!(ts.contains("oldCode()"));
    }

    #[test]
    fn test_versioning_config() {
        let mut config = VersioningConfig::new("2.0.0");

        config.add_version(VersionInfo::new("1.0.0", "Initial release"));
        config.add_version(VersionInfo::new("2.0.0", "Major update").breaking());

        config.add_change_point(VersionChangePoint::new(
            "feature-x",
            "2.0.0",
            "New feature X",
        ));

        assert_eq!(config.current_version, "2.0.0");
        assert_eq!(config.version_history.len(), 2);
        assert!(config.get_change_point("feature-x").is_some());
    }

    #[test]
    fn test_versioning_config_typescript() {
        let config = VersioningConfig::new("1.5.0");

        let version_const = config.to_typescript_version_constant();
        assert!(version_const.contains("1.5.0"));

        let imports = VersioningConfig::typescript_imports();
        assert!(imports.contains("patched"));
    }

    #[test]
    fn test_patterns_new_feature() {
        let change = patterns::new_feature(
            "new-api",
            "2.0.0",
            "New API endpoint",
            "await newApi();",
        );

        assert_eq!(change.change_id, "new-api");
        assert_eq!(change.branches.len(), 2);
    }

    #[test]
    fn test_patterns_bug_fix() {
        let change = patterns::bug_fix(
            "fix-calculation",
            "1.1.0",
            "Fixed calculation error",
            "return x * 2;",
            "return x * 3;",
        );

        assert_eq!(change.branches.len(), 2);
        assert!(change.branches[0].description.is_some());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let config = VersioningConfig::new("1.0.0");

        let json = serde_json::to_string(&config).unwrap();
        let restored: VersioningConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.current_version, restored.current_version);
    }
}
