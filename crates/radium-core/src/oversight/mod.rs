//! Metacognitive oversight system for agent alignment.
//!
//! This module provides Chain-Pattern Interrupt (CPI) functionality to prevent
//! reasoning lock-in and improve agent alignment with user intent.

#[cfg(feature = "workflow")]
pub mod metacognitive;

#[cfg(feature = "workflow")]
pub use metacognitive::{
    MetacognitiveError, MetacognitiveService, OversightRequest, OversightResponse, Result,
    WorkflowPhase,
};
