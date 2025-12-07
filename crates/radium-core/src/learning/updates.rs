//! Update operations for skillbook (ACE learning).
//!
//! Provides incremental update operations to prevent context collapse.
//! Instead of regenerating the entire skillbook, we apply specific
//! operations: ADD, UPDATE, TAG, REMOVE.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of update operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum UpdateOperationType {
    /// Add a new skill to the skillbook.
    Add,
    /// Update an existing skill's content.
    Update,
    /// Tag a skill as helpful, harmful, or neutral.
    Tag,
    /// Remove a skill from the skillbook.
    Remove,
}

/// Single update operation to apply to the skillbook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOperation {
    /// Type of operation.
    #[serde(rename = "type")]
    pub op_type: UpdateOperationType,
    /// Section for the skill (required for ADD, optional for others).
    pub section: Option<String>,
    /// Skill content (required for ADD, optional for UPDATE).
    pub content: Option<String>,
    /// Skill ID (required for UPDATE, TAG, REMOVE).
    pub skill_id: Option<String>,
    /// Metadata for the operation (e.g., helpful/harmful/neutral counts for TAG).
    #[serde(default)]
    pub metadata: HashMap<String, u32>,
}

impl UpdateOperation {
    /// Creates an ADD operation.
    pub fn add(section: String, content: String, skill_id: Option<String>) -> Self {
        Self {
            op_type: UpdateOperationType::Add,
            section: Some(section),
            content: Some(content),
            skill_id,
            metadata: HashMap::new(),
        }
    }

    /// Creates an UPDATE operation.
    pub fn update(skill_id: String, content: Option<String>) -> Self {
        Self {
            op_type: UpdateOperationType::Update,
            section: None,
            content,
            skill_id: Some(skill_id),
            metadata: HashMap::new(),
        }
    }

    /// Creates a TAG operation.
    pub fn tag(skill_id: String, tag: &str, increment: u32) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert(tag.to_string(), increment);

        Self {
            op_type: UpdateOperationType::Tag,
            section: None,
            content: None,
            skill_id: Some(skill_id),
            metadata,
        }
    }

    /// Creates a REMOVE operation.
    pub fn remove(skill_id: String) -> Self {
        Self {
            op_type: UpdateOperationType::Remove,
            section: None,
            content: None,
            skill_id: Some(skill_id),
            metadata: HashMap::new(),
        }
    }
}

/// Batch of update operations with reasoning.
///
/// The SkillManager generates these batches based on reflection analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateBatch {
    /// Reasoning for these updates.
    pub reasoning: String,
    /// List of operations to apply.
    pub operations: Vec<UpdateOperation>,
}

impl UpdateBatch {
    /// Creates a new update batch.
    pub fn new(reasoning: String) -> Self {
        Self { reasoning, operations: Vec::new() }
    }

    /// Adds an operation to the batch.
    pub fn add_operation(&mut self, operation: UpdateOperation) {
        self.operations.push(operation);
    }

    /// Checks if the batch is empty.
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_operation_add() {
        let op = UpdateOperation::add("task_guidance".to_string(), "Test skill".to_string(), None);
        assert_eq!(op.op_type, UpdateOperationType::Add);
        assert_eq!(op.section.as_deref(), Some("task_guidance"));
        assert_eq!(op.content.as_deref(), Some("Test skill"));
    }

    #[test]
    fn test_update_operation_tag() {
        let op = UpdateOperation::tag("skill-00001".to_string(), "helpful", 1);
        assert_eq!(op.op_type, UpdateOperationType::Tag);
        assert_eq!(op.skill_id.as_deref(), Some("skill-00001"));
        assert_eq!(op.metadata.get("helpful"), Some(&1));
    }

    #[test]
    fn test_update_batch() {
        let mut batch = UpdateBatch::new("Test reasoning".to_string());
        assert!(batch.is_empty());

        batch.add_operation(UpdateOperation::add("general".to_string(), "Test".to_string(), None));
        assert!(!batch.is_empty());
        assert_eq!(batch.operations.len(), 1);
    }
}
