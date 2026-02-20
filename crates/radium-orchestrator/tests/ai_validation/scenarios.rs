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

//
// Git Status Reporting Scenarios
//

/// Basic git status check.
pub fn git_status_basic() -> TestScenario {
    TestScenario {
        name: "Basic Git Status Check".to_string(),
        description: "Tests whether orchestrator calls git_status when asked about repository status".to_string(),
        user_input: "What is the current git status of this repository?".to_string(),
        expected_behavior: "Should immediately call git_status tool to check repository state. \
                           Should not assume the workspace is clean without checking. \
                           Should report actual staged, modified, untracked, and deleted files. \
                           Should show branch information.".to_string(),
    }
}

/// Detecting unstaged changes.
pub fn git_status_unstaged_changes() -> TestScenario {
    TestScenario {
        name: "Unstaged Changes Detection".to_string(),
        description: "Tests whether orchestrator correctly identifies unstaged changes vs staged changes".to_string(),
        user_input: "Do I have any unstaged changes in my working directory?".to_string(),
        expected_behavior: "Should call git_status tool to check working directory. \
                           Should distinguish between staged and unstaged changes. \
                           Should not report the workspace as clean without actually checking. \
                           Should list specific files with unstaged modifications.".to_string(),
    }
}

/// Verifying clean workspace state.
pub fn git_status_clean_workspace() -> TestScenario {
    TestScenario {
        name: "Clean Workspace Verification".to_string(),
        description: "Tests whether orchestrator uses git_status even when user expects clean state".to_string(),
        user_input: "Is my git working directory clean?".to_string(),
        expected_behavior: "Should call git_status tool to verify clean state. \
                           Should not assume clean state based on user's phrasing. \
                           Should report actual status from git, not assumptions.".to_string(),
    }
}

//
// File Operation Scenarios
//

/// Test that AI creates directory before writing file
pub fn file_ops_create_dir_first() -> TestScenario {
    TestScenario {
        name: "Create Directory Before File".to_string(),
        description: "Tests whether orchestrator creates necessary directories before writing files".to_string(),
        user_input: "Create a file at docs/new-feature.md with content '# New Feature'".to_string(),
        expected_behavior: "Should first call create_dir for 'docs' directory (or verify it exists), then call write_file with path 'docs/new-feature.md'. Should not attempt to write file without ensuring parent directory exists, though write_file does auto-create parents.".to_string(),
    }
}

/// Test that AI handles leading slashes correctly
pub fn file_ops_leading_slash_handling() -> TestScenario {
    TestScenario {
        name: "Leading Slash Auto-Correction".to_string(),
        description: "Tests whether orchestrator correctly handles paths with leading slashes".to_string(),
        user_input: "Create a file at /config/settings.json with empty JSON {}".to_string(),
        expected_behavior: "Should use write_file tool with path '/config/settings.json' (leading slash is auto-stripped internally). The file should be created at workspace_root/config/settings.json, NOT at system root. Should not show confusion or ask about the leading slash.".to_string(),
    }
}

/// Test that AI uses rename_file tool appropriately
pub fn file_ops_rename_file_usage() -> TestScenario {
    TestScenario {
        name: "Rename File Tool Usage".to_string(),
        description: "Tests whether orchestrator uses rename_file tool for moving/renaming operations".to_string(),
        user_input: "Move the file old_name.txt to archive/old_name.txt".to_string(),
        expected_behavior: "Should call rename_file tool with old_path='old_name.txt' and new_path='archive/old_name.txt'. Should not use combination of read+write+delete. May optionally call create_dir for 'archive' first, though rename_file auto-creates parent directories.".to_string(),
    }
}

/// Test that AI deletes files safely
pub fn file_ops_delete_safely() -> TestScenario {
    TestScenario {
        name: "Safe File Deletion".to_string(),
        description: "Tests whether orchestrator uses delete_file tool correctly and safely".to_string(),
        user_input: "Delete the file temporary_file.txt".to_string(),
        expected_behavior: "Should call delete_file tool with path 'temporary_file.txt'. Should not attempt to delete directories with this tool (delete_file rejects directories). Should use the tool directly without asking for confirmation unless the context suggests caution is needed.".to_string(),
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
            git_status_basic(),
            git_status_unstaged_changes(),
            git_status_clean_workspace(),
            file_ops_create_dir_first(),
            file_ops_leading_slash_handling(),
            file_ops_rename_file_usage(),
            file_ops_delete_safely(),
        ];

        for scenario in scenarios {
            assert!(!scenario.name.is_empty());
            assert!(!scenario.description.is_empty());
            assert!(!scenario.user_input.is_empty());
            assert!(!scenario.expected_behavior.is_empty());
        }
    }
}
