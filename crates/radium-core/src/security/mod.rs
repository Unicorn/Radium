//! Security module for secret management, credential protection, and privacy filtering.
//!
//! This module provides:
//! - Encrypted secret storage and credential protection
//! - Privacy filtering with sensitive data redaction
//! - Pattern-based detection of sensitive information

mod error;
mod patterns;
// mod secret_manager;  // TODO: Implement secret manager

pub use error::{SecurityError, SecurityResult};
// pub use secret_manager::SecretManager;

// Privacy module exports
pub mod privacy {
    pub use super::patterns::{Pattern, PatternLibrary, validate_luhn};
    pub use super::privacy_error::{PrivacyError, Result};
}

mod privacy_error;

pub use privacy::{PrivacyError, Result as PrivacyResult};
pub use patterns::{Pattern, PatternLibrary, validate_luhn};
