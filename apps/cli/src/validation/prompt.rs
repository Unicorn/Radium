//! Interactive validation prompting.

use anyhow::{Context, Result};
use colored::Colorize;
use inquire::Confirm;

use super::{display, extract, helper};

/// Validates sources from text and prompts user if validation fails.
///
/// # Arguments
///
/// * `text` - Text content to extract sources from
/// * `workspace_root` - Optional workspace root path
///
/// # Returns
///
/// `Ok(true)` if validation passed or user confirmed, `Ok(false)` if user declined
pub async fn validate_and_prompt(
    text: &str,
    workspace_root: Option<std::path::PathBuf>,
) -> Result<bool> {
    // Extract sources from text
    let sources = extract::extract_sources(text);

    if sources.is_empty() {
        // No sources to validate, proceed
        return Ok(true);
    }

    println!("{}", "Validating sources...".bold());
    println!();

    // Validate sources
    let results = helper::validate_sources(sources, workspace_root)
        .await
        .context("Failed to validate sources")?;

    // Display results
    let checklist = display::format_validation_results(&results);
    print!("{}", checklist);

    // Check if all sources are valid
    let all_valid = results.iter().all(|r| r.accessible);

    if all_valid {
        println!("  {} All sources are accessible\n", "✓".green());
        Ok(true)
    } else {
        let failed_count = results.iter().filter(|r| !r.accessible).count();
        println!();
        let proceed = Confirm::new(&format!(
            "Error: {} source(s) inaccessible. Proceed anyway? [y/N]",
            failed_count
        ))
        .with_default(false)
        .prompt()
        .context("Failed to read user input")?;

        if !proceed {
            println!("{}", "Aborted.".yellow());
            return Ok(false);
        }

        Ok(true)
    }
}
