//! Template discovery system.
//!
//! Discovers workflow templates from various locations:
//! - Project-local templates in `./templates/`
//! - User-level templates in `~/.radium/templates/`
//! - Workspace templates if RADIUM_WORKSPACE is set

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use super::templates::{WorkflowTemplate, WorkflowTemplateError};

/// Template discovery system.
#[derive(Debug, Clone)]
pub struct TemplateDiscovery {
    /// Search paths for templates.
    search_paths: Vec<PathBuf>,
}

impl TemplateDiscovery {
    /// Creates a new template discovery instance with default search paths.
    pub fn new() -> Self {
        Self { search_paths: Self::default_search_paths() }
    }

    /// Creates a template discovery instance with custom search paths.
    pub fn with_paths(paths: Vec<PathBuf>) -> Self {
        Self { search_paths: paths }
    }

    /// Returns the default search paths for templates.
    ///
    /// Search order (precedence from highest to lowest):
    /// 1. Project-local templates
    /// 2. User templates
    /// 3. Workspace templates
    /// 4. Project-level extension templates
    /// 5. User-level extension templates
    fn default_search_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // 1. Project-local templates
        if let Ok(cwd) = std::env::current_dir() {
            paths.push(cwd.join("templates"));
        }

        // 2. User templates in home directory
        // Allow env::var for standard HOME environment variable (path discovery)
        #[allow(clippy::disallowed_methods)]
        if let Ok(home) = std::env::var("HOME") {
            paths.push(PathBuf::from(home).join(".radium/templates"));
        }

        // 3. Workspace templates if RADIUM_WORKSPACE is set
        // Allow env::var for RADIUM_WORKSPACE (path discovery, not app config)
        #[allow(clippy::disallowed_methods)]
        if let Ok(workspace) = std::env::var("RADIUM_WORKSPACE") {
            let workspace_path = PathBuf::from(workspace);
            paths.push(workspace_path.join("templates"));
            paths.push(workspace_path.join(".radium/templates"));
        }

        // 4. Project-level extension templates (higher precedence than user extensions)
        if let Ok(cwd) = std::env::current_dir() {
            let project_extensions_dir = cwd.join(".radium").join("extensions");
            if project_extensions_dir.exists() {
                if let Ok(entries) = fs::read_dir(&project_extensions_dir) {
                    for entry in entries.flatten() {
                        let ext_path = entry.path();
                        if ext_path.is_dir() {
                            let templates_dir = ext_path.join("templates");
                            if templates_dir.exists() {
                                paths.push(templates_dir);
                            }
                        }
                    }
                }
            }
        }

        // 5. User-level extension templates (lowest precedence)
        #[allow(clippy::disallowed_methods)]
        if let Ok(home) = std::env::var("HOME") {
            let user_extensions_dir = PathBuf::from(home).join(".radium").join("extensions");
            if user_extensions_dir.exists() {
                if let Ok(entries) = fs::read_dir(&user_extensions_dir) {
                    for entry in entries.flatten() {
                        let ext_path = entry.path();
                        if ext_path.is_dir() {
                            let templates_dir = ext_path.join("templates");
                            if templates_dir.exists() {
                                paths.push(templates_dir);
                            }
                        }
                    }
                }
            }
        }

        paths
    }

    /// Discovers all templates in the search paths.
    ///
    /// Returns a HashMap of template name to WorkflowTemplate.
    pub fn discover_all(&self) -> Result<HashMap<String, WorkflowTemplate>, WorkflowTemplateError> {
        let mut templates = HashMap::new();

        for search_path in &self.search_paths {
            if !search_path.exists() {
                continue;
            }

            if let Ok(entries) = fs::read_dir(search_path) {
                for entry in entries.flatten() {
                    let path = entry.path();

                    // Only process .json files
                    if path.extension().and_then(|s| s.to_str()) != Some("json") {
                        continue;
                    }

                    // Try to load the template
                    if let Ok(template) = WorkflowTemplate::load_from_file(&path) {
                        templates.insert(template.name.clone(), template);
                    }
                }
            }
        }

        Ok(templates)
    }

    /// Finds a template by name.
    pub fn find_by_name(
        &self,
        name: &str,
    ) -> Result<Option<WorkflowTemplate>, WorkflowTemplateError> {
        for search_path in &self.search_paths {
            if !search_path.exists() {
                continue;
            }

            // Try both exact match and with .json extension
            let candidates =
                vec![search_path.join(name), search_path.join(format!("{}.json", name))];

            for candidate in candidates {
                if candidate.exists() && candidate.is_file() {
                    return Ok(Some(WorkflowTemplate::load_from_file(&candidate)?));
                }
            }
        }

        Ok(None)
    }

    /// Returns the search paths.
    pub fn search_paths(&self) -> &[PathBuf] {
        &self.search_paths
    }
}

impl Default for TemplateDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_discovery_new() {
        let discovery = TemplateDiscovery::new();
        assert!(!discovery.search_paths().is_empty());
    }

    #[test]
    fn test_template_discovery_with_paths() {
        let paths = vec![PathBuf::from("/custom/path")];
        let discovery = TemplateDiscovery::with_paths(paths.clone());
        assert_eq!(discovery.search_paths(), paths.as_slice());
    }

    #[test]
    fn test_discover_all_empty() {
        let discovery = TemplateDiscovery::with_paths(vec![PathBuf::from("/nonexistent")]);
        let templates = discovery.discover_all().unwrap();
        assert!(templates.is_empty());
    }
}
