//! Syntax highlighting support for code blocks.
//!
//! Provides language-specific syntax highlighting using syntect,
//! with support for multiple languages and theme integration.

#[cfg(feature = "syntax")]
pub mod highlighter;
#[cfg(feature = "syntax")]
pub mod languages;
#[cfg(feature = "syntax")]
pub mod theme_adapter;
#[cfg(feature = "tui-theme")]
pub mod tmtheme_loader;

#[cfg(feature = "syntax")]
pub use highlighter::{SyntaxHighlighter, StyledLine, StyledSpan};
#[cfg(feature = "syntax")]
pub use languages::LanguageRegistry;
#[cfg(feature = "tui-theme")]
pub use tmtheme_loader::{load_tmtheme, RadiumTheme};

