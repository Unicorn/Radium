//! Learning store for categorized mistake tracking.
//!
//! Tracks mistakes, solutions, and preferences in categories to build
//! pattern recognition for future oversight improvements.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Type of learning entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LearningType {
    /// A mistake that was made and corrected.
    Mistake,
    /// A user preference or constraint.
    Preference,
    /// A successful pattern or approach.
    Success,
}

/// Learning entry for a mistake, preference, or success.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningEntry {
    /// Type of learning entry.
    pub entry_type: LearningType,
    /// Category of the learning entry.
    pub category: String,
    /// Description of the mistake, preference, or success.
    pub description: String,
    /// Solution or explanation (required for mistakes, optional for others).
    pub solution: Option<String>,
    /// Timestamp when this entry was created.
    pub timestamp: DateTime<Utc>,
}

impl LearningEntry {
    /// Creates a new learning entry.
    pub fn new(
        entry_type: LearningType,
        category: String,
        description: String,
        solution: Option<String>,
    ) -> Self {
        Self {
            entry_type,
            category,
            description: Self::enforce_one_sentence(description),
            solution: solution.map(Self::enforce_one_sentence),
            timestamp: Utc::now(),
        }
    }

    /// Ensures text is a single sentence.
    fn enforce_one_sentence(text: String) -> String {
        // Remove newlines
        let mut sentence = text.replace('\n', " ").replace('\r', " ");

        // Split by sentence-ending punctuation
        let parts: Vec<&str> = sentence.split(&['.', '!', '?'][..]).collect();
        if let Some(first) = parts.first() {
            sentence = first.trim().to_string();
        }

        // Ensure it ends with sentence-ending punctuation
        if !sentence.ends_with(&['.', '!', '?'][..]) {
            sentence.push('.');
        }

        sentence
    }
}

/// Category data for learning entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CategoryData {
    /// Count of entries in this category.
    count: usize,
    /// Examples of entries in this category.
    examples: Vec<LearningEntry>,
    /// Last update timestamp.
    last_updated: DateTime<Utc>,
}

impl CategoryData {
    fn new() -> Self {
        Self { count: 0, examples: Vec::new(), last_updated: Utc::now() }
    }
}

/// Learning log structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LearningLog {
    /// Map of category to category data.
    categories: HashMap<String, CategoryData>,
    /// Last update timestamp.
    last_updated: DateTime<Utc>,
}

impl LearningLog {
    fn new() -> Self {
        Self { categories: HashMap::new(), last_updated: Utc::now() }
    }
}

/// Standard mistake categories.
pub const STANDARD_CATEGORIES: &[&str] = &[
    "Complex Solution Bias",
    "Feature Creep",
    "Premature Implementation",
    "Misalignment",
    "Overtooling",
    "Preference",
    "Success",
    "Other",
];

/// Errors that can occur during learning operations.
#[derive(Error, Debug)]
pub enum LearningError {
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Invalid learning entry.
    #[error("Invalid learning entry: {0}")]
    InvalidEntry(String),
}

/// Result type for learning operations.
pub type Result<T> = std::result::Result<T, LearningError>;

/// Learning store for tracking mistakes and solutions.
///
/// Stores learning entries in a JSON file, organized by category.
pub struct LearningStore {
    /// Path to the learning log file.
    log_path: PathBuf,
    /// In-memory cache of learning log.
    log: LearningLog,
}

impl LearningStore {
    /// Creates a new learning store.
    ///
    /// # Arguments
    /// * `data_dir` - Directory to store the learning log
    ///
    /// # Returns
    /// A new learning store instance
    ///
    /// # Errors
    /// Returns error if directory creation or file loading fails
    pub fn new(data_dir: impl AsRef<Path>) -> Result<Self> {
        let data_dir = data_dir.as_ref();
        fs::create_dir_all(data_dir)?;

        let log_path = data_dir.join("learning-log.json");
        let log = if log_path.exists() {
            Self::load_log(&log_path)?
        } else {
            let log = LearningLog::new();
            Self::save_log(&log_path, &log)?;
            log
        };

        Ok(Self { log_path, log })
    }

    /// Adds a learning entry.
    ///
    /// # Arguments
    /// * `entry_type` - Type of learning entry
    /// * `category` - Category for the entry
    /// * `description` - Description of the mistake/preference/success
    /// * `solution` - Solution or explanation (required for mistakes)
    ///
    /// # Returns
    /// The created learning entry and whether it was added (false if duplicate)
    ///
    /// # Errors
    /// Returns error if entry is invalid or save fails
    pub fn add_entry(
        &mut self,
        entry_type: LearningType,
        category: String,
        description: String,
        solution: Option<String>,
    ) -> Result<(LearningEntry, bool)> {
        // Validate entry
        if description.is_empty() {
            return Err(LearningError::InvalidEntry("Description cannot be empty".to_string()));
        }

        if entry_type == LearningType::Mistake && solution.is_none() {
            return Err(LearningError::InvalidEntry(
                "Solution is required for mistake entries".to_string(),
            ));
        }

        // Normalize category
        let category = Self::normalize_category(&category);

        // Create entry
        let entry = LearningEntry::new(entry_type, category.clone(), description, solution);

        // Check for similar entries to avoid duplicates
        let existing = self.get_entries_by_category(&category);
        let is_duplicate = existing.iter().any(|e| Self::is_similar(&e.description, &entry.description));

        if is_duplicate {
            return Ok((entry, false));
        }

        // Add to category
        let category_data = self.log.categories.entry(category.clone()).or_insert_with(CategoryData::new);
        category_data.count += 1;
        category_data.examples.push(entry.clone());
        category_data.last_updated = Utc::now();
        self.log.last_updated = Utc::now();

        // Save to disk
        Self::save_log(&self.log_path, &self.log)?;

        Ok((entry, true))
    }

    /// Gets all learning entries for a category.
    ///
    /// # Arguments
    /// * `category` - The category to get entries for
    ///
    /// # Returns
    /// Vector of learning entries for the category
    pub fn get_entries_by_category(&self, category: &str) -> Vec<LearningEntry> {
        self.log
            .categories
            .get(category)
            .map(|data| data.examples.clone())
            .unwrap_or_default()
    }

    /// Gets all learning entries grouped by category.
    ///
    /// # Returns
    /// Map of category to entries
    pub fn get_all_entries(&self) -> HashMap<String, Vec<LearningEntry>> {
        self.log
            .categories
            .iter()
            .map(|(category, data)| (category.clone(), data.examples.clone()))
            .collect()
    }

    /// Gets category summaries sorted by count (most frequent first).
    ///
    /// # Returns
    /// Vector of category summaries with count and most recent example
    pub fn get_category_summary(&self) -> Vec<CategorySummary> {
        let mut summaries: Vec<CategorySummary> = self
            .log
            .categories
            .iter()
            .map(|(category, data)| {
                let recent_example = data.examples.last().cloned();
                CategorySummary {
                    category: category.clone(),
                    count: data.count,
                    recent_example,
                }
            })
            .collect();

        summaries.sort_by(|a, b| b.count.cmp(&a.count));
        summaries
    }

    /// Generates learning context text for oversight prompts.
    ///
    /// # Arguments
    /// * `max_per_category` - Maximum examples per category to include
    ///
    /// # Returns
    /// Formatted learning context string
    pub fn generate_context(&self, max_per_category: usize) -> String {
        let mut context = String::new();

        for (category, data) in &self.log.categories {
            context.push_str(&format!("Category: {} (count: {})\n", category, data.count));

            let examples: Vec<&LearningEntry> = data
                .examples
                .iter()
                .rev()
                .take(max_per_category)
                .collect();

            for entry in examples {
                let label = match entry.entry_type {
                    LearningType::Mistake => "Mistake",
                    LearningType::Preference => "Preference",
                    LearningType::Success => "Success",
                };

                let solution_text = entry
                    .solution
                    .as_ref()
                    .map(|s| format!(" | Solution: {}", s))
                    .unwrap_or_default();

                context.push_str(&format!(
                    "- [{}] {}: {}{}\n",
                    entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    label,
                    entry.description,
                    solution_text
                ));
            }

            context.push('\n');
        }

        context.trim().to_string()
    }

    /// Normalizes a category name to a standard category if possible.
    fn normalize_category(category: &str) -> String {
        let lower = category.to_lowercase();

        // Map keywords to standard categories
        let mappings: Vec<(&str, &str)> = vec![
            ("complex", "Complex Solution Bias"),
            ("complicated", "Complex Solution Bias"),
            ("over-engineered", "Complex Solution Bias"),
            ("complexity", "Complex Solution Bias"),
            ("feature", "Feature Creep"),
            ("extra", "Feature Creep"),
            ("additional", "Feature Creep"),
            ("scope creep", "Feature Creep"),
            ("premature", "Premature Implementation"),
            ("early", "Premature Implementation"),
            ("jumping", "Premature Implementation"),
            ("too quick", "Premature Implementation"),
            ("misaligned", "Misalignment"),
            ("wrong direction", "Misalignment"),
            ("off target", "Misalignment"),
            ("misunderstood", "Misalignment"),
            ("overtool", "Overtooling"),
            ("too many tools", "Overtooling"),
            ("unnecessary tools", "Overtooling"),
        ];

        for (keyword, standard) in mappings {
            if lower.contains(keyword) {
                return standard.to_string();
            }
        }

        // Return original if no match
        category.to_string()
    }

    /// Checks if two descriptions are similar (simple word overlap).
    fn is_similar(a: &str, b: &str) -> bool {
        let a_words: Vec<&str> = a.to_lowercase().split_whitespace().collect();
        let b_words: Vec<&str> = b.to_lowercase().split_whitespace().collect();

        if a_words.is_empty() || b_words.is_empty() {
            return false;
        }

        let overlap: usize = a_words.iter().filter(|w| b_words.contains(w)).count();
        let min_len = a_words.len().min(b_words.len());
        let ratio = overlap as f64 / min_len as f64;

        ratio >= 0.6
    }

    /// Loads the learning log from disk.
    fn load_log(path: &Path) -> Result<LearningLog> {
        let content = fs::read_to_string(path)?;
        let log: LearningLog = serde_json::from_str(&content)?;
        Ok(log)
    }

    /// Saves the learning log to disk.
    fn save_log(path: &Path, log: &LearningLog) -> Result<()> {
        let json = serde_json::to_string_pretty(log)?;
        fs::write(path, json)?;
        Ok(())
    }
}

/// Summary of a learning category.
#[derive(Debug, Clone)]
pub struct CategorySummary {
    /// Category name.
    pub category: String,
    /// Count of entries in this category.
    pub count: usize,
    /// Most recent example entry.
    pub recent_example: Option<LearningEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_learning_entry_new() {
        let entry = LearningEntry::new(
            LearningType::Mistake,
            "Complex Solution Bias".to_string(),
            "Used complex approach when simple would work".to_string(),
            Some("Use simpler solution".to_string()),
        );

        assert_eq!(entry.entry_type, LearningType::Mistake);
        assert_eq!(entry.category, "Complex Solution Bias");
        assert!(entry.solution.is_some());
    }

    #[test]
    fn test_learning_entry_enforce_one_sentence() {
        let multi_line = "First sentence.\nSecond sentence.\nThird sentence.";
        let entry = LearningEntry::new(
            LearningType::Mistake,
            "Test".to_string(),
            multi_line.to_string(),
            None,
        );

        // Should only contain first sentence
        assert!(entry.description.contains("First sentence"));
        assert!(!entry.description.contains("Second sentence"));
    }

    #[test]
    fn test_learning_store_new() {
        let temp_dir = TempDir::new().unwrap();
        let store = LearningStore::new(temp_dir.path()).unwrap();
        assert!(temp_dir.path().join("learning-log.json").exists());
    }

    #[test]
    fn test_learning_store_add_entry() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = LearningStore::new(temp_dir.path()).unwrap();

        let (entry, added) = store
            .add_entry(
                LearningType::Mistake,
                "Complex Solution Bias".to_string(),
                "Over-engineered solution".to_string(),
                Some("Use simpler approach".to_string()),
            )
            .unwrap();

        assert!(added);
        assert_eq!(entry.category, "Complex Solution Bias");
    }

    #[test]
    fn test_learning_store_duplicate_detection() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = LearningStore::new(temp_dir.path()).unwrap();

        let (_, added1) = store
            .add_entry(
                LearningType::Mistake,
                "Test".to_string(),
                "Same mistake description".to_string(),
                Some("Solution".to_string()),
            )
            .unwrap();

        let (_, added2) = store
            .add_entry(
                LearningType::Mistake,
                "Test".to_string(),
                "Same mistake description".to_string(),
                Some("Solution".to_string()),
            )
            .unwrap();

        assert!(added1);
        assert!(!added2); // Duplicate should not be added
    }

    #[test]
    fn test_learning_store_category_normalization() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = LearningStore::new(temp_dir.path()).unwrap();

        let (entry, _) = store
            .add_entry(
                LearningType::Mistake,
                "complex".to_string(), // Should normalize to "Complex Solution Bias"
                "Test".to_string(),
                Some("Solution".to_string()),
            )
            .unwrap();

        assert_eq!(entry.category, "Complex Solution Bias");
    }

    #[test]
    fn test_learning_store_get_category_summary() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = LearningStore::new(temp_dir.path()).unwrap();

        store
            .add_entry(
                LearningType::Mistake,
                "Category A".to_string(),
                "Mistake 1".to_string(),
                Some("Solution 1".to_string()),
            )
            .unwrap();

        store
            .add_entry(
                LearningType::Mistake,
                "Category A".to_string(),
                "Mistake 2".to_string(),
                Some("Solution 2".to_string()),
            )
            .unwrap();

        store
            .add_entry(
                LearningType::Mistake,
                "Category B".to_string(),
                "Mistake 3".to_string(),
                Some("Solution 3".to_string()),
            )
            .unwrap();

        let summary = store.get_category_summary();
        assert_eq!(summary.len(), 2);
        assert_eq!(summary[0].count, 2); // Category A has 2 entries
        assert_eq!(summary[1].count, 1); // Category B has 1 entry
    }

    #[test]
    fn test_learning_store_generate_context() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = LearningStore::new(temp_dir.path()).unwrap();

        store
            .add_entry(
                LearningType::Mistake,
                "Test Category".to_string(),
                "Test mistake".to_string(),
                Some("Test solution".to_string()),
            )
            .unwrap();

        let context = store.generate_context(5);
        assert!(context.contains("Test Category"));
        assert!(context.contains("Test mistake"));
        assert!(context.contains("Test solution"));
    }

    #[test]
    fn test_is_similar() {
        assert!(LearningStore::is_similar("This is a test", "This is a test"));
        assert!(LearningStore::is_similar("This is a test", "This is a different test"));
        assert!(!LearningStore::is_similar("This is a test", "Completely different text"));
    }

    #[test]
    fn test_normalize_category() {
        assert_eq!(
            LearningStore::normalize_category("complex"),
            "Complex Solution Bias"
        );
        assert_eq!(LearningStore::normalize_category("feature creep"), "Feature Creep");
        assert_eq!(LearningStore::normalize_category("unknown category"), "unknown category");
    }
}

