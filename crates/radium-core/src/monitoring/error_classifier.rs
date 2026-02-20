//! Error classification for log analysis.
//!
//! This module provides heuristic-based classification of errors detected in process logs,
//! determining severity levels and suggesting appropriate agents for remediation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

/// Severity level of an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Informational message, no action needed.
    Info,
    /// Warning that may lead to issues.
    Low,
    /// Error that should be addressed soon.
    Medium,
    /// Serious error requiring prompt attention.
    High,
    /// Critical failure requiring immediate action.
    Critical,
}

impl std::fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorSeverity::Info => write!(f, "info"),
            ErrorSeverity::Low => write!(f, "low"),
            ErrorSeverity::Medium => write!(f, "medium"),
            ErrorSeverity::High => write!(f, "high"),
            ErrorSeverity::Critical => write!(f, "critical"),
        }
    }
}

/// Type of error detected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorType {
    /// Type error (JavaScript, TypeScript, etc.)
    TypeError,
    /// Compilation/build error
    CompilationError,
    /// Runtime crash or panic
    RuntimeCrash,
    /// Network or connection error
    NetworkError,
    /// Database error
    DatabaseError,
    /// File system error
    FileSystemError,
    /// Permission or authentication error
    PermissionError,
    /// Dependency or import error
    DependencyError,
    /// Configuration error
    ConfigurationError,
    /// Generic or unknown error
    GenericError,
}

impl std::fmt::Display for ErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorType::TypeError => write!(f, "type_error"),
            ErrorType::CompilationError => write!(f, "compilation_error"),
            ErrorType::RuntimeCrash => write!(f, "runtime_crash"),
            ErrorType::NetworkError => write!(f, "network_error"),
            ErrorType::DatabaseError => write!(f, "database_error"),
            ErrorType::FileSystemError => write!(f, "filesystem_error"),
            ErrorType::PermissionError => write!(f, "permission_error"),
            ErrorType::DependencyError => write!(f, "dependency_error"),
            ErrorType::ConfigurationError => write!(f, "configuration_error"),
            ErrorType::GenericError => write!(f, "generic_error"),
        }
    }
}

/// Result of error classification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationResult {
    /// Severity level of the error.
    pub severity: ErrorSeverity,
    /// Type of error detected.
    pub error_type: ErrorType,
    /// Confidence score (0.0 to 1.0).
    pub confidence: f64,
    /// Suggested agent ID to handle this error.
    pub suggested_agent: Option<String>,
    /// Human-readable reasoning for the classification.
    pub reasoning: String,
    /// Keywords that influenced the classification.
    pub matched_keywords: Vec<String>,
}

/// Weights for severity scoring factors.
#[derive(Debug, Clone)]
pub struct SeverityWeights {
    /// Weight for keyword matching (default: 0.5).
    pub keyword_weight: f64,
    /// Weight for pattern matching (default: 0.3).
    pub pattern_weight: f64,
    /// Weight for context signals (default: 0.2).
    pub context_weight: f64,
}

impl Default for SeverityWeights {
    fn default() -> Self {
        Self {
            keyword_weight: 0.5,
            pattern_weight: 0.3,
            context_weight: 0.2,
        }
    }
}

/// Error classifier using heuristics and pattern matching.
pub struct ErrorClassifier {
    /// Weights for scoring factors.
    weights: SeverityWeights,
    /// Error frequency tracking for escalation.
    frequency_tracker: HashMap<String, (u32, std::time::SystemTime)>,
}

impl ErrorClassifier {
    /// Creates a new error classifier with default weights.
    pub fn new() -> Self {
        Self {
            weights: SeverityWeights::default(),
            frequency_tracker: HashMap::new(),
        }
    }

    /// Creates a new error classifier with custom weights.
    pub fn with_weights(weights: SeverityWeights) -> Self {
        Self {
            weights,
            frequency_tracker: HashMap::new(),
        }
    }

    /// Classifies an error from a log line.
    ///
    /// # Arguments
    /// * `log_line` - The log line containing the error
    /// * `exit_code` - Optional exit code if the process crashed
    pub fn classify(&mut self, log_line: &str, exit_code: Option<i32>) -> ClassificationResult {
        let lower = log_line.to_lowercase();

        // Determine error type
        let error_type = self.detect_error_type(&lower);

        // Calculate severity from multiple factors
        let keyword_score = self.score_keywords(&lower);
        let pattern_score = self.score_patterns(log_line, &lower);
        let context_score = self.score_context(exit_code);

        // Weighted combination
        let base_score = context_score.mul_add(self.weights.context_weight, keyword_score.mul_add(self.weights.keyword_weight, pattern_score * self.weights.pattern_weight));

        // Track frequency and escalate if needed
        let frequency_multiplier = self.track_frequency(log_line);
        let final_score = (base_score * frequency_multiplier).min(1.0);

        // Map score to severity
        let severity = Self::score_to_severity(final_score);

        // Suggest agent based on error type
        let suggested_agent = self.suggest_agent(&error_type, &lower);

        // Build reasoning
        let mut reasoning_parts = Vec::new();
        if keyword_score > 0.0 {
            reasoning_parts.push(format!("keyword signals (score: {:.2})", keyword_score));
        }
        if pattern_score > 0.0 {
            reasoning_parts.push(format!("pattern matching (score: {:.2})", pattern_score));
        }
        if context_score > 0.0 {
            reasoning_parts.push(format!("context signals (score: {:.2})", context_score));
        }
        if frequency_multiplier > 1.0 {
            reasoning_parts.push(format!("recurring error (x{:.1})", frequency_multiplier));
        }

        let reasoning = if reasoning_parts.is_empty() {
            "No strong error signals detected".to_string()
        } else {
            format!("Based on {}", reasoning_parts.join(", "))
        };

        // Extract matched keywords
        let matched_keywords = self.extract_matched_keywords(&lower);

        let result = ClassificationResult {
            severity,
            error_type,
            confidence: final_score,
            suggested_agent,
            reasoning,
            matched_keywords,
        };

        debug!(
            severity = ?result.severity,
            error_type = ?result.error_type,
            confidence = result.confidence,
            suggested_agent = ?result.suggested_agent,
            "Classified error"
        );

        result
    }

    /// Scores keywords in the log line.
    fn score_keywords(&self, lower: &str) -> f64 {
        let mut score: f64 = 0.0;

        // Critical keywords
        if lower.contains("panic") || lower.contains("fatal") || lower.contains("segfault") {
            score += 0.9;
        }

        // High severity keywords
        if lower.contains("error") || lower.contains("exception") || lower.contains("failed") {
            score += 0.7;
        }

        // Medium severity keywords
        if lower.contains("warn") || lower.contains("deprecated") || lower.contains("invalid") {
            score += 0.4;
        }

        // Low severity keywords
        if lower.contains("info") || lower.contains("debug") {
            score += 0.1;
        }

        score.min(1.0)
    }

    /// Scores patterns in the log line.
    fn score_patterns(&self, original: &str, lower: &str) -> f64 {
        let mut score: f64 = 0.0;

        // Stack trace patterns
        if original.contains("at ") && (original.contains('(') && original.contains(')'))
            || original.contains("  File \"")
            || lower.contains("stack trace")
        {
            score += 0.8;
        }

        // Exit code patterns
        if lower.contains("exit code") || lower.contains("exited with") {
            score += 0.6;
        }

        // Exception patterns
        if original.contains("Exception:") || original.contains("Error:") {
            score += 0.7;
        }

        // Port/address in use
        if lower.contains("address already in use") || lower.contains("port") && lower.contains("in use") {
            score += 0.5;
        }

        // Compilation errors
        if lower.contains("cannot find") || lower.contains("undefined reference") {
            score += 0.6;
        }

        score.min(1.0)
    }

    /// Scores context signals.
    fn score_context(&self, exit_code: Option<i32>) -> f64 {
        match exit_code {
            Some(code) if code != 0 => {
                // Non-zero exit codes are serious
                if (0..=128).contains(&code) {
                    0.7 // Standard error exit
                } else {
                    0.9 // Likely killed by signal
                }
            }
            _ => 0.0,
        }
    }

    /// Tracks error frequency and returns a multiplier for escalation.
    fn track_frequency(&mut self, log_line: &str) -> f64 {
        use std::time::SystemTime;
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        // Create a fingerprint (simplified hash)
        let mut hasher = DefaultHasher::new();
        log_line.hash(&mut hasher);
        let fingerprint = format!("{:x}", hasher.finish());

        let now = SystemTime::now();
        let (count, last_seen) = self
            .frequency_tracker
            .entry(fingerprint)
            .or_insert((0, now));

        *count += 1;

        // If error occurred within 5 minutes, escalate
        if let Ok(duration) = now.duration_since(*last_seen) {
            if duration.as_secs() < 300 {
                // Within 5 minutes
                let multiplier = (f64::from(*count) - 1.0).mul_add(0.2, 1.0);
                *last_seen = now;
                return multiplier.min(2.0); // Cap at 2x
            }
        }

        // Reset if it's been a while
        *count = 1;
        *last_seen = now;
        1.0
    }

    /// Detects the error type from the log line.
    fn detect_error_type(&self, lower: &str) -> ErrorType {
        if lower.contains("typeerror") || lower.contains("type error") {
            ErrorType::TypeError
        } else if lower.contains("compilation")
            || lower.contains("build failed")
            || lower.contains("cannot find module")
            || lower.contains("undefined reference")
        {
            ErrorType::CompilationError
        } else if lower.contains("panic")
            || lower.contains("segfault")
            || lower.contains("killed")
            || lower.contains("core dumped")
        {
            ErrorType::RuntimeCrash
        } else if lower.contains("network")
            || lower.contains("connection")
            || lower.contains("econnrefused")
            || lower.contains("etimedout")
        {
            ErrorType::NetworkError
        } else if lower.contains("database")
            || lower.contains("sql")
            || lower.contains("query failed")
        {
            ErrorType::DatabaseError
        } else if lower.contains("no such file")
            || lower.contains("enoent")
            || lower.contains("permission denied")
            || lower.contains("eacces")
        {
            if lower.contains("permission") || lower.contains("eacces") {
                ErrorType::PermissionError
            } else {
                ErrorType::FileSystemError
            }
        } else if lower.contains("cannot find package")
            || lower.contains("module not found")
            || lower.contains("import")
        {
            ErrorType::DependencyError
        } else if lower.contains("config") || lower.contains("invalid configuration") {
            ErrorType::ConfigurationError
        } else {
            ErrorType::GenericError
        }
    }

    /// Suggests an agent based on error type and context.
    fn suggest_agent(&self, error_type: &ErrorType, lower: &str) -> Option<String> {
        // Check language-specific patterns first
        if lower.contains("rust") || lower.contains("cargo") {
            return Some("rust-expert-agent".to_string());
        }
        if lower.contains("typescript") || lower.contains("node") || lower.contains("npm") {
            return Some("typescript-agent".to_string());
        }
        if lower.contains("python") || lower.contains("pip") {
            return Some("python-agent".to_string());
        }

        // Fall back to error type
        match error_type {
            ErrorType::CompilationError => Some("build-agent".to_string()),
            ErrorType::DatabaseError => Some("database-agent".to_string()),
            ErrorType::NetworkError => Some("network-agent".to_string()),
            _ => Some("code-agent".to_string()),
        }
    }

    /// Maps a score to a severity level.
    fn score_to_severity(score: f64) -> ErrorSeverity {
        if score >= 0.8 {
            ErrorSeverity::Critical
        } else if score >= 0.6 {
            ErrorSeverity::High
        } else if score >= 0.4 {
            ErrorSeverity::Medium
        } else if score >= 0.2 {
            ErrorSeverity::Low
        } else {
            ErrorSeverity::Info
        }
    }

    /// Extracts keywords that matched during classification.
    fn extract_matched_keywords(&self, lower: &str) -> Vec<String> {
        let mut keywords = Vec::new();

        let all_keywords = vec![
            "panic", "fatal", "segfault", "error", "exception", "failed", "warn", "deprecated",
            "invalid", "typeerror", "compilation", "build failed", "network", "connection",
            "database", "sql", "permission denied", "module not found",
        ];

        for keyword in all_keywords {
            if lower.contains(keyword) {
                keywords.push(keyword.to_string());
            }
        }

        keywords
    }

    /// Clears old entries from the frequency tracker.
    pub fn cleanup_frequency_tracker(&mut self) {
        use std::time::SystemTime;

        let now = SystemTime::now();
        self.frequency_tracker.retain(|_, (_, last_seen)| {
            if let Ok(duration) = now.duration_since(*last_seen) {
                duration.as_secs() < 3600 // Keep entries less than 1 hour old
            } else {
                false
            }
        });
    }
}

impl Default for ErrorClassifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_critical_error_classification() {
        let mut classifier = ErrorClassifier::new();

        let result = classifier.classify("FATAL: process panicked at main.rs:42", Some(139));

        // With default weights, this should be High severity (keywords + context)
        assert!(result.severity >= ErrorSeverity::High);
        assert_eq!(result.error_type, ErrorType::RuntimeCrash);
        assert!(result.confidence > 0.6);
    }

    #[test]
    fn test_compilation_error_classification() {
        let mut classifier = ErrorClassifier::new();

        let result = classifier.classify("error: cannot find module 'foo' in scope", None);

        assert_eq!(result.error_type, ErrorType::CompilationError);
        // With keyword "error" and pattern "cannot find", this should be Medium or higher
        assert!(result.severity >= ErrorSeverity::Medium);
    }

    #[test]
    fn test_type_error_classification() {
        let mut classifier = ErrorClassifier::new();

        let result = classifier.classify("TypeError: Cannot read property 'id' of undefined", None);

        assert_eq!(result.error_type, ErrorType::TypeError);
        assert!(result.confidence > 0.5);
    }

    #[test]
    fn test_frequency_escalation() {
        let mut classifier = ErrorClassifier::new();

        let log_line = "Error: Connection refused";

        // First occurrence
        let result1 = classifier.classify(log_line, None);
        let score1 = result1.confidence;

        // Second occurrence (should escalate)
        let result2 = classifier.classify(log_line, None);
        let score2 = result2.confidence;

        assert!(score2 > score1, "Repeated errors should escalate in severity");
    }

    #[test]
    fn test_agent_suggestion() {
        let mut classifier = ErrorClassifier::new();

        let result = classifier.classify("cargo build failed: error in main.rs", None);

        assert_eq!(result.suggested_agent, Some("rust-expert-agent".to_string()));
    }

    #[test]
    fn test_low_severity_warning() {
        let mut classifier = ErrorClassifier::new();

        let result = classifier.classify("warn: deprecated function called", None);

        assert_eq!(result.severity, ErrorSeverity::Low);
    }
}
