//! Error Suggestions
//!
//! Actionable suggestions for resolving errors.

use serde::{Deserialize, Serialize};

/// A suggestion for resolving an error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    /// Short description of the suggestion
    pub summary: String,
    /// Detailed explanation (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    /// Example code or configuration (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,
    /// Link to documentation (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs_url: Option<String>,
    /// Whether this is an automated fix
    #[serde(default)]
    pub is_auto_fixable: bool,
}

impl Suggestion {
    /// Create a simple suggestion
    pub fn new(summary: impl Into<String>) -> Self {
        Self {
            summary: summary.into(),
            details: None,
            example: None,
            docs_url: None,
            is_auto_fixable: false,
        }
    }

    /// Add detailed explanation
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// Add example code
    pub fn with_example(mut self, example: impl Into<String>) -> Self {
        self.example = Some(example.into());
        self
    }

    /// Add documentation link
    pub fn with_docs(mut self, url: impl Into<String>) -> Self {
        self.docs_url = Some(url.into());
        self
    }

    /// Mark as auto-fixable
    pub fn auto_fixable(mut self) -> Self {
        self.is_auto_fixable = true;
        self
    }
}

/// Builder for creating suggestion lists
#[derive(Debug, Clone, Default)]
pub struct SuggestionBuilder {
    suggestions: Vec<Suggestion>,
}

impl SuggestionBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a simple suggestion
    pub fn add(mut self, summary: impl Into<String>) -> Self {
        self.suggestions.push(Suggestion::new(summary));
        self
    }

    /// Add a suggestion with details
    pub fn add_detailed(
        mut self,
        summary: impl Into<String>,
        details: impl Into<String>,
    ) -> Self {
        self.suggestions.push(
            Suggestion::new(summary).with_details(details),
        );
        self
    }

    /// Add a suggestion with example
    pub fn add_with_example(
        mut self,
        summary: impl Into<String>,
        example: impl Into<String>,
    ) -> Self {
        self.suggestions.push(
            Suggestion::new(summary).with_example(example),
        );
        self
    }

    /// Build the suggestions list
    pub fn build(self) -> Vec<Suggestion> {
        self.suggestions
    }

    /// Build as simple strings
    pub fn build_strings(self) -> Vec<String> {
        self.suggestions.into_iter().map(|s| s.summary).collect()
    }
}

/// Common suggestion patterns
pub mod common {
    use super::*;

    /// Suggestion for adding a required field
    pub fn add_required_field(field_name: &str, type_hint: &str) -> Suggestion {
        Suggestion::new(format!("Add the required '{}' field", field_name))
            .with_details(format!(
                "The '{}' field is required and must be of type {}",
                field_name, type_hint
            ))
            .with_example(format!(r#""{field_name}": <value>"#))
    }

    /// Suggestion for fixing invalid identifier
    pub fn fix_identifier(invalid: &str, suggested: &str) -> Suggestion {
        Suggestion::new(format!("Rename '{}' to '{}'", invalid, suggested))
            .with_details("Identifiers must start with a letter and contain only alphanumeric characters and underscores")
            .auto_fixable()
    }

    /// Suggestion for removing duplicate
    pub fn remove_duplicate(id: &str) -> Suggestion {
        Suggestion::new(format!("Remove or rename duplicate '{}'", id))
            .with_details("Each node must have a unique identifier")
    }

    /// Suggestion for adding connection
    pub fn add_connection(from: &str, to: &str) -> Suggestion {
        Suggestion::new(format!("Connect '{}' to '{}'", from, to))
            .with_example(format!(
                r#"{{ "id": "edge_new", "source": "{}", "target": "{}" }}"#,
                from, to
            ))
    }

    /// Suggestion for adding trigger
    pub fn add_trigger() -> Suggestion {
        Suggestion::new("Add a trigger node to start the workflow")
            .with_details("Every workflow must have exactly one trigger node")
            .with_example(r#"{
  "id": "trigger_1",
  "type": "trigger",
  "data": { "label": "Start" },
  "position": { "x": 0, "y": 0 }
}"#)
    }

    /// Suggestion for adding end node
    pub fn add_end_node() -> Suggestion {
        Suggestion::new("Add an end node to complete the workflow")
            .with_details("Workflows should have at least one end node")
            .with_example(r#"{
  "id": "end_1",
  "type": "end",
  "data": { "label": "End" },
  "position": { "x": 200, "y": 0 }
}"#)
    }

    /// Suggestion for type conversion
    pub fn type_conversion(from_type: &str, to_type: &str) -> Suggestion {
        Suggestion::new(format!(
            "Convert from {} to {}",
            from_type, to_type
        ))
        .with_details(format!(
            "The value is of type {} but {} is expected. Consider adding a type conversion.",
            from_type, to_type
        ))
    }

    /// Suggestion for null handling
    pub fn handle_null(variable: &str) -> Suggestion {
        Suggestion::new(format!("Handle null case for '{}'", variable))
            .with_details("This value might be null. Add null checking or provide a default value.")
            .with_example(format!(
                "const value = {} ?? defaultValue;",
                variable
            ))
    }

    /// Suggestion for increasing timeout
    pub fn increase_timeout(current: &str, suggested: &str) -> Suggestion {
        Suggestion::new(format!(
            "Increase timeout from {} to {}",
            current, suggested
        ))
        .with_details("The operation may need more time to complete")
    }

    /// Suggestion for retry configuration
    pub fn add_retry() -> Suggestion {
        Suggestion::new("Add retry configuration")
            .with_details("Adding retry logic can help handle transient failures")
            .with_example(r#""retry": {
  "maxAttempts": 3,
  "initialInterval": "1s",
  "backoffCoefficient": 2
}"#)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggestion_creation() {
        let suggestion = Suggestion::new("Fix the error")
            .with_details("More details here")
            .with_example("example code")
            .with_docs("https://docs.example.com");

        assert_eq!(suggestion.summary, "Fix the error");
        assert_eq!(suggestion.details, Some("More details here".to_string()));
        assert_eq!(suggestion.example, Some("example code".to_string()));
        assert!(!suggestion.is_auto_fixable);
    }

    #[test]
    fn test_suggestion_builder() {
        let suggestions = SuggestionBuilder::new()
            .add("First suggestion")
            .add_detailed("Second suggestion", "With details")
            .add_with_example("Third suggestion", "code example")
            .build();

        assert_eq!(suggestions.len(), 3);
        assert!(suggestions[1].details.is_some());
        assert!(suggestions[2].example.is_some());
    }

    #[test]
    fn test_common_suggestions() {
        let suggestion = common::add_required_field("name", "string");
        assert!(suggestion.summary.contains("name"));
        assert!(suggestion.details.is_some());

        let suggestion = common::fix_identifier("123abc", "abc123");
        assert!(suggestion.is_auto_fixable);

        let suggestion = common::add_trigger();
        assert!(suggestion.example.is_some());
    }

    #[test]
    fn test_suggestion_builder_strings() {
        let strings = SuggestionBuilder::new()
            .add("First")
            .add("Second")
            .build_strings();

        assert_eq!(strings, vec!["First", "Second"]);
    }
}
