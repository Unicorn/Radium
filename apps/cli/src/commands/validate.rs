//! Source validation command.
//!
//! Validates accessibility of source URIs across multiple protocols.

use anyhow::Context;
use colored::Colorize;
use serde::Serialize;

/// Execute the validate command.
///
/// Validates one or more source URIs and reports their accessibility.
pub async fn execute(sources: Vec<String>, json: bool) -> anyhow::Result<()> {
    if sources.is_empty() {
        anyhow::bail!("At least one source URI is required");
    }

    if !json {
        println!("{}", "rad validate".bold().cyan());
        println!();
        println!("  Validating {} source(s)...", sources.len());
        println!();
    }

    // Validate sources
    let results = crate::validation::validate_sources(sources.clone(), None)
        .await
        .context("Failed to validate sources")?;

    // Output results
    if json {
        output_json(&results)?;
    } else {
        output_human(&results);
    }

    // Exit with error code if any source failed
    let all_valid = results.iter().all(|r| r.accessible);
    if !all_valid {
        std::process::exit(1);
    }

    Ok(())
}

/// Output validation results in JSON format.
fn output_json(results: &[radium_core::context::SourceValidationResult]) -> anyhow::Result<()> {
    use serde::Serialize;

    #[derive(Serialize)]
    struct JsonOutput {
        total: usize,
        valid: usize,
        invalid: usize,
        all_valid: bool,
        results: Vec<JsonResult>,
    }

    #[derive(Serialize)]
    struct JsonResult {
        source: String,
        accessible: bool,
        error: Option<String>,
        size_bytes: i64,
    }

    let valid_count = results.iter().filter(|r| r.accessible).count();
    let invalid_count = results.len() - valid_count;

    let output = JsonOutput {
        total: results.len(),
        valid: valid_count,
        invalid: invalid_count,
        all_valid: invalid_count == 0,
        results: results
            .iter()
            .map(|r| JsonResult {
                source: r.source.clone(),
                accessible: r.accessible,
                error: if r.accessible {
                    None
                } else {
                    Some(r.error_message.clone())
                },
                size_bytes: r.size_bytes,
            })
            .collect(),
    };

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

/// Output validation results in human-readable format.
fn output_human(results: &[radium_core::context::SourceValidationResult]) {
    let valid_count = results.iter().filter(|r| r.accessible).count();
    let invalid_count = results.len() - valid_count;

    // Display individual results
    for result in results {
        if result.accessible {
            println!("  {} {}", "✓".green().bold(), result.source.dimmed());
            if result.size_bytes > 0 {
                println!("    Size: {} bytes", result.size_bytes.to_string().dimmed());
            }
        } else {
            println!("  {} {}", "✗".red().bold(), result.source);
            println!("    {}: {}", "Error".red(), result.error_message.dimmed());
        }
    }

    println!();

    // Display summary
    println!("{}", "Summary:".bold());
    println!("  Total:   {}", results.len());
    println!("  Valid:   {}", valid_count.to_string().green());
    println!("  Invalid: {}", invalid_count.to_string().red());
    println!();

    if invalid_count == 0 {
        println!("{}", "✓ All sources are accessible".green().bold());
    } else {
        println!(
            "{}",
            format!("✗ {} source(s) failed validation", invalid_count)
                .red()
                .bold()
        );
    }
}
