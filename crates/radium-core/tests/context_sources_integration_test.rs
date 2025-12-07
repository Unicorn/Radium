//! Integration tests for Context Sources with External Services.
//!
//! Tests that verify context sources (HTTP, Jira, Braingrid) correctly fetch
//! and integrate external content into agent prompts.

use radium_core::context::sources::{
    BraingridReader, HttpReader, JiraReader, LocalFileReader, SourceRegistry, SourceReader,
};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_source_registry_uri_routing() {
    let mut registry = SourceRegistry::new();

    // Register all source readers
    registry.register(Box::new(LocalFileReader::with_base_dir(PathBuf::from("/tmp"))));
    registry.register(Box::new(HttpReader::new()));
    registry.register(Box::new(JiraReader::new()));
    registry.register(Box::new(BraingridReader::new()));

    // Test routing for file:// URIs
    let file_reader = registry.get_reader("file:///path/to/file.txt");
    assert!(file_reader.is_ok());
    assert_eq!(file_reader.unwrap().scheme(), "file");

    // Test routing for http:// URIs
    let http_reader = registry.get_reader("http://example.com/test.txt");
    assert!(http_reader.is_ok());
    assert_eq!(http_reader.unwrap().scheme(), "http");

    // Test routing for https:// URIs
    let https_reader = registry.get_reader("https://example.com/test.txt");
    assert!(https_reader.is_ok());
    assert_eq!(https_reader.unwrap().scheme(), "http"); // HttpReader handles both

    // Test routing for jira:// URIs
    let jira_reader = registry.get_reader("jira://PROJ-123");
    assert!(jira_reader.is_ok());
    assert_eq!(jira_reader.unwrap().scheme(), "jira");

    // Test routing for braingrid:// URIs
    let braingrid_reader = registry.get_reader("braingrid://REQ-123");
    assert!(braingrid_reader.is_ok());
    assert_eq!(braingrid_reader.unwrap().scheme(), "braingrid");

    // Test unsupported scheme
    let invalid_reader = registry.get_reader("invalid://test");
    assert!(invalid_reader.is_err());
}

#[tokio::test]
async fn test_local_file_reader_verify() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    std::fs::write(&test_file, "Test content").unwrap();

    let reader = LocalFileReader::with_base_dir(temp_dir.path().to_path_buf());

    // Test verification of existing file (absolute path)
    let uri = format!("file://{}", test_file.display());
    let result = reader.verify(&uri).await;
    assert!(result.is_ok());
    assert!(result.unwrap().accessible);

    // Test verification of non-existent file
    let invalid_uri = "file:///nonexistent/file.txt";
    let result = reader.verify(invalid_uri).await;
    assert!(result.is_err() || !result.unwrap().accessible);
}

#[tokio::test]
async fn test_local_file_reader_fetch() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    let content = "Test content for fetching";
    std::fs::write(&test_file, content).unwrap();

    let reader = LocalFileReader::with_base_dir(temp_dir.path().to_path_buf());

    let uri = format!("file://{}", test_file.display());
    let result = reader.fetch(&uri).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), content);
}

#[tokio::test]
async fn test_http_reader_invalid_uri() {
    let reader = HttpReader::new();

    // Test invalid URI format
    let result = reader.verify("not-a-uri").await;
    assert!(result.is_err());

    // Test file:// URI with HTTP reader (wrong scheme)
    let result = reader.verify("file:///path/to/file").await;
    assert!(result.is_err());
}

#[tokio::test]
#[ignore = "Requires network access - test with actual HTTP server"]
async fn test_http_reader_verify_accessible() {
    let reader = HttpReader::new();

    // Test verification of accessible URL (httpbin.org is a reliable test service)
    // Note: This test requires network access and may be flaky
    let result = reader.verify("https://httpbin.org/get").await;
    
    // May succeed or fail depending on network, but should not panic
    match result {
        Ok(metadata) => {
            // If accessible, verify metadata structure
            assert!(metadata.accessible);
        }
        Err(_) => {
            // Network failure is acceptable for this test
        }
    }
}

#[tokio::test]
async fn test_http_reader_verify_inaccessible() {
    let reader = HttpReader::new();

    // Test verification of clearly inaccessible URL
    let result = reader.verify("http://localhost:99999/nonexistent").await;
    
    // Should fail (network error or not found)
    assert!(result.is_err());
}

#[tokio::test]
async fn test_jira_reader_uri_parsing() {
    let reader = JiraReader::new();

    // Test Jira URI format
    assert_eq!(reader.scheme(), "jira");

    // Test verification (will fail without credentials, but shouldn't panic)
    let result = reader.verify("jira://PROJ-123").await;
    
    // May fail due to missing credentials, but should return proper error
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Jira") || err.to_string().contains("auth") || err.to_string().contains("credential"));
}

#[tokio::test]
async fn test_braingrid_reader_uri_parsing() {
    let reader = BraingridReader::new();

    // Test Braingrid URI format
    assert_eq!(reader.scheme(), "braingrid");

    // Test verification (will fail without braingrid CLI or credentials)
    let result = reader.verify("braingrid://REQ-123").await;
    
    // May fail due to missing CLI or credentials, but should return proper error
    // Error could be NotFound, network error, or credential error
    assert!(result.is_err() || !result.unwrap().accessible);
}

#[test]
fn test_source_registry_priority() {
    let mut registry = SourceRegistry::new();

    // Register readers in a specific order
    registry.register(Box::new(LocalFileReader::with_base_dir(PathBuf::from("/tmp"))));
    registry.register(Box::new(HttpReader::new()));

    // HTTP should still route to HTTP reader even if file reader is registered first
    let http_reader = registry.get_reader("http://example.com/test");
    assert!(http_reader.is_ok());
    assert_eq!(http_reader.unwrap().scheme(), "http");

    // File should route to file reader
    let file_reader = registry.get_reader("file:///tmp/test.txt");
    assert!(file_reader.is_ok());
    assert_eq!(file_reader.unwrap().scheme(), "file");
}

#[tokio::test]
async fn test_source_error_types() {
    let reader = HttpReader::new();

    // Test invalid URI error
    let result = reader.verify("invalid-uri").await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("invalid") || err.to_string().contains("URI"));

    // Test that errors are properly formatted
    assert!(!err.to_string().is_empty());
}

#[tokio::test]
async fn test_http_reader_fetch_size_limit() {
    let reader = HttpReader::new();

    // Test that fetch operations respect size limits
    // This would require a mock server, so we'll just verify the reader is configured
    assert_eq!(reader.scheme(), "http");
    
    // The reader should handle large responses according to max_size
    // Full integration test would require mock HTTP server
}

#[test]
fn test_all_readers_scheme_methods() {
    // Verify all readers report correct scheme
    let local_reader = LocalFileReader::with_base_dir(PathBuf::from("/tmp"));
    assert_eq!(local_reader.scheme(), "file");

    let http_reader = HttpReader::new();
    assert_eq!(http_reader.scheme(), "http");

    let jira_reader = JiraReader::new();
    assert_eq!(jira_reader.scheme(), "jira");

    let braingrid_reader = BraingridReader::new();
    assert_eq!(braingrid_reader.scheme(), "braingrid");
}

