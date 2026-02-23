//! SMTP Send component schema
//!
//! The SMTP Send component delivers email messages via an SMTP server.
//! Supports plain text and HTML content types, multiple recipients,
//! CC/BCC, attachments, TLS, and SMTP authentication.

use serde::{Deserialize, Serialize};
use validator::Validate;

use super::behaviors::{ComponentBehaviors, RateLimitConfig};

/// Content type for the email body.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum EmailContentType {
    /// Plain text body.
    #[default]
    Plain,
    /// HTML body.
    Html,
}

/// A single email attachment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EmailAttachment {
    /// File name shown to the recipient.
    pub filename: String,

    /// Base64-encoded file content.
    pub content_base64: String,

    /// MIME type of the attachment (e.g. `"application/pdf"`).
    pub mime_type: String,
}

/// SMTP Send component input.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct SmtpSendInput {
    /// SMTP server hostname or IP address.
    #[validate(length(min = 1, message = "host must not be empty"))]
    pub host: String,

    /// SMTP server port (default 587).
    #[serde(default = "default_port")]
    pub port: u16,

    /// SMTP authentication username.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    /// Secret reference for the SMTP password (e.g. `"${{ secrets.SMTP_PASSWORD }}"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password_ref: Option<String>,

    /// Sender address shown in the `From` header.
    #[validate(length(min = 1, message = "from address must not be empty"))]
    pub from: String,

    /// Primary recipient addresses. At least one is required.
    #[validate(length(min = 1, message = "at least one recipient is required"))]
    pub to: Vec<String>,

    /// Carbon-copy recipient addresses.
    #[serde(default)]
    pub cc: Vec<String>,

    /// Blind carbon-copy recipient addresses.
    #[serde(default)]
    pub bcc: Vec<String>,

    /// Email subject line.
    #[validate(length(min = 1, message = "subject must not be empty"))]
    pub subject: String,

    /// Email body content.
    pub body: String,

    /// Content type of the body (plain text or HTML).
    #[serde(default)]
    pub content_type: EmailContentType,

    /// File attachments to include with the message.
    #[serde(default)]
    pub attachments: Vec<EmailAttachment>,

    /// Whether to use TLS when connecting to the SMTP server.
    #[serde(default = "default_true")]
    pub use_tls: bool,

    /// Shared component behaviors (retry, rate limit, timeout, etc.).
    #[serde(default = "smtp_send_default_behaviors")]
    #[validate(nested)]
    pub behaviors: ComponentBehaviors,
}

fn default_port() -> u16 {
    587
}

fn default_true() -> bool {
    true
}

fn smtp_send_default_behaviors() -> ComponentBehaviors {
    ComponentBehaviors {
        timeout_ms: 60_000,
        rate_limit: RateLimitConfig {
            requests_per_second: 5,
            burst: 10,
            ..Default::default()
        },
        ..Default::default()
    }
}

impl Default for SmtpSendInput {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: default_port(),
            username: None,
            password_ref: None,
            from: String::new(),
            to: Vec::new(),
            cc: Vec::new(),
            bcc: Vec::new(),
            subject: String::new(),
            body: String::new(),
            content_type: EmailContentType::default(),
            attachments: Vec::new(),
            use_tls: true,
            behaviors: smtp_send_default_behaviors(),
        }
    }
}

/// SMTP Send component output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SmtpSendOutput {
    /// Server-assigned message identifier.
    pub message_id: String,

    /// Recipient addresses that were accepted by the server.
    pub accepted: Vec<String>,

    /// Recipient addresses that were rejected by the server.
    pub rejected: Vec<String>,
}

impl Default for SmtpSendOutput {
    fn default() -> Self {
        Self {
            message_id: String::new(),
            accepted: Vec::new(),
            rejected: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_with_defaults() {
        let input = SmtpSendInput::default();
        assert!(input.host.is_empty());
        assert_eq!(input.port, 587);
        assert!(input.username.is_none());
        assert!(input.password_ref.is_none());
        assert!(input.from.is_empty());
        assert!(input.to.is_empty());
        assert!(input.cc.is_empty());
        assert!(input.bcc.is_empty());
        assert!(input.subject.is_empty());
        assert!(input.body.is_empty());
        assert_eq!(input.content_type, EmailContentType::Plain);
        assert!(input.attachments.is_empty());
        assert!(input.use_tls);
        assert_eq!(input.behaviors.timeout_ms, 60_000);
        assert_eq!(input.behaviors.rate_limit.requests_per_second, 5);
        assert_eq!(input.behaviors.rate_limit.burst, 10);
    }

    #[test]
    fn test_full_config_deserialization() {
        let yaml = r#"
host: "smtp.example.com"
port: 465
username: "user@example.com"
password_ref: "${{ secrets.SMTP_PASSWORD }}"
from: "sender@example.com"
to:
  - "alice@example.com"
  - "bob@example.com"
cc:
  - "carol@example.com"
bcc:
  - "audit@example.com"
subject: "Hello from Radium"
body: "<h1>Hello</h1>"
content_type: html
attachments:
  - filename: "report.pdf"
    content_base64: "SGVsbG8="
    mime_type: "application/pdf"
use_tls: true
behaviors:
  timeout_ms: 30000
"#;
        let input: SmtpSendInput = serde_yaml::from_str(yaml).expect("deserialize");
        assert_eq!(input.host, "smtp.example.com");
        assert_eq!(input.port, 465);
        assert_eq!(input.username, Some("user@example.com".to_string()));
        assert_eq!(
            input.password_ref,
            Some("${{ secrets.SMTP_PASSWORD }}".to_string())
        );
        assert_eq!(input.from, "sender@example.com");
        assert_eq!(input.to.len(), 2);
        assert_eq!(input.cc.len(), 1);
        assert_eq!(input.bcc.len(), 1);
        assert_eq!(input.subject, "Hello from Radium");
        assert_eq!(input.content_type, EmailContentType::Html);
        assert_eq!(input.attachments.len(), 1);
        assert_eq!(input.attachments[0].filename, "report.pdf");
        assert_eq!(input.attachments[0].mime_type, "application/pdf");
        assert!(input.use_tls);
        assert_eq!(input.behaviors.timeout_ms, 30_000);
    }

    #[test]
    fn test_output_serialize_deserialize() {
        let output = SmtpSendOutput {
            message_id: "<abc123@smtp.example.com>".to_string(),
            accepted: vec!["alice@example.com".to_string()],
            rejected: vec!["bad@nowhere.invalid".to_string()],
        };
        let yaml = serde_yaml::to_string(&output).expect("serialize");
        let restored: SmtpSendOutput = serde_yaml::from_str(&yaml).expect("deserialize");
        assert_eq!(restored.message_id, output.message_id);
        assert_eq!(restored.accepted, output.accepted);
        assert_eq!(restored.rejected, output.rejected);
    }

    #[test]
    fn test_content_type_default() {
        let content_type = EmailContentType::default();
        assert_eq!(content_type, EmailContentType::Plain);

        let serialized = serde_json::to_string(&content_type).expect("serialize");
        assert_eq!(serialized, "\"plain\"");

        let html = EmailContentType::Html;
        let serialized_html = serde_json::to_string(&html).expect("serialize html");
        assert_eq!(serialized_html, "\"html\"");
    }
}
