//! Core types for playbooks.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A playbook containing organizational knowledge, SOPs, or procedures.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Playbook {
    /// Unique URI identifier for this playbook (e.g., `radium://my-org/code-review-standards.md`).
    pub uri: String,
    /// Human-readable description of what this playbook covers.
    pub description: String,
    /// Tags for categorization and filtering.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Priority level determining when this playbook is included.
    pub priority: PlaybookPriority,
    /// Scopes where this playbook applies (e.g., `["requirement", "task", "pr-review"]`).
    #[serde(default)]
    pub applies_to: Vec<String>,
    /// Markdown content of the playbook.
    pub content: String,
}

impl Playbook {
    /// Creates a new playbook.
    pub fn new(
        uri: impl Into<String>,
        description: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            uri: uri.into(),
            description: description.into(),
            tags: Vec::new(),
            priority: PlaybookPriority::Recommended,
            applies_to: Vec::new(),
            content: content.into(),
        }
    }

    /// Validates the playbook structure.
    ///
    /// # Errors
    ///
    /// Returns error if URI is invalid or required fields are missing.
    pub fn validate(&self) -> crate::playbooks::error::Result<()> {
        use crate::playbooks::error::PlaybookError;

        // Validate URI format
        if !self.uri.starts_with("radium://") {
            return Err(PlaybookError::InvalidUri(self.uri.clone()));
        }

        // Validate required fields
        if self.description.is_empty() {
            return Err(PlaybookError::MissingField("description".to_string()));
        }

        if self.content.is_empty() {
            return Err(PlaybookError::MissingField("content".to_string()));
        }

        Ok(())
    }

    /// Checks if this playbook applies to the given scope.
    pub fn applies_to_scope(&self, scope: &str) -> bool {
        self.applies_to.is_empty() || self.applies_to.iter().any(|s| s == scope)
    }

    /// Checks if this playbook has any of the given tags.
    pub fn has_tags(&self, tags: &[String]) -> bool {
        if tags.is_empty() {
            return true;
        }
        tags.iter().any(|tag| self.tags.contains(tag))
    }
}

/// Priority level for playbooks.
///
/// Determines when and how playbooks are included in agent context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlaybookPriority {
    /// Optional playbooks - only included if explicitly requested.
    Optional = 0,
    /// Recommended playbooks - included by default when scope matches.
    Recommended = 1,
    /// Required playbooks - always included when scope matches.
    Required = 2,
}

impl fmt::Display for PlaybookPriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlaybookPriority::Optional => write!(f, "optional"),
            PlaybookPriority::Recommended => write!(f, "recommended"),
            PlaybookPriority::Required => write!(f, "required"),
        }
    }
}

impl Default for PlaybookPriority {
    fn default() -> Self {
        Self::Recommended
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playbook_validation_valid() {
        let playbook = Playbook::new(
            "radium://org/test.md",
            "Test playbook",
            "# Content",
        );
        assert!(playbook.validate().is_ok());
    }

    #[test]
    fn test_playbook_validation_invalid_uri() {
        let mut playbook = Playbook::new(
            "http://invalid",
            "Test",
            "Content",
        );
        assert!(playbook.validate().is_err());
    }

    #[test]
    fn test_playbook_applies_to_scope() {
        let mut playbook = Playbook::new(
            "radium://org/test.md",
            "Test",
            "Content",
        );
        playbook.applies_to = vec!["requirement".to_string(), "task".to_string()];

        assert!(playbook.applies_to_scope("requirement"));
        assert!(playbook.applies_to_scope("task"));
        assert!(!playbook.applies_to_scope("pr-review"));
    }

    #[test]
    fn test_playbook_applies_to_scope_empty() {
        let playbook = Playbook::new(
            "radium://org/test.md",
            "Test",
            "Content",
        );
        // Empty applies_to means applies to all scopes
        assert!(playbook.applies_to_scope("any-scope"));
    }

    #[test]
    fn test_playbook_has_tags() {
        let mut playbook = Playbook::new(
            "radium://org/test.md",
            "Test",
            "Content",
        );
        playbook.tags = vec!["code-review".to_string(), "quality".to_string()];

        assert!(playbook.has_tags(&["code-review".to_string()]));
        assert!(playbook.has_tags(&["quality".to_string()]));
        assert!(!playbook.has_tags(&["security".to_string()]));
        assert!(playbook.has_tags(&[])); // Empty tags means match all
    }

    #[test]
    fn test_playbook_priority_ordering() {
        assert!(PlaybookPriority::Required > PlaybookPriority::Recommended);
        assert!(PlaybookPriority::Recommended > PlaybookPriority::Optional);
    }
}

