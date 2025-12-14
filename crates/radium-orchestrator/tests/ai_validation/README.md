# AI-Powered Validation Tests

Meta-testing suite that uses AI models (Gemini/Claude) to evaluate the quality of Radium's orchestration system.

## Overview

Instead of writing brittle assertions for AI behavior, this test suite uses **AI to grade AI** - providing objective, structured evaluation of orchestrator responses.

### What It Tests

- **Tool Appropriateness**: Does the orchestrator select and use tools correctly?
- **Response Quality**: Are responses clear, accurate, and helpful?
- **Multi-Turn Coherence**: Does it maintain context across conversation turns?
- **Error Recovery**: How gracefully does it handle errors and failures?
- **File Reference Accuracy**: Are file:line references correct and verifiable?

## Cost Analysis

### Gemini Flash 2.0 (Default - Recommended)
- **Per test**: ~$0.0004 (less than 1 cent)
- **Full suite** (15 tests): ~$0.006 (less than 1 cent)
- **Model**: `gemini-2.0-flash-exp`

### Claude Sonnet 4.5 (Optional - Higher Quality)
- **Per test**: ~$0.009 (about 1 cent)
- **Full suite** (15 tests): ~$0.14 (14 cents)
- **Model**: `claude-sonnet-4-5-20250929`

## Running Tests

All tests are marked with `#[ignore]` since they:
1. Require API keys
2. Incur costs (minimal with Gemini)
3. Have external dependencies

### Run All AI Validation Tests

```bash
# With Gemini (default, cost-effective)
GEMINI_API_KEY=xxx cargo test --test ai_validation -- --ignored --nocapture

# With Claude (higher quality evaluation)
ANTHROPIC_API_KEY=xxx EVALUATOR_MODEL=claude cargo test --test ai_validation -- --ignored --nocapture
```

### Run Specific Test Category

```bash
# Tool appropriateness tests
GEMINI_API_KEY=xxx cargo test --test ai_validation tool_appropriateness -- --ignored --nocapture

# (Future) Response quality tests
GEMINI_API_KEY=xxx cargo test --test ai_validation response_quality -- --ignored --nocapture
```

### Run Single Test

```bash
GEMINI_API_KEY=xxx cargo test --test ai_validation test_tool_selection_for_simple_task -- --ignored --nocapture
```

## How It Works

### 1. Test Scenario
Each test defines a scenario with:
- User input
- Expected behavior
- Evaluation criteria

### 2. Real Orchestration
The test executes a real orchestrator (Gemini/Claude) with real tools, just like production.

### 3. AI Evaluation
An AI evaluator (Gemini Flash 2.0 by default) grades the response using:
- **Structured criteria**: Required elements, minimum score
- **Consistent evaluation**: Low temperature (0.2) for reproducibility
- **Detailed feedback**: Score (0-100), pass/fail, reasoning

### 4. Test Assertion
The test asserts based on:
- Pass/fail from evaluator
- Score threshold (typically 70/100)

## Example Test

```rust
#[tokio::test]
#[ignore = "AI validation test - requires API keys and costs money"]
async fn test_tool_selection_for_simple_task() {
    // Skip if no API keys
    if skip_if_no_orchestrator_key() || skip_if_no_evaluator_key() {
        return;
    }

    let scenario = tool_selection_simple();  // "Read README.md"
    let evaluator = AiEvaluator::from_env().expect("Failed to create evaluator");

    // Execute real orchestration
    let tools = create_file_operation_tools(workspace_root);
    let provider = create_test_orchestrator();  // Real Gemini/Claude
    let engine = OrchestrationEngine::with_defaults(provider, tools);

    let result = engine.execute(&scenario.user_input, &mut context).await?;

    // Evaluate with AI
    let criteria = EvaluationCriteria {
        aspect: EvaluationAspect::ToolAppropriateness,
        required_elements: vec![
            "Should call read_file tool".to_string(),
            "Should not ask for clarification".to_string(),
        ],
        min_score: 70,
    };

    let evaluation = evaluator.evaluate(
        &scenario.description,
        &scenario.user_input,
        &scenario.expected_behavior,
        &result.response,
        &criteria,
    ).await?;

    // Assert based on AI evaluation
    assert!(evaluation.passed);
    assert!(evaluation.score.unwrap() >= 70);
}
```

## Architecture

```
tests/ai_validation/
├── mod.rs                          # Shared utilities, skip helpers
├── evaluator.rs                    # AI evaluator core
├── scenarios.rs                    # Test scenario definitions
├── tool_appropriateness_test.rs    # ✅ Implemented (3 tests)
├── response_quality_test.rs        # 🚧 TODO (3 tests)
├── multi_turn_coherence_test.rs    # 🚧 TODO (2 tests)
├── error_recovery_test.rs          # 🚧 TODO (2 tests)
├── file_reference_test.rs          # 🚧 TODO (2 tests)
└── README.md                       # This file
```

## Current Status

### ✅ Completed
- Core infrastructure (evaluator, scenarios, utilities)
- Tool appropriateness tests (3 tests)
- Documentation

### 🚧 TODO
- Response quality tests (3 tests)
- Multi-turn coherence tests (2 tests)
- Error recovery tests (2 tests)
- File reference tests (2 tests)

**Total**: 3/15 tests implemented

## Benefits

### 1. Objective Quality Metrics
- Numeric scores for orchestrator intelligence
- Regression detection for prompt changes
- Benchmark for model improvements

### 2. Cost-Effective
- ~1 cent for full test suite with Gemini
- Much cheaper than manual QA
- Runs in CI/CD (with API keys)

### 3. Comprehensive Coverage
- Tests actual AI behavior, not just API calls
- Validates real-world scenarios
- Catches intelligence regressions

### 4. Maintainable
- No brittle assertions on exact AI output
- Adapts to improved AI responses
- Clear pass/fail criteria

## Flakiness Mitigation

1. **Low Temperature (0.2)**: Evaluator uses consistent, deterministic grading
2. **Structured Output**: JSON responses with validation
3. **Score Ranges**: Tests use thresholds (≥70) not exact values
4. **Retry Logic**: Can be added for transient failures
5. **Test Isolation**: Fresh context for each test

## Future Enhancements

- [ ] Add remaining test categories
- [ ] CI/CD integration with secret API keys
- [ ] Performance benchmarking (tokens, latency)
- [ ] Comparison testing (Gemini vs Claude)
- [ ] Historical score tracking
- [ ] Automated regression alerts

## Contributing

When adding new tests:

1. **Add scenario** in `scenarios.rs`
2. **Create test file** following the pattern in `tool_appropriateness_test.rs`
3. **Define criteria** with clear required elements and min score
4. **Update this README** with new test count
5. **Run test** to verify it passes with current orchestrator

## Notes

- Tests use REAL orchestrators and REAL tools (not mocks)
- Evaluation is subjective but consistent (AI-graded)
- Scores may vary slightly (~±5 points) due to model randomness
- Tests marked `#[ignore]` won't run in standard `cargo test`
- Requires `GEMINI_API_KEY` or `ANTHROPIC_API_KEY`

---

**Meta-testing**: Using AI to evaluate AI. The future of quality assurance. 🤖🔬
