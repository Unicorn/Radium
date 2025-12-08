//! Pattern analyzer for learning safe tool execution patterns.

use crate::monitoring::permission_analytics::{PermissionEvent, PermissionOutcome};
use crate::policy::{PolicyAction, PolicyPriority, PolicyRule};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Pattern detected from approval history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    /// Tool pattern (glob pattern).
    pub tool_pattern: String,
    /// Optional argument pattern (glob pattern).
    pub arg_pattern: Option<String>,
    /// Number of times this pattern was approved.
    pub frequency: u64,
    /// Confidence score (0.0 to 1.0).
    pub confidence: f64,
    /// Sample events that match this pattern.
    pub sample_events: Vec<PatternEvent>,
}

/// Sample event for a pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternEvent {
    /// Tool name.
    pub tool_name: String,
    /// Arguments.
    pub args: Vec<String>,
    /// Timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Pattern analyzer for detecting safe execution patterns.
pub struct PatternAnalyzer {
    /// Minimum frequency threshold for pattern detection.
    min_frequency: u64,
    /// Minimum confidence threshold for suggestions.
    min_confidence: f64,
}

impl PatternAnalyzer {
    /// Creates a new pattern analyzer.
    pub fn new(min_frequency: u64, min_confidence: f64) -> Self {
        Self {
            min_frequency,
            min_confidence,
        }
    }

    /// Analyzes events to detect patterns.
    pub fn analyze_patterns(&self, events: &[PermissionEvent]) -> Vec<Pattern> {
        // Filter to only manually approved events (asked -> allowed)
        let approved_events: Vec<&PermissionEvent> = events
            .iter()
            .filter(|e| {
                // Only consider events that were asked and then allowed
                // In practice, we'd track the full approval flow
                // For now, we'll consider all "allowed" events as approved
                e.outcome == PermissionOutcome::Allowed
            })
            .collect();

        if approved_events.len() < self.min_frequency as usize {
            return Vec::new();
        }

        // Group by tool name pattern
        let mut tool_patterns: HashMap<String, Vec<&PermissionEvent>> = HashMap::new();
        
        for event in &approved_events {
            // Generate pattern from tool name
            let pattern = self.generate_tool_pattern(&event.tool_name);
            tool_patterns.entry(pattern).or_insert_with(Vec::new).push(event);
        }

        // Convert to Pattern structs
        let mut patterns = Vec::new();
        for (tool_pattern, events) in tool_patterns {
            if events.len() >= self.min_frequency as usize {
                // Calculate confidence based on frequency and consistency
                let confidence = self.calculate_confidence(events.len(), approved_events.len());
                
                if confidence >= self.min_confidence {
                    // Extract arg pattern if consistent
                    let arg_pattern = self.extract_arg_pattern(events);
                    
                    // Get sample events
                    let sample_events: Vec<PatternEvent> = events
                        .iter()
                        .take(5)
                        .map(|e| PatternEvent {
                            tool_name: e.tool_name.clone(),
                            args: e.args.clone(),
                            timestamp: e.timestamp,
                        })
                        .collect();

                    patterns.push(Pattern {
                        tool_pattern,
                        arg_pattern,
                        frequency: events.len() as u64,
                        confidence,
                        sample_events,
                    });
                }
            }
        }

        // Sort by confidence descending
        patterns.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        patterns
    }

    /// Generates a tool pattern from a tool name.
    fn generate_tool_pattern(&self, tool_name: &str) -> String {
        // Simple pattern generation: convert specific names to glob patterns
        if tool_name.starts_with("read_") {
            "read_*".to_string()
        } else if tool_name.starts_with("write_") {
            "write_*".to_string()
        } else if tool_name.starts_with("mcp_") {
            "mcp_*".to_string()
        } else {
            // For specific tools, use exact match
            tool_name.to_string()
        }
    }

    /// Extracts a consistent argument pattern from events.
    fn extract_arg_pattern(&self, events: &[&PermissionEvent]) -> Option<String> {
        if events.is_empty() {
            return None;
        }

        // Check if all events have the same first argument
        let first_args: Vec<Option<&String>> = events
            .iter()
            .map(|e| e.args.first())
            .collect();

        if first_args.len() >= self.min_frequency as usize {
            if let Some(first_arg) = first_args[0] {
                // Check if all first args are the same
                if first_args.iter().all(|arg| arg == &Some(first_arg)) {
                    return Some(format!("{} *", first_arg));
                }
            }
        }

        None
    }

    /// Calculates confidence score.
    fn calculate_confidence(&self, pattern_count: usize, total_approved: usize) -> f64 {
        if total_approved == 0 {
            return 0.0;
        }

        // Base confidence from frequency
        let frequency_ratio = pattern_count as f64 / total_approved as f64;
        
        // Boost confidence for higher absolute frequency
        let frequency_boost = (pattern_count as f64 / 10.0).min(1.0);
        
        // Combined confidence
        (frequency_ratio * 0.7 + frequency_boost * 0.3).min(1.0)
    }
}

/// Policy suggestion generator.
pub struct PolicySuggestionGenerator;

impl PolicySuggestionGenerator {
    /// Generates a policy rule suggestion from a pattern.
    pub fn generate_suggestion(&self, pattern: &Pattern) -> PolicyRule {
        PolicyRule::new(
            format!("auto-allow-{}", pattern.tool_pattern.replace("*", "all")),
            pattern.tool_pattern.clone(),
            PolicyAction::Allow,
        )
        .with_priority(PolicyPriority::User)
        .with_arg_pattern_opt(pattern.arg_pattern.clone())
        .with_reason(format!(
            "Auto-generated from {} approved executions (confidence: {:.1}%)",
            pattern.frequency,
            pattern.confidence * 100.0
        ))
    }

    /// Generates suggestions from multiple patterns.
    pub fn generate_suggestions(&self, patterns: &[Pattern]) -> Vec<PolicyRule> {
        patterns
            .iter()
            .map(|p| self.generate_suggestion(p))
            .collect()
    }
}

// Helper trait for PolicyRule
trait PolicyRuleExt {
    fn with_arg_pattern_opt(self, pattern: Option<String>) -> Self;
}

impl PolicyRuleExt for PolicyRule {
    fn with_arg_pattern_opt(mut self, pattern: Option<String>) -> Self {
        if let Some(p) = pattern {
            self.arg_pattern = Some(p);
        }
        self
    }
}

