//! Learning system for tracking mistakes and solutions.
//!
//! This module provides functionality for recording agent mistakes, preferences,
//! and successes to build pattern recognition for future improvement.
//! Extends with ACE (Agentic Context Engineering) skillbook functionality.

#[cfg(feature = "workflow")]
pub mod integration;
#[cfg(feature = "workflow")]
pub mod recovery_learning;
pub mod skill_manager;
pub mod store;
pub mod updates;

#[cfg(feature = "workflow")]
pub use integration::{LearningConfig, LearningIntegration};
#[cfg(feature = "workflow")]
pub use recovery_learning::{
    RecoveryLearning, RecoveryLearningError, RecoveryPattern, Result as RecoveryLearningResult,
};
pub use skill_manager::{Result as SkillManagerResult, SkillManager, SkillManagerError};
pub use store::{
    CategorySummary, LearningEntry, LearningError, LearningStore, LearningType, Result,
    STANDARD_CATEGORIES, STANDARD_SECTIONS, Skill, SkillStatus,
};
pub use updates::{UpdateBatch, UpdateOperation, UpdateOperationType};
