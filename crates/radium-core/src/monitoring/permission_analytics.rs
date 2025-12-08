//! Permission analytics for tracking tool usage and policy decisions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Permission event record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionEvent {
    /// Timestamp of the event.
    pub timestamp: DateTime<Utc>,
    /// Tool name that was executed.
    pub tool_name: String,
    /// Tool arguments.
    pub args: Vec<String>,
    /// Agent ID that requested the tool.
    pub agent_id: Option<String>,
    /// Policy decision outcome.
    pub outcome: PermissionOutcome,
    /// Matched policy rule (if any).
    pub matched_rule: Option<String>,
    /// Reason for the decision.
    pub reason: Option<String>,
}

/// Permission outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PermissionOutcome {
    /// Tool execution was allowed.
    Allowed,
    /// Tool execution was denied.
    Denied,
    /// User approval was requested.
    Asked,
}

impl PermissionOutcome {
    /// Creates outcome from policy action string.
    pub fn from_action(action: &str) -> Self {
        match action {
            "allow" => Self::Allowed,
            "deny" => Self::Denied,
            "ask_user" | "askuser" => Self::Asked,
            _ => Self::Asked, // Default to asked for unknown actions
        }
    }
}

/// Permission analytics aggregator.
pub struct PermissionAnalytics {
    /// All events.
    events: Vec<PermissionEvent>,
    /// Maximum number of events to keep in memory.
    max_events: usize,
}

impl PermissionAnalytics {
    /// Creates a new permission analytics instance.
    pub fn new(max_events: usize) -> Self {
        Self {
            events: Vec::new(),
            max_events,
        }
    }

    /// Records a permission event.
    pub fn record_event(&mut self, event: PermissionEvent) {
        self.events.push(event);
        
        // Trim if we exceed max events
        if self.events.len() > self.max_events {
            self.events.remove(0);
        }
    }

    /// Gets all events.
    pub fn events(&self) -> &[PermissionEvent] {
        &self.events
    }

    /// Gets events filtered by date range.
    pub fn events_in_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<&PermissionEvent> {
        self.events
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .collect()
    }

    /// Gets tool usage statistics.
    pub fn tool_usage_stats(&self) -> HashMap<String, ToolUsageStats> {
        let mut stats: HashMap<String, ToolUsageStats> = HashMap::new();
        
        for event in &self.events {
            let tool_stats = stats.entry(event.tool_name.clone()).or_insert_with(|| {
                ToolUsageStats {
                    tool_name: event.tool_name.clone(),
                    total: 0,
                    allowed: 0,
                    denied: 0,
                    asked: 0,
                }
            });
            
            tool_stats.total += 1;
            match event.outcome {
                PermissionOutcome::Allowed => tool_stats.allowed += 1,
                PermissionOutcome::Denied => tool_stats.denied += 1,
                PermissionOutcome::Asked => tool_stats.asked += 1,
            }
        }
        
        stats
    }

    /// Gets agent usage statistics.
    pub fn agent_usage_stats(&self) -> HashMap<String, AgentUsageStats> {
        let mut stats: HashMap<String, AgentUsageStats> = HashMap::new();
        
        for event in &self.events {
            let agent_id = event.agent_id.as_ref().map(|s| s.as_str()).unwrap_or("unknown");
            let agent_stats = stats.entry(agent_id.to_string()).or_insert_with(|| {
                AgentUsageStats {
                    agent_id: agent_id.to_string(),
                    total: 0,
                    allowed: 0,
                    denied: 0,
                    asked: 0,
                }
            });
            
            agent_stats.total += 1;
            match event.outcome {
                PermissionOutcome::Allowed => agent_stats.allowed += 1,
                PermissionOutcome::Denied => agent_stats.denied += 1,
                PermissionOutcome::Asked => agent_stats.asked += 1,
            }
        }
        
        stats
    }

    /// Gets policy rule effectiveness statistics.
    pub fn rule_effectiveness_stats(&self) -> HashMap<String, RuleEffectivenessStats> {
        let mut stats: HashMap<String, RuleEffectivenessStats> = HashMap::new();
        
        for event in &self.events {
            if let Some(ref rule_name) = event.matched_rule {
                let rule_stats = stats.entry(rule_name.clone()).or_insert_with(|| {
                    RuleEffectivenessStats {
                        rule_name: rule_name.clone(),
                        trigger_count: 0,
                        allowed_count: 0,
                        denied_count: 0,
                        asked_count: 0,
                    }
                });
                
                rule_stats.trigger_count += 1;
                match event.outcome {
                    PermissionOutcome::Allowed => rule_stats.allowed_count += 1,
                    PermissionOutcome::Denied => rule_stats.denied_count += 1,
                    PermissionOutcome::Asked => rule_stats.asked_count += 1,
                }
            }
        }
        
        stats
    }

    /// Gets time-series data for trends.
    pub fn time_series_data(&self, period_hours: i64) -> Vec<TimeSeriesPoint> {
        let mut points: HashMap<i64, TimeSeriesPoint> = HashMap::new();
        let now = Utc::now();
        let cutoff = now - chrono::Duration::hours(period_hours);
        
        for event in &self.events {
            if event.timestamp < cutoff {
                continue;
            }
            
            // Round to hour
            let hour_key = event.timestamp.timestamp() / 3600;
            let point = points.entry(hour_key).or_insert_with(|| {
                TimeSeriesPoint {
                    timestamp: DateTime::from_timestamp(hour_key * 3600, 0).unwrap_or(now),
                    allowed: 0,
                    denied: 0,
                    asked: 0,
                }
            });
            
            match event.outcome {
                PermissionOutcome::Allowed => point.allowed += 1,
                PermissionOutcome::Denied => point.denied += 1,
                PermissionOutcome::Asked => point.asked += 1,
            }
        }
        
        let mut result: Vec<TimeSeriesPoint> = points.into_values().collect();
        result.sort_by_key(|p| p.timestamp);
        result
    }

    /// Detects anomalies in permission patterns.
    pub fn detect_anomalies(&self) -> Vec<Anomaly> {
        let mut anomalies = Vec::new();
        
        // Check for sudden spike in denials
        let recent_events: Vec<&PermissionEvent> = self.events
            .iter()
            .filter(|e| e.timestamp > Utc::now() - chrono::Duration::hours(1))
            .collect();
        
        let recent_denials = recent_events.iter()
            .filter(|e| e.outcome == PermissionOutcome::Denied)
            .count();
        
        if recent_denials > 10 {
            anomalies.push(Anomaly {
                severity: AnomalySeverity::High,
                category: AnomalyCategory::SpikeInDenials,
                message: format!("Unusual spike in permission denials: {} denials in the last hour", recent_denials),
                timestamp: Utc::now(),
            });
        }
        
        // Check for tools frequently denied
        let tool_stats = self.tool_usage_stats();
        for (tool_name, stats) in tool_stats {
            if stats.total > 5 && stats.denied as f64 / stats.total as f64 > 0.8 {
                anomalies.push(Anomaly {
                    severity: AnomalySeverity::Medium,
                    category: AnomalyCategory::FrequentlyDeniedTool,
                    message: format!("Tool '{}' is frequently denied ({}% denial rate)", tool_name, (stats.denied as f64 / stats.total as f64 * 100.0) as u32),
                    timestamp: Utc::now(),
                });
            }
        }
        
        anomalies
    }
}

/// Tool usage statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUsageStats {
    pub tool_name: String,
    pub total: u64,
    pub allowed: u64,
    pub denied: u64,
    pub asked: u64,
}

/// Agent usage statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentUsageStats {
    pub agent_id: String,
    pub total: u64,
    pub allowed: u64,
    pub denied: u64,
    pub asked: u64,
}

/// Rule effectiveness statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleEffectivenessStats {
    pub rule_name: String,
    pub trigger_count: u64,
    pub allowed_count: u64,
    pub denied_count: u64,
    pub asked_count: u64,
}

/// Time series data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub timestamp: DateTime<Utc>,
    pub allowed: u64,
    pub denied: u64,
    pub asked: u64,
}

/// Anomaly severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
}

/// Anomaly category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnomalyCategory {
    SpikeInDenials,
    FrequentlyDeniedTool,
    UnusualPattern,
}

/// Anomaly detection result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    pub severity: AnomalySeverity,
    pub category: AnomalyCategory,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

