//! Input Sanitization Module
//!
//! Provides input validation and sanitization for workflow definitions
//! to prevent injection attacks and ensure data integrity.
#![allow(dead_code)]

use std::collections::HashSet;

/// Result of a sanitization operation
#[derive(Debug, Clone)]
pub struct SanitizationResult {
    /// The sanitized value
    pub value: String,
    /// Whether any modifications were made
    pub was_modified: bool,
    /// Description of modifications made
    pub modifications: Vec<String>,
}

impl SanitizationResult {
    /// Create a result with no modifications
    pub fn unchanged(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            was_modified: false,
            modifications: Vec::new(),
        }
    }

    /// Create a result with modifications
    pub fn modified(value: impl Into<String>, modifications: Vec<String>) -> Self {
        Self {
            value: value.into(),
            was_modified: true,
            modifications,
        }
    }
}

/// Sanitization configuration
#[derive(Debug, Clone)]
pub struct SanitizerConfig {
    /// Maximum string length allowed
    pub max_string_length: usize,
    /// Maximum identifier length
    pub max_identifier_length: usize,
    /// Maximum expression length
    pub max_expression_length: usize,
    /// Whether to strip HTML tags
    pub strip_html: bool,
    /// Whether to normalize unicode
    pub normalize_unicode: bool,
    /// Custom blocked patterns (regex strings)
    pub blocked_patterns: Vec<String>,
}

impl Default for SanitizerConfig {
    fn default() -> Self {
        Self {
            max_string_length: 10_000,
            max_identifier_length: 256,
            max_expression_length: 1_000,
            strip_html: true,
            normalize_unicode: true,
            blocked_patterns: Vec::new(),
        }
    }
}

/// Input sanitizer for workflow definitions
pub struct InputSanitizer {
    config: SanitizerConfig,
    /// Reserved keywords that cannot be used as identifiers
    reserved_keywords: HashSet<String>,
    /// Dangerous patterns to detect
    dangerous_patterns: Vec<(&'static str, &'static str)>,
}

impl InputSanitizer {
    /// Create a new sanitizer with default config
    pub fn new() -> Self {
        Self::with_config(SanitizerConfig::default())
    }

    /// Create a sanitizer with custom config
    pub fn with_config(config: SanitizerConfig) -> Self {
        let reserved_keywords = [
            // JavaScript/TypeScript keywords
            "break", "case", "catch", "continue", "debugger", "default", "delete",
            "do", "else", "finally", "for", "function", "if", "in", "instanceof",
            "new", "return", "switch", "this", "throw", "try", "typeof", "var",
            "void", "while", "with", "class", "const", "enum", "export", "extends",
            "import", "super", "implements", "interface", "let", "package", "private",
            "protected", "public", "static", "yield", "await", "async",
            // Temporal workflow keywords
            "workflow", "activity", "signal", "query", "proxyActivities",
            // Built-in objects
            "Object", "Function", "Array", "String", "Boolean", "Number", "Date",
            "RegExp", "Error", "Math", "JSON", "Promise", "Proxy", "Reflect",
            "eval", "arguments", "undefined", "null", "NaN", "Infinity",
            // Node.js globals
            "require", "module", "exports", "process", "global", "Buffer",
            "__dirname", "__filename",
        ].iter().map(|s| s.to_string()).collect();

        let dangerous_patterns = vec![
            ("eval(", "Potential code injection via eval"),
            ("Function(", "Potential code injection via Function constructor"),
            ("setTimeout(", "Potential delayed code execution"),
            ("setInterval(", "Potential repeated code execution"),
            ("require(", "Potential module injection"),
            ("import(", "Potential dynamic import injection"),
            ("process.", "Potential process access"),
            ("child_process", "Potential child process spawning"),
            ("fs.", "Potential file system access"),
            ("__proto__", "Prototype pollution attempt"),
            ("constructor[", "Prototype pollution attempt"),
            (".constructor.", "Prototype pollution attempt"),
            ("${", "Template literal injection"),
            ("{{", "Template injection attempt"),
            ("<script", "Script injection attempt"),
            ("javascript:", "JavaScript URI injection"),
            ("data:", "Data URI injection"),
            ("onclick", "Event handler injection"),
            ("onerror", "Event handler injection"),
            ("onload", "Event handler injection"),
        ];

        Self {
            config,
            reserved_keywords,
            dangerous_patterns,
        }
    }

    /// Sanitize a workflow name
    pub fn sanitize_workflow_name(&self, name: &str) -> Result<SanitizationResult, SanitizationError> {
        let mut modifications = Vec::new();
        let mut value = name.to_string();

        // Trim whitespace
        let trimmed = value.trim();
        if trimmed.len() != value.len() {
            modifications.push("Trimmed whitespace".to_string());
            value = trimmed.to_string();
        }

        // Check length
        if value.is_empty() {
            return Err(SanitizationError::EmptyInput("workflow name".to_string()));
        }

        if value.len() > self.config.max_identifier_length {
            return Err(SanitizationError::TooLong {
                field: "workflow name".to_string(),
                max: self.config.max_identifier_length,
                actual: value.len(),
            });
        }

        // Validate identifier format
        if !self.is_valid_identifier(&value) {
            return Err(SanitizationError::InvalidIdentifier {
                value: value.clone(),
                reason: "Must start with letter or underscore, contain only alphanumeric and underscore".to_string(),
            });
        }

        // Check for reserved keywords
        if self.reserved_keywords.contains(&value.to_lowercase()) {
            return Err(SanitizationError::ReservedKeyword(value));
        }

        if modifications.is_empty() {
            Ok(SanitizationResult::unchanged(value))
        } else {
            Ok(SanitizationResult::modified(value, modifications))
        }
    }

    /// Sanitize a component ID
    pub fn sanitize_component_id(&self, id: &str) -> Result<SanitizationResult, SanitizationError> {
        let mut modifications = Vec::new();
        let mut value = id.to_string();

        // Trim whitespace
        let trimmed = value.trim();
        if trimmed.len() != value.len() {
            modifications.push("Trimmed whitespace".to_string());
            value = trimmed.to_string();
        }

        // Check length
        if value.is_empty() {
            return Err(SanitizationError::EmptyInput("component ID".to_string()));
        }

        if value.len() > self.config.max_identifier_length {
            return Err(SanitizationError::TooLong {
                field: "component ID".to_string(),
                max: self.config.max_identifier_length,
                actual: value.len(),
            });
        }

        // Component IDs allow hyphens and underscores
        if !self.is_valid_component_id(&value) {
            return Err(SanitizationError::InvalidIdentifier {
                value: value.clone(),
                reason: "Must contain only alphanumeric, hyphen, and underscore characters".to_string(),
            });
        }

        if modifications.is_empty() {
            Ok(SanitizationResult::unchanged(value))
        } else {
            Ok(SanitizationResult::modified(value, modifications))
        }
    }

    /// Sanitize an expression
    pub fn sanitize_expression(&self, expr: &str) -> Result<SanitizationResult, SanitizationError> {
        let mut modifications = Vec::new();
        let mut value = expr.to_string();

        // Trim whitespace
        let trimmed = value.trim();
        if trimmed.len() != value.len() {
            modifications.push("Trimmed whitespace".to_string());
            value = trimmed.to_string();
        }

        // Check length
        if value.len() > self.config.max_expression_length {
            return Err(SanitizationError::TooLong {
                field: "expression".to_string(),
                max: self.config.max_expression_length,
                actual: value.len(),
            });
        }

        // Check for dangerous patterns
        for (pattern, description) in &self.dangerous_patterns {
            if value.to_lowercase().contains(&pattern.to_lowercase()) {
                return Err(SanitizationError::DangerousPattern {
                    pattern: pattern.to_string(),
                    description: description.to_string(),
                });
            }
        }

        if modifications.is_empty() {
            Ok(SanitizationResult::unchanged(value))
        } else {
            Ok(SanitizationResult::modified(value, modifications))
        }
    }

    /// Sanitize a string value
    pub fn sanitize_string(&self, input: &str) -> Result<SanitizationResult, SanitizationError> {
        let mut modifications = Vec::new();
        let mut value = input.to_string();

        // Check length
        if value.len() > self.config.max_string_length {
            return Err(SanitizationError::TooLong {
                field: "string".to_string(),
                max: self.config.max_string_length,
                actual: value.len(),
            });
        }

        // Strip HTML if configured
        if self.config.strip_html {
            let stripped = self.strip_html_tags(&value);
            if stripped != value {
                modifications.push("Stripped HTML tags".to_string());
                value = stripped;
            }
        }

        // Normalize unicode if configured
        if self.config.normalize_unicode {
            let normalized = self.normalize_unicode(&value);
            if normalized != value {
                modifications.push("Normalized unicode".to_string());
                value = normalized;
            }
        }

        // Remove null bytes
        if value.contains('\0') {
            modifications.push("Removed null bytes".to_string());
            value = value.replace('\0', "");
        }

        if modifications.is_empty() {
            Ok(SanitizationResult::unchanged(value))
        } else {
            Ok(SanitizationResult::modified(value, modifications))
        }
    }

    /// Sanitize a URL
    pub fn sanitize_url(&self, url: &str) -> Result<SanitizationResult, SanitizationError> {
        let value = url.trim().to_string();

        // Check for dangerous URI schemes
        let lower = value.to_lowercase();
        if lower.starts_with("javascript:") {
            return Err(SanitizationError::DangerousPattern {
                pattern: "javascript:".to_string(),
                description: "JavaScript URI not allowed".to_string(),
            });
        }
        if lower.starts_with("data:") && !lower.starts_with("data:image/") {
            return Err(SanitizationError::DangerousPattern {
                pattern: "data:".to_string(),
                description: "Non-image data URIs not allowed".to_string(),
            });
        }
        if lower.starts_with("vbscript:") {
            return Err(SanitizationError::DangerousPattern {
                pattern: "vbscript:".to_string(),
                description: "VBScript URI not allowed".to_string(),
            });
        }

        // Validate URL structure (basic check)
        if !lower.starts_with("http://") && !lower.starts_with("https://") && !lower.starts_with("/") {
            return Err(SanitizationError::InvalidUrl {
                url: value,
                reason: "URL must start with http://, https://, or /".to_string(),
            });
        }

        Ok(SanitizationResult::unchanged(value))
    }

    /// Check if a string is a valid identifier
    fn is_valid_identifier(&self, s: &str) -> bool {
        if s.is_empty() {
            return false;
        }

        let mut chars = s.chars();

        // First character must be letter or underscore
        match chars.next() {
            Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
            _ => return false,
        }

        // Remaining characters must be alphanumeric or underscore
        chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
    }

    /// Check if a string is a valid component ID
    fn is_valid_component_id(&self, s: &str) -> bool {
        if s.is_empty() {
            return false;
        }

        s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    }

    /// Strip HTML tags from a string
    fn strip_html_tags(&self, s: &str) -> String {
        let mut result = String::with_capacity(s.len());
        let mut in_tag = false;

        for c in s.chars() {
            match c {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ if !in_tag => result.push(c),
                _ => {}
            }
        }

        result
    }

    /// Normalize unicode characters
    fn normalize_unicode(&self, s: &str) -> String {
        // Replace common lookalikes with ASCII equivalents
        s.chars()
            .map(|c| match c {
                '\u{2018}' | '\u{2019}' => '\'', // Smart quotes
                '\u{201C}' | '\u{201D}' => '"',  // Smart double quotes
                '\u{2013}' | '\u{2014}' => '-',  // En/em dash
                '\u{2026}' => '.',               // Ellipsis (just one dot)
                '\u{00A0}' => ' ',               // Non-breaking space
                _ => c,
            })
            .collect()
    }

    /// Check an expression for dangerous patterns (returns list of issues)
    pub fn check_dangerous_patterns(&self, expr: &str) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();
        let lower = expr.to_lowercase();

        for (pattern, description) in &self.dangerous_patterns {
            if lower.contains(&pattern.to_lowercase()) {
                issues.push(SecurityIssue {
                    severity: SecuritySeverity::High,
                    pattern: pattern.to_string(),
                    description: description.to_string(),
                    location: self.find_pattern_location(expr, pattern),
                });
            }
        }

        issues
    }

    /// Find the location of a pattern in a string
    fn find_pattern_location(&self, haystack: &str, needle: &str) -> Option<usize> {
        haystack.to_lowercase().find(&needle.to_lowercase())
    }
}

impl Default for InputSanitizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Security issue detected
#[derive(Debug, Clone)]
pub struct SecurityIssue {
    /// Severity of the issue
    pub severity: SecuritySeverity,
    /// Pattern that was detected
    pub pattern: String,
    /// Description of the issue
    pub description: String,
    /// Location in the input (character offset)
    pub location: Option<usize>,
}

/// Severity of a security issue
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecuritySeverity {
    /// Low severity - informational
    Low,
    /// Medium severity - should be reviewed
    Medium,
    /// High severity - likely malicious
    High,
    /// Critical severity - definitely malicious
    Critical,
}

impl std::fmt::Display for SecuritySeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecuritySeverity::Low => write!(f, "LOW"),
            SecuritySeverity::Medium => write!(f, "MEDIUM"),
            SecuritySeverity::High => write!(f, "HIGH"),
            SecuritySeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Sanitization error
#[derive(Debug, Clone)]
pub enum SanitizationError {
    /// Input is empty
    EmptyInput(String),
    /// Input is too long
    TooLong {
        field: String,
        max: usize,
        actual: usize,
    },
    /// Invalid identifier format
    InvalidIdentifier {
        value: String,
        reason: String,
    },
    /// Reserved keyword used
    ReservedKeyword(String),
    /// Dangerous pattern detected
    DangerousPattern {
        pattern: String,
        description: String,
    },
    /// Invalid URL
    InvalidUrl {
        url: String,
        reason: String,
    },
}

impl std::fmt::Display for SanitizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SanitizationError::EmptyInput(field) => {
                write!(f, "{} cannot be empty", field)
            }
            SanitizationError::TooLong { field, max, actual } => {
                write!(f, "{} is too long ({} chars, max {})", field, actual, max)
            }
            SanitizationError::InvalidIdentifier { value, reason } => {
                write!(f, "Invalid identifier '{}': {}", value, reason)
            }
            SanitizationError::ReservedKeyword(keyword) => {
                write!(f, "'{}' is a reserved keyword", keyword)
            }
            SanitizationError::DangerousPattern { pattern, description } => {
                write!(f, "Dangerous pattern '{}': {}", pattern, description)
            }
            SanitizationError::InvalidUrl { url, reason } => {
                write!(f, "Invalid URL '{}': {}", url, reason)
            }
        }
    }
}

impl std::error::Error for SanitizationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_workflow_name() {
        let sanitizer = InputSanitizer::new();

        // Valid names
        assert!(sanitizer.sanitize_workflow_name("myWorkflow").is_ok());
        assert!(sanitizer.sanitize_workflow_name("_private").is_ok());
        assert!(sanitizer.sanitize_workflow_name("workflow123").is_ok());

        // Invalid names
        assert!(sanitizer.sanitize_workflow_name("").is_err());
        assert!(sanitizer.sanitize_workflow_name("123invalid").is_err());
        assert!(sanitizer.sanitize_workflow_name("has-hyphen").is_err());
        assert!(sanitizer.sanitize_workflow_name("function").is_err()); // Reserved
    }

    #[test]
    fn test_sanitize_component_id() {
        let sanitizer = InputSanitizer::new();

        // Valid IDs
        assert!(sanitizer.sanitize_component_id("component-1").is_ok());
        assert!(sanitizer.sanitize_component_id("step_2").is_ok());
        assert!(sanitizer.sanitize_component_id("abc123").is_ok());

        // Invalid IDs
        assert!(sanitizer.sanitize_component_id("").is_err());
        assert!(sanitizer.sanitize_component_id("has space").is_err());
        assert!(sanitizer.sanitize_component_id("special@char").is_err());
    }

    #[test]
    fn test_sanitize_expression() {
        let sanitizer = InputSanitizer::new();

        // Valid expressions
        assert!(sanitizer.sanitize_expression("input.value + 1").is_ok());
        assert!(sanitizer.sanitize_expression("state.count > 0").is_ok());

        // Dangerous expressions
        let result = sanitizer.sanitize_expression("eval(input)");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SanitizationError::DangerousPattern { .. }));

        let result = sanitizer.sanitize_expression("process.exit(1)");
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitize_string() {
        let sanitizer = InputSanitizer::new();

        // Strip HTML
        let result = sanitizer.sanitize_string("<b>hello</b>").unwrap();
        assert!(result.was_modified);
        assert_eq!(result.value, "hello");

        // Remove null bytes
        let result = sanitizer.sanitize_string("hello\0world").unwrap();
        assert!(result.was_modified);
        assert_eq!(result.value, "helloworld");

        // Normalize unicode
        let result = sanitizer.sanitize_string("hello\u{2018}world\u{2019}").unwrap();
        assert!(result.was_modified);
        assert_eq!(result.value, "hello'world'");
    }

    #[test]
    fn test_sanitize_url() {
        let sanitizer = InputSanitizer::new();

        // Valid URLs
        assert!(sanitizer.sanitize_url("https://example.com/api").is_ok());
        assert!(sanitizer.sanitize_url("http://localhost:8080").is_ok());
        assert!(sanitizer.sanitize_url("/api/endpoint").is_ok());

        // Dangerous URLs
        assert!(sanitizer.sanitize_url("javascript:alert(1)").is_err());
        assert!(sanitizer.sanitize_url("data:text/html,<script>").is_err());
        assert!(sanitizer.sanitize_url("vbscript:msgbox").is_err());
    }

    #[test]
    fn test_check_dangerous_patterns() {
        let sanitizer = InputSanitizer::new();

        let issues = sanitizer.check_dangerous_patterns("safe expression");
        assert!(issues.is_empty());

        let issues = sanitizer.check_dangerous_patterns("eval(userInput)");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, SecuritySeverity::High);

        let issues = sanitizer.check_dangerous_patterns("process.env.SECRET");
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn test_trimming() {
        let sanitizer = InputSanitizer::new();

        let result = sanitizer.sanitize_workflow_name("  myWorkflow  ").unwrap();
        assert!(result.was_modified);
        assert_eq!(result.value, "myWorkflow");
        assert!(result.modifications.contains(&"Trimmed whitespace".to_string()));
    }

    #[test]
    fn test_reserved_keywords() {
        let sanitizer = InputSanitizer::new();

        // JavaScript keywords
        assert!(sanitizer.sanitize_workflow_name("class").is_err());
        assert!(sanitizer.sanitize_workflow_name("function").is_err());
        assert!(sanitizer.sanitize_workflow_name("await").is_err());

        // Temporal keywords
        assert!(sanitizer.sanitize_workflow_name("workflow").is_err());
        assert!(sanitizer.sanitize_workflow_name("activity").is_err());
    }

    #[test]
    fn test_length_limits() {
        let sanitizer = InputSanitizer::new();

        // Create a string that's too long
        let long_name = "a".repeat(300);
        let result = sanitizer.sanitize_workflow_name(&long_name);
        assert!(matches!(result.unwrap_err(), SanitizationError::TooLong { .. }));

        let long_expr = "x".repeat(2000);
        let result = sanitizer.sanitize_expression(&long_expr);
        assert!(matches!(result.unwrap_err(), SanitizationError::TooLong { .. }));
    }
}
