//! Variable definition and scoping
//!
//! Defines comprehensive variable definitions with scoping rules,
//! constraints, and metadata for workflow validation.

use serde::{Deserialize, Serialize};
use validator::Validate;

use super::{VariableType, VariableValue};

/// Variable scoping rules
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum VariableScope {
    /// Available throughout the entire workflow execution
    Workflow,
    /// Available only within a single activity
    Activity,
    /// Available only within the current block (e.g., loop iteration)
    #[default]
    Local,
}

impl VariableScope {
    /// Check if this scope is broader than another
    pub fn is_broader_than(&self, other: &VariableScope) -> bool {
        match (self, other) {
            (VariableScope::Workflow, VariableScope::Activity) => true,
            (VariableScope::Workflow, VariableScope::Local) => true,
            (VariableScope::Activity, VariableScope::Local) => true,
            _ => false,
        }
    }

    /// Get the TypeScript scope equivalent
    pub fn to_typescript(&self) -> &'static str {
        match self {
            VariableScope::Workflow => "workflow",
            VariableScope::Activity => "activity",
            VariableScope::Local => "local",
        }
    }
}

impl std::fmt::Display for VariableScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VariableScope::Workflow => write!(f, "workflow"),
            VariableScope::Activity => write!(f, "activity"),
            VariableScope::Local => write!(f, "local"),
        }
    }
}

/// Constraints that can be applied to variable values
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VariableConstraints {
    /// Minimum numeric value (for Number/Integer types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,

    /// Maximum numeric value (for Number/Integer types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,

    /// Minimum string length (for String types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,

    /// Maximum string length (for String types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,

    /// Regex pattern for validation (for String types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,

    /// Allowed values (enum constraint)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,

    /// Minimum array length (for Array types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_items: Option<usize>,

    /// Maximum array length (for Array types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<usize>,

    /// Whether the variable can be null
    #[serde(default)]
    pub nullable: bool,
}

impl VariableConstraints {
    /// Create new empty constraints
    pub fn new() -> Self {
        Self::default()
    }

    /// Set nullable flag
    pub fn nullable(mut self, nullable: bool) -> Self {
        self.nullable = nullable;
        self
    }

    /// Set min/max for numeric values
    pub fn range(mut self, min: Option<f64>, max: Option<f64>) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    /// Set min/max length for strings
    pub fn length(mut self, min: Option<usize>, max: Option<usize>) -> Self {
        self.min_length = min;
        self.max_length = max;
        self
    }

    /// Set pattern constraint
    pub fn pattern(mut self, pattern: impl Into<String>) -> Self {
        self.pattern = Some(pattern.into());
        self
    }

    /// Set enum values constraint
    pub fn enum_values(mut self, values: Vec<String>) -> Self {
        self.enum_values = Some(values);
        self
    }

    /// Validate a value against these constraints
    pub fn validate_value(&self, value: &VariableValue) -> Result<(), ConstraintViolation> {
        // Check nullable
        if value.is_null() {
            if self.nullable {
                return Ok(());
            } else {
                return Err(ConstraintViolation::NullNotAllowed);
            }
        }

        match value {
            VariableValue::Number(n) => {
                if let Some(min) = self.min {
                    if *n < min {
                        return Err(ConstraintViolation::BelowMinimum { min, actual: *n });
                    }
                }
                if let Some(max) = self.max {
                    if *n > max {
                        return Err(ConstraintViolation::AboveMaximum { max, actual: *n });
                    }
                }
            }
            VariableValue::Integer(i) => {
                if let Some(min) = self.min {
                    if (*i as f64) < min {
                        return Err(ConstraintViolation::BelowMinimum {
                            min,
                            actual: *i as f64,
                        });
                    }
                }
                if let Some(max) = self.max {
                    if (*i as f64) > max {
                        return Err(ConstraintViolation::AboveMaximum {
                            max,
                            actual: *i as f64,
                        });
                    }
                }
            }
            VariableValue::String(s) => {
                if let Some(min_len) = self.min_length {
                    if s.len() < min_len {
                        return Err(ConstraintViolation::TooShort {
                            min: min_len,
                            actual: s.len(),
                        });
                    }
                }
                if let Some(max_len) = self.max_length {
                    if s.len() > max_len {
                        return Err(ConstraintViolation::TooLong {
                            max: max_len,
                            actual: s.len(),
                        });
                    }
                }
                if let Some(pattern) = &self.pattern {
                    let re = regex::Regex::new(pattern)
                        .map_err(|_| ConstraintViolation::InvalidPattern(pattern.clone()))?;
                    if !re.is_match(s) {
                        return Err(ConstraintViolation::PatternMismatch {
                            pattern: pattern.clone(),
                            value: s.clone(),
                        });
                    }
                }
                if let Some(enum_values) = &self.enum_values {
                    if !enum_values.contains(s) {
                        return Err(ConstraintViolation::NotInEnum {
                            allowed: enum_values.clone(),
                            actual: s.clone(),
                        });
                    }
                }
            }
            VariableValue::Array(arr) => {
                if let Some(min_items) = self.min_items {
                    if arr.len() < min_items {
                        return Err(ConstraintViolation::TooFewItems {
                            min: min_items,
                            actual: arr.len(),
                        });
                    }
                }
                if let Some(max_items) = self.max_items {
                    if arr.len() > max_items {
                        return Err(ConstraintViolation::TooManyItems {
                            max: max_items,
                            actual: arr.len(),
                        });
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }
}

/// Constraint violation errors
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ConstraintViolation {
    #[error("null value is not allowed")]
    NullNotAllowed,

    #[error("value {actual} is below minimum {min}")]
    BelowMinimum { min: f64, actual: f64 },

    #[error("value {actual} is above maximum {max}")]
    AboveMaximum { max: f64, actual: f64 },

    #[error("string length {actual} is below minimum {min}")]
    TooShort { min: usize, actual: usize },

    #[error("string length {actual} is above maximum {max}")]
    TooLong { max: usize, actual: usize },

    #[error("value '{value}' does not match pattern '{pattern}'")]
    PatternMismatch { pattern: String, value: String },

    #[error("value '{actual}' is not in allowed values: {allowed:?}")]
    NotInEnum { allowed: Vec<String>, actual: String },

    #[error("array has {actual} items, minimum is {min}")]
    TooFewItems { min: usize, actual: usize },

    #[error("array has {actual} items, maximum is {max}")]
    TooManyItems { max: usize, actual: usize },

    #[error("invalid pattern: {0}")]
    InvalidPattern(String),
}

/// Complete variable definition with all metadata
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct VariableDefinition {
    /// Variable name (must be a valid identifier)
    #[validate(length(min = 1, message = "Variable name is required"))]
    pub name: String,

    /// Variable type
    #[serde(rename = "type")]
    pub variable_type: VariableType,

    /// Variable scope
    #[serde(default)]
    pub scope: VariableScope,

    /// Human-readable description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Default value if not provided
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<VariableValue>,

    /// Whether this variable is required (must be provided at workflow start)
    #[serde(default)]
    pub required: bool,

    /// Validation constraints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<VariableConstraints>,

    /// Expression for computed variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub computed: Option<String>,

    /// Source variable for transformations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// Transformation to apply (e.g., "uppercase", "trim", "parseInt")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<String>,
}

impl VariableDefinition {
    /// Create a new variable definition
    pub fn new(name: impl Into<String>, variable_type: VariableType) -> Self {
        Self {
            name: name.into(),
            variable_type,
            scope: VariableScope::default(),
            description: None,
            default_value: None,
            required: false,
            constraints: None,
            computed: None,
            source: None,
            transform: None,
        }
    }

    /// Set as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Set default value
    pub fn with_default(mut self, value: VariableValue) -> Self {
        self.default_value = Some(value);
        self
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set workflow scope
    pub fn workflow_scope(mut self) -> Self {
        self.scope = VariableScope::Workflow;
        self
    }

    /// Set activity scope
    pub fn activity_scope(mut self) -> Self {
        self.scope = VariableScope::Activity;
        self
    }

    /// Set constraints
    pub fn with_constraints(mut self, constraints: VariableConstraints) -> Self {
        self.constraints = Some(constraints);
        self
    }

    /// Set as computed with expression
    pub fn computed(mut self, expression: impl Into<String>) -> Self {
        self.computed = Some(expression.into());
        self
    }

    /// Set as transformed from source
    pub fn transformed_from(mut self, source: impl Into<String>, transform: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self.transform = Some(transform.into());
        self
    }

    /// Check if this is a computed variable
    pub fn is_computed(&self) -> bool {
        self.computed.is_some()
    }

    /// Check if this is a transformed variable
    pub fn is_transformed(&self) -> bool {
        self.source.is_some() && self.transform.is_some()
    }

    /// Check if the variable is nullable
    pub fn is_nullable(&self) -> bool {
        self.constraints
            .as_ref()
            .map(|c| c.nullable)
            .unwrap_or(false)
    }

    /// Validate a value against this definition
    pub fn validate_value(&self, value: &VariableValue) -> Result<(), DefinitionValidationError> {
        // Handle null values specially - they're allowed if the variable is nullable
        if value.is_null() {
            if self.is_nullable() {
                return Ok(());
            } else {
                return Err(DefinitionValidationError::ConstraintViolation(
                    ConstraintViolation::NullNotAllowed,
                ));
            }
        }

        // Check type compatibility
        if !value.is_compatible_with(&self.variable_type) {
            return Err(DefinitionValidationError::TypeMismatch {
                expected: self.variable_type.clone(),
                actual: value.type_of(),
            });
        }

        // Check constraints if any
        if let Some(constraints) = &self.constraints {
            constraints
                .validate_value(value)
                .map_err(DefinitionValidationError::ConstraintViolation)?;
        }

        Ok(())
    }

    /// Get the effective default value (or type default)
    pub fn effective_default(&self) -> VariableValue {
        if let Some(default) = &self.default_value {
            default.clone()
        } else if self.is_nullable() {
            VariableValue::Null
        } else {
            // Type-based defaults
            match &self.variable_type {
                VariableType::String => VariableValue::String(String::new()),
                VariableType::Number => VariableValue::Number(0.0),
                VariableType::Integer => VariableValue::Integer(0),
                VariableType::Boolean => VariableValue::Boolean(false),
                VariableType::Array => VariableValue::Array(vec![]),
                VariableType::Object => VariableValue::Object(serde_json::json!({})),
                VariableType::Datetime => VariableValue::Datetime(chrono::Utc::now()),
                VariableType::Duration => VariableValue::Duration(0),
                VariableType::Null => VariableValue::Null,
                VariableType::Any => VariableValue::Null,
            }
        }
    }

    /// Generate TypeScript type for this variable
    pub fn to_typescript_type(&self) -> String {
        let base_type = self.variable_type.to_typescript();
        if self.is_nullable() {
            format!("{} | null", base_type)
        } else {
            base_type.to_string()
        }
    }
}

/// Validation errors for variable definitions
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum DefinitionValidationError {
    #[error("type mismatch: expected {expected}, got {actual}")]
    TypeMismatch {
        expected: VariableType,
        actual: VariableType,
    },

    #[error("constraint violation: {0}")]
    ConstraintViolation(ConstraintViolation),

    #[error("variable name is not a valid identifier: {0}")]
    InvalidIdentifier(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_scope_ordering() {
        assert!(VariableScope::Workflow.is_broader_than(&VariableScope::Activity));
        assert!(VariableScope::Workflow.is_broader_than(&VariableScope::Local));
        assert!(VariableScope::Activity.is_broader_than(&VariableScope::Local));
        assert!(!VariableScope::Local.is_broader_than(&VariableScope::Workflow));
    }

    #[test]
    fn test_constraints_numeric_range() {
        let constraints = VariableConstraints::new().range(Some(0.0), Some(100.0));

        assert!(constraints.validate_value(&VariableValue::Number(50.0)).is_ok());
        assert!(constraints.validate_value(&VariableValue::Number(0.0)).is_ok());
        assert!(constraints.validate_value(&VariableValue::Number(100.0)).is_ok());

        let below_min = constraints.validate_value(&VariableValue::Number(-1.0));
        assert!(matches!(below_min, Err(ConstraintViolation::BelowMinimum { .. })));

        let above_max = constraints.validate_value(&VariableValue::Number(101.0));
        assert!(matches!(above_max, Err(ConstraintViolation::AboveMaximum { .. })));
    }

    #[test]
    fn test_constraints_string_length() {
        let constraints = VariableConstraints::new().length(Some(3), Some(10));

        assert!(constraints.validate_value(&VariableValue::String("hello".to_string())).is_ok());

        let too_short = constraints.validate_value(&VariableValue::String("hi".to_string()));
        assert!(matches!(too_short, Err(ConstraintViolation::TooShort { .. })));

        let too_long = constraints.validate_value(&VariableValue::String("this is way too long".to_string()));
        assert!(matches!(too_long, Err(ConstraintViolation::TooLong { .. })));
    }

    #[test]
    fn test_constraints_pattern() {
        let constraints = VariableConstraints::new().pattern(r"^\d{3}-\d{4}$");

        assert!(constraints.validate_value(&VariableValue::String("123-4567".to_string())).is_ok());

        let mismatch = constraints.validate_value(&VariableValue::String("invalid".to_string()));
        assert!(matches!(mismatch, Err(ConstraintViolation::PatternMismatch { .. })));
    }

    #[test]
    fn test_constraints_enum() {
        let constraints = VariableConstraints::new()
            .enum_values(vec!["red".to_string(), "green".to_string(), "blue".to_string()]);

        assert!(constraints.validate_value(&VariableValue::String("red".to_string())).is_ok());

        let not_in_enum = constraints.validate_value(&VariableValue::String("yellow".to_string()));
        assert!(matches!(not_in_enum, Err(ConstraintViolation::NotInEnum { .. })));
    }

    #[test]
    fn test_constraints_nullable() {
        let not_nullable = VariableConstraints::new();
        let nullable = VariableConstraints::new().nullable(true);

        assert!(not_nullable.validate_value(&VariableValue::Null).is_err());
        assert!(nullable.validate_value(&VariableValue::Null).is_ok());
    }

    #[test]
    fn test_variable_definition_builder() {
        let def = VariableDefinition::new("userId", VariableType::String)
            .required()
            .workflow_scope()
            .with_description("The user identifier")
            .with_constraints(VariableConstraints::new().length(Some(1), Some(100)));

        assert_eq!(def.name, "userId");
        assert_eq!(def.variable_type, VariableType::String);
        assert!(def.required);
        assert_eq!(def.scope, VariableScope::Workflow);
        assert!(def.description.is_some());
        assert!(def.constraints.is_some());
    }

    #[test]
    fn test_variable_definition_validate_value() {
        let def = VariableDefinition::new("count", VariableType::Integer)
            .with_constraints(VariableConstraints::new().range(Some(0.0), Some(100.0)));

        assert!(def.validate_value(&VariableValue::Integer(50)).is_ok());
        assert!(def.validate_value(&VariableValue::Integer(101)).is_err());
        assert!(def.validate_value(&VariableValue::String("not a number".to_string())).is_err());
    }

    #[test]
    fn test_computed_and_transformed() {
        let computed = VariableDefinition::new("total", VariableType::Number)
            .computed("price * quantity");

        assert!(computed.is_computed());
        assert!(!computed.is_transformed());

        let transformed = VariableDefinition::new("upperName", VariableType::String)
            .transformed_from("name", "uppercase");

        assert!(!transformed.is_computed());
        assert!(transformed.is_transformed());
    }

    #[test]
    fn test_effective_default() {
        let with_default = VariableDefinition::new("count", VariableType::Integer)
            .with_default(VariableValue::Integer(42));

        assert_eq!(with_default.effective_default(), VariableValue::Integer(42));

        let without_default = VariableDefinition::new("count", VariableType::Integer);
        assert_eq!(without_default.effective_default(), VariableValue::Integer(0));

        let nullable = VariableDefinition::new("optional", VariableType::String)
            .with_constraints(VariableConstraints::new().nullable(true));
        assert_eq!(nullable.effective_default(), VariableValue::Null);
    }

    #[test]
    fn test_typescript_type_generation() {
        let required = VariableDefinition::new("name", VariableType::String);
        assert_eq!(required.to_typescript_type(), "string");

        let nullable = VariableDefinition::new("name", VariableType::String)
            .with_constraints(VariableConstraints::new().nullable(true));
        assert_eq!(nullable.to_typescript_type(), "string | null");
    }

    #[test]
    fn test_serialization() {
        let def = VariableDefinition::new("userId", VariableType::String)
            .required()
            .with_description("User ID");

        let json = serde_json::to_string_pretty(&def).unwrap();
        assert!(json.contains("\"name\": \"userId\""));
        assert!(json.contains("\"type\": \"string\""));
        assert!(json.contains("\"required\": true"));

        // Deserialize back
        let parsed: VariableDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "userId");
        assert_eq!(parsed.variable_type, VariableType::String);
        assert!(parsed.required);
    }
}
