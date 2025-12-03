//! Requirement ID management for plans.
//!
//! Implements the REQ-XXX ID format used for plan identification,
//! with auto-incrementing counter stored in `.radium/requirement-counter.json`.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use thiserror::Error;

/// Requirement ID errors.
#[derive(Debug, Error)]
pub enum RequirementIdError {
    /// Invalid requirement ID format.
    #[error("invalid requirement ID format: {0}")]
    InvalidFormat(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Counter file not found.
    #[error("counter file not found")]
    CounterNotFound,
}

/// Result type for requirement ID operations.
pub type Result<T> = std::result::Result<T, RequirementIdError>;

/// Requirement ID in REQ-XXX format.
///
/// # Example
///
/// ```
/// use radium_core::workspace::RequirementId;
/// use std::str::FromStr;
///
/// let id = RequirementId::from_str("REQ-001").unwrap();
/// assert_eq!(id.number(), 1);
/// assert_eq!(id.to_string(), "REQ-001");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RequirementId {
    number: u32,
}

impl RequirementId {
    /// Create a new requirement ID from a number.
    ///
    /// # Example
    ///
    /// ```
    /// use radium_core::workspace::RequirementId;
    ///
    /// let id = RequirementId::new(42);
    /// assert_eq!(id.to_string(), "REQ-042");
    /// ```
    pub fn new(number: u32) -> Self {
        Self { number }
    }

    /// Get the numeric part of the requirement ID.
    pub fn number(&self) -> u32 {
        self.number
    }

    /// Generate the next requirement ID from the workspace counter.
    ///
    /// Reads the counter from `.radium/requirement-counter.json`,
    /// increments it, and saves it back.
    ///
    /// # Errors
    ///
    /// Returns error if counter file cannot be read or written.
    pub fn next(radium_dir: impl AsRef<Path>) -> Result<Self> {
        let counter_path = radium_dir.as_ref().join("requirement-counter.json");

        // Read current counter or start at 1
        let current = if counter_path.exists() {
            let content = fs::read_to_string(&counter_path)?;
            let counter: RequirementCounter = serde_json::from_str(&content)?;
            counter.next
        } else {
            1
        };

        // Create new ID
        let id = Self::new(current);

        // Save incremented counter
        let counter = RequirementCounter { next: current + 1 };
        let content = serde_json::to_string_pretty(&counter)?;
        fs::write(&counter_path, content)?;

        Ok(id)
    }

    /// Get the current counter value without incrementing.
    ///
    /// # Errors
    ///
    /// Returns error if counter file cannot be read.
    pub fn current(radium_dir: impl AsRef<Path>) -> Result<u32> {
        let counter_path = radium_dir.as_ref().join("requirement-counter.json");

        if counter_path.exists() {
            let content = fs::read_to_string(&counter_path)?;
            let counter: RequirementCounter = serde_json::from_str(&content)?;
            Ok(counter.next - 1)
        } else {
            Ok(0)
        }
    }

    /// Initialize the counter file with a specific value.
    ///
    /// # Errors
    ///
    /// Returns error if counter file cannot be written.
    pub fn init_counter(radium_dir: impl AsRef<Path>, next: u32) -> Result<()> {
        let counter_path = radium_dir.as_ref().join("requirement-counter.json");
        let counter = RequirementCounter { next };
        let content = serde_json::to_string_pretty(&counter)?;
        fs::write(&counter_path, content)?;
        Ok(())
    }
}

impl fmt::Display for RequirementId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "REQ-{:03}", self.number)
    }
}

impl FromStr for RequirementId {
    type Err = RequirementIdError;

    fn from_str(s: &str) -> Result<Self> {
        if !s.starts_with("REQ-") {
            return Err(RequirementIdError::InvalidFormat("ID must start with 'REQ-'".to_string()));
        }

        let number_str = &s[4..];
        let number = number_str.parse::<u32>().map_err(|_| {
            RequirementIdError::InvalidFormat(format!("invalid number: {}", number_str))
        })?;

        Ok(Self::new(number))
    }
}

/// Counter file format.
#[derive(Debug, Serialize, Deserialize)]
struct RequirementCounter {
    /// Next requirement ID number to use.
    next: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_requirement_id_new() {
        let id = RequirementId::new(1);
        assert_eq!(id.number(), 1);
        assert_eq!(id.to_string(), "REQ-001");

        let id = RequirementId::new(42);
        assert_eq!(id.number(), 42);
        assert_eq!(id.to_string(), "REQ-042");

        let id = RequirementId::new(999);
        assert_eq!(id.number(), 999);
        assert_eq!(id.to_string(), "REQ-999");
    }

    #[test]
    fn test_requirement_id_from_str() {
        let id = RequirementId::from_str("REQ-001").unwrap();
        assert_eq!(id.number(), 1);

        let id = RequirementId::from_str("REQ-042").unwrap();
        assert_eq!(id.number(), 42);

        let id = RequirementId::from_str("REQ-999").unwrap();
        assert_eq!(id.number(), 999);
    }

    #[test]
    fn test_requirement_id_from_str_invalid() {
        assert!(RequirementId::from_str("REQ").is_err());
        assert!(RequirementId::from_str("REQ-").is_err());
        assert!(RequirementId::from_str("REQ-abc").is_err());
        assert!(RequirementId::from_str("INVALID").is_err());
        assert!(RequirementId::from_str("001").is_err());
    }

    #[test]
    fn test_requirement_id_ordering() {
        let id1 = RequirementId::new(1);
        let id2 = RequirementId::new(2);
        let id3 = RequirementId::new(10);

        assert!(id1 < id2);
        assert!(id2 < id3);
        assert!(id1 < id3);
    }

    #[test]
    fn test_requirement_id_next() {
        let temp = TempDir::new().unwrap();

        // First ID should be REQ-001
        let id1 = RequirementId::next(temp.path()).unwrap();
        assert_eq!(id1, RequirementId::new(1));
        assert_eq!(id1.to_string(), "REQ-001");

        // Second ID should be REQ-002
        let id2 = RequirementId::next(temp.path()).unwrap();
        assert_eq!(id2, RequirementId::new(2));
        assert_eq!(id2.to_string(), "REQ-002");

        // Third ID should be REQ-003
        let id3 = RequirementId::next(temp.path()).unwrap();
        assert_eq!(id3, RequirementId::new(3));
        assert_eq!(id3.to_string(), "REQ-003");
    }

    #[test]
    fn test_requirement_id_current() {
        let temp = TempDir::new().unwrap();

        // No counter file yet
        let current = RequirementId::current(temp.path()).unwrap();
        assert_eq!(current, 0);

        // Generate some IDs
        RequirementId::next(temp.path()).unwrap();
        RequirementId::next(temp.path()).unwrap();

        // Current should be 2 (last generated ID)
        let current = RequirementId::current(temp.path()).unwrap();
        assert_eq!(current, 2);
    }

    #[test]
    fn test_requirement_id_init_counter() {
        let temp = TempDir::new().unwrap();

        // Initialize counter to start at 100
        RequirementId::init_counter(temp.path(), 100).unwrap();

        // Next ID should be REQ-100
        let id = RequirementId::next(temp.path()).unwrap();
        assert_eq!(id, RequirementId::new(100));
        assert_eq!(id.to_string(), "REQ-100");
    }

    #[test]
    fn test_requirement_id_persistence() {
        let temp = TempDir::new().unwrap();

        // Generate IDs
        RequirementId::next(temp.path()).unwrap();
        RequirementId::next(temp.path()).unwrap();

        // Counter file should persist
        let counter_path = temp.path().join("requirement-counter.json");
        assert!(counter_path.exists());

        // Read and verify counter file
        let content = fs::read_to_string(&counter_path).unwrap();
        let counter: RequirementCounter = serde_json::from_str(&content).unwrap();
        assert_eq!(counter.next, 3);
    }

    #[test]
    fn test_requirement_id_serialize() {
        let id = RequirementId::new(42);
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: RequirementId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
    }
}
