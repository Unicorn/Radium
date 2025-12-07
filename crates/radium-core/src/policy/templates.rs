//! Policy rule templates for common security scenarios.

use crate::policy::{PolicyEngine, PolicyError, PolicyResult};
use crate::workspace::Workspace;
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;

/// Policy template metadata.
#[derive(Debug, Clone)]
pub struct PolicyTemplate {
    /// Template name (e.g., "development", "production").
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Path to template file.
    pub path: PathBuf,
    /// Template content (loaded on demand).
    content: Option<String>,
}

impl PolicyTemplate {
    /// Creates a new policy template.
    pub fn new(name: String, description: String, path: PathBuf) -> Self {
        Self { name, description, path, content: None }
    }

    /// Loads template content from disk.
    ///
    /// # Returns
    /// Template content as string, or error if file read fails.
    pub fn load_content(&mut self) -> std::io::Result<String> {
        if let Some(ref content) = self.content {
            return Ok(content.clone());
        }
        let content = fs::read_to_string(&self.path)?;
        self.content = Some(content.clone());
        Ok(content)
    }

    /// Validates template by parsing it with PolicyEngine.
    ///
    /// # Returns
    /// `Ok(())` if template is valid, or error if validation fails.
    pub fn validate(&mut self) -> PolicyResult<()> {
        let content = self.load_content().map_err(|e| {
            PolicyError::InvalidConfiguration(format!("Failed to read template: {}", e))
        })?;
        
        // Try to parse as policy
        PolicyEngine::from_str(&content)?;
        Ok(())
    }

    /// Gets template content without loading it into cache.
    pub fn get_content(&self) -> std::io::Result<String> {
        fs::read_to_string(&self.path)
    }
}

/// Template discovery and management.
pub struct TemplateDiscovery {
    /// Templates directory.
    templates_dir: PathBuf,
    /// Cached templates.
    templates: HashMap<String, PolicyTemplate>,
}

impl TemplateDiscovery {
    /// Creates a new template discovery instance.
    ///
    /// # Arguments
    /// * `templates_dir` - Directory containing policy templates
    ///
    /// # Returns
    /// New TemplateDiscovery instance.
    pub fn new(templates_dir: impl Into<PathBuf>) -> Self {
        Self {
            templates_dir: templates_dir.into(),
            templates: HashMap::new(),
        }
    }

    /// Discovers all policy templates in the templates directory.
    ///
    /// Scans `templates/policies/` for `.toml` files and creates template
    /// entries for each.
    ///
    /// # Returns
    /// Vector of discovered templates.
    pub fn discover(&mut self) -> std::io::Result<Vec<PolicyTemplate>> {
        let policies_dir = self.templates_dir.join("policies");
        
        if !policies_dir.exists() {
            return Ok(vec![]);
        }

        let mut templates = Vec::new();

        for entry in fs::read_dir(&policies_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("toml") {
                let name = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                
                // Try to extract description from template content
                let description = if let Ok(content) = fs::read_to_string(&path) {
                    // Look for description in comments at top of file
                    content.lines()
                        .find(|line| line.trim_start().starts_with("#") && 
                              (line.contains("Policy") || line.contains("Configuration")))
                        .map(|line| line.trim_start_matches("#").trim().to_string())
                        .unwrap_or_else(|| format!("{} policy template", name))
                } else {
                    format!("{} policy template", name)
                };

                let template = PolicyTemplate::new(name.clone(), description, path.clone());
                templates.push(template.clone());
                self.templates.insert(name, template);
            }
        }

        Ok(templates)
    }

    /// Gets a specific template by name.
    ///
    /// # Arguments
    /// * `name` - Template name (without .toml extension)
    ///
    /// # Returns
    /// Template if found, or error if not found.
    pub fn get_template(&self, name: &str) -> Option<&PolicyTemplate> {
        self.templates.get(name)
    }

    /// Lists all discovered templates.
    ///
    /// # Returns
    /// Vector of template names and descriptions.
    pub fn list_templates(&self) -> Vec<(&String, &String)> {
        self.templates.iter().map(|(name, template)| (name, &template.description)).collect()
    }

    /// Validates all discovered templates.
    ///
    /// # Returns
    /// Vector of (template_name, validation_result) tuples.
    pub fn validate_all(&mut self) -> Vec<(String, PolicyResult<()>)> {
        self.templates
            .values_mut()
            .map(|template| {
                let name = template.name.clone();
                let result = template.validate();
                (name, result)
            })
            .collect()
    }
}

/// Merges template rules with existing policy rules.
///
/// # Arguments
/// * `existing_policy_path` - Path to existing policy.toml
/// * `template_content` - Template content to merge
/// * `replace` - If true, replace all rules; if false, append rules
///
/// # Returns
/// Merged policy content.
pub fn merge_template(
    existing_policy_path: &Path,
    template_content: &str,
    replace: bool,
) -> std::io::Result<String> {
    use toml::Value;

    // Parse template
    let template_config: Value = toml::from_str(template_content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    // Parse existing policy if it exists and we're not replacing
    let mut existing_config: Value = if existing_policy_path.exists() && !replace {
        let existing_content = fs::read_to_string(existing_policy_path)?;
        toml::from_str(&existing_content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?
    } else {
        // Create default structure
        let mut config = toml::map::Map::new();
        config.insert("approval_mode".to_string(), Value::String("ask".to_string()));
        config.insert("rules".to_string(), Value::Array(vec![]));
        Value::Table(config)
    };

    // Get rules from template
    let template_rules = template_config
        .get("rules")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    if replace {
        // Replace all rules with template rules
        existing_config.as_table_mut().unwrap().insert(
            "rules".to_string(),
            Value::Array(template_rules),
        );
    } else {
        // Merge: append template rules to existing rules
        let existing_table = existing_config.as_table_mut().unwrap();
        
        // Get or create rules array
        if !existing_table.contains_key("rules") {
            existing_table.insert("rules".to_string(), Value::Array(vec![]));
        }
        
        if let Some(Value::Array(existing_rules)) = existing_table.get_mut("rules") {
            existing_rules.extend(template_rules);
        }
    }

    // Convert back to TOML string
    toml::to_string_pretty(&existing_config)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

/// Helper to parse PolicyEngine from string (for template validation).
impl PolicyEngine {
    /// Creates a PolicyEngine from a string (for validation).
    ///
    /// This is a helper method for template validation.
    pub fn from_str(content: &str) -> PolicyResult<Self> {
        use tempfile::TempDir;
        use std::fs;

        // Write to temp file and use existing from_file method
        let temp_dir = TempDir::new().map_err(|e| {
            PolicyError::InvalidConfig(format!("Failed to create temp dir: {}", e))
        })?;
        let temp_file = temp_dir.path().join("policy.toml");
        fs::write(&temp_file, content).map_err(|e| {
            PolicyError::InvalidConfiguration(format!("Failed to write temp file: {}", e))
        })?;

        Self::from_file(&temp_file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_template_discovery() {
        let temp_dir = TempDir::new().unwrap();
        let templates_dir = temp_dir.path().join("templates");
        fs::create_dir_all(&templates_dir.join("policies")).unwrap();

        // Create a test template
        let template_file = templates_dir.join("policies").join("test.toml");
        fs::write(&template_file, r#"# Test Policy
approval_mode = "ask"
[[rules]]
name = "test"
priority = "user"
action = "allow"
tool_pattern = "read_*"
"#).unwrap();

        let mut discovery = TemplateDiscovery::new(&templates_dir);
        let templates = discovery.discover().unwrap();
        
        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].name, "test");
    }

    #[test]
    fn test_template_validation() {
        let temp_dir = TempDir::new().unwrap();
        let template_file = temp_dir.path().join("test.toml");
        fs::write(&template_file, r#"approval_mode = "ask"
[[rules]]
name = "test"
priority = "user"
action = "allow"
tool_pattern = "read_*"
"#).unwrap();

        let mut template = PolicyTemplate::new(
            "test".to_string(),
            "Test template".to_string(),
            template_file,
        );

        assert!(template.validate().is_ok());
    }

    #[test]
    fn test_merge_template() {
        let temp_dir = TempDir::new().unwrap();
        let existing_file = temp_dir.path().join("existing.toml");
        
        // Create existing policy
        fs::write(&existing_file, r#"approval_mode = "ask"
[[rules]]
name = "existing"
priority = "user"
action = "allow"
tool_pattern = "read_*"
"#).unwrap();

        // Merge with template
        let template_content = r#"approval_mode = "ask"
[[rules]]
name = "template"
priority = "user"
action = "deny"
tool_pattern = "write_*"
"#;

        let merged = merge_template(&existing_file, template_content, false).unwrap();
        assert!(merged.contains("existing"));
        assert!(merged.contains("template"));
    }
}

