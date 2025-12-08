//! Playbook management commands.

use clap::Subcommand;
use colored::Colorize;
use radium_core::playbooks::{
    discovery::PlaybookDiscovery, parser::PlaybookParser, registry::PlaybookRegistry,
    storage::PlaybookStorage, types::PlaybookPriority,
};
use radium_core::workspace::Workspace;
use std::path::PathBuf;

/// Playbook command options.
#[derive(Subcommand, Debug)]
pub enum PlaybookCommand {
    /// List all available playbooks
    List {
        /// Filter by scope (e.g., requirement, task, pr-review)
        #[arg(long)]
        scope: Option<String>,

        /// Filter by priority (required, recommended, optional)
        #[arg(long)]
        priority: Option<String>,

        /// Filter by tag(s) (comma-separated)
        #[arg(long)]
        tag: Option<String>,

        /// List playbooks from Braingrid instead of local
        #[arg(long)]
        remote: bool,

        /// Braingrid project ID (required for --remote)
        #[arg(long)]
        project_id: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Apply/install a playbook from file
    Apply {
        /// Path to playbook file
        file: PathBuf,
    },

    /// Delete a playbook by URI
    Delete {
        /// Playbook URI (e.g., radium://org/playbook.md)
        uri: String,
    },

    /// Search playbooks by tag(s)
    Search {
        /// Tag(s) to search for (comma-separated)
        tags: String,

        /// Search playbooks from Braingrid instead of local
        #[arg(long)]
        remote: bool,

        /// Braingrid project ID (required for --remote)
        #[arg(long)]
        project_id: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Sync playbooks between local and Braingrid
    Sync {
        /// Braingrid project ID
        #[arg(long)]
        project_id: Option<String>,

        /// Upload local playbooks to Braingrid (default: download only)
        #[arg(long)]
        upload: bool,
    },
}

/// Execute playbook command.
pub async fn execute_playbook_command(command: PlaybookCommand) -> anyhow::Result<()> {
    match command {
        PlaybookCommand::List { scope, priority, tag, json } => {
            list_playbooks(scope, priority, tag, json).await
        }
        PlaybookCommand::Apply { file } => apply_playbook(file).await,
        PlaybookCommand::Delete { uri } => delete_playbook(uri).await,
        PlaybookCommand::Search { tags, json } => search_playbooks(tags, json).await,
    }
}

/// List all playbooks.
async fn list_playbooks(
    scope: Option<String>,
    priority: Option<String>,
    tag: Option<String>,
    json: bool,
) -> anyhow::Result<()> {
    let discovery = PlaybookDiscovery::new()
        .map_err(|e| anyhow::anyhow!("Failed to initialize playbook discovery: {}", e))?;

    let mut playbooks = if let Some(scope_filter) = scope {
        discovery.find_by_scope(&scope_filter)?
    } else {
        discovery.discover_all()?.into_values().collect()
    };

    // Filter by priority if specified
    if let Some(priority_filter) = priority {
        let priority_enum = match priority_filter.to_lowercase().as_str() {
            "required" => PlaybookPriority::Required,
            "recommended" => PlaybookPriority::Recommended,
            "optional" => PlaybookPriority::Optional,
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid priority: {}. Must be: required, recommended, or optional",
                    priority_filter
                ));
            }
        };
        playbooks.retain(|p| p.priority == priority_enum);
    }

    // Filter by tags if specified
    if let Some(tag_filter) = tag {
        let tags: Vec<String> = tag_filter
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        playbooks.retain(|p| p.has_tags(&tags));
    }

    // Sort by priority (Required → Recommended → Optional)
    playbooks.sort_by(|a, b| b.priority.cmp(&a.priority));

    if json {
        let playbooks_json: Vec<serde_json::Value> = playbooks
            .iter()
            .map(|p| {
                serde_json::json!({
                    "uri": p.uri,
                    "description": p.description,
                    "tags": p.tags,
                    "priority": p.priority.to_string(),
                    "applies_to": p.applies_to,
                })
            })
            .collect();
        println!("{}", serde_json::json!({ "playbooks": playbooks_json }));
    } else {
        if playbooks.is_empty() {
            println!("No playbooks found.");
            println!(
                "Install playbooks using: {}",
                "rad playbook apply <file>".bright_blue()
            );
            return Ok(());
        }

        println!("Playbooks");
        println!("=========");
        println!();

        // Table header
        println!(
            "{:<50} {:<60} {:<15} {:<20} {:<30}",
            "URI", "Description", "Priority", "Scope", "Tags"
        );
        println!("{}", "-".repeat(175));

        for playbook in playbooks {
            let priority_str = match playbook.priority {
                PlaybookPriority::Required => playbook.priority.to_string().red().to_string(),
                PlaybookPriority::Recommended => {
                    playbook.priority.to_string().yellow().to_string()
                }
                PlaybookPriority::Optional => playbook.priority.to_string().green().to_string(),
            };

            let scope_str = if playbook.applies_to.is_empty() {
                "all".to_string()
            } else {
                playbook.applies_to.join(", ")
            };

            let tags_str = if playbook.tags.is_empty() {
                "(none)".dimmed().to_string()
            } else {
                playbook.tags.join(", ")
            };

            let description = if playbook.description.len() > 58 {
                format!("{}...", &playbook.description[..55])
            } else {
                playbook.description.clone()
            };

            println!(
                "{:<50} {:<60} {:<15} {:<20} {:<30}",
                playbook.uri, description, priority_str, scope_str, tags_str
            );
        }
    }

    Ok(())
}

/// Apply/install a playbook from file.
async fn apply_playbook(file: PathBuf) -> anyhow::Result<()> {
    // Parse the playbook file
    let playbook = PlaybookParser::parse_file(&file)
        .map_err(|e| anyhow::anyhow!("Failed to parse playbook file: {}", e))?;

    // Get the playbooks directory
    let playbooks_dir = PlaybookDiscovery::default_playbooks_dir()
        .map_err(|e| anyhow::anyhow!("Failed to get playbooks directory: {}", e))?;

    // Ensure directory exists
    std::fs::create_dir_all(&playbooks_dir)
        .map_err(|e| anyhow::anyhow!("Failed to create playbooks directory: {}", e))?;

    // Determine filename from URI or use original filename
    let filename = if let Some(uri_path) = playbook.uri.strip_prefix("radium://") {
        // Extract filename from URI (e.g., "org/playbook.md" -> "playbook.md")
        uri_path
            .split('/')
            .last()
            .unwrap_or_else(|| file.file_name().and_then(|n| n.to_str()).unwrap_or("playbook.md"))
    } else {
        file.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("playbook.md")
    };

    let target_path = playbooks_dir.join(filename);

    // Check if playbook already exists
    if target_path.exists() {
        return Err(anyhow::anyhow!(
            "Playbook with URI '{}' already exists at {}",
            playbook.uri,
            target_path.display()
        ));
    }

    // Save the playbook
    PlaybookStorage::save(&playbook, &target_path)
        .map_err(|e| anyhow::anyhow!("Failed to save playbook: {}", e))?;

    println!(
        "{} Playbook installed successfully!",
        "✓".green()
    );
    println!("URI: {}", playbook.uri.bright_blue());
    println!("Location: {}", target_path.display().bright_blue());

    Ok(())
}

/// Delete a playbook by URI.
async fn delete_playbook(uri: String) -> anyhow::Result<()> {
    let discovery = PlaybookDiscovery::new()
        .map_err(|e| anyhow::anyhow!("Failed to initialize playbook discovery: {}", e))?;

    // Find the playbook
    let playbook = discovery
        .find_by_uri(&uri)?
        .ok_or_else(|| anyhow::anyhow!("Playbook not found: {}", uri))?;

    // Get the playbooks directory
    let playbooks_dir = PlaybookDiscovery::default_playbooks_dir()
        .map_err(|e| anyhow::anyhow!("Failed to get playbooks directory: {}", e))?;

    // Find the file by scanning
    let all_playbooks = discovery.discover_all()?;
    let playbook_file = all_playbooks
        .get(&uri)
        .ok_or_else(|| anyhow::anyhow!("Playbook file not found: {}", uri))?;

    // We need to find the actual file path - scan the directory
    let mut found_path = None;
    for entry in std::fs::read_dir(&playbooks_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Ok(loaded) = PlaybookParser::parse_file(&path) {
                if loaded.uri == uri {
                    found_path = Some(path);
                    break;
                }
            }
        }
    }

    let file_path = found_path
        .ok_or_else(|| anyhow::anyhow!("Could not find file for playbook: {}", uri))?;

    // Delete the file
    PlaybookStorage::delete(&file_path)
        .map_err(|e| anyhow::anyhow!("Failed to delete playbook: {}", e))?;

    println!(
        "{} Playbook deleted successfully!",
        "✓".green()
    );
    println!("URI: {}", uri.bright_blue());

    Ok(())
}

/// Search playbooks by tag(s).
async fn search_playbooks(tags: String, json: bool) -> anyhow::Result<()> {
    let discovery = PlaybookDiscovery::new()
        .map_err(|e| anyhow::anyhow!("Failed to initialize playbook discovery: {}", e))?;

    let tag_list: Vec<String> = tags
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let playbooks = discovery.find_by_tags(&tag_list)?;

    if json {
        let playbooks_json: Vec<serde_json::Value> = playbooks
            .iter()
            .map(|p| {
                serde_json::json!({
                    "uri": p.uri,
                    "description": p.description,
                    "tags": p.tags,
                    "priority": p.priority.to_string(),
                    "applies_to": p.applies_to,
                })
            })
            .collect();
        println!("{}", serde_json::json!({ "playbooks": playbooks_json }));
    } else {
        if playbooks.is_empty() {
            println!("No playbooks found with tags: {}", tags);
            return Ok(());
        }

        println!("Playbooks matching tags: {}", tags.bright_blue());
        println!("{}", "=".repeat(50));
        println!();

        for playbook in playbooks {
            println!("{}", playbook.uri.bright_blue());
            println!("  Description: {}", playbook.description);
            println!("  Priority: {}", playbook.priority);
            if !playbook.applies_to.is_empty() {
                println!("  Scope: {}", playbook.applies_to.join(", "));
            }
            if !playbook.tags.is_empty() {
                println!("  Tags: {}", playbook.tags.join(", "));
            }
            println!();
        }
    }

    Ok(())
}

