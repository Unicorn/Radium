//! Verification types for component migration
//!
//! Types for comparing generated TypeScript against original behavior.

use serde::{Deserialize, Serialize};

/// Result of verifying a migrated component
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationResult {
    /// Overall pass/fail
    pub passed: bool,

    /// TypeScript compilation result
    pub typescript_compiles: bool,

    /// ESLint check result
    pub eslint_passes: bool,

    /// Behavior comparison result
    pub behavior_matches: bool,

    /// Individual test results
    #[serde(default)]
    pub test_results: Vec<TestResult>,

    /// Behavioral differences found
    #[serde(default)]
    pub differences: Vec<BehaviorDifference>,

    /// Error messages (if any)
    #[serde(default)]
    pub errors: Vec<String>,
}

impl VerificationResult {
    /// Create a passing verification result
    pub fn passed() -> Self {
        Self {
            passed: true,
            typescript_compiles: true,
            eslint_passes: true,
            behavior_matches: true,
            test_results: Vec::new(),
            differences: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Create a failing verification result
    pub fn failed(error: impl Into<String>) -> Self {
        Self {
            passed: false,
            typescript_compiles: false,
            eslint_passes: false,
            behavior_matches: false,
            test_results: Vec::new(),
            differences: Vec::new(),
            errors: vec![error.into()],
        }
    }

    /// Add a test result
    pub fn add_test_result(&mut self, result: TestResult) {
        if !result.passed {
            self.passed = false;
        }
        self.test_results.push(result);
    }

    /// Add a behavior difference
    pub fn add_difference(&mut self, diff: BehaviorDifference) {
        if diff.severity == DifferenceSeverity::Breaking {
            self.passed = false;
            self.behavior_matches = false;
        }
        self.differences.push(diff);
    }

    /// Add an error
    pub fn add_error(&mut self, error: impl Into<String>) {
        self.passed = false;
        self.errors.push(error.into());
    }

    /// Check if there are any breaking differences
    pub fn has_breaking_changes(&self) -> bool {
        self.differences
            .iter()
            .any(|d| d.severity == DifferenceSeverity::Breaking)
    }

    /// Get all breaking differences
    pub fn breaking_differences(&self) -> Vec<&BehaviorDifference> {
        self.differences
            .iter()
            .filter(|d| d.severity == DifferenceSeverity::Breaking)
            .collect()
    }
}

impl Default for VerificationResult {
    fn default() -> Self {
        Self::passed()
    }
}

/// Result of a single test
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestResult {
    /// Test name
    pub test_name: String,

    /// Whether it passed
    pub passed: bool,

    /// Original output (from TypeScript)
    #[serde(default)]
    pub original_output: String,

    /// Generated output (from Rust-generated TS)
    #[serde(default)]
    pub generated_output: String,

    /// Error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Execution time in ms
    #[serde(default)]
    pub duration_ms: u64,
}

impl TestResult {
    /// Create a passing test result
    pub fn passed(name: impl Into<String>) -> Self {
        Self {
            test_name: name.into(),
            passed: true,
            original_output: String::new(),
            generated_output: String::new(),
            error: None,
            duration_ms: 0,
        }
    }

    /// Create a failing test result
    pub fn failed(name: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            test_name: name.into(),
            passed: false,
            original_output: String::new(),
            generated_output: String::new(),
            error: Some(error.into()),
            duration_ms: 0,
        }
    }

    /// Set outputs for comparison
    pub fn with_outputs(
        mut self,
        original: impl Into<String>,
        generated: impl Into<String>,
    ) -> Self {
        self.original_output = original.into();
        self.generated_output = generated.into();
        self
    }

    /// Set duration
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }
}

/// Behavioral difference between original and generated code
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BehaviorDifference {
    /// Scenario where difference occurred
    pub scenario: String,

    /// Original behavior
    pub original_behavior: String,

    /// Generated behavior
    pub generated_behavior: String,

    /// Severity of the difference
    pub severity: DifferenceSeverity,

    /// Additional notes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl BehaviorDifference {
    /// Create a new behavior difference
    pub fn new(
        scenario: impl Into<String>,
        original: impl Into<String>,
        generated: impl Into<String>,
        severity: DifferenceSeverity,
    ) -> Self {
        Self {
            scenario: scenario.into(),
            original_behavior: original.into(),
            generated_behavior: generated.into(),
            severity,
            notes: None,
        }
    }

    /// Add notes
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// Create a breaking difference
    pub fn breaking(
        scenario: impl Into<String>,
        original: impl Into<String>,
        generated: impl Into<String>,
    ) -> Self {
        Self::new(scenario, original, generated, DifferenceSeverity::Breaking)
    }

    /// Create a warning difference
    pub fn warning(
        scenario: impl Into<String>,
        original: impl Into<String>,
        generated: impl Into<String>,
    ) -> Self {
        Self::new(scenario, original, generated, DifferenceSeverity::Warning)
    }

    /// Create a cosmetic difference
    pub fn cosmetic(
        scenario: impl Into<String>,
        original: impl Into<String>,
        generated: impl Into<String>,
    ) -> Self {
        Self::new(scenario, original, generated, DifferenceSeverity::Cosmetic)
    }
}

/// Severity of a behavioral difference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DifferenceSeverity {
    /// Breaking change - must be fixed
    Breaking,
    /// Warning - should be reviewed
    Warning,
    /// Cosmetic - acceptable difference
    Cosmetic,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verification_result_passed() {
        let result = VerificationResult::passed();
        assert!(result.passed);
        assert!(result.typescript_compiles);
        assert!(result.eslint_passes);
        assert!(result.behavior_matches);
    }

    #[test]
    fn test_verification_result_failed() {
        let result = VerificationResult::failed("Compilation error");
        assert!(!result.passed);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_verification_result_add_difference() {
        let mut result = VerificationResult::passed();
        result.add_difference(BehaviorDifference::breaking(
            "null handling",
            "returns null",
            "throws error",
        ));

        assert!(!result.passed);
        assert!(!result.behavior_matches);
        assert!(result.has_breaking_changes());
    }

    #[test]
    fn test_test_result() {
        let result = TestResult::passed("test_serialization")
            .with_outputs(r#"{"a": 1}"#, r#"{"a": 1}"#)
            .with_duration(5);

        assert!(result.passed);
        assert_eq!(result.duration_ms, 5);
    }

    #[test]
    fn test_behavior_difference() {
        let diff = BehaviorDifference::warning(
            "empty array handling",
            "returns []",
            "returns null",
        )
        .with_notes("Consider changing to match original");

        assert_eq!(diff.severity, DifferenceSeverity::Warning);
        assert!(diff.notes.is_some());
    }

    #[test]
    fn test_breaking_differences() {
        let mut result = VerificationResult::passed();
        result.add_difference(BehaviorDifference::cosmetic("formatting", "a", "b"));
        result.add_difference(BehaviorDifference::breaking("logic", "x", "y"));
        result.add_difference(BehaviorDifference::warning("edge case", "1", "2"));

        let breaking = result.breaking_differences();
        assert_eq!(breaking.len(), 1);
        assert_eq!(breaking[0].scenario, "logic");
    }
}
