//! Metacognitive oversight system for agent alignment.
//!
//! This module provides Chain-Pattern Interrupt (CPI) functionality to prevent
//! reasoning lock-in and improve agent alignment with user intent.

pub mod metacognitive;

pub use metacognitive::{
    MetacognitiveError, MetacognitiveService, OversightRequest, OversightResponse, Result,
};
