//! Helper functions for source validation.

use anyhow::Context;
use radium_core::context::sources::{BraingridReader, HttpReader, JiraReader, LocalFileReader};
use radium_core::context::{SourceRegistry, SourceValidator, SourceValidationResult};
use radium_core::Workspace;
use std::path::PathBuf;

/// Validates a list of sources using the source validator.
///
/// Creates a validator with all readers configured for the current workspace.
pub async fn validate_sources(
    sources: Vec<String>,
    workspace_root: Option<PathBuf>,
) -> anyhow::Result<Vec<SourceValidationResult>> {
    // Determine workspace root
    let root = if let Some(ws_root) = workspace_root {
        ws_root
    } else {
        Workspace::discover()
            .map(|w| w.root().to_path_buf())
            .unwrap_or_else(|_| {
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
            })
    };

    // Create and initialize source registry with all readers
    let mut registry = SourceRegistry::new();
    registry.register(Box::new(LocalFileReader::with_base_dir(&root)));
    registry.register(Box::new(HttpReader::new()));
    registry.register(Box::new(JiraReader::new()));
    registry.register(Box::new(BraingridReader::new()));

    // Create source validator
    let validator = SourceValidator::new(registry);

    // Validate sources concurrently
    let results = validator.validate_sources(sources).await;

    Ok(results)
}
