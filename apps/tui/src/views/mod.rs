//! TUI view modules

pub mod header;
pub mod loading;
pub mod markdown;
pub mod model_selector;
pub mod prompt;
pub mod sessions;
pub mod splash;
pub mod split;

pub use header::{HeaderInfo, render_header};
pub use loading::{LoadingState, render_loading};
pub use markdown::render_markdown;
pub use model_selector::{ModelInfo, render_model_selector};
pub use prompt::{PromptData, render_prompt, render_setup_wizard};
pub use sessions::render_sessions;
pub use splash::render_splash;
pub use split::{SplitViewState, render_split_view};
