//! Jira source reader.

use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

use super::traits::SourceReader;
use super::types::{SourceError, SourceMetadata};

/// Reader for Jira ticket sources (jira:// scheme).
pub struct JiraReader {
    /// HTTP client for making requests.
    client: Client,

    /// Base URL for Jira instance (e.g., "https://your-company.atlassian.net").
    base_url: Option<String>,

    /// API token for authentication.
    token: Option<String>,

    /// Email/username for authentication.
    email: Option<String>,
}

impl JiraReader {
    /// Creates a new Jira reader with credentials from environment variables.
    ///
    /// Reads from:
    /// - `JIRA_BASE_URL` - Base URL for Jira instance
    /// - `JIRA_EMAIL` - Email/username for authentication
    /// - `JIRA_TOKEN` - API token for authentication
    pub fn new() -> Self {
        #[allow(clippy::disallowed_methods)]
        let base_url = std::env::var("JIRA_BASE_URL").ok();
        #[allow(clippy::disallowed_methods)]
        let email = std::env::var("JIRA_EMAIL").ok();
        #[allow(clippy::disallowed_methods)]
        let token = std::env::var("JIRA_TOKEN").ok();

        Self {
            client: Client::new(),
            base_url,
            email,
            token,
        }
    }

    /// Creates a new Jira reader with explicit credentials.
    pub fn with_credentials(
        base_url: String,
        email: String,
        token: String,
    ) -> Self {
        Self {
            client: Client::new(),
            base_url: Some(base_url),
            email: Some(email),
            token: Some(token),
        }
    }

    /// Gets email and token for authentication.
    fn get_credentials(&self) -> Result<(&str, &str), SourceError> {
        let email = self.email.as_deref().ok_or_else(|| {
            SourceError::unauthorized("JIRA_EMAIL environment variable not set")
        })?;
        let token = self.token.as_deref().ok_or_else(|| {
            SourceError::unauthorized("JIRA_TOKEN environment variable not set")
        })?;

        Ok((email, token))
    }

    /// Extracts ticket ID from URI.
    fn extract_ticket_id(&self, uri: &str) -> Result<String, SourceError> {
        // Remove jira:// scheme
        let ticket_id = uri.strip_prefix("jira://").unwrap_or(uri).trim();

        if ticket_id.is_empty() {
            return Err(SourceError::invalid_uri(&format!(
                "Invalid Jira URI format: {} (expected jira://TICKET-ID)",
                uri
            )));
        }

        Ok(ticket_id.to_string())
    }

    /// Gets the base URL for API requests.
    fn get_base_url(&self) -> Result<&str, SourceError> {
        self.base_url.as_deref().ok_or_else(|| {
            SourceError::unauthorized("JIRA_BASE_URL environment variable not set")
        })
    }
}

impl Default for JiraReader {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SourceReader for JiraReader {
    fn scheme(&self) -> &str {
        "jira"
    }

    async fn verify(&self, uri: &str) -> Result<SourceMetadata, SourceError> {
        let ticket_id = self.extract_ticket_id(uri)?;
        let base_url = self.get_base_url()?;
        let (email, token) = self.get_credentials()?;

        // Build API endpoint URL
        let api_url = format!("{}/rest/api/2/issue/{}", base_url, ticket_id);

        // Make request to check if ticket exists
        let response = self
            .client
            .get(&api_url)
            .basic_auth(email, Some(token))
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| {
                SourceError::network_error(&format!("Failed to connect to Jira API: {}", e))
            })?;

        let status = response.status();

        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(SourceError::unauthorized(
                "Jira authentication failed. Check JIRA_EMAIL and JIRA_TOKEN.",
            ));
        }

        if !status.is_success() {
            if status == reqwest::StatusCode::NOT_FOUND {
                return Err(SourceError::not_found(&format!(
                    "Jira ticket not found: {}",
                    ticket_id
                )));
            }
            return Err(SourceError::network_error(&format!(
                "Jira API returned error: HTTP {}",
                status.as_u16()
            )));
        }

        // Parse response to get metadata
        let json: Value = response.json().await.map_err(|e| {
            SourceError::other(format!("Failed to parse Jira API response: {}", e))
        })?;

        // Extract size estimate (JSON response size)
        let size_bytes = serde_json::to_string(&json)
            .ok()
            .map(|s| s.len() as u64);

        // Extract updated time if available
        let last_modified = json
            .get("fields")
            .and_then(|f| f.get("updated"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(SourceMetadata::with_details(
            true,
            size_bytes,
            last_modified,
            Some("application/json".to_string()),
        ))
    }

    async fn fetch(&self, uri: &str) -> Result<String, SourceError> {
        let ticket_id = self.extract_ticket_id(uri)?;
        let base_url = self.get_base_url()?;
        let (email, token) = self.get_credentials()?;

        // Build API endpoint URL
        let api_url = format!("{}/rest/api/2/issue/{}", base_url, ticket_id);

        // Make request to fetch ticket
        let response = self
            .client
            .get(&api_url)
            .basic_auth(email, Some(token))
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| {
                SourceError::network_error(&format!("Failed to connect to Jira API: {}", e))
            })?;

        let status = response.status();

        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(SourceError::unauthorized(
                "Jira authentication failed. Check JIRA_EMAIL and JIRA_TOKEN.",
            ));
        }

        if !status.is_success() {
            if status == reqwest::StatusCode::NOT_FOUND {
                return Err(SourceError::not_found(&format!(
                    "Jira ticket not found: {}",
                    ticket_id
                )));
            }
            return Err(SourceError::network_error(&format!(
                "Jira API returned error: HTTP {}",
                status.as_u16()
            )));
        }

        // Parse response
        let json: Value = response.json().await.map_err(|e| {
            SourceError::other(format!("Failed to parse Jira API response: {}", e))
        })?;

        // Format ticket as readable text
        let key = json.get("key").and_then(|v| v.as_str()).unwrap_or("Unknown");
        let summary = json
            .get("fields")
            .and_then(|f| f.get("summary"))
            .and_then(|v| v.as_str())
            .unwrap_or("No summary");
        let description = json
            .get("fields")
            .and_then(|f| f.get("description"))
            .and_then(|v| v.as_str())
            .unwrap_or("No description");

        let mut content = format!("Jira Ticket: {}\n\n", key);
        content.push_str(&format!("Summary: {}\n\n", summary));
        content.push_str(&format!("Description:\n{}\n\n", description));

        // Add comments if available
        if let Some(comments) = json
            .get("fields")
            .and_then(|f| f.get("comment"))
            .and_then(|c| c.get("comments"))
            .and_then(|cs| cs.as_array())
        {
            if !comments.is_empty() {
                content.push_str("Comments:\n");
                for comment in comments {
                    if let Some(body) = comment.get("body").and_then(|v| v.as_str()) {
                        if let Some(author) = comment
                            .get("author")
                            .and_then(|a| a.get("displayName"))
                            .and_then(|v| v.as_str())
                        {
                            content.push_str(&format!("\n{}: {}\n", author, body));
                        }
                    }
                }
            }
        }

        Ok(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_ticket_id() {
        let reader = JiraReader::new();
        assert_eq!(reader.extract_ticket_id("jira://PROJ-123").unwrap(), "PROJ-123");
        assert_eq!(
            reader.extract_ticket_id("jira://PROJ-456").unwrap(),
            "PROJ-456"
        );
    }

    #[test]
    fn test_scheme() {
        let reader = JiraReader::new();
        assert_eq!(reader.scheme(), "jira");
    }
}
