//! TUI components for the enhanced workflow dashboard.
//!
//! This module provides reusable UI components for displaying workflow execution,
//! agent timelines, telemetry, logs, and checkpoints.

pub mod agent_timeline;
pub mod checkpoint_modal;
pub mod checkpoint_interrupt_modal;
pub mod dialog;
pub mod execution_detail_view;
pub mod execution_history_view;
pub mod help_row;
pub mod summary_view;
pub mod interactive_table;
pub mod logo;
pub mod log_viewer;
pub mod loop_indicator;
pub mod output_window;
pub mod progress_indicator;
pub mod requirement_progress_bar;
pub mod spinner;
pub mod status_footer;
pub mod status_icon;
pub mod task_list;
pub mod task_list_panel;
pub mod textarea;
pub mod telemetry_bar;
pub mod title_bar;
pub mod toast;

pub use agent_timeline::AgentTimeline;
pub use checkpoint_modal::CheckpointModal;
pub use checkpoint_interrupt_modal::CheckpointInterruptModal;
pub use dialog::{Dialog, DialogChoice, DialogManager, render_dialog};
pub use execution_detail_view::{Action as ExecutionDetailAction, ExecutionDetailView};
pub use execution_history_view::{Action as ExecutionHistoryAction, ExecutionHistoryView, SortColumn};
pub use help_row::render_help_row;
pub use summary_view::{Action as SummaryAction, SummaryView};
pub use interactive_table::InteractiveTable;
pub use logo::{render_logo, render_logo_compact};
pub use log_viewer::LogViewer;
pub use loop_indicator::LoopIndicator;
pub use output_window::OutputWindow;
pub use progress_indicator::{render_progress_bar_simple, render_progress_gauge, render_progress_gauge_custom};
pub use requirement_progress_bar::{render_requirement_progress, render_requirement_progress_new, render_inline_progress, render_inline_progress_new};
pub use spinner::{Spinner, SpinnerFrames};
pub use status_footer::{AppMode, StatusFooter};
pub use status_icon::{get_status_color, render_status_icon};
pub use task_list::{render_task_summary, TaskItem, TaskList};
pub use task_list_panel::TaskListPanel;
pub use telemetry_bar::TelemetryBar;
pub use title_bar::render_title_bar;
pub use toast::{Toast, ToastManager, ToastVariant, render_toasts, render_toasts_with_areas};
