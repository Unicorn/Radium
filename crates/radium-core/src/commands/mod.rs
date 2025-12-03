//! Custom commands system.
//!
//! Provides TOML-based custom command definitions with:
//! - Argument substitution: `{{args}}`, `{{arg1}}`, etc.
//! - Shell command injection: `!{command}`
//! - File content injection: `@{file}`
//! - Namespaced commands via directory structure
//! - User vs project command precedence
//!
//! # Example
//!
//! Create a command file `.radium/commands/hello.toml`:
//!
//! ```toml
//! name = "hello"
//! description = "Greet someone"
//! template = "Hello {{arg1}}! Today is !{date +%A}."
//! args = ["name"]
//! ```
//!
//! Then discover and execute:
//!
//! ```rust,no_run
//! use radium_core::commands::{CommandRegistry, CustomCommand};
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut registry = CommandRegistry::new()
//!     .with_project_dir(".radium/commands");
//!
//! registry.discover()?;
//!
//! let command = registry.get("hello").unwrap();
//! let output = command.execute(&["World".to_string()], Path::new("."))?;
//! println!("{}", output);
//! # Ok(())
//! # }
//! ```

mod custom;
mod error;

pub use custom::{CommandRegistry, CustomCommand};
pub use error::{CommandError, Result};
