//! Encode/Decode component schema
//!
//! The Encode/Decode component converts data between formats such as base64,
//! URL encoding, hex, and JSON/CSV representations. This is a pure (stateless,
//! side-effect-free) component -- it does NOT embed ComponentBehaviors.

use serde::{Deserialize, Serialize};
use validator::Validate;

/// Whether to encode or decode the input.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum EncodeDecodeAction {
    /// Encode the input into the target format (default).
    #[default]
    Encode,
    /// Decode the input from the target format back to its original form.
    Decode,
}

/// The format to encode/decode between.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum EncodeDecodeFormat {
    /// Base64 encoding (default).
    #[default]
    Base64,
    /// URL percent-encoding (application/x-www-form-urlencoded style).
    UrlEncoding,
    /// Hexadecimal encoding.
    Hex,
    /// Serialize a value to a JSON string.
    JsonStringify,
    /// Parse a JSON string into a structured value.
    JsonParse,
    /// Convert CSV text into a JSON array of objects.
    CsvToJson,
    /// Convert a JSON array of objects into CSV text.
    JsonToCsv,
}

/// Optional format-specific options for the encode/decode operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct FormatOptions {
    /// The column delimiter used when parsing or generating CSV.
    /// Defaults to `","` when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub csv_delimiter: Option<String>,

    /// Whether the CSV data includes a header row.
    /// Defaults to `true` when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub csv_has_headers: Option<bool>,

    /// When true, JSON output is pretty-printed with indentation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pretty_print: Option<bool>,
}

/// Encode/Decode component input.
///
/// Pure tier -- no retry, rate limit, or other I/O behaviors.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct EncodeDecodeInput {
    /// Whether to encode or decode the input data.
    #[serde(default)]
    pub action: EncodeDecodeAction,

    /// The format to use for the encode/decode operation.
    #[serde(default)]
    pub format: EncodeDecodeFormat,

    /// The input string to encode or decode.
    #[validate(length(min = 1, message = "input must not be empty"))]
    pub input: String,

    /// Optional format-specific configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<FormatOptions>,
}

impl Default for EncodeDecodeInput {
    fn default() -> Self {
        Self {
            action: EncodeDecodeAction::default(),
            format: EncodeDecodeFormat::default(),
            input: String::new(),
            options: None,
        }
    }
}

/// Encode/Decode component output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EncodeDecodeOutput {
    /// The encoded or decoded result string.
    pub result: String,
}

impl Default for EncodeDecodeOutput {
    fn default() -> Self {
        Self {
            result: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_input_with_defaults() {
        let input = EncodeDecodeInput::default();
        assert_eq!(input.action, EncodeDecodeAction::Encode);
        assert_eq!(input.format, EncodeDecodeFormat::Base64);
        assert_eq!(input.input, "");
        assert!(input.options.is_none());

        // Empty input should fail validation.
        let err = input.validate();
        assert!(err.is_err(), "empty input should fail validation");

        // Non-empty input should pass.
        let valid = EncodeDecodeInput {
            input: "hello".to_string(),
            ..EncodeDecodeInput::default()
        };
        assert!(valid.validate().is_ok());
    }

    #[test]
    fn test_full_config_deserialization() {
        let yaml = r#"
action: decode
format: hex
input: "68656c6c6f"
options:
  pretty_print: true
"#;
        let input: EncodeDecodeInput = serde_yaml::from_str(yaml).expect("deserialize");
        assert_eq!(input.action, EncodeDecodeAction::Decode);
        assert_eq!(input.format, EncodeDecodeFormat::Hex);
        assert_eq!(input.input, "68656c6c6f");
        let opts = input.options.as_ref().expect("options present");
        assert_eq!(opts.pretty_print, Some(true));
        assert!(opts.csv_delimiter.is_none());
        assert!(opts.csv_has_headers.is_none());
        assert!(input.validate().is_ok());
    }

    #[test]
    fn test_output_serialize_deserialize() {
        let output = EncodeDecodeOutput {
            result: "aGVsbG8=".to_string(),
        };
        let yaml = serde_yaml::to_string(&output).expect("serialize");
        let restored: EncodeDecodeOutput = serde_yaml::from_str(&yaml).expect("deserialize");
        assert_eq!(restored.result, "aGVsbG8=");

        // Default output has an empty result.
        let default_out = EncodeDecodeOutput::default();
        assert_eq!(default_out.result, "");
    }

    #[test]
    fn test_format_options() {
        let yaml = r#"
action: encode
format: csv_to_json
input: "name,age\nAlice,30\nBob,25"
options:
  csv_delimiter: ","
  csv_has_headers: true
  pretty_print: false
"#;
        let input: EncodeDecodeInput = serde_yaml::from_str(yaml).expect("deserialize");
        assert_eq!(input.format, EncodeDecodeFormat::CsvToJson);
        let opts = input.options.as_ref().expect("options present");
        assert_eq!(opts.csv_delimiter.as_deref(), Some(","));
        assert_eq!(opts.csv_has_headers, Some(true));
        assert_eq!(opts.pretty_print, Some(false));

        // Round-trip: fields with None values must be omitted from serialized output.
        let minimal_opts = FormatOptions {
            csv_delimiter: None,
            csv_has_headers: None,
            pretty_print: Some(true),
        };
        let serialized = serde_yaml::to_string(&minimal_opts).expect("serialize");
        assert!(!serialized.contains("csv_delimiter"));
        assert!(!serialized.contains("csv_has_headers"));
        assert!(serialized.contains("pretty_print"));
    }
}
