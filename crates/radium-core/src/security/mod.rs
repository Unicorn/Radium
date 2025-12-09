//! Security module for secret management, credential protection, and privacy filtering.
//!
//! This module provides:
//! - Encrypted secret storage and credential protection
//! - Privacy filtering with sensitive data redaction
//! - Pattern-based detection of sensitive information

mod audit;
mod error;
mod filter;
mod injector;
mod migration;
mod patterns;
mod scanner;
mod secret_manager;

pub use audit::{AuditEntry, AuditFilter, AuditLogger, AuditOperation};
pub use error::{SecurityError, SecurityResult};
pub use filter::{CredentialMatch, SecretFilter};
pub use injector::SecretInjector;
pub use migration::{MigrationManager, MigrationReport};
pub use scanner::{ScanReport, SecretMatch, SecretScanner, Severity};
pub use secret_manager::SecretManager;

// Privacy module files
mod privacy_error;
mod privacy;

// Re-export privacy types at top level
pub use privacy::{PrivacyFilter, RedactionStyle, RedactionStats};
pub use privacy_error::{PrivacyError, Result as PrivacyResult};
pub use patterns::{Pattern, PatternLibrary, validate_luhn};
