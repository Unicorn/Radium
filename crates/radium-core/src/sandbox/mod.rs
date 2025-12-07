//! Sandboxing system for safe agent execution.
//!
//! This module provides sandboxing support for executing shell commands
//! and file operations in isolated environments.
//!
//! # Example
//!
//! ```rust,no_run
//! use radium_core::sandbox::{SandboxFactory, SandboxConfig, SandboxType, Sandbox};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = SandboxConfig::new(SandboxType::Docker)
//!     .with_image("rust:latest".to_string());
//!
//! let mut sandbox = SandboxFactory::create(config)?;
//! sandbox.initialize().await?;
//!
//! let output = sandbox.execute("cargo", &["--version".to_string()], None).await?;
//! println!("Output: {}", String::from_utf8_lossy(&output.stdout));
//!
//! sandbox.cleanup().await?;
//! # Ok(())
//! # }
//! ```

mod config;
mod docker;
mod error;
#[allow(clippy::module_inception)]
mod sandbox;
mod seatbelt;

pub use config::{NetworkMode, SandboxConfig, SandboxProfile, SandboxType};
pub use error::{Result, SandboxError};
pub use sandbox::{NoSandbox, Sandbox, SandboxFactory};

#[cfg(feature = "docker-sandbox")]
pub use docker::DockerSandbox;

#[cfg(all(target_os = "macos", feature = "seatbelt-sandbox"))]
pub use seatbelt::SeatbeltSandbox;
