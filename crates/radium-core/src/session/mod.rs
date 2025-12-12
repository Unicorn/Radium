//! Session management for daemon-backed multi-client sessions.
//!
//! Provides session state management, persistence, and resume capabilities
//! to enable long-running sessions that survive process restarts and support
//! multiple clients attaching to the same session.

pub mod manager;
pub mod state;
pub mod storage;

pub use manager::SessionManager;
pub use state::{Session, SessionState};
pub use storage::SessionStorage;
