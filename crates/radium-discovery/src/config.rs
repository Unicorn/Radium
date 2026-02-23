//! Discovery service configuration from environment variables

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(String),
}

#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    pub neo4j_uri: String,
    pub neo4j_user: String,
    pub neo4j_password: String,
    pub port: u16,
    /// Allowed CORS origins. Set via `CORS_ALLOW_ORIGINS` (comma-separated).
    /// Defaults to localhost:3000 and localhost:8080 for local development.
    /// Set to `*` to allow any origin (not recommended for production).
    pub allowed_origins: Vec<String>,
}

impl DiscoveryConfig {
    #[allow(clippy::disallowed_methods)] // Config loading requires direct env var access
    pub fn from_env() -> Result<Self, ConfigError> {
        let neo4j_uri = std::env::var("NEO4J_URI")
            .map_err(|_| ConfigError::MissingEnvVar("NEO4J_URI".into()))?;
        let neo4j_user = std::env::var("NEO4J_USER")
            .map_err(|_| ConfigError::MissingEnvVar("NEO4J_USER".into()))?;
        let neo4j_password = std::env::var("NEO4J_PASSWORD")
            .map_err(|_| ConfigError::MissingEnvVar("NEO4J_PASSWORD".into()))?;
        let port = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(3030);
        let allowed_origins = std::env::var("CORS_ALLOW_ORIGINS")
            .ok()
            .map(|s| {
                s.split(',')
                    .map(|o| o.trim().to_string())
                    .filter(|o| !o.is_empty())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| {
                vec![
                    "http://localhost:3000".to_string(),
                    "http://localhost:8080".to_string(),
                ]
            });

        Ok(Self {
            neo4j_uri,
            neo4j_user,
            neo4j_password,
            port,
            allowed_origins,
        })
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)] // Tests need to manipulate env vars directly
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_config_missing_neo4j_uri() {
        std::env::remove_var("NEO4J_URI");
        std::env::remove_var("NEO4J_USER");
        std::env::remove_var("NEO4J_PASSWORD");
        let result = DiscoveryConfig::from_env();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("NEO4J_URI"));
    }

    #[test]
    #[serial]
    fn test_default_port() {
        std::env::remove_var("PORT");
        std::env::set_var("NEO4J_URI", "bolt://localhost:7687");
        std::env::set_var("NEO4J_USER", "neo4j");
        std::env::set_var("NEO4J_PASSWORD", "test");
        let config = DiscoveryConfig::from_env().unwrap();
        assert_eq!(config.port, 3030);
        // Cleanup
        std::env::remove_var("NEO4J_URI");
        std::env::remove_var("NEO4J_USER");
        std::env::remove_var("NEO4J_PASSWORD");
    }

    #[test]
    #[serial]
    fn test_default_cors_origins() {
        std::env::remove_var("CORS_ALLOW_ORIGINS");
        std::env::set_var("NEO4J_URI", "bolt://localhost:7687");
        std::env::set_var("NEO4J_USER", "neo4j");
        std::env::set_var("NEO4J_PASSWORD", "test");
        let config = DiscoveryConfig::from_env().unwrap();
        assert!(config.allowed_origins.contains(&"http://localhost:3000".to_string()));
        assert!(config.allowed_origins.contains(&"http://localhost:8080".to_string()));
        std::env::remove_var("NEO4J_URI");
        std::env::remove_var("NEO4J_USER");
        std::env::remove_var("NEO4J_PASSWORD");
    }

    #[test]
    #[serial]
    fn test_custom_cors_origins() {
        std::env::set_var("CORS_ALLOW_ORIGINS", "https://app.example.com, https://dashboard.example.com");
        std::env::set_var("NEO4J_URI", "bolt://localhost:7687");
        std::env::set_var("NEO4J_USER", "neo4j");
        std::env::set_var("NEO4J_PASSWORD", "test");
        let config = DiscoveryConfig::from_env().unwrap();
        assert_eq!(config.allowed_origins.len(), 2);
        assert!(config.allowed_origins.contains(&"https://app.example.com".to_string()));
        assert!(config.allowed_origins.contains(&"https://dashboard.example.com".to_string()));
        std::env::remove_var("CORS_ALLOW_ORIGINS");
        std::env::remove_var("NEO4J_URI");
        std::env::remove_var("NEO4J_USER");
        std::env::remove_var("NEO4J_PASSWORD");
    }
}
