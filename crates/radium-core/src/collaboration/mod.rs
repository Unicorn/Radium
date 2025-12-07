//! Agent collaboration system for multi-agent coordination.
//!
//! This module provides functionality for agents to communicate, coordinate,
//! and collaborate on complex tasks, including:
//! - Message passing between agents
//! - Resource locking for workspace coordination
//! - Task delegation with supervisor-worker patterns
//! - Progress tracking and synchronization

pub mod error;
pub mod lock_manager;
pub mod message_bus;
pub mod progress;

pub use error::{CollaborationError, Result};
pub use lock_manager::{LockHandle, LockType, ResourceLockManager};
pub use message_bus::{
    AgentMessage, DatabaseMessageRepository, MessageBus, MessageRepository, MessageType,
};
pub use progress::{
    AggregatedProgress, ProgressRepository, ProgressSnapshot, ProgressStatus, ProgressTracker,
};

