//! Telemetry parsing and tracking for token usage and costs.

use super::error::{MonitoringError, Result};
use super::service::MonitoringService;
use crate::hooks::registry::HookType;
use crate::hooks::types::HookContext;
use async_trait::async_trait;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Telemetry record for token usage and costs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryRecord {
    /// Agent ID this telemetry belongs to.
    pub agent_id: String,

    /// Timestamp of telemetry capture.
    pub timestamp: u64,

    /// Input/prompt tokens.
    pub input_tokens: u64,

    /// Output/completion tokens.
    pub output_tokens: u64,

    /// Cached tokens (reused from cache).
    pub cached_tokens: u64,

    /// Cache creation tokens (tokens written to cache).
    pub cache_creation_tokens: u64,

    /// Cache read tokens (tokens read from cache).
    pub cache_read_tokens: u64,

    /// Total tokens (input + output).
    pub total_tokens: u64,

    /// Estimated cost in USD.
    pub estimated_cost: f64,

    /// Model name.
    pub model: Option<String>,

    /// Provider name.
    pub provider: Option<String>,
    
    /// Tool name (if this telemetry is associated with a tool call).
    pub tool_name: Option<String>,
    
    /// Tool arguments (JSON string representation).
    pub tool_args: Option<String>,
    
    /// Whether the tool was approved (true) or denied (false).
    pub tool_approved: Option<bool>,
    
    /// Tool approval type ("user", "auto", "policy").
    pub tool_approval_type: Option<String>,
    
    /// Engine ID used for this execution (e.g., "claude", "openai", "gemini").
    pub engine_id: Option<String>,
    
    /// Behavior type if this telemetry is for a behavior (e.g., "loop", "trigger", "checkpoint", "vibecheck").
    pub behavior_type: Option<String>,
    
    /// Behavior invocation count (for loop behavior).
    pub behavior_invocation_count: Option<u64>,
    
    /// Behavior evaluation duration in milliseconds.
    pub behavior_duration_ms: Option<u64>,
    
    /// Behavior outcome (e.g., "triggered", "skipped", "failed").
    pub behavior_outcome: Option<String>,
    
    /// API key identifier (hash of API key used for this execution).
    pub api_key_id: Option<String>,
    
    /// Team name for cost attribution (derived from API key metadata).
    pub team_name: Option<String>,
    
    /// Project name for cost attribution (derived from API key metadata).
    pub project_name: Option<String>,
    
    /// Cost center for chargeback (derived from API key metadata).
    pub cost_center: Option<String>,
    
    /// Model tier used ("smart" | "eco").
    pub model_tier: Option<String>,
    
    /// Routing decision type ("auto" | "manual" | "override" | "fallback").
    pub routing_decision: Option<String>,
    
    /// Complexity score (0-100) if routing was used.
    pub complexity_score: Option<f64>,
    
    /// A/B test group assignment ("control" | "test") if A/B testing was used.
    pub ab_test_group: Option<String>,
    
    /// Finish reason from model response (e.g., "stop", "length", "safety").
    pub finish_reason: Option<String>,
    
    /// Whether content was blocked by safety filters.
    pub safety_blocked: bool,
    
    /// Number of citations in the response.
    pub citation_count: Option<u32>,
}

impl TelemetryRecord {
    /// Creates a new telemetry record.
    pub fn new(agent_id: String) -> Self {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        Self {
            agent_id,
            timestamp,
            input_tokens: 0,
            output_tokens: 0,
            cached_tokens: 0,
            cache_creation_tokens: 0,
            cache_read_tokens: 0,
            total_tokens: 0,
            estimated_cost: 0.0,
            model: None,
            provider: None,
            tool_name: None,
            tool_args: None,
            tool_approved: None,
            tool_approval_type: None,
            engine_id: None,
            behavior_type: None,
            behavior_invocation_count: None,
            behavior_duration_ms: None,
            behavior_outcome: None,
            api_key_id: None,
            team_name: None,
            project_name: None,
            cost_center: None,
            model_tier: None,
            routing_decision: None,
            complexity_score: None,
            ab_test_group: None,
            finish_reason: None,
            safety_blocked: false,
            citation_count: None,
        }
    }

    /// Sets token counts.
    #[must_use]
    pub fn with_tokens(mut self, input: u64, output: u64) -> Self {
        self.input_tokens = input;
        self.output_tokens = output;
        self.total_tokens = input + output;
        self
    }

    /// Sets cache statistics.
    #[must_use]
    pub fn with_cache_stats(mut self, cached: u64, creation: u64, read: u64) -> Self {
        self.cached_tokens = cached;
        self.cache_creation_tokens = creation;
        self.cache_read_tokens = read;
        self
    }

    /// Sets model information.
    #[must_use]
    pub fn with_model(mut self, model: String, provider: String) -> Self {
        self.model = Some(model);
        self.provider = Some(provider);
        self
    }

    /// Sets tool approval information.
    #[must_use]
    pub fn with_tool_approval(
        mut self,
        tool_name: String,
        tool_args: Option<Vec<String>>,
        approved: bool,
        approval_type: String,
    ) -> Self {
        self.tool_name = Some(tool_name);
        self.tool_args = tool_args.map(|args| serde_json::to_string(&args).unwrap_or_default());
        self.tool_approved = Some(approved);
        self.tool_approval_type = Some(approval_type);
        self
    }

    /// Sets engine ID.
    #[must_use]
    pub fn with_engine_id(mut self, engine_id: String) -> Self {
        self.engine_id = Some(engine_id);
        self
    }

    /// Sets behavior metrics.
    #[must_use]
    pub fn with_behavior_metrics(
        mut self,
        behavior_type: String,
        invocation_count: Option<u64>,
        duration_ms: Option<u64>,
        outcome: Option<String>,
    ) -> Self {
        self.behavior_type = Some(behavior_type);
        self.behavior_invocation_count = invocation_count;
        self.behavior_duration_ms = duration_ms;
        self.behavior_outcome = outcome;
        self
    }

    /// Sets API key attribution metadata.
    #[must_use]
    pub fn with_attribution(
        mut self,
        api_key_id: Option<String>,
        team_name: Option<String>,
        project_name: Option<String>,
        cost_center: Option<String>,
    ) -> Self {
        self.api_key_id = api_key_id;
        self.team_name = team_name;
        self.project_name = project_name;
        self.cost_center = cost_center;
        self
    }

    /// Sets model tier used for routing ("smart" | "eco").
    #[must_use]
    pub fn with_model_tier(mut self, tier: String) -> Self {
        self.model_tier = Some(tier);
        self
    }

    /// Sets routing decision type ("auto" | "manual" | "override" | "fallback").
    #[must_use]
    pub fn with_routing_decision(mut self, decision: String) -> Self {
        self.routing_decision = Some(decision);
        self
    }

    /// Sets complexity score (0-100) from routing decision.
    #[must_use]
    pub fn with_complexity_score(mut self, score: f64) -> Self {
        self.complexity_score = Some(score);
        self
    }

    /// Sets the A/B test group for this telemetry record.
    pub fn with_ab_test_group(mut self, group: String) -> Self {
        self.ab_test_group = Some(group);
        self
    }

    /// Sets local model cost based on execution duration.
    ///
    /// This method calculates cost for local/self-hosted models using duration-based
    /// pricing from the cost tracker. Use this for local engines (Ollama, LM Studio, etc.)
    /// instead of token-based cost calculation.
    ///
    /// # Arguments
    /// * `engine_id` - Engine identifier (e.g., "ollama", "lm-studio")
    /// * `duration` - Execution duration
    /// * `cost_tracker` - Local model cost tracker
    ///
    /// # Returns
    /// Self with populated cost, duration, engine_id, and provider fields
    pub fn with_local_cost(
        mut self,
        engine_id: &str,
        duration: std::time::Duration,
        cost_tracker: &crate::monitoring::LocalModelCostTracker,
    ) -> Self {
        // Calculate cost using the cost tracker
        let cost = cost_tracker.calculate_cost(engine_id, duration);

        // Populate telemetry fields
        self.estimated_cost = cost;
        self.behavior_duration_ms = Some(duration.as_millis() as u64);
        self.engine_id = Some(engine_id.to_string());
        self.provider = Some("local".to_string());

        self
    }

    /// Calculates estimated cost based on model pricing.
    /// Uses engine-specific pricing when engine_id is set, otherwise falls back to model-based pricing.
    pub fn calculate_cost(&mut self) -> &mut Self {
        // Try engine-specific pricing first
        let (input_price, output_price) = if let Some(ref engine_id) = self.engine_id {
            match engine_id.as_str() {
                "openai" => {
                    // OpenAI pricing varies by model
                    match self.model.as_deref() {
                        Some("gpt-4") | Some("gpt-4-turbo") => (30.0, 60.0),
                        Some("gpt-3.5-turbo") => (0.5, 1.5),
                        _ => (10.0, 30.0), // Default for OpenAI
                    }
                }
                "claude" => {
                    // Claude pricing varies by model
                    match self.model.as_deref() {
                        Some("claude-3-opus") | Some("claude-3-opus-20240229") => (15.0, 75.0),
                        Some("claude-3-sonnet") | Some("claude-3-sonnet-20240229") => (3.0, 15.0),
                        Some("claude-3-haiku") | Some("claude-3-haiku-20240307") => (0.25, 1.25),
                        _ => (3.0, 15.0), // Default for Claude
                    }
                }
                "gemini" => {
                    // Gemini pricing
                    match self.model.as_deref() {
                        Some("gemini-pro") | Some("gemini-2.0-flash-exp") => (0.5, 1.5),
                        _ => (0.5, 1.5), // Default for Gemini
                    }
                }
                "mock" => (0.0, 0.0), // Mock engine is free
                _ => (1.0, 2.0), // Default fallback for unknown engines
            }
        } else {
            // Fall back to model-based pricing (legacy)
            match self.model.as_deref() {
                Some("gpt-4") => (30.0, 60.0),
                Some("gpt-3.5-turbo") => (0.5, 1.5),
                Some("claude-3-opus") => (15.0, 75.0),
                Some("claude-3-sonnet") => (3.0, 15.0),
                Some("claude-3-haiku") => (0.25, 1.25),
                Some("gemini-pro") => (0.5, 1.5),
                _ => (1.0, 2.0), // Default fallback
            }
        };

        #[allow(clippy::cast_precision_loss)]
        let input_cost = (self.input_tokens as f64 / 1_000_000.0) * input_price;
        #[allow(clippy::cast_precision_loss)]
        let output_cost = (self.output_tokens as f64 / 1_000_000.0) * output_price;

        self.estimated_cost = input_cost + output_cost;
        self
    }
}

/// Telemetry parser for extracting token usage from model outputs.
pub struct TelemetryParser;

impl TelemetryParser {
    /// Parses OpenAI-style usage output.
    ///
    /// Expected format:
    /// ```json
    /// {
    ///   "usage": {
    ///     "prompt_tokens": 100,
    ///     "completion_tokens": 50,
    ///     "total_tokens": 150
    ///   }
    /// }
    /// ```
    pub fn parse_openai(json: &str) -> Result<(u64, u64)> {
        #[derive(Deserialize)]
        struct Usage {
            prompt_tokens: u64,
            completion_tokens: u64,
        }

        #[derive(Deserialize)]
        struct Response {
            usage: Usage,
        }

        let response: Response = serde_json::from_str(json)
            .map_err(|e| MonitoringError::TelemetryParse(e.to_string()))?;

        Ok((response.usage.prompt_tokens, response.usage.completion_tokens))
    }

    /// Parses Anthropic-style usage output.
    ///
    /// Expected format:
    /// ```json
    /// {
    ///   "usage": {
    ///     "input_tokens": 100,
    ///     "output_tokens": 50
    ///   }
    /// }
    /// ```
    pub fn parse_anthropic(json: &str) -> Result<(u64, u64)> {
        #[derive(Deserialize)]
        struct Usage {
            input_tokens: u64,
            output_tokens: u64,
        }

        #[derive(Deserialize)]
        struct Response {
            usage: Usage,
        }

        let response: Response = serde_json::from_str(json)
            .map_err(|e| MonitoringError::TelemetryParse(e.to_string()))?;

        Ok((response.usage.input_tokens, response.usage.output_tokens))
    }

    /// Parses Google Gemini-style usage output.
    ///
    /// Expected format:
    /// ```json
    /// {
    ///   "usageMetadata": {
    ///     "promptTokenCount": 100,
    ///     "candidatesTokenCount": 50,
    ///     "totalTokenCount": 150
    ///   }
    /// }
    /// ```
    pub fn parse_gemini(json: &str) -> Result<(u64, u64)> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct UsageMetadata {
            prompt_token_count: u64,
            candidates_token_count: u64,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Response {
            usage_metadata: UsageMetadata,
        }

        let response: Response = serde_json::from_str(json)
            .map_err(|e| MonitoringError::TelemetryParse(e.to_string()))?;

        Ok((
            response.usage_metadata.prompt_token_count,
            response.usage_metadata.candidates_token_count,
        ))
    }
}

/// Telemetry summary for aggregated queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySummary {
    /// Agent ID.
    pub agent_id: String,
    /// Total tokens across all telemetry records.
    pub total_tokens: u64,
    /// Total estimated cost across all telemetry records.
    pub total_cost: f64,
    /// Number of telemetry records.
    pub record_count: u64,
}

/// Extension trait for MonitoringService to add telemetry tracking.
#[async_trait(?Send)]
pub trait TelemetryTracking {
    /// Records telemetry for an agent.
    async fn record_telemetry(&self, record: &TelemetryRecord) -> Result<()>;

    /// Gets telemetry records for an agent.
    fn get_agent_telemetry(&self, agent_id: &str) -> Result<Vec<TelemetryRecord>>;

    /// Gets total token usage for an agent.
    fn get_total_tokens(&self, agent_id: &str) -> Result<(u64, u64, u64)>;

    /// Gets total estimated cost for an agent.
    fn get_total_cost(&self, agent_id: &str) -> Result<f64>;

    /// Gets telemetry summary for all agents (optimized aggregation).
    fn get_telemetry_summary(&self) -> Result<Vec<TelemetrySummary>>;
    
    /// Gets behavior metrics for a workflow or agent.
    fn get_behavior_metrics(&self, workflow_id: Option<&str>) -> Result<Vec<TelemetryRecord>>;
    
    /// Gets behavior metrics filtered by behavior type.
    fn get_behavior_metrics_by_type(&self, behavior_type: &str) -> Result<Vec<TelemetryRecord>>;
}

#[async_trait(?Send)]
impl TelemetryTracking for MonitoringService {
    async fn record_telemetry(&self, record: &TelemetryRecord) -> Result<()> {
        // Execute TelemetryCollection hooks to allow augmentation (outside DB lock)
        let mut effective_record = record.clone();

        // Clone hook registry reference to avoid holding &self across await
        let hook_registry = self.get_hook_registry();
        
        if let Some(registry) = hook_registry {
            let hook_context = HookContext::new(
                "telemetry_collection",
                serde_json::json!({
                    "agent_id": record.agent_id,
                    "input_tokens": record.input_tokens,
                    "output_tokens": record.output_tokens,
                    "total_tokens": record.total_tokens,
                    "estimated_cost": record.estimated_cost,
                    "model": record.model,
                    "provider": record.provider,
                }),
            );

            if let Ok(results) = registry.execute_hooks(HookType::TelemetryCollection, &hook_context).await {
                for result in results {
                    // If hook modifies telemetry, update it
                    if let Some(modified_data) = result.modified_data {
                        if let Some(custom_fields) = modified_data.as_object() {
                            // Allow hooks to add custom fields (we'll store them as JSON in a metadata field if needed)
                            // For now, we just process the standard fields
                            if let Some(new_cost) = custom_fields.get("estimated_cost").and_then(|v| v.as_f64()) {
                                effective_record.estimated_cost = new_cost;
                            }
                        }
                    }
                }
            }
        }

        // Now write to database (synchronous, no await needed)
        // Use the internal sync method
        self.record_telemetry_sync(&effective_record)
    }

    fn get_agent_telemetry(&self, agent_id: &str) -> Result<Vec<TelemetryRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT agent_id, timestamp, input_tokens, output_tokens, cached_tokens,
                    cache_creation_tokens, cache_read_tokens, total_tokens,
                    estimated_cost, model, provider, tool_name, tool_args, tool_approved, tool_approval_type, engine_id,
                    behavior_type, behavior_invocation_count, behavior_duration_ms, behavior_outcome,
                    api_key_id, team_name, project_name, cost_center,
                    model_tier, routing_decision, complexity_score, ab_test_group,
                    finish_reason, safety_blocked, citation_count
             FROM telemetry WHERE agent_id = ?1 ORDER BY timestamp DESC",
        )?;

        let records = stmt
            .query_map(params![agent_id], |row| {
                Ok(TelemetryRecord {
                    agent_id: row.get(0)?,
                    timestamp: row.get(1)?,
                    input_tokens: row.get(2)?,
                    output_tokens: row.get(3)?,
                    cached_tokens: row.get(4)?,
                    cache_creation_tokens: row.get(5)?,
                    cache_read_tokens: row.get(6)?,
                    total_tokens: row.get(7)?,
                    estimated_cost: row.get(8)?,
                    model: row.get(9)?,
                    provider: row.get(10)?,
                    tool_name: row.get(11)?,
                    tool_args: row.get(12)?,
                    tool_approved: row.get(13)?,
                    tool_approval_type: row.get(14)?,
                    engine_id: row.get(15)?,
                    behavior_type: row.get(16).ok(),
                    behavior_invocation_count: row.get(17).ok(),
                    behavior_duration_ms: row.get(18).ok(),
                    behavior_outcome: row.get(19).ok(),
                    api_key_id: row.get(20).ok(),
                    team_name: row.get(21).ok(),
                    project_name: row.get(22).ok(),
                    cost_center: row.get(23).ok(),
                    model_tier: row.get(24).ok(),
                    routing_decision: row.get(25).ok(),
                    complexity_score: row.get(26).ok(),
                    ab_test_group: row.get(27).ok(),
                    finish_reason: row.get(28).ok(),
                    safety_blocked: row.get(29).unwrap_or(false),
                    citation_count: row.get(30).ok(),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(records)
    }

    fn get_total_tokens(&self, agent_id: &str) -> Result<(u64, u64, u64)> {
        let mut stmt = self.conn.prepare(
            "SELECT SUM(input_tokens), SUM(output_tokens), SUM(total_tokens)
             FROM telemetry WHERE agent_id = ?1",
        )?;

        let result = stmt.query_row(params![agent_id], |row| {
            Ok((
                row.get::<_, Option<i64>>(0)?.unwrap_or(0) as u64,
                row.get::<_, Option<i64>>(1)?.unwrap_or(0) as u64,
                row.get::<_, Option<i64>>(2)?.unwrap_or(0) as u64,
            ))
        })?;

        Ok(result)
    }

    fn get_total_cost(&self, agent_id: &str) -> Result<f64> {
        let mut stmt =
            self.conn.prepare("SELECT SUM(estimated_cost) FROM telemetry WHERE agent_id = ?1")?;

        let cost = stmt
            .query_row(params![agent_id], |row| Ok(row.get::<_, Option<f64>>(0)?.unwrap_or(0.0)))?;

        Ok(cost)
    }

    fn get_telemetry_summary(&self) -> Result<Vec<TelemetrySummary>> {
        let mut stmt = self.conn.prepare(
            "SELECT agent_id,
                    SUM(total_tokens) as total_tokens,
                    SUM(estimated_cost) as total_cost,
                    COUNT(*) as record_count
             FROM telemetry
             GROUP BY agent_id
             ORDER BY agent_id",
        )?;

        let summaries = stmt
            .query_map([], |row| {
                Ok(TelemetrySummary {
                    agent_id: row.get(0)?,
                    total_tokens: row.get::<_, Option<i64>>(1)?.unwrap_or(0) as u64,
                    total_cost: row.get::<_, Option<f64>>(2)?.unwrap_or(0.0),
                    record_count: row.get::<_, i64>(3)? as u64,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(summaries)
    }
    
    fn get_behavior_metrics(&self, workflow_id: Option<&str>) -> Result<Vec<TelemetryRecord>> {
        let query = if let Some(_wf_id) = workflow_id {
            // Filter by workflow_id if provided (would need workflow_id in telemetry table for full support)
            // For now, filter by behavior_type is not null
            "SELECT agent_id, timestamp, input_tokens, output_tokens, cached_tokens,
                    cache_creation_tokens, cache_read_tokens, total_tokens,
                    estimated_cost, model, provider, tool_name, tool_args, tool_approved, tool_approval_type, engine_id,
                    behavior_type, behavior_invocation_count, behavior_duration_ms, behavior_outcome,
                    api_key_id, team_name, project_name, cost_center,
                    model_tier, routing_decision, complexity_score, ab_test_group,
                    finish_reason, safety_blocked, citation_count
             FROM telemetry WHERE behavior_type IS NOT NULL ORDER BY timestamp DESC"
        } else {
            "SELECT agent_id, timestamp, input_tokens, output_tokens, cached_tokens,
                    cache_creation_tokens, cache_read_tokens, total_tokens,
                    estimated_cost, model, provider, tool_name, tool_args, tool_approved, tool_approval_type, engine_id,
                    behavior_type, behavior_invocation_count, behavior_duration_ms, behavior_outcome,
                    api_key_id, team_name, project_name, cost_center,
                    model_tier, routing_decision, complexity_score, ab_test_group,
                    finish_reason, safety_blocked, citation_count
             FROM telemetry WHERE behavior_type IS NOT NULL ORDER BY timestamp DESC"
        };
        
        let mut stmt = self.conn.prepare(query)?;
        let records = stmt.query_map([], |row| {
            Ok(TelemetryRecord {
                agent_id: row.get(0)?,
                timestamp: row.get(1)?,
                input_tokens: row.get(2)?,
                output_tokens: row.get(3)?,
                cached_tokens: row.get(4)?,
                cache_creation_tokens: row.get(5)?,
                cache_read_tokens: row.get(6)?,
                total_tokens: row.get(7)?,
                estimated_cost: row.get(8)?,
                model: row.get(9)?,
                provider: row.get(10)?,
                tool_name: row.get(11)?,
                tool_args: row.get(12)?,
                tool_approved: row.get(13)?,
                tool_approval_type: row.get(14)?,
                engine_id: row.get(15)?,
                behavior_type: row.get(16).ok(),
                behavior_invocation_count: row.get(17).ok(),
                behavior_duration_ms: row.get(18).ok(),
                behavior_outcome: row.get(19).ok(),
                api_key_id: row.get(20).ok(),
                team_name: row.get(21).ok(),
                project_name: row.get(22).ok(),
                cost_center: row.get(23).ok(),
                model_tier: row.get(24).ok(),
                routing_decision: row.get(25).ok(),
                complexity_score: row.get(26).ok(),
                ab_test_group: None,  // Not queried in this path
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
        
        Ok(records)
    }
    
    fn get_behavior_metrics_by_type(&self, behavior_type: &str) -> Result<Vec<TelemetryRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT agent_id, timestamp, input_tokens, output_tokens, cached_tokens,
                    cache_creation_tokens, cache_read_tokens, total_tokens,
                    estimated_cost, model, provider, tool_name, tool_args, tool_approved, tool_approval_type, engine_id,
                    behavior_type, behavior_invocation_count, behavior_duration_ms, behavior_outcome,
                    api_key_id, team_name, project_name, cost_center,
                    model_tier, routing_decision, complexity_score
             FROM telemetry WHERE behavior_type = ?1 ORDER BY timestamp DESC",
        )?;
        
        let records = stmt.query_map(params![behavior_type], |row| {
            Ok(TelemetryRecord {
                agent_id: row.get(0)?,
                timestamp: row.get(1)?,
                input_tokens: row.get(2)?,
                output_tokens: row.get(3)?,
                cached_tokens: row.get(4)?,
                cache_creation_tokens: row.get(5)?,
                cache_read_tokens: row.get(6)?,
                total_tokens: row.get(7)?,
                estimated_cost: row.get(8)?,
                model: row.get(9)?,
                provider: row.get(10)?,
                tool_name: row.get(11)?,
                tool_args: row.get(12)?,
                tool_approved: row.get(13)?,
                tool_approval_type: row.get(14)?,
                engine_id: row.get(15)?,
                behavior_type: row.get(16).ok(),
                behavior_invocation_count: row.get(17).ok(),
                behavior_duration_ms: row.get(18).ok(),
                behavior_outcome: row.get(19).ok(),
                api_key_id: row.get(20).ok(),
                team_name: row.get(21).ok(),
                project_name: row.get(22).ok(),
                cost_center: row.get(23).ok(),
                model_tier: row.get(24).ok(),
                routing_decision: row.get(25).ok(),
                complexity_score: row.get(26).ok(),
                ab_test_group: None,  // Not queried in this path
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
        
        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitoring::service::{AgentRecord, MonitoringService};

    #[test]
    fn test_telemetry_record_new() {
        let record = TelemetryRecord::new("agent-1".to_string());
        assert_eq!(record.agent_id, "agent-1");
        assert_eq!(record.input_tokens, 0);
        assert_eq!(record.output_tokens, 0);
    }

    #[test]
    fn test_telemetry_record_with_tokens() {
        let record = TelemetryRecord::new("agent-1".to_string()).with_tokens(100, 50);

        assert_eq!(record.input_tokens, 100);
        assert_eq!(record.output_tokens, 50);
        assert_eq!(record.total_tokens, 150);
    }

    #[test]
    fn test_telemetry_record_calculate_cost() {
        let mut record = TelemetryRecord::new("agent-1".to_string())
            .with_tokens(1_000_000, 1_000_000)
            .with_model("gpt-3.5-turbo".to_string(), "openai".to_string());

        record.calculate_cost();

        // 0.5 (input) + 1.5 (output) = 2.0
        assert!((record.estimated_cost - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_openai() {
        let json =
            r#"{"usage": {"prompt_tokens": 100, "completion_tokens": 50, "total_tokens": 150}}"#;
        let (input, output) = TelemetryParser::parse_openai(json).unwrap();
        assert_eq!(input, 100);
        assert_eq!(output, 50);
    }

    #[test]
    fn test_parse_anthropic() {
        let json = r#"{"usage": {"input_tokens": 100, "output_tokens": 50}}"#;
        let (input, output) = TelemetryParser::parse_anthropic(json).unwrap();
        assert_eq!(input, 100);
        assert_eq!(output, 50);
    }

    #[test]
    fn test_parse_gemini() {
        let json = r#"{"usageMetadata": {"promptTokenCount": 100, "candidatesTokenCount": 50, "totalTokenCount": 150}}"#;
        let (input, output) = TelemetryParser::parse_gemini(json).unwrap();
        assert_eq!(input, 100);
        assert_eq!(output, 50);
    }

    #[tokio::test]
    async fn test_record_telemetry() {
        use super::super::service::AgentRecord;

        let service = MonitoringService::new().unwrap();

        // Register agent first (foreign key constraint)
        let agent = AgentRecord::new("agent-1".to_string(), "developer".to_string());
        service.register_agent(&agent).unwrap();

        let mut record = TelemetryRecord::new("agent-1".to_string())
            .with_tokens(100, 50)
            .with_model("gpt-3.5-turbo".to_string(), "openai".to_string());

        record.calculate_cost();
        service.record_telemetry(&record).await.unwrap();

        let retrieved = service.get_agent_telemetry("agent-1").unwrap();
        assert_eq!(retrieved.len(), 1);
        assert_eq!(retrieved[0].input_tokens, 100);
        assert_eq!(retrieved[0].output_tokens, 50);
    }

    #[tokio::test]
    async fn test_get_total_tokens() {
        use super::super::service::AgentRecord;

        let service = MonitoringService::new().unwrap();

        // Register agent first
        let agent = AgentRecord::new("agent-1".to_string(), "developer".to_string());
        service.register_agent(&agent).unwrap();

        let record1 = TelemetryRecord::new("agent-1".to_string()).with_tokens(100, 50);
        let record2 = TelemetryRecord::new("agent-1".to_string()).with_tokens(200, 100);

        service.record_telemetry(&record1).await.unwrap();
        service.record_telemetry(&record2).await.unwrap();

        let (input, output, total) = service.get_total_tokens("agent-1").unwrap();
        assert_eq!(input, 300);
        assert_eq!(output, 150);
        assert_eq!(total, 450);
    }

    #[tokio::test]
    async fn test_get_total_cost() {
        use super::super::service::AgentRecord;

        let service = MonitoringService::new().unwrap();

        // Register agent first
        let agent = AgentRecord::new("agent-1".to_string(), "developer".to_string());
        service.register_agent(&agent).unwrap();

        let mut record1 = TelemetryRecord::new("agent-1".to_string())
            .with_tokens(1_000_000, 1_000_000)
            .with_model("gpt-3.5-turbo".to_string(), "openai".to_string());
        record1.calculate_cost();

        service.record_telemetry(&record1).await.unwrap();

        let total_cost = service.get_total_cost("agent-1").unwrap();
        assert!((total_cost - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_telemetry_record_with_cache_stats() {
        let record = TelemetryRecord::new("agent-1".to_string()).with_cache_stats(100, 50, 25);

        assert_eq!(record.cached_tokens, 100);
        assert_eq!(record.cache_creation_tokens, 50);
        assert_eq!(record.cache_read_tokens, 25);
    }

    #[test]
    fn test_telemetry_record_with_model() {
        let record = TelemetryRecord::new("agent-1".to_string())
            .with_model("gpt-4".to_string(), "openai".to_string());

        assert_eq!(record.model, Some("gpt-4".to_string()));
        assert_eq!(record.provider, Some("openai".to_string()));
    }

    #[test]
    fn test_calculate_cost_gpt4() {
        let mut record = TelemetryRecord::new("agent-1".to_string())
            .with_tokens(1_000_000, 1_000_000)
            .with_model("gpt-4".to_string(), "openai".to_string());

        record.calculate_cost();

        // GPT-4: $30 input + $60 output = $90 per 1M tokens
        assert!((record.estimated_cost - 90.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_cost_claude_opus() {
        let mut record = TelemetryRecord::new("agent-1".to_string())
            .with_tokens(1_000_000, 1_000_000)
            .with_model("claude-3-opus".to_string(), "anthropic".to_string());

        record.calculate_cost();

        // Claude Opus: $15 input + $75 output = $90 per 1M tokens
        assert!((record.estimated_cost - 90.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_cost_claude_haiku() {
        let mut record = TelemetryRecord::new("agent-1".to_string())
            .with_tokens(1_000_000, 1_000_000)
            .with_model("claude-3-haiku".to_string(), "anthropic".to_string());

        record.calculate_cost();

        // Claude Haiku: $0.25 input + $1.25 output = $1.50 per 1M tokens
        assert!((record.estimated_cost - 1.5).abs() < 0.01);
    }

    #[test]
    fn test_calculate_cost_zero_tokens() {
        let mut record = TelemetryRecord::new("agent-1".to_string())
            .with_tokens(0, 0)
            .with_model("gpt-3.5-turbo".to_string(), "openai".to_string());

        record.calculate_cost();

        assert_eq!(record.estimated_cost, 0.0);
    }

    #[test]
    fn test_calculate_cost_unknown_model() {
        let mut record = TelemetryRecord::new("agent-1".to_string())
            .with_tokens(1_000_000, 1_000_000)
            .with_model("unknown-model".to_string(), "unknown".to_string());

        record.calculate_cost();

        // Default fallback: $1 input + $2 output = $3 per 1M tokens
        assert!((record.estimated_cost - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_openai_invalid_json() {
        let result = TelemetryParser::parse_openai("invalid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_anthropic_invalid_json() {
        let result = TelemetryParser::parse_anthropic("invalid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_gemini_invalid_json() {
        let result = TelemetryParser::parse_gemini("invalid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_telemetry_builder_pattern_chaining() {
        let mut record = TelemetryRecord::new("agent-1".to_string())
            .with_tokens(100, 50)
            .with_cache_stats(10, 5, 3)
            .with_model("gpt-3.5-turbo".to_string(), "openai".to_string());

        record.calculate_cost();

        assert_eq!(record.input_tokens, 100);
        assert_eq!(record.output_tokens, 50);
        assert_eq!(record.cached_tokens, 10);
        assert_eq!(record.model, Some("gpt-3.5-turbo".to_string()));
        assert!(record.estimated_cost > 0.0);
    }

    #[test]
    fn test_telemetry_summary_accuracy() {
        let service = MonitoringService::new().unwrap();

        // Register agents
        let agent1 = AgentRecord::new("agent-1".to_string(), "developer".to_string());
        let agent2 = AgentRecord::new("agent-2".to_string(), "architect".to_string());
        let agent3 = AgentRecord::new("agent-3".to_string(), "reviewer".to_string());
        service.register_agent(&agent1).unwrap();
        service.register_agent(&agent2).unwrap();
        service.register_agent(&agent3).unwrap();

        // Insert telemetry for agent-1: 100 tokens, $0.01
        let mut t1 = TelemetryRecord::new("agent-1".to_string())
            .with_tokens(50, 50)
            .with_model("gpt-4".to_string(), "openai".to_string());
        t1.calculate_cost();
        service.record_telemetry_sync(&t1).unwrap();

        // Insert telemetry for agent-2: 200 tokens, $0.02
        let mut t2 = TelemetryRecord::new("agent-2".to_string())
            .with_tokens(100, 100)
            .with_model("gpt-4".to_string(), "openai".to_string());
        t2.calculate_cost();
        service.record_telemetry_sync(&t2).unwrap();
        service.record_telemetry_sync(&t2).unwrap(); // Duplicate for testing

        // Insert telemetry for agent-3: 300 tokens, $0.03
        let mut t3 = TelemetryRecord::new("agent-3".to_string())
            .with_tokens(150, 150)
            .with_model("gpt-4".to_string(), "openai".to_string());
        t3.calculate_cost();
        service.record_telemetry_sync(&t3).unwrap();

        // Get summary
        let summary = service.get_telemetry_summary().unwrap();
        assert_eq!(summary.len(), 3);

        // Verify agent-1
        let s1 = summary.iter().find(|s| s.agent_id == "agent-1").unwrap();
        assert_eq!(s1.total_tokens, 100);
        assert_eq!(s1.record_count, 1);
        assert!((s1.total_cost - t1.estimated_cost).abs() < 0.0001);

        // Verify agent-2 (2 records)
        let s2 = summary.iter().find(|s| s.agent_id == "agent-2").unwrap();
        assert_eq!(s2.total_tokens, 400); // 200 * 2
        assert_eq!(s2.record_count, 2);
        assert!((s2.total_cost - (t2.estimated_cost * 2.0)).abs() < 0.0001);

        // Verify agent-3
        let s3 = summary.iter().find(|s| s.agent_id == "agent-3").unwrap();
        assert_eq!(s3.total_tokens, 300);
        assert_eq!(s3.record_count, 1);
        assert!((s3.total_cost - t3.estimated_cost).abs() < 0.0001);
    }

    #[test]
    fn test_telemetry_summary_empty() {
        let service = MonitoringService::new().unwrap();

        // No telemetry records
        let summary = service.get_telemetry_summary().unwrap();
        assert_eq!(summary.len(), 0);
    }

    #[test]
    fn test_telemetry_summary_no_telemetry_for_agents() {
        let service = MonitoringService::new().unwrap();

        // Register agents but no telemetry
        let agent1 = AgentRecord::new("agent-1".to_string(), "developer".to_string());
        service.register_agent(&agent1).unwrap();

        // Summary should be empty (no telemetry records)
        let summary = service.get_telemetry_summary().unwrap();
        assert_eq!(summary.len(), 0);
    }

    #[test]
    fn test_telemetry_record_routing_fields() {
        let record = TelemetryRecord::new("agent-1".to_string())
            .with_model_tier("smart".to_string())
            .with_routing_decision("auto".to_string())
            .with_complexity_score(75.5);

        assert_eq!(record.model_tier, Some("smart".to_string()));
        assert_eq!(record.routing_decision, Some("auto".to_string()));
        assert_eq!(record.complexity_score, Some(75.5));
    }

    #[test]
    fn test_with_local_cost() {
        use tempfile::TempDir;
        use crate::monitoring::LocalModelCostTracker;

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("engine-costs.toml");

        let toml_content = r#"
[engines.ollama]
cost_per_second = 0.0001
min_billable_duration = 0.1
"#;
        std::fs::write(&config_path, toml_content).unwrap();

        let tracker = LocalModelCostTracker::new(&config_path).unwrap();
        let record = TelemetryRecord::new("agent-1".to_string())
            .with_local_cost("ollama", std::time::Duration::from_secs(2), &tracker);

        assert!((record.estimated_cost - 0.0002).abs() < 0.000001);
        assert_eq!(record.behavior_duration_ms, Some(2000));
        assert_eq!(record.engine_id, Some("ollama".to_string()));
        assert_eq!(record.provider, Some("local".to_string()));
    }

    #[test]
    fn test_with_local_cost_missing_engine() {
        use tempfile::TempDir;
        use crate::monitoring::LocalModelCostTracker;

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("engine-costs.toml");

        let toml_content = r#"
[engines.ollama]
cost_per_second = 0.0001
min_billable_duration = 0.1
"#;
        std::fs::write(&config_path, toml_content).unwrap();

        let tracker = LocalModelCostTracker::new(&config_path).unwrap();
        let record = TelemetryRecord::new("agent-1".to_string())
            .with_local_cost("unknown-engine", std::time::Duration::from_secs(5), &tracker);

        assert_eq!(record.estimated_cost, 0.0);
        assert_eq!(record.behavior_duration_ms, Some(5000));
        assert_eq!(record.engine_id, Some("unknown-engine".to_string()));
        assert_eq!(record.provider, Some("local".to_string()));
    }

    #[test]
    fn test_with_local_cost_minimum_billable() {
        use tempfile::TempDir;
        use crate::monitoring::LocalModelCostTracker;

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("engine-costs.toml");

        let toml_content = r#"
[engines.ollama]
cost_per_second = 0.0001
min_billable_duration = 0.1
"#;
        std::fs::write(&config_path, toml_content).unwrap();

        let tracker = LocalModelCostTracker::new(&config_path).unwrap();
        let record = TelemetryRecord::new("agent-1".to_string())
            .with_local_cost("ollama", std::time::Duration::from_millis(50), &tracker);

        // Should use 0.1s minimum, so cost = 0.1 * 0.0001 = 0.00001
        assert!((record.estimated_cost - 0.00001).abs() < 0.000001);
        assert_eq!(record.behavior_duration_ms, Some(50)); // Actual duration, not minimum
        assert_eq!(record.engine_id, Some("ollama".to_string()));
    }

    #[tokio::test]
    async fn test_telemetry_record_routing_fields_persistence() {
        use super::super::service::AgentRecord;

        let service = MonitoringService::new().unwrap();

        // Register agent first
        let agent = AgentRecord::new("agent-1".to_string(), "developer".to_string());
        service.register_agent(&agent).unwrap();

        let record = TelemetryRecord::new("agent-1".to_string())
            .with_tokens(100, 50)
            .with_model("claude-sonnet".to_string(), "anthropic".to_string())
            .with_model_tier("smart".to_string())
            .with_routing_decision("auto".to_string())
            .with_complexity_score(65.0);

        service.record_telemetry(&record).await.unwrap();

        let retrieved = service.get_agent_telemetry("agent-1").unwrap();
        assert_eq!(retrieved.len(), 1);
        assert_eq!(retrieved[0].model_tier, Some("smart".to_string()));
        assert_eq!(retrieved[0].routing_decision, Some("auto".to_string()));
        assert_eq!(retrieved[0].complexity_score, Some(65.0));
    }
}
