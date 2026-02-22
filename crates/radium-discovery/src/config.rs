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
}

impl DiscoveryConfig {
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

        Ok(Self {
            neo4j_uri,
            neo4j_user,
            neo4j_password,
            port,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_missing_neo4j_uri() {
        std::env::remove_var("NEO4J_URI");
        std::env::remove_var("NEO4J_USER");
        std::env::remove_var("NEO4J_PASSWORD");
        let result = DiscoveryConfig::from_env();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("NEO4J_URI"));
    }

    #[test]
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
}
