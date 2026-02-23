//! Object Storage component schema
//!
//! The Object Storage component performs operations against S3-compatible
//! object storage providers including AWS S3, Cloudflare R2, Google Cloud
//! Storage, and self-hosted MinIO. Supports Get, Put, Delete, List, Copy,
//! and Head operations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

use super::behaviors::{ComponentBehaviors, RateLimitConfig};

/// Object storage provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum StorageProvider {
    /// Amazon Web Services S3 (default).
    #[default]
    S3,
    /// Cloudflare R2 (S3-compatible).
    R2,
    /// Google Cloud Storage.
    Gcs,
    /// Self-hosted MinIO (S3-compatible).
    MinIo,
}

/// Object storage operation to perform.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum StorageAction {
    /// Retrieve an object by key (default).
    #[default]
    Get,
    /// Upload or overwrite an object.
    Put,
    /// Remove an object by key.
    Delete,
    /// List objects in a bucket, optionally filtered by prefix.
    List,
    /// Copy an object from one key to another.
    Copy,
    /// Retrieve object metadata without downloading the body.
    Head,
}

/// Metadata about a single object returned from a List operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ObjectInfo {
    /// Object key (path).
    pub key: String,

    /// Size of the object in bytes.
    pub size: u64,

    /// RFC 3339 timestamp of the last modification.
    pub last_modified: String,

    /// ETag of the object (may be absent for some providers).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,
}

/// Object Storage component input.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct ObjectStorageInput {
    /// Target storage provider.
    #[serde(default)]
    pub provider: StorageProvider,

    /// Bucket name.
    #[validate(length(min = 1, message = "bucket must not be empty"))]
    pub bucket: String,

    /// Object key (path within the bucket).
    #[validate(length(min = 1, message = "key must not be empty"))]
    pub key: String,

    /// Operation to perform.
    #[serde(default)]
    pub action: StorageAction,

    /// Raw content body for Put operations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,

    /// MIME type to set on the object for Put operations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,

    /// Arbitrary key-value metadata to attach to the object.
    #[serde(default)]
    pub metadata: HashMap<String, String>,

    /// Source object key for Copy operations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copy_source: Option<String>,

    /// Key prefix filter for List operations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,

    /// Maximum number of objects to return for List operations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_keys: Option<u32>,

    /// AWS region override (uses provider default when absent).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,

    /// Custom endpoint URL for MinIO or R2 deployments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint_url: Option<String>,

    /// Shared component behaviors (retry, rate limit, timeout, etc.).
    #[serde(default = "object_storage_default_behaviors")]
    #[validate(nested)]
    pub behaviors: ComponentBehaviors,
}

fn object_storage_default_behaviors() -> ComponentBehaviors {
    ComponentBehaviors {
        timeout_ms: 120_000,
        heartbeat_interval_ms: Some(10_000),
        rate_limit: RateLimitConfig {
            requests_per_second: 20,
            burst: 40,
            ..Default::default()
        },
        ..Default::default()
    }
}

impl Default for ObjectStorageInput {
    fn default() -> Self {
        Self {
            provider: StorageProvider::default(),
            bucket: String::new(),
            key: String::new(),
            action: StorageAction::default(),
            body: None,
            content_type: None,
            metadata: HashMap::new(),
            copy_source: None,
            prefix: None,
            max_keys: None,
            region: None,
            endpoint_url: None,
            behaviors: object_storage_default_behaviors(),
        }
    }
}

/// Object Storage component output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ObjectStorageOutput {
    /// Object content returned by a Get operation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,

    /// MIME type of the retrieved or inspected object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,

    /// Size of the object in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_length: Option<u64>,

    /// ETag of the object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,

    /// RFC 3339 timestamp of the last modification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<String>,

    /// Objects returned by a List operation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub objects: Option<Vec<ObjectInfo>>,

    /// Whether the object was successfully deleted.
    #[serde(default)]
    pub deleted: bool,
}

impl Default for ObjectStorageOutput {
    fn default() -> Self {
        Self {
            body: None,
            content_type: None,
            content_length: None,
            etag: None,
            last_modified: None,
            objects: None,
            deleted: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_with_defaults() {
        let input = ObjectStorageInput {
            bucket: "my-bucket".to_string(),
            key: "path/to/object.txt".to_string(),
            ..Default::default()
        };

        assert_eq!(input.provider, StorageProvider::S3);
        assert_eq!(input.action, StorageAction::Get);
        assert!(input.body.is_none());
        assert!(input.content_type.is_none());
        assert!(input.metadata.is_empty());
        assert!(input.copy_source.is_none());
        assert!(input.prefix.is_none());
        assert!(input.max_keys.is_none());
        assert!(input.region.is_none());
        assert!(input.endpoint_url.is_none());
        assert_eq!(input.behaviors.timeout_ms, 120_000);
        assert_eq!(input.behaviors.heartbeat_interval_ms, Some(10_000));
        assert_eq!(input.behaviors.rate_limit.requests_per_second, 20);
        assert_eq!(input.behaviors.rate_limit.burst, 40);
    }

    #[test]
    fn test_full_config_deserialization() {
        let yaml = r#"
provider: r2
bucket: "assets-prod"
key: "images/logo.png"
action: put
body: "base64encodedcontent"
content_type: "image/png"
metadata:
  uploaded_by: "workflow-engine"
  project: "unicorn"
copy_source: "images/logo-old.png"
prefix: "images/"
max_keys: 100
region: "us-east-1"
endpoint_url: "https://my-r2-account.r2.cloudflarestorage.com"
"#;
        let input: ObjectStorageInput = serde_yaml::from_str(yaml).expect("deserialize");

        assert_eq!(input.provider, StorageProvider::R2);
        assert_eq!(input.bucket, "assets-prod");
        assert_eq!(input.key, "images/logo.png");
        assert_eq!(input.action, StorageAction::Put);
        assert_eq!(input.body.as_deref(), Some("base64encodedcontent"));
        assert_eq!(input.content_type.as_deref(), Some("image/png"));
        assert_eq!(input.metadata.get("uploaded_by").map(String::as_str), Some("workflow-engine"));
        assert_eq!(input.metadata.get("project").map(String::as_str), Some("unicorn"));
        assert_eq!(input.copy_source.as_deref(), Some("images/logo-old.png"));
        assert_eq!(input.prefix.as_deref(), Some("images/"));
        assert_eq!(input.max_keys, Some(100));
        assert_eq!(input.region.as_deref(), Some("us-east-1"));
        assert!(input.endpoint_url.is_some());
    }

    #[test]
    fn test_output_serialize_deserialize() {
        let output = ObjectStorageOutput {
            body: Some("hello world".to_string()),
            content_type: Some("text/plain".to_string()),
            content_length: Some(11),
            etag: Some("\"abc123\"".to_string()),
            last_modified: Some("2024-01-15T10:30:00Z".to_string()),
            objects: Some(vec![
                ObjectInfo {
                    key: "docs/readme.txt".to_string(),
                    size: 1024,
                    last_modified: "2024-01-10T08:00:00Z".to_string(),
                    etag: Some("\"def456\"".to_string()),
                },
                ObjectInfo {
                    key: "docs/guide.md".to_string(),
                    size: 4096,
                    last_modified: "2024-01-12T14:00:00Z".to_string(),
                    etag: None,
                },
            ]),
            deleted: false,
        };

        let json = serde_json::to_string(&output).expect("serialize");
        let restored: ObjectStorageOutput = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(restored.body.as_deref(), Some("hello world"));
        assert_eq!(restored.content_type.as_deref(), Some("text/plain"));
        assert_eq!(restored.content_length, Some(11));
        assert_eq!(restored.etag.as_deref(), Some("\"abc123\""));
        assert_eq!(restored.last_modified.as_deref(), Some("2024-01-15T10:30:00Z"));
        let objects = restored.objects.as_ref().expect("objects present");
        assert_eq!(objects.len(), 2);
        assert_eq!(objects[0].key, "docs/readme.txt");
        assert_eq!(objects[0].size, 1024);
        assert!(objects[0].etag.is_some());
        assert!(objects[1].etag.is_none());
        assert!(!restored.deleted);
    }

    #[test]
    fn test_storage_action_default() {
        let action = StorageAction::default();
        assert_eq!(action, StorageAction::Get);

        // Verify all variants round-trip through serde
        let variants = [
            StorageAction::Get,
            StorageAction::Put,
            StorageAction::Delete,
            StorageAction::List,
            StorageAction::Copy,
            StorageAction::Head,
        ];
        for variant in &variants {
            let json = serde_json::to_string(variant).expect("serialize");
            let restored: StorageAction = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(&restored, variant);
        }
    }

    #[test]
    fn test_storage_provider_default() {
        let provider = StorageProvider::default();
        assert_eq!(provider, StorageProvider::S3);

        // Verify all variants round-trip through serde
        let variants = [
            StorageProvider::S3,
            StorageProvider::R2,
            StorageProvider::Gcs,
            StorageProvider::MinIo,
        ];
        for variant in &variants {
            let json = serde_json::to_string(variant).expect("serialize");
            let restored: StorageProvider = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(&restored, variant);
        }
    }
}
