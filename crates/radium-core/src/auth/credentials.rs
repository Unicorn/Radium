//! Credential storage and retrieval system.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::error::{AuthError, AuthResult};
use super::providers::{Provider, ProviderType};

/// Current version of the credentials file format.
const CREDENTIALS_VERSION: &str = "1.0";

/// Credentials file structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CredentialsFile {
    /// Version of the credentials file format.
    version: String,
    /// Map of provider name to provider credentials.
    providers: HashMap<String, Provider>,
}

/// Manages credential storage and retrieval.
///
/// Credentials are stored in `~/.radium/auth/credentials.json` with file permissions
/// set to 0600 (read/write for owner only).
///
/// # Examples
///
/// ```no_run
/// use radium_core::auth::{CredentialStore, ProviderType};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let store = CredentialStore::new()?;
///
/// // Store a credential
/// store.store(ProviderType::Gemini, "your-api-key".to_string())?;
///
/// // Retrieve a credential
/// let api_key = store.get(ProviderType::Gemini)?;
///
/// // Remove a credential
/// store.remove(ProviderType::Gemini)?;
/// # Ok(())
/// # }
/// ```
pub struct CredentialStore {
    file_path: PathBuf,
}

impl CredentialStore {
    /// Creates a new credential store with the default path (`~/.radium/auth/credentials.json`).
    ///
    /// # Errors
    ///
    /// Returns an error if the HOME environment variable is not set.
    pub fn new() -> AuthResult<Self> {
        let file_path = Self::default_credentials_path()?;
        Ok(Self { file_path })
    }

    /// Creates a credential store with a custom path.
    ///
    /// This is primarily useful for testing with temporary directories.
    #[must_use]
    pub fn with_path(path: PathBuf) -> Self {
        Self { file_path: path }
    }

    /// Returns the default credentials file path.
    fn default_credentials_path() -> AuthResult<PathBuf> {
        // Allow env::var for HOME environment variable (path discovery)
        #[allow(clippy::disallowed_methods)]
        let home =
            std::env::var("HOME").map_err(|_| AuthError::PermissionDenied("HOME not set".to_string()))?;
        let auth_dir = Path::new(&home).join(".radium/auth");
        Ok(auth_dir.join("credentials.json"))
    }

    /// Ensures the auth directory exists with proper permissions.
    fn ensure_auth_dir(&self) -> AuthResult<()> {
        let dir = self.file_path.parent().ok_or(AuthError::InvalidFormat)?;

        if !dir.exists() {
            fs::create_dir_all(dir)?;
        }

        // Set directory permissions to 0700 (rwx------)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o700);
            fs::set_permissions(dir, perms)?;
        }

        Ok(())
    }

    /// Loads credentials from the file.
    ///
    /// If the file doesn't exist, returns an empty credentials structure.
    fn load(&self) -> AuthResult<CredentialsFile> {
        if !self.file_path.exists() {
            return Ok(CredentialsFile {
                version: CREDENTIALS_VERSION.to_string(),
                providers: HashMap::new(),
            });
        }

        let mut file = File::open(&self.file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let creds: CredentialsFile = serde_json::from_str(&contents)?;
        Ok(creds)
    }

    /// Saves credentials to the file with proper permissions.
    fn save(&self, creds: &CredentialsFile) -> AuthResult<()> {
        self.ensure_auth_dir()?;

        let json = serde_json::to_string_pretty(creds)?;
        let mut file = File::create(&self.file_path)?;
        file.write_all(json.as_bytes())?;

        // Set file permissions to 0600 (rw-------)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o600);
            fs::set_permissions(&self.file_path, perms)?;
        }

        Ok(())
    }

    /// Stores a credential for the specified provider.
    ///
    /// If a credential already exists for this provider, it will be overwritten.
    ///
    /// # Arguments
    ///
    /// * `provider_type` - The provider to store credentials for
    /// * `api_key` - The API key to store
    ///
    /// # Errors
    ///
    /// Returns an error if file operations fail or permissions cannot be set.
    pub fn store(&self, provider_type: ProviderType, api_key: String) -> AuthResult<()> {
        let mut creds = self.load()?;

        let provider = Provider {
            kind: provider_type,
            api_key,
            enabled: true,
            last_updated: time::OffsetDateTime::now_utc(),
        };

        creds.providers.insert(provider_type.as_str().to_string(), provider);
        self.save(&creds)?;

        Ok(())
    }

    /// Retrieves a credential for the specified provider.
    ///
    /// First checks the credentials file. If not found, falls back to environment variables.
    ///
    /// # Arguments
    ///
    /// * `provider_type` - The provider to retrieve credentials for
    ///
    /// # Returns
    ///
    /// The API key if found in either the file or environment variables.
    ///
    /// # Errors
    ///
    /// Returns `AuthError::CredentialNotFound` if the credential is not found in either location.
    pub fn get(&self, provider_type: ProviderType) -> AuthResult<String> {
        // First, try loading from file
        let creds = self.load()?;
        if let Some(provider) = creds.providers.get(provider_type.as_str()) {
            if provider.enabled {
                return Ok(provider.api_key.clone());
            }
        }

        // Fallback to environment variables
        // Allow env::var for API key environment variables (credential discovery)
        #[allow(clippy::disallowed_methods)]
        for env_var in provider_type.env_var_names() {
            if let Ok(key) = std::env::var(env_var) {
                return Ok(key);
            }
        }

        Err(AuthError::CredentialNotFound(provider_type.as_str().to_string()))
    }

    /// Removes a credential for the specified provider.
    ///
    /// # Arguments
    ///
    /// * `provider_type` - The provider to remove credentials for
    ///
    /// # Errors
    ///
    /// Returns an error if file operations fail.
    pub fn remove(&self, provider_type: ProviderType) -> AuthResult<()> {
        let mut creds = self.load()?;
        creds.providers.remove(provider_type.as_str());
        self.save(&creds)?;
        Ok(())
    }

    /// Lists all providers with stored credentials.
    ///
    /// Only returns providers that are enabled.
    ///
    /// # Errors
    ///
    /// Returns an error if file operations fail.
    pub fn list(&self) -> AuthResult<Vec<ProviderType>> {
        let creds = self.load()?;
        Ok(creds
            .providers
            .values()
            .filter(|p| p.enabled)
            .map(|p| p.kind)
            .collect())
    }

    /// Checks if a provider is configured.
    ///
    /// Checks both the credentials file and environment variables.
    ///
    /// # Arguments
    ///
    /// * `provider_type` - The provider to check
    ///
    /// # Returns
    ///
    /// `true` if credentials are available, `false` otherwise.
    #[must_use]
    pub fn is_configured(&self, provider_type: ProviderType) -> bool {
        self.get(provider_type).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_credential_store_new() {
        let store = CredentialStore::new();
        assert!(store.is_ok());
    }

    #[test]
    fn test_credential_store_with_path() {
        let temp_dir = TempDir::new().unwrap();
        let creds_path = temp_dir.path().join("credentials.json");
        let store = CredentialStore::with_path(creds_path.clone());
        assert_eq!(store.file_path, creds_path);
    }

    #[test]
    fn test_store_and_retrieve() {
        let temp_dir = TempDir::new().unwrap();
        let creds_path = temp_dir.path().join("credentials.json");
        let store = CredentialStore::with_path(creds_path);

        store.store(ProviderType::Gemini, "test-key".to_string()).unwrap();
        let key = store.get(ProviderType::Gemini).unwrap();
        assert_eq!(key, "test-key");
    }

    #[test]
    fn test_remove_credential() {
        let temp_dir = TempDir::new().unwrap();
        let creds_path = temp_dir.path().join("credentials.json");
        let store = CredentialStore::with_path(creds_path);

        store.store(ProviderType::OpenAI, "test-key".to_string()).unwrap();
        store.remove(ProviderType::OpenAI).unwrap();

        let result = store.get(ProviderType::OpenAI);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_credentials() {
        let temp_dir = TempDir::new().unwrap();
        let creds_path = temp_dir.path().join("credentials.json");
        let store = CredentialStore::with_path(creds_path);

        store.store(ProviderType::Gemini, "key1".to_string()).unwrap();
        store.store(ProviderType::OpenAI, "key2".to_string()).unwrap();

        let list = store.list().unwrap();
        assert_eq!(list.len(), 2);
        assert!(list.contains(&ProviderType::Gemini));
        assert!(list.contains(&ProviderType::OpenAI));
    }

    #[test]
    fn test_is_configured() {
        let temp_dir = TempDir::new().unwrap();
        let creds_path = temp_dir.path().join("credentials.json");
        let store = CredentialStore::with_path(creds_path);

        assert!(!store.is_configured(ProviderType::Gemini));

        store.store(ProviderType::Gemini, "test-key".to_string()).unwrap();
        assert!(store.is_configured(ProviderType::Gemini));
    }

    #[test]
    fn test_overwrite_credential() {
        let temp_dir = TempDir::new().unwrap();
        let creds_path = temp_dir.path().join("credentials.json");
        let store = CredentialStore::with_path(creds_path);

        store.store(ProviderType::Gemini, "old-key".to_string()).unwrap();
        store.store(ProviderType::Gemini, "new-key".to_string()).unwrap();

        let key = store.get(ProviderType::Gemini).unwrap();
        assert_eq!(key, "new-key");
    }

    #[test]
    fn test_empty_store() {
        let temp_dir = TempDir::new().unwrap();
        let creds_path = temp_dir.path().join("credentials.json");
        let store = CredentialStore::with_path(creds_path);

        let list = store.list().unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn test_credentials_file_format() {
        let temp_dir = TempDir::new().unwrap();
        let creds_path = temp_dir.path().join("credentials.json");
        let store = CredentialStore::with_path(creds_path.clone());

        store.store(ProviderType::Gemini, "test-key".to_string()).unwrap();

        // Read the file and verify format
        let mut file = File::open(&creds_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        assert!(contents.contains("\"version\""));
        assert!(contents.contains("\"providers\""));
        assert!(contents.contains("\"gemini\""));
        assert!(contents.contains("\"test-key\""));
    }
}
