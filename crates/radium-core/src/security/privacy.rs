//! Privacy filter for redacting sensitive data.

use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use super::audit::AuditLogger;
use super::patterns::PatternLibrary;
use super::privacy_error::{PrivacyError, Result};
use chrono::Utc;

/// Style of redaction to apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedactionStyle {
    /// Replace entire match with "***".
    Full,
    /// Show first and last 25%, redact middle 50%.
    Partial,
    /// Replace with hash: "[REDACTED:sha256:abc12345]".
    Hash,
}

/// Statistics about redaction operations.
#[derive(Debug, Clone, Default)]
pub struct RedactionStats {
    /// Total number of redactions performed.
    pub count: usize,
    /// Count per pattern type.
    pub patterns: HashMap<String, usize>,
}

/// Privacy filter for redacting sensitive data from text.
#[derive(Clone)]
pub struct PrivacyFilter {
    /// Pattern library for detecting sensitive data.
    pattern_library: Arc<RwLock<PatternLibrary>>,
    /// Allowlist of values that should not be redacted.
    allowlist: Arc<RwLock<HashSet<String>>>,
    /// Redaction style to use.
    style: RedactionStyle,
    /// Optional audit logger for recording redactions.
    audit_logger: Option<Arc<AuditLogger>>,
}

impl PrivacyFilter {
    /// Creates a new privacy filter.
    ///
    /// # Arguments
    /// * `style` - The redaction style to use
    /// * `pattern_library` - The pattern library for detection
    pub fn new(style: RedactionStyle, pattern_library: PatternLibrary) -> Self {
        Self {
            pattern_library: Arc::new(RwLock::new(pattern_library)),
            allowlist: Arc::new(RwLock::new(HashSet::new())),
            style,
            audit_logger: None,
        }
    }

    /// Creates a new privacy filter with audit logging.
    ///
    /// # Arguments
    /// * `style` - The redaction style to use
    /// * `pattern_library` - The pattern library for detection
    /// * `audit_logger` - Optional audit logger for recording redactions
    pub fn with_audit_logger(
        style: RedactionStyle,
        pattern_library: PatternLibrary,
        audit_logger: Option<Arc<AuditLogger>>,
    ) -> Self {
        Self {
            pattern_library: Arc::new(RwLock::new(pattern_library)),
            allowlist: Arc::new(RwLock::new(HashSet::new())),
            style,
            audit_logger,
        }
    }

    /// Adds a value to the allowlist.
    ///
    /// Values in the allowlist will not be redacted even if they match patterns.
    pub fn add_to_allowlist(&self, value: String) {
        let mut allowlist = self.allowlist.write().unwrap();
        allowlist.insert(value);
    }

    /// Redacts sensitive data from the given text.
    ///
    /// # Arguments
    /// * `text` - The text to redact
    /// * `context` - Context where redaction occurs (e.g., "ContextManager.build_context")
    /// * `agent_id` - Optional agent ID that triggered the redaction
    ///
    /// # Returns
    /// A tuple of (redacted_text, statistics)
    pub fn redact(&self, text: &str) -> Result<(String, RedactionStats)> {
        self.redact_with_context(text, "unknown", None)
    }

    /// Redacts sensitive data from the given text with context information.
    ///
    /// # Arguments
    /// * `text` - The text to redact
    /// * `context` - Context where redaction occurs (e.g., "ContextManager.build_context")
    /// * `agent_id` - Optional agent ID that triggered the redaction
    ///
    /// # Returns
    /// A tuple of (redacted_text, statistics)
    pub fn redact_with_context(
        &self,
        text: &str,
        context: &str,
        agent_id: Option<&str>,
    ) -> Result<(String, RedactionStats)> {
        let pattern_library = self.pattern_library.read().unwrap();
        let allowlist = self.allowlist.read().unwrap();
        let matches = pattern_library.find_matches(text);

        let mut stats = RedactionStats::default();
        let mut redacted = text.to_string();

        // Process matches in reverse order to maintain string indices
        let mut all_matches: Vec<(usize, usize, String, String)> = Vec::new();
        for (pattern_name, matched_values) in &matches {
            for value in matched_values {
                // Check allowlist
                if allowlist.contains(value) {
                    continue;
                }

                // Find all occurrences of this value in the text
                let mut start = 0;
                while let Some(pos) = redacted[start..].find(value) {
                    let actual_pos = start + pos;
                    let end = actual_pos + value.len();
                    all_matches.push((actual_pos, end, pattern_name.clone(), value.clone()));
                    start = end;
                }
            }
        }

        // Sort by position (reverse order for safe replacement)
        all_matches.sort_by(|a, b| b.0.cmp(&a.0));

        // Apply redactions
        for (start, end, pattern_name, original_value) in all_matches {
            let redacted_value = match self.style {
                RedactionStyle::Full => "***".to_string(),
                RedactionStyle::Partial => Self::redact_partial(&original_value),
                RedactionStyle::Hash => Self::redact_hash(&original_value),
            };

            redacted.replace_range(start..end, &redacted_value);
            stats.count += 1;
            *stats.patterns.entry(pattern_name.clone()).or_insert(0) += 1;
        }

        // Log audit entry if logger is present and redactions occurred
        if stats.count > 0 {
            if let Some(ref logger) = self.audit_logger {
                // Create audit entry for each pattern type
                for (pattern_type, count) in &stats.patterns {
                    let entry = super::audit::AuditEntry {
                        timestamp: Utc::now().to_rfc3339(),
                        agent_id: agent_id.map(String::from),
                        pattern_type: pattern_type.clone(),
                        redaction_count: *count,
                        context: context.to_string(),
                        mode: format!("{:?}", self.style).to_lowercase(),
                    };
                    if let Err(e) = logger.log(entry) {
                        // Log error but don't fail redaction
                        tracing::warn!("Failed to log audit entry: {}", e);
                    }
                }
            }
        }

        Ok((redacted, stats))
    }

    /// Applies partial redaction to a value.
    ///
    /// Shows first and last 25%, redacts middle 50%.
    fn redact_partial(value: &str) -> String {
        let len = value.len();
        if len <= 4 {
            // Too short to partially redact, use full
            return "***".to_string();
        }

        let show_len = (len as f64 * 0.25).ceil() as usize;
        let prefix = &value[..show_len];
        let suffix = &value[len - show_len..];
        let redacted_len = len - (show_len * 2);
        let redacted_part = "*".repeat(redacted_len.min(10)); // Cap at 10 stars for readability

        format!("{}{}{}", prefix, redacted_part, suffix)
    }

    /// Applies hash redaction to a value.
    ///
    /// Returns "[REDACTED:sha256:first8chars]".
    fn redact_hash(value: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(value.as_bytes());
        let hash = hasher.finalize();
        let hash_str = format!("{:x}", hash);
        let truncated = &hash_str[..8.min(hash_str.len())];
        format!("[REDACTED:sha256:{}]", truncated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_redaction() {
        let library = PatternLibrary::default();
        let filter = PrivacyFilter::new(RedactionStyle::Full, library);
        let text = "Connect to 192.168.1.100";
        let (redacted, stats) = filter.redact(text).unwrap();
        assert!(redacted.contains("***"));
        assert!(!redacted.contains("192.168.1.100"));
        assert_eq!(stats.count, 1);
    }

    #[test]
    fn test_partial_redaction() {
        let library = PatternLibrary::default();
        let filter = PrivacyFilter::new(RedactionStyle::Partial, library);
        let text = "Connect to 192.168.1.100";
        let (redacted, stats) = filter.redact(text).unwrap();
        assert!(redacted.contains("192"));
        assert!(redacted.contains("100"));
        assert!(redacted.contains("*"));
        assert_eq!(stats.count, 1);
    }

    #[test]
    fn test_hash_redaction() {
        let library = PatternLibrary::default();
        let filter = PrivacyFilter::new(RedactionStyle::Hash, library);
        let text = "API key: sk_live_abc123";
        let (redacted, stats) = filter.redact(text).unwrap();
        assert!(redacted.contains("[REDACTED:sha256:"));
        assert!(!redacted.contains("sk_live_abc123"));
        assert_eq!(stats.count, 1);
    }

    #[test]
    fn test_allowlist() {
        let library = PatternLibrary::default();
        let filter = PrivacyFilter::new(RedactionStyle::Full, library);
        filter.add_to_allowlist("192.168.1.1".to_string());
        
        let text = "Connect to 192.168.1.1 and 192.168.1.100";
        let (redacted, stats) = filter.redact(text).unwrap();
        // Allowed IP should not be redacted
        assert!(redacted.contains("192.168.1.1"));
        // Other IP should be redacted
        assert!(!redacted.contains("192.168.1.100"));
        assert_eq!(stats.count, 1); // Only one redaction
    }

    #[test]
    fn test_multiple_patterns() {
        let library = PatternLibrary::default();
        let filter = PrivacyFilter::new(RedactionStyle::Full, library);
        let text = "Contact user@example.com at 192.168.1.100";
        let (redacted, stats) = filter.redact(text).unwrap();
        assert_eq!(stats.count, 2); // Email and IP
        assert!(stats.patterns.contains_key("email"));
        assert!(stats.patterns.contains_key("ipv4"));
    }

    #[test]
    fn test_partial_redaction_short_value() {
        let result = PrivacyFilter::redact_partial("123");
        assert_eq!(result, "***"); // Too short, uses full
    }

    #[test]
    fn test_hash_redaction_consistency() {
        let value = "test_value";
        let hash1 = PrivacyFilter::redact_hash(value);
        let hash2 = PrivacyFilter::redact_hash(value);
        assert_eq!(hash1, hash2); // Same input should produce same hash
        assert!(hash1.starts_with("[REDACTED:sha256:"));
    }
}

