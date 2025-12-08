//! Secret filter for pre-LLM credential redaction.
//!
//! Replaces real credential values with placeholders before sending
//! content to LLMs to prevent credential exposure in agent responses.

use std::sync::Arc;
use regex::Regex;

use super::error::{SecurityError, SecurityResult};
use super::secret_manager::SecretManager;

/// Credential match found during pattern detection.
#[derive(Debug, Clone)]
pub struct CredentialMatch {
    /// Start position in the content.
    pub start: usize,
    /// End position in the content.
    pub end: usize,
    /// Type of credential detected.
    pub credential_type: String,
    /// Matched text (truncated for safety).
    pub matched_text: String,
}

/// Secret filter for redacting credentials from content.
///
/// Replaces registered secret values and detects unregistered credentials
/// using pattern matching, replacing them with placeholders before sending
/// to LLMs.
///
/// # Example
///
/// ```no_run
/// use radium_core::security::{SecretManager, SecretFilter};
/// use std::sync::Arc;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let manager = Arc::new(SecretManager::new(
///     std::path::PathBuf::from("~/.radium/auth/secrets.vault"),
///     "master-password"
/// )?);
///
/// let filter = SecretFilter::new(manager);
///
/// // Content with real API key
/// let content = "Use API key sk-test123456789012345678901234567890123456";
/// let filtered = filter.redact_secrets(content)?;
///
/// // Content now has placeholder
/// assert!(filtered.contains("{{SECRET:"));
/// # Ok(())
/// # }
/// ```
pub struct SecretFilter {
    /// Reference to the secret manager.
    secret_manager: Arc<SecretManager>,
    /// Compiled regex patterns for credential detection.
    patterns: Vec<(String, Regex, String)>, // (name, pattern, placeholder_type)
}

impl SecretFilter {
    /// Creates a new secret filter.
    ///
    /// # Arguments
    ///
    /// * `secret_manager` - Shared reference to the secret manager
    pub fn new(secret_manager: Arc<SecretManager>) -> Self {
        let patterns = Self::compile_patterns();
        Self {
            secret_manager,
            patterns,
        }
    }

    /// Compiles regex patterns for common credential types.
    fn compile_patterns() -> Vec<(String, Regex, String)> {
        vec![
            (
                "openai_key".to_string(),
                Regex::new(r"sk-[A-Za-z0-9]{48}").unwrap(),
                "OpenAI API Key".to_string(),
            ),
            (
                "google_api_key".to_string(),
                Regex::new(r"AIza[0-9A-Za-z-_]{35}").unwrap(),
                "Google API Key".to_string(),
            ),
            (
                "github_token".to_string(),
                Regex::new(r"ghp_[A-Za-z0-9]{36}|gho_[A-Za-z0-9]{36}").unwrap(),
                "GitHub Token".to_string(),
            ),
            (
                "aws_key".to_string(),
                Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(),
                "AWS Access Key".to_string(),
            ),
            (
                "generic_api_key".to_string(),
                Regex::new(r#"(?i)(api[_-]?key|apikey)['"]?\s*[:=]\s*['"]?([A-Za-z0-9]{20,})"#).unwrap(),
                "Generic API Key".to_string(),
            ),
            (
                "bearer_token".to_string(),
                Regex::new(r"(?i)bearer\s+([A-Za-z0-9\-._~+/]+=*)").unwrap(),
                "Bearer Token".to_string(),
            ),
        ]
    }

    /// Checks if a string is already a placeholder.
    fn is_placeholder(&self, text: &str) -> bool {
        text.contains("{{SECRET:") || text.starts_with("$SECRET_")
    }

    /// Redacts all secrets from content, replacing them with placeholders.
    ///
    /// # Arguments
    ///
    /// * `content` - Content to filter
    ///
    /// # Returns
    ///
    /// Filtered content with secrets replaced by placeholders
    ///
    /// # Errors
    ///
    /// Returns an error if secret manager operations fail.
    pub fn redact_secrets(&self, content: &str) -> SecurityResult<String> {
        let mut filtered = content.to_string();

        // First, redact registered secrets
        if let Ok(secret_names) = self.secret_manager.list_secrets() {
            for name in secret_names {
                if let Ok(value) = self.secret_manager.get_secret(&name) {
                    // Skip if the value is already a placeholder (avoid double-redaction)
                    if self.is_placeholder(&value) {
                        continue;
                    }

                    // Replace all occurrences of the secret value with placeholder
                    let placeholder = format!("{{{{SECRET:{}}}}}", name);
                    filtered = filtered.replace(&value, &placeholder);
                }
            }
        }

        // Then, detect and redact unregistered credentials using patterns
        for (pattern_name, pattern, cred_type) in &self.patterns {
            // Find all matches
            let mut replacements = Vec::new();
            for cap in pattern.captures_iter(&filtered) {
                if let Some(matched) = cap.get(0) {
                    let matched_text = matched.as_str();
                    let start = matched.start();
                    let end = matched.end();

                    // Skip if already a placeholder
                    if self.is_placeholder(matched_text) {
                        continue;
                    }

                    // Create placeholder based on pattern
                    let placeholder = if pattern_name == "generic_api_key" || pattern_name == "bearer_token" {
                        // For patterns with capture groups, use detected type
                        format!("{{{{SECRET:detected_{}}}}}", pattern_name)
                    } else {
                        format!("{{{{SECRET:detected_{}}}}}", pattern_name)
                    };

                    replacements.push((start, end, placeholder));
                }
            }

            // Apply replacements in reverse order to preserve indices
            replacements.sort_by(|a, b| b.0.cmp(&a.0));
            for (start, end, placeholder) in replacements {
                filtered.replace_range(start..end, &placeholder);
            }
        }

        Ok(filtered)
    }

    /// Detects potential credentials in content without redacting.
    ///
    /// # Arguments
    ///
    /// * `content` - Content to scan
    ///
    /// # Returns
    ///
    /// Vector of credential matches with positions and types
    pub fn detect_credentials(&self, content: &str) -> Vec<CredentialMatch> {
        let mut matches = Vec::new();

        // Check registered secrets first
        if let Ok(secret_names) = self.secret_manager.list_secrets() {
            for name in secret_names {
                if let Ok(value) = self.secret_manager.get_secret(&name) {
                    // Find all occurrences
                    let mut start = 0;
                    while let Some(pos) = content[start..].find(&value) {
                        let abs_pos = start + pos;
                        let end = abs_pos + value.len();

                        matches.push(CredentialMatch {
                            start: abs_pos,
                            end,
                            credential_type: format!("Registered Secret: {}", name),
                            matched_text: format!("{}...", &value[..value.len().min(8)]),
                        });

                        start = end;
                    }
                }
            }
        }

        // Detect unregistered credentials using patterns
        for (pattern_name, pattern, cred_type) in &self.patterns {
            for cap in pattern.captures_iter(content) {
                if let Some(matched) = cap.get(0) {
                    let matched_text = matched.as_str();
                    let start = matched.start();
                    let end = matched.end();

                    // Skip if already a placeholder
                    if self.is_placeholder(matched_text) {
                        continue;
                    }

                    matches.push(CredentialMatch {
                        start,
                        end,
                        credential_type: cred_type.clone(),
                        matched_text: format!("{}...", &matched_text[..matched_text.len().min(8)]),
                    });
                }
            }
        }

        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_filter() -> (SecretFilter, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("secrets.vault");
        let manager = Arc::new(SecretManager::new(vault_path, "TestPassword123!").unwrap());
        let filter = SecretFilter::new(manager);
        (filter, temp_dir)
    }

    #[test]
    fn test_redact_registered_secret() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("secrets.vault");
        let manager = Arc::new(SecretManager::new(vault_path, "TestPassword123!").unwrap());

        // Store a secret
        manager.store_secret("api_key", "sk-test123456789012345678901234567890123456").unwrap();

        let filter = SecretFilter::new(manager);
        let content = "Use API key sk-test123456789012345678901234567890123456 here";
        let filtered = filter.redact_secrets(content).unwrap();

        assert!(filtered.contains("{{SECRET:api_key}}"));
        assert!(!filtered.contains("sk-test123456789012345678901234567890123456"));
    }

    #[test]
    fn test_detect_unregistered_credential() {
        let (filter, _temp_dir) = create_test_filter();

        let content = "API key: sk-test123456789012345678901234567890123456";
        let matches = filter.detect_credentials(content);

        assert!(!matches.is_empty());
        assert!(matches.iter().any(|m| m.credential_type.contains("OpenAI")));
    }

    #[test]
    fn test_preserve_existing_placeholders() {
        let (filter, _temp_dir) = create_test_filter();

        let content = "Use {{SECRET:api_key}} here";
        let filtered = filter.redact_secrets(content).unwrap();

        // Placeholder should remain unchanged
        assert_eq!(filtered, content);
    }

    #[test]
    fn test_is_placeholder() {
        let (filter, _temp_dir) = create_test_filter();

        assert!(filter.is_placeholder("{{SECRET:name}}"));
        assert!(filter.is_placeholder("$SECRET_NAME"));
        assert!(!filter.is_placeholder("sk-test123"));
    }
}

