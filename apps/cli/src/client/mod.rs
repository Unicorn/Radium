//! Client infrastructure for daemon connections.

pub mod daemon_client;
pub mod session_manager;

// Re-exported for external use but not currently used internally
// pub use daemon_client::DaemonClient;
// pub use session_manager::CliSessionManager;
