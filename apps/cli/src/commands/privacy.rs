//! Privacy command implementation.

use colored::Colorize;
use radium_core::security::PatternLibrary;
use radium_core::workspace::Workspace;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

/// Privacy command subcommands.
#[derive(Debug, clap::Subcommand)]
pub enum PrivacyCommand {
    /// Check a file for sensitive data.
    ///
    /// Scans the specified file for sensitive data patterns and reports findings.
    /// Exit code: 0 if no sensitive data found, 2 if sensitive data found, 1 on error.
    Check {
        /// Path to file to check
        file: PathBuf,
    },
    /// Test a custom regex pattern.
    ///
    /// Validates a regex pattern and tests it against sample text.
    /// If no text is provided, reads from stdin.
    TestPattern {
        /// Regex pattern to test
        pattern: String,
        /// Sample text to test against (optional, reads from stdin if not provided)
        text: Option<String>,
    },
}

/// Execute the privacy command.
pub async fn execute(command: PrivacyCommand) -> anyhow::Result<()> {
    match command {
        PrivacyCommand::Check { file } => check_file(&file).await,
        PrivacyCommand::TestPattern { pattern, text } => test_pattern(&pattern, text).await,
    }
}

/// Check a file for sensitive data.
async fn check_file(file_path: &PathBuf) -> anyhow::Result<()> {
    // Read file contents
    let content = fs::read_to_string(file_path)
        .map_err(|e| anyhow::anyhow!("Failed to read file {}: {}", file_path.display(), e))?;

    // Load config from workspace to get custom patterns
    let workspace = Workspace::discover().ok();
    let mut pattern_library = PatternLibrary::default();

    // TODO: Load custom patterns from config if workspace found
    // For now, just use default patterns

    // Check for sensitive data
    let matches = pattern_library.find_matches(&content);

    if matches.is_empty() {
        println!("{}", "✓ No sensitive data found".green().bold());
        std::process::exit(0);
    }

    // Report findings
    println!("{}", "⚠ Sensitive data found:".yellow().bold());
    println!();

    let mut total_findings = 0;
    for (pattern_name, matched_values) in &matches {
        println!("  {}: {} occurrence(s)", pattern_name.bright_red().bold(), matched_values.len());
        total_findings += matched_values.len();

        // Show first few examples
        for (idx, value) in matched_values.iter().take(3).enumerate() {
            // Find line numbers
            let line_num = content
                .lines()
                .enumerate()
                .find(|(_, line)| line.contains(value))
                .map(|(i, _)| i + 1)
                .unwrap_or(0);

            if idx < 2 {
                println!("    Line {}: {}", line_num, value.bright_red());
            } else if matched_values.len() > 3 {
                println!("    ... and {} more", matched_values.len() - 3);
                break;
            }
        }
        println!();
    }

    println!("Total: {} sensitive data pattern(s) found", total_findings.to_string().bright_red().bold());
    std::process::exit(2);
}

/// Test a regex pattern.
async fn test_pattern(pattern: &str, text: Option<String>) -> anyhow::Result<()> {
    // Validate regex pattern
    let regex = regex::Regex::new(pattern)
        .map_err(|e| anyhow::anyhow!("Invalid regex pattern: {}", e))?;

    println!("{}", "✓ Pattern is valid".green().bold());
    println!("  Pattern: {}", pattern.cyan());
    println!();

    // Get sample text
    let sample_text = if let Some(text) = text {
        text
    } else {
        println!("Reading sample text from stdin...");
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    };

    if sample_text.is_empty() {
        println!("{}", "⚠ No sample text provided".yellow());
        return Ok(());
    }

    // Test pattern
    let matches: Vec<_> = regex.find_iter(&sample_text).collect();

    if matches.is_empty() {
        println!("{}", "No matches found".yellow());
    } else {
        println!("{} {} found:", "✓".green().bold(), matches.len().to_string().green().bold());
        println!();

        for (idx, mat) in matches.iter().take(10).enumerate() {
            let start = mat.start().saturating_sub(20).max(0);
            let end = (mat.end() + 20).min(sample_text.len());
            let context = &sample_text[start..end];
            let match_start = mat.start() - start;
            let match_end = mat.end() - start;

            let before = &context[..match_start];
            let matched = &context[match_start..match_end];
            let after = &context[match_end..];

            println!("  Match {}: {}{}{}", 
                idx + 1,
                before,
                matched.bright_red().bold(),
                after
            );
        }

        if matches.len() > 10 {
            println!("  ... and {} more", matches.len() - 10);
        }
    }

    Ok(())
}

