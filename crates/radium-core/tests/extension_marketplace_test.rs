//! Tests for extension marketplace discovery API.

use radium_core::extensions::marketplace::{MarketplaceClient, MarketplaceError, MarketplaceExtension};
use std::time::Duration;

#[test]
fn test_marketplace_client_creation() {
    let client = MarketplaceClient::with_url("http://localhost:8080".to_string());
    assert!(client.is_ok());
}

#[test]
fn test_marketplace_client_default() {
    // This will fail if RADIUM_MARKETPLACE_URL is invalid, but should create client
    let client = MarketplaceClient::new();
    // Should succeed even if URL is not reachable (client creation doesn't connect)
    assert!(client.is_ok());
}

#[test]
fn test_cache_ttl() {
    let mut client = MarketplaceClient::with_url("http://localhost:8080".to_string()).unwrap();
    client.set_cache_ttl(Duration::from_secs(60));
    // Cache TTL should be set (no way to verify without making requests)
}

#[test]
fn test_marketplace_extension_serialization() {
    let ext = MarketplaceExtension {
        name: "test-extension".to_string(),
        version: "1.0.0".to_string(),
        description: "Test extension".to_string(),
        author: "Test Author".to_string(),
        download_url: "https://example.com/test.tar.gz".to_string(),
        download_count: Some(100),
        rating: Some(4.5),
        tags: vec!["test".to_string(), "example".to_string()],
        manifest: None,
    };

    // Test serialization
    let json = serde_json::to_string(&ext).unwrap();
    assert!(json.contains("test-extension"));
    assert!(json.contains("1.0.0"));

    // Test deserialization
    let deserialized: MarketplaceExtension = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "test-extension");
    assert_eq!(deserialized.version, "1.0.0");
    assert_eq!(deserialized.download_count, Some(100));
    assert_eq!(deserialized.rating, Some(4.5));
}

#[test]
fn test_marketplace_extension_without_optional_fields() {
    let ext = MarketplaceExtension {
        name: "simple-extension".to_string(),
        version: "2.0.0".to_string(),
        description: "Simple extension".to_string(),
        author: "Author".to_string(),
        download_url: "https://example.com/simple.tar.gz".to_string(),
        download_count: None,
        rating: None,
        tags: vec![],
        manifest: None,
    };

    // Should serialize/deserialize fine without optional fields
    let json = serde_json::to_string(&ext).unwrap();
    let deserialized: MarketplaceExtension = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "simple-extension");
    assert_eq!(deserialized.download_count, None);
    assert_eq!(deserialized.rating, None);
}

// Note: Integration tests with mock HTTP server would require additional dependencies
// like wiremock or httpmock. For now, we test the data structures and basic functionality.
// Full integration tests can be added when a marketplace server is available.

