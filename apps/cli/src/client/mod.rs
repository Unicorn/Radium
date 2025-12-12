//! Client infrastructure for daemon connections.

pub mod daemon_client;
pub mod session_manager;

pub use daemon_client::DaemonClient;
pub use session_manager::CliSessionManager;
