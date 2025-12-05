//! Learning system for tracking mistakes and solutions.
//!
//! This module provides functionality for recording agent mistakes, preferences,
//! and successes to build pattern recognition for future improvement.

pub mod store;

pub use store::{
    LearningEntry, LearningError, LearningStore, LearningType, Result as LearningResult,
};

