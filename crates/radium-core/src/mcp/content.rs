//! Rich content support for MCP (text, images, audio).

use crate::mcp::{McpContent, McpError, Result};
use serde_json::{Value, json};

/// Content type detection and handling utilities.
pub struct ContentHandler;

impl ContentHandler {
    /// Detect content type from data.
    pub fn detect_content_type(data: &[u8], mime_type: Option<&str>) -> String {
        if let Some(mime) = mime_type {
            return mime.to_string();
        }

        // Simple content type detection based on magic bytes
        if data.starts_with(b"\x89PNG\r\n\x1a\n") {
            "image/png".to_string()
        } else if data.starts_with(b"\xff\xd8\xff") {
            "image/jpeg".to_string()
        } else if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
            "image/gif".to_string()
        } else if data.starts_with(b"RIFF") && data.len() > 8 && &data[8..12] == b"WEBP" {
            "image/webp".to_string()
        } else if data.starts_with(b"fLaC") {
            "audio/flac".to_string()
        } else if data.starts_with(b"OggS") {
            "audio/ogg".to_string()
        } else if data.starts_with(b"ID3") || data.starts_with(b"\xff\xfb") {
            "audio/mpeg".to_string()
        } else {
            "application/octet-stream".to_string()
        }
    }

    /// Serialize content for API compatibility.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn serialize_content(content: &McpContent) -> Result<Value> {
        match content {
            McpContent::Text { text } => Ok(json!({
                "type": "text",
                "text": text
            })),
            McpContent::Image { data, mime_type } => {
                // Check if data is a URL or base64
                let data_value = if data.starts_with("http://") || data.starts_with("https://") {
                    json!({
                        "url": data
                    })
                } else {
                    json!({
                        "data": data
                    })
                };

                Ok(json!({
                    "type": "image",
                    "mime_type": mime_type,
                    "data": data_value
                }))
            }
            McpContent::Audio { data, mime_type } => {
                let data_value = if data.starts_with("http://") || data.starts_with("https://") {
                    json!({
                        "url": data
                    })
                } else {
                    json!({
                        "data": data
                    })
                };

                Ok(json!({
                    "type": "audio",
                    "mime_type": mime_type,
                    "data": data_value
                }))
            }
        }
    }

    /// Parse content from API format.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails.
    pub fn parse_content(value: &Value) -> Result<McpContent> {
        let content_type = value
            .get("type")
            .and_then(|t| t.as_str())
            .ok_or_else(|| McpError::protocol(
                "Content missing 'type' field",
                "The content object is missing the required 'type' field. Valid types are: 'text', 'image', or 'audio'. Check the server response format.",
            ))?;

        match content_type {
            "text" => {
                let text = value.get("text").and_then(|t| t.as_str()).ok_or_else(|| {
                    McpError::protocol(
                        "Text content missing 'text' field",
                        "Text content must include a 'text' field with the text content. Check the server response format.",
                    )
                })?;
                Ok(McpContent::Text { text: text.to_string() })
            }
            "image" => {
                let mime_type =
                    value.get("mime_type").and_then(|m| m.as_str()).unwrap_or("image/png");
                let data = value
                    .get("data")
                    .and_then(|d| {
                        if let Some(url) = d.get("url").and_then(|u| u.as_str()) {
                            Some(url.to_string())
                        } else {
                            d.get("data").and_then(|d| d.as_str()).map(|s| s.to_string())
                        }
                    })
                    .ok_or_else(|| {
                        McpError::protocol(
                            "Image content missing 'data' field",
                            "Image content must include a 'data' field with either a URL or base64-encoded image data. Check the server response format.",
                        )
                    })?;
                Ok(McpContent::Image { data, mime_type: mime_type.to_string() })
            }
            "audio" => {
                let mime_type =
                    value.get("mime_type").and_then(|m| m.as_str()).unwrap_or("audio/mpeg");
                let data = value
                    .get("data")
                    .and_then(|d| {
                        if let Some(url) = d.get("url").and_then(|u| u.as_str()) {
                            Some(url.to_string())
                        } else {
                            d.get("data").and_then(|d| d.as_str()).map(|s| s.to_string())
                        }
                    })
                    .ok_or_else(|| {
                        McpError::protocol(
                            "Audio content missing 'data' field",
                            "Audio content must include a 'data' field with either a URL or base64-encoded audio data. Check the server response format.",
                        )
                    })?;
                Ok(McpContent::Audio { data, mime_type: mime_type.to_string() })
            }
            _ => Err(McpError::protocol(
                format!("Unknown content type: '{}'", content_type),
                format!(
                    "The content type '{}' is not supported. Valid types are: 'text', 'image', or 'audio'. Check the server response format.",
                    content_type
                ),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_content_type_png() {
        let png_data = b"\x89PNG\r\n\x1a\n";
        let mime_type = ContentHandler::detect_content_type(png_data, None);
        assert_eq!(mime_type, "image/png");
    }

    #[test]
    fn test_detect_content_type_jpeg() {
        let jpeg_data = b"\xff\xd8\xff\xe0";
        let mime_type = ContentHandler::detect_content_type(jpeg_data, None);
        assert_eq!(mime_type, "image/jpeg");
    }

    #[test]
    fn test_serialize_text_content() {
        let content = McpContent::Text { text: "Hello, world!".to_string() };
        let json = ContentHandler::serialize_content(&content).unwrap();
        assert_eq!(json["type"], "text");
        assert_eq!(json["text"], "Hello, world!");
    }

    #[test]
    fn test_serialize_image_content() {
        let content = McpContent::Image {
            data: "base64data".to_string(),
            mime_type: "image/png".to_string(),
        };
        let json = ContentHandler::serialize_content(&content).unwrap();
        assert_eq!(json["type"], "image");
        assert_eq!(json["mime_type"], "image/png");
    }

    #[test]
    fn test_parse_text_content() {
        let json = json!({
            "type": "text",
            "text": "Hello, world!"
        });
        let content = ContentHandler::parse_content(&json).unwrap();
        match content {
            McpContent::Text { text } => assert_eq!(text, "Hello, world!"),
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_parse_image_content() {
        let json = json!({
            "type": "image",
            "mime_type": "image/png",
            "data": {
                "data": "base64data"
            }
        });
        let content = ContentHandler::parse_content(&json).unwrap();
        match content {
            McpContent::Image { data, mime_type } => {
                assert_eq!(data, "base64data");
                assert_eq!(mime_type, "image/png");
            }
            _ => panic!("Expected image content"),
        }
    }
}
