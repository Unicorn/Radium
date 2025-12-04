//! Telemetry parsing and tracking for token usage and costs.

use super::error::{MonitoringError, Result};
use super::service::MonitoringService;
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
        }
    }

    /// Sets token counts.
    pub fn with_tokens(mut self, input: u64, output: u64) -> Self {
        self.input_tokens = input;
        self.output_tokens = output;
        self.total_tokens = input + output;
        self
    }

    /// Sets cache statistics.
    pub fn with_cache_stats(mut self, cached: u64, creation: u64, read: u64) -> Self {
        self.cached_tokens = cached;
        self.cache_creation_tokens = creation;
        self.cache_read_tokens = read;
        self
    }

    /// Sets model information.
    pub fn with_model(mut self, model: String, provider: String) -> Self {
        self.model = Some(model);
        self.provider = Some(provider);
        self
    }

    /// Calculates estimated cost based on model pricing.
    pub fn calculate_cost(&mut self) -> &mut Self {
        // Pricing per 1M tokens (approximate, as of 2024)
        let (input_price, output_price) = match self.model.as_deref() {
            Some("gpt-4") => (30.0, 60.0),
            Some("gpt-3.5-turbo") => (0.5, 1.5),
            Some("claude-3-opus") => (15.0, 75.0),
            Some("claude-3-sonnet") => (3.0, 15.0),
            Some("claude-3-haiku") => (0.25, 1.25),
            Some("gemini-pro") => (0.5, 1.5),
            _ => (1.0, 2.0), // Default fallback
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

/// Extension trait for MonitoringService to add telemetry tracking.
pub trait TelemetryTracking {
    /// Records telemetry for an agent.
    fn record_telemetry(&self, record: &TelemetryRecord) -> Result<()>;

    /// Gets telemetry records for an agent.
    fn get_agent_telemetry(&self, agent_id: &str) -> Result<Vec<TelemetryRecord>>;

    /// Gets total token usage for an agent.
    fn get_total_tokens(&self, agent_id: &str) -> Result<(u64, u64, u64)>;

    /// Gets total estimated cost for an agent.
    fn get_total_cost(&self, agent_id: &str) -> Result<f64>;
}

impl TelemetryTracking for MonitoringService {
    fn record_telemetry(&self, record: &TelemetryRecord) -> Result<()> {
        self.conn.execute(
            "INSERT INTO telemetry (agent_id, timestamp, input_tokens, output_tokens, cached_tokens,
                                    cache_creation_tokens, cache_read_tokens, total_tokens,
                                    estimated_cost, model, provider)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                record.agent_id,
                record.timestamp,
                record.input_tokens,
                record.output_tokens,
                record.cached_tokens,
                record.cache_creation_tokens,
                record.cache_read_tokens,
                record.total_tokens,
                record.estimated_cost,
                record.model,
                record.provider,
            ],
        )?;
        Ok(())
    }

    fn get_agent_telemetry(&self, agent_id: &str) -> Result<Vec<TelemetryRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT agent_id, timestamp, input_tokens, output_tokens, cached_tokens,
                    cache_creation_tokens, cache_read_tokens, total_tokens,
                    estimated_cost, model, provider
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
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_record_telemetry() {
        use super::super::service::AgentRecord;

        let service = MonitoringService::new().unwrap();

        // Register agent first (foreign key constraint)
        let agent = AgentRecord::new("agent-1".to_string(), "developer".to_string());
        service.register_agent(&agent).unwrap();

        let mut record = TelemetryRecord::new("agent-1".to_string())
            .with_tokens(100, 50)
            .with_model("gpt-3.5-turbo".to_string(), "openai".to_string());

        record.calculate_cost();
        service.record_telemetry(&record).unwrap();

        let retrieved = service.get_agent_telemetry("agent-1").unwrap();
        assert_eq!(retrieved.len(), 1);
        assert_eq!(retrieved[0].input_tokens, 100);
        assert_eq!(retrieved[0].output_tokens, 50);
    }

    #[test]
    fn test_get_total_tokens() {
        use super::super::service::AgentRecord;

        let service = MonitoringService::new().unwrap();

        // Register agent first
        let agent = AgentRecord::new("agent-1".to_string(), "developer".to_string());
        service.register_agent(&agent).unwrap();

        let record1 = TelemetryRecord::new("agent-1".to_string()).with_tokens(100, 50);
        let record2 = TelemetryRecord::new("agent-1".to_string()).with_tokens(200, 100);

        service.record_telemetry(&record1).unwrap();
        service.record_telemetry(&record2).unwrap();

        let (input, output, total) = service.get_total_tokens("agent-1").unwrap();
        assert_eq!(input, 300);
        assert_eq!(output, 150);
        assert_eq!(total, 450);
    }

    #[test]
    fn test_get_total_cost() {
        use super::super::service::AgentRecord;

        let service = MonitoringService::new().unwrap();

        // Register agent first
        let agent = AgentRecord::new("agent-1".to_string(), "developer".to_string());
        service.register_agent(&agent).unwrap();

        let mut record1 = TelemetryRecord::new("agent-1".to_string())
            .with_tokens(1_000_000, 1_000_000)
            .with_model("gpt-3.5-turbo".to_string(), "openai".to_string());
        record1.calculate_cost();

        service.record_telemetry(&record1).unwrap();

        let total_cost = service.get_total_cost("agent-1").unwrap();
        assert!((total_cost - 2.0).abs() < 0.01);
    }
}
