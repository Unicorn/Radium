//! Thread-safe registry for managing playbooks at runtime.

use crate::playbooks::discovery::PlaybookDiscovery;
use crate::playbooks::error::{PlaybookError, Result};
use crate::playbooks::types::Playbook;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Thread-safe playbook registry.
///
/// Provides in-memory storage and lookup for playbooks with support for
/// tag-based search and scope filtering.
pub struct PlaybookRegistry {
    /// In-memory playbook storage keyed by URI.
    playbooks: Arc<RwLock<HashMap<String, Playbook>>>,
}

impl PlaybookRegistry {
    /// Create a new empty playbook registry.
    pub fn new() -> Self {
        Self {
            playbooks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new playbook registry and discover playbooks from the default location.
    ///
    /// # Errors
    ///
    /// Returns error if discovery fails.
    pub fn discover() -> Result<Self> {
        let discovery = PlaybookDiscovery::new()?;
        let playbooks = discovery.discover_all()?;

        Ok(Self {
            playbooks: Arc::new(RwLock::new(playbooks)),
        })
    }

    /// Register a playbook in the registry.
    ///
    /// If a playbook with the same URI already exists, it will be replaced.
    ///
    /// # Errors
    ///
    /// Returns error if the registry lock cannot be acquired.
    pub fn register(&self, playbook: Playbook) -> Result<()> {
        let mut playbooks = self.playbooks.write().map_err(|e| {
            PlaybookError::InvalidConfig(format!("Failed to acquire write lock: {}", e))
        })?;

        playbooks.insert(playbook.uri.clone(), playbook);
        Ok(())
    }

    /// Get a playbook by URI.
    ///
    /// Returns `None` if no playbook with the given URI exists.
    ///
    /// # Errors
    ///
    /// Returns error if the registry lock cannot be acquired.
    pub fn get(&self, uri: &str) -> Result<Option<Playbook>> {
        let playbooks = self.playbooks.read().map_err(|e| {
            PlaybookError::InvalidConfig(format!("Failed to acquire read lock: {}", e))
        })?;

        Ok(playbooks.get(uri).cloned())
    }

    /// List all playbooks in the registry.
    ///
    /// # Errors
    ///
    /// Returns error if the registry lock cannot be acquired.
    pub fn list_all(&self) -> Result<Vec<Playbook>> {
        let playbooks = self.playbooks.read().map_err(|e| {
            PlaybookError::InvalidConfig(format!("Failed to acquire read lock: {}", e))
        })?;

        Ok(playbooks.values().cloned().collect())
    }

    /// Search playbooks by tags.
    ///
    /// Returns all playbooks that have any of the specified tags.
    /// If tags is empty, returns all playbooks.
    ///
    /// # Errors
    ///
    /// Returns error if the registry lock cannot be acquired.
    pub fn search_by_tags(&self, tags: &[String]) -> Result<Vec<Playbook>> {
        let playbooks = self.playbooks.read().map_err(|e| {
            PlaybookError::InvalidConfig(format!("Failed to acquire read lock: {}", e))
        })?;

        let matching: Vec<Playbook> = playbooks
            .values()
            .filter(|playbook| playbook.has_tags(tags))
            .cloned()
            .collect();

        Ok(matching)
    }

    /// Filter playbooks by scope (applies_to).
    ///
    /// Returns all playbooks that apply to the given scope.
    ///
    /// # Errors
    ///
    /// Returns error if the registry lock cannot be acquired.
    pub fn filter_by_scope(&self, scope: &str) -> Result<Vec<Playbook>> {
        let playbooks = self.playbooks.read().map_err(|e| {
            PlaybookError::InvalidConfig(format!("Failed to acquire read lock: {}", e))
        })?;

        let matching: Vec<Playbook> = playbooks
            .values()
            .filter(|playbook| playbook.applies_to_scope(scope))
            .cloned()
            .collect();

        Ok(matching)
    }

    /// Remove a playbook from the registry by URI.
    ///
    /// Returns `true` if a playbook was removed, `false` if it didn't exist.
    ///
    /// # Errors
    ///
    /// Returns error if the registry lock cannot be acquired.
    pub fn remove(&self, uri: &str) -> Result<bool> {
        let mut playbooks = self.playbooks.write().map_err(|e| {
            PlaybookError::InvalidConfig(format!("Failed to acquire write lock: {}", e))
        })?;

        Ok(playbooks.remove(uri).is_some())
    }

    /// Get the number of playbooks in the registry.
    ///
    /// # Errors
    ///
    /// Returns error if the registry lock cannot be acquired.
    pub fn len(&self) -> Result<usize> {
        let playbooks = self.playbooks.read().map_err(|e| {
            PlaybookError::InvalidConfig(format!("Failed to acquire read lock: {}", e))
        })?;

        Ok(playbooks.len())
    }

    /// Check if the registry is empty.
    ///
    /// # Errors
    ///
    /// Returns error if the registry lock cannot be acquired.
    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }
}

impl Default for PlaybookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::playbooks::types::PlaybookPriority;

    #[test]
    fn test_register_and_get() {
        let registry = PlaybookRegistry::new();

        let playbook = Playbook {
            uri: "radium://org/test.md".to_string(),
            description: "Test".to_string(),
            tags: vec![],
            priority: PlaybookPriority::Required,
            applies_to: vec![],
            content: "# Test".to_string(),
        };

        registry.register(playbook.clone()).unwrap();
        let retrieved = registry.get("radium://org/test.md").unwrap();

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().uri, playbook.uri);
    }

    #[test]
    fn test_search_by_tags() {
        let registry = PlaybookRegistry::new();

        let playbook1 = Playbook {
            uri: "radium://org/test1.md".to_string(),
            description: "Test 1".to_string(),
            tags: vec!["code-review".to_string()],
            priority: PlaybookPriority::Required,
            applies_to: vec![],
            content: "# Test 1".to_string(),
        };

        let playbook2 = Playbook {
            uri: "radium://org/test2.md".to_string(),
            description: "Test 2".to_string(),
            tags: vec!["security".to_string()],
            priority: PlaybookPriority::Recommended,
            applies_to: vec![],
            content: "# Test 2".to_string(),
        };

        registry.register(playbook1).unwrap();
        registry.register(playbook2).unwrap();

        let matching = registry.search_by_tags(&["code-review".to_string()]).unwrap();
        assert_eq!(matching.len(), 1);
        assert_eq!(matching[0].uri, "radium://org/test1.md");
    }

    #[test]
    fn test_filter_by_scope() {
        let registry = PlaybookRegistry::new();

        let playbook1 = Playbook {
            uri: "radium://org/test1.md".to_string(),
            description: "Test 1".to_string(),
            tags: vec![],
            priority: PlaybookPriority::Required,
            applies_to: vec!["requirement".to_string()],
            content: "# Test 1".to_string(),
        };

        let playbook2 = Playbook {
            uri: "radium://org/test2.md".to_string(),
            description: "Test 2".to_string(),
            tags: vec![],
            priority: PlaybookPriority::Recommended,
            applies_to: vec!["task".to_string()],
            content: "# Test 2".to_string(),
        };

        registry.register(playbook1).unwrap();
        registry.register(playbook2).unwrap();

        let matching = registry.filter_by_scope("requirement").unwrap();
        assert_eq!(matching.len(), 1);
        assert_eq!(matching[0].uri, "radium://org/test1.md");
    }

    #[test]
    fn test_remove() {
        let registry = PlaybookRegistry::new();

        let playbook = Playbook::new(
            "radium://org/test.md",
            "Test",
            "Content",
        );

        registry.register(playbook).unwrap();
        assert_eq!(registry.len().unwrap(), 1);

        let removed = registry.remove("radium://org/test.md").unwrap();
        assert!(removed);
        assert_eq!(registry.len().unwrap(), 0);
    }
}

