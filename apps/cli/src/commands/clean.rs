//! Clean command implementation.

use colored::Colorize;
use radium_core::{Workspace, WorkspaceConfig};
use std::path::PathBuf;

/// Execute the clean command.
///
/// Removes workspace artifacts while preserving the workspace structure.
pub async fn execute(verbose: bool, dir: Option<String>) -> anyhow::Result<()> {
    // Determine workspace directory
    let workspace = if let Some(dir) = dir {
        let path = PathBuf::from(dir);
        Workspace::discover_with_config(&WorkspaceConfig {
            root: Some(path),
            create_if_missing: false,
        })?
    } else {
        Workspace::discover()?
    };

    println!("{}", "Cleaning workspace artifacts...".bold().cyan());
    println!();

    let structure = workspace.structure();
    let mut total_removed = 0;

    // Clean artifacts directory
    if let Ok(removed) = clean_directory(&structure.artifacts_dir(), verbose) {
        total_removed += removed;
    }

    // Clean memory directory
    if let Ok(removed) = clean_directory(&structure.memory_dir(), verbose) {
        total_removed += removed;
    }

    // Clean logs directory
    if let Ok(removed) = clean_directory(&structure.logs_dir(), verbose) {
        total_removed += removed;
    }

    // Clean prompts directory
    if let Ok(removed) = clean_directory(&structure.prompts_dir(), verbose) {
        total_removed += removed;
    }

    // Clean inputs directory
    if let Ok(removed) = clean_directory(&structure.inputs_dir(), verbose) {
        total_removed += removed;
    }

    println!();
    if total_removed > 0 {
        println!("{}", format!("✓ Removed {} files", total_removed).green().bold());
    } else {
        println!("{}", "✓ Workspace already clean".green().bold());
    }

    Ok(())
}

fn clean_directory(dir: &std::path::Path, verbose: bool) -> anyhow::Result<usize> {
    if !dir.exists() {
        return Ok(0);
    }

    let dir_name = dir.file_name().unwrap().to_string_lossy();
    if verbose {
        println!("  Cleaning {}...", dir_name.dimmed());
    }

    let mut count = 0;

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if verbose {
                println!("    Removing {}", path.file_name().unwrap().to_string_lossy());
            }
            std::fs::remove_file(&path)?;
            count += 1;
        } else if path.is_dir() {
            if verbose {
                println!("    Removing directory {}", path.file_name().unwrap().to_string_lossy());
            }
            std::fs::remove_dir_all(&path)?;
            count += 1;
        }
    }

    if verbose && count > 0 {
        println!("    {} items removed", count.to_string().green());
    }

    Ok(count)
}
