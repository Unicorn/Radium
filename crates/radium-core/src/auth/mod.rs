//! Authentication and credential management.
//!
//! This module provides secure credential storage and retrieval for AI provider API keys.
//!
//! # Features
//!
//! - Secure file-based credential storage in `~/.radium/auth/credentials.json`
//! - Support for multiple providers (Gemini, OpenAI, etc.)
//! - Environment variable fallback for CI/CD environments
//! - File permissions management (0600 for files, 0700 for directories)
//!
//! # Example
//!
//! ```no_run
//! use radium_core::auth::{CredentialStore, ProviderType};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let store = CredentialStore::new()?;
//!
//! // Store a credential
//! store.store(ProviderType::Gemini, "your-api-key".to_string())?;
//!
//! // Retrieve a credential (with environment fallback)
//! let api_key = store.get(ProviderType::Gemini)?;
//! # Ok(())
//! # }
//! ```

mod credentials;
mod error;
mod middleware;
mod providers;
mod token;

pub use credentials::CredentialStore;
pub use error::{AuthError, AuthResult};
pub use middleware::{authenticate_request, AuthConfig};
pub use providers::ProviderType;
pub use token::{Token, TokenStore};
