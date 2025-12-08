//! Security layer for request validation, logging, and rate limiting.
//!
//! This module provides centralized security policies including request/response
//! logging with sensitive data redaction and rate limiting for tool execution.

use crate::mcp::proxy::types::{SecurityConfig, SecurityLayer as SecurityLayerTrait};
use crate::mcp::McpToolResult;
use crate::mcp::{McpError, Result};
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

/// Token bucket for rate limiting.
struct TokenBucket {
    /// Current number of tokens.
    tokens: f64,
    /// Time of last token refill.
    last_refill: Instant,
    /// Maximum token capacity.
    capacity: f64,
}

impl TokenBucket {
    fn new(capacity: f64) -> Self {
        Self {
            tokens: capacity,
            last_refill: Instant::now(),
            capacity,
        }
    }

    /// Refill tokens based on elapsed time.
    fn refill(&mut self, rate_per_minute: f64) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        let elapsed_seconds = elapsed.as_secs_f64();
        let elapsed_minutes = elapsed_seconds / 60.0;

        // Add tokens proportional to time elapsed
        let tokens_to_add = rate_per_minute * elapsed_minutes;
        self.tokens = (self.tokens + tokens_to_add).min(self.capacity);
        self.last_refill = now;
    }

    /// Try to consume a token.
    ///
    /// # Returns
    ///
    /// True if a token was consumed, false otherwise.
    fn try_consume(&mut self) -> bool {
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

/// Rate limiter using token bucket algorithm.
struct RateLimiter {
    /// Token buckets keyed by rate limit key (e.g., "agent_id:tool_name").
    buckets: Arc<Mutex<HashMap<String, TokenBucket>>>,
    /// Rate limit per minute.
    rate_per_minute: f64,
    /// Background cleanup task handle.
    cleanup_task: Option<JoinHandle<()>>,
}

impl RateLimiter {
    /// Create a new rate limiter.
    fn new(rate_per_minute: u32) -> Self {
        let buckets = Arc::new(Mutex::new(HashMap::new()));
        let rate_per_minute_f64 = rate_per_minute as f64;

        // Spawn cleanup task to remove old buckets every 5 minutes
        let cleanup_buckets = Arc::clone(&buckets);
        let cleanup_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes
            loop {
                interval.tick().await;
                let mut buckets = cleanup_buckets.lock().await;
                // Remove buckets that haven't been used in 10 minutes
                // (Simple approach: remove all, they'll be recreated if needed)
                // In production, we'd track last access time
                if buckets.len() > 1000 {
                    // Only cleanup if we have too many buckets
                    buckets.clear();
                }
            }
        });

        Self {
            buckets,
            rate_per_minute: rate_per_minute_f64,
            cleanup_task: Some(cleanup_task),
        }
    }

    /// Check if a request is allowed under rate limits.
    ///
    /// # Arguments
    ///
    /// * `key` - Rate limit key (e.g., "agent_id:tool_name")
    ///
    /// # Returns
    ///
    /// Ok(()) if allowed, error if rate limit exceeded
    async fn check_rate_limit(&self, key: &str) -> Result<()> {
        let mut buckets = self.buckets.lock().await;

        let bucket = buckets
            .entry(key.to_string())
            .or_insert_with(|| TokenBucket::new(self.rate_per_minute));

        bucket.refill(self.rate_per_minute);

        if bucket.try_consume() {
            Ok(())
        } else {
            Err(McpError::protocol(
                format!("Rate limit exceeded for key: {}", key),
                format!(
                    "Too many requests. Rate limit is {} requests per minute. Please wait before retrying.",
                    self.rate_per_minute
                ),
            ))
        }
    }
}

impl Drop for RateLimiter {
    fn drop(&mut self) {
        if let Some(task) = self.cleanup_task.take() {
            task.abort();
        }
    }
}

/// Default implementation of security layer.
pub struct DefaultSecurityLayer {
    /// Security configuration.
    config: SecurityConfig,
    /// Rate limiter.
    rate_limiter: Arc<RateLimiter>,
    /// Compiled redaction patterns.
    redaction_patterns: Vec<Regex>,
}

impl DefaultSecurityLayer {
    /// Create a new security layer.
    ///
    /// # Arguments
    ///
    /// * `config` - Security configuration
    ///
    /// # Errors
    ///
    /// Returns an error if redaction patterns cannot be compiled.
    pub fn new(config: SecurityConfig) -> Result<Self> {
        let rate_limiter = Arc::new(RateLimiter::new(config.rate_limit_per_minute));

        // Compile redaction patterns
        let mut redaction_patterns = Vec::new();
        for pattern_str in &config.redact_patterns {
            let regex = Regex::new(pattern_str).map_err(|e| {
                McpError::config(
                    format!("Invalid redaction pattern '{}': {}", pattern_str, e),
                    format!(
                        "Fix the redaction pattern. It should be a valid regex. Example:\n  redact_patterns = [\"api[_-]?key\", \"password\", \"token\"]"
                    ),
                )
            })?;
            redaction_patterns.push(regex);
        }

        Ok(Self {
            config,
            rate_limiter,
            redaction_patterns,
        })
    }

    /// Redact sensitive data from text.
    ///
    /// # Arguments
    ///
    /// * `text` - Text to redact
    ///
    /// # Returns
    ///
    /// Text with sensitive patterns replaced with "[REDACTED]"
    fn redact_sensitive_data(&self, text: &str) -> String {
        let mut result = text.to_string();
        for pattern in &self.redaction_patterns {
            result = pattern.replace_all(&result, "[REDACTED]").to_string();
        }
        result
    }

    /// Log a request with redaction.
    ///
    /// # Arguments
    ///
    /// * `tool_name` - Name of the tool being called
    /// * `arguments` - Tool execution arguments
    /// * `agent_id` - Identifier for the requesting agent
    async fn log_request(&self, tool_name: &str, arguments: &Value, agent_id: &str) {
        if !self.config.log_requests {
            return;
        }

        // Serialize arguments to JSON for logging
        let args_str = serde_json::to_string(arguments).unwrap_or_else(|_| "{}".to_string());
        let redacted_args = self.redact_sensitive_data(&args_str);

        tracing::info!(
            tool_name = %tool_name,
            agent_id = %agent_id,
            arguments = %redacted_args,
            "Tool execution request"
        );
    }

    /// Log a response with redaction.
    ///
    /// # Arguments
    ///
    /// * `tool_name` - Name of the tool that was called
    /// * `result` - Tool execution result
    /// * `agent_id` - Identifier for the requesting agent
    async fn log_response(&self, tool_name: &str, result: &McpToolResult, agent_id: &str) {
        if !self.config.log_responses {
            return;
        }

        // Serialize result to JSON for logging
        let result_str = serde_json::to_string(result).unwrap_or_else(|_| "{}".to_string());
        let redacted_result = self.redact_sensitive_data(&result_str);

        tracing::info!(
            tool_name = %tool_name,
            agent_id = %agent_id,
            is_error = result.is_error,
            result = %redacted_result,
            "Tool execution response"
        );
    }
}

#[async_trait::async_trait]
impl SecurityLayerTrait for DefaultSecurityLayer {
    async fn check_request(
        &self,
        tool_name: &str,
        arguments: &Value,
        agent_id: &str,
    ) -> Result<()> {
        // Check rate limit
        let rate_limit_key = format!("{}:{}", agent_id, tool_name);
        self.rate_limiter.check_rate_limit(&rate_limit_key).await?;

        // Log request
        self.log_request(tool_name, arguments, agent_id).await;

        Ok(())
    }

    async fn check_response(
        &self,
        tool_name: &str,
        result: &McpToolResult,
        agent_id: &str,
    ) -> Result<()> {
        // Log response
        self.log_response(tool_name, result, agent_id).await;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::proxy::types::SecurityConfig;

    fn create_test_config() -> SecurityConfig {
        SecurityConfig {
            log_requests: true,
            log_responses: true,
            redact_patterns: vec!["api[_-]?key".to_string(), "password".to_string()],
            rate_limit_per_minute: 60,
        }
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let mut config = create_test_config();
        config.rate_limit_per_minute = 2;

        let security = DefaultSecurityLayer::new(config).unwrap();

        // First two requests should succeed
        let result1 = security
            .check_request("test_tool", &json!({}), "agent1")
            .await;
        assert!(result1.is_ok());

        let result2 = security
            .check_request("test_tool", &json!({}), "agent1")
            .await;
        assert!(result2.is_ok());

        // Third request should be rate limited
        let result3 = security
            .check_request("test_tool", &json!({}), "agent1")
            .await;
        assert!(result3.is_err());
        assert!(result3.unwrap_err().to_string().contains("Rate limit"));
    }

    #[tokio::test]
    async fn test_redaction() {
        let security = DefaultSecurityLayer::new(create_test_config()).unwrap();

        let text = r#"{"api_key": "secret123", "password": "mypass", "normal": "data"}"#;
        let redacted = security.redact_sensitive_data(text);

        assert!(redacted.contains("[REDACTED]"));
        assert!(!redacted.contains("secret123"));
        assert!(!redacted.contains("mypass"));
        assert!(redacted.contains("normal"));
        assert!(redacted.contains("data"));
    }

    #[tokio::test]
    async fn test_rate_limit_different_keys() {
        let mut config = create_test_config();
        config.rate_limit_per_minute = 1;

        let security = DefaultSecurityLayer::new(config).unwrap();

        // Different agent/tool combinations should have separate rate limits
        let result1 = security
            .check_request("tool1", &json!({}), "agent1")
            .await;
        assert!(result1.is_ok());

        let result2 = security
            .check_request("tool2", &json!({}), "agent1")
            .await;
        assert!(result2.is_ok()); // Different tool, should be allowed

        let result3 = security
            .check_request("tool1", &json!({}), "agent2")
            .await;
        assert!(result3.is_ok()); // Different agent, should be allowed
    }
}
