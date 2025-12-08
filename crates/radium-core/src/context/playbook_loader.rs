//! Playbook loader for context injection.

use crate::playbooks::registry::PlaybookRegistry;
use crate::playbooks::types::{Playbook, PlaybookPriority};
use std::sync::Arc;

/// Loader for playbooks in context injection.
pub struct PlaybookLoader {
    /// Reference to the playbook registry.
    registry: Arc<PlaybookRegistry>,
}

impl PlaybookLoader {
    /// Create a new playbook loader.
    pub fn new(registry: Arc<PlaybookRegistry>) -> Self {
        Self { registry }
    }

    /// Load playbooks for a given scope and optional tags.
    ///
    /// # Arguments
    /// * `scope` - The execution scope (e.g., "requirement", "task", "pr-review")
    /// * `tags` - Optional tags to filter playbooks
    ///
    /// # Errors
    ///
    /// Returns error if registry access fails.
    pub fn load_for_scope(
        &self,
        scope: &str,
        tags: Option<&[String]>,
    ) -> crate::playbooks::error::Result<Vec<Playbook>> {
        // Filter by scope first
        let mut playbooks = self.registry.filter_by_scope(scope)?;

        // Filter by tags if provided
        if let Some(tag_list) = tags {
            playbooks.retain(|playbook| playbook.has_tags(tag_list));
        }

        // Sort by priority (Required → Recommended → Optional)
        playbooks.sort_by(|a, b| b.priority.cmp(&a.priority));

        Ok(playbooks)
    }

    /// Format playbooks as markdown sections for inclusion in context.
    ///
    /// Playbooks are formatted with priority labels and organized by priority.
    pub fn format_playbooks(playbooks: &[Playbook]) -> String {
        if playbooks.is_empty() {
            return String::new();
        }

        let mut output = String::from("# Organizational Playbooks\n\n");

        // Group by priority
        let mut required = Vec::new();
        let mut recommended = Vec::new();
        let mut optional = Vec::new();

        for playbook in playbooks {
            match playbook.priority {
                PlaybookPriority::Required => required.push(playbook),
                PlaybookPriority::Recommended => recommended.push(playbook),
                PlaybookPriority::Optional => optional.push(playbook),
            }
        }

        // Format required playbooks
        if !required.is_empty() {
            output.push_str("## Priority: Required\n\n");
            for playbook in required {
                output.push_str(&format!("### {}\n\n", playbook.description));
                output.push_str(&format!("**URI**: `{}`\n\n", playbook.uri));
                if !playbook.tags.is_empty() {
                    output.push_str(&format!(
                        "**Tags**: {}\n\n",
                        playbook.tags.join(", ")
                    ));
                }
                output.push_str(&playbook.content);
                output.push_str("\n\n---\n\n");
            }
        }

        // Format recommended playbooks
        if !recommended.is_empty() {
            output.push_str("## Priority: Recommended\n\n");
            for playbook in recommended {
                output.push_str(&format!("### {}\n\n", playbook.description));
                output.push_str(&format!("**URI**: `{}`\n\n", playbook.uri));
                if !playbook.tags.is_empty() {
                    output.push_str(&format!(
                        "**Tags**: {}\n\n",
                        playbook.tags.join(", ")
                    ));
                }
                output.push_str(&playbook.content);
                output.push_str("\n\n---\n\n");
            }
        }

        // Format optional playbooks
        if !optional.is_empty() {
            output.push_str("## Priority: Optional\n\n");
            for playbook in optional {
                output.push_str(&format!("### {}\n\n", playbook.description));
                output.push_str(&format!("**URI**: `{}`\n\n", playbook.uri));
                if !playbook.tags.is_empty() {
                    output.push_str(&format!(
                        "**Tags**: {}\n\n",
                        playbook.tags.join(", ")
                    ));
                }
                output.push_str(&playbook.content);
                output.push_str("\n\n---\n\n");
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::playbooks::types::PlaybookPriority;

    #[test]
    fn test_load_for_scope() {
        let registry = Arc::new(PlaybookRegistry::new());

        let playbook1 = Playbook {
            uri: "radium://org/test1.md".to_string(),
            description: "Test 1".to_string(),
            tags: vec!["code-review".to_string()],
            priority: PlaybookPriority::Required,
            applies_to: vec!["requirement".to_string()],
            content: "# Test 1".to_string(),
        };

        let playbook2 = Playbook {
            uri: "radium://org/test2.md".to_string(),
            description: "Test 2".to_string(),
            tags: vec!["security".to_string()],
            priority: PlaybookPriority::Recommended,
            applies_to: vec!["task".to_string()],
            content: "# Test 2".to_string(),
        };

        registry.register(playbook1).unwrap();
        registry.register(playbook2).unwrap();

        let loader = PlaybookLoader::new(registry);
        let playbooks = loader.load_for_scope("requirement", None).unwrap();

        assert_eq!(playbooks.len(), 1);
        assert_eq!(playbooks[0].uri, "radium://org/test1.md");
    }

    #[test]
    fn test_load_for_scope_with_tags() {
        let registry = Arc::new(PlaybookRegistry::new());

        let playbook1 = Playbook {
            uri: "radium://org/test1.md".to_string(),
            description: "Test 1".to_string(),
            tags: vec!["code-review".to_string()],
            priority: PlaybookPriority::Required,
            applies_to: vec!["requirement".to_string()],
            content: "# Test 1".to_string(),
        };

        let playbook2 = Playbook {
            uri: "radium://org/test2.md".to_string(),
            description: "Test 2".to_string(),
            tags: vec!["security".to_string()],
            priority: PlaybookPriority::Recommended,
            applies_to: vec!["requirement".to_string()],
            content: "# Test 2".to_string(),
        };

        registry.register(playbook1).unwrap();
        registry.register(playbook2).unwrap();

        let loader = PlaybookLoader::new(registry);
        let playbooks = loader
            .load_for_scope("requirement", Some(&["code-review".to_string()]))
            .unwrap();

        assert_eq!(playbooks.len(), 1);
        assert_eq!(playbooks[0].uri, "radium://org/test1.md");
    }

    #[test]
    fn test_format_playbooks() {
        let playbooks = vec![
            Playbook {
                uri: "radium://org/test1.md".to_string(),
                description: "Required Playbook".to_string(),
                tags: vec!["test".to_string()],
                priority: PlaybookPriority::Required,
                applies_to: vec![],
                content: "# Required Content".to_string(),
            },
            Playbook {
                uri: "radium://org/test2.md".to_string(),
                description: "Recommended Playbook".to_string(),
                tags: vec![],
                priority: PlaybookPriority::Recommended,
                applies_to: vec![],
                content: "# Recommended Content".to_string(),
            },
        ];

        let formatted = PlaybookLoader::format_playbooks(&playbooks);
        assert!(formatted.contains("Organizational Playbooks"));
        assert!(formatted.contains("Priority: Required"));
        assert!(formatted.contains("Priority: Recommended"));
        assert!(formatted.contains("Required Content"));
        assert!(formatted.contains("Recommended Content"));
    }

    #[test]
    fn test_format_empty_playbooks() {
        let formatted = PlaybookLoader::format_playbooks(&[]);
        assert!(formatted.is_empty());
    }
}

