//! Discovery service client for indexing and telemetry.
//!
//! Provides a thin HTTP client wrapper that communicates with the discovery
//! service in a fire-and-forget fashion. All methods log warnings on failure
//! but never propagate errors to the caller.

use reqwest::Client;

/// Client for the discovery service. All methods are fire-and-forget.
#[derive(Clone)]
pub struct DiscoveryClient {
    client: Client,
    base_url: String,
}

impl DiscoveryClient {
    /// Create from env var. Returns `None` if `DISCOVERY_SERVICE_URL` is not set.
    #[allow(clippy::disallowed_methods)] // Loading service URL from env at startup
    pub fn from_env() -> Option<Self> {
        let base_url = std::env::var("DISCOVERY_SERVICE_URL").ok()?;
        Some(Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        })
    }

    /// Index a component/service/project. Fails silently (logs warning).
    pub async fn index(&self, request: &serde_json::Value) {
        let url = format!("{}/v1/discover/index", self.base_url);
        match self.client.post(&url).json(request).send().await {
            Ok(resp) if resp.status().is_success() => {
                tracing::debug!("Discovery index succeeded");
            }
            Ok(resp) => {
                tracing::warn!(
                    "Discovery index returned {}: {}",
                    resp.status(),
                    resp.text().await.unwrap_or_default()
                );
            }
            Err(e) => {
                tracing::warn!("Discovery index failed (non-blocking): {e}");
            }
        }
    }

    /// Record a telemetry event. Fails silently.
    pub async fn telemetry(
        &self,
        id: &str,
        event: &str,
        user_id: &str,
        component_ids: &[String],
    ) {
        let url = format!("{}/v1/discover/index/{id}/telemetry", self.base_url);
        let body = serde_json::json!({
            "event": event,
            "user_id": user_id,
            "component_ids": component_ids,
        });
        if let Err(e) = self.client.post(&url).json(&body).send().await {
            tracing::warn!("Discovery telemetry failed (non-blocking): {e}");
        }
    }
}

impl std::fmt::Debug for DiscoveryClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiscoveryClient")
            .field("base_url", &self.base_url)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_from_env_none() {
        std::env::remove_var("DISCOVERY_SERVICE_URL");
        assert!(DiscoveryClient::from_env().is_none());
    }

    #[test]
    #[serial]
    fn test_from_env_some() {
        std::env::set_var("DISCOVERY_SERVICE_URL", "http://localhost:3030");
        let client = DiscoveryClient::from_env();
        assert!(client.is_some());
        std::env::remove_var("DISCOVERY_SERVICE_URL");
    }

    #[test]
    #[serial]
    fn test_from_env_trims_trailing_slash() {
        std::env::set_var("DISCOVERY_SERVICE_URL", "http://localhost:3030/");
        let client = DiscoveryClient::from_env().unwrap();
        assert_eq!(client.base_url, "http://localhost:3030");
        std::env::remove_var("DISCOVERY_SERVICE_URL");
    }
}
