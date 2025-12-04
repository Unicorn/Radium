//! TUI components for the enhanced workflow dashboard.
//!
//! This module provides reusable UI components for displaying workflow execution,
//! agent timelines, telemetry, logs, and checkpoints.

pub mod agent_timeline;
pub mod output_window;
pub mod log_viewer;
pub mod telemetry_bar;
pub mod status_footer;
pub mod checkpoint_modal;
pub mod loop_indicator;

pub use agent_timeline::AgentTimeline;
pub use output_window::OutputWindow;
pub use log_viewer::LogViewer;
pub use telemetry_bar::TelemetryBar;
pub use status_footer::StatusFooter;
pub use checkpoint_modal::CheckpointModal;
pub use loop_indicator::LoopIndicator;
