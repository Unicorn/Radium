//! AI-powered validation tests for orchestrator quality.
//!
//! This test suite uses AI models to evaluate the quality of Radium's orchestration,
//! providing objective metrics for:
//! - Tool selection appropriateness
//! - Response quality
//! - Multi-turn coherence
//! - Error recovery
//! - File reference accuracy
//!
//! ## Running Tests
//!
//! All tests are marked with `#[ignore]` since they require API keys and incur costs.
//!
//! ```bash
//! # Run all AI validation tests with Gemini (default, ~1 cent for full suite)
//! GEMINI_API_KEY=xxx cargo test --test ai_validation -- --ignored --nocapture
//!
//! # Run all AI validation tests with Claude (higher quality, ~14 cents for full suite)
//! ANTHROPIC_API_KEY=xxx EVALUATOR_MODEL=claude cargo test --test ai_validation -- --ignored --nocapture
//!
//! # Run specific test
//! GEMINI_API_KEY=xxx cargo test --test ai_validation test_tool_selection_for_simple_task -- --ignored --nocapture
//! ```

#[path = "ai_validation/mod.rs"]
mod ai_validation;

#[path = "ai_validation/tool_appropriateness_test.rs"]
mod tool_appropriateness_test;

// Additional test modules will be added here as they're implemented
// mod response_quality_test;
// mod multi_turn_coherence_test;
// mod error_recovery_test;
// mod file_reference_test;
