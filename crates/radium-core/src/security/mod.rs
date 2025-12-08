//! Security module for privacy and sensitive data protection.
//!
//! This module provides functionality for detecting and redacting sensitive
//! information from agent context, logs, and telemetry data.
//!
//! # Features
//!
//! - Pattern-based sensitive data detection
//! - Configurable redaction styles (full, partial, hash)
//! - Built-in patterns for common sensitive data types
//! - Custom pattern support
//! - Audit logging of redactions
//! - Thread-safe operations
//!
//! # Example
//!
//! ```no_run
//! use radium_core::security::{PrivacyFilter, RedactionStyle, PatternLibrary};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let patterns = PatternLibrary::default();
//! let filter = PrivacyFilter::new(RedactionStyle::Partial, patterns);
//! let (redacted, stats) = filter.redact("Connect to 192.168.1.100")?;
//! # Ok(())
//! # }
//! ```

pub mod error;

pub use error::{PrivacyError, Result};

