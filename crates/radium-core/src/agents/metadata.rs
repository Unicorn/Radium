//! Enhanced agent metadata with YAML frontmatter support.
//!
//! Provides rich agent metadata including model recommendations,
//! capabilities, and performance profiles.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Agent metadata errors.
#[derive(Debug, Error)]
pub enum MetadataError {
    /// YAML parsing error.
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// Invalid frontmatter.
    #[error("invalid frontmatter: {0}")]
    InvalidFrontmatter(String),

    /// Missing required field.
    #[error("missing required field: {0}")]
    MissingField(String),

    /// Invalid value.
    #[error("invalid value for {field}: {reason}")]
    InvalidValue { field: String, reason: String },

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for metadata operations.
pub type Result<T> = std::result::Result<T, MetadataError>;

/// Model priority levels for selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelPriority {
    /// Optimize for speed - fast models, lower cost.
    Speed,

    /// Balanced speed and quality.
    Balanced,

    /// Optimize for deep reasoning.
    Thinking,

    /// Expert-level reasoning, highest cost.
    Expert,
}

impl std::fmt::Display for ModelPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Speed => write!(f, "speed"),
            Self::Balanced => write!(f, "balanced"),
            Self::Thinking => write!(f, "thinking"),
            Self::Expert => write!(f, "expert"),
        }
    }
}

/// Cost tier for model selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CostTier {
    /// Low cost: $0.00 - $0.10 per 1M tokens.
    Low,

    /// Medium cost: $0.10 - $1.00 per 1M tokens.
    Medium,

    /// High cost: $1.00 - $10.00 per 1M tokens.
    High,

    /// Premium cost: $10.00+ per 1M tokens.
    Premium,
}

impl std::fmt::Display for CostTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Premium => write!(f, "premium"),
        }
    }
}

/// Model recommendation for an agent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelRecommendation {
    /// Engine to use (e.g., "gemini", "openai", "anthropic").
    pub engine: String,

    /// Model ID (e.g., "gemini-2.0-flash-exp", "gpt-4o").
    pub model: String,

    /// Reasoning for this recommendation.
    pub reasoning: String,

    /// Priority level for this model.
    pub priority: ModelPriority,

    /// Cost tier.
    pub cost_tier: CostTier,

    /// Optional: requires user approval before use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_approval: Option<bool>,
}

/// Recommended models for different use cases.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecommendedModels {
    /// Primary recommended model for most tasks.
    pub primary: ModelRecommendation,

    /// Fallback model when primary is unavailable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback: Option<ModelRecommendation>,

    /// Premium model for critical or complex tasks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub premium: Option<ModelRecommendation>,
}

/// Thinking depth level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThinkingDepth {
    /// Minimal thinking required.
    Low,

    /// Moderate thinking.
    Medium,

    /// Deep thinking required.
    High,

    /// Expert-level deep thinking.
    Expert,
}

impl std::fmt::Display for ThinkingDepth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Expert => write!(f, "expert"),
        }
    }
}

/// Iteration speed level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IterationSpeed {
    /// Slow iteration (complex processing).
    Slow,

    /// Medium iteration speed.
    Medium,

    /// Fast iteration.
    Fast,

    /// Instant/near-instant iteration.
    Instant,
}

impl std::fmt::Display for IterationSpeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Slow => write!(f, "slow"),
            Self::Medium => write!(f, "medium"),
            Self::Fast => write!(f, "fast"),
            Self::Instant => write!(f, "instant"),
        }
    }
}

/// Context requirements level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContextRequirements {
    /// Minimal context needed.
    Low,

    /// Moderate context.
    Medium,

    /// High context requirements.
    High,

    /// Extensive context needed.
    Extensive,
}

impl std::fmt::Display for ContextRequirements {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Extensive => write!(f, "extensive"),
        }
    }
}

/// Output volume level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputVolume {
    /// Minimal output.
    Low,

    /// Moderate output.
    Medium,

    /// High output volume.
    High,

    /// Extensive output.
    Extensive,
}

impl std::fmt::Display for OutputVolume {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Extensive => write!(f, "extensive"),
        }
    }
}

/// Agent performance profile.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerformanceProfile {
    /// Thinking depth required.
    pub thinking_depth: ThinkingDepth,

    /// Expected iteration speed.
    pub iteration_speed: IterationSpeed,

    /// Context requirements.
    pub context_requirements: ContextRequirements,

    /// Expected output volume.
    pub output_volume: OutputVolume,
}

/// Enhanced agent metadata with YAML frontmatter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    /// Agent identifier (kebab-case).
    pub name: String,

    /// Display name (optional, defaults to name).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Agent category (e.g., "engineering", "design").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Color for UI display.
    pub color: String,

    /// Short summary (one-line description).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    /// Detailed description (can be multiline).
    pub description: String,

    /// Model recommendations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended_models: Option<RecommendedModels>,

    /// Agent capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<Vec<String>>,

    /// Performance profile.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance_profile: Option<PerformanceProfile>,

    /// Quality gates required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality_gates: Option<Vec<String>>,

    /// Agents this works well with.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub works_well_with: Option<Vec<String>>,

    /// Typical workflow patterns.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typical_workflows: Option<Vec<String>>,

    /// Tool restrictions (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<String>>,

    /// Additional constraints.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<HashMap<String, serde_yaml::Value>>,
}

impl AgentMetadata {
    /// Parse agent metadata from markdown file with YAML frontmatter.
    ///
    /// Expected format:
    /// ```markdown
    /// ---
    /// name: agent-id
    /// color: blue
    /// description: Agent description
    /// ---
    ///
    /// # Agent Prompt Content
    /// ...
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if YAML frontmatter is invalid or missing required fields.
    pub fn from_markdown(content: &str) -> Result<(Self, String)> {
        // Split frontmatter and content
        let (frontmatter, prompt) = Self::split_frontmatter(content)?;

        // Parse YAML frontmatter
        let metadata: AgentMetadata = serde_yaml::from_str(&frontmatter)?;

        // Validate required fields
        metadata.validate()?;

        Ok((metadata, prompt))
    }

    /// Parse agent metadata from a file.
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be read or parsed.
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<(Self, String)> {
        let content = std::fs::read_to_string(path)?;
        Self::from_markdown(&content)
    }

    /// Split YAML frontmatter from markdown content.
    fn split_frontmatter(content: &str) -> Result<(String, String)> {
        let trimmed = content.trim_start();

        // Check if content starts with frontmatter delimiter
        if !trimmed.starts_with("---") {
            return Err(MetadataError::InvalidFrontmatter(
                "content does not start with '---'".to_string(),
            ));
        }

        // Find the closing delimiter
        let after_first = &trimmed[3..];
        let end_idx = after_first.find("\n---").ok_or_else(|| {
            MetadataError::InvalidFrontmatter("no closing '---' delimiter found".to_string())
        })?;

        let frontmatter = &after_first[..end_idx];
        let content = &after_first[end_idx + 4..]; // Skip "\n---"

        Ok((frontmatter.to_string(), content.trim().to_string()))
    }

    /// Validate metadata has required fields.
    fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(MetadataError::MissingField("name".to_string()));
        }

        if self.color.is_empty() {
            return Err(MetadataError::MissingField("color".to_string()));
        }

        if self.description.is_empty() {
            return Err(MetadataError::MissingField("description".to_string()));
        }

        // Validate model recommendations if present
        if let Some(ref models) = self.recommended_models {
            Self::validate_model_recommendation(&models.primary, "primary")?;

            if let Some(ref fallback) = models.fallback {
                Self::validate_model_recommendation(fallback, "fallback")?;
            }

            if let Some(ref premium) = models.premium {
                Self::validate_model_recommendation(premium, "premium")?;
            }
        }

        Ok(())
    }

    /// Validate a single model recommendation.
    fn validate_model_recommendation(rec: &ModelRecommendation, field_name: &str) -> Result<()> {
        if rec.engine.is_empty() {
            return Err(MetadataError::InvalidValue {
                field: format!("{}.engine", field_name),
                reason: "cannot be empty".to_string(),
            });
        }

        if rec.model.is_empty() {
            return Err(MetadataError::InvalidValue {
                field: format!("{}.model", field_name),
                reason: "cannot be empty".to_string(),
            });
        }

        if rec.reasoning.is_empty() {
            return Err(MetadataError::InvalidValue {
                field: format!("{}.reasoning", field_name),
                reason: "cannot be empty".to_string(),
            });
        }

        Ok(())
    }

    /// Get the display name or fall back to name.
    pub fn get_display_name(&self) -> &str {
        self.display_name.as_deref().unwrap_or(&self.name)
    }

    /// Get the summary or fall back to description (truncated).
    pub fn get_summary(&self) -> String {
        if let Some(ref summary) = self.summary {
            summary.clone()
        } else {
            // Truncate description to first line or 120 chars
            let first_line = self.description.lines().next().unwrap_or(&self.description);
            if first_line.len() > 120 {
                format!("{}...", &first_line[..117])
            } else {
                first_line.to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_priority_display() {
        assert_eq!(ModelPriority::Speed.to_string(), "speed");
        assert_eq!(ModelPriority::Balanced.to_string(), "balanced");
        assert_eq!(ModelPriority::Thinking.to_string(), "thinking");
        assert_eq!(ModelPriority::Expert.to_string(), "expert");
    }

    #[test]
    fn test_cost_tier_display() {
        assert_eq!(CostTier::Low.to_string(), "low");
        assert_eq!(CostTier::Medium.to_string(), "medium");
        assert_eq!(CostTier::High.to_string(), "high");
        assert_eq!(CostTier::Premium.to_string(), "premium");
    }

    #[test]
    fn test_split_frontmatter_basic() {
        let content = r"---
name: test-agent
color: blue
description: Test agent
---

# Test Content
This is the prompt.";

        let (frontmatter, prompt) = AgentMetadata::split_frontmatter(content).unwrap();

        assert!(frontmatter.contains("name: test-agent"));
        assert!(prompt.contains("# Test Content"));
    }

    #[test]
    fn test_split_frontmatter_no_delimiter() {
        let content = "No frontmatter here";
        let result = AgentMetadata::split_frontmatter(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_split_frontmatter_no_closing() {
        let content = "---\nname: test\n";
        let result = AgentMetadata::split_frontmatter(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_minimal_metadata() {
        let content = r"---
name: test-agent
color: blue
description: Test agent description
---

# Test Prompt";

        let (metadata, prompt) = AgentMetadata::from_markdown(content).unwrap();

        assert_eq!(metadata.name, "test-agent");
        assert_eq!(metadata.color, "blue");
        assert_eq!(metadata.description, "Test agent description");
        assert!(prompt.contains("# Test Prompt"));
    }

    #[test]
    fn test_parse_full_metadata() {
        let content = r"---
name: architect-ux
display_name: ArchitectUX
category: design
color: purple
summary: Technical architecture specialist
description: |
  Comprehensive UX architect who bridges the gap between specs
  and implementation.
recommended_models:
  primary:
    engine: gemini
    model: gemini-2.0-flash-exp
    reasoning: Fast iteration for CSS generation
    priority: speed
    cost_tier: low
  fallback:
    engine: openai
    model: gpt-4o-mini
    reasoning: Balanced cost and quality
    priority: balanced
    cost_tier: low
capabilities:
  - css_architecture
  - responsive_design
performance_profile:
  thinking_depth: medium
  iteration_speed: fast
  context_requirements: medium
  output_volume: high
---

# ArchitectUX Prompt";

        let (metadata, _) = AgentMetadata::from_markdown(content).unwrap();

        assert_eq!(metadata.name, "architect-ux");
        assert_eq!(metadata.get_display_name(), "ArchitectUX");
        assert_eq!(metadata.category, Some("design".to_string()));
        assert_eq!(metadata.color, "purple");

        // Check model recommendations
        let models = metadata.recommended_models.unwrap();
        assert_eq!(models.primary.engine, "gemini");
        assert_eq!(models.primary.model, "gemini-2.0-flash-exp");
        assert_eq!(models.primary.priority, ModelPriority::Speed);
        assert_eq!(models.primary.cost_tier, CostTier::Low);

        // Check capabilities
        let caps = metadata.capabilities.unwrap();
        assert!(caps.contains(&"css_architecture".to_string()));
        assert!(caps.contains(&"responsive_design".to_string()));

        // Check performance profile
        let perf = metadata.performance_profile.unwrap();
        assert_eq!(perf.thinking_depth, ThinkingDepth::Medium);
        assert_eq!(perf.iteration_speed, IterationSpeed::Fast);
    }

    #[test]
    fn test_validate_missing_name() {
        let content = r"---
color: blue
description: Test
---";

        let result = AgentMetadata::from_markdown(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_missing_color() {
        let content = r"---
name: test
description: Test
---";

        let result = AgentMetadata::from_markdown(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_summary_from_summary_field() {
        let content = r"---
name: test
color: blue
summary: Short summary
description: Long description
---";

        let (metadata, _) = AgentMetadata::from_markdown(content).unwrap();
        assert_eq!(metadata.get_summary(), "Short summary");
    }

    #[test]
    fn test_get_summary_from_description() {
        let content = r"---
name: test
color: blue
description: First line of description
---";

        let (metadata, _) = AgentMetadata::from_markdown(content).unwrap();
        assert_eq!(metadata.get_summary(), "First line of description");
    }
}
