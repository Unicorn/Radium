//! Policy rule suggestions based on learning patterns.

use crate::learning::pattern_analyzer::{Pattern, PatternAnalyzer, PolicySuggestionGenerator};
use crate::monitoring::permission_analytics::PermissionEvent;
use crate::policy::PolicyRule;
use serde::{Deserialize, Serialize};

/// Policy suggestion with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicySuggestion {
    /// Unique suggestion ID.
    pub id: String,
    /// Suggested policy rule.
    pub rule: PolicyRule,
    /// Source pattern.
    pub source_pattern: Pattern,
    /// Confidence score (0.0 to 1.0).
    pub confidence: f64,
}

/// Policy suggestion service.
pub struct PolicySuggestionService {
    analyzer: PatternAnalyzer,
    generator: PolicySuggestionGenerator,
}

impl PolicySuggestionService {
    /// Creates a new policy suggestion service.
    pub fn new(min_frequency: u64, min_confidence: f64) -> Self {
        Self {
            analyzer: PatternAnalyzer::new(min_frequency, min_confidence),
            generator: PolicySuggestionGenerator,
        }
    }

    /// Analyzes events and generates suggestions.
    pub fn analyze_and_suggest(&self, events: &[PermissionEvent]) -> Vec<PolicySuggestion> {
        let patterns = self.analyzer.analyze_patterns(events);
        let rules = self.generator.generate_suggestions(&patterns);

        patterns
            .into_iter()
            .zip(rules.into_iter())
            .enumerate()
            .map(|(idx, (pattern, rule))| {
                let confidence = pattern.confidence;
                PolicySuggestion {
                    id: format!("suggestion-{}", idx + 1),
                    rule,
                    source_pattern: pattern,
                    confidence,
                }
            })
            .collect()
    }

    /// Gets suggestions from stored events.
    pub fn get_suggestions(&self, events: &[PermissionEvent]) -> Vec<PolicySuggestion> {
        self.analyze_and_suggest(events)
    }
}

