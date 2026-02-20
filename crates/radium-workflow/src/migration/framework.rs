//! Migration framework traits and types
//!
//! Defines the core trait for component migrations and related analysis types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use super::record::MigrationRecord;
use super::verification::VerificationResult;

/// Trait for component migration implementations
pub trait ComponentMigration {
    /// Component type identifier (e.g., "trigger", "activity", "loop")
    fn component_type(&self) -> &str;

    /// Original TypeScript file path (if applicable)
    fn typescript_source(&self) -> Option<PathBuf>;

    /// Analyze the existing TypeScript component
    fn analyze(&self) -> Result<ComponentAnalysis, MigrationError>;

    /// Generate Rust schema from analysis
    fn generate_rust_schema(&self, analysis: &ComponentAnalysis) -> Result<String, MigrationError>;

    /// Generate TypeScript from Rust schema
    fn generate_typescript(&self) -> Result<String, MigrationError>;

    /// Verify generated code matches original behavior
    fn verify(&self) -> Result<VerificationResult, MigrationError>;

    /// Create migration record
    fn create_record(&self) -> MigrationRecord;
}

/// Analysis result from examining an existing TypeScript component
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentAnalysis {
    /// Component type identifier
    pub component_type: String,

    /// Original TypeScript source path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typescript_source: Option<PathBuf>,

    /// Input schema analysis
    pub input_schema: SchemaAnalysis,

    /// Output schema analysis
    pub output_schema: SchemaAnalysis,

    /// Config schema analysis (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_schema: Option<SchemaAnalysis>,

    /// Dependencies on other components/modules
    #[serde(default)]
    pub dependencies: Vec<String>,

    /// External calls made by this component
    #[serde(default)]
    pub external_calls: Vec<ExternalCall>,

    /// Error handling patterns found
    #[serde(default)]
    pub error_patterns: Vec<ErrorPattern>,

    /// Validation rules found
    #[serde(default)]
    pub validation_rules: Vec<ValidationRule>,
}

impl ComponentAnalysis {
    /// Create a new component analysis
    pub fn new(component_type: impl Into<String>) -> Self {
        Self {
            component_type: component_type.into(),
            typescript_source: None,
            input_schema: SchemaAnalysis::default(),
            output_schema: SchemaAnalysis::default(),
            config_schema: None,
            dependencies: Vec::new(),
            external_calls: Vec::new(),
            error_patterns: Vec::new(),
            validation_rules: Vec::new(),
        }
    }

    /// Set TypeScript source path
    pub fn with_source(mut self, path: PathBuf) -> Self {
        self.typescript_source = Some(path);
        self
    }

    /// Add a dependency
    pub fn add_dependency(&mut self, dep: impl Into<String>) {
        self.dependencies.push(dep.into());
    }

    /// Add an external call
    pub fn add_external_call(&mut self, call: ExternalCall) {
        self.external_calls.push(call);
    }

    /// Add a validation rule
    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }
}

/// Schema analysis for input/output/config
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SchemaAnalysis {
    /// Fields in the schema
    #[serde(default)]
    pub fields: Vec<FieldAnalysis>,

    /// Required field names
    #[serde(default)]
    pub required_fields: Vec<String>,

    /// Optional field names
    #[serde(default)]
    pub optional_fields: Vec<String>,

    /// Default values for fields
    #[serde(default)]
    pub default_values: HashMap<String, String>,
}

impl SchemaAnalysis {
    /// Create a new schema analysis
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a field to the analysis
    pub fn add_field(&mut self, field: FieldAnalysis) {
        if field.is_optional {
            self.optional_fields.push(field.name.clone());
        } else {
            self.required_fields.push(field.name.clone());
        }
        if let Some(default) = &field.default_value {
            self.default_values
                .insert(field.name.clone(), default.clone());
        }
        self.fields.push(field);
    }
}

/// Analysis of a single field
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldAnalysis {
    /// Field name
    pub name: String,

    /// TypeScript type
    pub typescript_type: String,

    /// Equivalent Rust type
    pub rust_type: String,

    /// Whether the field is optional
    #[serde(default)]
    pub is_optional: bool,

    /// Default value (as code string)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,

    /// Validation rules for this field
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation: Option<String>,

    /// Description/documentation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl FieldAnalysis {
    /// Create a new required field
    pub fn required(
        name: impl Into<String>,
        typescript_type: impl Into<String>,
        rust_type: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            typescript_type: typescript_type.into(),
            rust_type: rust_type.into(),
            is_optional: false,
            default_value: None,
            validation: None,
            description: None,
        }
    }

    /// Create a new optional field
    pub fn optional(
        name: impl Into<String>,
        typescript_type: impl Into<String>,
        rust_type: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            typescript_type: typescript_type.into(),
            rust_type: rust_type.into(),
            is_optional: true,
            default_value: None,
            validation: None,
            description: None,
        }
    }

    /// Set default value
    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default_value = Some(default.into());
        self
    }

    /// Set validation
    pub fn with_validation(mut self, validation: impl Into<String>) -> Self {
        self.validation = Some(validation.into());
        self
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// External call made by a component
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalCall {
    /// Target service/endpoint
    pub target: String,

    /// HTTP method or RPC method
    pub method: String,

    /// How input is mapped
    pub input_mapping: String,

    /// How output is mapped
    pub output_mapping: String,
}

impl ExternalCall {
    /// Create a new external call
    pub fn new(
        target: impl Into<String>,
        method: impl Into<String>,
        input_mapping: impl Into<String>,
        output_mapping: impl Into<String>,
    ) -> Self {
        Self {
            target: target.into(),
            method: method.into(),
            input_mapping: input_mapping.into(),
            output_mapping: output_mapping.into(),
        }
    }
}

/// Error handling pattern found in component
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorPattern {
    /// Error type/code
    pub error_type: String,

    /// How it's handled
    pub handling: String,

    /// Whether the error is retryable
    #[serde(default)]
    pub retryable: bool,
}

impl ErrorPattern {
    /// Create a new error pattern
    pub fn new(
        error_type: impl Into<String>,
        handling: impl Into<String>,
        retryable: bool,
    ) -> Self {
        Self {
            error_type: error_type.into(),
            handling: handling.into(),
            retryable,
        }
    }
}

/// Validation rule found in component
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationRule {
    /// Field being validated
    pub field: String,

    /// Validation rule/constraint
    pub rule: String,

    /// Error message when validation fails
    pub message: String,
}

impl ValidationRule {
    /// Create a new validation rule
    pub fn new(
        field: impl Into<String>,
        rule: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            field: field.into(),
            rule: rule.into(),
            message: message.into(),
        }
    }
}

/// Migration errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum MigrationError {
    /// Analysis failed
    #[error("Analysis failed: {0}")]
    AnalysisFailed(String),

    /// Schema generation failed
    #[error("Schema generation failed: {0}")]
    SchemaGenerationFailed(String),

    /// Code generation failed
    #[error("Code generation failed: {0}")]
    CodeGenerationFailed(String),

    /// Verification failed
    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(String),

    /// Component not found
    #[error("Component not found: {0}")]
    ComponentNotFound(String),
}

impl From<std::io::Error> for MigrationError {
    fn from(err: std::io::Error) -> Self {
        MigrationError::IoError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_analysis_builder() {
        let mut analysis = ComponentAnalysis::new("trigger");
        analysis.add_dependency("temporal");
        analysis.add_validation_rule(ValidationRule::new(
            "trigger_type",
            "required",
            "Trigger type is required",
        ));

        assert_eq!(analysis.component_type, "trigger");
        assert_eq!(analysis.dependencies.len(), 1);
        assert_eq!(analysis.validation_rules.len(), 1);
    }

    #[test]
    fn test_schema_analysis() {
        let mut schema = SchemaAnalysis::new();
        schema.add_field(FieldAnalysis::required("name", "string", "String"));
        schema.add_field(
            FieldAnalysis::optional("count", "number", "i32").with_default("0"),
        );

        assert_eq!(schema.fields.len(), 2);
        assert_eq!(schema.required_fields, vec!["name"]);
        assert_eq!(schema.optional_fields, vec!["count"]);
        assert!(schema.default_values.contains_key("count"));
    }

    #[test]
    fn test_field_analysis_builder() {
        let field = FieldAnalysis::required("url", "string", "String")
            .with_validation("url")
            .with_description("The URL to request");

        assert_eq!(field.name, "url");
        assert!(!field.is_optional);
        assert!(field.validation.is_some());
        assert!(field.description.is_some());
    }

    #[test]
    fn test_external_call() {
        let call = ExternalCall::new(
            "kong-api",
            "POST",
            "params -> body",
            "response.data -> result",
        );

        assert_eq!(call.target, "kong-api");
        assert_eq!(call.method, "POST");
    }

    #[test]
    fn test_migration_error_display() {
        let err = MigrationError::AnalysisFailed("missing field".to_string());
        assert!(err.to_string().contains("Analysis failed"));
    }
}
