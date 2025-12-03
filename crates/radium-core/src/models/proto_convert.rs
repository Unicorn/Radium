//! Proto conversion utilities for Radium Core models.
//!
//! This module provides common utility functions for converting between
//! proto message types and Rust model types, eliminating code duplication
//! across Agent, Task, and Workflow conversion implementations.

use chrono::{DateTime, Utc};
use serde::{Serialize, de::DeserializeOwned};

/// Parse an RFC3339 timestamp string into a `DateTime<Utc>`.
///
/// # Arguments
/// * `s` - The RFC3339 formatted timestamp string
/// * `field_name` - The name of the field being parsed (for error messages)
///
/// # Returns
/// `Ok(DateTime<Utc>)` if parsing succeeds, or an error string describing the failure.
///
/// # Errors
/// Returns an error string if the timestamp cannot be parsed from RFC3339 format.
pub fn parse_rfc3339_timestamp(s: &str, field_name: &str) -> Result<DateTime<Utc>, String> {
    Ok(DateTime::parse_from_rfc3339(s)
        .map_err(|e| format!("Failed to parse {}: {}", field_name, e))?
        .with_timezone(&Utc))
}

/// Format a `DateTime<Utc>` as an RFC3339 timestamp string.
///
/// # Arguments
/// * `dt` - The datetime to format
///
/// # Returns
/// An RFC3339 formatted timestamp string.
pub fn format_rfc3339_timestamp(dt: &DateTime<Utc>) -> String {
    dt.to_rfc3339()
}

/// Serialize a value to a JSON string with a fallback default.
///
/// # Arguments
/// * `value` - The value to serialize (must implement `Serialize`)
/// * `default` - The default string to use if serialization fails
///
/// # Returns
/// A JSON string representation of the value, or the default if serialization fails.
pub fn json_to_string<T: Serialize>(value: &T, default: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| default.to_string())
}

/// Deserialize a JSON string into a Rust type.
///
/// # Arguments
/// * `s` - The JSON string to deserialize
///
/// # Returns
/// `Ok(T)` if deserialization succeeds, or a `serde_json::Error` if it fails.
///
/// # Errors
/// Returns a `serde_json::Error` if the string is not valid JSON or cannot be
/// deserialized into the target type.
pub fn json_from_str<T: DeserializeOwned>(s: &str) -> Result<T, serde_json::Error> {
    serde_json::from_str(s)
}

/// Deserialize an optional JSON field from a string.
///
/// If the string is empty, returns `None`. Otherwise, attempts to deserialize
/// the JSON string into the target type.
///
/// # Arguments
/// * `s` - The JSON string to deserialize (may be empty)
///
/// # Returns
/// `Ok(None)` if the string is empty, `Ok(Some(T))` if deserialization succeeds,
/// or an error if the string is not empty but deserialization fails.
///
/// # Errors
/// Returns a `serde_json::Error` if the string is not empty but is not valid JSON
/// or cannot be deserialized into the target type.
pub fn optional_json_from_str<T: DeserializeOwned>(
    s: &str,
) -> Result<Option<T>, serde_json::Error> {
    if s.is_empty() { Ok(None) } else { serde_json::from_str(s).map(Some) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;
    use serde::{Deserialize, Serialize};

    #[test]
    fn test_parse_rfc3339_timestamp_valid() {
        let timestamp_str = "2023-12-01T10:30:00Z";
        let result = parse_rfc3339_timestamp(timestamp_str, "test_field");
        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(dt.year(), 2023);
        assert_eq!(dt.month(), 12);
        assert_eq!(dt.day(), 1);
    }

    #[test]
    fn test_parse_rfc3339_timestamp_with_timezone() {
        let timestamp_str = "2023-12-01T10:30:00+05:00";
        let result = parse_rfc3339_timestamp(timestamp_str, "test_field");
        assert!(result.is_ok());
        let dt = result.unwrap();
        // Should convert to UTC
        assert_eq!(dt.timezone(), Utc);
    }

    #[test]
    fn test_parse_rfc3339_timestamp_invalid() {
        let timestamp_str = "invalid-date";
        let result = parse_rfc3339_timestamp(timestamp_str, "test_field");
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("Failed to parse test_field"));
    }

    #[test]
    fn test_format_rfc3339_timestamp() {
        let dt = Utc::now();
        let formatted = format_rfc3339_timestamp(&dt);
        // Should be valid RFC3339
        let parsed = parse_rfc3339_timestamp(&formatted, "formatted");
        assert!(parsed.is_ok());
        // Should round-trip (within 1 second due to formatting precision)
        let parsed_dt = parsed.unwrap();
        let diff = (parsed_dt - dt).num_seconds().abs();
        assert!(diff <= 1);
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestStruct {
        field1: String,
        field2: i32,
    }

    #[test]
    fn test_json_to_string_valid() {
        let value = TestStruct { field1: "test".to_string(), field2: 42 };
        let json = json_to_string(&value, "{}");
        assert!(json.contains("test"));
        assert!(json.contains("42"));
    }

    #[test]
    fn test_json_to_string_fallback() {
        // This test is hard to trigger since serde_json::to_string rarely fails
        // But we can verify the function signature works
        let value = TestStruct { field1: "test".to_string(), field2: 42 };
        let json = json_to_string(&value, "fallback");
        assert!(!json.is_empty());
        assert_ne!(json, "fallback"); // Should succeed, so not fallback
    }

    #[test]
    fn test_json_from_str_valid() {
        let json = r#"{"field1":"test","field2":42}"#;
        let result: Result<TestStruct, _> = json_from_str(json);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value.field1, "test");
        assert_eq!(value.field2, 42);
    }

    #[test]
    fn test_json_from_str_invalid() {
        let json = "invalid json";
        let result: Result<TestStruct, _> = json_from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_json_from_str_missing_fields() {
        let json = r#"{"field1":"test"}"#;
        let result: Result<TestStruct, _> = json_from_str(json);
        // Should fail because field2 is missing
        assert!(result.is_err());
    }

    #[test]
    fn test_optional_json_from_str_empty() {
        let result: Result<Option<TestStruct>, _> = optional_json_from_str("");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_optional_json_from_str_valid() {
        let json = r#"{"field1":"test","field2":42}"#;
        let result: Result<Option<TestStruct>, _> = optional_json_from_str(json);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(value.is_some());
        let value = value.unwrap();
        assert_eq!(value.field1, "test");
        assert_eq!(value.field2, 42);
    }

    #[test]
    fn test_optional_json_from_str_invalid() {
        let json = "invalid json";
        let result: Result<Option<TestStruct>, _> = optional_json_from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_timestamp_round_trip() {
        let original = Utc::now();
        let formatted = format_rfc3339_timestamp(&original);
        let parsed = parse_rfc3339_timestamp(&formatted, "round_trip").unwrap();
        // Should be very close (within 1 second)
        let diff = (parsed - original).num_seconds().abs();
        assert!(diff <= 1);
    }
}
