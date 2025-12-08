//! Model routing system for Smart/Eco tier selection.
//!
//! This module provides intelligent model routing based on task complexity,
//! allowing automatic selection between high-capability (Smart) and
//! cost-effective (Eco) tier models.

pub mod complexity;
pub mod router;
pub mod types;

pub use complexity::ComplexityEstimator;
pub use router::{DecisionType, ModelRouter, RoutingDecision};
pub use types::{ComplexityScore, ComplexityWeights, RoutingTier, TaskType};

