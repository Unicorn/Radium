//! Model routing system for Smart/Eco tier selection.
//!
//! This module provides intelligent model routing based on task complexity,
//! allowing automatic selection between high-capability (Smart) and
//! cost-effective (Eco) tier models.

pub mod complexity;
pub mod cost_tracker;
pub mod router;
pub mod types;

pub use complexity::ComplexityEstimator;
pub use cost_tracker::{CostMetrics, CostTracker, TierMetrics};
pub use router::{DecisionType, ModelRouter, RoutingDecision};
pub use types::{ComplexityScore, ComplexityWeights, RoutingTier, TaskType};

