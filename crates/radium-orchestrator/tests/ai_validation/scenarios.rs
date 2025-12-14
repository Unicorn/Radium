//! Test scenarios for AI validation.
//!
//! This module defines scenarios that test different aspects of orchestrator quality.

/// A test scenario for orchestrator evaluation.
#[derive(Debug, Clone)]
pub struct TestScenario {
    /// Name of the scenario.
    pub name: String,
    /// Description of what this scenario tests.
    pub description: String,
    /// The user's input/request.
    pub user_input: String,
    /// Expected behavior from the orchestrator.
    pub expected_behavior: String,
}

//
// Tool Appropriateness Scenarios
//

/// Simple task that requires using a specific tool.
pub fn tool_selection_simple() -> TestScenario {
    TestScenario {
        name: "Simple Tool Selection".to_string(),
        description: "Tests whether orchestrator selects the appropriate tool for a straightforward task".to_string(),
        user_input: "Read the contents of README.md file".to_string(),
        expected_behavior: "Should immediately call read_file tool with path 'README.md' without asking for clarification. Should not use glob_file_search or other search tools since the exact filename is provided.".to_string(),
    }
}

/// Task requiring multiple tools in sequence.
pub fn tool_selection_multi_step() -> TestScenario {
    TestScenario {
        name: "Multi-Step Tool Usage".to_string(),
        description: "Tests whether orchestrator can chain multiple tools appropriately".to_string(),
        user_input: "Find all TypeScript files and count how many there are".to_string(),
        expected_behavior: "Should use glob_file_search with pattern '**/*.ts' or '**/*.tsx', then provide a count of the results. Should not use grep or read individual files.".to_string(),
    }
}

/// Task where tool usage is unnecessary.
pub fn tool_avoidance_when_unnecessary() -> TestScenario {
    TestScenario {
        name: "Avoid Unnecessary Tools".to_string(),
        description: "Tests whether orchestrator avoids using tools when not needed".to_string(),
        user_input: "What is the difference between const and let in JavaScript?".to_string(),
        expected_behavior: "Should answer from knowledge without calling any tools. This is a general programming question that doesn't require file access or code search.".to_string(),
    }
}

//
// Response Quality Scenarios
//

/// Task requiring clear, helpful explanation.
pub fn response_quality_explanation() -> TestScenario {
    TestScenario {
        name: "Clear Explanation".to_string(),
        description: "Tests whether responses are clear, accurate, and helpful".to_string(),
        user_input: "Explain how async/await works in Rust".to_string(),
        expected_behavior: "Should provide a clear, accurate explanation of Rust's async/await. Response should be structured, mention key concepts (Future, Poll, executors), and be appropriate for the context without being overly verbose.".to_string(),
    }
}

/// Task requiring concise answer.
pub fn response_quality_conciseness() -> TestScenario {
    TestScenario {
        name: "Concise Response".to_string(),
        description: "Tests whether orchestrator can provide concise answers when appropriate".to_string(),
        user_input: "What's the command to list files in a directory?".to_string(),
        expected_behavior: "Should provide a brief, direct answer (e.g., 'ls' or 'ls -la') without excessive explanation unless asked for details.".to_string(),
    }
}

/// Task requiring actionable response.
pub fn response_quality_actionable() -> TestScenario {
    TestScenario {
        name: "Actionable Guidance".to_string(),
        description: "Tests whether responses include actionable next steps".to_string(),
        user_input: "I want to add error handling to my API endpoints".to_string(),
        expected_behavior: "Should ask clarifying questions about the codebase OR use tools to find existing API endpoints first, then provide specific guidance based on actual code structure. Should not give generic advice without context.".to_string(),
    }
}

//
// Multi-Turn Coherence Scenarios
//

/// Follow-up question referring to previous context.
pub fn multi_turn_context_tracking() -> TestScenario {
    TestScenario {
        name: "Context Tracking".to_string(),
        description: "Tests whether orchestrator maintains context across turns".to_string(),
        user_input: "Can you also check the tests?".to_string(),
        expected_behavior: "Should recognize that 'also' implies a previous action was taken, and 'the tests' refers to tests related to whatever was previously discussed. Should maintain full context from prior conversation.".to_string(),
    }
}

/// Reference resolution across turns.
pub fn multi_turn_reference_resolution() -> TestScenario {
    TestScenario {
        name: "Reference Resolution".to_string(),
        description: "Tests whether orchestrator correctly resolves references from earlier in conversation".to_string(),
        user_input: "Update that file to use async functions instead".to_string(),
        expected_behavior: "Should correctly identify which file 'that file' refers to based on prior context. Should remember what changes were previously discussed.".to_string(),
    }
}

//
// Error Recovery Scenarios
//

/// Graceful handling of tool failure.
pub fn error_recovery_tool_failure() -> TestScenario {
    TestScenario {
        name: "Tool Failure Recovery".to_string(),
        description: "Tests whether orchestrator handles tool failures gracefully".to_string(),
        user_input: "Read the file nonexistent.txt".to_string(),
        expected_behavior: "When read_file fails with 'file not found', should acknowledge the error clearly and offer helpful alternatives (e.g., 'File not found. Would you like me to search for similar files?' or use glob_file_search to find potential matches).".to_string(),
    }
}

/// Recovery from invalid input.
pub fn error_recovery_invalid_request() -> TestScenario {
    TestScenario {
        name: "Invalid Request Handling".to_string(),
        description: "Tests whether orchestrator handles ambiguous or invalid requests appropriately".to_string(),
        user_input: "Fix it".to_string(),
        expected_behavior: "Should recognize that 'it' is ambiguous and politely ask for clarification about what needs to be fixed, rather than making assumptions or failing silently.".to_string(),
    }
}

//
// File Reference Accuracy Scenarios
//

/// Providing accurate file:line references.
pub fn file_reference_accuracy() -> TestScenario {
    TestScenario {
        name: "File Reference Accuracy".to_string(),
        description: "Tests whether file references use correct format and point to actual locations".to_string(),
        user_input: "Where is the read_file function defined?".to_string(),
        expected_behavior: "Should use tools to find the function, then provide a reference in format 'file_path:line_number' or 'file_path:start_line-end_line' that points to the actual definition. Should verify the reference is correct.".to_string(),
    }
}

/// Ensuring references are to actual code locations.
pub fn file_reference_validation() -> TestScenario {
    TestScenario {
        name: "File Reference Validation".to_string(),
        description: "Tests whether file references point to actual, relevant code".to_string(),
        user_input: "Show me where errors are logged in the orchestrator".to_string(),
        expected_behavior: "Should search for error logging code, provide specific file:line references, and verify those references actually contain error logging code (not just mentioning errors in comments).".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenarios_are_valid() {
        // Ensure all scenario functions return valid TestScenario instances
        let scenarios = vec![
            tool_selection_simple(),
            tool_selection_multi_step(),
            tool_avoidance_when_unnecessary(),
            response_quality_explanation(),
            response_quality_conciseness(),
            response_quality_actionable(),
            multi_turn_context_tracking(),
            multi_turn_reference_resolution(),
            error_recovery_tool_failure(),
            error_recovery_invalid_request(),
            file_reference_accuracy(),
            file_reference_validation(),
        ];

        for scenario in scenarios {
            assert!(!scenario.name.is_empty());
            assert!(!scenario.description.is_empty());
            assert!(!scenario.user_input.is_empty());
            assert!(!scenario.expected_behavior.is_empty());
        }
    }
}
