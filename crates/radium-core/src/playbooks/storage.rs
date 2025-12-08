//! File storage operations for playbooks.

use crate::playbooks::error::{PlaybookError, Result};
use crate::playbooks::parser::PlaybookParser;
use crate::playbooks::types::Playbook;
use std::fs;
use std::path::{Path, PathBuf};

/// File storage for playbooks.
pub struct PlaybookStorage;

impl PlaybookStorage {
    /// Save a playbook to a file.
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be written or parent directory cannot be created.
    pub fn save(playbook: &Playbook, path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| PlaybookError::Io(e))?;
        }

        // Format playbook as YAML frontmatter + markdown
        let content = Self::format_playbook(playbook);

        // Write to file
        fs::write(path, content).map_err(|e| PlaybookError::LoadError {
            path: path.to_path_buf(),
            source: e,
        })?;

        Ok(())
    }

    /// Load a playbook from a file.
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be read or parsed.
    pub fn load(path: &Path) -> Result<Playbook> {
        PlaybookParser::parse_file(path)
    }

    /// Delete a playbook file.
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be deleted.
    pub fn delete(path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(PlaybookError::NotFound(
                path.display().to_string(),
            ));
        }

        fs::remove_file(path).map_err(|e| PlaybookError::Io(e))?;
        Ok(())
    }

    /// Format a playbook as YAML frontmatter + markdown content.
    fn format_playbook(playbook: &Playbook) -> String {
        use serde_yaml;

        // Build frontmatter
        let mut frontmatter = serde_yaml::to_string(&serde_json::json!({
            "uri": playbook.uri,
            "description": playbook.description,
            "tags": playbook.tags,
            "priority": playbook.priority.to_string(),
            "applies_to": playbook.applies_to,
        }))
        .unwrap_or_else(|_| String::new());

        // Combine frontmatter and content
        format!("---\n{}---\n\n{}", frontmatter, playbook.content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::playbooks::types::PlaybookPriority;
    use tempfile::TempDir;

    #[test]
    fn test_save_and_load_playbook() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test-playbook.md");

        let playbook = Playbook {
            uri: "radium://org/test.md".to_string(),
            description: "Test playbook".to_string(),
            tags: vec!["test".to_string()],
            priority: PlaybookPriority::Required,
            applies_to: vec!["requirement".to_string()],
            content: "# Test Content".to_string(),
        };

        PlaybookStorage::save(&playbook, &path).unwrap();
        let loaded = PlaybookStorage::load(&path).unwrap();

        assert_eq!(playbook.uri, loaded.uri);
        assert_eq!(playbook.description, loaded.description);
        assert_eq!(playbook.content, loaded.content);
    }

    #[test]
    fn test_delete_playbook() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test-playbook.md");

        let playbook = Playbook::new(
            "radium://org/test.md",
            "Test",
            "Content",
        );

        PlaybookStorage::save(&playbook, &path).unwrap();
        assert!(path.exists());

        PlaybookStorage::delete(&path).unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn test_delete_nonexistent_playbook() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("nonexistent.md");

        let result = PlaybookStorage::delete(&path);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PlaybookError::NotFound(_)));
    }
}

