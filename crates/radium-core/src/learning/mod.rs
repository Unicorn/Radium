//! Learning system for tracking mistakes and solutions.
//!
//! This module provides functionality for recording agent mistakes, preferences,
//! and successes to build pattern recognition for future improvement.
//! Extends with ACE (Agentic Context Engineering) skillbook functionality.

pub mod integration;
pub mod skill_manager;
pub mod store;
pub mod updates;

pub use integration::{LearningConfig, LearningIntegration};
pub use skill_manager::{SkillManager, SkillManagerError, Result as SkillManagerResult};
pub use store::{
    CategorySummary, LearningEntry, LearningError, LearningStore, LearningType, Result, Skill,
    SkillStatus, STANDARD_CATEGORIES, STANDARD_SECTIONS,
};
pub use updates::{UpdateBatch, UpdateOperation, UpdateOperationType};

