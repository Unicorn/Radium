//! Health checking and automatic reconnection for upstream servers.
//!
//! This module provides periodic health checking and automatic reconnection
//! logic with exponential backoff for upstream MCP servers.

use crate::mcp::proxy::upstream_pool::UpstreamPool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, Mutex};
use tokio::task::JoinHandle;

/// Health checker for monitoring and reconnecting upstream servers.
pub struct HealthChecker {
    /// Pool of upstream connections to monitor.
    pool: Arc<UpstreamPool>,
    /// Active health check tasks.
    check_tasks: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
    /// Shutdown signal sender.
    shutdown_tx: broadcast::Sender<()>,
}

impl HealthChecker {
    /// Create a new health checker.
    pub fn new(pool: Arc<UpstreamPool>) -> Self {
        let (shutdown_tx, _) = broadcast::channel(16);
        Self {
            pool,
            check_tasks: Arc::new(Mutex::new(HashMap::new())),
            shutdown_tx,
        }
    }

    /// Start health checking for an upstream server.
    ///
    /// # Arguments
    ///
    /// * `upstream_name` - Name of the upstream to monitor
    /// * `interval_secs` - Health check interval in seconds
    pub async fn start_health_check(&self, upstream_name: String, interval_secs: u64) {
        let pool = Arc::clone(&self.pool);
        let mut shutdown_rx = self.shutdown_tx.subscribe();
        let check_tasks = Arc::clone(&self.check_tasks);

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
            let mut backoff_seconds = 1u64;
            let max_backoff = 60u64;

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Perform health check
                        let state = pool.get_state(&upstream_name).await;

                        match state {
                            Some(crate::mcp::proxy::types::ConnectionState::Connected) => {
                                // Check if client is still responsive
                                if let Some(client) = pool.get_upstream(&upstream_name).await {
                                    let client_guard = client.lock().await;
                                    if client_guard.is_connected() {
                                        // Connection is healthy
                                        pool.mark_healthy(&upstream_name).await;
                                        backoff_seconds = 1; // Reset backoff
                                    } else {
                                        // Connection lost
                                        pool.mark_unhealthy(&upstream_name).await;
                                    }
                                } else {
                                    // Client not available
                                    pool.mark_unhealthy(&upstream_name).await;
                                }
                            }
                            Some(crate::mcp::proxy::types::ConnectionState::Disconnected) |
                            Some(crate::mcp::proxy::types::ConnectionState::Unhealthy) => {
                                // Attempt reconnection with exponential backoff
                                tokio::time::sleep(Duration::from_secs(backoff_seconds)).await;

                                match pool.reconnect_upstream(&upstream_name).await {
                                    Ok(_) => {
                                        tracing::info!(
                                            upstream_name = %upstream_name,
                                            "Successfully reconnected to upstream"
                                        );
                                        pool.mark_healthy(&upstream_name).await;
                                        backoff_seconds = 1; // Reset backoff
                                    }
                                    Err(e) => {
                                        tracing::warn!(
                                            upstream_name = %upstream_name,
                                            backoff_seconds = backoff_seconds,
                                            error = %e,
                                            "Failed to reconnect to upstream, will retry"
                                        );
                                        pool.mark_unhealthy(&upstream_name).await;
                                        // Exponential backoff: double the wait time, max 60s
                                        backoff_seconds = (backoff_seconds * 2).min(max_backoff);
                                    }
                                }
                            }
                            None => {
                                // Upstream removed, stop health checking
                                break;
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        // Shutdown signal received
                        break;
                    }
                }
            }
        });

        let mut tasks = check_tasks.lock().await;
        tasks.insert(upstream_name, handle);
    }

    /// Stop health checking for a specific upstream.
    ///
    /// # Arguments
    ///
    /// * `upstream_name` - Name of the upstream to stop monitoring
    pub async fn stop_health_check(&self, upstream_name: &str) {
        let mut tasks = self.check_tasks.lock().await;
        if let Some(handle) = tasks.remove(upstream_name) {
            handle.abort();
        }
    }

    /// Stop all health check tasks.
    pub async fn stop_all(&self) {
        // Send shutdown signal
        let _ = self.shutdown_tx.send(());

        // Wait for all tasks to finish (with timeout)
        let mut tasks = self.check_tasks.lock().await;
        for (name, handle) in tasks.drain() {
            handle.abort();
            tracing::debug!(upstream_name = %name, "Stopped health check task");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::proxy::types::{ConnectionState, UpstreamConfig};
    use crate::mcp::{McpServerConfig, TransportType};

    fn create_test_config(name: &str) -> UpstreamConfig {
        UpstreamConfig {
            server: McpServerConfig {
                name: name.to_string(),
                transport: TransportType::Stdio,
                command: Some("echo".to_string()),
                args: Some(vec!["test".to_string()]),
                url: None,
                auth: None,
            },
            priority: 1,
            health_check_interval: 1, // 1 second for tests
            tools: None,
        }
    }

    #[tokio::test]
    async fn test_health_checker_creation() {
        let pool = Arc::new(UpstreamPool::new());
        let checker = HealthChecker::new(pool);
        
        // Checker should be created successfully
        let _ = checker;
    }

    #[tokio::test]
    async fn test_start_and_stop_health_check() {
        let pool = Arc::new(UpstreamPool::new());
        let checker = HealthChecker::new(Arc::clone(&pool));

        // Add an upstream
        let config = create_test_config("test-upstream");
        let _ = pool.add_upstream(config).await;

        // Start health check
        checker.start_health_check("test-upstream".to_string(), 1).await;

        // Wait a bit
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Stop health check
        checker.stop_health_check("test-upstream").await;

        // Stop all
        checker.stop_all().await;
    }

    #[tokio::test]
    async fn test_stop_all_health_checks() {
        let pool = Arc::new(UpstreamPool::new());
        let checker = HealthChecker::new(Arc::clone(&pool));

        // Add multiple upstreams
        let config1 = create_test_config("upstream1");
        let config2 = create_test_config("upstream2");
        let _ = pool.add_upstream(config1).await;
        let _ = pool.add_upstream(config2).await;

        // Start health checks
        checker.start_health_check("upstream1".to_string(), 1).await;
        checker.start_health_check("upstream2".to_string(), 1).await;

        // Wait a bit
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Stop all
        checker.stop_all().await;

        // All tasks should be stopped
        let tasks = checker.check_tasks.lock().await;
        assert!(tasks.is_empty());
    }
}
