//! Training backends (local + cloud).
//!
//! The backend-agnostic types live in `radium-training`. This module contains
//! concrete backend implementations used by the Radium binary.

pub mod burn_trainer;

pub use burn_trainer::BurnBigramTrainer;

