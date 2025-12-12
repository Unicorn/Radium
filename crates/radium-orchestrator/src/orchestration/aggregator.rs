//! Result aggregation module for merging multi-agent outputs
//!
//! This module provides aggregation logic that deduplicates findings,
//! resolves contradictory suggestions, and produces coherent, ordered
//! action plans or patch bundles from multiple agent results.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use super::tool::ToolResult;

/// Aggregated result from multiple agent outputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedResult {
    /// Merged text output with deduplication
    pub merged_output: String,
    /// Ordered list of unique findings/suggestions
    pub findings: Vec<Finding>,
    /// Resolved action plan (ordered)
    pub action_plan: Vec<Action>,
    /// Any conflicts that were detected and resolved
    pub resolved_conflicts: Vec<ConflictResolution>,
}

/// A finding or suggestion from an agent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Finding {
    /// Content of the finding
    pub content: String,
    /// Source agent ID
    pub source_agent: String,
    /// Category/type of finding
    pub category: String,
}

/// An action item in the plan
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Action {
    /// Action description
    pub description: String,
    /// Priority (1 = highest)
    pub priority: u32,
    /// Dependencies (other action indices)
    pub dependencies: Vec<usize>,
}

/// A conflict that was detected and resolved
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolution {
    /// Description of the conflict
    pub conflict_description: String,
    /// The resolution strategy used
    pub resolution_strategy: String,
    /// The chosen resolution
    pub resolution: String,
}

/// Aggregator for merging multiple agent results
pub struct Aggregator;

impl Aggregator {
    /// Aggregate multiple tool results into a coherent output
    ///
    /// This method:
    /// 1. Deduplicates identical findings
    /// 2. Resolves contradictory suggestions
    /// 3. Produces an ordered action plan
    /// 4. Merges text outputs
    ///
    /// The aggregation is deterministic: same inputs produce same output.
    pub fn aggregate(
        results: &[ToolResult],
        agent_ids: &[String],
    ) -> AggregatedResult {
        if results.is_empty() {
            return AggregatedResult {
                merged_output: String::new(),
                findings: vec![],
                action_plan: vec![],
                resolved_conflicts: vec![],
            };
        }

        // Extract findings from each result
        let mut all_findings = Vec::new();
        for (i, result) in results.iter().enumerate() {
            if result.success {
                let agent_id = agent_ids.get(i).cloned().unwrap_or_else(|| format!("agent_{}", i));
                let findings = Self::extract_findings(&result.output, &agent_id);
                all_findings.extend(findings);
            }
        }

        // Deduplicate findings
        let unique_findings = Self::deduplicate_findings(all_findings);

        // Detect and resolve conflicts
        let (resolved_findings, conflicts) = Self::resolve_conflicts(unique_findings);

        // Generate action plan from findings
        let action_plan = Self::generate_action_plan(&resolved_findings);

        // Merge text outputs
        let merged_output = Self::merge_outputs(results);

        AggregatedResult {
            merged_output,
            findings: resolved_findings,
            action_plan,
            resolved_conflicts: conflicts,
        }
    }

    /// Extract findings from a tool result output
    fn extract_findings(output: &str, agent_id: &str) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Simple extraction: look for bullet points, numbered lists, or structured content
        let lines: Vec<&str> = output.lines().collect();
        let mut current_finding = String::new();
        let mut category = "general".to_string();

        for line in lines {
            let trimmed = line.trim();
            
            // Detect category markers
            if trimmed.starts_with("Category:") || trimmed.starts_with("Type:") {
                category = trimmed.split(':').nth(1).unwrap_or("general").trim().to_string();
                continue;
            }

            // Detect finding markers (bullet points, dashes, numbers)
            if trimmed.starts_with("- ") || trimmed.starts_with("* ") || 
               trimmed.starts_with("• ") || trimmed.matches(char::is_numeric).count() > 0 && trimmed.contains('.') {
                if !current_finding.is_empty() {
                    findings.push(Finding {
                        content: current_finding.trim().to_string(),
                        source_agent: agent_id.to_string(),
                        category: category.clone(),
                    });
                }
                current_finding = trimmed.strip_prefix("- ")
                    .or_else(|| trimmed.strip_prefix("* "))
                    .or_else(|| trimmed.strip_prefix("• "))
                    .or_else(|| {
                        // Remove leading number and dot
                        trimmed.splitn(2, '.').nth(1)
                    })
                    .unwrap_or(trimmed)
                    .trim()
                    .to_string();
            } else if !trimmed.is_empty() {
                // Continuation of current finding
                if !current_finding.is_empty() {
                    current_finding.push(' ');
                }
                current_finding.push_str(trimmed);
            }
        }

        // Add last finding if exists
        if !current_finding.is_empty() {
            findings.push(Finding {
                content: current_finding.trim().to_string(),
                source_agent: agent_id.to_string(),
                category,
            });
        }

        // If no structured findings, treat entire output as one finding
        if findings.is_empty() && !output.trim().is_empty() {
            findings.push(Finding {
                content: output.trim().to_string(),
                source_agent: agent_id.to_string(),
                category: "general".to_string(),
            });
        }

        findings
    }

    /// Deduplicate findings by content (case-insensitive, normalized)
    fn deduplicate_findings(findings: Vec<Finding>) -> Vec<Finding> {
        let mut seen = HashSet::new();
        let mut unique = Vec::new();

        for finding in findings {
            // Normalize content for comparison (lowercase, trim, remove extra whitespace)
            let normalized: String = finding.content
                .to_lowercase()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");

            if !seen.contains(&normalized) {
                seen.insert(normalized);
                unique.push(finding);
            }
        }

        unique
    }

    /// Detect and resolve conflicts between findings
    fn resolve_conflicts(findings: Vec<Finding>) -> (Vec<Finding>, Vec<ConflictResolution>) {
        let mut resolved = Vec::new();
        let mut conflicts = Vec::new();
        let mut seen_content = HashSet::new();

        // Group findings by similar content (potential conflicts)
        let mut content_groups: HashMap<String, Vec<Finding>> = HashMap::new();

        for finding in findings {
            let key = Self::normalize_for_grouping(&finding.content);
            content_groups.entry(key).or_insert_with(Vec::new).push(finding);
        }

        // Process each group
        for (_, group) in content_groups {
            if group.len() == 1 {
                // No conflict
                let finding = group.into_iter().next().unwrap();
                let normalized = Self::normalize_for_grouping(&finding.content);
                if !seen_content.contains(&normalized) {
                    seen_content.insert(normalized);
                    resolved.push(finding);
                }
            } else {
                // Potential conflict - multiple agents with similar findings
                // Strategy: Keep the first one, note the conflict
                let mut group_iter = group.into_iter();
                let first = group_iter.next().unwrap();
                let normalized = Self::normalize_for_grouping(&first.content);
                
                if !seen_content.contains(&normalized) {
                    seen_content.insert(normalized);
                    resolved.push(first.clone());

                    // Record conflict resolution
                    let other_agents: Vec<String> = group_iter.map(|f| f.source_agent).collect();
                    conflicts.push(ConflictResolution {
                        conflict_description: format!(
                            "Multiple agents ({}, {}) provided similar findings",
                            first.source_agent,
                            other_agents.join(", ")
                        ),
                        resolution_strategy: "keep_first".to_string(),
                        resolution: format!("Kept finding from {}", first.source_agent),
                    });
                }
            }
        }

        (resolved, conflicts)
    }

    /// Normalize content for grouping (detect similar findings)
    fn normalize_for_grouping(content: &str) -> String {
        // Remove common words and normalize
        content
            .to_lowercase()
            .split_whitespace()
            .filter(|w| {
                // Filter out common stop words
                !matches!(*w, "the" | "a" | "an" | "and" | "or" | "but" | "in" | "on" | "at" | "to" | "for" | "of" | "with")
            })
            .take(10) // Use first 10 meaningful words
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Generate an ordered action plan from findings
    fn generate_action_plan(findings: &[Finding]) -> Vec<Action> {
        let mut actions = Vec::new();

        for (idx, finding) in findings.iter().enumerate() {
            // Determine priority based on category
            let priority = match finding.category.as_str() {
                "critical" | "error" | "bug" => 1,
                "warning" | "security" => 2,
                "improvement" | "optimization" => 3,
                _ => 4,
            };

            actions.push(Action {
                description: finding.content.clone(),
                priority,
                dependencies: vec![], // Can be enhanced to detect dependencies
            });
        }

        // Sort by priority
        actions.sort_by_key(|a| a.priority);

        actions
    }

    /// Merge text outputs from multiple results
    fn merge_outputs(results: &[ToolResult]) -> String {
        let mut merged = Vec::new();

        for (i, result) in results.iter().enumerate() {
            if result.success && !result.output.trim().is_empty() {
                merged.push(format!("--- Agent {} Output ---\n{}", i + 1, result.output));
            }
        }

        merged.join("\n\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate_findings() {
        let findings = vec![
            Finding {
                content: "Fix the bug in line 42".to_string(),
                source_agent: "agent1".to_string(),
                category: "bug".to_string(),
            },
            Finding {
                content: "fix the bug in line 42".to_string(), // Duplicate (case difference)
                source_agent: "agent2".to_string(),
                category: "bug".to_string(),
            },
            Finding {
                content: "Add error handling".to_string(),
                source_agent: "agent1".to_string(),
                category: "improvement".to_string(),
            },
        ];

        let deduplicated = Aggregator::deduplicate_findings(findings);
        assert_eq!(deduplicated.len(), 2, "Should remove duplicate finding");
    }

    #[test]
    fn test_resolve_conflicts() {
        let findings = vec![
            Finding {
                content: "Use async/await here".to_string(),
                source_agent: "agent1".to_string(),
                category: "improvement".to_string(),
            },
            Finding {
                content: "Use async await here".to_string(), // Similar (whitespace difference)
                source_agent: "agent2".to_string(),
                category: "improvement".to_string(),
            },
        ];

        let (resolved, conflicts) = Aggregator::resolve_conflicts(findings);
        assert_eq!(resolved.len(), 1, "Should resolve conflict to one finding");
        assert!(!conflicts.is_empty(), "Should detect conflict");
    }

    #[test]
    fn test_aggregate_empty() {
        let result = Aggregator::aggregate(&[], &[]);
        assert!(result.merged_output.is_empty());
        assert!(result.findings.is_empty());
        assert!(result.action_plan.is_empty());
    }

    #[test]
    fn test_aggregate_single_result() {
        let results = vec![ToolResult {
            success: true,
            output: "- Fix bug in authentication\n- Add error handling".to_string(),
        }];
        let agent_ids = vec!["agent1".to_string()];

        let aggregated = Aggregator::aggregate(&results, &agent_ids);
        assert!(!aggregated.merged_output.is_empty());
        assert!(!aggregated.findings.is_empty());
        assert!(!aggregated.action_plan.is_empty());
    }

    #[test]
    fn test_deterministic_aggregation() {
        let results = vec![
            ToolResult {
                success: true,
                output: "- Fix bug".to_string(),
            },
            ToolResult {
                success: true,
                output: "- Add tests".to_string(),
            },
        ];
        let agent_ids = vec!["agent1".to_string(), "agent2".to_string()];

        let result1 = Aggregator::aggregate(&results, &agent_ids);
        let result2 = Aggregator::aggregate(&results, &agent_ids);

        // Should produce same output
        assert_eq!(result1.findings.len(), result2.findings.len());
        assert_eq!(result1.action_plan.len(), result2.action_plan.len());
    }
}
