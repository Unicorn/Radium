//! API key attribution tracking for cost attribution.

use crate::auth::{CredentialStore, ProviderType};
use sha2::{Digest, Sha256};
use std::fmt;

/// Attribution metadata derived from API key configuration.
#[derive(Debug, Clone)]
pub struct AttributionMetadata {
    /// API key identifier (hash of API key).
    pub api_key_id: String,
    /// Team name for cost attribution.
    pub team_name: Option<String>,
    /// Project name for cost attribution.
    pub project_name: Option<String>,
    /// Cost center for chargeback.
    pub cost_center: Option<String>,
}

impl AttributionMetadata {
    /// Creates attribution metadata from API key and provider type.
    ///
    /// Loads Provider metadata from CredentialStore and generates api_key_id.
    ///
    /// # Arguments
    /// * `api_key` - The API key used
    /// * `provider_type` - The provider type
    ///
    /// # Returns
    /// AttributionMetadata if provider is found, None if not found or error occurs.
    pub fn from_api_key(api_key: &str, provider_type: ProviderType) -> Option<Self> {
        // Generate api_key_id from API key (SHA256 hash of first 16 chars)
        let api_key_id = generate_api_key_id(api_key);

        // Try to load Provider metadata from CredentialStore
        let store = CredentialStore::new().ok()?;
        let creds = store.load().ok()?;
        let provider = creds.providers.get(provider_type.as_str())?;

        Some(Self {
            api_key_id,
            team_name: provider.team_name.clone(),
            project_name: provider.project_name.clone(),
            cost_center: provider.cost_center.clone(),
        })
    }

    /// Creates attribution metadata with only api_key_id (no metadata available).
    ///
    /// Used when Provider metadata is not configured.
    ///
    /// # Arguments
    /// * `api_key` - The API key used
    pub fn from_api_key_only(api_key: &str) -> Self {
        Self {
            api_key_id: generate_api_key_id(api_key),
            team_name: None,
            project_name: None,
            cost_center: None,
        }
    }
}

impl fmt::Display for AttributionMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "api_key_id={}", self.api_key_id)?;
        if let Some(ref team) = self.team_name {
            write!(f, ", team={}", team)?;
        }
        if let Some(ref project) = self.project_name {
            write!(f, ", project={}", project)?;
        }
        if let Some(ref center) = self.cost_center {
            write!(f, ", cost_center={}", center)?;
        }
        Ok(())
    }
}

/// Generates a consistent API key identifier from an API key.
///
    /// Uses SHA256 hash of the first 16 characters of the API key.
    /// This provides a consistent identifier without storing the actual key.
    ///
    /// # Arguments
    /// * `api_key` - The API key to generate an ID for
    ///
    /// # Returns
    /// A hex-encoded hash string (32 characters)
pub fn generate_api_key_id(api_key: &str) -> String {
    // Take first 16 chars for hashing (or full key if shorter)
    let key_prefix = if api_key.len() >= 16 {
        &api_key[..16]
    } else {
        api_key
    };

    let mut hasher = Sha256::new();
    hasher.update(key_prefix.as_bytes());
    let hash = hasher.finalize();
    
    // Return first 16 hex characters (8 bytes) for a shorter, readable ID
    format!("{:x}", hash)[..16].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_api_key_id_consistency() {
        let api_key = "sk-test12345678901234567890";
        let id1 = generate_api_key_id(api_key);
        let id2 = generate_api_key_id(api_key);
        assert_eq!(id1, id2);
        assert_eq!(id1.len(), 16);
    }

    #[test]
    fn test_generate_api_key_id_different_keys() {
        let id1 = generate_api_key_id("sk-key1");
        let id2 = generate_api_key_id("sk-key2");
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_attribution_metadata_from_api_key_only() {
        let metadata = AttributionMetadata::from_api_key_only("sk-test");
        assert!(!metadata.api_key_id.is_empty());
        assert_eq!(metadata.team_name, None);
        assert_eq!(metadata.project_name, None);
        assert_eq!(metadata.cost_center, None);
    }

    #[test]
    fn test_attribution_metadata_display() {
        let metadata = AttributionMetadata {
            api_key_id: "abc123".to_string(),
            team_name: Some("backend".to_string()),
            project_name: Some("api-v2".to_string()),
            cost_center: None,
        };
        let display = format!("{}", metadata);
        assert!(display.contains("api_key_id=abc123"));
        assert!(display.contains("team=backend"));
        assert!(display.contains("project=api-v2"));
    }
}

