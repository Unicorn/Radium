//! Variable type definitions
//!
//! Defines all supported variable types for workflow definitions.
//! These types are used throughout the compiler for type checking
//! and TypeScript code generation.

use serde::{Deserialize, Serialize};
use std::fmt;

/// All supported variable types in workflow definitions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum VariableType {
    /// Text values
    String,
    /// Floating-point numeric values
    Number,
    /// Whole number values
    Integer,
    /// True/false values
    Boolean,
    /// JSON objects
    Object,
    /// Arrays/lists
    Array,
    /// Timestamps (ISO 8601)
    Datetime,
    /// Time durations (milliseconds)
    Duration,
    /// Null value
    Null,
    /// Any type (for migration compatibility, generates warning)
    #[serde(rename = "any")]
    Any,
}

impl VariableType {
    /// Check if this is a primitive type
    pub fn is_primitive(&self) -> bool {
        matches!(
            self,
            Self::String | Self::Number | Self::Integer | Self::Boolean | Self::Null
        )
    }

    /// Check if this is a complex/composite type
    pub fn is_complex(&self) -> bool {
        matches!(self, Self::Object | Self::Array)
    }

    /// Check if this is a temporal type
    pub fn is_temporal(&self) -> bool {
        matches!(self, Self::Datetime | Self::Duration)
    }

    /// Check if this is the Any type (which should generate warnings)
    pub fn is_any(&self) -> bool {
        matches!(self, Self::Any)
    }

    /// Get the TypeScript type equivalent
    pub fn to_typescript(&self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Number | Self::Integer | Self::Duration => "number",
            Self::Boolean => "boolean",
            Self::Object => "Record<string, unknown>",
            Self::Array => "unknown[]",
            Self::Datetime => "Date",
            Self::Null => "null",
            Self::Any => "unknown", // Never use 'any' - use 'unknown' for type safety
        }
    }

    /// Get the default value as TypeScript code
    pub fn default_value_ts(&self) -> &'static str {
        match self {
            Self::String => "''",
            Self::Number | Self::Integer | Self::Duration => "0",
            Self::Boolean => "false",
            Self::Object => "{}",
            Self::Array => "[]",
            Self::Datetime => "new Date()",
            Self::Null => "null",
            Self::Any => "undefined",
        }
    }

    /// Get the Rust type equivalent for documentation
    pub fn rust_type(&self) -> &'static str {
        match self {
            Self::String => "String",
            Self::Number => "f64",
            Self::Integer => "i64",
            Self::Boolean => "bool",
            Self::Object => "serde_json::Value",
            Self::Array => "Vec<serde_json::Value>",
            Self::Datetime => "chrono::DateTime<Utc>",
            Self::Duration => "chrono::Duration",
            Self::Null => "Option<()>",
            Self::Any => "serde_json::Value",
        }
    }

    /// Check if a value of this type can be assigned to a variable of the expected type
    pub fn is_assignable_to(&self, expected: &VariableType) -> bool {
        match (self, expected) {
            // Any accepts all types
            (_, VariableType::Any) => true,
            // Same types are always compatible
            (a, b) if a == b => true,
            // Integer can be assigned to Number
            (VariableType::Integer, VariableType::Number) => true,
            // Null can be assigned to any nullable type (handled at VariableDefinition level)
            // Other combinations are not compatible
            _ => false,
        }
    }
}

impl Default for VariableType {
    fn default() -> Self {
        Self::String
    }
}

impl fmt::Display for VariableType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String => write!(f, "string"),
            Self::Number => write!(f, "number"),
            Self::Integer => write!(f, "integer"),
            Self::Boolean => write!(f, "boolean"),
            Self::Object => write!(f, "object"),
            Self::Array => write!(f, "array"),
            Self::Datetime => write!(f, "datetime"),
            Self::Duration => write!(f, "duration"),
            Self::Null => write!(f, "null"),
            Self::Any => write!(f, "any"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_type_serialization() {
        assert_eq!(serde_json::to_string(&VariableType::String).unwrap(), "\"string\"");
        assert_eq!(serde_json::to_string(&VariableType::Number).unwrap(), "\"number\"");
        assert_eq!(serde_json::to_string(&VariableType::Integer).unwrap(), "\"integer\"");
        assert_eq!(serde_json::to_string(&VariableType::Boolean).unwrap(), "\"boolean\"");
        assert_eq!(serde_json::to_string(&VariableType::Object).unwrap(), "\"object\"");
        assert_eq!(serde_json::to_string(&VariableType::Array).unwrap(), "\"array\"");
        assert_eq!(serde_json::to_string(&VariableType::Datetime).unwrap(), "\"datetime\"");
        assert_eq!(serde_json::to_string(&VariableType::Duration).unwrap(), "\"duration\"");
        assert_eq!(serde_json::to_string(&VariableType::Null).unwrap(), "\"null\"");
        assert_eq!(serde_json::to_string(&VariableType::Any).unwrap(), "\"any\"");
    }

    #[test]
    fn test_variable_type_deserialization() {
        assert_eq!(serde_json::from_str::<VariableType>("\"string\"").unwrap(), VariableType::String);
        assert_eq!(serde_json::from_str::<VariableType>("\"integer\"").unwrap(), VariableType::Integer);
        assert_eq!(serde_json::from_str::<VariableType>("\"datetime\"").unwrap(), VariableType::Datetime);
        assert_eq!(serde_json::from_str::<VariableType>("\"any\"").unwrap(), VariableType::Any);
    }

    #[test]
    fn test_is_primitive() {
        assert!(VariableType::String.is_primitive());
        assert!(VariableType::Number.is_primitive());
        assert!(VariableType::Integer.is_primitive());
        assert!(VariableType::Boolean.is_primitive());
        assert!(VariableType::Null.is_primitive());

        assert!(!VariableType::Object.is_primitive());
        assert!(!VariableType::Array.is_primitive());
        assert!(!VariableType::Datetime.is_primitive());
    }

    #[test]
    fn test_is_complex() {
        assert!(VariableType::Object.is_complex());
        assert!(VariableType::Array.is_complex());

        assert!(!VariableType::String.is_complex());
        assert!(!VariableType::Number.is_complex());
    }

    #[test]
    fn test_is_temporal() {
        assert!(VariableType::Datetime.is_temporal());
        assert!(VariableType::Duration.is_temporal());

        assert!(!VariableType::String.is_temporal());
        assert!(!VariableType::Number.is_temporal());
    }

    #[test]
    fn test_typescript_types() {
        assert_eq!(VariableType::String.to_typescript(), "string");
        assert_eq!(VariableType::Number.to_typescript(), "number");
        assert_eq!(VariableType::Integer.to_typescript(), "number");
        assert_eq!(VariableType::Boolean.to_typescript(), "boolean");
        assert_eq!(VariableType::Object.to_typescript(), "Record<string, unknown>");
        assert_eq!(VariableType::Array.to_typescript(), "unknown[]");
        assert_eq!(VariableType::Datetime.to_typescript(), "Date");
        assert_eq!(VariableType::Duration.to_typescript(), "number");
        assert_eq!(VariableType::Null.to_typescript(), "null");
        assert_eq!(VariableType::Any.to_typescript(), "unknown");
    }

    #[test]
    fn test_default_values() {
        assert_eq!(VariableType::String.default_value_ts(), "''");
        assert_eq!(VariableType::Number.default_value_ts(), "0");
        assert_eq!(VariableType::Boolean.default_value_ts(), "false");
        assert_eq!(VariableType::Object.default_value_ts(), "{}");
        assert_eq!(VariableType::Array.default_value_ts(), "[]");
        assert_eq!(VariableType::Datetime.default_value_ts(), "new Date()");
        assert_eq!(VariableType::Null.default_value_ts(), "null");
    }

    #[test]
    fn test_is_assignable_to() {
        // Same types are compatible
        assert!(VariableType::String.is_assignable_to(&VariableType::String));
        assert!(VariableType::Number.is_assignable_to(&VariableType::Number));

        // Integer can be assigned to Number
        assert!(VariableType::Integer.is_assignable_to(&VariableType::Number));
        assert!(!VariableType::Number.is_assignable_to(&VariableType::Integer));

        // Any accepts all types
        assert!(VariableType::String.is_assignable_to(&VariableType::Any));
        assert!(VariableType::Object.is_assignable_to(&VariableType::Any));

        // Incompatible types
        assert!(!VariableType::String.is_assignable_to(&VariableType::Number));
        assert!(!VariableType::Boolean.is_assignable_to(&VariableType::String));
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", VariableType::String), "string");
        assert_eq!(format!("{}", VariableType::Datetime), "datetime");
        assert_eq!(format!("{}", VariableType::Any), "any");
    }
}
