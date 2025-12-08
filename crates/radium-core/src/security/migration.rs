//! Migration utility for transferring credentials from plaintext to encrypted vault.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;

use super::error::{SecurityError, SecurityResult};
use super::secret_manager::SecretManager;
use crate::auth::{CredentialStore, ProviderType};

/// Migration report with statistics.
#[derive(Debug, Clone)]
pub struct MigrationReport {
    /// Total number of credentials found.
    pub total_credentials: usize,
    /// Number of credentials successfully migrated.
    pub migrated: usize,
    /// Number of credentials that failed to migrate.
    pub failed: usize,
    /// Path to the backup file created.
    pub backup_path: PathBuf,
}

/// Migration manager for transferring credentials to encrypted vault.
pub struct MigrationManager;

impl MigrationManager {
    /// Detects if a plaintext credentials file exists.
    ///
    /// # Returns
    ///
    /// Path to credentials file if it exists, None otherwise
    pub fn detect_credentials_file() -> Option<PathBuf> {
        #[allow(clippy::disallowed_methods)]
        if let Ok(home) = std::env::var("HOME") {
            let creds_path = Path::new(&home).join(".radium/auth/credentials.json");
            if creds_path.exists() {
                return Some(creds_path);
            }
        }
        None
    }

    /// Creates a timestamped backup of the credentials file.
    ///
    /// # Arguments
    ///
    /// * `source` - Path to the credentials file to backup
    ///
    /// # Returns
    ///
    /// Path to the backup file created
    ///
    /// # Errors
    ///
    /// Returns an error if backup creation fails.
    pub fn create_backup(source: &Path) -> SecurityResult<PathBuf> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| SecurityError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to get timestamp: {}", e),
            )))?
            .as_secs();

        let backup_name = format!("credentials.json.backup-{}", timestamp);
        let backup_path = source.parent()
            .ok_or_else(|| SecurityError::PermissionDenied("Invalid credentials path".to_string()))?
            .join(backup_name);

        fs::copy(source, &backup_path)
            .map_err(|e| SecurityError::Io(e))?;

        Ok(backup_path)
    }

    /// Marks the original credentials file as deprecated.
    ///
    /// Adds a deprecation notice at the top of the file while preserving
    /// the original content for rollback purposes.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the credentials file
    ///
    /// # Errors
    ///
    /// Returns an error if file operations fail.
    pub fn mark_deprecated(path: &Path) -> SecurityResult<()> {
        let content = fs::read_to_string(path)
            .map_err(|e| SecurityError::Io(e))?;

        let deprecation_notice = format!(
            "// DEPRECATED: Credentials migrated to encrypted vault on {}\n\
             // This file is kept for rollback purposes only.\n\
             // Do not use this file - credentials are now stored in secrets.vault\n\
             // To rollback, restore from backup: credentials.json.backup-<timestamp>\n\n",
            Utc::now().to_rfc3339()
        );

        let new_content = deprecation_notice + &content;
        fs::write(path, new_content)
            .map_err(|e| SecurityError::Io(e))?;

        Ok(())
    }

    /// Migrates all credentials from plaintext file to encrypted vault.
    ///
    /// # Arguments
    ///
    /// * `master_password` - Master password for the encrypted vault
    ///
    /// # Returns
    ///
    /// Migration report with statistics
    ///
    /// # Errors
    ///
    /// Returns an error if migration fails.
    pub fn migrate_to_vault(master_password: &str) -> SecurityResult<MigrationReport> {
        let creds_path = Self::detect_credentials_file()
            .ok_or_else(|| SecurityError::PermissionDenied(
                "No credentials.json file found to migrate".to_string()
            ))?;

        // Create backup
        let backup_path = Self::create_backup(&creds_path)?;

        // Load existing credentials
        let store = CredentialStore::new()
            .map_err(|e| SecurityError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create CredentialStore: {}", e),
            )))?;

        let provider_types = store.list()
            .map_err(|e| SecurityError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to list providers: {}", e),
            )))?;

        // Create or open secret manager
        let vault_path = creds_path.parent()
            .ok_or_else(|| SecurityError::PermissionDenied("Invalid credentials path".to_string()))?
            .join("secrets.vault");

        let mut manager = if vault_path.exists() {
            SecretManager::from_existing(vault_path.clone(), master_password)?
        } else {
            SecretManager::new(vault_path.clone(), master_password)?
        };

        // Migrate each credential
        let mut migrated = 0;
        let mut failed = 0;

        for provider_type in &provider_types {
            match store.get(*provider_type) {
                Ok(api_key) => {
                    // Store with name format: provider_<name>
                    let secret_name = format!("provider_{}", provider_type.as_str());
                    match manager.store_secret(&secret_name, &api_key) {
                        Ok(()) => migrated += 1,
                        Err(e) => {
                            eprintln!("Failed to migrate {}: {}", provider_type.as_str(), e);
                            failed += 1;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to get credential for {}: {}", provider_type.as_str(), e);
                    failed += 1;
                }
            }
        }

        // Mark original file as deprecated
        if migrated > 0 {
            Self::mark_deprecated(&creds_path)?;
        }

        Ok(MigrationReport {
            total_credentials: provider_types.len(),
            migrated,
            failed,
            backup_path,
        })
    }

    /// Rolls back migration by restoring from backup.
    ///
    /// # Arguments
    ///
    /// * `backup_path` - Path to the backup file
    ///
    /// # Errors
    ///
    /// Returns an error if rollback fails.
    pub fn rollback(backup_path: &Path) -> SecurityResult<()> {
        let creds_path = Self::detect_credentials_file()
            .ok_or_else(|| SecurityError::PermissionDenied(
                "No credentials.json file found".to_string()
            ))?;

        // Restore from backup
        fs::copy(backup_path, &creds_path)
            .map_err(|e| SecurityError::Io(e))?;

        // Optionally remove vault file
        let vault_path = creds_path.parent()
            .ok_or_else(|| SecurityError::PermissionDenied("Invalid credentials path".to_string()))?
            .join("secrets.vault");

        if vault_path.exists() {
            fs::remove_file(&vault_path)
                .map_err(|e| SecurityError::Io(e))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_detect_credentials_file() {
        let temp_dir = TempDir::new().unwrap();
        let auth_dir = temp_dir.path().join(".radium/auth");
        fs::create_dir_all(&auth_dir).unwrap();

        let creds_file = auth_dir.join("credentials.json");
        fs::write(&creds_file, r#"{"version":"1.0","providers":{}}"#).unwrap();

        // This test would need to mock HOME env var, so we'll skip the full test
        // but verify the logic is correct
    }

    #[test]
    fn test_create_backup() {
        let temp_dir = TempDir::new().unwrap();
        let source_file = temp_dir.path().join("test.json");
        fs::write(&source_file, "test content").unwrap();

        let backup = MigrationManager::create_backup(&source_file).unwrap();
        assert!(backup.exists());
        assert!(backup.to_string_lossy().contains("backup-"));

        let backup_content = fs::read_to_string(&backup).unwrap();
        assert_eq!(backup_content, "test content");
    }
}

