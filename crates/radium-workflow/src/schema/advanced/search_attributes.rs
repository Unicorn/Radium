//! Search Attributes
//!
//! Implement search attributes for workflow discovery:
//! - Attribute type definitions
//! - Attribute updates at runtime
//! - Standard attributes
//! - Query support

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

/// Search attribute type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum SearchAttributeType {
    /// Boolean value
    Bool,
    /// DateTime value
    Datetime,
    /// Floating point value
    Double,
    /// Integer value
    Int,
    /// Keyword (exact match string)
    Keyword,
    /// List of keywords
    KeywordList,
    /// Full-text searchable string
    Text,
}

impl SearchAttributeType {
    /// Get the TypeScript type for this attribute
    pub fn typescript_type(&self) -> &'static str {
        match self {
            SearchAttributeType::Bool => "boolean",
            SearchAttributeType::Datetime => "Date",
            SearchAttributeType::Double => "number",
            SearchAttributeType::Int => "number",
            SearchAttributeType::Keyword => "string",
            SearchAttributeType::KeywordList => "string[]",
            SearchAttributeType::Text => "string",
        }
    }

    /// Get the Temporal attribute type name
    pub fn temporal_type(&self) -> &'static str {
        match self {
            SearchAttributeType::Bool => "Bool",
            SearchAttributeType::Datetime => "Datetime",
            SearchAttributeType::Double => "Double",
            SearchAttributeType::Int => "Int",
            SearchAttributeType::Keyword => "Keyword",
            SearchAttributeType::KeywordList => "KeywordList",
            SearchAttributeType::Text => "Text",
        }
    }
}

/// Search attribute definition
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SearchAttributeDefinition {
    /// Attribute name
    #[validate(length(min = 1))]
    pub name: String,

    /// Attribute type
    pub attribute_type: SearchAttributeType,

    /// Description of the attribute
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Default value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,

    /// Whether this attribute is indexed for search
    #[serde(default = "default_true")]
    pub indexed: bool,
}

fn default_true() -> bool {
    true
}

impl SearchAttributeDefinition {
    /// Create a new attribute definition
    pub fn new(name: impl Into<String>, attribute_type: SearchAttributeType) -> Self {
        Self {
            name: name.into(),
            attribute_type,
            description: None,
            default: None,
            indexed: true,
        }
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set default value
    pub fn with_default(mut self, value: serde_json::Value) -> Self {
        self.default = Some(value);
        self
    }

    /// Mark as not indexed
    pub fn not_indexed(mut self) -> Self {
        self.indexed = false;
        self
    }

    /// Generate TypeScript interface field
    pub fn to_typescript_field(&self) -> String {
        let optional = if self.default.is_some() { "?" } else { "" };
        format!(
            "{}{}: {};",
            self.name,
            optional,
            self.attribute_type.typescript_type()
        )
    }
}

/// Search attribute value with type information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum TypedSearchAttributeValue {
    /// Boolean value
    Bool(bool),
    /// DateTime value
    Datetime(DateTime<Utc>),
    /// Floating point value
    Double(f64),
    /// Integer value
    Int(i64),
    /// Keyword string
    Keyword(String),
    /// List of keywords
    KeywordList(Vec<String>),
    /// Text string
    Text(String),
}

impl TypedSearchAttributeValue {
    /// Convert to TypeScript representation
    pub fn to_typescript(&self) -> String {
        match self {
            TypedSearchAttributeValue::Bool(b) => b.to_string(),
            TypedSearchAttributeValue::Datetime(dt) => {
                format!("new Date('{}')", dt.to_rfc3339())
            }
            TypedSearchAttributeValue::Double(d) => d.to_string(),
            TypedSearchAttributeValue::Int(i) => i.to_string(),
            TypedSearchAttributeValue::Keyword(s) => format!("'{}'", s),
            TypedSearchAttributeValue::KeywordList(list) => {
                let items: Vec<_> = list.iter().map(|s| format!("'{}'", s)).collect();
                format!("[{}]", items.join(", "))
            }
            TypedSearchAttributeValue::Text(s) => format!("'{}'", s),
        }
    }
}

/// An update to a search attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchAttributeUpdate {
    /// Attribute name
    pub name: String,
    /// New value
    pub value: TypedSearchAttributeValue,
}

impl SearchAttributeUpdate {
    /// Create a new update
    pub fn new(name: impl Into<String>, value: TypedSearchAttributeValue) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }

    /// Convert to TypeScript update code
    pub fn to_typescript(&self) -> String {
        format!(
            "upsertSearchAttributes({{ {}: [{}] }});",
            self.name,
            self.value.to_typescript()
        )
    }
}

/// Collection of search attributes for a workflow
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowSearchAttributes {
    /// Attribute definitions
    #[serde(default)]
    pub definitions: HashMap<String, SearchAttributeDefinition>,
    /// Initial values
    #[serde(default)]
    pub initial_values: HashMap<String, TypedSearchAttributeValue>,
}

impl WorkflowSearchAttributes {
    /// Create empty search attributes
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with standard attributes
    pub fn with_standard_attributes() -> Self {
        let mut attrs = Self::new();
        for def in standard_search_attributes() {
            attrs.add_definition(def);
        }
        attrs
    }

    /// Add an attribute definition
    pub fn add_definition(&mut self, definition: SearchAttributeDefinition) {
        self.definitions
            .insert(definition.name.clone(), definition);
    }

    /// Set initial value for an attribute
    pub fn set_initial_value(&mut self, name: &str, value: TypedSearchAttributeValue) {
        self.initial_values.insert(name.to_string(), value);
    }

    /// Get definition by name
    pub fn get_definition(&self, name: &str) -> Option<&SearchAttributeDefinition> {
        self.definitions.get(name)
    }

    /// Generate TypeScript interface for search attributes
    pub fn to_typescript_interface(&self) -> String {
        let mut code = String::from("interface WorkflowSearchAttributes {\n");

        for def in self.definitions.values() {
            if let Some(desc) = &def.description {
                code.push_str(&format!("  /** {} */\n", desc));
            }
            code.push_str(&format!("  {}\n", def.to_typescript_field()));
        }

        code.push_str("}\n");
        code
    }

    /// Generate TypeScript initial values object
    pub fn to_typescript_initial_values(&self) -> String {
        if self.initial_values.is_empty() {
            return "{}".to_string();
        }

        let mut parts: Vec<String> = Vec::new();
        for (name, value) in &self.initial_values {
            parts.push(format!("{}: [{}]", name, value.to_typescript()));
        }

        format!("{{\n  {}\n}}", parts.join(",\n  "))
    }

    /// Generate TypeScript code for upsertSearchAttributes helper
    pub fn to_typescript_helper() -> &'static str {
        r#"import { upsertSearchAttributes } from '@temporalio/workflow';

/**
 * Update a search attribute value
 */
function updateSearchAttribute(name: string, value: unknown): void {
  upsertSearchAttributes({ [name]: [value] });
}
"#
    }
}

/// Standard search attributes available on all workflows
pub fn standard_search_attributes() -> Vec<SearchAttributeDefinition> {
    vec![
        SearchAttributeDefinition::new("CustomStatus", SearchAttributeType::Keyword)
            .with_description("Custom workflow status for filtering")
            .with_default(serde_json::json!("pending")),
        SearchAttributeDefinition::new("CustomStringField", SearchAttributeType::Text)
            .with_description("Custom searchable text field"),
        SearchAttributeDefinition::new("CustomIntField", SearchAttributeType::Int)
            .with_description("Custom integer field for filtering"),
        SearchAttributeDefinition::new("CustomBoolField", SearchAttributeType::Bool)
            .with_description("Custom boolean field for filtering"),
        SearchAttributeDefinition::new("CustomDatetimeField", SearchAttributeType::Datetime)
            .with_description("Custom datetime field for time-based queries"),
        SearchAttributeDefinition::new("CustomKeywordListField", SearchAttributeType::KeywordList)
            .with_description("Custom list of keywords for multi-value filtering"),
    ]
}

/// Pre-defined useful search attributes
pub mod presets {
    use super::*;

    /// User ID attribute
    pub fn user_id() -> SearchAttributeDefinition {
        SearchAttributeDefinition::new("UserId", SearchAttributeType::Keyword)
            .with_description("ID of the user who initiated the workflow")
    }

    /// Tenant ID attribute
    pub fn tenant_id() -> SearchAttributeDefinition {
        SearchAttributeDefinition::new("TenantId", SearchAttributeType::Keyword)
            .with_description("ID of the tenant for multi-tenant systems")
    }

    /// Environment attribute
    pub fn environment() -> SearchAttributeDefinition {
        SearchAttributeDefinition::new("Environment", SearchAttributeType::Keyword)
            .with_description("Deployment environment (prod, staging, dev)")
    }

    /// Priority attribute
    pub fn priority() -> SearchAttributeDefinition {
        SearchAttributeDefinition::new("Priority", SearchAttributeType::Int)
            .with_description("Workflow priority (higher = more important)")
    }

    /// Tags attribute
    pub fn tags() -> SearchAttributeDefinition {
        SearchAttributeDefinition::new("Tags", SearchAttributeType::KeywordList)
            .with_description("List of tags for categorization")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_attribute_type_typescript() {
        assert_eq!(SearchAttributeType::Bool.typescript_type(), "boolean");
        assert_eq!(SearchAttributeType::Int.typescript_type(), "number");
        assert_eq!(SearchAttributeType::Keyword.typescript_type(), "string");
        assert_eq!(SearchAttributeType::KeywordList.typescript_type(), "string[]");
    }

    #[test]
    fn test_search_attribute_definition() {
        let def = SearchAttributeDefinition::new("CustomStatus", SearchAttributeType::Keyword)
            .with_description("Workflow status")
            .with_default(serde_json::json!("pending"));

        assert_eq!(def.name, "CustomStatus");
        assert!(def.description.is_some());
        assert!(def.default.is_some());
        assert!(def.indexed);
    }

    #[test]
    fn test_search_attribute_definition_typescript_field() {
        let def = SearchAttributeDefinition::new("Status", SearchAttributeType::Keyword);
        assert_eq!(def.to_typescript_field(), "Status: string;");

        let def_with_default = def.with_default(serde_json::json!("pending"));
        assert_eq!(def_with_default.to_typescript_field(), "Status?: string;");
    }

    #[test]
    fn test_typed_search_attribute_value_typescript() {
        assert_eq!(TypedSearchAttributeValue::Bool(true).to_typescript(), "true");
        assert_eq!(TypedSearchAttributeValue::Int(42).to_typescript(), "42");
        assert_eq!(
            TypedSearchAttributeValue::Keyword("test".to_string()).to_typescript(),
            "'test'"
        );
        assert_eq!(
            TypedSearchAttributeValue::KeywordList(vec!["a".to_string(), "b".to_string()])
                .to_typescript(),
            "['a', 'b']"
        );
    }

    #[test]
    fn test_search_attribute_update() {
        let update = SearchAttributeUpdate::new(
            "CustomStatus",
            TypedSearchAttributeValue::Keyword("completed".to_string()),
        );

        let ts = update.to_typescript();
        assert!(ts.contains("upsertSearchAttributes"));
        assert!(ts.contains("CustomStatus"));
        assert!(ts.contains("'completed'"));
    }

    #[test]
    fn test_workflow_search_attributes() {
        let mut attrs = WorkflowSearchAttributes::new();
        attrs.add_definition(
            SearchAttributeDefinition::new("UserId", SearchAttributeType::Keyword),
        );
        attrs.set_initial_value(
            "UserId",
            TypedSearchAttributeValue::Keyword("user-123".to_string()),
        );

        assert!(attrs.get_definition("UserId").is_some());
        assert!(attrs.initial_values.contains_key("UserId"));
    }

    #[test]
    fn test_workflow_search_attributes_typescript_interface() {
        let mut attrs = WorkflowSearchAttributes::new();
        attrs.add_definition(
            SearchAttributeDefinition::new("Status", SearchAttributeType::Keyword)
                .with_description("Current status"),
        );
        attrs.add_definition(SearchAttributeDefinition::new("Count", SearchAttributeType::Int));

        let ts = attrs.to_typescript_interface();
        assert!(ts.contains("interface WorkflowSearchAttributes"));
        assert!(ts.contains("Status: string"));
        assert!(ts.contains("Count: number"));
    }

    #[test]
    fn test_standard_search_attributes() {
        let attrs = standard_search_attributes();
        assert!(!attrs.is_empty());

        let names: Vec<_> = attrs.iter().map(|a| a.name.as_str()).collect();
        assert!(names.contains(&"CustomStatus"));
    }

    #[test]
    fn test_preset_attributes() {
        let user_id = presets::user_id();
        assert_eq!(user_id.name, "UserId");
        assert_eq!(user_id.attribute_type, SearchAttributeType::Keyword);

        let tags = presets::tags();
        assert_eq!(tags.attribute_type, SearchAttributeType::KeywordList);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let def = SearchAttributeDefinition::new("Test", SearchAttributeType::Bool)
            .with_description("Test attribute");

        let json = serde_json::to_string(&def).unwrap();
        let restored: SearchAttributeDefinition = serde_json::from_str(&json).unwrap();

        assert_eq!(def.name, restored.name);
        assert_eq!(def.attribute_type, restored.attribute_type);
    }
}
