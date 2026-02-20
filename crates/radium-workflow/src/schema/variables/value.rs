//! Variable value definitions
//!
//! Defines runtime values for workflow variables with type information.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use super::VariableType;

/// Runtime value for a workflow variable
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum VariableValue {
    /// String value
    String(String),
    /// Floating-point number
    Number(f64),
    /// Integer value
    Integer(i64),
    /// Boolean value
    Boolean(bool),
    /// JSON object
    Object(JsonValue),
    /// Array of values
    Array(Vec<JsonValue>),
    /// Datetime value
    Datetime(DateTime<Utc>),
    /// Duration in milliseconds
    Duration(i64),
    /// Null value
    Null,
}

impl VariableValue {
    /// Get the type of this value
    pub fn type_of(&self) -> VariableType {
        match self {
            Self::String(_) => VariableType::String,
            Self::Number(_) => VariableType::Number,
            Self::Integer(_) => VariableType::Integer,
            Self::Boolean(_) => VariableType::Boolean,
            Self::Object(_) => VariableType::Object,
            Self::Array(_) => VariableType::Array,
            Self::Datetime(_) => VariableType::Datetime,
            Self::Duration(_) => VariableType::Duration,
            Self::Null => VariableType::Null,
        }
    }

    /// Check if this value is compatible with the expected type
    pub fn is_compatible_with(&self, expected: &VariableType) -> bool {
        self.type_of().is_assignable_to(expected)
    }

    /// Convert to a JSON value
    pub fn to_json(&self) -> JsonValue {
        match self {
            Self::String(s) => JsonValue::String(s.clone()),
            Self::Number(n) => serde_json::json!(n),
            Self::Integer(i) => serde_json::json!(i),
            Self::Boolean(b) => JsonValue::Bool(*b),
            Self::Object(v) => v.clone(),
            Self::Array(a) => JsonValue::Array(a.clone()),
            Self::Datetime(dt) => JsonValue::String(dt.to_rfc3339()),
            Self::Duration(ms) => serde_json::json!(ms),
            Self::Null => JsonValue::Null,
        }
    }

    /// Create from a JSON value with type hint
    pub fn from_json_with_type(value: &JsonValue, type_hint: &VariableType) -> Option<Self> {
        match (value, type_hint) {
            (JsonValue::String(s), VariableType::String) => Some(Self::String(s.clone())),
            (JsonValue::String(s), VariableType::Datetime) => {
                DateTime::parse_from_rfc3339(s)
                    .ok()
                    .map(|dt| Self::Datetime(dt.with_timezone(&Utc)))
            }
            (JsonValue::Number(n), VariableType::Number) => {
                n.as_f64().map(Self::Number)
            }
            (JsonValue::Number(n), VariableType::Integer) => {
                n.as_i64().map(Self::Integer)
            }
            (JsonValue::Number(n), VariableType::Duration) => {
                n.as_i64().map(Self::Duration)
            }
            (JsonValue::Bool(b), VariableType::Boolean) => Some(Self::Boolean(*b)),
            (JsonValue::Array(a), VariableType::Array) => Some(Self::Array(a.clone())),
            (JsonValue::Object(_), VariableType::Object) => Some(Self::Object(value.clone())),
            (JsonValue::Null, VariableType::Null) => Some(Self::Null),
            (v, VariableType::Any) => Self::from_json(v),
            _ => None,
        }
    }

    /// Create from a JSON value, inferring the type
    pub fn from_json(value: &JsonValue) -> Option<Self> {
        match value {
            JsonValue::String(s) => Some(Self::String(s.clone())),
            JsonValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Some(Self::Integer(i))
                } else {
                    n.as_f64().map(Self::Number)
                }
            }
            JsonValue::Bool(b) => Some(Self::Boolean(*b)),
            JsonValue::Array(a) => Some(Self::Array(a.clone())),
            JsonValue::Object(_) => Some(Self::Object(value.clone())),
            JsonValue::Null => Some(Self::Null),
        }
    }

    /// Check if this value is null
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Try to coerce this value to a string
    pub fn as_string(&self) -> Option<String> {
        match self {
            Self::String(s) => Some(s.clone()),
            Self::Number(n) => Some(n.to_string()),
            Self::Integer(i) => Some(i.to_string()),
            Self::Boolean(b) => Some(b.to_string()),
            Self::Null => None,
            _ => Some(self.to_json().to_string()),
        }
    }

    /// Try to coerce this value to a number
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(*n),
            Self::Integer(i) => Some(*i as f64),
            Self::String(s) => s.parse().ok(),
            Self::Duration(ms) => Some(*ms as f64),
            _ => None,
        }
    }

    /// Try to coerce this value to a boolean
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(*b),
            Self::String(s) => match s.to_lowercase().as_str() {
                "true" | "1" | "yes" => Some(true),
                "false" | "0" | "no" => Some(false),
                _ => None,
            },
            Self::Integer(i) => Some(*i != 0),
            Self::Number(n) => Some(*n != 0.0),
            Self::Null => Some(false),
            _ => None,
        }
    }
}

impl Default for VariableValue {
    fn default() -> Self {
        Self::Null
    }
}

impl std::fmt::Display for VariableValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => write!(f, "\"{}\"", s),
            Self::Number(n) => write!(f, "{}", n),
            Self::Integer(i) => write!(f, "{}", i),
            Self::Boolean(b) => write!(f, "{}", b),
            Self::Object(v) => write!(f, "{}", v),
            Self::Array(a) => write!(f, "{:?}", a),
            Self::Datetime(dt) => write!(f, "{}", dt.to_rfc3339()),
            Self::Duration(ms) => write!(f, "{}ms", ms),
            Self::Null => write!(f, "null"),
        }
    }
}

// Convenience From implementations
impl From<String> for VariableValue {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<&str> for VariableValue {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<f64> for VariableValue {
    fn from(n: f64) -> Self {
        Self::Number(n)
    }
}

impl From<i64> for VariableValue {
    fn from(i: i64) -> Self {
        Self::Integer(i)
    }
}

impl From<i32> for VariableValue {
    fn from(i: i32) -> Self {
        Self::Integer(i64::from(i))
    }
}

impl From<bool> for VariableValue {
    fn from(b: bool) -> Self {
        Self::Boolean(b)
    }
}

impl From<DateTime<Utc>> for VariableValue {
    fn from(dt: DateTime<Utc>) -> Self {
        Self::Datetime(dt)
    }
}

impl From<Duration> for VariableValue {
    fn from(d: Duration) -> Self {
        Self::Duration(d.num_milliseconds())
    }
}

impl From<Vec<JsonValue>> for VariableValue {
    fn from(a: Vec<JsonValue>) -> Self {
        Self::Array(a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_of() {
        assert_eq!(VariableValue::String("test".to_string()).type_of(), VariableType::String);
        assert_eq!(VariableValue::Number(3.14).type_of(), VariableType::Number);
        assert_eq!(VariableValue::Integer(42).type_of(), VariableType::Integer);
        assert_eq!(VariableValue::Boolean(true).type_of(), VariableType::Boolean);
        assert_eq!(VariableValue::Null.type_of(), VariableType::Null);
    }

    #[test]
    fn test_is_compatible_with() {
        let int_value = VariableValue::Integer(42);

        assert!(int_value.is_compatible_with(&VariableType::Integer));
        assert!(int_value.is_compatible_with(&VariableType::Number)); // int -> number ok
        assert!(!int_value.is_compatible_with(&VariableType::String));
        assert!(int_value.is_compatible_with(&VariableType::Any)); // any accepts all
    }

    #[test]
    fn test_to_json() {
        assert_eq!(VariableValue::String("test".to_string()).to_json(), JsonValue::String("test".to_string()));
        assert_eq!(VariableValue::Integer(42).to_json(), serde_json::json!(42));
        assert_eq!(VariableValue::Boolean(true).to_json(), JsonValue::Bool(true));
        assert_eq!(VariableValue::Null.to_json(), JsonValue::Null);
    }

    #[test]
    fn test_from_json_with_type() {
        let string_val = VariableValue::from_json_with_type(
            &JsonValue::String("test".to_string()),
            &VariableType::String,
        );
        assert_eq!(string_val, Some(VariableValue::String("test".to_string())));

        let int_val = VariableValue::from_json_with_type(
            &serde_json::json!(42),
            &VariableType::Integer,
        );
        assert_eq!(int_val, Some(VariableValue::Integer(42)));

        // Type mismatch
        let mismatch = VariableValue::from_json_with_type(
            &JsonValue::String("not a number".to_string()),
            &VariableType::Number,
        );
        assert_eq!(mismatch, None);
    }

    #[test]
    fn test_as_string_coercion() {
        assert_eq!(VariableValue::String("test".to_string()).as_string(), Some("test".to_string()));
        assert_eq!(VariableValue::Integer(42).as_string(), Some("42".to_string()));
        assert_eq!(VariableValue::Number(3.14).as_string(), Some("3.14".to_string()));
        assert_eq!(VariableValue::Boolean(true).as_string(), Some("true".to_string()));
        assert_eq!(VariableValue::Null.as_string(), None);
    }

    #[test]
    fn test_as_number_coercion() {
        assert_eq!(VariableValue::Number(3.14).as_number(), Some(3.14));
        assert_eq!(VariableValue::Integer(42).as_number(), Some(42.0));
        assert_eq!(VariableValue::String("3.14".to_string()).as_number(), Some(3.14));
        assert_eq!(VariableValue::String("invalid".to_string()).as_number(), None);
    }

    #[test]
    fn test_as_boolean_coercion() {
        assert_eq!(VariableValue::Boolean(true).as_boolean(), Some(true));
        assert_eq!(VariableValue::String("true".to_string()).as_boolean(), Some(true));
        assert_eq!(VariableValue::String("false".to_string()).as_boolean(), Some(false));
        assert_eq!(VariableValue::Integer(0).as_boolean(), Some(false));
        assert_eq!(VariableValue::Integer(1).as_boolean(), Some(true));
        assert_eq!(VariableValue::Null.as_boolean(), Some(false));
    }

    #[test]
    fn test_from_implementations() {
        assert_eq!(VariableValue::from("test"), VariableValue::String("test".to_string()));
        assert_eq!(VariableValue::from(42i64), VariableValue::Integer(42));
        assert_eq!(VariableValue::from(3.14f64), VariableValue::Number(3.14));
        assert_eq!(VariableValue::from(true), VariableValue::Boolean(true));
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", VariableValue::String("test".to_string())), "\"test\"");
        assert_eq!(format!("{}", VariableValue::Integer(42)), "42");
        assert_eq!(format!("{}", VariableValue::Boolean(true)), "true");
        assert_eq!(format!("{}", VariableValue::Null), "null");
    }
}
