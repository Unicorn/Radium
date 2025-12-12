//! Diff display utilities for CLI and TUI.
//!
//! This module provides functions to format and display diffs from patch results
//! and file operation results in a human-readable format.

use crate::workspace::patch::PatchResult;
use crate::workspace::tool_integration::IntegrationResult;
use std::fmt::Write;

/// Format a patch result for CLI display.
pub fn format_patch_result_for_cli(result: &PatchResult) -> String {
    let mut output = String::new();

    if result.success {
        writeln!(output, "✓ Patch applied successfully").unwrap();
    } else {
        writeln!(output, "✗ Patch application failed").unwrap();
    }

    writeln!(output).unwrap();

    // Summary
    writeln!(output, "Summary:").unwrap();
    writeln!(
        output,
        "  Files: {} changed, {} failed",
        result.summary.files_changed, result.summary.files_failed
    )
    .unwrap();
    writeln!(
        output,
        "  Hunks: {} applied, {} failed",
        result.summary.hunks_applied, result.summary.hunks_failed
    )
    .unwrap();
    writeln!(
        output,
        "  Lines: +{} -{}",
        result.summary.total_lines_added, result.summary.total_lines_removed
    )
    .unwrap();
    writeln!(output).unwrap();

    // Changed files
    if !result.changed_files.is_empty() {
        writeln!(output, "Changed files:").unwrap();
        for file in &result.changed_files {
            let status = if file.created {
                "created"
            } else if file.deleted {
                "deleted"
            } else {
                "modified"
            };
            writeln!(
                output,
                "  {} {} ({} hunks, +{} -{})",
                status, file.path.display(), file.hunks_applied, file.lines_added, file.lines_removed
            )
            .unwrap();
        }
        writeln!(output).unwrap();
    }

    // Diffs
    if !result.changed_files.is_empty() {
        writeln!(output, "Diffs:").unwrap();
        writeln!(output, "{}", "─".repeat(80)).unwrap();
        for file in &result.changed_files {
            writeln!(output, "{}", file.diff).unwrap();
            writeln!(output, "{}", "─".repeat(80)).unwrap();
        }
    }

    // Errors
    if !result.errors.is_empty() {
        writeln!(output, "Errors:").unwrap();
        for error in &result.errors {
            writeln!(output, "  ✗ {}", error).unwrap();
            if let Some(suggestion) = error.suggest_fix() {
                writeln!(output, "    Suggestion: {}", suggestion).unwrap();
            }
        }
    }

    output
}

/// Format an integration result for CLI display.
pub fn format_integration_result_for_cli(result: &IntegrationResult) -> String {
    let mut output = String::new();

    if result.success {
        writeln!(output, "✓ Operation completed successfully").unwrap();
    } else {
        writeln!(output, "✗ Operation failed").unwrap();
    }

    writeln!(output).unwrap();

    // Changed paths
    if !result.changed_paths.is_empty() {
        writeln!(output, "Changed paths:").unwrap();
        for path in &result.changed_paths {
            writeln!(output, "  {}", path.display()).unwrap();
        }
        writeln!(output).unwrap();
    }

    // Diffs
    if !result.diffs.is_empty() {
        writeln!(output, "Diffs:").unwrap();
        writeln!(output, "{}", "─".repeat(80)).unwrap();
        for diff in &result.diffs {
            writeln!(output, "{}", diff).unwrap();
            writeln!(output, "{}", "─".repeat(80)).unwrap();
        }
    }

    // Errors
    if !result.errors.is_empty() {
        writeln!(output, "Errors:").unwrap();
        for error in &result.errors {
            writeln!(output, "  ✗ {}", error).unwrap();
            if let Some(suggestion) = error.suggest_fix() {
                writeln!(output, "    Suggestion: {}", suggestion).unwrap();
            }
        }
    }

    output
}

/// Format a patch result for TUI display (simplified, color-coded).
pub fn format_patch_result_for_tui(result: &PatchResult) -> Vec<String> {
    let mut lines = Vec::new();

    if result.success {
        lines.push("✓ Patch applied successfully".to_string());
    } else {
        lines.push("✗ Patch application failed".to_string());
    }

    lines.push(String::new());

    // Summary
    lines.push("Summary:".to_string());
    lines.push(format!(
        "  Files: {} changed, {} failed",
        result.summary.files_changed, result.summary.files_failed
    ));
    lines.push(format!(
        "  Hunks: {} applied, {} failed",
        result.summary.hunks_applied, result.summary.hunks_failed
    ));
    lines.push(format!(
        "  Lines: +{} -{}",
        result.summary.total_lines_added, result.summary.total_lines_removed
    ));
    lines.push(String::new());

    // Changed files
    if !result.changed_files.is_empty() {
        lines.push("Changed files:".to_string());
        for file in &result.changed_files {
            let status = if file.created {
                "created"
            } else if file.deleted {
                "deleted"
            } else {
                "modified"
            };
            lines.push(format!(
                "  {} {} ({} hunks, +{} -{})",
                status, file.path.display(), file.hunks_applied, file.lines_added, file.lines_removed
            ));
        }
        lines.push(String::new());
    }

    // Diffs (first 50 lines to avoid overwhelming TUI)
    if !result.changed_files.is_empty() {
        lines.push("Diffs:".to_string());
        for file in &result.changed_files {
            let diff_lines: Vec<&str> = file.diff.lines().collect();
            let preview: Vec<&str> = diff_lines.iter().take(50).copied().collect();
            lines.extend(preview.iter().map(|s| s.to_string()));
            if diff_lines.len() > 50 {
                lines.push(format!("... ({} more lines)", diff_lines.len() - 50));
            }
        }
    }

    // Errors
    if !result.errors.is_empty() {
        lines.push(String::new());
        lines.push("Errors:".to_string());
        for error in &result.errors {
            lines.push(format!("  ✗ {}", error));
            if let Some(suggestion) = error.suggest_fix() {
                lines.push(format!("    Suggestion: {}", suggestion));
            }
        }
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::patch::{ChangedFile, PatchSummary};
    use std::path::PathBuf;

    #[test]
    fn test_format_patch_result_for_cli() {
        let result = PatchResult {
            success: true,
            changed_files: vec![ChangedFile {
                path: PathBuf::from("test.txt"),
                created: false,
                deleted: false,
                diff: "--- a/test.txt\n+++ b/test.txt\n@@ -1,1 +1,1 @@\n-old\n+new".to_string(),
                hunks_applied: 1,
                lines_added: 1,
                lines_removed: 1,
            }],
            errors: Vec::new(),
            summary: PatchSummary {
                total_files: 1,
                files_changed: 1,
                files_failed: 0,
                total_hunks: 1,
                hunks_applied: 1,
                hunks_failed: 0,
                total_lines_added: 1,
                total_lines_removed: 1,
            },
        };

        let formatted = format_patch_result_for_cli(&result);
        assert!(formatted.contains("✓ Patch applied successfully"));
        assert!(formatted.contains("test.txt"));
        assert!(formatted.contains("Diffs:"));
    }

    #[test]
    fn test_format_integration_result_for_cli() {
        let result = IntegrationResult {
            success: true,
            changed_paths: vec![PathBuf::from("test.txt")],
            errors: Vec::new(),
            diffs: vec!["--- a/test.txt\n+++ b/test.txt\n@@ -1,1 +1,1 @@\n-old\n+new".to_string()],
        };

        let formatted = format_integration_result_for_cli(&result);
        assert!(formatted.contains("✓ Operation completed successfully"));
        assert!(formatted.contains("test.txt"));
    }
}
