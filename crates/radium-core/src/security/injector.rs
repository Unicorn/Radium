//! Secret injector for pre-tool credential injection.
//!
//! Replaces placeholder references with real credential values just before
//! tool execution, enabling agents to use credentials without ever seeing
//! the actual values.

use std::collections::HashMap;
use std::sync::Arc;
use regex::Regex;

use super::error::{SecurityError, SecurityResult};
use super::secret_manager::SecretManager;

/// Secret injector for replacing placeholders with real values.
///
/// Replaces `{{SECRET:name}}` placeholders and `$SECRET_NAME` environment
/// variable syntax with actual secret values just before tool execution.
///
/// # Example
///
/// ```no_run
/// use radium_core::security::{SecretManager, SecretInjector};
/// use std::sync::Arc;
/// use std::collections::HashMap;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let manager = Arc::new(SecretManager::new(
///     std::path::PathBuf::from("~/.radium/auth/secrets.vault"),
///     "master-password"
/// )?);
///
/// let injector = SecretInjector::new(manager);
///
/// // Content with placeholder
/// let content = "Use {{SECRET:api_key}} here";
/// let injected = injector.inject_secrets(content)?;
///
/// // Content now has real value
/// assert!(!injected.contains("{{SECRET:"));
///
/// // Environment variables
/// let mut env = HashMap::new();
/// env.insert("API_KEY".to_string(), "{{SECRET:api_key}}".to_string());
/// injector.inject_env_vars(&mut env)?;
/// # Ok(())
/// # }
/// ```
pub struct SecretInjector {
    /// Reference to the secret manager.
    secret_manager: Arc<SecretManager>,
    /// Compiled regex for {{SECRET:name}} placeholders.
    placeholder_regex: Regex,
    /// Compiled regex for $SECRET_NAME environment variable syntax.
    env_var_regex: Regex,
}

impl SecretInjector {
    /// Creates a new secret injector.
    ///
    /// # Arguments
    ///
    /// * `secret_manager` - Shared reference to the secret manager
    pub fn new(secret_manager: Arc<SecretManager>) -> Self {
        // Pattern: {{SECRET:name}}
        let placeholder_regex = Regex::new(r"\{\{SECRET:(\w+)\}\}").unwrap();

        // Pattern: $SECRET_NAME or ${SECRET_NAME}
        let env_var_regex = Regex::new(r"\$SECRET_(\w+)|\$\{SECRET_(\w+)\}").unwrap();

        Self {
            secret_manager,
            placeholder_regex,
            env_var_regex,
        }
    }

    /// Extracts secret names from content without injecting.
    ///
    /// Useful for validation before injection.
    ///
    /// # Arguments
    ///
    /// * `content` - Content to scan for placeholders
    ///
    /// # Returns
    ///
    /// Vector of unique secret names found
    pub fn extract_secret_names(&self, content: &str) -> Vec<String> {
        let mut names = std::collections::HashSet::new();

        // Extract from {{SECRET:name}} format
        for cap in self.placeholder_regex.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                names.insert(name.as_str().to_string());
            }
        }

        // Extract from $SECRET_NAME format
        for cap in self.env_var_regex.captures_iter(content) {
            if let Some(name) = cap.get(1).or_else(|| cap.get(2)) {
                names.insert(name.as_str().to_string());
            }
        }

        names.into_iter().collect()
    }

    /// Injects real secret values into content, replacing placeholders.
    ///
    /// # Arguments
    ///
    /// * `content` - Content with placeholders
    ///
    /// # Returns
    ///
    /// Content with placeholders replaced by real values
    ///
    /// # Errors
    ///
    /// Returns an error if any secret is not found.
    pub fn inject_secrets(&self, content: &str) -> SecurityResult<String> {
        let mut injected = content.to_string();

        // Replace {{SECRET:name}} placeholders
        let mut replacements = Vec::new();
        for cap in self.placeholder_regex.captures_iter(content) {
            if let Some(full_match) = cap.get(0) {
                if let Some(name_match) = cap.get(1) {
                    let secret_name = name_match.as_str();
                    let start = full_match.start();
                    let end = full_match.end();

                    // Get secret value
                    let value = self.secret_manager.get_secret(secret_name)
                        .map_err(|e| match e {
                            SecurityError::SecretNotFound(_) => {
                                SecurityError::SecretNotFound(secret_name.to_string())
                            }
                            _ => e,
                        })?;

                    replacements.push((start, end, value));
                }
            }
        }

        // Apply replacements in reverse order to preserve indices
        replacements.sort_by(|a, b| b.0.cmp(&a.0));
        for (start, end, value) in replacements {
            injected.replace_range(start..end, &value);
        }

        // Replace $SECRET_NAME environment variable syntax
        let mut env_replacements = Vec::new();
        for cap in self.env_var_regex.captures_iter(&injected) {
            if let Some(full_match) = cap.get(0) {
                if let Some(name_match) = cap.get(1).or_else(|| cap.get(2)) {
                    let secret_name = name_match.as_str();
                    let start = full_match.start();
                    let end = full_match.end();

                    // Get secret value
                    let value = self.secret_manager.get_secret(secret_name)
                        .map_err(|e| match e {
                            SecurityError::SecretNotFound(_) => {
                                SecurityError::SecretNotFound(secret_name.to_string())
                            }
                            _ => e,
                        })?;

                    env_replacements.push((start, end, value));
                }
            }
        }

        // Apply replacements in reverse order
        env_replacements.sort_by(|a, b| b.0.cmp(&a.0));
        for (start, end, value) in env_replacements {
            injected.replace_range(start..end, &value);
        }

        Ok(injected)
    }

    /// Injects secrets into environment variables.
    ///
    /// Scans environment variable values for placeholders and replaces them
    /// with real secret values.
    ///
    /// # Arguments
    ///
    /// * `env` - Environment variable map to modify
    ///
    /// # Errors
    ///
    /// Returns an error if any secret is not found.
    pub fn inject_env_vars(&self, env: &mut HashMap<String, String>) -> SecurityResult<()> {
        let mut updates = Vec::new();

        // Scan all environment variable values
        for (key, value) in env.iter() {
            // Check if value contains placeholders
            let secret_names = self.extract_secret_names(value);
            if !secret_names.is_empty() {
                let mut new_value = value.clone();

                // Replace all placeholders in the value
                for name in &secret_names {
                    let placeholder1 = format!("{{{{SECRET:{}}}}}", name);
                    let placeholder2 = format!("$SECRET_{}", name);
                    let placeholder3 = format!("${{SECRET_{}}}", name);

                    let secret_value = self.secret_manager.get_secret(name)
                        .map_err(|e| match e {
                            SecurityError::SecretNotFound(_) => {
                                SecurityError::SecretNotFound(name.clone())
                            }
                            _ => e,
                        })?;

                    new_value = new_value.replace(&placeholder1, &secret_value);
                    new_value = new_value.replace(&placeholder2, &secret_value);
                    new_value = new_value.replace(&placeholder3, &secret_value);
                }

                updates.push((key.clone(), new_value));
            }
        }

        // Apply updates
        for (key, value) in updates {
            env.insert(key, value);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_injector() -> (SecretInjector, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("secrets.vault");
        let manager = Arc::new(SecretManager::new(vault_path, "TestPassword123!").unwrap());
        let injector = SecretInjector::new(manager);
        (injector, temp_dir)
    }

    #[test]
    fn test_inject_single_secret() {
        let (injector, temp_dir) = create_test_injector();
        let mut manager = SecretManager::new(
            temp_dir.path().join("secrets.vault"),
            "TestPassword123!",
        ).unwrap();

        // Store a secret
        manager.store_secret("api_key", "sk-real-value-12345").unwrap();

        let injector = SecretInjector::new(Arc::new(manager));
        let content = "Use {{SECRET:api_key}} here";
        let injected = injector.inject_secrets(content).unwrap();

        assert_eq!(injected, "Use sk-real-value-12345 here");
        assert!(!injected.contains("{{SECRET:"));
    }

    #[test]
    fn test_inject_environment_variables() {
        let (injector, temp_dir) = create_test_injector();
        let mut manager = SecretManager::new(
            temp_dir.path().join("secrets.vault"),
            "TestPassword123!",
        ).unwrap();

        manager.store_secret("api_key", "sk-real-value").unwrap();

        let injector = SecretInjector::new(Arc::new(manager));
        let mut env = HashMap::new();
        env.insert("API_KEY".to_string(), "{{SECRET:api_key}}".to_string());

        injector.inject_env_vars(&mut env).unwrap();

        assert_eq!(env.get("API_KEY"), Some(&"sk-real-value".to_string()));
    }

    #[test]
    fn test_missing_secret_fails_safely() {
        let (injector, _temp_dir) = create_test_injector();

        let content = "Use {{SECRET:missing}} here";
        let result = injector.inject_secrets(content);

        assert!(matches!(result, Err(SecurityError::SecretNotFound(_))));
    }

    #[test]
    fn test_extract_secret_names() {
        let (injector, _temp_dir) = create_test_injector();

        let content = "Use {{SECRET:api_key}} and {{SECRET:token}} here";
        let names = injector.extract_secret_names(content);

        assert_eq!(names.len(), 2);
        assert!(names.contains(&"api_key".to_string()));
        assert!(names.contains(&"token".to_string()));
    }

    #[test]
    fn test_inject_env_var_syntax() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("secrets.vault");
        let mut manager = SecretManager::new(vault_path, "TestPassword123!").unwrap();

        manager.store_secret("api_key", "sk-real-value").unwrap();

        let manager = Arc::new(manager);
        let injector = SecretInjector::new(manager);
        let content = "Use $SECRET_api_key here";
        let injected = injector.inject_secrets(content).unwrap();

        assert_eq!(injected, "Use sk-real-value here");
    }
}

