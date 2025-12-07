//! TUI view modules

pub mod header;
pub mod history;
pub mod loading;
pub mod markdown;
pub mod model_selector;
pub mod prompt;
pub mod sessions;
pub mod shortcuts;
pub mod splash;
pub mod split;
pub mod workflow;

pub use header::{HeaderInfo, render_header};
pub use history::{HistoryEntry, render_history, render_history_with_log};
pub use loading::{LoadingState, render_loading};
pub use markdown::render_markdown;
pub use model_selector::{ModelInfo, render_model_selector};
pub use prompt::{PromptData, render_prompt, render_setup_wizard};
pub use sessions::render_sessions;
pub use shortcuts::render_shortcuts;
pub use splash::render_splash;
pub use split::{SplitViewState, render_split_view};
pub use workflow::render_workflow;
