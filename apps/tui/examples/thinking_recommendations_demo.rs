//! Demo of ThinkingPanel and RecommendationsPanel usage
//!
//! This example demonstrates how to use the transparent thinking process
//! and interactive recommendations features in the Radium TUI.
//!
//! Run with: cargo run --package radium-tui --example thinking_recommendations_demo

use radium_tui::views::{ThinkingPanel, ThinkingStepStatus, RecommendationsPanel};

fn main() {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║   Radium TUI - Thinking & Recommendations Panel Demo         ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();

    // ═══════════════════════════════════════════════════════════════════
    // PART 1: Thinking Panel - Transparent Reasoning Process
    // ═══════════════════════════════════════════════════════════════════

    println!("═══ Part 1: Thinking Panel Demo ═══");
    println!();

    let mut thinking_panel = ThinkingPanel::new();

    // Start a thinking session with context
    thinking_panel.start_session("Analyzing test coverage for the project");
    println!("✓ Started thinking session: \"Analyzing test coverage\"");

    // Add thinking steps as the agent reasons through the problem
    thinking_panel.add_step("Searching for coverage reports");
    println!("  → Added step: Searching for coverage reports");

    std::thread::sleep(std::time::Duration::from_millis(500));

    // Update the last step with findings
    thinking_panel.update_last_step(
        ThinkingStepStatus::CompletedWithFindings,
        Some("Found coverage reports in coverage/ directory".to_string())
    );
    println!("  ✓ Updated step: Found coverage reports");

    // Add more thinking steps
    thinking_panel.add_step("Analyzing coverage percentages");
    println!("  → Added step: Analyzing coverage percentages");

    std::thread::sleep(std::time::Duration::from_millis(500));

    thinking_panel.update_last_step(
        ThinkingStepStatus::CompletedWithFindings,
        Some("Line coverage: 73%, Branch coverage: 68%".to_string())
    );
    println!("  ✓ Updated step: Coverage analyzed");

    thinking_panel.add_step("Identifying uncovered modules");
    println!("  → Added step: Identifying uncovered modules");

    std::thread::sleep(std::time::Duration::from_millis(500));

    thinking_panel.update_last_step(
        ThinkingStepStatus::CompletedWithFindings,
        Some("Found 5 modules with <50% coverage".to_string())
    );
    println!("  ✓ Updated step: Identified uncovered modules");

    // End the session
    thinking_panel.end_session();
    println!("✓ Thinking session complete");
    println!();

    // Show panel state
    println!("Panel state:");
    println!("  - Visible: {}", thinking_panel.is_visible());
    println!("  - Expanded: {}", thinking_panel.is_expanded());
    println!();

    // ═══════════════════════════════════════════════════════════════════
    // PART 2: Recommendations Panel - Interactive Next Steps
    // ═══════════════════════════════════════════════════════════════════

    println!("═══ Part 2: Recommendations Panel Demo ═══");
    println!();

    let mut recommendations_panel = RecommendationsPanel::new();

    // Start a recommendations session
    recommendations_panel.start_session("Improve test coverage to 85%+");
    println!("✓ Started recommendations session");

    // Add actionable recommendations with commands
    recommendations_panel.add_recommendation(
        "Install coverage tool",
        Some("cargo install cargo-tarpaulin".to_string()),
        Some("Required for generating detailed coverage reports".to_string())
    );
    println!("  → Added recommendation: Install coverage tool");

    recommendations_panel.add_recommendation(
        "Generate coverage report",
        Some("cargo tarpaulin --out Html --output-dir coverage".to_string()),
        Some("Creates HTML coverage report in coverage/ directory".to_string())
    );
    println!("  → Added recommendation: Generate coverage report");

    recommendations_panel.add_recommendation(
        "Add tests for uncovered modules",
        None,
        Some("Focus on: auth.rs, config.rs, parser.rs, utils.rs, validator.rs".to_string())
    );
    println!("  → Added recommendation: Add tests for uncovered modules");

    recommendations_panel.add_recommendation(
        "Set up CI coverage checks",
        Some("echo 'coverage: 85%' >> .github/workflows/ci.yml".to_string()),
        Some("Fail CI if coverage drops below 85%".to_string())
    );
    println!("  → Added recommendation: Set up CI coverage checks");

    println!();

    // Request execution from user
    recommendations_panel.request_execution();
    println!("✓ Requesting user confirmation for execution");
    println!("  Mode: {:?}", recommendations_panel.mode());
    println!();

    // Simulate user interaction
    println!("═══ Simulating User Interaction ═══");
    println!();

    // User confirms execution
    println!("User presses 'Y' to confirm execution");
    let confirmed = recommendations_panel.confirm_execution();
    println!("  → Execution confirmed: {}", confirmed);
    println!("  → Mode: {:?}", recommendations_panel.mode());
    println!();

    // Execute recommendations sequentially
    println!("═══ Executing Recommendations ═══");
    println!();

    loop {
        // Get next recommendation to execute
        let next = recommendations_panel.get_next_to_execute();
        if next.is_none() {
            break;
        }

        let (index, description, has_command) = {
            let (idx, rec) = next.unwrap();
            (idx, rec.description.clone(), rec.command.is_some())
        };

        println!("Executing step {}: {}", index + 1, description);

        recommendations_panel.mark_executing(index);

        // Simulate command execution
        if has_command {
            println!("  $ <command>");
            std::thread::sleep(std::time::Duration::from_secs(1));

            // Simulate successful execution
            let output = format!("Command completed successfully (simulated)\nOutput: OK");
            recommendations_panel.mark_completed(index, output);
            println!("  ✓ Completed");
        } else {
            // No command to execute, just mark as completed
            recommendations_panel.mark_completed(
                index,
                "Manual step - requires developer action".to_string()
            );
            println!("  ℹ Manual step (no command)");
        }

        println!();
    }

    println!("✓ All recommendations executed");
    println!("  Mode: {:?}", recommendations_panel.mode());
    println!();

    // ═══════════════════════════════════════════════════════════════════
    // PART 3: Usage Patterns and Integration
    // ═══════════════════════════════════════════════════════════════════

    println!("═══ Part 3: Integration Patterns ═══");
    println!();

    println!("Usage in Radium TUI:");
    println!();

    println!("1. Keyboard Shortcuts:");
    println!("   - Press 't' to toggle thinking panel expand/collapse");
    println!("   - Press 'Y' to confirm recommendation execution");
    println!("   - Press 'N' to decline recommendation execution");
    println!();

    println!("2. Rendering:");
    println!("   - ThinkingPanel renders in bottom-left quadrant");
    println!("   - RecommendationsPanel renders in bottom-right quadrant");
    println!("   - Both panels are only visible when active");
    println!();

    println!("3. State Management:");
    println!("   - Panels are stored in App struct");
    println!("   - State persists across renders");
    println!("   - Panels can be cleared with .clear() method");
    println!();

    println!("4. Example Integration in Agent:");
    println!();
    println!("   // Start thinking session");
    println!("   app.thinking_panel.start_session(\"Understanding user request\");");
    println!("   ");
    println!("   // Add thinking steps during analysis");
    println!("   app.thinking_panel.add_step(\"Parsing user input\");");
    println!("   app.thinking_panel.update_last_step(");
    println!("       ThinkingStepStatus::Completed,");
    println!("       Some(\"Request parsed successfully\".into())");
    println!("   );");
    println!("   ");
    println!("   // Build recommendations based on analysis");
    println!("   app.recommendations_panel.start_session(\"Suggested next steps\");");
    println!("   app.recommendations_panel.add_recommendation(");
    println!("       \"Run tests\",");
    println!("       Some(\"cargo test\".into()),");
    println!("       None");
    println!("   );");
    println!("   ");
    println!("   // Request user approval");
    println!("   app.recommendations_panel.request_execution();");
    println!();

    println!("5. Interaction Flow:");
    println!("   Display → AwaitingConfirmation → Executing → Completed");
    println!("   - Display: Just showing recommendations");
    println!("   - AwaitingConfirmation: Asking user Y/N");
    println!("   - Executing: Running commands sequentially");
    println!("   - Completed: All done or failed");
    println!();

    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║   Demo Complete!                                              ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
}
