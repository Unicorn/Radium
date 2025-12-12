//! Code verification module
//!
//! Verifies generated TypeScript code using tsc and ESLint.

mod eslint;
mod tsc;

use std::path::Path;
use thiserror::Error;

pub use eslint::EslintResult;
pub use tsc::TscResult;

use eslint::EslintRunError;
use tsc::TscRunError;

/// Verification errors
#[derive(Debug, Error)]
pub enum VerificationError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("TypeScript compilation failed: {0}")]
    TscFailed(String),
    #[error("ESLint check failed: {0}")]
    EslintFailed(String),
    #[error("Timeout waiting for verification")]
    Timeout,
    #[error("TSC execution error: {0}")]
    TscError(#[from] TscRunError),
    #[error("ESLint execution error: {0}")]
    EslintError(#[from] EslintRunError),
}

/// Combined verification result
#[derive(Debug)]
pub struct VerificationResult {
    pub tsc: TscResult,
    pub eslint: EslintResult,
}

impl VerificationResult {
    /// Check if all verifications passed
    pub fn passed(&self) -> bool {
        self.tsc.success && self.eslint.success
    }

    /// Get total number of errors
    pub fn error_count(&self) -> usize {
        self.tsc.errors.len() + self.eslint.errors.len()
    }
}

/// Run all verification checks on generated code
pub async fn verify_code(project_dir: &Path) -> Result<VerificationResult, VerificationError> {
    let tsc_result = tsc::run_tsc(project_dir).await?;
    let eslint_result = eslint::run_eslint(project_dir).await?;

    Ok(VerificationResult {
        tsc: tsc_result,
        eslint: eslint_result,
    })
}

/// Quick verification for testing (skip eslint if not available)
pub async fn verify_quick(project_dir: &Path) -> Result<TscResult, VerificationError> {
    Ok(tsc::run_tsc(project_dir).await?)
}
