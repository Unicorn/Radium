//! Clipboard command implementation.
//!
//! Provides clipboard-based editor integration for universal editor support.

use anyhow::{Context, Result};
use clap::Subcommand;
use colored::Colorize;
use radium_core::clipboard::{self, parser};

#[derive(Subcommand, Debug)]
pub enum ClipboardCommand {
    /// Send code from clipboard to Radium
    Send,
    /// Receive processed code from Radium to clipboard
    Receive,
}

/// Execute the clipboard send command.
///
/// Reads code from clipboard, parses context, and sends to Radium.
pub async fn send() -> Result<()> {
    println!("{}", "rad clipboard send".bold().cyan());
    println!();

    // Read from clipboard
    println!("  {} Reading from clipboard...", "•".dimmed());
    let clipboard_content = clipboard::read_clipboard()
        .context("Failed to read from clipboard")?;

    if clipboard_content.trim().is_empty() {
        anyhow::bail!("Clipboard is empty");
    }

    // Parse clipboard content
    let parsed = parser::parse_clipboard(&clipboard_content);
    
    println!("  {} Content parsed", "✓".green());
    if let Some(ref path) = parsed.file_path {
        println!("    File path: {}", path.cyan());
    }
    if let Some(ref lang) = parsed.language {
        println!("    Language: {}", lang.cyan());
    }

    // Build context for rad step
    let context = serde_json::json!({
        "file_path": parsed.file_path,
        "language": parsed.language,
        "selection": parsed.content,
        "surrounding_lines": ""
    });

    // Execute rad step with context
    println!("  {} Sending to Radium...", "•".dimmed());
    
    // For now, print context (integration with rad step would go here)
    println!("\n{}", "Context:".bold());
    println!("{}", serde_json::to_string_pretty(&context)?);
    
    println!("\n{} Processed code from clipboard and prepared for Radium.", "✓".green());
    println!("  Use 'rad step <agent-id>' with this context to process the code.");

    Ok(())
}

/// Execute the clipboard receive command.
///
/// Formats last agent output and writes to clipboard.
pub async fn receive() -> Result<()> {
    println!("{}", "rad clipboard receive".bold().cyan());
    println!();

    // This would retrieve last agent output from storage/session
    // For now, provide instructions
    println!("  {} Clipboard receive not yet fully implemented.", "•".dimmed());
    println!("  This command will format the last agent output and write to clipboard.");
    println!("  Output will include file path annotation for context.");

    Ok(())
}
