//! Process management and monitoring.
//!
//! This module provides functionality for spawning, tracking, and managing
//! external processes with log capture and lifecycle management.

mod registry;
mod log_watcher;

pub use registry::{ProcessRegistry, ProcessHandle, ProcessStatus};
pub use log_watcher::LogWatcher;
