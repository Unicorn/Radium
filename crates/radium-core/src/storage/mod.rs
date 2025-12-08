//! Storage layer for Radium Core.
//!
//! This module provides data persistence using SQLite with the Repository pattern.
//! It includes repositories for agents, workflows, and tasks.

// SQL strings don't need hash-less raw strings
#![allow(clippy::needless_raw_string_hashes)]

#[cfg(feature = "monitoring")]
pub mod analytics_repository;
pub mod database;
pub mod error;
pub mod repositories;

pub use database::Database;
pub use error::StorageError;
pub use repositories::{
    AgentRepository, SqliteAgentRepository, SqliteTaskRepository, SqliteWorkflowRepository,
    TaskRepository, WorkflowRepository,
};
#[cfg(feature = "monitoring")]
pub use analytics_repository::AnalyticsRepository;
