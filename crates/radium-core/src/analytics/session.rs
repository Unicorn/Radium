//! Session analytics and metrics tracking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use crate::analytics::code_changes::CodeChanges;
use crate::monitoring::{MonitoringService, TelemetryTracking};

/// Session metrics aggregated from telemetry and agent activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetrics {
    /// Session ID
    pub session_id: String,
    /// Session start time
    pub start_time: DateTime<Utc>,
    /// Session end time (if completed)
    pub end_time: Option<DateTime<Utc>>,
    /// Wall time (total session duration)
    pub wall_time: Duration,
    /// Agent active time (time agents were actually running)
    pub agent_active_time: Duration,
    /// API time (time spent in API calls)
    pub api_time: Duration,
    /// Tool execution time
    pub tool_time: Duration,
    /// Total tool calls
    pub tool_calls: u64,
    /// Successful tool calls
    pub successful_tool_calls: u64,
    /// Failed tool calls
    pub failed_tool_calls: u64,
    /// Tool approvals allowed
    pub tool_approvals_allowed: u64,
    /// Tool approvals denied
    pub tool_approvals_denied: u64,
    /// Tool approvals asked (user interaction required)
    pub tool_approvals_asked: u64,
    /// Code changes (lines added)
    pub lines_added: i64,
    /// Code changes (lines removed)
    pub lines_removed: i64,
    /// Model usage statistics (model -> (requests, input_tokens, output_tokens))
    pub model_usage: HashMap<String, ModelUsageStats>,
    /// Engine usage statistics (engine_id -> (requests, input_tokens, output_tokens))
    pub engine_usage: HashMap<String, ModelUsageStats>,
    /// Total cached tokens
    pub total_cached_tokens: u64,
    /// Total cache creation tokens
    pub total_cache_creation_tokens: u64,
    /// Total cache read tokens
    pub total_cache_read_tokens: u64,
    /// Total estimated cost
    pub total_cost: f64,
}

/// Model usage statistics for a single model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsageStats {
    /// Number of requests
    pub requests: u64,
    /// Total input tokens
    pub input_tokens: u64,
    /// Total output tokens
    pub output_tokens: u64,
    /// Cached tokens
    pub cached_tokens: u64,
    /// Estimated cost
    pub estimated_cost: f64,
}

impl Default for SessionMetrics {
    fn default() -> Self {
        Self {
            session_id: String::new(),
            start_time: Utc::now(),
            end_time: None,
            wall_time: Duration::ZERO,
            agent_active_time: Duration::ZERO,
            api_time: Duration::ZERO,
            tool_time: Duration::ZERO,
            tool_calls: 0,
            successful_tool_calls: 0,
            failed_tool_calls: 0,
            tool_approvals_allowed: 0,
            tool_approvals_denied: 0,
            tool_approvals_asked: 0,
            lines_added: 0,
            lines_removed: 0,
            model_usage: HashMap::new(),
            engine_usage: HashMap::new(),
            total_cached_tokens: 0,
            total_cache_creation_tokens: 0,
            total_cache_read_tokens: 0,
            total_cost: 0.0,
        }
    }
}

impl SessionMetrics {
    /// Calculate success rate percentage.
    pub fn success_rate(&self) -> f64 {
        if self.tool_calls == 0 {
            0.0
        } else {
            (self.successful_tool_calls as f64 / self.tool_calls as f64) * 100.0
        }
    }

    /// Calculate cache hit rate percentage.
    pub fn cache_hit_rate(&self) -> f64 {
        let total_input = self.model_usage.values().map(|s| s.input_tokens).sum::<u64>();
        if total_input == 0 {
            0.0
        } else {
            (self.total_cached_tokens as f64 / total_input as f64) * 100.0
        }
    }

    /// Get total tokens across all models.
    pub fn total_tokens(&self) -> (u64, u64) {
        self.model_usage.values().fold((0u64, 0u64), |(inp, out), stats| {
            (inp + stats.input_tokens, out + stats.output_tokens)
        })
    }
}

/// Session analytics aggregator.
pub struct SessionAnalytics {
    pub(crate) monitoring: MonitoringService,
}

impl SessionAnalytics {
    /// Create a new session analytics instance.
    pub fn new(monitoring: MonitoringService) -> Self {
        Self { monitoring }
    }

    /// Get reference to monitoring service.
    pub fn monitoring(&self) -> &MonitoringService {
        &self.monitoring
    }

    /// Generate metrics for a session from agent IDs.
    ///
    /// Aggregates telemetry data from all agents in the session.
    pub fn generate_session_metrics(
        &self,
        session_id: &str,
        agent_ids: &[String],
        start_time: DateTime<Utc>,
        end_time: Option<DateTime<Utc>>,
    ) -> anyhow::Result<SessionMetrics> {
        self.generate_session_metrics_with_workspace(
            session_id, agent_ids, start_time, end_time, None,
        )
    }

    /// Generate metrics for a session with workspace for code change tracking.
    pub fn generate_session_metrics_with_workspace(
        &self,
        session_id: &str,
        agent_ids: &[String],
        start_time: DateTime<Utc>,
        end_time: Option<DateTime<Utc>>,
        workspace_root: Option<&Path>,
    ) -> anyhow::Result<SessionMetrics> {
        let mut metrics = SessionMetrics {
            session_id: session_id.to_string(),
            start_time,
            end_time,
            wall_time: if let Some(end) = end_time {
                end.signed_duration_since(start_time).to_std()?
            } else {
                Utc::now().signed_duration_since(start_time).to_std()?
            },
            ..Default::default()
        };

        // Aggregate telemetry from all agents
        for agent_id in agent_ids {
            let telemetry = self.monitoring.get_agent_telemetry(agent_id)?;

            for record in telemetry {
                // Aggregate model usage
                let model_key = record.model.clone().unwrap_or_else(|| "unknown".to_string());

                let stats = metrics.model_usage.entry(model_key.clone()).or_insert_with(|| {
                    ModelUsageStats {
                        requests: 0,
                        input_tokens: 0,
                        output_tokens: 0,
                        cached_tokens: 0,
                        estimated_cost: 0.0,
                    }
                });

                stats.requests += 1;
                stats.input_tokens += record.input_tokens;
                stats.output_tokens += record.output_tokens;
                stats.cached_tokens += record.cached_tokens;
                stats.estimated_cost += record.estimated_cost;

                // Aggregate engine usage
                if let Some(ref engine_id) = record.engine_id {
                    let engine_stats = metrics.engine_usage.entry(engine_id.clone()).or_insert_with(|| {
                        ModelUsageStats {
                            requests: 0,
                            input_tokens: 0,
                            output_tokens: 0,
                            cached_tokens: 0,
                            estimated_cost: 0.0,
                        }
                    });

                    engine_stats.requests += 1;
                    engine_stats.input_tokens += record.input_tokens;
                    engine_stats.output_tokens += record.output_tokens;
                    engine_stats.cached_tokens += record.cached_tokens;
                    engine_stats.estimated_cost += record.estimated_cost;
                }

                // Aggregate totals
                metrics.total_cached_tokens += record.cached_tokens;
                metrics.total_cache_creation_tokens += record.cache_creation_tokens;
                metrics.total_cache_read_tokens += record.cache_read_tokens;
                metrics.total_cost += record.estimated_cost;

                // Aggregate tool approval metrics
                if let Some(approved) = record.tool_approved {
                    if approved {
                        metrics.tool_approvals_allowed += 1;
                    } else {
                        metrics.tool_approvals_denied += 1;
                    }
                }
                if let Some(approval_type) = &record.tool_approval_type {
                    if approval_type == "user" {
                        metrics.tool_approvals_asked += 1;
                    }
                }

                // Estimate API time (rough: assume 100ms per request)
                metrics.api_time += Duration::from_millis(100);
            }

            // Get agent record for timing
            if let Ok(agent) = self.monitoring.get_agent(agent_id) {
                if let Some(end) = agent.end_time {
                    let duration = Duration::from_secs(end.saturating_sub(agent.start_time));
                    metrics.agent_active_time += duration;
                }
            }
        }

        // Estimate tool time (remaining active time after API time)
        if metrics.agent_active_time > metrics.api_time {
            metrics.tool_time = metrics.agent_active_time - metrics.api_time;
        }

        // Calculate code changes if workspace is available
        if let Some(workspace) = workspace_root {
            if let Ok(code_changes) =
                CodeChanges::from_git_since(workspace, start_time.timestamp() as u64)
            {
                metrics.lines_added = code_changes.lines_added;
                metrics.lines_removed = code_changes.lines_removed;
            }
        }

        Ok(metrics)
    }

    /// Get metrics for a specific session by ID.
    pub fn get_session_metrics(&self, session_id: &str) -> anyhow::Result<SessionMetrics> {
        // Try to discover workspace for code change tracking
        let workspace_root =
            crate::workspace::Workspace::discover().ok().map(|w| w.root().to_path_buf());

        // Find all agents associated with this session
        // For now, we'll use a simple approach: session_id might be in agent_id or plan_id
        let agents = self.monitoring.list_agents()?;
        let session_agents: Vec<String> = agents
            .iter()
            .filter(|a| {
                a.id.contains(session_id)
                    || a.plan_id.as_ref().is_some_and(|p| p.contains(session_id))
            })
            .map(|a| a.id.clone())
            .collect();

        if session_agents.is_empty() {
            return Err(anyhow::anyhow!("No agents found for session {}", session_id));
        }

        // Get earliest start time and latest end time
        let mut start_time = Utc::now();
        let mut end_time: Option<DateTime<Utc>> = None;

        for agent_id in &session_agents {
            if let Ok(agent) = self.monitoring.get_agent(agent_id) {
                let agent_start =
                    DateTime::from_timestamp(agent.start_time as i64, 0).unwrap_or_else(Utc::now);
                if agent_start < start_time {
                    start_time = agent_start;
                }
                if let Some(end) = agent.end_time {
                    let agent_end =
                        DateTime::from_timestamp(end as i64, 0).unwrap_or_else(Utc::now);
                    end_time = Some(end_time.map_or(agent_end, |e| e.max(agent_end)));
                }
            }
        }

        self.generate_session_metrics_with_workspace(
            session_id,
            &session_agents,
            start_time,
            end_time,
            workspace_root.as_deref(),
        )
    }

    /// Get aggregated model usage statistics across all sessions.
    ///
    /// Loads all session reports from storage and aggregates model usage statistics.
    pub fn get_aggregated_model_usage(
        &self,
        workspace_root: Option<&Path>,
    ) -> anyhow::Result<HashMap<String, ModelUsageStats>> {
        use super::storage::SessionStorage;

        // Try to discover workspace if not provided
        let workspace = workspace_root
            .map(|p| p.to_path_buf())
            .or_else(|| crate::workspace::Workspace::discover().ok().map(|w| w.root().to_path_buf()));

        let workspace_path = workspace.ok_or_else(|| anyhow::anyhow!("Workspace not found"))?;

        let storage = SessionStorage::new(&workspace_path)?;
        let reports = storage.list_reports()?;

        // Aggregate model usage from all reports
        let mut aggregated: HashMap<String, ModelUsageStats> = HashMap::new();

        for report in reports {
            for (model, stats) in &report.metrics.model_usage {
                let entry = aggregated.entry(model.clone()).or_insert_with(|| {
                    ModelUsageStats {
                        requests: 0,
                        input_tokens: 0,
                        output_tokens: 0,
                        cached_tokens: 0,
                        estimated_cost: 0.0,
                    }
                });

                entry.requests += stats.requests;
                entry.input_tokens += stats.input_tokens;
                entry.output_tokens += stats.output_tokens;
                entry.cached_tokens += stats.cached_tokens;
                entry.estimated_cost += stats.estimated_cost;
            }
        }

        Ok(aggregated)
    }
}
