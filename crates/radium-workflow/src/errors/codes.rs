//! Error Codes
//!
//! Unique identifiers for each error type, following the pattern:
//! WF-{CATEGORY}{NUMBER}
//!
//! Categories:
//! - V: Validation (001-099)
//! - C: Compilation (100-199)
//! - G: Graph (200-299)
//! - T: Type (300-399)
//! - R: Runtime (400-499)
//! - F: Configuration (500-599)
//! - I: I/O (600-699)
//! - X: Internal (900-999)

use super::{ErrorCategory, ErrorSeverity};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique error codes for workflow errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCode {
    // Validation errors (V001-V099)
    /// Required field is missing
    ValidationMissingRequiredField,
    /// Field value is invalid
    ValidationInvalidFieldValue,
    /// Field value is out of range
    ValidationOutOfRange,
    /// Invalid identifier (naming convention)
    ValidationInvalidIdentifier,
    /// Duplicate identifier
    ValidationDuplicateIdentifier,
    /// Unknown reference
    ValidationUnknownReference,
    /// Circular reference detected
    ValidationCircularReference,
    /// Invalid expression syntax
    ValidationInvalidExpression,
    /// Schema validation failed
    ValidationSchemaFailed,
    /// Constraint violation
    ValidationConstraintViolation,
    /// Deprecated feature used
    ValidationDeprecatedFeature,
    /// Unused variable
    ValidationUnusedVariable,

    // Compilation errors (C100-C199)
    /// Template rendering failed
    CompilationTemplateError,
    /// Code generation failed
    CompilationCodeGenError,
    /// TypeScript syntax error
    CompilationTypescriptSyntax,
    /// Import resolution failed
    CompilationImportError,
    /// Missing dependency
    CompilationMissingDependency,
    /// Unsupported feature
    CompilationUnsupportedFeature,

    // Graph errors (G200-G299)
    /// No trigger node found
    GraphNoTrigger,
    /// Multiple trigger nodes
    GraphMultipleTriggers,
    /// Unreachable node
    GraphUnreachableNode,
    /// Orphan node (no connections)
    GraphOrphanNode,
    /// Cycle detected in graph
    GraphCycleDetected,
    /// Invalid edge connection
    GraphInvalidEdge,
    /// Missing required edge
    GraphMissingEdge,
    /// Dead end (no path to end)
    GraphDeadEnd,

    // Type errors (T300-T399)
    /// Type mismatch
    TypeMismatch,
    /// Incompatible types
    TypeIncompatible,
    /// Missing type annotation
    TypeMissingAnnotation,
    /// Invalid type conversion
    TypeInvalidConversion,
    /// Nullable type not handled
    TypeNullNotHandled,
    /// Unknown type
    TypeUnknown,

    // Runtime errors (R400-R499)
    /// Activity failed
    RuntimeActivityFailed,
    /// Workflow timeout
    RuntimeTimeout,
    /// Signal not handled
    RuntimeSignalNotHandled,
    /// Query failed
    RuntimeQueryFailed,
    /// Cancellation error
    RuntimeCancellation,
    /// Retry exhausted
    RuntimeRetryExhausted,

    // Configuration errors (F500-F599)
    /// Invalid configuration
    ConfigInvalid,
    /// Missing configuration
    ConfigMissing,
    /// Configuration conflict
    ConfigConflict,
    /// Environment variable missing
    ConfigEnvMissing,
    /// Invalid timeout value
    ConfigInvalidTimeout,

    // I/O errors (I600-I699)
    /// File not found
    IOFileNotFound,
    /// File read error
    IOReadError,
    /// File write error
    IOWriteError,
    /// Permission denied
    IOPermissionDenied,
    /// Directory error
    IODirectoryError,

    // Internal errors (X900-X999)
    /// Internal error
    InternalError,
    /// Assertion failed
    InternalAssertionFailed,
    /// Not implemented
    InternalNotImplemented,
    /// Panic recovered
    InternalPanicRecovered,
}

impl ErrorCode {
    /// Get the error category
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::ValidationMissingRequiredField
            | Self::ValidationInvalidFieldValue
            | Self::ValidationOutOfRange
            | Self::ValidationInvalidIdentifier
            | Self::ValidationDuplicateIdentifier
            | Self::ValidationUnknownReference
            | Self::ValidationCircularReference
            | Self::ValidationInvalidExpression
            | Self::ValidationSchemaFailed
            | Self::ValidationConstraintViolation
            | Self::ValidationDeprecatedFeature
            | Self::ValidationUnusedVariable => ErrorCategory::Validation,

            Self::CompilationTemplateError
            | Self::CompilationCodeGenError
            | Self::CompilationTypescriptSyntax
            | Self::CompilationImportError
            | Self::CompilationMissingDependency
            | Self::CompilationUnsupportedFeature => ErrorCategory::Compilation,

            Self::GraphNoTrigger
            | Self::GraphMultipleTriggers
            | Self::GraphUnreachableNode
            | Self::GraphOrphanNode
            | Self::GraphCycleDetected
            | Self::GraphInvalidEdge
            | Self::GraphMissingEdge
            | Self::GraphDeadEnd => ErrorCategory::Graph,

            Self::TypeMismatch
            | Self::TypeIncompatible
            | Self::TypeMissingAnnotation
            | Self::TypeInvalidConversion
            | Self::TypeNullNotHandled
            | Self::TypeUnknown => ErrorCategory::Type,

            Self::RuntimeActivityFailed
            | Self::RuntimeTimeout
            | Self::RuntimeSignalNotHandled
            | Self::RuntimeQueryFailed
            | Self::RuntimeCancellation
            | Self::RuntimeRetryExhausted => ErrorCategory::Runtime,

            Self::ConfigInvalid
            | Self::ConfigMissing
            | Self::ConfigConflict
            | Self::ConfigEnvMissing
            | Self::ConfigInvalidTimeout => ErrorCategory::Configuration,

            Self::IOFileNotFound
            | Self::IOReadError
            | Self::IOWriteError
            | Self::IOPermissionDenied
            | Self::IODirectoryError => ErrorCategory::IO,

            Self::InternalError
            | Self::InternalAssertionFailed
            | Self::InternalNotImplemented
            | Self::InternalPanicRecovered => ErrorCategory::Internal,
        }
    }

    /// Get the default severity for this error code
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            // Warnings (non-blocking)
            Self::ValidationDeprecatedFeature | Self::ValidationUnusedVariable => {
                ErrorSeverity::Warning
            }

            // Info
            // (none currently, but can be added)

            // Everything else is an error
            _ => ErrorSeverity::Error,
        }
    }

    /// Get the numeric code
    pub fn numeric_code(&self) -> u32 {
        match self {
            // Validation (001-099)
            Self::ValidationMissingRequiredField => 1,
            Self::ValidationInvalidFieldValue => 2,
            Self::ValidationOutOfRange => 3,
            Self::ValidationInvalidIdentifier => 4,
            Self::ValidationDuplicateIdentifier => 5,
            Self::ValidationUnknownReference => 6,
            Self::ValidationCircularReference => 7,
            Self::ValidationInvalidExpression => 8,
            Self::ValidationSchemaFailed => 9,
            Self::ValidationConstraintViolation => 10,
            Self::ValidationDeprecatedFeature => 11,
            Self::ValidationUnusedVariable => 12,

            // Compilation (100-199)
            Self::CompilationTemplateError => 100,
            Self::CompilationCodeGenError => 101,
            Self::CompilationTypescriptSyntax => 102,
            Self::CompilationImportError => 103,
            Self::CompilationMissingDependency => 104,
            Self::CompilationUnsupportedFeature => 105,

            // Graph (200-299)
            Self::GraphNoTrigger => 200,
            Self::GraphMultipleTriggers => 201,
            Self::GraphUnreachableNode => 202,
            Self::GraphOrphanNode => 203,
            Self::GraphCycleDetected => 204,
            Self::GraphInvalidEdge => 205,
            Self::GraphMissingEdge => 206,
            Self::GraphDeadEnd => 207,

            // Type (300-399)
            Self::TypeMismatch => 300,
            Self::TypeIncompatible => 301,
            Self::TypeMissingAnnotation => 302,
            Self::TypeInvalidConversion => 303,
            Self::TypeNullNotHandled => 304,
            Self::TypeUnknown => 305,

            // Runtime (400-499)
            Self::RuntimeActivityFailed => 400,
            Self::RuntimeTimeout => 401,
            Self::RuntimeSignalNotHandled => 402,
            Self::RuntimeQueryFailed => 403,
            Self::RuntimeCancellation => 404,
            Self::RuntimeRetryExhausted => 405,

            // Configuration (500-599)
            Self::ConfigInvalid => 500,
            Self::ConfigMissing => 501,
            Self::ConfigConflict => 502,
            Self::ConfigEnvMissing => 503,
            Self::ConfigInvalidTimeout => 504,

            // I/O (600-699)
            Self::IOFileNotFound => 600,
            Self::IOReadError => 601,
            Self::IOWriteError => 602,
            Self::IOPermissionDenied => 603,
            Self::IODirectoryError => 604,

            // Internal (900-999)
            Self::InternalError => 900,
            Self::InternalAssertionFailed => 901,
            Self::InternalNotImplemented => 902,
            Self::InternalPanicRecovered => 903,
        }
    }

    /// Get the category prefix
    fn category_prefix(&self) -> char {
        match self.category() {
            ErrorCategory::Validation => 'V',
            ErrorCategory::Compilation => 'C',
            ErrorCategory::Graph => 'G',
            ErrorCategory::Type => 'T',
            ErrorCategory::Runtime => 'R',
            ErrorCategory::Configuration => 'F',
            ErrorCategory::IO => 'I',
            ErrorCategory::Internal => 'X',
        }
    }

    /// Get the documentation URL for this error code
    pub fn docs_url(&self) -> String {
        format!(
            "https://docs.radium.dev/errors/{}",
            self.to_string().to_lowercase()
        )
    }

    /// Get default suggestions for this error code
    pub fn suggestions(&self) -> Vec<String> {
        match self {
            Self::ValidationMissingRequiredField => vec![
                "Ensure all required fields are provided in your workflow definition".to_string(),
                "Check the schema documentation for required fields".to_string(),
            ],
            Self::ValidationInvalidFieldValue => vec![
                "Check the expected format for this field".to_string(),
                "Review the allowed values in the documentation".to_string(),
            ],
            Self::ValidationOutOfRange => vec![
                "Ensure the value is within the allowed range".to_string(),
                "Check minimum and maximum constraints in the schema".to_string(),
            ],
            Self::ValidationInvalidIdentifier => vec![
                "Use only alphanumeric characters and underscores".to_string(),
                "Start identifiers with a letter, not a number".to_string(),
            ],
            Self::ValidationDuplicateIdentifier => vec![
                "Use unique IDs for each node".to_string(),
                "Rename one of the duplicate identifiers".to_string(),
            ],
            Self::ValidationUnknownReference => vec![
                "Check that the referenced node or variable exists".to_string(),
                "Verify spelling of the reference".to_string(),
            ],
            Self::ValidationCircularReference => vec![
                "Break the circular dependency by restructuring the workflow".to_string(),
                "Consider using signals or queries for communication".to_string(),
            ],
            Self::ValidationInvalidExpression => vec![
                "Check expression syntax".to_string(),
                "Ensure all variables in the expression are defined".to_string(),
            ],
            Self::ValidationSchemaFailed => vec![
                "Validate your workflow against the JSON schema".to_string(),
                "Check for missing or invalid properties".to_string(),
            ],
            Self::ValidationConstraintViolation => vec![
                "Review the constraints for this field".to_string(),
                "Ensure the value meets all requirements".to_string(),
            ],
            Self::ValidationDeprecatedFeature => vec![
                "Update to use the recommended alternative".to_string(),
                "Check the migration guide for deprecated features".to_string(),
            ],
            Self::ValidationUnusedVariable => vec![
                "Remove the unused variable".to_string(),
                "Use the variable in your workflow logic".to_string(),
            ],

            Self::CompilationTemplateError => vec![
                "Check template syntax".to_string(),
                "Ensure all template variables are defined".to_string(),
            ],
            Self::CompilationCodeGenError => vec![
                "Review the workflow structure".to_string(),
                "Check for unsupported combinations of features".to_string(),
            ],
            Self::CompilationTypescriptSyntax => vec![
                "Check generated TypeScript for syntax errors".to_string(),
                "Ensure custom code follows TypeScript syntax".to_string(),
            ],
            Self::CompilationImportError => vec![
                "Verify all imports are available".to_string(),
                "Check that dependencies are installed".to_string(),
            ],
            Self::CompilationMissingDependency => vec![
                "Install the missing dependency".to_string(),
                "Check package.json for required packages".to_string(),
            ],
            Self::CompilationUnsupportedFeature => vec![
                "Use a supported alternative".to_string(),
                "Check the documentation for supported features".to_string(),
            ],

            Self::GraphNoTrigger => vec![
                "Add a trigger node to start the workflow".to_string(),
                "Ensure exactly one trigger node exists".to_string(),
            ],
            Self::GraphMultipleTriggers => vec![
                "Remove extra trigger nodes".to_string(),
                "Use conditional logic within a single trigger".to_string(),
            ],
            Self::GraphUnreachableNode => vec![
                "Connect the node to the workflow graph".to_string(),
                "Remove the unreachable node if unused".to_string(),
            ],
            Self::GraphOrphanNode => vec![
                "Connect the node to other nodes".to_string(),
                "Remove the orphan node if unused".to_string(),
            ],
            Self::GraphCycleDetected => vec![
                "Use loop nodes for iteration instead of cycles".to_string(),
                "Break the cycle by restructuring the workflow".to_string(),
            ],
            Self::GraphInvalidEdge => vec![
                "Check that source and target nodes exist".to_string(),
                "Verify edge compatibility between node types".to_string(),
            ],
            Self::GraphMissingEdge => vec![
                "Add the missing connection between nodes".to_string(),
                "Ensure all nodes are properly connected".to_string(),
            ],
            Self::GraphDeadEnd => vec![
                "Connect the node to an end node or another path".to_string(),
                "Add an end node to complete the workflow".to_string(),
            ],

            Self::TypeMismatch => vec![
                "Check that types match at both ends of the connection".to_string(),
                "Add type conversion if needed".to_string(),
            ],
            Self::TypeIncompatible => vec![
                "Use compatible types for this operation".to_string(),
                "Convert the value to the expected type".to_string(),
            ],
            Self::TypeMissingAnnotation => vec![
                "Add explicit type annotation".to_string(),
                "Let the type be inferred from usage".to_string(),
            ],
            Self::TypeInvalidConversion => vec![
                "Use a valid conversion method".to_string(),
                "Check if the conversion is supported".to_string(),
            ],
            Self::TypeNullNotHandled => vec![
                "Add null check before using the value".to_string(),
                "Use optional chaining or default values".to_string(),
            ],
            Self::TypeUnknown => vec![
                "Define the type explicitly".to_string(),
                "Check if the variable is properly initialized".to_string(),
            ],

            Self::RuntimeActivityFailed => vec![
                "Check activity implementation for errors".to_string(),
                "Review activity logs for details".to_string(),
                "Consider adding retry logic".to_string(),
            ],
            Self::RuntimeTimeout => vec![
                "Increase timeout if appropriate".to_string(),
                "Optimize the operation to complete faster".to_string(),
                "Consider breaking into smaller operations".to_string(),
            ],
            Self::RuntimeSignalNotHandled => vec![
                "Add a handler for this signal".to_string(),
                "Check signal name spelling".to_string(),
            ],
            Self::RuntimeQueryFailed => vec![
                "Check query handler implementation".to_string(),
                "Verify query parameters are valid".to_string(),
            ],
            Self::RuntimeCancellation => vec![
                "Handle cancellation gracefully".to_string(),
                "Add cleanup logic in cancellation handlers".to_string(),
            ],
            Self::RuntimeRetryExhausted => vec![
                "Review the operation that failed".to_string(),
                "Consider increasing retry limits".to_string(),
                "Investigate the root cause of failures".to_string(),
            ],

            Self::ConfigInvalid => vec![
                "Review configuration format".to_string(),
                "Check documentation for valid options".to_string(),
            ],
            Self::ConfigMissing => vec![
                "Provide the required configuration".to_string(),
                "Check if configuration file exists".to_string(),
            ],
            Self::ConfigConflict => vec![
                "Remove conflicting configuration options".to_string(),
                "Choose one of the conflicting options".to_string(),
            ],
            Self::ConfigEnvMissing => vec![
                "Set the required environment variable".to_string(),
                "Check .env file for the variable".to_string(),
            ],
            Self::ConfigInvalidTimeout => vec![
                "Use a valid timeout format (e.g., '30s', '5m')".to_string(),
                "Ensure timeout is positive".to_string(),
            ],

            Self::IOFileNotFound => vec![
                "Check the file path".to_string(),
                "Ensure the file exists".to_string(),
            ],
            Self::IOReadError => vec![
                "Check file permissions".to_string(),
                "Ensure file is not locked by another process".to_string(),
            ],
            Self::IOWriteError => vec![
                "Check directory permissions".to_string(),
                "Ensure disk has sufficient space".to_string(),
            ],
            Self::IOPermissionDenied => vec![
                "Check file/directory permissions".to_string(),
                "Run with appropriate privileges".to_string(),
            ],
            Self::IODirectoryError => vec![
                "Ensure directory exists".to_string(),
                "Check directory permissions".to_string(),
            ],

            Self::InternalError => vec![
                "Report this issue with error details".to_string(),
                "Try again or restart the application".to_string(),
            ],
            Self::InternalAssertionFailed => vec![
                "Report this bug with reproduction steps".to_string(),
            ],
            Self::InternalNotImplemented => vec![
                "This feature is not yet implemented".to_string(),
                "Check roadmap for availability".to_string(),
            ],
            Self::InternalPanicRecovered => vec![
                "Report this issue with stack trace".to_string(),
            ],
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WF-{}{:03}", self.category_prefix(), self.numeric_code())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_format() {
        assert_eq!(
            ErrorCode::ValidationMissingRequiredField.to_string(),
            "WF-V001"
        );
        assert_eq!(ErrorCode::CompilationTemplateError.to_string(), "WF-C100");
        assert_eq!(ErrorCode::GraphNoTrigger.to_string(), "WF-G200");
        assert_eq!(ErrorCode::TypeMismatch.to_string(), "WF-T300");
        assert_eq!(ErrorCode::RuntimeActivityFailed.to_string(), "WF-R400");
        assert_eq!(ErrorCode::ConfigInvalid.to_string(), "WF-F500");
        assert_eq!(ErrorCode::IOFileNotFound.to_string(), "WF-I600");
        assert_eq!(ErrorCode::InternalError.to_string(), "WF-X900");
    }

    #[test]
    fn test_error_code_category() {
        assert_eq!(
            ErrorCode::ValidationMissingRequiredField.category(),
            ErrorCategory::Validation
        );
        assert_eq!(
            ErrorCode::CompilationTemplateError.category(),
            ErrorCategory::Compilation
        );
        assert_eq!(ErrorCode::GraphNoTrigger.category(), ErrorCategory::Graph);
        assert_eq!(ErrorCode::TypeMismatch.category(), ErrorCategory::Type);
    }

    #[test]
    fn test_error_code_severity() {
        assert_eq!(
            ErrorCode::ValidationMissingRequiredField.severity(),
            ErrorSeverity::Error
        );
        assert_eq!(
            ErrorCode::ValidationDeprecatedFeature.severity(),
            ErrorSeverity::Warning
        );
        assert_eq!(
            ErrorCode::ValidationUnusedVariable.severity(),
            ErrorSeverity::Warning
        );
    }

    #[test]
    fn test_error_code_suggestions() {
        let suggestions = ErrorCode::ValidationMissingRequiredField.suggestions();
        assert!(!suggestions.is_empty());
        assert!(suggestions[0].contains("required"));
    }

    #[test]
    fn test_all_codes_have_suggestions() {
        // Test a sample of codes from each category
        let codes = [
            ErrorCode::ValidationMissingRequiredField,
            ErrorCode::CompilationTemplateError,
            ErrorCode::GraphNoTrigger,
            ErrorCode::TypeMismatch,
            ErrorCode::RuntimeActivityFailed,
            ErrorCode::ConfigInvalid,
            ErrorCode::IOFileNotFound,
            ErrorCode::InternalError,
        ];

        for code in codes {
            let suggestions = code.suggestions();
            assert!(
                !suggestions.is_empty(),
                "Code {:?} should have suggestions",
                code
            );
        }
    }
}
