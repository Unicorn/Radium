//! Local file source reader.

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use super::traits::SourceReader;
use super::types::{SourceError, SourceMetadata};

/// Reader for local file sources (file:// scheme or relative paths).
pub struct LocalFileReader {
    /// Base directory for resolving relative paths.
    base_dir: PathBuf,
}

impl LocalFileReader {
    /// Creates a new local file reader with the current working directory as base.
    pub fn new() -> Self {
        Self {
            base_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        }
    }

    /// Creates a new local file reader with a specific base directory.
    pub fn with_base_dir(base_dir: impl AsRef<Path>) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
        }
    }

    /// Resolves a URI or path to an absolute PathBuf.
    fn resolve_path(&self, uri: &str) -> PathBuf {
        // Remove file:// scheme if present
        let path_str = uri.strip_prefix("file://").unwrap_or(uri);
        let path = PathBuf::from(path_str);

        if path.is_absolute() {
            path
        } else {
            self.base_dir.join(path)
        }
    }
}

impl Default for LocalFileReader {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SourceReader for LocalFileReader {
    fn scheme(&self) -> &str {
        "file"
    }

    async fn verify(&self, uri: &str) -> Result<SourceMetadata, SourceError> {
        let path = self.resolve_path(uri);

        // Check if file exists and is readable
        match tokio::fs::metadata(&path).await {
            Ok(metadata) => {
                if !metadata.is_file() {
                    return Err(SourceError::not_found(&format!(
                        "Path exists but is not a file: {}",
                        path.display()
                    )));
                }

                // Extract file size
                let size_bytes = Some(metadata.len());

                // Extract last modified time
                let last_modified = metadata.modified().ok().and_then(|time| {
                    time.duration_since(std::time::UNIX_EPOCH)
                        .ok()
                        .and_then(|duration| {
                            DateTime::<Utc>::from_timestamp(
                                duration.as_secs() as i64,
                                duration.subsec_nanos(),
                            )
                        })
                        .map(|dt| dt.to_rfc3339())
                });

                // For local files, content type is typically determined by extension
                // We'll leave it as None for now - can be enhanced later
                let content_type = None;

                Ok(SourceMetadata::with_details(true, size_bytes, last_modified, content_type))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Err(SourceError::not_found(&format!("File not found: {}", path.display())))
            }
            Err(e) => Err(SourceError::IoError(e)),
        }
    }

    async fn fetch(&self, uri: &str) -> Result<String, SourceError> {
        let path = self.resolve_path(uri);

        tokio::fs::read_to_string(&path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                SourceError::not_found(&format!("File not found: {}", path.display()))
            } else {
                SourceError::IoError(e)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_verify_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        tokio::fs::write(&file_path, "test content").await.unwrap();

        let reader = LocalFileReader::with_base_dir(temp_dir.path());
        let result = reader.verify("test.txt").await;

        assert!(result.is_ok());
        let metadata = result.unwrap();
        assert!(metadata.accessible);
        assert!(metadata.size_bytes.is_some());
        assert_eq!(metadata.size_bytes.unwrap(), 12);
    }

    #[tokio::test]
    async fn test_verify_nonexistent_file() {
        let reader = LocalFileReader::new();
        let result = reader.verify("nonexistent.txt").await;

        assert!(result.is_err());
        match result.unwrap_err() {
            SourceError::NotFound(_) => {}
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_fetch_file_content() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let content = "test content\nwith multiple lines";
        tokio::fs::write(&file_path, content).await.unwrap();

        let reader = LocalFileReader::with_base_dir(temp_dir.path());
        let result = reader.fetch("test.txt").await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), content);
    }

    #[tokio::test]
    async fn test_file_scheme_uri() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        tokio::fs::write(&file_path, "content").await.unwrap();

        let reader = LocalFileReader::with_base_dir(temp_dir.path());
        let uri = format!("file://{}", file_path.display());
        let result = reader.verify(&uri).await;

        assert!(result.is_ok());
        assert!(result.unwrap().accessible);
    }
}
