//! Supabase REST API client.
//!
//! Wraps `reqwest` to communicate with Supabase's PostgREST API using
//! the service-role key for server-side access.

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{de::DeserializeOwned, Serialize};

use super::error::SupabaseError;

/// Configuration for connecting to a Supabase project.
#[derive(Debug, Clone)]
pub struct SupabaseConfig {
    /// The Supabase project URL (e.g. `https://<ref>.supabase.co`).
    pub url: String,
    /// The service-role key for privileged server-side access.
    pub service_role_key: String,
}

impl SupabaseConfig {
    /// Build configuration from environment variables.
    ///
    /// Required variables:
    /// - `SUPABASE_URL`
    /// - `SUPABASE_SERVICE_ROLE_KEY`
    pub fn from_env() -> Result<Self, SupabaseError> {
        let url = std::env::var("SUPABASE_URL").ok();
        let service_role_key = std::env::var("SUPABASE_SERVICE_ROLE_KEY").ok();
        Self::from_values(url, service_role_key)
    }

    /// Build configuration from optional values, returning descriptive errors
    /// for any that are missing. Split out from `from_env` for testability
    /// (env manipulation is unsafe in modern Rust).
    fn from_values(
        url: Option<String>,
        service_role_key: Option<String>,
    ) -> Result<Self, SupabaseError> {
        let url = url.ok_or_else(|| {
            SupabaseError::ConfigError(
                "SUPABASE_URL environment variable is not set".to_string(),
            )
        })?;

        let service_role_key = service_role_key.ok_or_else(|| {
            SupabaseError::ConfigError(
                "SUPABASE_SERVICE_ROLE_KEY environment variable is not set".to_string(),
            )
        })?;

        Ok(Self {
            url,
            service_role_key,
        })
    }
}

/// HTTP client for the Supabase PostgREST API.
#[derive(Debug, Clone)]
pub struct SupabaseClient {
    config: SupabaseConfig,
    http: reqwest::Client,
}

impl SupabaseClient {
    /// Create a new client from the given configuration.
    pub fn new(config: SupabaseConfig) -> Self {
        Self {
            config,
            http: reqwest::Client::new(),
        }
    }

    /// Return the full PostgREST URL for a given table.
    pub fn table_url(&self, table: &str) -> String {
        let base = self.config.url.trim_end_matches('/');
        format!("{base}/rest/v1/{table}")
    }

    /// Build the standard headers required by the Supabase REST API.
    pub fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();

        // apikey header — required by the Supabase gateway
        headers.insert(
            "apikey",
            HeaderValue::from_str(&self.config.service_role_key)
                .expect("service_role_key must be a valid header value"),
        );

        // Authorization header — Bearer token for PostgREST
        let bearer = format!("Bearer {}", self.config.service_role_key);
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&bearer)
                .expect("bearer token must be a valid header value"),
        );

        // Content-Type
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        headers
    }

    // ── Query helpers ────────────────────────────────────────────────

    /// Select rows from a table. `query` contains PostgREST query parameters
    /// (e.g. `[("id", "eq.123"), ("select", "id,name")]`).
    pub async fn select<T: DeserializeOwned>(
        &self,
        table: &str,
        query: &[(&str, &str)],
    ) -> Result<Vec<T>, SupabaseError> {
        let url = self.table_url(table);
        let response = self
            .http
            .get(&url)
            .headers(self.headers())
            .query(query)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let message = response.text().await.unwrap_or_default();
            return Err(SupabaseError::ApiError {
                status: status.as_u16(),
                message,
            });
        }

        let body = response.text().await?;
        serde_json::from_str::<Vec<T>>(&body).map_err(|e| {
            SupabaseError::DeserializationError(format!(
                "Failed to deserialize response from {table}: {e}"
            ))
        })
    }

    /// Select a single row from a table. Returns `SupabaseError::NotFound` when
    /// PostgREST responds with 406 (no rows matched the singleton request).
    pub async fn select_one<T: DeserializeOwned>(
        &self,
        table: &str,
        query: &[(&str, &str)],
    ) -> Result<T, SupabaseError> {
        let url = self.table_url(table);
        let mut headers = self.headers();
        // Tell PostgREST we expect exactly one object back.
        headers.insert(
            "Accept",
            HeaderValue::from_static("application/vnd.pgrst.object+json"),
        );

        let response = self
            .http
            .get(&url)
            .headers(headers)
            .query(query)
            .send()
            .await?;

        let status = response.status();
        if status.as_u16() == 406 {
            // PostgREST returns 406 when the singleton request matches zero rows.
            return Err(SupabaseError::NotFound {
                resource: table.to_string(),
                key: "query".to_string(),
                value: format!("{query:?}"),
            });
        }

        if !status.is_success() {
            let message = response.text().await.unwrap_or_default();
            return Err(SupabaseError::ApiError {
                status: status.as_u16(),
                message,
            });
        }

        let body = response.text().await?;
        serde_json::from_str::<T>(&body).map_err(|e| {
            SupabaseError::DeserializationError(format!(
                "Failed to deserialize single row from {table}: {e}"
            ))
        })
    }

    /// Insert a row and return the created record.
    ///
    /// Sends `Prefer: return=representation` so PostgREST echoes the row back.
    pub async fn insert<T: DeserializeOwned, B: Serialize>(
        &self,
        table: &str,
        body: &B,
    ) -> Result<T, SupabaseError> {
        let url = self.table_url(table);
        let mut headers = self.headers();
        headers.insert("Prefer", HeaderValue::from_static("return=representation"));
        // Expect a single object back.
        headers.insert(
            "Accept",
            HeaderValue::from_static("application/vnd.pgrst.object+json"),
        );

        let response = self
            .http
            .post(&url)
            .headers(headers)
            .json(body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let message = response.text().await.unwrap_or_default();
            return Err(SupabaseError::ApiError {
                status: status.as_u16(),
                message,
            });
        }

        let text = response.text().await?;
        serde_json::from_str::<T>(&text).map_err(|e| {
            SupabaseError::DeserializationError(format!(
                "Failed to deserialize insert response from {table}: {e}"
            ))
        })
    }

    /// Update rows matching `query` and return the updated records.
    pub async fn update<T: DeserializeOwned, B: Serialize>(
        &self,
        table: &str,
        query: &[(&str, &str)],
        body: &B,
    ) -> Result<Vec<T>, SupabaseError> {
        let url = self.table_url(table);
        let mut headers = self.headers();
        headers.insert("Prefer", HeaderValue::from_static("return=representation"));

        let response = self
            .http
            .patch(&url)
            .headers(headers)
            .query(query)
            .json(body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let message = response.text().await.unwrap_or_default();
            return Err(SupabaseError::ApiError {
                status: status.as_u16(),
                message,
            });
        }

        let text = response.text().await?;
        serde_json::from_str::<Vec<T>>(&text).map_err(|e| {
            SupabaseError::DeserializationError(format!(
                "Failed to deserialize update response from {table}: {e}"
            ))
        })
    }

    /// Delete rows matching `query`.
    pub async fn delete(
        &self,
        table: &str,
        query: &[(&str, &str)],
    ) -> Result<(), SupabaseError> {
        let url = self.table_url(table);

        let response = self
            .http
            .delete(&url)
            .headers(self.headers())
            .query(query)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let message = response.text().await.unwrap_or_default();
            return Err(SupabaseError::ApiError {
                status: status.as_u16(),
                message,
            });
        }

        Ok(())
    }
}

// ── Unit tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build a client with predictable config values.
    fn test_client() -> SupabaseClient {
        SupabaseClient::new(SupabaseConfig {
            url: "https://abc123.supabase.co".to_string(),
            service_role_key: "test-service-role-key".to_string(),
        })
    }

    #[test]
    fn test_table_url() {
        let client = test_client();
        assert_eq!(
            client.table_url("workflows"),
            "https://abc123.supabase.co/rest/v1/workflows"
        );
    }

    #[test]
    fn test_table_url_strips_trailing_slash() {
        let client = SupabaseClient::new(SupabaseConfig {
            url: "https://abc123.supabase.co/".to_string(),
            service_role_key: "key".to_string(),
        });
        assert_eq!(
            client.table_url("items"),
            "https://abc123.supabase.co/rest/v1/items"
        );
    }

    #[test]
    fn test_headers_include_auth() {
        let client = test_client();
        let headers = client.headers();

        // apikey header
        assert_eq!(
            headers.get("apikey").unwrap().to_str().unwrap(),
            "test-service-role-key"
        );

        // Authorization: Bearer header
        assert_eq!(
            headers.get(AUTHORIZATION).unwrap().to_str().unwrap(),
            "Bearer test-service-role-key"
        );

        // Content-Type
        assert_eq!(
            headers.get(CONTENT_TYPE).unwrap().to_str().unwrap(),
            "application/json"
        );
    }

    #[test]
    fn test_config_from_env_missing_url() {
        // Both values missing — should fail on SUPABASE_URL first.
        let result = SupabaseConfig::from_values(None, None);
        assert!(result.is_err());
        match result.unwrap_err() {
            SupabaseError::ConfigError(msg) => {
                assert!(
                    msg.contains("SUPABASE_URL"),
                    "Error message should mention SUPABASE_URL, got: {msg}"
                );
            }
            other => panic!("Expected ConfigError, got: {other:?}"),
        }
    }

    #[test]
    fn test_config_from_env_missing_key() {
        // URL present but key missing.
        let result = SupabaseConfig::from_values(
            Some("https://test.supabase.co".to_string()),
            None,
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            SupabaseError::ConfigError(msg) => {
                assert!(
                    msg.contains("SUPABASE_SERVICE_ROLE_KEY"),
                    "Error message should mention SUPABASE_SERVICE_ROLE_KEY, got: {msg}"
                );
            }
            other => panic!("Expected ConfigError, got: {other:?}"),
        }
    }

    #[test]
    fn test_config_from_values_success() {
        let result = SupabaseConfig::from_values(
            Some("https://test.supabase.co".to_string()),
            Some("secret-key".to_string()),
        );
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.url, "https://test.supabase.co");
        assert_eq!(config.service_role_key, "secret-key");
    }
}
