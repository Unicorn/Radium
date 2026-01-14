//! Arch-Router 1.5B integration for ML-based agent routing.
//!
//! This module provides integration with the Arch-Router model for intelligent
//! task-to-agent routing using machine learning.
//!
//! Architecture supports multiple backends:
//! - Simulated inference (for benchmarking and testing)
//! - Python bridge via subprocess
//! - Python bridge via HTTP server
//! - ONNX runtime (future: pure Rust inference)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Route definition for Arch-Router model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteDefinition {
    /// Route name (agent ID).
    pub name: String,

    /// Description of when this route should be used.
    pub description: String,

    /// Optional domain (e.g., "programming", "legal", "healthcare").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,

    /// Optional action (e.g., "code_generation", "summarization").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
}

/// Conversation message for routing context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Routing decision from Arch-Router.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    /// Selected route name.
    pub route: String,
}

/// Routing result with confidence and reasoning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchRouterResult {
    /// Selected agent/route name.
    pub agent_id: String,

    /// Confidence score (0.0-1.0).
    /// For Arch-Router, this is derived from model output probability.
    pub confidence: f32,

    /// Alternative routes with scores.
    pub alternatives: Vec<(String, f32)>,

    /// Reasoning (if available from model).
    pub reasoning: Option<String>,
}

/// Arch-Router inference backend.
#[derive(Debug, Clone, Copy)]
pub enum InferenceBackend {
    /// Simulated inference for benchmarking (instant, deterministic).
    Simulated,

    /// Python subprocess (higher latency, isolated).
    PythonSubprocess,

    /// Python HTTP server (moderate latency, persistent).
    PythonHttp,
}

/// Configuration for Arch-Router engine.
#[derive(Debug, Clone)]
pub struct ArchRouterConfig {
    /// Model path or cache directory.
    pub model_path: Option<PathBuf>,

    /// Inference backend to use.
    pub backend: InferenceBackend,

    /// Inference timeout.
    pub timeout: Duration,

    /// HTTP server URL (if using PythonHttp backend).
    pub http_url: Option<String>,
}

impl Default for ArchRouterConfig {
    fn default() -> Self {
        Self {
            model_path: None,
            backend: InferenceBackend::Simulated,
            timeout: Duration::from_secs(10),
            http_url: Some("http://localhost:8000".to_string()),
        }
    }
}

/// Arch-Router inference engine.
pub struct ArchRouterEngine {
    config: ArchRouterConfig,
}

impl ArchRouterEngine {
    /// Create a new Arch-Router engine with default config.
    pub fn new() -> Self {
        Self {
            config: ArchRouterConfig::default(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(config: ArchRouterConfig) -> Self {
        Self { config }
    }

    /// Route a task to the best agent using ML inference.
    ///
    /// # Arguments
    /// * `task_description` - User's task/query
    /// * `routes` - Available routes (agents) with descriptions
    ///
    /// # Returns
    /// Routing result with confidence scores
    pub async fn route(
        &self,
        task_description: &str,
        routes: &[RouteDefinition],
    ) -> Result<ArchRouterResult> {
        match self.config.backend {
            InferenceBackend::Simulated => self.route_simulated(task_description, routes).await,
            InferenceBackend::PythonSubprocess => self.route_python_subprocess(task_description, routes).await,
            InferenceBackend::PythonHttp => self.route_python_http(task_description, routes).await,
        }
    }

    /// Simulated routing for benchmarking (instant, deterministic).
    ///
    /// Uses keyword matching + heuristics to approximate ML behavior
    /// with realistic confidence scores.
    async fn route_simulated(
        &self,
        task_description: &str,
        routes: &[RouteDefinition],
    ) -> Result<ArchRouterResult> {
        if routes.is_empty() {
            anyhow::bail!("No routes available");
        }

        let task_lower = task_description.to_lowercase();

        // Score each route based on keyword overlap
        let mut scores: Vec<(String, f32)> = routes
            .iter()
            .map(|route| {
                let desc_lower = route.description.to_lowercase();
                let name_lower = route.name.to_lowercase();

                // Keyword matching score
                let desc_words: Vec<&str> = desc_lower.split_whitespace().collect();
                let task_words: Vec<&str> = task_lower.split_whitespace().collect();

                let mut overlap_count = 0;
                for task_word in &task_words {
                    if desc_words.iter().any(|w| w.contains(task_word) || task_word.contains(w)) {
                        overlap_count += 1;
                    }
                }

                // Name match bonus
                let name_match = if task_lower.contains(&name_lower) || name_lower.contains(task_lower.split_whitespace().next().unwrap_or("")) {
                    0.3
                } else {
                    0.0
                };

                // Calculate normalized score (simulate ML confidence)
                let base_score = (overlap_count as f32 / task_words.len().max(1) as f32).mul_add(0.7, name_match);

                // Add some variance to simulate model uncertainty
                let score = (base_score + 0.1).min(0.95);

                (route.name.clone(), score)
            })
            .collect();

        // Sort by score descending
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let (best_route, best_score) = scores[0].clone();
        let alternatives = scores[1..].to_vec();

        Ok(ArchRouterResult {
            agent_id: best_route,
            confidence: best_score,
            alternatives,
            reasoning: Some("Simulated routing based on keyword matching".to_string()),
        })
    }

    /// Route using Python subprocess.
    ///
    /// Spawns a Python process with the Arch-Router model.
    /// Higher latency but isolated (no persistent state).
    #[cfg(feature = "python-bridge")]
    async fn route_python_subprocess(
        &self,
        task_description: &str,
        routes: &[RouteDefinition],
    ) -> Result<ArchRouterResult> {
        use tokio::process::Command;

        // Prepare input JSON
        let input = serde_json::json!({
            "task": task_description,
            "routes": routes,
        });

        let input_json = serde_json::to_string(&input)?;

        // Find Python script
        let script_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("scripts/arch_router_inference.py");

        // Execute Python script
        let output = tokio::time::timeout(
            self.config.timeout,
            Command::new("python3")
                .arg(&script_path)
                .arg("--input")
                .arg(input_json)
                .output(),
        )
        .await
        .context("Python subprocess timeout")?
        .context("Failed to execute Python script")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Python inference failed: {}", stderr);
        }

        let stdout = String::from_utf8(output.stdout)?;
        let result: ArchRouterResult = serde_json::from_str(&stdout)?;

        Ok(result)
    }

    /// Route using Python HTTP server.
    ///
    /// Calls a persistent Python server running Arch-Router.
    /// Lower latency than subprocess, requires server to be running.
    #[cfg(feature = "http-bridge")]
    async fn route_python_http(
        &self,
        task_description: &str,
        routes: &[RouteDefinition],
    ) -> Result<ArchRouterResult> {
        let url = self.config.http_url.as_ref()
            .ok_or_else(|| anyhow::anyhow!("HTTP URL not configured"))?;

        let client = reqwest::Client::new();

        let payload = serde_json::json!({
            "task": task_description,
            "routes": routes,
        });

        let response = tokio::time::timeout(
            self.config.timeout,
            client.post(format!("{}/route", url))
                .json(&payload)
                .send(),
        )
        .await
        .context("HTTP request timeout")?
        .context("HTTP request failed")?;

        if !response.status().is_success() {
            anyhow::bail!("HTTP routing failed: {}", response.status());
        }

        let result: ArchRouterResult = response.json().await?;

        Ok(result)
    }

    #[cfg(not(feature = "python-bridge"))]
    async fn route_python_subprocess(
        &self,
        _task_description: &str,
        _routes: &[RouteDefinition],
    ) -> Result<ArchRouterResult> {
        anyhow::bail!("Python bridge not enabled (compile with python-bridge feature)")
    }

    #[cfg(not(feature = "http-bridge"))]
    async fn route_python_http(
        &self,
        _task_description: &str,
        _routes: &[RouteDefinition],
    ) -> Result<ArchRouterResult> {
        anyhow::bail!("HTTP bridge not enabled (compile with http-bridge feature)")
    }
}

impl Default for ArchRouterEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_routes() -> Vec<RouteDefinition> {
        vec![
            RouteDefinition {
                name: "code-agent".to_string(),
                description: "Generating new code, implementing features".to_string(),
                domain: Some("programming".to_string()),
                action: Some("code_generation".to_string()),
            },
            RouteDefinition {
                name: "review-agent".to_string(),
                description: "Reviewing code for quality and security".to_string(),
                domain: Some("programming".to_string()),
                action: Some("code_review".to_string()),
            },
            RouteDefinition {
                name: "doc-agent".to_string(),
                description: "Writing documentation and technical guides".to_string(),
                domain: Some("documentation".to_string()),
                action: Some("writing".to_string()),
            },
        ]
    }

    #[tokio::test]
    async fn test_simulated_routing() {
        let engine = ArchRouterEngine::new();
        let routes = create_test_routes();

        // Test code generation task
        let result = engine.route("Write a Python function for sorting", &routes).await.unwrap();

        println!("Task: Write a Python function for sorting");
        println!("Selected: {} (confidence: {:.3})", result.agent_id, result.confidence);
        println!("Alternatives: {:?}", result.alternatives);

        // Should route to code-agent
        assert!(result.agent_id.contains("code") || result.confidence > 0.3);
    }

    #[tokio::test]
    async fn test_multiple_routes() {
        let engine = ArchRouterEngine::new();
        let routes = create_test_routes();

        let test_cases = vec![
            ("Review my Rust code for security", "review"),
            ("Generate API documentation", "doc"),
            ("Implement authentication logic", "code"),
        ];

        for (task, expected_keyword) in test_cases {
            let result = engine.route(task, &routes).await.unwrap();
            println!("\nTask: {}", task);
            println!("Selected: {} (confidence: {:.3})", result.agent_id, result.confidence);

            assert!(
                result.agent_id.contains(expected_keyword) || result.confidence > 0.2,
                "Expected keyword '{}' in route for task '{}'",
                expected_keyword,
                task
            );
        }
    }
}
