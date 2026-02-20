//! Variable reference and lookup system
//!
//! Implements JSONPath-style variable references for accessing nested data
//! within workflow state.

use serde::{Deserialize, Serialize};
use std::str::FromStr;

use super::VariableType;

/// A reference to a variable within workflow state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VariableReference {
    /// Full path string (e.g., "$.workflow.input.userId")
    pub path: String,

    /// Parsed path segments
    pub segments: Vec<PathSegment>,

    /// Expected type (if known from definition)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_type: Option<VariableType>,
}

/// Segments of a path expression
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PathSegment {
    /// Root reference ($)
    Root,
    /// Property access (.name)
    Property(String),
    /// Array index access ([0])
    Index(usize),
    /// Wildcard for all array elements ([*])
    Wildcard,
    /// Filter expression ([?(@.active)])
    Filter(String),
}

impl VariableReference {
    /// Parse a JSONPath-style reference string
    pub fn parse(path: &str) -> Result<Self, ReferenceParseError> {
        if !path.starts_with('$') {
            return Err(ReferenceParseError::MissingRoot);
        }

        let mut segments = vec![PathSegment::Root];
        let mut current = &path[1..];

        while !current.is_empty() {
            if current.starts_with('.') {
                // Property access
                current = &current[1..];
                let end = current
                    .find(|c: char| c == '.' || c == '[')
                    .unwrap_or(current.len());
                let prop = &current[..end];
                if prop.is_empty() {
                    return Err(ReferenceParseError::EmptyProperty);
                }
                // Validate property name is a valid identifier
                if !is_valid_identifier(prop) {
                    return Err(ReferenceParseError::InvalidIdentifier(prop.to_string()));
                }
                segments.push(PathSegment::Property(prop.to_string()));
                current = &current[end..];
            } else if current.starts_with('[') {
                // Index, wildcard, or filter
                let end = find_matching_bracket(current)?;
                let inner = &current[1..end];

                if inner.is_empty() {
                    return Err(ReferenceParseError::EmptyBracket);
                } else if inner == "*" {
                    segments.push(PathSegment::Wildcard);
                } else if inner.starts_with("?(") && inner.ends_with(')') {
                    let filter_expr = &inner[2..inner.len() - 1];
                    segments.push(PathSegment::Filter(filter_expr.to_string()));
                } else {
                    // Try to parse as index
                    let idx: usize = inner
                        .parse()
                        .map_err(|_| ReferenceParseError::InvalidIndex(inner.to_string()))?;
                    segments.push(PathSegment::Index(idx));
                }
                current = &current[end + 1..];
            } else {
                return Err(ReferenceParseError::UnexpectedCharacter(
                    current.chars().next().unwrap(),
                ));
            }
        }

        Ok(Self {
            path: path.to_string(),
            segments,
            expected_type: None,
        })
    }

    /// Create a simple variable reference
    pub fn simple(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            path: format!("$.{}", name),
            segments: vec![PathSegment::Root, PathSegment::Property(name)],
            expected_type: None,
        }
    }

    /// Set the expected type for this reference
    pub fn with_type(mut self, var_type: VariableType) -> Self {
        self.expected_type = Some(var_type);
        self
    }

    /// Generate TypeScript code for accessing this reference
    pub fn to_typescript(&self) -> String {
        let mut result = String::new();
        let mut in_wildcard = false;

        for segment in &self.segments {
            match segment {
                PathSegment::Root => result.push_str("state"),
                PathSegment::Property(name) => {
                    if in_wildcard {
                        result.push('.');
                        result.push_str(name);
                    } else {
                        result.push('.');
                        result.push_str(name);
                    }
                }
                PathSegment::Index(idx) => {
                    result.push('[');
                    result.push_str(&idx.to_string());
                    result.push(']');
                }
                PathSegment::Wildcard => {
                    // For wildcards, we need to use map
                    result = format!("{}.map(item => item", result);
                    in_wildcard = true;
                }
                PathSegment::Filter(expr) => {
                    // Convert @. references to item.
                    let ts_expr = expr.replace("@.", "item.");
                    result = format!("{}.filter(item => {})", result, ts_expr);
                }
            }
        }

        // Close any open wildcard maps
        if in_wildcard {
            result.push(')');
        }

        result
    }

    /// Generate TypeScript code with optional chaining for safe access
    pub fn to_typescript_safe(&self) -> String {
        let mut result = String::new();

        for segment in &self.segments {
            match segment {
                PathSegment::Root => result.push_str("state"),
                PathSegment::Property(name) => {
                    result.push_str("?.");
                    result.push_str(name);
                }
                PathSegment::Index(idx) => {
                    result.push_str("?.[");
                    result.push_str(&idx.to_string());
                    result.push(']');
                }
                PathSegment::Wildcard => {
                    result = format!("{}?.map(item => item", result);
                }
                PathSegment::Filter(expr) => {
                    let ts_expr = expr.replace("@.", "item.");
                    result = format!("{}?.filter(item => {})", result, ts_expr);
                }
            }
        }

        result
    }

    /// Get the root variable name (first property after $)
    pub fn root_variable(&self) -> Option<&str> {
        self.segments.get(1).and_then(|s| {
            if let PathSegment::Property(name) = s {
                Some(name.as_str())
            } else {
                None
            }
        })
    }

    /// Get the full property path (excluding root)
    pub fn property_path(&self) -> Vec<&str> {
        self.segments
            .iter()
            .filter_map(|s| {
                if let PathSegment::Property(name) = s {
                    Some(name.as_str())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Check if this reference contains a wildcard or filter
    pub fn is_collection_access(&self) -> bool {
        self.segments
            .iter()
            .any(|s| matches!(s, PathSegment::Wildcard | PathSegment::Filter(_)))
    }

    /// Check if this reference has any array index access
    pub fn has_index_access(&self) -> bool {
        self.segments
            .iter()
            .any(|s| matches!(s, PathSegment::Index(_)))
    }

    /// Get all variable names referenced (for dependency tracking)
    pub fn referenced_variables(&self) -> Vec<String> {
        let mut vars = Vec::new();

        if let Some(root) = self.root_variable() {
            vars.push(root.to_string());
        }

        // Also extract variables from filter expressions
        for segment in &self.segments {
            if let PathSegment::Filter(expr) = segment {
                // Simple extraction of @.property references
                let mut remaining = expr.as_str();
                while let Some(pos) = remaining.find("@.") {
                    remaining = &remaining[pos + 2..];
                    let end = remaining
                        .find(|c: char| !c.is_alphanumeric() && c != '_')
                        .unwrap_or(remaining.len());
                    if end > 0 {
                        // This is a reference within the filter, not a root variable
                        remaining = &remaining[end..];
                    }
                }
            }
        }

        vars
    }
}

impl FromStr for VariableReference {
    type Err = ReferenceParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl std::fmt::Display for VariableReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path)
    }
}

/// Errors that can occur when parsing a variable reference
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ReferenceParseError {
    #[error("variable reference must start with $")]
    MissingRoot,

    #[error("empty property name after '.'")]
    EmptyProperty,

    #[error("unclosed bracket in reference")]
    UnclosedBracket,

    #[error("empty brackets '[]' are not allowed")]
    EmptyBracket,

    #[error("invalid array index: {0}")]
    InvalidIndex(String),

    #[error("invalid identifier: {0}")]
    InvalidIdentifier(String),

    #[error("unexpected character: {0}")]
    UnexpectedCharacter(char),
}

/// Find the matching closing bracket
fn find_matching_bracket(s: &str) -> Result<usize, ReferenceParseError> {
    let mut depth = 0;
    for (i, c) in s.chars().enumerate() {
        match c {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(i);
                }
            }
            _ => {}
        }
    }
    Err(ReferenceParseError::UnclosedBracket)
}

/// Check if a string is a valid identifier
fn is_valid_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    if !first.is_alphabetic() && first != '_' {
        return false;
    }
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_reference() {
        let ref1 = VariableReference::parse("$.name").unwrap();
        assert_eq!(ref1.segments.len(), 2);
        assert_eq!(ref1.root_variable(), Some("name"));
        assert_eq!(ref1.to_typescript(), "state.name");
    }

    #[test]
    fn test_nested_reference() {
        let ref1 = VariableReference::parse("$.workflow.input.userId").unwrap();
        assert_eq!(ref1.segments.len(), 4);
        assert_eq!(ref1.root_variable(), Some("workflow"));
        assert_eq!(ref1.property_path(), vec!["workflow", "input", "userId"]);
        assert_eq!(ref1.to_typescript(), "state.workflow.input.userId");
    }

    #[test]
    fn test_array_index() {
        let ref1 = VariableReference::parse("$.items[0].name").unwrap();
        assert_eq!(ref1.segments.len(), 4);
        assert!(ref1.has_index_access());
        assert_eq!(ref1.to_typescript(), "state.items[0].name");
    }

    #[test]
    fn test_multiple_indices() {
        let ref1 = VariableReference::parse("$.matrix[0][1]").unwrap();
        assert_eq!(ref1.segments.len(), 4);
        assert_eq!(ref1.to_typescript(), "state.matrix[0][1]");
    }

    #[test]
    fn test_wildcard() {
        let ref1 = VariableReference::parse("$.items[*].id").unwrap();
        assert!(ref1.is_collection_access());
        assert_eq!(ref1.to_typescript(), "state.items.map(item => item.id)");
    }

    #[test]
    fn test_filter() {
        let ref1 = VariableReference::parse("$.items[?(@.active)].name").unwrap();
        assert!(ref1.is_collection_access());
        assert_eq!(
            ref1.to_typescript(),
            "state.items.filter(item => item.active).name"
        );
    }

    #[test]
    fn test_safe_access() {
        let ref1 = VariableReference::parse("$.user.profile.email").unwrap();
        assert_eq!(
            ref1.to_typescript_safe(),
            "state?.user?.profile?.email"
        );
    }

    #[test]
    fn test_simple_factory() {
        let ref1 = VariableReference::simple("userId");
        assert_eq!(ref1.path, "$.userId");
        assert_eq!(ref1.root_variable(), Some("userId"));
    }

    #[test]
    fn test_with_type() {
        let ref1 = VariableReference::simple("count").with_type(VariableType::Integer);
        assert_eq!(ref1.expected_type, Some(VariableType::Integer));
    }

    #[test]
    fn test_missing_root() {
        let err = VariableReference::parse("name.value").unwrap_err();
        assert_eq!(err, ReferenceParseError::MissingRoot);
    }

    #[test]
    fn test_empty_property() {
        let err = VariableReference::parse("$..value").unwrap_err();
        assert_eq!(err, ReferenceParseError::EmptyProperty);
    }

    #[test]
    fn test_invalid_index() {
        let err = VariableReference::parse("$.items[abc]").unwrap_err();
        assert!(matches!(err, ReferenceParseError::InvalidIndex(_)));
    }

    #[test]
    fn test_unclosed_bracket() {
        let err = VariableReference::parse("$.items[0").unwrap_err();
        assert_eq!(err, ReferenceParseError::UnclosedBracket);
    }

    #[test]
    fn test_referenced_variables() {
        let ref1 = VariableReference::parse("$.workflow.input.userId").unwrap();
        assert_eq!(ref1.referenced_variables(), vec!["workflow"]);

        let ref2 = VariableReference::simple("count");
        assert_eq!(ref2.referenced_variables(), vec!["count"]);
    }

    #[test]
    fn test_from_str() {
        let ref1: VariableReference = "$.name".parse().unwrap();
        assert_eq!(ref1.root_variable(), Some("name"));
    }

    #[test]
    fn test_display() {
        let ref1 = VariableReference::parse("$.user.name").unwrap();
        assert_eq!(format!("{}", ref1), "$.user.name");
    }

    #[test]
    fn test_valid_identifiers() {
        assert!(is_valid_identifier("name"));
        assert!(is_valid_identifier("_private"));
        assert!(is_valid_identifier("name123"));
        assert!(is_valid_identifier("snake_case"));

        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("123name"));
        assert!(!is_valid_identifier("name-with-dash"));
    }

    #[test]
    fn test_complex_filter() {
        let ref1 = VariableReference::parse("$.orders[?(@.status == 'active')]").unwrap();
        assert!(ref1.is_collection_access());
    }

    #[test]
    fn test_serialization() {
        let ref1 = VariableReference::parse("$.user.name").unwrap();
        let json = serde_json::to_string(&ref1).unwrap();
        assert!(json.contains("\"path\":\"$.user.name\""));

        let parsed: VariableReference = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.path, "$.user.name");
    }
}
