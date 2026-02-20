//! AI-powered validation tests for orchestrator quality.
//!
//! This module provides meta-testing capabilities using AI models (Gemini/Claude)
//! to evaluate the quality of Radium's orchestration responses.
//!
//! ## Cost Analysis
//! - **Gemini Flash 2.0** (default): ~$0.0004 per test (~1 cent for full suite)
//! - **Claude Sonnet 4.5** (optional): ~$0.009 per test (~14 cents for full suite)
//!
//! ## Running Tests
//! ```bash
//! # With Gemini (default, cost-effective)
//! GEMINI_API_KEY=xxx cargo test --test ai_validation -- --ignored
//!
//! # With Claude (higher quality)
//! ANTHROPIC_API_KEY=xxx EVALUATOR_MODEL=claude cargo test --test ai_validation -- --ignored
//! ```

use std::env;

pub mod evaluator;
pub mod scenarios;
pub mod git_status_test;
pub mod file_operations_test;

/// Skip test if no API key is available for evaluation.
///
/// Returns true if the test should be skipped.
pub fn skip_if_no_evaluator_key() -> bool {
    let evaluator_model = env::var("EVALUATOR_MODEL").unwrap_or_else(|_| "gemini".to_string());

    let has_key = match evaluator_model.as_str() {
        "claude" => env::var("ANTHROPIC_API_KEY").is_ok(),
        _ => env::var("GEMINI_API_KEY").is_ok(),
    };

    if !has_key {
        println!("⏭️  Skipping AI validation test - no API key available");
        println!("   Set GEMINI_API_KEY (default) or ANTHROPIC_API_KEY with EVALUATOR_MODEL=claude");
    }

    !has_key
}

/// Skip test if no API key is available for the orchestrator under test.
///
/// Returns true if the test should be skipped.
pub fn skip_if_no_orchestrator_key() -> bool {
    let has_gemini = env::var("GEMINI_API_KEY").is_ok();
    let has_anthropic = env::var("ANTHROPIC_API_KEY").is_ok();

    if !has_gemini && !has_anthropic {
        println!("⏭️  Skipping orchestration test - no API key available");
        println!("   Set GEMINI_API_KEY or ANTHROPIC_API_KEY");
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skip_logic() {
        // This test just ensures the skip functions compile and run
        // Actual behavior depends on environment variables
        let _ = skip_if_no_evaluator_key();
        let _ = skip_if_no_orchestrator_key();
    }
}
