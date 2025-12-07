//! Learning store for categorized mistake tracking.
//!
//! Tracks mistakes, solutions, and preferences in categories to build
//! pattern recognition for future oversight improvements.

use std::collections::HashMap;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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
        let mut sentence = text.replace(['\n', '\r'], " ");

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
    /// Map of skill ID to skill.
    #[serde(default)]
    skills: HashMap<String, Skill>,
    /// Map of section to list of skill IDs.
    #[serde(default)]
    sections: HashMap<String, Vec<String>>,
    /// Next ID counter for generating skill IDs.
    #[serde(default)]
    next_skill_id: u32,
    /// Last update timestamp.
    last_updated: DateTime<Utc>,
}

impl LearningLog {
    fn new() -> Self {
        Self {
            categories: HashMap::new(),
            skills: HashMap::new(),
            sections: HashMap::new(),
            next_skill_id: 0,
            last_updated: Utc::now(),
        }
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

/// Standard skill sections for organizing skills.
pub const STANDARD_SECTIONS: &[&str] =
    &["task_guidance", "tool_usage", "error_handling", "code_patterns", "communication", "general"];

/// Skill entry for skillbook (ACE learning).
///
/// Skills are strategies that can be helpful, harmful, or neutral.
/// They are organized by sections and tracked with usage counts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// Unique identifier for the skill.
    pub id: String,
    /// Section this skill belongs to.
    pub section: String,
    /// Content/description of the skill.
    pub content: String,
    /// Count of times this skill was helpful.
    pub helpful: u32,
    /// Count of times this skill was harmful.
    pub harmful: u32,
    /// Count of times this skill was neutral.
    pub neutral: u32,
    /// Timestamp when this skill was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when this skill was last updated.
    pub updated_at: DateTime<Utc>,
    /// Status of the skill (active or invalid).
    #[serde(default = "default_skill_status")]
    pub status: SkillStatus,
}

fn default_skill_status() -> SkillStatus {
    SkillStatus::Active
}

/// Status of a skill.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillStatus {
    /// Skill is active and can be used.
    Active,
    /// Skill is invalid/soft-deleted.
    Invalid,
}

impl Skill {
    /// Creates a new skill.
    pub fn new(id: String, section: String, content: String) -> Self {
        Self {
            id,
            section,
            content: Self::enforce_one_sentence(content),
            helpful: 0,
            harmful: 0,
            neutral: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: SkillStatus::Active,
        }
    }

    /// Ensures text is a single sentence.
    fn enforce_one_sentence(text: String) -> String {
        // Remove newlines
        let mut sentence = text.replace(['\n', '\r'], " ");

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

    /// Tags this skill as helpful, harmful, or neutral.
    ///
    /// # Arguments
    /// * `tag` - The tag to apply ("helpful", "harmful", or "neutral")
    /// * `increment` - Amount to increment (default: 1)
    ///
    /// # Errors
    /// Returns error if tag is invalid
    pub fn tag(&mut self, tag: &str, increment: u32) -> Result<()> {
        match tag {
            "helpful" => self.helpful += increment,
            "harmful" => self.harmful += increment,
            "neutral" => self.neutral += increment,
            _ => {
                return Err(LearningError::InvalidEntry(format!(
                    "Invalid tag: {}. Must be 'helpful', 'harmful', or 'neutral'",
                    tag
                )));
            }
        }
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Returns a dictionary with LLM-relevant fields only.
    ///
    /// Excludes created_at and updated_at which are internal metadata.
    pub fn to_llm_dict(&self) -> HashMap<String, String> {
        let mut dict = HashMap::new();
        dict.insert("id".to_string(), self.id.clone());
        dict.insert("section".to_string(), self.section.clone());
        dict.insert("content".to_string(), self.content.clone());
        dict.insert("helpful".to_string(), self.helpful.to_string());
        dict.insert("harmful".to_string(), self.harmful.to_string());
        dict.insert("neutral".to_string(), self.neutral.to_string());
        dict
    }
}

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
/// Stores learning entries and skills in a JSON file, organized by category and section.
/// Extends the original mistake-tracking system with ACE skillbook functionality.
pub struct LearningStore {
    /// Path to the learning log file.
    log_path: PathBuf,
    /// In-memory cache of learning log (includes both entries and skills).
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
        let is_duplicate =
            existing.iter().any(|e| Self::is_similar(&e.description, &entry.description));

        if is_duplicate {
            return Ok((entry, false));
        }

        // Add to category
        let category_data = self.log.categories.entry(category).or_insert_with(CategoryData::new);
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
        self.log.categories.get(category).map(|data| data.examples.clone()).unwrap_or_default()
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
                CategorySummary { category: category.clone(), count: data.count, recent_example }
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
            writeln!(context, "Category: {} (count: {})", category, data.count).unwrap();

            let examples: Vec<&LearningEntry> =
                data.examples.iter().rev().take(max_per_category).collect();

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

                writeln!(
                    context,
                    "- [{}] {}: {}{}",
                    entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    label,
                    entry.description,
                    solution_text
                )
                .unwrap();
            }

            context.push('\n');
        }

        context.trim().to_string()
    }

    /// Generates a skill ID for a given section.
    ///
    /// # Arguments
    /// * `section` - The section name
    ///
    /// # Returns
    /// A new skill ID in format "section-00001"
    fn generate_skill_id(&mut self, section: &str) -> String {
        self.log.next_skill_id += 1;
        let section_prefix = section.split_whitespace().next().unwrap_or("general").to_lowercase();
        format!("{}-{:05}", section_prefix, self.log.next_skill_id)
    }

    /// Adds a new skill to the skillbook.
    ///
    /// # Arguments
    /// * `section` - The section this skill belongs to
    /// * `content` - The skill content/description
    /// * `skill_id` - Optional skill ID (auto-generated if None)
    ///
    /// # Returns
    /// The created skill
    pub fn add_skill(
        &mut self,
        section: String,
        content: String,
        skill_id: Option<String>,
    ) -> Result<Skill> {
        let id = skill_id.unwrap_or_else(|| self.generate_skill_id(&section));
        let skill = Skill::new(id.clone(), section.clone(), content);

        // Add to skills map
        self.log.skills.insert(id.clone(), skill.clone());

        // Add to sections map
        self.log.sections.entry(section).or_default().push(id);

        // Update timestamp
        self.log.last_updated = Utc::now();

        // Save to disk
        Self::save_log(&self.log_path, &self.log)?;

        Ok(skill)
    }

    /// Tags a skill as helpful, harmful, or neutral.
    ///
    /// # Arguments
    /// * `skill_id` - The skill ID to tag
    /// * `tag` - The tag to apply ("helpful", "harmful", or "neutral")
    /// * `increment` - Amount to increment (default: 1)
    ///
    /// # Errors
    /// Returns error if skill not found or tag is invalid
    pub fn tag_skill(&mut self, skill_id: &str, tag: &str, increment: u32) -> Result<()> {
        let skill =
            self.log.skills.get_mut(skill_id).ok_or_else(|| {
                LearningError::InvalidEntry(format!("Skill not found: {}", skill_id))
            })?;

        skill.tag(tag, increment)?;
        self.log.last_updated = Utc::now();

        // Save to disk
        Self::save_log(&self.log_path, &self.log)?;

        Ok(())
    }

    /// Gets all skills for a specific section.
    ///
    /// # Arguments
    /// * `section` - The section to get skills for
    /// * `include_invalid` - Whether to include invalid/soft-deleted skills
    ///
    /// # Returns
    /// Vector of skills for the section
    pub fn get_skills_by_section(&self, section: &str, include_invalid: bool) -> Vec<Skill> {
        let Some(skill_ids) = self.log.sections.get(section) else {
            return Vec::new();
        };

        skill_ids
            .iter()
            .filter_map(|id| self.log.skills.get(id).cloned())
            .filter(|skill| include_invalid || skill.status == SkillStatus::Active)
            .collect()
    }

    /// Gets all skills in the skillbook.
    ///
    /// # Arguments
    /// * `include_invalid` - Whether to include invalid/soft-deleted skills
    ///
    /// # Returns
    /// Vector of all skills
    pub fn get_all_skills(&self, include_invalid: bool) -> Vec<Skill> {
        self.log
            .skills
            .values()
            .filter(|skill| include_invalid || skill.status == SkillStatus::Active)
            .cloned()
            .collect()
    }

    /// Gets a skill by ID.
    ///
    /// # Arguments
    /// * `skill_id` - The skill ID
    ///
    /// # Returns
    /// The skill if found, None otherwise
    pub fn get_skill(&self, skill_id: &str) -> Option<Skill> {
        self.log.skills.get(skill_id).cloned()
    }

    /// Formats skills as context for agent prompts.
    ///
    /// # Arguments
    /// * `max_per_section` - Maximum skills per section to include
    ///
    /// # Returns
    /// Formatted skillbook context string
    pub fn as_context(&self, max_per_section: usize) -> String {
        let mut context = String::new();

        if self.log.skills.is_empty() {
            return context;
        }

        context.push_str("# Skillbook Strategies\n\n");

        // Group skills by section
        for (section, skill_ids) in &self.log.sections {
            let skills: Vec<&Skill> = skill_ids
                .iter()
                .filter_map(|id| self.log.skills.get(id))
                .filter(|skill| skill.status == SkillStatus::Active)
                .take(max_per_section)
                .collect();

            if skills.is_empty() {
                continue;
            }

            writeln!(context, "## {}\n", section).unwrap();

            for skill in skills {
                // Show helpful/harmful counts
                let counts = if skill.helpful > 0 || skill.harmful > 0 {
                    format!(" (helpful={}, harmful={})", skill.helpful, skill.harmful)
                } else {
                    String::new()
                };

                writeln!(context, "- [{}] {}{}", skill.id, skill.content, counts).unwrap();
            }

            context.push('\n');
        }

        context.trim().to_string()
    }

    /// Applies a batch of update operations to the skillbook.
    ///
    /// # Arguments
    /// * `update` - The update batch to apply
    ///
    /// # Errors
    /// Returns error if any operation fails
    pub fn apply_update(&mut self, update: &crate::learning::UpdateBatch) -> Result<()> {
        for operation in &update.operations {
            self.apply_operation(operation)?;
        }

        // Update timestamp
        self.log.last_updated = Utc::now();

        // Save to disk
        Self::save_log(&self.log_path, &self.log)?;

        Ok(())
    }

    /// Applies a single update operation.
    ///
    /// # Arguments
    /// * `operation` - The operation to apply
    ///
    /// # Errors
    /// Returns error if operation is invalid
    fn apply_operation(&mut self, operation: &crate::learning::UpdateOperation) -> Result<()> {
        use crate::learning::UpdateOperationType;

        match operation.op_type {
            UpdateOperationType::Add => {
                let section = operation.section.as_ref().ok_or_else(|| {
                    LearningError::InvalidEntry("Section required for ADD operation".to_string())
                })?;

                let content = operation.content.as_ref().ok_or_else(|| {
                    LearningError::InvalidEntry("Content required for ADD operation".to_string())
                })?;

                self.add_skill(section.clone(), content.clone(), operation.skill_id.clone())?;
            }
            UpdateOperationType::Update => {
                let skill_id = operation.skill_id.as_ref().ok_or_else(|| {
                    LearningError::InvalidEntry(
                        "Skill ID required for UPDATE operation".to_string(),
                    )
                })?;

                let skill = self.log.skills.get_mut(skill_id).ok_or_else(|| {
                    LearningError::InvalidEntry(format!("Skill not found: {}", skill_id))
                })?;

                if let Some(ref content) = operation.content {
                    skill.content = Skill::enforce_one_sentence(content.clone());
                }

                skill.updated_at = Utc::now();
            }
            UpdateOperationType::Tag => {
                let skill_id = operation.skill_id.as_ref().ok_or_else(|| {
                    LearningError::InvalidEntry("Skill ID required for TAG operation".to_string())
                })?;

                // Apply each tag in metadata
                for (tag, increment) in &operation.metadata {
                    self.tag_skill(skill_id, tag, *increment)?;
                }
            }
            UpdateOperationType::Remove => {
                let skill_id = operation.skill_id.as_ref().ok_or_else(|| {
                    LearningError::InvalidEntry(
                        "Skill ID required for REMOVE operation".to_string(),
                    )
                })?;

                // Soft delete: mark as invalid
                if let Some(skill) = self.log.skills.get_mut(skill_id) {
                    skill.status = SkillStatus::Invalid;
                    skill.updated_at = Utc::now();
                }
            }
        }

        Ok(())
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
        let a_lower = a.to_lowercase();
        let b_lower = b.to_lowercase();
        let a_words: Vec<&str> = a_lower.split_whitespace().collect();
        let b_words: Vec<&str> = b_lower.split_whitespace().collect();

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
        let _store = LearningStore::new(temp_dir.path()).unwrap();
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
        assert_eq!(LearningStore::normalize_category("complex"), "Complex Solution Bias");
        assert_eq!(LearningStore::normalize_category("feature creep"), "Feature Creep");
        assert_eq!(LearningStore::normalize_category("unknown category"), "unknown category");
    }
}
