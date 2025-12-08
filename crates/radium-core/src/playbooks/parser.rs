//! Parser for playbook YAML frontmatter and markdown content.

use crate::playbooks::error::{PlaybookError, Result};
use crate::playbooks::types::{Playbook, PlaybookPriority};
use serde_yaml;

/// Parser for playbook files with YAML frontmatter.
pub struct PlaybookParser;

impl PlaybookParser {
    /// Parse a playbook from markdown content with YAML frontmatter.
    ///
    /// Expected format:
    /// ```markdown
    /// ---
    /// uri: radium://my-org/code-review-standards.md
    /// description: Code review checklist for all PRs
    /// tags: [code-review, quality, standards]
    /// priority: required
    /// applies_to: [requirement, task, pr-review]
    /// ---
    ///
    /// # Markdown content here
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if frontmatter is invalid, missing required fields, or URI is invalid.
    pub fn parse(content: &str) -> Result<Playbook> {
        // Split frontmatter and content
        let (frontmatter, markdown_content) = Self::split_frontmatter(content)?;

        // Parse YAML frontmatter
        let mut playbook: Playbook = serde_yaml::from_str(&frontmatter)
            .map_err(|e| PlaybookError::ParseError {
                path: None,
                source: e,
            })?;

        // Set the markdown content
        playbook.content = markdown_content;

        // Validate the playbook
        playbook.validate()?;

        Ok(playbook)
    }

    /// Parse a playbook from a file.
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be read or parsed.
    pub fn parse_file(path: impl AsRef<std::path::Path>) -> Result<Playbook> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path).map_err(|e| PlaybookError::LoadError {
            path: path.to_path_buf(),
            source: e,
        })?;

        let mut playbook = Self::parse(&content)?;
        // Note: We don't store the path in Playbook, but we could add it if needed
        Ok(playbook)
    }

    /// Split YAML frontmatter from markdown content.
    ///
    /// This follows the same pattern as `AgentMetadata::split_frontmatter`.
    fn split_frontmatter(content: &str) -> Result<(String, String)> {
        let trimmed = content.trim_start();

        // Check if content starts with frontmatter delimiter
        if !trimmed.starts_with("---") {
            return Err(PlaybookError::InvalidFrontmatter(
                "content does not start with '---'".to_string(),
            ));
        }

        // Find the closing delimiter
        let after_first = &trimmed[3..];
        let end_idx = after_first.find("\n---").ok_or_else(|| {
            PlaybookError::InvalidFrontmatter("no closing '---' delimiter found".to_string())
        })?;

        let frontmatter = &after_first[..end_idx];
        let content = &after_first[end_idx + 4..]; // Skip "\n---"

        Ok((frontmatter.to_string(), content.trim().to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::playbooks::types::PlaybookPriority;

    #[test]
    fn test_parse_valid_playbook() {
        let content = r#"---
uri: radium://my-org/code-review-standards.md
description: Code review checklist for all PRs
tags: [code-review, quality, standards]
priority: required
applies_to: [requirement, task, pr-review]
---
# Code Review Standards

This playbook defines our code review process.
"#;

        let playbook = PlaybookParser::parse(content).unwrap();
        assert_eq!(playbook.uri, "radium://my-org/code-review-standards.md");
        assert_eq!(playbook.description, "Code review checklist for all PRs");
        assert_eq!(playbook.tags, vec!["code-review", "quality", "standards"]);
        assert_eq!(playbook.priority, PlaybookPriority::Required);
        assert_eq!(
            playbook.applies_to,
            vec!["requirement", "task", "pr-review"]
        );
        assert!(playbook.content.contains("Code Review Standards"));
    }

    #[test]
    fn test_parse_missing_frontmatter() {
        let content = "# Just markdown, no frontmatter";
        let result = PlaybookParser::parse(content);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PlaybookError::InvalidFrontmatter(_)
        ));
    }

    #[test]
    fn test_parse_missing_required_fields() {
        let content = r#"---
description: Missing URI
---
# Content
"#;

        let result = PlaybookParser::parse(content);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PlaybookError::InvalidUri(_) | PlaybookError::MissingField(_)
        ));
    }

    #[test]
    fn test_parse_invalid_uri() {
        let content = r#"---
uri: http://invalid-uri
description: Test
---
# Content
"#;

        let result = PlaybookParser::parse(content);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PlaybookError::InvalidUri(_)));
    }

    #[test]
    fn test_parse_default_priority() {
        let content = r#"---
uri: radium://org/test.md
description: Test playbook
---
# Content
"#;

        let playbook = PlaybookParser::parse(content).unwrap();
        // Default priority should be Recommended
        assert_eq!(playbook.priority, PlaybookPriority::Recommended);
    }

    #[test]
    fn test_parse_empty_tags() {
        let content = r#"---
uri: radium://org/test.md
description: Test playbook
tags: []
---
# Content
"#;

        let playbook = PlaybookParser::parse(content).unwrap();
        assert!(playbook.tags.is_empty());
    }
}

