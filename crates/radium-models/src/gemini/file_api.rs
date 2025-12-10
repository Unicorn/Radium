//! Gemini File API implementation.
//!
//! This module provides functionality for uploading, managing, and deleting files
//! via the Gemini File API. Files uploaded through this API can be used in
//! multimodal prompts for large media files (>20MB) that cannot be sent inline.

use chrono::{DateTime, Utc};
use radium_abstraction::ModelError;
use reqwest::multipart::{Form, Part};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// File state as returned by the Gemini File API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FileState {
    /// File is being processed by Gemini.
    Processing,
    /// File is ready to use.
    Active,
    /// File processing failed.
    Failed,
}

/// Represents a file uploaded to the Gemini File API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiFile {
    /// File name/ID in format "files/{file-id}".
    pub name: String,
    /// Full URI for accessing the file.
    pub uri: String,
    /// Current state of the file.
    #[serde(deserialize_with = "deserialize_state")]
    pub state: FileState,
    /// Expiration time (48 hours from creation).
    #[serde(rename = "expire_time", deserialize_with = "deserialize_datetime")]
    pub expire_time: DateTime<Utc>,
    /// File size in bytes.
    #[serde(rename = "size_bytes")]
    pub size_bytes: u64,
    /// Optional display name for the file.
    #[serde(rename = "display_name", default)]
    pub display_name: Option<String>,
    /// MIME type of the file.
    #[serde(rename = "mime_type")]
    pub mime_type: String,
}

/// Helper function to deserialize state string to FileState enum.
fn deserialize_state<'de, D>(deserializer: D) -> Result<FileState, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "PROCESSING" => Ok(FileState::Processing),
        "ACTIVE" => Ok(FileState::Active),
        "FAILED" => Ok(FileState::Failed),
        _ => Err(serde::de::Error::custom(format!("Unknown file state: {}", s))),
    }
}

/// Helper function to deserialize datetime string to DateTime<Utc>.
fn deserialize_datetime<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    DateTime::parse_from_rfc3339(&s)
        .map_err(|e| serde::de::Error::custom(format!("Failed to parse datetime: {}", e)))
        .map(|dt| dt.with_timezone(&Utc))
}

/// Client for interacting with the Gemini File API.
pub struct GeminiFileApi {
    /// API key for authentication.
    api_key: String,
    /// HTTP client for making requests.
    http_client: Client,
    /// Base URL for the File API.
    base_url: String,
}

impl GeminiFileApi {
    /// Creates a new `GeminiFileApi` instance with the given API key.
    ///
    /// # Arguments
    /// * `api_key` - The Gemini API key for authentication
    #[must_use]
    pub fn with_api_key(api_key: String) -> Self {
        Self {
            api_key,
            http_client: Client::new(),
            base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
        }
    }

    /// Uploads a file to the Gemini File API.
    ///
    /// # Arguments
    /// * `file_path` - Path to the file to upload
    /// * `mime_type` - Optional MIME type (detected from extension if not provided)
    /// * `display_name` - Optional display name for the file
    ///
    /// # Errors
    /// Returns `ModelError` if the file cannot be read, uploaded, or processed.
    pub async fn upload_file(
        &self,
        file_path: &Path,
        mime_type: Option<String>,
        display_name: Option<String>,
    ) -> Result<GeminiFile, ModelError> {
        debug!(path = %file_path.display(), "Uploading file to Gemini File API");

        // Validate file exists and is readable
        if !file_path.exists() {
            return Err(ModelError::InvalidMediaSource {
                media_source: file_path.display().to_string(),
                reason: "File does not exist".to_string(),
            });
        }

        // Read file contents
        let file_bytes = tokio::fs::read(file_path).await.map_err(|e| {
            ModelError::InvalidMediaSource {
                media_source: file_path.display().to_string(),
                reason: format!("Failed to read file: {}", e),
            }
        })?;

        // Warn if file is smaller than 20MB (File API is intended for large files)
        const SIZE_THRESHOLD: u64 = 20 * 1024 * 1024; // 20MB
        if (file_bytes.len() as u64) < SIZE_THRESHOLD {
            warn!(
                path = %file_path.display(),
                size = file_bytes.len(),
                "File is smaller than 20MB; consider using inline base64 encoding instead"
            );
        }

        // Get filename from path
        let file_name = file_path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| {
                ModelError::InvalidMediaSource {
                    media_source: file_path.display().to_string(),
                    reason: "Invalid filename".to_string(),
                }
            })?;

        // Determine MIME type
        let mime = mime_type.unwrap_or_else(|| detect_mime_type_from_extension(file_path));

        // Build multipart form
        let mut form = Form::new().part(
            "file",
            Part::bytes(file_bytes)
                .file_name(file_name.to_string())
                .mime_str(&mime)
                .map_err(|e| {
                    ModelError::RequestError(format!("Failed to set MIME type: {}", e))
                })?,
        );

        // Add display name if provided
        if let Some(display) = display_name {
            form = form.text("display_name", display);
        }

        // Construct upload URL
        let upload_url = format!(
            "https://generativelanguage.googleapis.com/upload/v1beta/files?key={}",
            self.api_key
        );

        // Make upload request
        let response = self
            .http_client
            .post(&upload_url)
            .header("X-Goog-Upload-Protocol", "multipart")
            .multipart(form)
            .send()
            .await
            .map_err(|e| {
                ModelError::RequestError(format!("Failed to upload file: {}", e))
            })?;

        // Check response status and map to appropriate error
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(map_http_error(status, &error_text, "file upload"));
        }

        // Parse response
        let file: GeminiFile = response.json().await.map_err(|e| {
            ModelError::SerializationError(format!("Failed to parse upload response: {}", e))
        })?;

        debug!(
            file_name = %file.name,
            state = ?file.state,
            "File uploaded successfully"
        );

        // Poll until file is ACTIVE
        if file.state == FileState::Processing {
            self.poll_until_active(&file.name).await
        } else {
            Ok(file)
        }
    }

    /// Retrieves file metadata by name/ID.
    ///
    /// # Arguments
    /// * `file_name` - File name/ID in format "files/{file-id}"
    ///
    /// # Errors
    /// Returns `ModelError` if the file cannot be retrieved.
    pub async fn get_file(&self, file_name: &str) -> Result<GeminiFile, ModelError> {
        debug!(file_name = %file_name, "Retrieving file metadata");

        let url = format!(
            "{}/{}?key={}",
            self.base_url, file_name, self.api_key
        );

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| {
                ModelError::RequestError(format!("Failed to retrieve file: {}", e))
            })?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(map_http_error(status, &error_text, &format!("retrieve file {}", file_name)));
        }

        let file: GeminiFile = response.json().await.map_err(|e| {
            ModelError::SerializationError(format!("Failed to parse file response: {}", e))
        })?;

        Ok(file)
    }

    /// Polls file state until it becomes ACTIVE or fails.
    ///
    /// Uses exponential backoff: 1s → 2s → 4s → 8s → 10s (cap).
    /// Times out after 5 minutes (300 seconds).
    ///
    /// # Arguments
    /// * `file_name` - File name/ID to poll
    ///
    /// # Errors
    /// Returns `ModelError` if polling times out or file fails.
    async fn poll_until_active(&self, file_name: &str) -> Result<GeminiFile, ModelError> {
        debug!(file_name = %file_name, "Starting state polling");

        let start_time = Instant::now();
        let timeout = Duration::from_secs(300); // 5 minutes
        let mut delay = Duration::from_secs(1);
        let max_delay = Duration::from_secs(10);

        loop {
            // Check timeout
            if start_time.elapsed() > timeout {
                return Err(ModelError::RequestError(format!(
                    "File processing timeout after 5 minutes: {}",
                    file_name
                )));
            }

            // Get current file state
            let file = self.get_file(file_name).await?;

            match file.state {
                FileState::Active => {
                    debug!(file_name = %file_name, "File is now ACTIVE");
                    return Ok(file);
                }
                FileState::Failed => {
                    return Err(ModelError::ModelResponseError(format!(
                        "File processing failed: {}",
                        file_name
                    )));
                }
                FileState::Processing => {
                    debug!(
                        file_name = %file_name,
                        elapsed = ?start_time.elapsed(),
                        "File still processing, waiting..."
                    );
                    // Wait with exponential backoff
                    tokio::time::sleep(delay).await;
                    delay = (delay * 2).min(max_delay);
                }
            }
        }
    }

    /// Lists all files uploaded to the Gemini File API.
    ///
    /// # Errors
    /// Returns `ModelError` if the list request fails.
    pub async fn list_files(&self) -> Result<Vec<GeminiFile>, ModelError> {
        debug!("Listing all files");

        let url = format!("{}/files?key={}", self.base_url, self.api_key);

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| {
                ModelError::RequestError(format!("Failed to list files: {}", e))
            })?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(map_http_error(status, &error_text, "list files"));
        }

        #[derive(Deserialize)]
        struct ListFilesResponse {
            files: Vec<GeminiFile>,
        }

        let list_response: ListFilesResponse = response.json().await.map_err(|e| {
            ModelError::SerializationError(format!("Failed to parse files list: {}", e))
        })?;

        debug!(count = list_response.files.len(), "Retrieved file list");
        Ok(list_response.files)
    }

    /// Deletes a file from the Gemini File API.
    ///
    /// # Arguments
    /// * `file_name` - File name/ID in format "files/{file-id}"
    ///
    /// # Errors
    /// Returns `ModelError` if deletion fails (except 404, which is handled gracefully).
    pub async fn delete_file(&self, file_name: &str) -> Result<(), ModelError> {
        debug!(file_name = %file_name, "Deleting file");

        let url = format!(
            "{}/{}?key={}",
            self.base_url, file_name, self.api_key
        );

        let response = self
            .http_client
            .delete(&url)
            .send()
            .await
            .map_err(|e| {
                ModelError::RequestError(format!("Failed to delete file: {}", e))
            })?;

        let status = response.status();
        
        // Handle 404 gracefully (file already deleted or expired)
        if status == 404 {
            debug!(file_name = %file_name, "File not found (already deleted or expired)");
            return Ok(());
        }

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(map_http_error(status, &error_text, &format!("delete file {}", file_name)));
        }

        debug!(file_name = %file_name, "File deleted successfully");
        Ok(())
    }
}

/// Maps HTTP status codes to appropriate ModelError variants.
fn map_http_error(status: reqwest::StatusCode, error_text: &str, operation: &str) -> ModelError {
    match status.as_u16() {
        400 => ModelError::RequestError(format!(
            "Invalid request for {}: {}",
            operation, error_text
        )),
        401 | 403 => ModelError::UnsupportedModelProvider(format!(
            "Authentication failed for {}: {}",
            operation, error_text
        )),
        404 => ModelError::RequestError(format!(
            "Not found for {}: {}",
            operation, error_text
        )),
        413 => ModelError::RequestError(format!(
            "File too large for {}: {}",
            operation, error_text
        )),
        429 => ModelError::QuotaExceeded {
            provider: "gemini".to_string(),
            message: Some(format!("Rate limit exceeded for {}: {}", operation, error_text)),
        },
        500..=599 => ModelError::RequestError(format!(
            "Server error for {} ({}): {}",
            operation, status, error_text
        )),
        _ => ModelError::RequestError(format!(
            "Unexpected error for {} ({}): {}",
            operation, status, error_text
        )),
    }
}

/// Detects MIME type from file extension.
fn detect_mime_type_from_extension(path: &Path) -> String {
    path.extension()
        .and_then(|ext| ext.to_str())
        .and_then(|ext| {
            Some(match ext.to_lowercase().as_str() {
                "png" => "image/png",
                "jpg" | "jpeg" => "image/jpeg",
                "gif" => "image/gif",
                "webp" => "image/webp",
                "mp3" => "audio/mpeg",
                "wav" => "audio/wav",
                "ogg" => "audio/ogg",
                "mp4" => "video/mp4",
                "webm" => "video/webm",
                "pdf" => "application/pdf",
                "txt" => "text/plain",
                "md" => "text/markdown",
                _ => "application/octet-stream",
            })
        })
        .unwrap_or("application/octet-stream")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_state_deserialization() {
        let json = r#""PROCESSING""#;
        let state: FileState = serde_json::from_str(json).unwrap();
        assert_eq!(state, FileState::Processing);

        let json = r#""ACTIVE""#;
        let state: FileState = serde_json::from_str(json).unwrap();
        assert_eq!(state, FileState::Active);

        let json = r#""FAILED""#;
        let state: FileState = serde_json::from_str(json).unwrap();
        assert_eq!(state, FileState::Failed);
    }

    #[test]
    fn test_gemini_file_api_creation() {
        let api = GeminiFileApi::with_api_key("test-key".to_string());
        // Just verify it compiles and can be created
        assert!(!api.base_url.is_empty());
    }

    #[test]
    fn test_gemini_file_deserialization() {
        let json = r#"{
            "name": "files/abc123",
            "uri": "https://generativelanguage.googleapis.com/v1beta/files/abc123",
            "state": "ACTIVE",
            "expire_time": "2024-12-10T12:00:00Z",
            "size_bytes": 1048576,
            "display_name": "test.pdf",
            "mime_type": "application/pdf"
        }"#;

        let file: GeminiFile = serde_json::from_str(json).unwrap();
        assert_eq!(file.name, "files/abc123");
        assert_eq!(file.state, FileState::Active);
        assert_eq!(file.size_bytes, 1048576);
        assert_eq!(file.mime_type, "application/pdf");
        assert_eq!(file.display_name, Some("test.pdf".to_string()));
    }

    #[test]
    fn test_detect_mime_type_from_extension() {
        assert_eq!(
            detect_mime_type_from_extension(Path::new("test.png")),
            "image/png"
        );
        assert_eq!(
            detect_mime_type_from_extension(Path::new("test.jpg")),
            "image/jpeg"
        );
        assert_eq!(
            detect_mime_type_from_extension(Path::new("test.jpeg")),
            "image/jpeg"
        );
        assert_eq!(
            detect_mime_type_from_extension(Path::new("test.mp4")),
            "video/mp4"
        );
        assert_eq!(
            detect_mime_type_from_extension(Path::new("test.pdf")),
            "application/pdf"
        );
        assert_eq!(
            detect_mime_type_from_extension(Path::new("test.unknown")),
            "application/octet-stream"
        );
        assert_eq!(
            detect_mime_type_from_extension(Path::new("test")),
            "application/octet-stream"
        );
    }

    #[test]
    fn test_map_http_error() {
        let error_400 = map_http_error(
            reqwest::StatusCode::from_u16(400).unwrap(),
            "Bad request",
            "upload",
        );
        assert!(matches!(error_400, ModelError::RequestError(_)));

        let error_401 = map_http_error(
            reqwest::StatusCode::from_u16(401).unwrap(),
            "Unauthorized",
            "upload",
        );
        assert!(matches!(error_401, ModelError::UnsupportedModelProvider(_)));

        let error_429 = map_http_error(
            reqwest::StatusCode::from_u16(429).unwrap(),
            "Rate limit",
            "upload",
        );
        assert!(matches!(error_429, ModelError::QuotaExceeded { .. }));

        let error_500 = map_http_error(
            reqwest::StatusCode::from_u16(500).unwrap(),
            "Server error",
            "upload",
        );
        assert!(matches!(error_500, ModelError::RequestError(_)));
    }
}

