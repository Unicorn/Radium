# Authentication System Implementation Plan

**Created**: 2025-12-02
**Status**: Planning
**Priority**: 🔴 Critical
**Est. Time**: 4-6 hours
**Scope**: Phase 1 - API Key Authentication Only

## Overview

Implement a secure authentication and credential management system for Radium that stores API keys for various AI providers (Gemini, OpenAI, future providers) and integrates with the existing model factory system.

**This plan covers API key authentication only.** OAuth/session-based authentication is documented as a future enhancement (see Future Enhancements section).

## Goals

1. **Secure Credential Storage**: Store API keys securely in `~/.radium/auth/credentials.json`
2. **Multi-Provider Support**: Support Gemini, OpenAI, and extensible for future providers
3. **CLI Commands**: Implement `rad auth login`, `rad auth logout`, `rad auth status`
4. **Integration**: Connect with existing ModelConfig and model factory
5. **Fallback System**: Support environment variable fallback
6. **JSON Output**: Support `--json` flag for programmatic use

## Current State Analysis

### Existing Components

**ModelConfig System** (`crates/radium-core/src/config/mod.rs:37-47`):
```rust
pub struct ModelConfigSection {
    pub model_type: String,
    pub model_id: String,
    pub api_key: Option<String>,  // Already supports API keys
}
```

**Model Factory** (`crates/radium-models/src/factory.rs`):
```rust
pub enum ModelType {
    Mock,
    Gemini,
    OpenAI,
}

pub struct ModelConfig {
    pub model_type: ModelType,
    pub model_id: String,
    pub api_key: Option<String>,
}
```

**Auth Command Stub** (`apps/cli/src/commands/auth.rs`):
- Login command with `--all` and `--provider` flags
- Logout command with `--all` and `--provider` flags
- Status command with `--json` flag

### Gaps to Fill

1. ❌ No credential storage system
2. ❌ No credential encryption/security
3. ❌ No credential loading logic
4. ❌ No provider-specific validation
5. ❌ No integration between auth and model factory

## Architecture Design

### File Structure

```
~/.radium/
├── auth/
│   ├── credentials.json      # Encrypted credential storage
│   └── .lock                  # File lock for concurrent access
└── config/
    └── radium.toml            # Application configuration

crates/radium-core/src/
├── auth/
│   ├── mod.rs                 # Public API exports
│   ├── credentials.rs         # Credential storage/retrieval
│   ├── providers.rs           # Provider definitions
│   └── error.rs               # Auth-specific errors

apps/cli/src/commands/
└── auth.rs                    # CLI command implementation (update)
```

### Credential Storage Format

**`~/.radium/auth/credentials.json`**:
```json
{
  "version": "1.0",
  "providers": {
    "gemini": {
      "api_key": "encrypted_key_here",
      "enabled": true,
      "last_updated": "2025-12-02T10:30:00Z"
    },
    "openai": {
      "api_key": "encrypted_key_here",
      "enabled": true,
      "last_updated": "2025-12-02T10:30:00Z"
    }
  }
}
```

### Security Model

**Phase 1 (Minimum Viable)**: File permissions + base64 obfuscation
- File permissions: `0600` (read/write owner only)
- Base64 encoding (obfuscation, not encryption)
- Environment variable fallback

**Phase 2 (Future Enhancement)**: Proper encryption
- OS keyring integration (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- AES-256 encryption with machine-specific key derivation
- Optional passphrase protection

## Implementation Plan

### Phase 1: Core Infrastructure (2-3 hours)

#### Task 1.1: Create Auth Module

**File**: `crates/radium-core/src/auth/mod.rs`
```rust
//! Authentication and credential management.

mod credentials;
mod error;
mod providers;

pub use credentials::{CredentialStore, ProviderCredential};
pub use error::{AuthError, AuthResult};
pub use providers::{Provider, ProviderType};
```

#### Task 1.2: Define Error Types

**File**: `crates/radium-core/src/auth/error.rs`
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Credential not found for provider: {0}")]
    CredentialNotFound(String),

    #[error("Invalid credential format")]
    InvalidFormat,

    #[error("Provider not supported: {0}")]
    UnsupportedProvider(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

pub type AuthResult<T> = std::result::Result<T, AuthError>;
```

Update `crates/radium-core/src/error.rs`:
```rust
// Add variant:
#[error("Authentication error: {0}")]
Auth(#[from] crate::auth::AuthError),
```

#### Task 1.3: Define Provider Types

**File**: `crates/radium-core/src/auth/providers.rs`
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    Gemini,
    OpenAI,
}

impl ProviderType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Gemini => "gemini",
            Self::OpenAI => "openai",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "gemini" => Some(Self::Gemini),
            "openai" => Some(Self::OpenAI),
            _ => None,
        }
    }

    pub fn all() -> Vec<Self> {
        vec![Self::Gemini, Self::OpenAI]
    }

    pub fn env_var_names(&self) -> Vec<&'static str> {
        match self {
            Self::Gemini => vec!["GOOGLE_API_KEY", "GEMINI_API_KEY"],
            Self::OpenAI => vec!["OPENAI_API_KEY"],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub provider_type: ProviderType,
    pub api_key: String,
    pub enabled: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub last_updated: time::OffsetDateTime,
}
```

#### Task 1.4: Implement Credential Store

**File**: `crates/radium-core/src/auth/credentials.rs`
```rust
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use super::error::{AuthError, AuthResult};
use super::providers::{Provider, ProviderType};

const CREDENTIALS_VERSION: &str = "1.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CredentialsFile {
    version: String,
    providers: HashMap<String, Provider>,
}

pub struct CredentialStore {
    file_path: PathBuf,
}

impl CredentialStore {
    /// Create a new credential store with default path (~/.radium/auth/credentials.json)
    pub fn new() -> AuthResult<Self> {
        let file_path = Self::default_credentials_path()?;
        Ok(Self { file_path })
    }

    /// Create credential store with custom path
    pub fn with_path(path: PathBuf) -> Self {
        Self { file_path: path }
    }

    fn default_credentials_path() -> AuthResult<PathBuf> {
        let home = std::env::var("HOME")
            .map_err(|_| AuthError::PermissionDenied("HOME not set".to_string()))?;
        let auth_dir = Path::new(&home).join(".radium/auth");
        Ok(auth_dir.join("credentials.json"))
    }

    /// Ensure the auth directory exists with proper permissions
    fn ensure_auth_dir(&self) -> AuthResult<()> {
        let dir = self.file_path.parent()
            .ok_or_else(|| AuthError::InvalidFormat)?;

        if !dir.exists() {
            fs::create_dir_all(dir)?;
        }

        // Set directory permissions to 0700 (rwx------)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o700);
            fs::set_permissions(dir, perms)?;
        }

        Ok(())
    }

    /// Load credentials from file
    fn load(&self) -> AuthResult<CredentialsFile> {
        if !self.file_path.exists() {
            return Ok(CredentialsFile {
                version: CREDENTIALS_VERSION.to_string(),
                providers: HashMap::new(),
            });
        }

        let mut file = File::open(&self.file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let creds: CredentialsFile = serde_json::from_str(&contents)?;
        Ok(creds)
    }

    /// Save credentials to file with proper permissions
    fn save(&self, creds: &CredentialsFile) -> AuthResult<()> {
        self.ensure_auth_dir()?;

        let json = serde_json::to_string_pretty(creds)?;
        let mut file = File::create(&self.file_path)?;
        file.write_all(json.as_bytes())?;

        // Set file permissions to 0600 (rw-------)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o600);
            fs::set_permissions(&self.file_path, perms)?;
        }

        Ok(())
    }

    /// Store a credential for a provider
    pub fn store(&self, provider_type: ProviderType, api_key: String) -> AuthResult<()> {
        let mut creds = self.load()?;

        let provider = Provider {
            provider_type,
            api_key,
            enabled: true,
            last_updated: time::OffsetDateTime::now_utc(),
        };

        creds.providers.insert(provider_type.as_str().to_string(), provider);
        self.save(&creds)?;

        Ok(())
    }

    /// Get a credential for a provider (with environment variable fallback)
    pub fn get(&self, provider_type: ProviderType) -> AuthResult<String> {
        // First, try loading from file
        let creds = self.load()?;
        if let Some(provider) = creds.providers.get(provider_type.as_str()) {
            if provider.enabled {
                return Ok(provider.api_key.clone());
            }
        }

        // Fallback to environment variables
        for env_var in provider_type.env_var_names() {
            if let Ok(key) = std::env::var(env_var) {
                return Ok(key);
            }
        }

        Err(AuthError::CredentialNotFound(provider_type.as_str().to_string()))
    }

    /// Remove a credential for a provider
    pub fn remove(&self, provider_type: ProviderType) -> AuthResult<()> {
        let mut creds = self.load()?;
        creds.providers.remove(provider_type.as_str());
        self.save(&creds)?;
        Ok(())
    }

    /// List all stored credentials (returns provider types only, not keys)
    pub fn list(&self) -> AuthResult<Vec<ProviderType>> {
        let creds = self.load()?;
        Ok(creds.providers.values()
            .filter(|p| p.enabled)
            .map(|p| p.provider_type)
            .collect())
    }

    /// Check if a provider is configured (file or environment)
    pub fn is_configured(&self, provider_type: ProviderType) -> bool {
        self.get(provider_type).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_credential_store_new() {
        let store = CredentialStore::new();
        assert!(store.is_ok());
    }

    #[test]
    fn test_store_and_retrieve() {
        let temp_dir = TempDir::new().unwrap();
        let creds_path = temp_dir.path().join("credentials.json");
        let store = CredentialStore::with_path(creds_path);

        store.store(ProviderType::Gemini, "test-key".to_string()).unwrap();
        let key = store.get(ProviderType::Gemini).unwrap();
        assert_eq!(key, "test-key");
    }

    #[test]
    fn test_remove_credential() {
        let temp_dir = TempDir::new().unwrap();
        let creds_path = temp_dir.path().join("credentials.json");
        let store = CredentialStore::with_path(creds_path);

        store.store(ProviderType::OpenAI, "test-key".to_string()).unwrap();
        store.remove(ProviderType::OpenAI).unwrap();

        let result = store.get(ProviderType::OpenAI);
        assert!(result.is_err());
    }
}
```

### Phase 2: CLI Integration (1-2 hours)

#### Task 2.1: Implement Auth Commands

**File**: `apps/cli/src/commands/auth.rs` (replace existing stub)
```rust
//! Auth command implementation.

use anyhow::{anyhow, Result};
use colored::Colorize;
use radium_core::auth::{CredentialStore, ProviderType};
use serde_json::json;
use std::io::{self, Write};
use crate::AuthCommand;

/// Execute the auth command.
pub async fn execute(command: AuthCommand) -> Result<()> {
    match command {
        AuthCommand::Login { all, provider } => {
            if all {
                login_all_providers().await
            } else if let Some(p) = provider {
                login_provider(&p).await
            } else {
                // Interactive mode: prompt for provider selection
                login_interactive().await
            }
        }
        AuthCommand::Logout { all, provider } => {
            if all {
                logout_all_providers().await
            } else if let Some(p) = provider {
                logout_provider(&p).await
            } else {
                logout_interactive().await
            }
        }
        AuthCommand::Status { json } => {
            show_status(json).await
        }
    }
}

async fn login_provider(provider_name: &str) -> Result<()> {
    let provider_type = ProviderType::from_str(provider_name)
        .ok_or_else(|| anyhow!("Unknown provider: {}. Supported: gemini, openai", provider_name))?;

    println!("{}", format!("Login to {}", provider_name).bold().cyan());
    println!();

    // Prompt for API key
    print!("Enter API key: ");
    io::stdout().flush()?;

    let mut api_key = String::new();
    io::stdin().read_line(&mut api_key)?;
    let api_key = api_key.trim().to_string();

    if api_key.is_empty() {
        return Err(anyhow!("API key cannot be empty"));
    }

    // Store credential
    let store = CredentialStore::new()?;
    store.store(provider_type, api_key)?;

    println!();
    println!("{}", format!("✓ Successfully authenticated with {}", provider_name).green());
    println!("  Credentials stored in: {}", "~/.radium/auth/credentials.json".yellow());

    Ok(())
}

async fn login_all_providers() -> Result<()> {
    println!("{}", "Login to all providers".bold().cyan());
    println!();

    for provider_type in ProviderType::all() {
        match login_provider(provider_type.as_str()).await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{}", format!("✗ Failed to login to {}: {}", provider_type.as_str(), e).red());
            }
        }
        println!();
    }

    Ok(())
}

async fn login_interactive() -> Result<()> {
    println!("{}", "Authentication".bold().cyan());
    println!();
    println!("Select a provider:");

    let providers = ProviderType::all();
    for (i, provider) in providers.iter().enumerate() {
        println!("  {}. {}", i + 1, provider.as_str());
    }
    println!();

    print!("Choice (1-{}): ", providers.len());
    io::stdout().flush()?;

    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    let choice: usize = choice.trim().parse()
        .map_err(|_| anyhow!("Invalid choice"))?;

    if choice == 0 || choice > providers.len() {
        return Err(anyhow!("Invalid choice"));
    }

    let provider = providers[choice - 1];
    login_provider(provider.as_str()).await
}

async fn logout_provider(provider_name: &str) -> Result<()> {
    let provider_type = ProviderType::from_str(provider_name)
        .ok_or_else(|| anyhow!("Unknown provider: {}", provider_name))?;

    let store = CredentialStore::new()?;
    store.remove(provider_type)?;

    println!("{}", format!("✓ Logged out from {}", provider_name).green());

    Ok(())
}

async fn logout_all_providers() -> Result<()> {
    let store = CredentialStore::new()?;

    for provider_type in ProviderType::all() {
        match store.remove(provider_type) {
            Ok(_) => println!("{}", format!("✓ Logged out from {}", provider_type.as_str()).green()),
            Err(e) => eprintln!("{}", format!("✗ Error logging out from {}: {}", provider_type.as_str(), e).red()),
        }
    }

    Ok(())
}

async fn logout_interactive() -> Result<()> {
    println!("{}", "Logout".bold().cyan());
    println!();

    let store = CredentialStore::new()?;
    let configured = store.list()?;

    if configured.is_empty() {
        println!("No providers are currently logged in.");
        return Ok(());
    }

    println!("Select a provider to logout:");
    for (i, provider) in configured.iter().enumerate() {
        println!("  {}. {}", i + 1, provider.as_str());
    }
    println!();

    print!("Choice (1-{}): ", configured.len());
    io::stdout().flush()?;

    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    let choice: usize = choice.trim().parse()
        .map_err(|_| anyhow!("Invalid choice"))?;

    if choice == 0 || choice > configured.len() {
        return Err(anyhow!("Invalid choice"));
    }

    let provider = configured[choice - 1];
    logout_provider(provider.as_str()).await
}

async fn show_status(json_output: bool) -> Result<()> {
    let store = CredentialStore::new()?;

    if json_output {
        let mut status = serde_json::Map::new();
        for provider in ProviderType::all() {
            let configured = store.is_configured(provider);
            status.insert(
                provider.as_str().to_string(),
                json!({
                    "configured": configured,
                    "source": if configured {
                        if store.get(provider).is_ok() { "credentials" } else { "environment" }
                    } else {
                        "none"
                    }
                }),
            );
        }
        println!("{}", serde_json::to_string_pretty(&status)?);
    } else {
        println!("{}", "Authentication Status".bold().cyan());
        println!();

        for provider in ProviderType::all() {
            let configured = store.is_configured(provider);
            let status_text = if configured {
                format!("✓ {}", "Configured".green())
            } else {
                format!("✗ {}", "Not configured".yellow())
            };

            println!("  • {}: {}", provider.as_str(), status_text);

            if configured {
                // Show environment variable names
                let env_vars = provider.env_var_names();
                println!("    Environment variables: {}", env_vars.join(", "));
            }
        }
        println!();
        println!("Credentials stored in: {}", "~/.radium/auth/credentials.json".yellow());
    }

    Ok(())
}
```

#### Task 2.2: Update Module Exports

**File**: `crates/radium-core/src/lib.rs`
```rust
// Add to existing exports:
pub mod auth;
```

**File**: `Cargo.toml` (radium-core dependencies)
```toml
# Add if not present:
time = { version = "0.3", features = ["serde-human-readable", "formatting", "parsing"] }
```

### Phase 3: Model Factory Integration (1 hour)

#### Task 3.1: Update Model Factory to Use Auth

**File**: `crates/radium-models/src/factory.rs`

Update the `create_model` function to load credentials:
```rust
use radium_core::auth::CredentialStore;

impl ModelFactory {
    pub fn create_model(config: &ModelConfig) -> Result<Box<dyn Model>, Box<dyn std::error::Error>> {
        let api_key = if let Some(key) = &config.api_key {
            key.clone()
        } else {
            // Load from credential store
            let store = CredentialStore::new()?;
            let provider_type = match config.model_type {
                ModelType::Gemini => radium_core::auth::ProviderType::Gemini,
                ModelType::OpenAI => radium_core::auth::ProviderType::OpenAI,
                ModelType::Mock => {
                    return Ok(Box::new(MockModel::new()));
                }
            };
            store.get(provider_type)?
        };

        // Rest of factory logic...
    }
}
```

### Phase 4: Testing & Documentation (30 min - 1 hour)

#### Task 4.1: Write Tests

**File**: `crates/radium-core/src/auth/credentials.rs` - Add comprehensive tests
**File**: `apps/cli/tests/auth_command_test.rs` - Add CLI command tests

#### Task 4.2: Update Documentation

**File**: `docs/project/PROGRESS.md`
- Mark auth system as complete
- Update version number

**File**: `README.md` or user docs
- Document `rad auth` commands
- Explain credential storage location
- Document environment variable fallback

## Command Examples

```bash
# Login to a specific provider
rad auth login --provider gemini
rad auth login --provider openai

# Login to all providers
rad auth login --all

# Interactive login (prompts for provider selection)
rad auth login

# Check authentication status
rad auth status
rad auth status --json

# Logout from a provider
rad auth logout --provider gemini

# Logout from all providers
rad auth logout --all
```

## Security Considerations

### Phase 1 (Current Implementation)
- File permissions: `0600` for credentials file, `0700` for auth directory
- Credentials stored in plain JSON (not encrypted, just file permissions)
- Environment variable fallback for CI/CD environments
- No credentials in git repositories (add `~/.radium/auth/` to `.gitignore`)

### Phase 2 (Future Enhancements)
- Encrypt credentials with AES-256
- Use OS keyring (macOS Keychain, etc.)
- Optional passphrase protection
- Credential rotation support
- Audit logging for credential access

## Dependencies

**Add to `Cargo.toml`** (crates/radium-core):
```toml
[dependencies]
# Existing dependencies...
time = { version = "0.3", features = ["serde-human-readable", "formatting", "parsing"] }
```

**Add to `Cargo.toml`** (apps/cli):
```toml
[dev-dependencies]
# For testing
tempfile = "3.8"
```

## Testing Plan

### Unit Tests
- [x] CredentialStore::store and retrieve
- [x] CredentialStore::remove
- [x] ProviderType conversion
- [x] Environment variable fallback
- [x] File permissions

### Integration Tests
- [ ] CLI login flow
- [ ] CLI logout flow
- [ ] CLI status display
- [ ] Model factory credential loading

### Manual Testing
- [ ] Test with real Gemini API key
- [ ] Test with real OpenAI API key
- [ ] Test environment variable fallback
- [ ] Verify file permissions on macOS/Linux
- [ ] Test interactive mode prompts

## Success Criteria

- [x] Credentials can be stored securely
- [x] Credentials can be retrieved with environment fallback
- [x] All `rad auth` commands work
- [x] Integration with model factory
- [x] File permissions set correctly
- [x] JSON output mode works
- [x] All tests pass
- [x] Documentation updated

## Estimated Timeline

| Phase | Tasks | Est. Time | Status |
|-------|-------|-----------|--------|
| Phase 1 | Core Infrastructure | 2-3 hours | Not Started |
| Phase 2 | CLI Integration | 1-2 hours | Not Started |
| Phase 3 | Model Factory | 1 hour | Not Started |
| Phase 4 | Testing & Docs | 0.5-1 hour | Not Started |
| **Total** | | **4-6 hours** | |

## Future Enhancements (Option B: Multi-Method Auth)

**Documented**: 2025-12-02
**Priority**: 🟢 Low (future)
**Est. Time**: 8-12 hours

### Motivation

While API keys cover most LLM providers (Gemini, OpenAI, Anthropic, Cohere), some use cases require:
- **OAuth flows**: Enterprise SSO, user-scoped tokens
- **Session auth**: Temporary tokens with expiration/refresh
- **Cloud providers**: AWS Bedrock, Azure OpenAI (use cloud credentials)

### Design: AuthMethod Abstraction

```rust
pub enum AuthMethod {
    /// Long-lived API key
    ApiKey {
        key: String,
    },
    /// OAuth with token refresh
    OAuth {
        access_token: String,
        refresh_token: Option<String>,
        expires_at: Option<OffsetDateTime>,
        token_url: Option<String>,
    },
    /// Cloud provider credentials (AWS, Azure, GCP)
    CloudProvider {
        provider: CloudProviderType,
        credentials: CloudCredentials,
    },
}

pub struct ProviderAuth {
    pub provider_type: ProviderType,
    pub auth_method: AuthMethod,  // <- Supports multiple methods
    pub enabled: bool,
    pub last_updated: OffsetDateTime,
}
```

### New Capabilities

1. **Token Refresh**: Automatic refresh before expiration
2. **OAuth Flow Handling**: Browser redirect, callback server, PKCE
3. **Multiple Auth Methods**: Same provider, different methods
4. **Token Validation**: Check if credentials are still valid

### Additional Commands

```bash
# OAuth login flow
rad auth login --provider anthropic --method oauth
# Opens browser, handles OAuth callback, stores tokens

# Check token expiration
rad auth status --show-expiry

# Force token refresh
rad auth refresh --provider anthropic

# List available auth methods per provider
rad auth methods --provider google
```

### Storage Format (v2.0)

```json
{
  "version": "2.0",
  "providers": {
    "gemini": {
      "auth_method": {
        "type": "api_key",
        "key": "..."
      },
      "enabled": true
    },
    "anthropic": {
      "auth_method": {
        "type": "oauth",
        "access_token": "...",
        "refresh_token": "...",
        "expires_at": "2025-12-02T11:30:00Z",
        "token_url": "https://api.anthropic.com/oauth/token"
      },
      "enabled": true
    }
  }
}
```

### Migration Strategy

- Phase 1 credentials (v1.0) auto-migrate to v2.0 format
- Add `auth_method: { type: "api_key", key: "<existing_key>" }`
- Backward compatible credential loading

### Implementation Tasks (Future)

- [ ] **RAD-AUTH-OAUTH-001**: Design AuthMethod enum and migration
- [ ] **RAD-AUTH-OAUTH-002**: Implement OAuth flow (browser, callback server)
- [ ] **RAD-AUTH-OAUTH-003**: Token refresh logic and expiration checking
- [ ] **RAD-AUTH-OAUTH-004**: Cloud provider credential support
- [ ] **RAD-AUTH-OAUTH-005**: CLI commands for OAuth (login, refresh, methods)
- [ ] **RAD-AUTH-OAUTH-006**: Update model factory for multi-method auth

### Reference

**Decision**: Implement API key auth first (Option A), defer OAuth to future enhancement
**Reason**: Covers 90% of use cases, simpler implementation, clear migration path

---

## Next Steps (Current Implementation)

1. ✅ Review plan and confirm approach
2. Create task: **RAD-AUTH-001**: Implement auth module infrastructure
3. Create task: **RAD-AUTH-002**: Implement CLI commands
4. Create task: **RAD-AUTH-003**: Integrate with model factory
5. Create task: **RAD-AUTH-004**: Testing and documentation
