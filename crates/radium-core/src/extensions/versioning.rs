//! Extension versioning and update management.
//!
//! Provides functionality for version comparison, constraint checking,
//! and update detection.

use crate::extensions::structure::Extension;
use semver::{Version, VersionReq};
use std::cmp::Ordering;
use std::path::Path;
use thiserror::Error;

/// Versioning errors.
#[derive(Debug, Error)]
pub enum VersioningError {
    /// Invalid version format.
    #[error("invalid version format: {0}")]
    InvalidVersion(String),

    /// Invalid version constraint.
    #[error("invalid version constraint: {0}")]
    InvalidConstraint(String),

    /// Version comparison error.
    #[error("version comparison error: {0}")]
    Comparison(String),
}

/// Result type for versioning operations.
pub type Result<T> = std::result::Result<T, VersioningError>;

/// Version comparator for extension versions.
pub struct VersionComparator;

impl VersionComparator {
    /// Parses a version string into a Version struct.
    ///
    /// # Arguments
    /// * `version_str` - Version string (e.g., "1.2.3")
    ///
    /// # Returns
    /// Parsed Version
    ///
    /// # Errors
    /// Returns error if version string is invalid
    pub fn parse(version_str: &str) -> Result<Version> {
        Version::parse(version_str)
            .map_err(|e| VersioningError::InvalidVersion(format!("{}: {}", version_str, e)))
    }

    /// Compares two version strings.
    ///
    /// # Arguments
    /// * `v1` - First version string
    /// * `v2` - Second version string
    ///
    /// # Returns
    /// Ordering (Less, Equal, Greater)
    ///
    /// # Errors
    /// Returns error if either version string is invalid
    pub fn compare(v1: &str, v2: &str) -> Result<Ordering> {
        let version1 = Self::parse(v1)?;
        let version2 = Self::parse(v2)?;
        Ok(version1.cmp(&version2))
    }

    /// Checks if a version satisfies a constraint.
    ///
    /// # Arguments
    /// * `version_str` - Version string to check
    /// * `constraint` - Version constraint (e.g., "^1.2.0", "~2.0.0", ">=1.0.0")
    ///
    /// # Returns
    /// True if version satisfies constraint
    ///
    /// # Errors
    /// Returns error if version or constraint is invalid
    pub fn is_compatible_with(version_str: &str, constraint: &str) -> Result<bool> {
        let version = Self::parse(version_str)?;
        let req = VersionReq::parse(constraint)
            .map_err(|e| VersioningError::InvalidConstraint(format!("{}: {}", constraint, e)))?;
        Ok(req.matches(&version))
    }

    /// Checks if a version is newer than another.
    ///
    /// # Arguments
    /// * `new_version` - New version string
    /// * `old_version` - Old version string
    ///
    /// # Returns
    /// True if new_version is newer than old_version
    ///
    /// # Errors
    /// Returns error if either version is invalid
    pub fn is_newer(new_version: &str, old_version: &str) -> Result<bool> {
        match Self::compare(new_version, old_version)? {
            Ordering::Greater => Ok(true),
            _ => Ok(false),
        }
    }
}

/// Information about an available update.
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    /// Extension name
    pub name: String,
    /// Current installed version
    pub current_version: String,
    /// Available new version
    pub new_version: String,
    /// Optional description of the update
    pub description: Option<String>,
    /// Optional download URL
    pub download_url: Option<String>,
}

/// Update checker for detecting available updates.
pub struct UpdateChecker;

impl UpdateChecker {
    /// Checks if an extension has an available update.
    ///
    /// # Arguments
    /// * `extension` - Currently installed extension
    /// * `new_version_str` - Version string of the new package
    ///
    /// # Returns
    /// True if new version is available (newer than current)
    ///
    /// # Errors
    /// Returns error if version comparison fails
    pub fn check_for_update(extension: &Extension, new_version_str: &str) -> Result<bool> {
        VersionComparator::is_newer(new_version_str, &extension.version)
    }

    /// Checks all installed extensions for available updates.
    ///
    /// # Arguments
    /// * `extensions` - List of installed extensions
    /// * `get_latest_version` - Function to get latest version for an extension
    ///
    /// # Returns
    /// Vector of UpdateInfo for extensions with available updates
    ///
    /// # Errors
    /// Returns error if version comparison fails
    pub fn check_all_updates<F>(extensions: &[Extension], get_latest_version: F) -> Result<Vec<UpdateInfo>>
    where
        F: Fn(&str) -> Option<(String, Option<String>, Option<String>)>, // (version, description, download_url)
    {
        let mut updates = Vec::new();

        for extension in extensions {
            if let Some((new_version, description, download_url)) = get_latest_version(&extension.name) {
                if Self::check_for_update(extension, &new_version)? {
                    updates.push(UpdateInfo {
                        name: extension.name.clone(),
                        current_version: extension.version.clone(),
                        new_version,
                        description,
                        download_url,
                    });
                }
            }
        }

        Ok(updates)
    }

    /// Validates that a new version is compatible with constraints.
    ///
    /// # Arguments
    /// * `new_version_str` - New version string
    /// * `constraint` - Version constraint (optional)
    ///
    /// # Returns
    /// True if version satisfies constraint (or no constraint provided)
    ///
    /// # Errors
    /// Returns error if version or constraint is invalid
    pub fn validate_constraint(new_version_str: &str, constraint: Option<&str>) -> Result<bool> {
        if let Some(constraint) = constraint {
            VersionComparator::is_compatible_with(new_version_str, constraint)
        } else {
            Ok(true)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parse() {
        assert!(VersionComparator::parse("1.0.0").is_ok());
        assert!(VersionComparator::parse("2.1.3").is_ok());
        assert!(VersionComparator::parse("1.0.0-beta").is_ok());
        assert!(VersionComparator::parse("invalid").is_err());
    }

    #[test]
    fn test_version_compare() {
        assert_eq!(
            VersionComparator::compare("1.0.0", "2.0.0").unwrap(),
            Ordering::Less
        );
        assert_eq!(
            VersionComparator::compare("2.0.0", "1.0.0").unwrap(),
            Ordering::Greater
        );
        assert_eq!(
            VersionComparator::compare("1.0.0", "1.0.0").unwrap(),
            Ordering::Equal
        );
        assert_eq!(
            VersionComparator::compare("1.0.0", "1.0.0-beta").unwrap(),
            Ordering::Greater
        );
    }

    #[test]
    fn test_version_is_newer() {
        assert!(VersionComparator::is_newer("2.0.0", "1.0.0").unwrap());
        assert!(!VersionComparator::is_newer("1.0.0", "2.0.0").unwrap());
        assert!(!VersionComparator::is_newer("1.0.0", "1.0.0").unwrap());
    }

    #[test]
    fn test_version_constraint() {
        // Caret constraint (^1.2.0 matches 1.2.0, 1.3.0, but not 2.0.0)
        assert!(VersionComparator::is_compatible_with("1.2.0", "^1.2.0").unwrap());
        assert!(VersionComparator::is_compatible_with("1.3.0", "^1.2.0").unwrap());
        assert!(!VersionComparator::is_compatible_with("2.0.0", "^1.2.0").unwrap());

        // Tilde constraint (~2.0.0 matches 2.0.0, 2.0.1, but not 2.1.0)
        assert!(VersionComparator::is_compatible_with("2.0.0", "~2.0.0").unwrap());
        assert!(VersionComparator::is_compatible_with("2.0.1", "~2.0.0").unwrap());
        assert!(!VersionComparator::is_compatible_with("2.1.0", "~2.0.0").unwrap());

        // Greater than or equal
        assert!(VersionComparator::is_compatible_with("1.0.0", ">=1.0.0").unwrap());
        assert!(VersionComparator::is_compatible_with("2.0.0", ">=1.0.0").unwrap());
        assert!(!VersionComparator::is_compatible_with("0.9.0", ">=1.0.0").unwrap());
    }

    #[test]
    fn test_update_checker() {
        use crate::extensions::manifest::{ExtensionComponents, ExtensionManifest};
        use std::collections::HashMap;
        use std::path::PathBuf;

        let manifest = ExtensionManifest {
            name: "test-ext".to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            author: "Test".to_string(),
            components: ExtensionComponents::default(),
            dependencies: Vec::new(),
            metadata: HashMap::new(),
        };

        let extension = Extension::new(manifest, PathBuf::from("/tmp"));

        assert!(UpdateChecker::check_for_update(&extension, "2.0.0").unwrap());
        assert!(!UpdateChecker::check_for_update(&extension, "1.0.0").unwrap());
        assert!(!UpdateChecker::check_for_update(&extension, "0.9.0").unwrap());
    }
}

