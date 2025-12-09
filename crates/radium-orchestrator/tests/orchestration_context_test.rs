//! Integration tests for context file loading in orchestration

use radium_orchestrator::orchestration::context_loader::{ContextFileLoaderAdapter, ContextFileLoaderTrait};
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_context_loader_adapter() {
    let temp_dir = TempDir::new().unwrap();
    let adapter = ContextFileLoaderAdapter::new(temp_dir.path().to_path_buf());
    
    // Should not panic even if no context files exist
    let result = adapter.load_hierarchical(temp_dir.path());
    assert!(result.is_ok());
}

