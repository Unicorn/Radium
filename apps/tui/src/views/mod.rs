//! TUI view modules

pub mod agent;
pub mod dashboard;
pub mod task;
pub mod workflow;

pub use agent::render_agent_view;
pub use dashboard::render_dashboard;
pub use task::render_task_view;
pub use workflow::render_workflow_view;
