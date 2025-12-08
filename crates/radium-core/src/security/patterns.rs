//! Pattern library for detecting sensitive data types.

use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;

use super::privacy_error::{PrivacyError, Result};

/// A pattern for detecting sensitive data.
#[derive(Debug, Clone)]
pub struct Pattern {
    /// Human-readable name for this pattern.
    pub name: String,
    /// Compiled regex pattern.
    pub regex: Arc<Regex>,
    /// Optional validator function for additional validation (e.g., Luhn for credit cards).
    pub validator: Option<Arc<dyn Fn(&str) -> bool + Send + Sync>>,
}

impl Pattern {
    /// Creates a new pattern without a validator.
    pub fn new(name: impl Into<String>, regex: Regex) -> Self {
        Self {
            name: name.into(),
            regex: Arc::new(regex),
            validator: None,
        }
    }

    /// Creates a new pattern with a validator function.
    pub fn with_validator(
        name: impl Into<String>,
        regex: Regex,
        validator: impl Fn(&str) -> bool + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            regex: Arc::new(regex),
            validator: Some(Arc::new(validator)),
        }
    }

    /// Checks if the pattern matches the given text.
    ///
    /// Returns all matches found in the text.
    pub fn find_matches(&self, text: &str) -> Vec<String> {
        let mut matches = Vec::new();
        for cap in self.regex.captures_iter(text) {
            if let Some(matched) = cap.get(0) {
                let matched_str = matched.as_str().to_string();
                // If there's a validator, check it
                if let Some(ref validator) = self.validator {
                    if validator(&matched_str) {
                        matches.push(matched_str);
                    }
                } else {
                    matches.push(matched_str);
                }
            }
        }
        matches
    }
}

/// Library of patterns for detecting sensitive data.
#[derive(Debug, Clone)]
pub struct PatternLibrary {
    /// Collection of patterns.
    patterns: Vec<Pattern>,
}

// Built-in pattern regexes compiled lazily
static IPV4_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\b")
        .expect("IPv4 regex should be valid")
});

static IPV6_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}\b|\b::1\b|\bfe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]+\b")
        .expect("IPv6 regex should be valid")
});

static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    // Simplified RFC 5322 email pattern
    Regex::new(r"\b[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}\b")
        .expect("Email regex should be valid")
});

static AWS_ACCOUNT_ID_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b[0-9]{12}\b")
        .expect("AWS account ID regex should be valid")
});

static CREDIT_CARD_REGEX: Lazy<Regex> = Lazy::new(|| {
    // Matches 13-19 digit numbers that could be credit cards
    Regex::new(r"\b[0-9]{13,19}\b")
        .expect("Credit card regex should be valid")
});

static SSN_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b[0-9]{3}-[0-9]{2}-[0-9]{4}\b")
        .expect("SSN regex should be valid")
});

static API_KEY_REGEX: Lazy<Regex> = Lazy::new(|| {
    // Common API key patterns: sk_, pk_, Bearer tokens, etc.
    Regex::new(r"\b(?:sk|pk|tk|api[_-]?key|bearer|token)[_\-=:]\s*[a-zA-Z0-9_\-]{20,}\b")
        .expect("API key regex should be valid")
});

static PHONE_REGEX: Lazy<Regex> = Lazy::new(|| {
    // US phone number format
    Regex::new(r"\b[0-9]{3}-[0-9]{3}-[0-9]{4}\b|\b\([0-9]{3}\)\s*[0-9]{3}-[0-9]{4}\b")
        .expect("Phone regex should be valid")
});

/// Validates a credit card number using the Luhn algorithm.
pub fn validate_luhn(number: &str) -> bool {
    let digits: Vec<u32> = number
        .chars()
        .filter_map(|c| c.to_digit(10))
        .collect();

    if digits.len() < 13 || digits.len() > 19 {
        return false;
    }

    let sum: u32 = digits
        .iter()
        .rev()
        .enumerate()
        .map(|(i, &digit)| {
            if i % 2 == 1 {
                let doubled = digit * 2;
                if doubled > 9 {
                    doubled - 9
                } else {
                    doubled
                }
            } else {
                digit
            }
        })
        .sum();

    sum % 10 == 0
}

impl PatternLibrary {
    /// Creates a new empty pattern library.
    pub fn new() -> Self {
        Self { patterns: Vec::new() }
    }

    /// Adds a pattern to the library.
    pub fn add_pattern(&mut self, pattern: Pattern) {
        self.patterns.push(pattern);
    }

    /// Finds all matches in the given text, returning pattern names and their matches.
    ///
    /// Returns a map from pattern name to list of matched values.
    pub fn find_matches(&self, text: &str) -> HashMap<String, Vec<String>> {
        let mut results = HashMap::new();
        for pattern in &self.patterns {
            let matches = pattern.find_matches(text);
            if !matches.is_empty() {
                results.insert(pattern.name.clone(), matches);
            }
        }
        results
    }

    /// Checks if any pattern matches the given text.
    pub fn has_matches(&self, text: &str) -> bool {
        self.patterns.iter().any(|p| p.regex.is_match(text))
    }
}

impl Default for PatternLibrary {
    fn default() -> Self {
        let mut library = Self::new();

        // IPv4 addresses
        library.add_pattern(Pattern::new(
            "ipv4",
            (*IPV4_REGEX).clone(),
        ));

        // IPv6 addresses
        library.add_pattern(Pattern::new(
            "ipv6",
            (*IPV6_REGEX).clone(),
        ));

        // Email addresses
        library.add_pattern(Pattern::new(
            "email",
            (*EMAIL_REGEX).clone(),
        ));

        // AWS Account IDs (12 digits)
        library.add_pattern(Pattern::new(
            "aws_account_id",
            (*AWS_ACCOUNT_ID_REGEX).clone(),
        ));

        // Credit cards with Luhn validation
        library.add_pattern(Pattern::with_validator(
            "credit_card",
            (*CREDIT_CARD_REGEX).clone(),
            validate_luhn,
        ));

        // Social Security Numbers
        library.add_pattern(Pattern::new(
            "ssn",
            (*SSN_REGEX).clone(),
        ));

        // API keys and tokens
        library.add_pattern(Pattern::new(
            "api_key",
            (*API_KEY_REGEX).clone(),
        ));

        // Phone numbers
        library.add_pattern(Pattern::new(
            "phone",
            (*PHONE_REGEX).clone(),
        ));

        library
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipv4_pattern() {
        let library = PatternLibrary::default();
        let text = "Connect to 192.168.1.100";
        let matches = library.find_matches(text);
        assert!(matches.contains_key("ipv4"));
        assert!(matches["ipv4"].contains(&"192.168.1.100".to_string()));
    }

    #[test]
    fn test_email_pattern() {
        let library = PatternLibrary::default();
        let text = "Contact user@example.com for details";
        let matches = library.find_matches(text);
        assert!(matches.contains_key("email"));
        assert!(matches["email"].contains(&"user@example.com".to_string()));
    }

    #[test]
    fn test_aws_account_id_pattern() {
        let library = PatternLibrary::default();
        let text = "AWS account 123456789012";
        let matches = library.find_matches(text);
        assert!(matches.contains_key("aws_account_id"));
        assert!(matches["aws_account_id"].contains(&"123456789012".to_string()));
    }

    #[test]
    fn test_credit_card_with_luhn() {
        let library = PatternLibrary::default();
        // Valid Luhn number (test card)
        let valid_text = "Card number 4532015112830366";
        let matches = library.find_matches(valid_text);
        assert!(matches.contains_key("credit_card"));
        
        // Invalid Luhn number
        let invalid_text = "Card number 4532015112830367";
        let invalid_matches = library.find_matches(invalid_text);
        // Should not match because Luhn validation fails
        assert!(!invalid_matches.contains_key("credit_card"));
    }

    #[test]
    fn test_ssn_pattern() {
        let library = PatternLibrary::default();
        let text = "SSN: 123-45-6789";
        let matches = library.find_matches(text);
        assert!(matches.contains_key("ssn"));
        assert!(matches["ssn"].contains(&"123-45-6789".to_string()));
    }

    #[test]
    fn test_api_key_pattern() {
        let library = PatternLibrary::default();
        let text = "API key: sk_test_PLACEHOLDER_API_KEY_FOR_TESTING_ONLY_NOT_A_REAL_SECRET";
        let matches = library.find_matches(text);
        assert!(matches.contains_key("api_key"));
    }

    #[test]
    fn test_phone_pattern() {
        let library = PatternLibrary::default();
        let text = "Call 555-123-4567";
        let matches = library.find_matches(text);
        assert!(matches.contains_key("phone"));
        assert!(matches["phone"].contains(&"555-123-4567".to_string()));
    }

    #[test]
    fn test_luhn_validation() {
        // Valid Luhn numbers
        assert!(validate_luhn("4532015112830366"));
        assert!(validate_luhn("4111111111111111"));
        
        // Invalid Luhn numbers
        assert!(!validate_luhn("4532015112830367"));
        assert!(!validate_luhn("1234567890123456"));
    }

    #[test]
    fn test_multiple_patterns() {
        let library = PatternLibrary::default();
        let text = "Contact user@example.com at 192.168.1.100 or call 555-123-4567";
        let matches = library.find_matches(text);
        assert!(matches.contains_key("email"));
        assert!(matches.contains_key("ipv4"));
        assert!(matches.contains_key("phone"));
    }
}

