//! Code block management commands.
//!
//! Provides commands for listing, copying, saving, and appending code blocks
//! extracted from agent responses.

use anyhow::{Context, Result};
use colored::Colorize;
use comfy_table::{Cell, Color as ComfyColor, Table};
use radium_core::{
    code_blocks::{BlockSelector, CodeBlockStore},
    Workspace,
};

use crate::colors::RadiumBrandColors;
use std::fs;
use std::path::{Path, PathBuf};

/// Code block command subcommands.
#[derive(Debug, clap::Subcommand)]
pub enum CodeCommand {
    /// List all code blocks for a session
    List {
        /// Session ID (defaults to most recent session)
        #[arg(short, long)]
        session_id: Option<String>,
    },
    /// Copy code blocks to clipboard
    Copy {
        /// Block selection (e.g., "1", "1,3,5", "2..5", "1,3..5,7")
        indexes: String,
        /// Session ID (defaults to most recent session)
        #[arg(short, long)]
        session_id: Option<String>,
    },
    /// Save a code block to a file
    Save {
        /// Block index
        index: usize,
        /// Output file path
        #[arg(short, long)]
        file: PathBuf,
        /// Session ID (defaults to most recent session)
        #[arg(short, long)]
        session_id: Option<String>,
    },
    /// Append a code block to a file
    Append {
        /// Block index
        index: usize,
        /// Output file path
        #[arg(short, long)]
        file: PathBuf,
        /// Session ID (defaults to most recent session)
        #[arg(short, long)]
        session_id: Option<String>,
    },
}

/// Execute the code command.
pub async fn execute(cmd: CodeCommand) -> Result<()> {
    let workspace = Workspace::discover()
        .context("Failed to load workspace. Run 'rad init' first.")?;
    let workspace_root = workspace.root();

    match cmd {
        CodeCommand::List { session_id } => {
            list_blocks(workspace_root, session_id).await
        }
        CodeCommand::Copy { indexes, session_id } => {
            copy_blocks(workspace_root, &indexes, session_id).await
        }
        CodeCommand::Save { index, file, session_id } => {
            save_block(workspace_root, index, &file, session_id).await
        }
        CodeCommand::Append { index, file, session_id } => {
            append_block(workspace_root, index, &file, session_id).await
        }
    }
}

/// List all code blocks for a session.
async fn list_blocks(workspace_root: &Path, session_id: Option<String>) -> Result<()> {
    println!("{}", "rad code list".bold().cyan());
    println!();

    let session_id = session_id
        .or_else(|| find_last_session(workspace_root))
        .context("No session ID provided and no sessions found")?;

    let store = CodeBlockStore::new(workspace_root, session_id.clone())?;
    let blocks = store.list_blocks(None)?;

    if blocks.is_empty() {
        println!("  {} No code blocks found for session '{}'", "•".dimmed(), session_id);
        return Ok(());
    }

    println!("{}", "Code Blocks:".bold());
    println!("  Session: {}", session_id.cyan());
    println!();

    let mut table = Table::new();
    table.set_header(vec!["Index", "Language", "Preview"]);

    // Use Radium brand colors
    let primary_rgb = RadiumBrandColors::PRIMARY_RGB;
    let warning_rgb = RadiumBrandColors::WARNING_RGB;

    for block in &blocks {
        let preview = preview_content(&block.content, 3);
        let lang = block
            .language
            .as_deref()
            .unwrap_or("text")
            .to_string();

        table.add_row(vec![
            Cell::new(block.index.to_string())
                .fg(ComfyColor::Rgb {
                    r: primary_rgb.0,
                    g: primary_rgb.1,
                    b: primary_rgb.2,
                }),
            Cell::new(lang)
                .fg(ComfyColor::Rgb {
                    r: warning_rgb.0,
                    g: warning_rgb.1,
                    b: warning_rgb.2,
                }),
            Cell::new(preview),
        ]);
    }

    println!("{}", table);
    println!();
    println!("  {} Total: {} block(s)", "✓".green(), blocks.len());

    Ok(())
}

/// Copy code blocks to clipboard.
async fn copy_blocks(
    workspace_root: &Path,
    indexes: &str,
    session_id: Option<String>,
) -> Result<()> {
    println!("{}", "rad code copy".bold().cyan());
    println!();

    let session_id = session_id
        .or_else(|| find_last_session(workspace_root))
        .context("No session ID provided and no sessions found")?;

    let selector = parse_selection(indexes)?;
    let store = CodeBlockStore::new(workspace_root, session_id)?;
    let blocks = store.get_blocks(selector)?;

    if blocks.is_empty() {
        anyhow::bail!("No blocks selected");
    }

    // Concatenate blocks with separators
    let mut content = String::new();
    for (i, block) in blocks.iter().enumerate() {
        if i > 0 {
            content.push_str(&create_separator(
                block.language.as_deref().unwrap_or("text"),
            ));
            content.push('\n');
        }
        content.push_str(&block.content);
    }

    // Copy to clipboard
    use radium_core::clipboard;
    clipboard::write_clipboard(&content).context("Failed to write to clipboard")?;

    println!("  {} Copied {} block(s) to clipboard", "✓".green(), blocks.len());
    println!();

    Ok(())
}

/// Save a code block to a file.
async fn save_block(
    workspace_root: &Path,
    index: usize,
    file: &PathBuf,
    session_id: Option<String>,
) -> Result<()> {
    println!("{}", "rad code save".bold().cyan());
    println!();

    let session_id = session_id
        .or_else(|| find_last_session(workspace_root))
        .context("No session ID provided and no sessions found")?;

    let store = CodeBlockStore::new(workspace_root, session_id)?;
    let block = store.get_block(index)?;

    // Create parent directories if needed
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(file, &block.content)?;

    println!("  {} Saved block {} to {}", "✓".green(), index, file.display().to_string().cyan());
    println!();

    Ok(())
}

/// Append a code block to a file.
async fn append_block(
    workspace_root: &Path,
    index: usize,
    file: &PathBuf,
    session_id: Option<String>,
) -> Result<()> {
    println!("{}", "rad code append".bold().cyan());
    println!();

    let session_id = session_id
        .or_else(|| find_last_session(workspace_root))
        .context("No session ID provided and no sessions found")?;

    let store = CodeBlockStore::new(workspace_root, session_id)?;
    let block = store.get_block(index)?;

    // Create parent directories if needed
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent)?;
    }

    // Add separator if file exists
    let mut content = String::new();
    if file.exists() {
        content.push_str(&create_separator(
            block.language.as_deref().unwrap_or("text"),
        ));
        content.push('\n');
    }

    content.push_str(&block.content);
    content.push('\n');

    use std::fs::OpenOptions;
    use std::io::Write;
    let mut file_handle = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file)?;
    file_handle.write_all(content.as_bytes())?;

    println!("  {} Appended block {} to {}", "✓".green(), index, file.display().to_string().cyan());
    println!();

    Ok(())
}

/// Parse selection syntax into BlockSelector.
///
/// Supports:
/// - Single: "1"
/// - Multiple: "1,3,5"
/// - Range: "2..5"
/// - Mixed: "1,3..5,7"
fn parse_selection(selection: &str) -> Result<BlockSelector> {
    let parts: Vec<&str> = selection.split(',').collect();

    if parts.len() == 1 {
        // Single number or range
        let part = parts[0].trim();
        if let Some(range_pos) = part.find("..") {
            let start = part[..range_pos].trim().parse::<usize>()
                .context("Invalid range start")?;
            let end = part[range_pos + 2..].trim().parse::<usize>()
                .context("Invalid range end")?;
            return Ok(BlockSelector::Range(start, end));
        } else {
            let index = part.parse::<usize>()
                .context("Invalid block index")?;
            return Ok(BlockSelector::Single(index));
        }
    }

    // Multiple parts - collect all indices
    let mut indices = Vec::new();
    for part in parts {
        let part = part.trim();
        if let Some(range_pos) = part.find("..") {
            let start = part[..range_pos].trim().parse::<usize>()
                .context("Invalid range start")?;
            let end = part[range_pos + 2..].trim().parse::<usize>()
                .context("Invalid range end")?;
            for i in start..=end {
                indices.push(i);
            }
        } else {
            let index = part.parse::<usize>()
                .context("Invalid block index")?;
            indices.push(index);
        }
    }

    if indices.len() == 1 {
        Ok(BlockSelector::Single(indices[0]))
    } else {
        Ok(BlockSelector::Multiple(indices))
    }
}

/// Find the most recent session by checking modification times.
fn find_last_session(workspace_root: &Path) -> Option<String> {
    let code_blocks_dir = workspace_root
        .join(".radium")
        .join("_internals")
        .join("code-blocks");

    if !code_blocks_dir.exists() {
        return None;
    }

    let entries = match fs::read_dir(&code_blocks_dir) {
        Ok(entries) => entries,
        Err(_) => return None,
    };

    let mut sessions: Vec<(String, std::time::SystemTime)> = Vec::new();

    for entry in entries.flatten() {
        if entry.path().is_dir() {
            if let Some(session_id) = entry.file_name().to_str() {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        sessions.push((session_id.to_string(), modified));
                    }
                }
            }
        }
    }

    // Sort by modification time (most recent first)
    sessions.sort_by(|a, b| b.1.cmp(&a.1));

    sessions.first().map(|(id, _)| id.clone())
}

/// Create a separator comment for concatenating code blocks.
fn create_separator(language: &str) -> String {
    let sep = match language {
        "rust" | "c" | "cpp" | "c++" | "java" | "javascript" | "typescript" | "go" => {
            "// ---"
        }
        "python" | "ruby" | "yaml" | "bash" | "sh" | "zsh" => "# ---",
        "html" | "xml" => "<!-- --- -->",
        _ => "// ---",
    };
    format!("{}\n", sep)
}

/// Generate a preview of content (first N lines).
fn preview_content(content: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = content.lines().take(max_lines).collect();
    let preview = lines.join("\n");
    if content.lines().count() > max_lines {
        format!("{}\n...", preview)
    } else {
        preview
    }
}

