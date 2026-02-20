//! Skill-based routing using the Open Agent Skills standard.
//!
//! This module implements routing to agents based on the Ai-Agent-Skills specification,
//! allowing Radium to leverage the ecosystem of standardized skills and potentially
//! contribute Radium-specific skills to the marketplace.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::{Context, Result};

use super::capability_matcher::CapabilityMatcher;

/// Skill definition following the Agent Skills specification.
///
/// Spec: https://agentskills.io/specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDefinition {
    /// Skill identifier (lowercase-hyphen format, 1-64 chars).
    /// Must match parent directory name.
    pub name: String,

    /// Human-readable description with keywords for matching (1-1024 chars).
    /// This is the primary field used for semantic routing.
    pub description: String,

    /// License name or bundled file reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,

    /// Environment requirements (1-500 chars).
    /// Example: "Requires git, jq, and network access"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compatibility: Option<String>,

    /// Additional metadata as key-value pairs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,

    /// Experimental: Space-delimited list of pre-approved tools.
    /// Example: "Bash(git:*) Bash(jq:*) Read"
    #[serde(skip_serializing_if = "Option::is_none", rename = "allowed-tools")]
    pub allowed_tools: Option<String>,

    /// Full path to the SKILL.md file.
    #[serde(skip)]
    pub skill_path: PathBuf,

    /// The markdown body content (instructions).
    /// Loaded on-demand for progressive disclosure.
    #[serde(skip)]
    pub instructions: Option<String>,
}

impl SkillDefinition {
    /// Parse a SKILL.md file and extract the skill definition.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read skill file: {}", path.display()))?;

        Self::from_markdown_with_path(&content, path.to_path_buf())
    }

    /// Parse SKILL.md content with frontmatter.
    fn from_markdown_with_path(content: &str, skill_path: PathBuf) -> Result<Self> {
        // Parse frontmatter (YAML between --- delimiters)
        let (frontmatter, body) = Self::parse_frontmatter(content)?;

        let mut skill: SkillDefinition = serde_yaml::from_str(&frontmatter)
            .context("Failed to parse skill frontmatter")?;

        skill.skill_path = skill_path;
        skill.instructions = Some(body.trim().to_string());

        // Validate required fields
        Self::validate(&skill)?;

        Ok(skill)
    }

    /// Parse YAML frontmatter from markdown content.
    fn parse_frontmatter(content: &str) -> Result<(String, String)> {
        let lines: Vec<&str> = content.lines().collect();

        // Find frontmatter delimiters (---)
        let mut start_idx = None;
        let mut end_idx = None;

        for (i, line) in lines.iter().enumerate() {
            if line.trim() == "---" {
                if start_idx.is_none() {
                    start_idx = Some(i);
                } else {
                    end_idx = Some(i);
                    break;
                }
            }
        }

        match (start_idx, end_idx) {
            (Some(start), Some(end)) if end > start => {
                let frontmatter = lines[start + 1..end].join("\n");
                let body = lines[end + 1..].join("\n");
                Ok((frontmatter, body))
            }
            _ => anyhow::bail!("Invalid frontmatter format: expected --- delimiters"),
        }
    }

    /// Validate skill definition according to spec.
    fn validate(skill: &SkillDefinition) -> Result<()> {
        // Name validation: 1-64 chars, lowercase-hyphen only
        if skill.name.is_empty() || skill.name.len() > 64 {
            anyhow::bail!("Skill name must be 1-64 characters");
        }

        let valid_name = skill.name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            && !skill.name.starts_with('-')
            && !skill.name.ends_with('-')
            && !skill.name.contains("--");

        if !valid_name {
            anyhow::bail!(
                "Skill name '{}' must be lowercase letters, numbers, and hyphens only",
                skill.name
            );
        }

        // Description validation: 1-1024 chars, non-empty
        if skill.description.is_empty() || skill.description.len() > 1024 {
            anyhow::bail!("Skill description must be 1-1024 characters");
        }

        // Compatibility validation: max 500 chars
        if let Some(ref compat) = skill.compatibility {
            if compat.len() > 500 {
                anyhow::bail!("Compatibility field must be <= 500 characters");
            }
        }

        Ok(())
    }

    /// Get the skill's category from metadata, if any.
    pub fn category(&self) -> Option<&str> {
        self.metadata.as_ref().and_then(|m| m.get("category").map(std::string::String::as_str))
    }

    /// Get the skill's author from metadata, if any.
    pub fn author(&self) -> Option<&str> {
        self.metadata.as_ref().and_then(|m| m.get("author").map(std::string::String::as_str))
    }
}

/// Routing result with confidence score.
#[derive(Debug, Clone)]
pub struct SkillRoutingResult {
    pub skill_name: String,
    pub confidence: f32,
    pub reasoning: String,
}

/// Skill-based router for agent selection.
pub struct SkillRouter {
    /// Registry of available skills indexed by name.
    registry: Arc<RwLock<HashMap<String, SkillDefinition>>>,

    /// Capability matcher for semantic routing.
    matcher: CapabilityMatcher,

    /// Minimum confidence threshold for routing (default: 0.5).
    confidence_threshold: f32,
}

impl SkillRouter {
    /// Create a new skill router with default configuration.
    pub fn new() -> Self {
        Self {
            registry: Arc::new(RwLock::new(HashMap::new())),
            matcher: CapabilityMatcher::new(),
            confidence_threshold: 0.5,
        }
    }

    /// Create a skill router with custom confidence threshold.
    pub fn with_threshold(threshold: f32) -> Self {
        Self {
            registry: Arc::new(RwLock::new(HashMap::new())),
            matcher: CapabilityMatcher::new(),
            confidence_threshold: threshold,
        }
    }

    /// Load skills from a directory containing SKILL.md files.
    ///
    /// Searches recursively for all SKILL.md files and registers them.
    pub async fn load_skills_from_directory(&self, dir: impl AsRef<Path>) -> Result<usize> {
        let dir = dir.as_ref();
        let mut count = 0;

        for entry in walkdir::WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            if entry.file_name() == "SKILL.md" {
                match SkillDefinition::from_file(entry.path()) {
                    Ok(skill) => {
                        self.register_skill(skill).await?;
                        count += 1;
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to load skill from {}: {}", entry.path().display(), e);
                    }
                }
            }
        }

        Ok(count)
    }

    /// Register a skill in the router.
    pub async fn register_skill(&self, skill: SkillDefinition) -> Result<()> {
        let mut registry = self.registry.write().await;
        registry.insert(skill.name.clone(), skill);
        Ok(())
    }

    /// Unregister a skill by name.
    pub async fn unregister_skill(&self, name: &str) -> Result<()> {
        let mut registry = self.registry.write().await;
        registry
            .remove(name)
            .ok_or_else(|| anyhow::anyhow!("Skill '{}' not found", name))?;
        Ok(())
    }

    /// Get a skill by name.
    pub async fn get_skill(&self, name: &str) -> Option<SkillDefinition> {
        let registry = self.registry.read().await;
        registry.get(name).cloned()
    }

    /// List all registered skills.
    pub async fn list_skills(&self) -> Vec<String> {
        let registry = self.registry.read().await;
        registry.keys().cloned().collect()
    }

    /// Route a task to the best matching skill.
    ///
    /// Returns the skill name and confidence score, or None if no skill
    /// meets the confidence threshold.
    pub async fn route(&self, task_description: &str) -> Result<Option<SkillRoutingResult>> {
        let registry = self.registry.read().await;

        if registry.is_empty() {
            return Ok(None);
        }

        // Collect all skills for matching
        let skills: Vec<&SkillDefinition> = registry.values().collect();

        // Use capability matcher to find best skill
        let best_match = self.matcher.find_best_match(task_description, &skills).await;

        match best_match {
            Some((skill, confidence, reasoning)) if confidence >= self.confidence_threshold => {
                Ok(Some(SkillRoutingResult {
                    skill_name: skill.name.clone(),
                    confidence,
                    reasoning,
                }))
            }
            _ => Ok(None),
        }
    }

    /// Route to multiple skills (top-k results).
    ///
    /// Returns up to `k` skills ranked by confidence, all above threshold.
    pub async fn route_top_k(&self, task_description: &str, k: usize) -> Result<Vec<SkillRoutingResult>> {
        let registry = self.registry.read().await;

        if registry.is_empty() {
            return Ok(Vec::new());
        }

        let skills: Vec<&SkillDefinition> = registry.values().collect();

        let mut results = self.matcher.rank_skills(task_description, &skills).await;

        // Filter by threshold and take top k
        results.retain(|(_, confidence, _)| *confidence >= self.confidence_threshold);
        results.truncate(k);

        Ok(results
            .into_iter()
            .map(|(skill, confidence, reasoning)| SkillRoutingResult {
                skill_name: skill.name.clone(),
                confidence,
                reasoning,
            })
            .collect())
    }

    /// Get the number of registered skills.
    pub async fn skill_count(&self) -> usize {
        let registry = self.registry.read().await;
        registry.len()
    }
}

impl Default for SkillRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
name: test-skill
description: A test skill for unit testing
license: MIT
---

# Instructions
This is the body content.
"#;

        let (frontmatter, body) = SkillDefinition::parse_frontmatter(content).unwrap();
        assert!(frontmatter.contains("name: test-skill"));
        assert!(body.contains("# Instructions"));
    }

    #[test]
    fn test_skill_name_validation() {
        // Valid names
        assert!(SkillDefinition::validate(&SkillDefinition {
            name: "test-skill".to_string(),
            description: "Valid skill".to_string(),
            license: None,
            compatibility: None,
            metadata: None,
            allowed_tools: None,
            skill_path: PathBuf::from("/test"),
            instructions: None,
        })
        .is_ok());

        // Invalid: uppercase
        assert!(SkillDefinition::validate(&SkillDefinition {
            name: "Test-Skill".to_string(),
            description: "Invalid skill".to_string(),
            license: None,
            compatibility: None,
            metadata: None,
            allowed_tools: None,
            skill_path: PathBuf::from("/test"),
            instructions: None,
        })
        .is_err());

        // Invalid: starts with hyphen
        assert!(SkillDefinition::validate(&SkillDefinition {
            name: "-test-skill".to_string(),
            description: "Invalid skill".to_string(),
            license: None,
            compatibility: None,
            metadata: None,
            allowed_tools: None,
            skill_path: PathBuf::from("/test"),
            instructions: None,
        })
        .is_err());
    }

    #[tokio::test]
    async fn test_skill_registration() {
        let router = SkillRouter::new();

        let skill = SkillDefinition {
            name: "test-skill".to_string(),
            description: "A test skill".to_string(),
            license: Some("MIT".to_string()),
            compatibility: None,
            metadata: None,
            allowed_tools: None,
            skill_path: PathBuf::from("/test/SKILL.md"),
            instructions: Some("Do something useful".to_string()),
        };

        router.register_skill(skill).await.unwrap();
        assert_eq!(router.skill_count().await, 1);

        let retrieved = router.get_skill("test-skill").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().description, "A test skill");
    }
}
