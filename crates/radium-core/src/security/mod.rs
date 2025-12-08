//! Security module for secret management, credential protection, and privacy filtering.
//!
//! This module provides:
//! - Encrypted secret storage and credential protection
//! - Privacy filtering with sensitive data redaction
//! - Pattern-based detection of sensitive information

mod error;
mod filter;
mod injector;
mod patterns;
mod scanner;
mod secret_manager;

pub use error::{SecurityError, SecurityResult};
pub use filter::{CredentialMatch, SecretFilter};
pub use injector::SecretInjector;
pub use scanner::{ScanReport, SecretMatch, SecretScanner, Severity};
pub use secret_manager::SecretManager;

// Privacy module exports
pub mod privacy {
    pub use super::patterns::{Pattern, PatternLibrary, validate_luhn};
    pub use super::privacy_error::{PrivacyError, Result};
    pub use super::privacy_filter::{PrivacyFilter, RedactionStyle, RedactionStats};
}

mod privacy_error;
mod privacy_filter;

pub use privacy::{PrivacyError, Result as PrivacyResult, PrivacyFilter, RedactionStyle, RedactionStats};
pub use patterns::{Pattern, PatternLibrary, validate_luhn};
