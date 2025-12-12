//! Braingrid integration for playbook storage and synchronization.

use crate::context::braingrid_client::BraingridClient;
use crate::playbooks::error::{PlaybookError, Result};
use crate::playbooks::types::Playbook;
use std::collections::HashMap;

/// Braingrid storage backend for playbooks.
///
/// Note: This is a placeholder implementation. Braingrid may not have native
/// playbook support yet. This structure can be extended when Braingrid adds
/// playbook resource types or custom resource support.
pub struct BraingridPlaybookStorage {
    /// Braingrid client for API operations.
    client: BraingridClient,
    /// Project ID for Braingrid operations.
    project_id: String,
}

impl BraingridPlaybookStorage {
    /// Create a new Braingrid playbook storage.
    pub fn new(project_id: impl Into<String>) -> Self {
        let project_id = project_id.into();
        Self {
            client: BraingridClient::new(project_id.clone()),
            project_id,
        }
    }

    /// Fetch a playbook from Braingrid by URI.
    ///
    /// # Errors
    ///
    /// Returns error if playbook cannot be fetched or parsed.
    ///
    /// # Note
    ///
    /// This is a placeholder implementation. Actual implementation depends on
    /// Braingrid API support for playbooks or custom resources.
    pub async fn fetch_playbook(&self, uri: &str) -> Result<Playbook> {
        // TODO: Implement when Braingrid adds playbook support
        // For now, this is a placeholder that returns an error
        Err(PlaybookError::NotFound(format!(
            "Braingrid playbook support not yet implemented. URI: {}",
            uri
        )))
    }

    /// List all playbooks from Braingrid.
    ///
    /// # Errors
    ///
    /// Returns error if playbooks cannot be listed.
    ///
    /// # Note
    ///
    /// This is a placeholder implementation.
    pub async fn list_playbooks(&self) -> Result<Vec<Playbook>> {
        // TODO: Implement when Braingrid adds playbook support
        Ok(Vec::new())
    }

    /// Search playbooks by tags in Braingrid.
    ///
    /// # Errors
    ///
    /// Returns error if search fails.
    ///
    /// # Note
    ///
    /// This is a placeholder implementation.
    pub async fn search_playbooks(&self, tags: &[String]) -> Result<Vec<Playbook>> {
        // TODO: Implement when Braingrid adds playbook support
        Ok(Vec::new())
    }

    /// Save a playbook to Braingrid.
    ///
    /// # Errors
    ///
    /// Returns error if playbook cannot be saved.
    ///
    /// # Note
    ///
    /// This is a placeholder implementation.
    pub async fn save_playbook(&self, playbook: &Playbook) -> Result<()> {
        // TODO: Implement when Braingrid adds playbook support
        Err(PlaybookError::InvalidConfig(
            "Braingrid playbook write support not yet implemented".to_string(),
        ))
    }

    /// Delete a playbook from Braingrid by URI.
    ///
    /// # Errors
    ///
    /// Returns error if playbook cannot be deleted.
    ///
    /// # Note
    ///
    /// This is a placeholder implementation.
    pub async fn delete_playbook(&self, uri: &str) -> Result<()> {
        // TODO: Implement when Braingrid adds playbook support
        Err(PlaybookError::NotFound(format!(
            "Braingrid playbook delete support not yet implemented. URI: {}",
            uri
        )))
    }
}

/// Merge local and remote playbooks.
///
/// Local playbooks take precedence over remote playbooks when URIs conflict.
pub fn merge_playbooks(
    local: HashMap<String, Playbook>,
    remote: Vec<Playbook>,
) -> HashMap<String, Playbook> {
    let mut merged = local;

    for remote_playbook in remote {
        // Only add remote playbook if local doesn't have it
        if !merged.contains_key(&remote_playbook.uri) {
            merged.insert(remote_playbook.uri.clone(), remote_playbook);
        }
    }

    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::playbooks::types::PlaybookPriority;

    #[test]
    fn test_merge_playbooks_local_precedence() {
        let mut local = HashMap::new();
        local.insert(
            "radium://org/test.md".to_string(),
            Playbook {
                uri: "radium://org/test.md".to_string(),
                description: "Local version".to_string(),
                tags: vec![],
                priority: PlaybookPriority::Required,
                applies_to: vec![],
                content: "# Local".to_string(),
            },
        );

        let remote = vec![
            Playbook {
                uri: "radium://org/test.md".to_string(),
                description: "Remote version".to_string(),
                tags: vec![],
                priority: PlaybookPriority::Recommended,
                applies_to: vec![],
                content: "# Remote".to_string(),
            },
            Playbook {
                uri: "radium://org/new.md".to_string(),
                description: "New remote".to_string(),
                tags: vec![],
                priority: PlaybookPriority::Optional,
                applies_to: vec![],
                content: "# New".to_string(),
            },
        ];

        let merged = merge_playbooks(local, remote);

        // Local version should be kept
        assert_eq!(merged.get("radium://org/test.md").unwrap().description, "Local version");
        // Remote-only playbook should be added
        assert!(merged.contains_key("radium://org/new.md"));
    }
}

