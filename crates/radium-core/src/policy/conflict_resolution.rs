//! Policy rule conflict detection and resolution.

use super::rules::PolicyRule;
use super::types::{PolicyError, PolicyResult};
use glob::Pattern;
use std::collections::HashSet;

/// Represents a conflict between two policy rules.
#[derive(Debug, Clone)]
pub struct PolicyConflict {
    /// First rule involved in the conflict.
    pub rule1: PolicyRule,
    /// Second rule involved in the conflict.
    pub rule2: PolicyRule,
    /// Type of conflict detected.
    pub conflict_type: ConflictType,
    /// Example tool name that would trigger both rules.
    pub example_tool: String,
    /// Example arguments that would trigger both rules (if applicable).
    pub example_args: Vec<String>,
}

/// Types of conflicts that can occur between policy rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictType {
    /// Both rules match the same tool but have conflicting actions.
    /// Example: One rule allows "read_*", another denies "read_*".
    ConflictingActions,
    /// Rules have overlapping patterns where one is more specific.
    /// Example: "read_*" and "read_file" both match "read_file".
    OverlappingPatterns,
    /// Rules have different priorities but same pattern and conflicting actions.
    /// Example: Admin rule allows "bash:*", User rule denies "bash:*".
    PriorityConflict,
    /// Rules have identical patterns and priorities but different actions.
    /// Example: Two User rules with "read_*" pattern, one allows, one denies.
    DuplicatePattern,
}

impl ConflictType {
    /// Returns a human-readable description of the conflict type.
    pub fn description(&self) -> &'static str {
        match self {
            ConflictType::ConflictingActions => {
                "Both rules match the same tool but have conflicting actions"
            }
            ConflictType::OverlappingPatterns => {
                "Rules have overlapping patterns where one is more specific"
            }
            ConflictType::PriorityConflict => {
                "Rules have different priorities but same pattern and conflicting actions"
            }
            ConflictType::DuplicatePattern => {
                "Rules have identical patterns and priorities but different actions"
            }
        }
    }
}

/// Detects conflicts between policy rules.
pub struct ConflictDetector;

impl ConflictDetector {
    /// Detects all conflicts in a list of policy rules.
    ///
    /// # Arguments
    /// * `rules` - List of policy rules to check for conflicts
    ///
    /// # Returns
    /// Vector of detected conflicts.
    ///
    /// # Errors
    /// Returns error if pattern parsing fails.
    pub fn detect_conflicts(rules: &[PolicyRule]) -> PolicyResult<Vec<PolicyConflict>> {
        let mut conflicts = Vec::new();

        // Check all pairs of rules
        for i in 0..rules.len() {
            for j in (i + 1)..rules.len() {
                if let Some(conflict) = Self::detect_pair_conflict(&rules[i], &rules[j])? {
                    conflicts.push(conflict);
                }
            }
        }

        Ok(conflicts)
    }

    /// Detects conflicts between two specific rules.
    ///
    /// # Arguments
    /// * `rule1` - First rule
    /// * `rule2` - Second rule
    ///
    /// # Returns
    /// `Some(Conflict)` if a conflict is detected, `None` otherwise.
    ///
    /// # Errors
    /// Returns error if pattern parsing fails.
    fn detect_pair_conflict(
        rule1: &PolicyRule,
        rule2: &PolicyRule,
    ) -> PolicyResult<Option<PolicyConflict>> {
        // Parse patterns
        let pattern1 = Pattern::new(&rule1.tool_pattern)
            .map_err(|e| PolicyError::PatternError(format!("Invalid pattern in rule '{}': {}", rule1.name, e)))?;
        let pattern2 = Pattern::new(&rule2.tool_pattern)
            .map_err(|e| PolicyError::PatternError(format!("Invalid pattern in rule '{}': {}", rule2.name, e)))?;

        // Check for exact pattern match
        if rule1.tool_pattern == rule2.tool_pattern {
            // Same pattern - check for conflicts
            if rule1.action != rule2.action {
                // Different actions with same pattern
                if rule1.priority == rule2.priority {
                    return Ok(Some(PolicyConflict {
                        rule1: rule1.clone(),
                        rule2: rule2.clone(),
                        conflict_type: ConflictType::DuplicatePattern,
                        example_tool: Self::find_example_match(&pattern1)?,
                        example_args: Vec::new(),
                    }));
                } else {
                    // Different priorities - priority conflict
                    return Ok(Some(PolicyConflict {
                        rule1: rule1.clone(),
                        rule2: rule2.clone(),
                        conflict_type: ConflictType::PriorityConflict,
                        example_tool: Self::find_example_match(&pattern1)?,
                        example_args: Vec::new(),
                    }));
                }
            }
            // Same pattern and same action - no conflict
            return Ok(None);
        }

        // Check for overlapping patterns
        let overlap = Self::find_pattern_overlap(&pattern1, &pattern2)?;
        
        if let Some((example_tool, example_args)) = overlap {
            // Patterns overlap - check if actions conflict
            if rule1.action != rule2.action {
                // Check if one pattern is more specific than the other
                let rule1_specific = Self::is_more_specific(&rule1.tool_pattern, &rule2.tool_pattern);
                let rule2_specific = Self::is_more_specific(&rule2.tool_pattern, &rule1.tool_pattern);

                if rule1_specific || rule2_specific {
                    // One pattern is more specific - overlapping patterns conflict
                    return Ok(Some(PolicyConflict {
                        rule1: rule1.clone(),
                        rule2: rule2.clone(),
                        conflict_type: ConflictType::OverlappingPatterns,
                        example_tool,
                        example_args,
                    }));
                } else {
                    // Patterns overlap but neither is clearly more specific - conflicting actions
                    return Ok(Some(PolicyConflict {
                        rule1: rule1.clone(),
                        rule2: rule2.clone(),
                        conflict_type: ConflictType::ConflictingActions,
                        example_tool,
                        example_args,
                    }));
                }
            }
            // Patterns overlap but same action - not a conflict (one will win based on priority)
            return Ok(None);
        }

        // No conflict detected
        Ok(None)
    }

    /// Finds an example tool name that matches a pattern.
    fn find_example_match(pattern: &Pattern) -> PolicyResult<String> {
        // Try common tool name patterns
        let examples = vec![
            "read_file",
            "write_file",
            "bash:sh",
            "bash:command",
            "mcp_server_tool",
            "read_config",
            "delete_file",
            "update_file",
        ];

        for example in examples {
            if pattern.matches(example) {
                return Ok(example.to_string());
            }
        }

        // If no example matches, try to generate one from the pattern
        // This is a simple heuristic - replace wildcards with common values
        let generated = pattern.as_str()
            .replace("*", "example")
            .replace("?", "x");
        
        Ok(generated)
    }

    /// Finds overlap between two patterns by checking if there's a tool that matches both.
    fn find_pattern_overlap(
        pattern1: &Pattern,
        pattern2: &Pattern,
    ) -> PolicyResult<Option<(String, Vec<String>)>> {
        // Test with common tool names
        let test_tools = vec![
            "read_file",
            "write_file",
            "read_config",
            "read_directory",
            "write_config",
            "bash:sh",
            "bash:command",
            "bash:exec",
            "mcp_server_tool",
            "mcp_server_read",
            "delete_file",
            "update_file",
        ];

        for tool in test_tools {
            if pattern1.matches(tool) && pattern2.matches(tool) {
                // Found overlap - return example with empty args
                return Ok(Some((tool.to_string(), Vec::new())));
            }
        }

        // No overlap found
        Ok(None)
    }

    /// Checks if pattern1 is more specific than pattern2.
    ///
    /// A pattern is more specific if it has fewer wildcards or is a subset.
    pub fn is_more_specific(pattern1: &str, pattern2: &str) -> bool {
        // Count wildcards - fewer wildcards = more specific
        let wildcards1 = pattern1.chars().filter(|c| *c == '*' || *c == '?').count();
        let wildcards2 = pattern2.chars().filter(|c| *c == '*' || *c == '?').count();

        if wildcards1 < wildcards2 {
            return true;
        }

        // If same number of wildcards, check if one is a prefix of the other
        if wildcards1 == wildcards2 {
            // If pattern1 is longer and starts with pattern2 (minus wildcards), it's more specific
            if pattern1.len() > pattern2.len() && pattern2.replace("*", "").replace("?", "") == "" {
                return true;
            }
        }

        false
    }
}

/// Resolution strategy for handling conflicts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionStrategy {
    /// Keep the rule with higher priority (default behavior).
    KeepHigherPriority,
    /// Keep the rule with more specific pattern.
    KeepMoreSpecific,
    /// Remove both conflicting rules.
    RemoveBoth,
    /// Keep the first rule, remove the second.
    KeepFirst,
    /// Keep the second rule, remove the first.
    KeepSecond,
    /// Rename one of the rules to make them distinct.
    Rename,
}

/// Resolves conflicts in policy rules using a specified strategy.
pub struct ConflictResolver;

impl ConflictResolver {
    /// Resolves conflicts by applying a resolution strategy.
    ///
    /// # Arguments
    /// * `conflicts` - List of conflicts to resolve
    /// * `strategy` - Resolution strategy to apply
    /// * `rules` - Mutable reference to rules list (will be modified)
    ///
    /// # Returns
    /// Vector of rule names that were removed.
    pub fn resolve_conflicts(
        conflicts: &[PolicyConflict],
        strategy: ResolutionStrategy,
        rules: &mut Vec<PolicyRule>,
    ) -> Vec<String> {
        let mut removed_rules = HashSet::new();

        for conflict in conflicts {
            if removed_rules.contains(&conflict.rule1.name)
                || removed_rules.contains(&conflict.rule2.name)
            {
                // One of the rules was already removed in a previous conflict resolution
                continue;
            }

            let to_remove = match strategy {
                ResolutionStrategy::KeepHigherPriority => {
                    if conflict.rule1.priority > conflict.rule2.priority {
                        Some(conflict.rule2.name.clone())
                    } else if conflict.rule2.priority > conflict.rule1.priority {
                        Some(conflict.rule1.name.clone())
                    } else {
                        // Same priority - keep first (rule1)
                        Some(conflict.rule2.name.clone())
                    }
                }
                ResolutionStrategy::KeepMoreSpecific => {
                    if ConflictDetector::is_more_specific(&conflict.rule1.tool_pattern, &conflict.rule2.tool_pattern) {
                        Some(conflict.rule2.name.clone())
                    } else if ConflictDetector::is_more_specific(&conflict.rule2.tool_pattern, &conflict.rule1.tool_pattern) {
                        Some(conflict.rule1.name.clone())
                    } else {
                        // Can't determine - keep first
                        Some(conflict.rule2.name.clone())
                    }
                }
                ResolutionStrategy::RemoveBoth => {
                    removed_rules.insert(conflict.rule1.name.clone());
                    removed_rules.insert(conflict.rule2.name.clone());
                    rules.retain(|r| r.name != conflict.rule1.name && r.name != conflict.rule2.name);
                    continue;
                }
                ResolutionStrategy::KeepFirst => Some(conflict.rule2.name.clone()),
                ResolutionStrategy::KeepSecond => Some(conflict.rule1.name.clone()),
                ResolutionStrategy::Rename => {
                    // Rename rule2 to make it distinct
                    let new_name = format!("{}_conflict_resolved", conflict.rule2.name);
                    if let Some(rule) = rules.iter_mut().find(|r| r.name == conflict.rule2.name) {
                        rule.name = new_name;
                    }
                    continue;
                }
            };

            if let Some(rule_name) = to_remove {
                removed_rules.insert(rule_name.clone());
                rules.retain(|r| r.name != rule_name);
            }
        }

        removed_rules.into_iter().collect()
    }

    /// Auto-resolves conflicts using intelligent heuristics.
    ///
    /// Strategy:
    /// - Priority conflicts: Keep higher priority rule
    /// - Overlapping patterns: Keep more specific rule
    /// - Duplicate patterns: Keep first rule (user's first preference)
    /// - Conflicting actions: Keep higher priority rule
    ///
    /// # Arguments
    /// * `conflicts` - List of conflicts to resolve
    /// * `rules` - Mutable reference to rules list (will be modified)
    ///
    /// # Returns
    /// Vector of rule names that were removed.
    pub fn auto_resolve(conflicts: &[PolicyConflict], rules: &mut Vec<PolicyRule>) -> Vec<String> {
        let mut removed_rules = HashSet::new();

        for conflict in conflicts {
            if removed_rules.contains(&conflict.rule1.name)
                || removed_rules.contains(&conflict.rule2.name)
            {
                continue;
            }

            let to_remove = match conflict.conflict_type {
                ConflictType::PriorityConflict | ConflictType::ConflictingActions => {
                    // Keep higher priority
                    if conflict.rule1.priority > conflict.rule2.priority {
                        Some(conflict.rule2.name.clone())
                    } else {
                        Some(conflict.rule1.name.clone())
                    }
                }
                ConflictType::OverlappingPatterns => {
                    // Keep more specific pattern
                    if ConflictDetector::is_more_specific(&conflict.rule1.tool_pattern, &conflict.rule2.tool_pattern) {
                        Some(conflict.rule2.name.clone())
                    } else {
                        Some(conflict.rule1.name.clone())
                    }
                }
                ConflictType::DuplicatePattern => {
                    // Keep first rule (appears first in list)
                    Some(conflict.rule2.name.clone())
                }
            };

            if let Some(rule_name) = to_remove {
                removed_rules.insert(rule_name.clone());
                rules.retain(|r| r.name != rule_name);
            }
        }

        removed_rules.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PolicyAction, PolicyPriority};

    #[test]
    fn test_detect_conflicting_actions() {
        let rule1 = PolicyRule::new("allow-reads", "read_*", PolicyAction::Allow);
        let rule2 = PolicyRule::new("deny-reads", "read_*", PolicyAction::Deny);

        let conflicts = ConflictDetector::detect_conflicts(&[rule1.clone(), rule2.clone()]).unwrap();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].conflict_type, ConflictType::DuplicatePattern);
        assert_eq!(conflicts[0].rule1.name, "allow-reads");
        assert_eq!(conflicts[0].rule2.name, "deny-reads");
    }

    #[test]
    fn test_detect_priority_conflict() {
        let rule1 = PolicyRule::new("allow-bash", "bash:*", PolicyAction::Allow)
            .with_priority(PolicyPriority::Admin);
        let rule2 = PolicyRule::new("deny-bash", "bash:*", PolicyAction::Deny)
            .with_priority(PolicyPriority::User);

        let conflicts = ConflictDetector::detect_conflicts(&[rule1.clone(), rule2.clone()]).unwrap();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].conflict_type, ConflictType::PriorityConflict);
    }

    #[test]
    fn test_detect_overlapping_patterns() {
        let rule1 = PolicyRule::new("allow-all-reads", "read_*", PolicyAction::Allow);
        let rule2 = PolicyRule::new("deny-read-file", "read_file", PolicyAction::Deny);

        let conflicts = ConflictDetector::detect_conflicts(&[rule1.clone(), rule2.clone()]).unwrap();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].conflict_type, ConflictType::OverlappingPatterns);
    }

    #[test]
    fn test_resolve_keep_higher_priority() {
        let rule1 = PolicyRule::new("admin-allow", "bash:*", PolicyAction::Allow)
            .with_priority(PolicyPriority::Admin);
        let rule2 = PolicyRule::new("user-deny", "bash:*", PolicyAction::Deny)
            .with_priority(PolicyPriority::User);

        let conflicts = ConflictDetector::detect_conflicts(&[rule1.clone(), rule2.clone()]).unwrap();
        let mut rules = vec![rule1.clone(), rule2.clone()];
        
        let removed = ConflictResolver::resolve_conflicts(
            &conflicts,
            ResolutionStrategy::KeepHigherPriority,
            &mut rules,
        );

        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0], "user-deny");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].name, "admin-allow");
    }

    #[test]
    fn test_auto_resolve() {
        let rule1 = PolicyRule::new("allow-all-reads", "read_*", PolicyAction::Allow)
            .with_priority(PolicyPriority::User);
        let rule2 = PolicyRule::new("deny-read-file", "read_file", PolicyAction::Deny)
            .with_priority(PolicyPriority::User);

        let conflicts = ConflictDetector::detect_conflicts(&[rule1.clone(), rule2.clone()]).unwrap();
        let mut rules = vec![rule1.clone(), rule2.clone()];
        
        let removed = ConflictResolver::auto_resolve(&conflicts, &mut rules);

        // Should keep more specific pattern (read_file over read_*)
        assert_eq!(removed.len(), 1);
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].name, "deny-read-file"); // More specific pattern is kept
    }
}

