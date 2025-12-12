//! Project scanning tool for comprehensive project analysis
//!
//! This module provides the `project_scan` tool that analyzes a project
//! by reading README, manifest files, directory structure, and other metadata.

use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::process::Command;

use super::file_tools::WorkspaceRootProvider;
use super::tool::{Tool, ToolArguments, ToolHandler, ToolParameters, ToolResult};
use crate::error::{OrchestrationError, Result};

/// Project scan operation handler
struct ProjectScanHandler {
    /// Workspace root provider
    workspace_root: Arc<dyn WorkspaceRootProvider>,
}

#[async_trait]
impl ToolHandler for ProjectScanHandler {
    async fn execute(&self, args: &ToolArguments) -> Result<ToolResult> {
        let workspace_root = self.workspace_root.workspace_root().ok_or_else(|| {
            OrchestrationError::Other("Workspace root not available".to_string())
        })?;

        let depth = args.get_string("depth").unwrap_or_else(|| "quick".to_string());

        let mut scan_result = String::new();
        scan_result.push_str("# Project Scan Results\n\n");

        // 1. Find and read README
        if let Some(readme_content) = find_and_read_readme(&workspace_root).await {
            scan_result.push_str("## README\n\n");
            scan_result.push_str("```\n");
            scan_result.push_str(&readme_content);
            scan_result.push_str("\n```\n\n");
        }

        // 2. Find and read manifest
        if let Some((manifest_name, manifest_content)) = find_and_read_manifest(&workspace_root).await {
            scan_result.push_str(&format!("## {}\n\n", manifest_name));
            scan_result.push_str("```\n");
            scan_result.push_str(&manifest_content);
            scan_result.push_str("\n```\n\n");
        }

        // 3. Directory structure (always included for now, even in quick mode)
        if let Ok(dir_listing) = get_directory_listing(&workspace_root).await {
            scan_result.push_str("## Directory Structure\n\n");
            scan_result.push_str("```\n");
            scan_result.push_str(&dir_listing);
            scan_result.push_str("\n```\n\n");
        }

        if depth == "full" {
            // 4. Git status (if available)
            if let Ok(git_status) = get_git_status(&workspace_root).await {
                scan_result.push_str("## Git Status\n\n");
                scan_result.push_str("```\n");
                scan_result.push_str(&git_status);
                scan_result.push_str("\n```\n\n");
            }

            // 5. File counts by extension
            if let Ok(file_counts) = count_files_by_type(&workspace_root).await {
                scan_result.push_str("## File Statistics\n\n");
                scan_result.push_str(&file_counts);
                scan_result.push_str("\n");
            }

            // 6. Detect frameworks/technologies
            if let Some(tech_stack) = detect_tech_stack(&workspace_root).await {
                scan_result.push_str("## Detected Technologies\n\n");
                scan_result.push_str(&tech_stack);
                scan_result.push_str("\n");
            }
        }

        Ok(ToolResult::success(scan_result))
    }
}

/// Find and read the README file (searches for common patterns)
pub async fn find_and_read_readme(workspace_root: &Path) -> Option<String> {
    let patterns = vec!["README.md", "README", "readme.md", "readme", "Readme.md"];

    for pattern in patterns {
        let path = workspace_root.join(pattern);
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path).await {
                // Limit README size to 5000 characters to avoid overwhelming output
                let trimmed = if content.len() > 5000 {
                    format!("{}...\n\n[README truncated - {} total characters]",
                           &content[..5000], content.len())
                } else {
                    content
                };
                return Some(trimmed);
            }
        }
    }

    None
}

/// Find and read the project manifest file
pub async fn find_and_read_manifest(workspace_root: &Path) -> Option<(String, String)> {
    let manifests = vec![
        "Cargo.toml",
        "package.json",
        "pyproject.toml",
        "go.mod",
        "pom.xml",
        "build.gradle",
        "Gemfile",
    ];

    for manifest in manifests {
        let path = workspace_root.join(manifest);
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path).await {
                // Limit manifest size
                let trimmed = if content.len() > 3000 {
                    format!("{}...\n\n[Manifest truncated - {} total characters]",
                           &content[..3000], content.len())
                } else {
                    content
                };
                return Some((manifest.to_string(), trimmed));
            }
        }
    }

    None
}

/// Get directory listing using ls command
pub async fn get_directory_listing(workspace_root: &Path) -> Result<String> {
    let output = Command::new("ls")
        .arg("-la")
        .current_dir(workspace_root)
        .output()
        .await
        .map_err(|e| OrchestrationError::Other(format!("Failed to run ls: {}", e)))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(OrchestrationError::Other("ls command failed".to_string()))
    }
}

/// Get git status
pub async fn get_git_status(workspace_root: &Path) -> Result<String> {
    let output = Command::new("git")
        .arg("status")
        .current_dir(workspace_root)
        .output()
        .await
        .map_err(|e| OrchestrationError::Other(format!("Failed to run git status: {}", e)))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(OrchestrationError::Other("git status failed (not a git repository?)".to_string()))
    }
}

/// Count files by extension
pub async fn count_files_by_type(workspace_root: &Path) -> Result<String> {
    // Use find command to count files by extension
    let output = Command::new("find")
        .arg(".")
        .arg("-type")
        .arg("f")
        .arg("!")
        .arg("-path")
        .arg("*/.*")  // Exclude hidden directories
        .arg("!")
        .arg("-path")
        .arg("*/node_modules/*")  // Exclude node_modules
        .arg("!")
        .arg("-path")
        .arg("*/target/*")  // Exclude Rust target
        .arg("!")
        .arg("-path")
        .arg("*/.git/*")  // Exclude .git
        .current_dir(workspace_root)
        .output()
        .await
        .map_err(|e| OrchestrationError::Other(format!("Failed to run find: {}", e)))?;

    if !output.status.success() {
        return Err(OrchestrationError::Other("find command failed".to_string()));
    }

    let files = String::from_utf8_lossy(&output.stdout);
    let mut extension_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for line in files.lines() {
        if let Some(ext) = PathBuf::from(line).extension() {
            let ext_str = ext.to_string_lossy().to_string();
            *extension_counts.entry(ext_str).or_insert(0) += 1;
        }
    }

    // Sort by count descending
    let mut counts: Vec<_> = extension_counts.iter().collect();
    counts.sort_by(|a, b| b.1.cmp(a.1));

    let mut result = String::new();
    for (ext, count) in counts.iter().take(15) {  // Top 15 extensions
        result.push_str(&format!("- .{}: {} files\n", ext, count));
    }

    Ok(result)
}

/// Detect tech stack based on files present
pub async fn detect_tech_stack(workspace_root: &Path) -> Option<String> {
    let mut technologies = Vec::new();

    // Check for Rust
    if workspace_root.join("Cargo.toml").exists() {
        technologies.push("Rust");
    }

    // Check for Node.js
    if workspace_root.join("package.json").exists() {
        technologies.push("Node.js/JavaScript");
    }

    // Check for Python
    if workspace_root.join("pyproject.toml").exists() || workspace_root.join("requirements.txt").exists() {
        technologies.push("Python");
    }

    // Check for Go
    if workspace_root.join("go.mod").exists() {
        technologies.push("Go");
    }

    // Check for Java
    if workspace_root.join("pom.xml").exists() || workspace_root.join("build.gradle").exists() {
        technologies.push("Java");
    }

    // Check for Ruby
    if workspace_root.join("Gemfile").exists() {
        technologies.push("Ruby");
    }

    // Check for Docker
    if workspace_root.join("Dockerfile").exists() || workspace_root.join("docker-compose.yml").exists() {
        technologies.push("Docker");
    }

    // Check for Git
    if workspace_root.join(".git").exists() {
        technologies.push("Git");
    }

    if technologies.is_empty() {
        None
    } else {
        Some(technologies.join(", "))
    }
}

/// Create the project_scan tool
pub fn create_project_scan_tool(workspace_root: Arc<dyn WorkspaceRootProvider>) -> Tool {
    let parameters = ToolParameters::new()
        .add_property(
            "depth",
            "string",
            "Scan depth: 'quick' (README + manifest only) or 'full' (includes git status, file stats, tech detection)",
            false,
        );

    let handler = Arc::new(ProjectScanHandler { workspace_root });

    Tool::new(
        "project_scan",
        "project_scan",
        "Comprehensive project analysis: reads README, manifest files, analyzes structure, detects tech stack",
        parameters,
        handler,
    )
}

/// Create all project analysis tools
pub fn create_project_analysis_tools(workspace_root: Arc<dyn WorkspaceRootProvider>) -> Vec<Tool> {
    vec![create_project_scan_tool(workspace_root)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    struct TestWorkspaceRoot {
        root: PathBuf,
    }

    impl WorkspaceRootProvider for TestWorkspaceRoot {
        fn workspace_root(&self) -> Option<PathBuf> {
            Some(self.root.clone())
        }
    }

    #[tokio::test]
    async fn test_find_readme() {
        let temp_dir = TempDir::new().unwrap();
        let readme_path = temp_dir.path().join("README.md");
        tokio::fs::write(&readme_path, "# Test Project\n\nThis is a test.").await.unwrap();

        let content = find_and_read_readme(temp_dir.path()).await;
        assert!(content.is_some());
        assert!(content.unwrap().contains("Test Project"));
    }

    #[tokio::test]
    async fn test_find_manifest() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_path = temp_dir.path().join("Cargo.toml");
        tokio::fs::write(&cargo_path, "[package]\nname = \"test\"\nversion = \"0.1.0\"").await.unwrap();

        let manifest = find_and_read_manifest(temp_dir.path()).await;
        assert!(manifest.is_some());
        let (name, content) = manifest.unwrap();
        assert_eq!(name, "Cargo.toml");
        assert!(content.contains("test"));
    }

    #[tokio::test]
    async fn test_project_scan_tool() {
        let temp_dir = TempDir::new().unwrap();

        // Create test files
        tokio::fs::write(temp_dir.path().join("README.md"), "# Test\nA test project").await.unwrap();
        tokio::fs::write(temp_dir.path().join("Cargo.toml"), "[package]\nname = \"test\"").await.unwrap();

        let workspace_root = Arc::new(TestWorkspaceRoot {
            root: temp_dir.path().to_path_buf(),
        });

        let tool = create_project_scan_tool(workspace_root);
        let args = ToolArguments::new(serde_json::json!({
            "depth": "quick"
        }));

        let result = tool.execute(&args).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("README"));
        assert!(result.output.contains("Cargo.toml"));
    }
}
