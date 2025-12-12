//! Token-based authentication for daemon connections.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;
use uuid::Uuid;

/// Authentication token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    /// Token ID
    pub id: String,
    /// Token secret (base64 encoded)
    pub secret: String,
    /// When the token was created
    pub created_at: DateTime<Utc>,
    /// When the token expires (None = never expires)
    pub expires_at: Option<DateTime<Utc>>,
}

impl Token {
    /// Generate a new token with a random secret.
    pub fn generate() -> Self {
        let id = Uuid::new_v4().to_string();
        let mut secret_bytes = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut secret_bytes);
        let secret = base64::encode(secret_bytes);

        Self {
            id,
            secret,
            created_at: Utc::now(),
            expires_at: None, // Tokens don't expire by default
        }
    }

    /// Validate a token secret.
    pub fn validate_secret(&self, provided_secret: &str) -> bool {
        self.secret == provided_secret && !self.is_expired()
    }

    /// Check if the token is expired.
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }
}

/// Token store for file-based persistence.
#[derive(Debug)]
pub struct TokenStore {
    tokens_file: PathBuf,
}

impl TokenStore {
    /// Create a new token store.
    ///
    /// # Arguments
    /// * `workspace_root` - Workspace root directory
    ///
    /// # Returns
    /// New TokenStore instance.
    pub fn new(workspace_root: &Path) -> Result<Self> {
        let auth_dir = workspace_root
            .join(".radium")
            .join("auth");

        fs::create_dir_all(&auth_dir)
            .context("Failed to create auth directory")?;

        let tokens_file = auth_dir.join("tokens.json");

        Ok(Self { tokens_file })
    }

    /// Load all tokens from disk.
    pub fn load_tokens(&self) -> Result<Vec<Token>> {
        if !self.tokens_file.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.tokens_file)
            .context("Failed to read tokens file")?;

        let tokens: Vec<Token> = serde_json::from_str(&content)
            .context("Failed to parse tokens file")?;

        Ok(tokens)
    }

    /// Save tokens to disk.
    pub fn save_tokens(&self, tokens: &[Token]) -> Result<()> {
        let json = serde_json::to_string_pretty(tokens)
            .context("Failed to serialize tokens")?;

        fs::write(&self.tokens_file, json)
            .context("Failed to write tokens file")?;

        Ok(())
    }

    /// Add a new token.
    pub fn add_token(&self, token: Token) -> Result<()> {
        let mut tokens = self.load_tokens()?;
        tokens.push(token);
        self.save_tokens(&tokens)?;
        info!("Added new authentication token");
        Ok(())
    }

    /// Validate a token by ID and secret.
    pub fn validate_token(&self, token_id: &str, secret: &str) -> Result<bool> {
        let tokens = self.load_tokens()?;
        for token in tokens {
            if token.id == token_id {
                return Ok(token.validate_secret(secret));
            }
        }
        Ok(false)
    }
}
