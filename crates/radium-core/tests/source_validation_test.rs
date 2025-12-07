//! Integration tests for Source Validation gRPC endpoint.
//!
//! Tests the ValidateSources endpoint which validates accessibility of sources
//! across multiple protocols (file://, http://, jira://, braingrid://).

mod common;

use common::{create_test_client, start_test_server};
use radium_core::proto::ValidateSourcesRequest;
use std::fs;
use std::io::Write;
use tempfile::TempDir;

#[tokio::test]
async fn test_validate_sources_empty_list() {
    let port = start_test_server().await;
    let mut client = create_test_client(port).await;

    let request = tonic::Request::new(ValidateSourcesRequest { sources: vec![] });

    let response = client.validate_sources(request).await.expect("ValidateSources failed");
    let inner = response.into_inner();

    assert_eq!(inner.results.len(), 0);
    assert!(inner.all_valid, "Empty list should be considered all_valid");
}

#[tokio::test]
async fn test_validate_sources_single_valid_file() {
    let port = start_test_server().await;
    let mut client = create_test_client(port).await;

    // Create a temporary file for testing
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("test.txt");
    let mut file = fs::File::create(&test_file).expect("Failed to create test file");
    writeln!(file, "Test content").expect("Failed to write to test file");

    let file_uri = format!("file://{}", test_file.display());
    let request = tonic::Request::new(ValidateSourcesRequest { sources: vec![file_uri.clone()] });

    let response = client.validate_sources(request).await.expect("ValidateSources failed");
    let inner = response.into_inner();

    assert_eq!(inner.results.len(), 1);
    assert!(inner.all_valid, "Single valid file should result in all_valid=true");

    let result = &inner.results[0];
    assert_eq!(result.source, file_uri);
    assert!(result.accessible, "File should be accessible");
    assert_eq!(result.error_message, "", "No error message for valid source");
    assert!(result.size_bytes > 0, "File should have non-zero size");
}

#[tokio::test]
async fn test_validate_sources_single_invalid_file() {
    let port = start_test_server().await;
    let mut client = create_test_client(port).await;

    let file_uri = "file:///nonexistent/path/to/file.txt".to_string();
    let request = tonic::Request::new(ValidateSourcesRequest { sources: vec![file_uri.clone()] });

    let response = client.validate_sources(request).await.expect("ValidateSources failed");
    let inner = response.into_inner();

    assert_eq!(inner.results.len(), 1);
    assert!(!inner.all_valid, "Nonexistent file should result in all_valid=false");

    let result = &inner.results[0];
    assert_eq!(result.source, file_uri);
    assert!(!result.accessible, "Nonexistent file should not be accessible");
    assert!(!result.error_message.is_empty(), "Error message should be present");
    assert!(
        result.error_message.to_lowercase().contains("not found"),
        "Error message should mention 'not found', got: {}",
        result.error_message
    );
    assert_eq!(result.size_bytes, 0, "Nonexistent file should have zero size");
}

#[tokio::test]
async fn test_validate_sources_mixed_results() {
    let port = start_test_server().await;
    let mut client = create_test_client(port).await;

    // Create one valid file
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let valid_file = temp_dir.path().join("valid.txt");
    let mut file = fs::File::create(&valid_file).expect("Failed to create test file");
    writeln!(file, "Valid content").expect("Failed to write to test file");

    let valid_uri = format!("file://{}", valid_file.display());
    let invalid_uri = "file:///nonexistent/invalid.txt".to_string();

    let request =
        tonic::Request::new(ValidateSourcesRequest { sources: vec![valid_uri.clone(), invalid_uri.clone()] });

    let response = client.validate_sources(request).await.expect("ValidateSources failed");
    let inner = response.into_inner();

    assert_eq!(inner.results.len(), 2);
    assert!(!inner.all_valid, "Mixed results should have all_valid=false");

    // Find results (order might vary due to concurrent processing)
    let valid_result = inner.results.iter().find(|r| r.source == valid_uri).expect("Valid result not found");
    let invalid_result =
        inner.results.iter().find(|r| r.source == invalid_uri).expect("Invalid result not found");

    assert!(valid_result.accessible, "Valid file should be accessible");
    assert_eq!(valid_result.error_message, "");
    assert!(valid_result.size_bytes > 0);

    assert!(!invalid_result.accessible, "Invalid file should not be accessible");
    assert!(!invalid_result.error_message.is_empty());
    assert_eq!(invalid_result.size_bytes, 0);
}

#[tokio::test]
async fn test_validate_sources_multiple_valid_files() {
    let port = start_test_server().await;
    let mut client = create_test_client(port).await;

    // Create multiple valid files
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut sources = Vec::new();

    for i in 1..=3 {
        let file_path = temp_dir.path().join(format!("file{}.txt", i));
        let mut file = fs::File::create(&file_path).expect("Failed to create test file");
        writeln!(file, "Content {}", i).expect("Failed to write to test file");
        sources.push(format!("file://{}", file_path.display()));
    }

    let request = tonic::Request::new(ValidateSourcesRequest { sources: sources.clone() });

    let response = client.validate_sources(request).await.expect("ValidateSources failed");
    let inner = response.into_inner();

    assert_eq!(inner.results.len(), 3);
    assert!(inner.all_valid, "All valid files should result in all_valid=true");

    // Verify all results are accessible
    for result in &inner.results {
        assert!(result.accessible, "File {} should be accessible", result.source);
        assert_eq!(result.error_message, "");
        assert!(result.size_bytes > 0);
    }
}

#[tokio::test]
async fn test_validate_sources_http_invalid_uri() {
    let port = start_test_server().await;
    let mut client = create_test_client(port).await;

    // Use an invalid URI format (not a proper URL)
    let invalid_http_uri = "http://".to_string();
    let request =
        tonic::Request::new(ValidateSourcesRequest { sources: vec![invalid_http_uri.clone()] });

    let response = client.validate_sources(request).await.expect("ValidateSources failed");
    let inner = response.into_inner();

    assert_eq!(inner.results.len(), 1);
    assert!(!inner.all_valid);

    let result = &inner.results[0];
    assert_eq!(result.source, invalid_http_uri);
    assert!(!result.accessible);
    assert!(!result.error_message.is_empty());
}

#[tokio::test]
async fn test_validate_sources_path_without_scheme() {
    let port = start_test_server().await;
    let mut client = create_test_client(port).await;

    // Create a file and test with path (no scheme)
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("test_no_scheme.txt");
    let mut file = fs::File::create(&test_file).expect("Failed to create test file");
    writeln!(file, "Test").expect("Failed to write to test file");

    // Use absolute path without file:// scheme
    let path_str = test_file.to_str().expect("Invalid path").to_string();
    let request = tonic::Request::new(ValidateSourcesRequest { sources: vec![path_str.clone()] });

    let response = client.validate_sources(request).await.expect("ValidateSources failed");
    let inner = response.into_inner();

    assert_eq!(inner.results.len(), 1);
    assert!(inner.all_valid, "Path without scheme should default to file:// and be valid");

    let result = &inner.results[0];
    assert_eq!(result.source, path_str);
    assert!(result.accessible, "File should be accessible via path without scheme");
    assert_eq!(result.error_message, "");
}

#[tokio::test]
async fn test_validate_sources_unsupported_scheme() {
    let port = start_test_server().await;
    let mut client = create_test_client(port).await;

    let unsupported_uri = "ftp://example.com/file.txt".to_string();
    let request =
        tonic::Request::new(ValidateSourcesRequest { sources: vec![unsupported_uri.clone()] });

    let response = client.validate_sources(request).await.expect("ValidateSources failed");
    let inner = response.into_inner();

    assert_eq!(inner.results.len(), 1);
    assert!(!inner.all_valid);

    let result = &inner.results[0];
    assert_eq!(result.source, unsupported_uri);
    assert!(!result.accessible);
    assert!(!result.error_message.is_empty());
    assert!(
        result.error_message.contains("No reader registered"),
        "Error should mention no reader registered, got: {}",
        result.error_message
    );
}
