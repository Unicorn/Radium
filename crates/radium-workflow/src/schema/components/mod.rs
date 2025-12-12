//! Component schemas for workflow nodes
//!
//! This module contains the type definitions for Start, Stop, Log, and other
//! components used in workflow definitions.

mod log;
mod start;
mod stop;

pub use log::{LogInput, LogLevel, LogOutput};
pub use start::{StartInput, StartOutput};
pub use stop::{StopInput, StopOutput};
