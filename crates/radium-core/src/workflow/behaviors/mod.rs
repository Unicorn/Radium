//! Workflow behaviors for dynamic execution control.
//!
//! This module implements workflow behaviors that allow agents
//! to control workflow execution through behavior.json files:
//!
//! - **Loop**: Repeat previous steps with max iterations
//! - **Trigger**: Dynamically insert agent execution
//! - **Checkpoint**: Pause workflow execution for manual intervention
//!
//! ## How it works
//!
//! Agents can write a `radium/.radium/memory/behavior.json` file with:
//! ```json
//! {
//!   "action": "loop" | "trigger" | "checkpoint" | "continue" | "stop",
//!   "reason": "Why this action was chosen",
//!   "triggerAgentId": "agent-to-trigger" // Required for trigger action
//! }
//! ```

pub mod checkpoint;
pub mod loop_behavior;
pub mod trigger;
pub mod types;

pub use checkpoint::{CheckpointDecision, CheckpointEvaluator, CheckpointState};
pub use loop_behavior::{LoopBehaviorConfig, LoopCounters, LoopDecision, LoopEvaluator};
pub use trigger::{TriggerBehaviorConfig, TriggerDecision, TriggerEvaluator};
pub use types::{BehaviorAction, BehaviorError, BehaviorEvaluator};
