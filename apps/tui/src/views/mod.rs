//! TUI view modules

pub mod prompt;
pub mod splash;
pub mod header;
pub mod loading;
pub mod sessions;
pub mod model_selector;
pub mod split;
pub mod markdown;

pub use prompt::{render_prompt, render_setup_wizard, PromptData};
pub use splash::render_splash;
pub use header::{render_header, HeaderInfo};
pub use loading::{render_loading, LoadingState};
pub use sessions::render_sessions;
pub use model_selector::{render_model_selector, ModelInfo};
pub use split::{render_split_view, SplitViewState};
pub use markdown::render_markdown;
