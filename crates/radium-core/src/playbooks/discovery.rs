//! Playbook discovery from file system.

use crate::playbooks::error::{PlaybookError, Result};
use crate::playbooks::parser::PlaybookParser;
use crate::playbooks::types::Playbook;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Playbook discovery service.
///
/// Scans directories for playbook files (*.md) and loads them.
pub struct PlaybookDiscovery {
    /// Search paths for playbooks.
    search_paths: Vec<PathBuf>,
}

impl PlaybookDiscovery {
    /// Create a new playbook discovery service with default search paths.
    ///
    /// Default: `~/.radium/playbooks/`
    pub fn new() -> Result<Self> {
        let default_path = Self::default_playbooks_dir()?;
        Ok(Self {
            search_paths: vec![default_path],
        })
    }

    /// Create a new playbook discovery service with custom search paths.
    pub fn with_paths(search_paths: Vec<PathBuf>) -> Self {
        Self { search_paths }
    }

    /// Get the default playbooks directory (`~/.radium/playbooks/`).
    pub fn default_playbooks_dir() -> Result<PathBuf> {
        #[allow(clippy::disallowed_methods)]
        let home = std::env::var("HOME")
            .map_err(|_| PlaybookError::InvalidConfig("HOME environment variable not set".to_string()))?;
        Ok(PathBuf::from(home).join(".radium").join("playbooks"))
    }

    /// Discover all playbooks in the configured search paths.
    ///
    /// # Errors
    ///
    /// Returns error if discovery fails, but continues scanning even if individual
    /// playbooks fail to load (they are logged and skipped).
    pub fn discover_all(&self) -> Result<HashMap<String, Playbook>> {
        let mut playbooks = HashMap::new();

        for search_path in &self.search_paths {
            if !search_path.exists() {
                continue;
            }

            self.discover_in_directory(search_path, &mut playbooks)?;
        }

        Ok(playbooks)
    }

    /// Discover playbooks in a specific directory.
    fn discover_in_directory(
        &self,
        dir: &Path,
        playbooks: &mut HashMap<String, Playbook>,
    ) -> Result<()> {
        for entry in WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            if path.is_file() {
                // Check if this is a markdown file
                if let Some(ext) = path.extension() {
                    if ext == "md" || ext == "markdown" {
                        // Continue discovery even if a playbook fails to load
                        if let Err(e) = self.load_playbook(path, playbooks) {
                            tracing::debug!(
                                path = %path.display(),
                                error = %e,
                                "Skipping invalid playbook file"
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Load a playbook from a file.
    fn load_playbook(
        &self,
        path: &Path,
        playbooks: &mut HashMap<String, Playbook>,
    ) -> Result<()> {
        let playbook = PlaybookParser::parse_file(path)?;

        // Use URI as the key
        playbooks.insert(playbook.uri.clone(), playbook);

        Ok(())
    }

    /// Find a playbook by URI.
    ///
    /// # Errors
    ///
    /// Returns error if discovery fails.
    pub fn find_by_uri(&self, uri: &str) -> Result<Option<Playbook>> {
        let all = self.discover_all()?;
        Ok(all.get(uri).cloned())
    }

    /// Find playbooks by tags.
    ///
    /// Returns all playbooks that have any of the specified tags.
    ///
    /// # Errors
    ///
    /// Returns error if discovery fails.
    pub fn find_by_tags(&self, tags: &[String]) -> Result<Vec<Playbook>> {
        let all = self.discover_all()?;
        let mut matching = Vec::new();

        for playbook in all.values() {
            if playbook.has_tags(tags) {
                matching.push(playbook.clone());
            }
        }

        Ok(matching)
    }

    /// Find playbooks by scope (applies_to).
    ///
    /// Returns all playbooks that apply to the given scope.
    ///
    /// # Errors
    ///
    /// Returns error if discovery fails.
    pub fn find_by_scope(&self, scope: &str) -> Result<Vec<Playbook>> {
        let all = self.discover_all()?;
        let mut matching = Vec::new();

        for playbook in all.values() {
            if playbook.applies_to_scope(scope) {
                matching.push(playbook.clone());
            }
        }

        Ok(matching)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::playbooks::storage::PlaybookStorage;
    use crate::playbooks::types::PlaybookPriority;
    use tempfile::TempDir;

    #[test]
    fn test_discover_all_playbooks() {
        let temp_dir = TempDir::new().unwrap();
        let playbooks_dir = temp_dir.path().join("playbooks");
        std::fs::create_dir_all(&playbooks_dir).unwrap();

        let playbook1 = Playbook {
            uri: "radium://org/test1.md".to_string(),
            description: "Test 1".to_string(),
            tags: vec!["test".to_string()],
            priority: PlaybookPriority::Required,
            applies_to: vec!["requirement".to_string()],
            content: "# Test 1".to_string(),
        };

        let playbook2 = Playbook {
            uri: "radium://org/test2.md".to_string(),
            description: "Test 2".to_string(),
            tags: vec!["test".to_string(), "code-review".to_string()],
            priority: PlaybookPriority::Recommended,
            applies_to: vec!["task".to_string()],
            content: "# Test 2".to_string(),
        };

        PlaybookStorage::save(&playbook1, &playbooks_dir.join("test1.md")).unwrap();
        PlaybookStorage::save(&playbook2, &playbooks_dir.join("test2.md")).unwrap();

        let discovery = PlaybookDiscovery::with_paths(vec![playbooks_dir]);
        let playbooks = discovery.discover_all().unwrap();

        assert_eq!(playbooks.len(), 2);
        assert!(playbooks.contains_key("radium://org/test1.md"));
        assert!(playbooks.contains_key("radium://org/test2.md"));
    }

    #[test]
    fn test_find_by_tags() {
        let temp_dir = TempDir::new().unwrap();
        let playbooks_dir = temp_dir.path().join("playbooks");
        std::fs::create_dir_all(&playbooks_dir).unwrap();

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

        PlaybookStorage::save(&playbook1, &playbooks_dir.join("test1.md")).unwrap();
        PlaybookStorage::save(&playbook2, &playbooks_dir.join("test2.md")).unwrap();

        let discovery = PlaybookDiscovery::with_paths(vec![playbooks_dir]);
        let matching = discovery.find_by_tags(&["code-review".to_string()]).unwrap();

        assert_eq!(matching.len(), 1);
        assert_eq!(matching[0].uri, "radium://org/test1.md");
    }

    #[test]
    fn test_find_by_scope() {
        let temp_dir = TempDir::new().unwrap();
        let playbooks_dir = temp_dir.path().join("playbooks");
        std::fs::create_dir_all(&playbooks_dir).unwrap();

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

        PlaybookStorage::save(&playbook1, &playbooks_dir.join("test1.md")).unwrap();
        PlaybookStorage::save(&playbook2, &playbooks_dir.join("test2.md")).unwrap();

        let discovery = PlaybookDiscovery::with_paths(vec![playbooks_dir]);
        let matching = discovery.find_by_scope("requirement").unwrap();

        assert_eq!(matching.len(), 1);
        assert_eq!(matching[0].uri, "radium://org/test1.md");
    }
}

