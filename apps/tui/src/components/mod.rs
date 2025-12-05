//! TUI components for the enhanced workflow dashboard.
//!
//! This module provides reusable UI components for displaying workflow execution,
//! agent timelines, telemetry, logs, and checkpoints.

pub mod agent_timeline;
pub mod checkpoint_modal;
pub mod log_viewer;
pub mod loop_indicator;
pub mod output_window;
pub mod status_footer;
pub mod telemetry_bar;

pub use agent_timeline::AgentTimeline;
pub use checkpoint_modal::CheckpointModal;
pub use log_viewer::LogViewer;
pub use loop_indicator::LoopIndicator;
pub use output_window::OutputWindow;
pub use status_footer::StatusFooter;
pub use telemetry_bar::TelemetryBar;
